use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT, ACCEPT};
use serde::Serialize;
use std::sync::OnceLock;

static HTTP_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

fn http_client() -> &'static reqwest::Client {
    HTTP_CLIENT.get_or_init(|| {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("ViClip"));
        headers.insert(ACCEPT, HeaderValue::from_static("*/*"));
        reqwest::Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client")
    })
}

#[derive(Serialize, Clone)]
pub struct UpdateInfo {
    pub has_update: bool,
    pub current_version: String,
    pub latest_version: String,
    pub download_url: String,
    pub body: String,
}

fn parse_semver(v: &str) -> (u32, u32, u32) {
    let parts: Vec<u32> = v
        .trim_start_matches('v')
        .split('.')
        .filter_map(|s| s.parse().ok())
        .collect();
    (
        parts.first().copied().unwrap_or(0),
        parts.get(1).copied().unwrap_or(0),
        parts.get(2).copied().unwrap_or(0),
    )
}

fn version_greater(a: &str, b: &str) -> bool {
    let (a0, a1, a2) = parse_semver(a);
    let (b0, b1, b2) = parse_semver(b);
    (a0, a1, a2) > (b0, b1, b2)
}

#[tauri::command]
pub async fn check_update() -> Result<UpdateInfo, String> {
    let current = env!("CARGO_PKG_VERSION");

    let response = http_client()
        .get("https://api.github.com/repos/wwnetboy/ViClip/releases/latest")
        .header(ACCEPT, HeaderValue::from_static("application/vnd.github+json"))
        .send()
        .await
        .map_err(|e| format!("Failed to check updates: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("GitHub API returned {}", response.status()));
    }

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    let tag_name = json["tag_name"].as_str().unwrap_or("");
    let latest = tag_name.trim_start_matches('v');

    let has_update = version_greater(latest, current);

    let download_url = json["assets"]
        .as_array()
        .and_then(|assets| {
            assets.iter().find_map(|asset| {
                let name = asset["name"].as_str().unwrap_or("");
                let url = asset["browser_download_url"].as_str().unwrap_or("");
                let lname = name.to_lowercase();
                if lname.ends_with(".msi") || lname.ends_with(".exe") {
                    Some(url.to_string())
                } else {
                    None
                }
            })
        })
        .unwrap_or_default();

    Ok(UpdateInfo {
        has_update,
        current_version: current.to_string(),
        latest_version: latest.to_string(),
        download_url,
        body: json["body"].as_str().unwrap_or("").to_string(),
    })
}

#[tauri::command]
pub async fn download_and_install_update(url: String) -> Result<(), String> {
    let response = http_client()
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Download failed: {}", e))?;

    let total = response.content_length().unwrap_or(0);

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Download failed: {}", e))?;

    if total > 0 && bytes.len() as u64 != total {
        return Err("Download incomplete".to_string());
    }

    let temp_dir = std::env::temp_dir().join("ViClip").join("update");
    std::fs::create_dir_all(&temp_dir)
        .map_err(|e| format!("Failed to create temp dir: {}", e))?;

    let filename = url
        .split('/')
        .last()
        .unwrap_or("ViClip_Setup.exe");
    let filepath = temp_dir.join(filename);

    std::fs::write(&filepath, &bytes)
        .map_err(|e| format!("Failed to save installer: {}", e))?;

    // Launch the installer; the process will replace the running app
    open::that(&filepath)
        .map_err(|e| format!("Failed to launch installer: {}", e))?;

    Ok(())
}
