use serde::{Deserialize, Serialize};
use std::sync::{Mutex, OnceLock};
use tauri::Manager;

fn http_client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("failed to create HTTP client")
    })
}

/// Cached proxy HTTP client keyed by proxy URL string.
/// Cleared when proxy setting changes so the next request rebuilds.
static PROXY_CLIENT: OnceLock<Mutex<Option<(String, reqwest::Client)>>> = OnceLock::new();

fn proxy_client_cache() -> &'static Mutex<Option<(String, reqwest::Client)>> {
    PROXY_CLIENT.get_or_init(|| Mutex::new(None))
}

fn get_proxy_client(proxy_url: &str) -> Result<reqwest::Client, String> {
    if proxy_url.is_empty() {
        return Ok(http_client().clone());
    }
    let mut cache = proxy_client_cache().lock().map_err(|e| e.to_string())?;
    if let Some((url, client)) = cache.as_ref() {
        if url == proxy_url {
            return Ok(client.clone());
        }
    }
    let proxy = reqwest::Proxy::all(proxy_url)
        .map_err(|e| format!("Invalid proxy config ({}): {}", proxy_url, e))?;
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .proxy(proxy)
        .build()
        .map_err(|e| format!("Failed to create proxy HTTP client: {}", e))?;
    *cache = Some((proxy_url.to_string(), client.clone()));
    Ok(client)
}

/// Invalidate the cached proxy client — call when proxy setting changes.
pub fn invalidate_proxy_client() {
    if let Some(cache) = PROXY_CLIENT.get() {
        if let Ok(mut c) = cache.lock() {
            *c = None;
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TranslateResponse {
    pub source_text: String,
    pub target_text: String,
    pub engine: String,
}

#[tauri::command]
pub async fn translate(
    app: tauri::AppHandle,
    text: String,
    target_lang: String,
) -> Result<TranslateResponse, String> {
    let source_lang = "auto".to_string();

    let state = app.state::<crate::db::DbState>();

    // Read engine setting from cache, check translation history in DB
    let engine = crate::db::get_setting_sync(&app, "default_translate_engine")
        .unwrap_or_else(|| "google".to_string());

    {
        let conn = state.conn.lock().map_err(|e| e.to_string())?;
        let cached: Option<String> = conn
            .query_row(
                "SELECT target_text FROM translation_history WHERE source_text = ?1 AND target_lang = ?2 AND engine = ?3 ORDER BY created_at DESC LIMIT 1",
                rusqlite::params![text, target_lang, engine],
                |row| row.get(0),
            )
            .ok();
        if let Some(cached_text) = cached {
            return Ok(TranslateResponse {
                source_text: text,
                target_text: cached_text,
                engine,
            });
        }
    }

    let result = if engine == "ai" {
        translate_ai(&app, &text, &source_lang, &target_lang).await?
    } else {
        translate_google(&app, &text, &source_lang, &target_lang).await?
    };

    // Save to history/cache
    {
        let conn = state.conn.lock().map_err(|e| e.to_string())?;
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO translation_history (id, source_text, target_text, source_lang, target_lang, engine, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![id, text, result.target_text, source_lang, target_lang, engine, &now],
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(result)
}

async fn translate_ai(
    app: &tauri::AppHandle,
    text: &str,
    source_lang: &str,
    target_lang: &str,
) -> Result<TranslateResponse, String> {
    let api_url = crate::db::get_setting_sync(app, "ai_api_url").unwrap_or_default();
    let api_key = crate::db::get_setting_sync(app, "ai_api_key").unwrap_or_default();
    let model = crate::db::get_setting_sync(app, "ai_model").unwrap_or_else(|| "gpt-4o-mini".to_string());

    if api_url.is_empty() || api_key.is_empty() {
        return Err("AI translation not configured. Please fill in the API URL and Key in settings".to_string());
    }

    let full_url = if api_url.contains("/chat/completions") || api_url.contains("/completions") {
        api_url.clone()
    } else {
        let base = api_url.trim_end_matches('/');
        format!("{}/v1/chat/completions", base)
    };

    let prompt = format!(
        "Translate the following text from {source} to {target}. Only output the translated text, nothing else.\n\nText: {text}",
        source = if source_lang == "auto" { "auto-detected language" } else { source_lang },
        target = target_lang,
        text = text
    );

    let resp = http_client()
        .post(&full_url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "model": model,
            "messages": [
                {"role": "system", "content": "You are a professional translator. Only output the translated text."},
                {"role": "user", "content": prompt}
            ],
            "temperature": 0.3
        }))
        .send().await.map_err(|e| format!("AI translation request failed: {}", e))?;

    let status = resp.status();
    let body_text = resp.text().await.map_err(|e| format!("Failed to read response: {}", e))?;

    if !status.is_success() {
        return Err(format!("AI translation HTTP {}: {}", status.as_u16(), &body_text[..body_text.len().min(80)]));
    }

    let json: serde_json::Value = serde_json::from_str(&body_text)
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    let translated = json["choices"][0]["message"]["content"]
        .as_str()
        .ok_or("AI response format error: choices[0].message.content not found")?
        .trim()
        .to_string();

    Ok(TranslateResponse {
        source_text: text.to_string(),
        target_text: translated,
        engine: "ai".to_string(),
    })
}

async fn translate_google(
    app: &tauri::AppHandle,
    text: &str,
    _source_lang: &str,
    target_lang: &str,
) -> Result<TranslateResponse, String> {
    let api_key = crate::db::get_setting_sync(app, "google_api_key").unwrap_or_default();
    let proxy_url = crate::db::get_setting_sync(app, "translate_proxy").unwrap_or_default();

    let client = get_proxy_client(&proxy_url)?;

    if api_key.is_empty() {
        let resp = client
            .get("https://translate.googleapis.com/translate_a/single")
            .query(&[
                ("client", "gtx"),
                ("sl", "auto"),
                ("tl", target_lang),
                ("dt", "t"),
                ("q", text),
            ])
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")
            .send().await.map_err(|e| fmt_reqwest_error(&e))?;

        let status = resp.status();
        let body = resp.text().await.map_err(|e| format!("Failed to read Google response: {}", e))?;

        if !status.is_success() {
            return Err(format!(
                "Google Translate service returned HTTP {}. The free API may be experiencing issues. Please retry later, configure a proxy, or switch to AI translation in Settings.",
                status.as_u16()
            ));
        }

        let json: serde_json::Value = serde_json::from_str(&body)
            .map_err(|e| format!("Failed to parse Google response: {}", e))?;

        let translated = json[0][0][0]
            .as_str()
            .ok_or("Google Translate free API returned an unexpected response. It may be temporarily unstable. Please retry or switch to AI translation in Settings.")?
            .to_string();

        return Ok(TranslateResponse {
            source_text: text.to_string(),
            target_text: translated,
            engine: "google".to_string(),
        });
    }

    let resp = client
        .post("https://translation.googleapis.com/language/translate/v2")
        .query(&[("key", api_key.as_str())])
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")
        .json(&serde_json::json!({
            "q": text,
            "target": target_lang,
            "format": "text"
        }))
        .send().await.map_err(|e| fmt_reqwest_error(&e))?;

    let json: serde_json::Value = resp.json().await
        .map_err(|e| format!("Failed to parse Google response: {}", e))?;

    if let Some(error) = json.get("error") {
        let msg = error["message"].as_str().unwrap_or("Unknown error");
        return Err(format!("Google translate error: {}", &msg[..msg.len().min(80)]));
    }

    let translated = json["data"]["translations"][0]["translatedText"]
        .as_str()
        .ok_or("Google Cloud Translation returned an unexpected response. Please check your API key and try again.")?
        .to_string();

    Ok(TranslateResponse {
        source_text: text.to_string(),
        target_text: translated,
        engine: "google".to_string(),
    })
}

fn fmt_reqwest_error(err: &reqwest::Error) -> String {
    if err.is_connect() {
        "Unable to connect to Google Translate. Please check your network, configure a proxy in Settings, or switch to AI translation.".to_string()
    } else if err.is_timeout() {
        "Google Translate request timed out. The free API may be unstable. Please retry, or switch to AI translation in Settings.".to_string()
    } else {
        format!("Google Translate request failed: {}. The free API may be unstable. Consider using an API key or switching to AI translation in Settings.", err)
    }
}


