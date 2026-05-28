use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
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
    pub detected_lang: Option<String>,
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
                detected_lang: None,
            });
        }
    }

    let result = match engine.as_str() {
        "ai" => translate_ai(&app, &text, &source_lang, &target_lang).await?,
        "baidu" => translate_baidu(&app, &text, &target_lang).await?,
        "youdao" => translate_youdao(&app, &text, &target_lang).await?,
        "tencent" => translate_tencent(&app, &text, &target_lang).await?,
        "volctrans" => translate_volctrans(&app, &text, &target_lang).await?,
        _ => translate_google(&app, &text, &source_lang, &target_lang).await?,
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
        detected_lang: None,
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

        let detected = json.get(2).and_then(|v| v.as_str()).map(|s| s.to_string());

        return Ok(TranslateResponse {
            source_text: text.to_string(),
            target_text: translated,
            engine: "google".to_string(),
            detected_lang: detected,
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

    let detected = json["data"]["translations"][0]["detectedSourceLanguage"]
        .as_str()
        .map(|s| s.to_string());

    Ok(TranslateResponse {
        source_text: text.to_string(),
        target_text: translated,
        engine: "google".to_string(),
        detected_lang: detected,
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

// ── HMAC-SHA256 helper ──

type HmacSha256 = Hmac<Sha256>;

fn hmac_sha256(key: &[u8], msg: &[u8]) -> Vec<u8> {
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC key size");
    mac.update(msg);
    mac.finalize().into_bytes().to_vec()
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

// ── Language code mapping ──

fn to_baidu_lang(code: &str) -> &str {
    match code {
        "zh-TW" => "cht",
        "ja" => "jp",
        "ko" => "kor",
        "fr" => "fra",
        "es" => "spa",
        "ar" => "ara",
        "vi" => "vie",
        "ms" => "may",
        "sv" => "swe",
        "da" => "dan",
        "fi" => "fin",
        "ro" => "rom",
        "he" => "heb",
        "uk" => "ukr",
        "no" => "nor",
        "bg" => "bul",
        "hr" => "hrv",
        "sr" => "srp",
        "tl" => "fil",
        _ => code,
    }
}

fn to_youdao_lang(code: &str) -> &str {
    match code {
        "zh" => "zh-CHS",
        "zh-TW" => "zh-CHT",
        _ => code,
    }
}

fn to_volctrans_lang(code: &str) -> &str {
    match code {
        "zh-TW" => "zh-Hant",
        _ => code,
    }
}

// ── Random salt ──

fn random_salt() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .to_string()
}

// ═══════════════════════ BAIDU ═══════════════════════

async fn translate_baidu(
    app: &tauri::AppHandle,
    text: &str,
    target_lang: &str,
) -> Result<TranslateResponse, String> {
    let appid = crate::db::get_setting_sync(app, "baidu_appid").unwrap_or_default();
    let secret_key = crate::db::get_setting_sync(app, "baidu_secret_key").unwrap_or_default();

    if appid.is_empty() || secret_key.is_empty() {
        return Err("Baidu translation not configured. Please fill in the App ID and Secret Key in Settings.".to_string());
    }

    let salt = random_salt();
    let sign_str = format!("{}{}{}{}", appid, text, salt, secret_key);
    let sign = format!("{:x}", md5::Md5::digest(sign_str.as_bytes()));

    let resp = http_client()
        .get("https://fanyi-api.baidu.com/api/trans/vip/translate")
        .query(&[
            ("q", text),
            ("from", "auto"),
            ("to", to_baidu_lang(target_lang)),
            ("appid", appid.as_str()),
            ("salt", salt.as_str()),
            ("sign", sign.as_str()),
        ])
        .send()
        .await
        .map_err(|e| format!("Baidu translation request failed: {}", e))?;

    let status = resp.status();
    let body = resp.text().await.map_err(|e| format!("Failed to read Baidu response: {}", e))?;

    if !status.is_success() {
        return Err(format!("Baidu translation HTTP {}: {}", status.as_u16(), &body[..body.len().min(80)]));
    }

    let json: serde_json::Value =
        serde_json::from_str(&body).map_err(|e| format!("Failed to parse Baidu response: {}", e))?;

    if let Some(err_code) = json.get("error_code") {
        let err_msg = json["error_msg"].as_str().unwrap_or("Unknown error");
        return Err(format!("Baidu translation error {}: {}", err_code, err_msg));
    }

    let translated = json["trans_result"][0]["dst"]
        .as_str()
        .ok_or("Baidu translation returned an unexpected response.")?
        .to_string();

    let detected = json["from"].as_str().map(|s| s.to_string());

    Ok(TranslateResponse {
        source_text: text.to_string(),
        target_text: translated,
        engine: "baidu".to_string(),
        detected_lang: detected,
    })
}

// ═══════════════════════ YOUDAO ═══════════════════════

async fn translate_youdao(
    app: &tauri::AppHandle,
    text: &str,
    target_lang: &str,
) -> Result<TranslateResponse, String> {
    let app_key = crate::db::get_setting_sync(app, "youdao_app_key").unwrap_or_default();
    let app_secret = crate::db::get_setting_sync(app, "youdao_app_secret").unwrap_or_default();

    if app_key.is_empty() || app_secret.is_empty() {
        return Err("Youdao translation not configured. Please fill in the App Key and App Secret in Settings.".to_string());
    }

    let salt = random_salt();
    let curtime = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .to_string();

    // Youdao v3 sign: SHA256(appKey + truncate(q) + salt + curtime + appSecret)
    // Truncation: if text > 20 chars → first10 + totalLen + last10, else full text
    let sign_input = if text.chars().count() > 20 {
        let total = text.chars().count();
        let first10: String = text.chars().take(10).collect();
        let last10: String = text.chars().skip(total - 10).collect();
        format!("{}{}{}", first10, total, last10)
    } else {
        text.to_string()
    };
    let sign_str = format!("{}{}{}{}{}", app_key, sign_input, salt, curtime, app_secret);
    let sign = format!("{:x}", Sha256::digest(sign_str.as_bytes()));

    let from_lang = if text.chars().any(|c| c > '\u{007f}') { "zh-CHS" } else { "en" };
    // If user selected zh, use zh-CHS for youdao
    let to_lang = to_youdao_lang(target_lang);

    let resp = http_client()
        .post("https://openapi.youdao.com/api")
        .form(&[
            ("q", text),
            ("from", from_lang),
            ("to", to_lang),
            ("appKey", app_key.as_str()),
            ("salt", salt.as_str()),
            ("sign", sign.as_str()),
            ("signType", "v3"),
            ("curtime", curtime.as_str()),
        ])
        .send()
        .await
        .map_err(|e| format!("Youdao translation request failed: {}", e))?;

    let body = resp.text().await.map_err(|e| format!("Failed to read Youdao response: {}", e))?;

    let json: serde_json::Value =
        serde_json::from_str(&body).map_err(|e| format!("Failed to parse Youdao response: {}", e))?;

    let error_code = json["errorCode"].as_str().unwrap_or("");
    if error_code != "0" {
        return Err(format!("Youdao translation error {}: {}", error_code, &body[..body.len().min(80)]));
    }

    let translated = json["translation"][0]
        .as_str()
        .ok_or("Youdao translation returned an unexpected response.")?
        .to_string();

    Ok(TranslateResponse {
        source_text: text.to_string(),
        target_text: translated,
        engine: "youdao".to_string(),
        detected_lang: None,
    })
}

// ═══════════════════════ TENCENT ═══════════════════════

fn sign_tc3(secret_id: &str, secret_key: &str, payload: &str, timestamp: i64) -> String {
    let date = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let service = "tmt";
    let host = "tmt.tencentcloudapi.com";

    let canonical_headers = format!("content-type:application/json\nhost:{}\n", host);
    let signed_headers = "content-type;host";
    let hashed_payload = format!("{:x}", Sha256::digest(payload.as_bytes()));

    let canonical_request = format!(
        "POST\n/\n\n{}\n{}\n{}",
        canonical_headers, signed_headers, hashed_payload
    );
    let hashed_canonical_request = format!("{:x}", Sha256::digest(canonical_request.as_bytes()));

    let algorithm = "TC3-HMAC-SHA256";
    let credential_scope = format!("{}/{}/tc3_request", date, service);
    let string_to_sign = format!(
        "{}\n{}\n{}\n{}",
        algorithm, timestamp, credential_scope, hashed_canonical_request
    );

    let secret_date = hmac_sha256(format!("TC3{}", secret_key).as_bytes(), date.as_bytes());
    let secret_service = hmac_sha256(&secret_date, service.as_bytes());
    let secret_signing = hmac_sha256(&secret_service, b"tc3_request");
    let signature = hex(&hmac_sha256(&secret_signing, string_to_sign.as_bytes()));

    format!(
        "TC3-HMAC-SHA256 Credential={}/{}, SignedHeaders={}, Signature={}",
        secret_id, credential_scope, signed_headers, signature
    )
}

async fn translate_tencent(
    app: &tauri::AppHandle,
    text: &str,
    target_lang: &str,
) -> Result<TranslateResponse, String> {
    let secret_id = crate::db::get_setting_sync(app, "tencent_secret_id").unwrap_or_default();
    let secret_key = crate::db::get_setting_sync(app, "tencent_secret_key").unwrap_or_default();

    if secret_id.is_empty() || secret_key.is_empty() {
        return Err("Tencent Cloud translation not configured. Please fill in the Secret ID and Secret Key in Settings.".to_string());
    }

    let payload = serde_json::json!({
        "SourceText": text,
        "Source": "auto",
        "Target": target_lang,
        "ProjectId": 0,
    })
    .to_string();

    let timestamp = chrono::Utc::now().timestamp();
    let authorization = sign_tc3(&secret_id, &secret_key, &payload, timestamp);

    let resp = http_client()
        .post("https://tmt.tencentcloudapi.com")
        .header("Authorization", &authorization)
        .header("Content-Type", "application/json")
        .header("Host", "tmt.tencentcloudapi.com")
        .header("X-TC-Action", "TextTranslate")
        .header("X-TC-Version", "2018-03-21")
        .header("X-TC-Timestamp", timestamp.to_string())
        .header("X-TC-Region", "ap-guangzhou")
        .body(payload)
        .send()
        .await
        .map_err(|e| format!("Tencent translation request failed: {}", e))?;

    let body = resp.text().await.map_err(|e| format!("Failed to read Tencent response: {}", e))?;

    let json: serde_json::Value =
        serde_json::from_str(&body).map_err(|e| format!("Failed to parse Tencent response: {}", e))?;

    if let Some(err) = json["Response"]["Error"].as_object() {
        return Err(format!(
            "Tencent translation error [{}]: {}",
            err["Code"].as_str().unwrap_or(""),
            err["Message"].as_str().unwrap_or("Unknown error")
        ));
    }

    let translated = json["Response"]["TargetText"]
        .as_str()
        .ok_or("Tencent translation returned an unexpected response.")?
        .to_string();

    let detected = json["Response"]["Source"].as_str().map(|s| s.to_string());

    Ok(TranslateResponse {
        source_text: text.to_string(),
        target_text: translated,
        engine: "tencent".to_string(),
        detected_lang: detected,
    })
}

// ═══════════════════════ VOLCTRANS ═══════════════════════

fn sign_volctrans(access_key_id: &str, secret_access_key: &str, payload: &str) -> String {
    let now = chrono::Utc::now();
    let date = now.format("%Y%m%d").to_string();
    let timestamp = now.format("%Y%m%dT%H%M%SZ").to_string();
    let region = "cn-north-1";
    let service = "translate";

    // Hashed payload (lowercase hex)
    let hashed_payload = format!("{:x}", Sha256::digest(payload.as_bytes()));

    // Canonical request
    let canonical_uri = "/";
    let canonical_querystring = "Action=TranslateText&Version=2020-06-01";
    let canonical_headers = format!(
        "content-type:application/json\nhost:translate.volcengineapi.com\nx-date:{}\n",
        timestamp
    );
    let signed_headers = "content-type;host;x-date";

    let canonical_request = format!(
        "POST\n{}\n{}\n{}\n{}\n{}",
        canonical_uri, canonical_querystring, canonical_headers, signed_headers, hashed_payload
    );
    let hashed_canonical_request = format!("{:x}", Sha256::digest(canonical_request.as_bytes()));

    // String to sign
    let algorithm = "HMAC-SHA256";
    let credential_scope = format!("{}/{}/{}/request", date, region, service);
    let string_to_sign = format!(
        "{}\n{}\n{}\n{}",
        algorithm, timestamp, credential_scope, hashed_canonical_request
    );

    // Derive signing key
    let k_date = hmac_sha256(secret_access_key.as_bytes(), date.as_bytes());
    let k_region = hmac_sha256(&k_date, region.as_bytes());
    let k_service = hmac_sha256(&k_region, service.as_bytes());
    let k_signing = hmac_sha256(&k_service, b"request");

    let signature = hex(&hmac_sha256(&k_signing, string_to_sign.as_bytes()));

    format!(
        "HMAC-SHA256 Credential={}/{}, SignedHeaders={}, Signature={}",
        access_key_id, credential_scope, signed_headers, signature
    )
}

async fn translate_volctrans(
    app: &tauri::AppHandle,
    text: &str,
    target_lang: &str,
) -> Result<TranslateResponse, String> {
    let access_key_id = crate::db::get_setting_sync(app, "volctrans_access_key_id").unwrap_or_default();
    let secret_access_key = crate::db::get_setting_sync(app, "volctrans_secret_access_key").unwrap_or_default();

    if access_key_id.is_empty() || secret_access_key.is_empty() {
        return Err("Volctrans translation not configured. Please fill in the Access Key ID and Secret Access Key in Settings.".to_string());
    }

    let payload = serde_json::json!({
        "TargetLanguage": to_volctrans_lang(target_lang),
        "TextList": [text],
    })
    .to_string();

    let authorization = sign_volctrans(&access_key_id, &secret_access_key, &payload);
    let now = chrono::Utc::now();
    let x_date = now.format("%Y%m%dT%H%M%SZ").to_string();

    let resp = http_client()
        .post("https://translate.volcengineapi.com?Action=TranslateText&Version=2020-06-01")
        .header("Authorization", &authorization)
        .header("Content-Type", "application/json")
        .header("Host", "translate.volcengineapi.com")
        .header("X-Date", &x_date)
        .body(payload)
        .send()
        .await
        .map_err(|e| format!("Volctrans request failed: {}", e))?;

    let body = resp.text().await.map_err(|e| format!("Failed to read Volctrans response: {}", e))?;

    let json: serde_json::Value =
        serde_json::from_str(&body).map_err(|e| format!("Failed to parse Volctrans response: {}", e))?;

    if let Some(err) = json["ResponseMetadata"]["Error"].as_object() {
        return Err(format!(
            "Volctrans error [{}]: {}",
            err["Code"].as_str().unwrap_or(""),
            err["Message"].as_str().unwrap_or("Unknown error")
        ));
    }

    let translated = json["TranslationList"][0]["Translation"]
        .as_str()
        .ok_or("Volctrans returned an unexpected response.")?
        .to_string();

    let detected = json["TranslationList"][0]["DetectedSourceLanguage"]
        .as_str()
        .map(|s| s.to_string());

    Ok(TranslateResponse {
        source_text: text.to_string(),
        target_text: translated,
        engine: "volctrans".to_string(),
        detected_lang: detected,
    })
}


