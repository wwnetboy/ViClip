use rusqlite::{Connection, params};
use tauri::{AppHandle, Emitter, Manager};
use std::path::PathBuf;
use std::sync::Mutex;
use std::collections::{HashMap, HashSet};

pub struct DbState {
    pub conn: Mutex<Connection>,
}

// In-memory settings cache: loaded once from DB, updated on every write.
// Reduces DB lock contention for high-frequency reads (translator, paste, etc.).
static SETTINGS_CACHE: std::sync::OnceLock<Mutex<HashMap<String, String>>> = std::sync::OnceLock::new();

fn settings_cache() -> &'static Mutex<HashMap<String, String>> {
    SETTINGS_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Populate cache from DB. Called once at startup.
pub fn warm_settings_cache(app: &AppHandle) {
    if let Some(state) = app.try_state::<DbState>() {
        if let Ok(conn) = state.conn.lock() {
            if let Ok(mut stmt) = conn.prepare("SELECT key, value FROM settings") {
                if let Ok(rows) = stmt.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))) {
                    let mut cache = settings_cache().lock().unwrap();
                    for row in rows.flatten() {
                        cache.insert(row.0, row.1);
                    }
                }
            }
        }
    }
}

fn update_cached_setting(key: &str, value: &str) {
    if let Ok(mut cache) = settings_cache().lock() {
        cache.insert(key.to_string(), value.to_string());
    }
}

/// Read a setting from cache (falls back to DB on cache miss).
fn cached_setting(app: &AppHandle, key: &str) -> String {
    if let Ok(cache) = settings_cache().lock() {
        if let Some(v) = cache.get(key) {
            return v.clone();
        }
    }
    // Cache miss — read from DB and populate cache
    let value = get_setting_from_db(app, key);
    if !value.is_empty() {
        update_cached_setting(key, &value);
    }
    value
}

fn get_setting_from_db(app: &AppHandle, key: &str) -> String {
    if let Some(state) = app.try_state::<DbState>() {
        if let Ok(conn) = state.conn.lock() {
            return conn.query_row(
                "SELECT value FROM settings WHERE key = ?1",
                params![key],
                |row| row.get(0),
            ).unwrap_or_default();
        }
    }
    String::new()
}

const SCHEMA_SQL: &str = "
    CREATE TABLE IF NOT EXISTS clipboard_records (
        id TEXT PRIMARY KEY,
        type TEXT NOT NULL,
        content TEXT NOT NULL,
        source_app TEXT DEFAULT '',
        created_at TEXT NOT NULL
    );

    CREATE INDEX IF NOT EXISTS idx_clipboard_created_at
        ON clipboard_records(created_at);

    CREATE INDEX IF NOT EXISTS idx_clipboard_type
        ON clipboard_records(type);

    CREATE TABLE IF NOT EXISTS phrase_groups (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        sort_order INTEGER DEFAULT 0,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL
    );

    CREATE TABLE IF NOT EXISTS phrases (
        id TEXT PRIMARY KEY,
        group_id TEXT NOT NULL,
        title TEXT NOT NULL,
        content TEXT NOT NULL,
        sort_order INTEGER DEFAULT 0,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL,
        FOREIGN KEY (group_id) REFERENCES phrase_groups(id) ON DELETE CASCADE
    );

    CREATE TABLE IF NOT EXISTS translation_history (
        id TEXT PRIMARY KEY,
        source_text TEXT NOT NULL,
        target_text TEXT NOT NULL,
        source_lang TEXT DEFAULT 'auto',
        target_lang TEXT NOT NULL,
        engine TEXT NOT NULL,
        created_at TEXT NOT NULL
    );

    CREATE INDEX IF NOT EXISTS idx_translation_created_at
        ON translation_history(created_at);

    CREATE TABLE IF NOT EXISTS settings (
        key TEXT PRIMARY KEY,
        value TEXT NOT NULL
    );
";

fn default_data_dir() -> PathBuf {
    let exe = std::env::current_exe().expect("failed to get exe path");
    exe.parent()
        .unwrap_or_else(|| std::path::Path::new("."))
        .join("data")
}

fn db_path() -> PathBuf {
    let default_dir = default_data_dir();
    let default_db = default_dir.join("data.db");
    std::fs::create_dir_all(&default_dir).ok();

    if !default_db.exists() {
        return default_db;
    }

    let mut current = default_db;
    let mut visited: HashSet<PathBuf> = HashSet::new();

    loop {
        let conn = match Connection::open(&current) {
            Ok(c) => c,
            Err(_) => break,
        };

        let path: String = match conn.query_row(
            "SELECT value FROM settings WHERE key = 'storage_path'",
            [],
            |row| row.get::<_, String>(0),
        ) {
            Ok(p) if !p.is_empty() => p,
            _ => break,
        };

        let custom_dir = PathBuf::from(&path);
        let custom_db = custom_dir.join("data.db");

        if custom_db == current || !visited.insert(custom_db.clone()) {
            break;
        }

        if !custom_db.exists() {
            break;
        }

        current = custom_db;
    }

    current
}

static STORAGE_DIR_CACHE: std::sync::OnceLock<Mutex<Option<PathBuf>>> = std::sync::OnceLock::new();

pub fn invalidate_storage_dir_cache() {
    if let Some(cache) = STORAGE_DIR_CACHE.get() {
        if let Ok(mut c) = cache.lock() {
            *c = None;
        }
    }
}

pub fn get_storage_dir(app: &AppHandle) -> PathBuf {
    if let Some(cache) = STORAGE_DIR_CACHE.get() {
        if let Ok(c) = cache.lock() {
            if let Some(ref path) = *c {
                return path.clone();
            }
        }
    }

    let resolved = if let Some(custom) = get_setting_sync(app, "storage_path") {
        if !custom.is_empty() {
            let custom_dir = PathBuf::from(&custom);
            if custom_dir.exists() || std::fs::create_dir_all(&custom_dir).is_ok() {
                custom_dir
            } else {
                default_data_dir()
            }
        } else {
            default_data_dir()
        }
    } else {
        default_data_dir()
    };

    let cache = STORAGE_DIR_CACHE.get_or_init(|| Mutex::new(None));
    if let Ok(mut c) = cache.lock() {
        *c = Some(resolved.clone());
    }
    resolved
}

pub fn init_db(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let path = db_path();
    let conn = Connection::open(&path)?;

    conn.execute_batch(
        "PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL; PRAGMA cache_size=-8000;",
    )?;

    conn.execute_batch(SCHEMA_SQL)?;

    conn.execute_batch("
        INSERT OR IGNORE INTO settings (key, value) VALUES ('clipboard_retention', '1month');
        INSERT OR IGNORE INTO settings (key, value) VALUES ('default_translate_engine', 'google');
        INSERT OR IGNORE INTO settings (key, value) VALUES ('theme', 'light');
        INSERT OR IGNORE INTO settings (key, value) VALUES ('language', 'zh-CN');
        INSERT OR IGNORE INTO settings (key, value) VALUES ('google_api_key', '');
        INSERT OR IGNORE INTO settings (key, value) VALUES ('translate_proxy', '');
        INSERT OR IGNORE INTO settings (key, value) VALUES ('radial_menu_enabled', '1');
        INSERT OR IGNORE INTO settings (key, value) VALUES ('autostart', '1');
        INSERT OR IGNORE INTO settings (key, value) VALUES ('minimize_to_tray', '0');
        INSERT OR IGNORE INTO settings (key, value) VALUES ('shortcut_key', 'Alt+V');

        UPDATE settings SET value = 'google' WHERE key = 'default_translate_engine' AND value = 'builtin';
        ",
    )?;

    app.manage(DbState {
        conn: Mutex::new(conn),
    });

    warm_settings_cache(app);

    Ok(())
}

pub fn prune_old_records(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let days;
    let image_contents: Vec<String>;

    {
        let state = app.state::<DbState>();
        let conn = state.conn.lock().map_err(|e| e.to_string())?;

        let retention: String = conn
            .query_row(
                "SELECT value FROM settings WHERE key = 'clipboard_retention'",
                [],
                |row| row.get(0),
            )
            .unwrap_or_else(|_| "1month".to_string());

        days = match retention.as_str() {
            "1week" => 7,
            "1month" => 30,
            "3months" => 90,
            "6months" => 180,
            "1year" => 365,
            "forever" => return Ok(()),
            _ => 30,
        };

        // Collect image records before deletion for file cleanup
        {
            let mut stmt = conn.prepare(
                "SELECT content FROM clipboard_records WHERE type = 'image' AND datetime(created_at) < datetime('now', ?1)",
            )?;
            let rows = stmt.query_map(params![format!("-{} days", days)], |row| {
                row.get::<_, String>(0)
            })?;
            image_contents = rows.filter_map(|r| r.ok()).collect();
        }

        conn.execute(
            "DELETE FROM clipboard_records WHERE datetime(created_at) < datetime('now', ?1)",
            params![format!("-{} days", days)],
        )?;
    }

    // Clean up image files and thumbnails only if no remaining records reference them.
    // Content-hash filenames mean multiple records can share the same file on disk.
    let base_dir = get_storage_dir(app);
    let state = app.state::<DbState>();
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    for content in &image_contents {
        let still_referenced: bool = conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM clipboard_records WHERE content = ?1",
                params![content],
                |row| row.get(0),
            )
            .unwrap_or(false);
        if still_referenced {
            continue;
        }
        let file_path = base_dir.join(content);
        let _ = std::fs::remove_file(&file_path);
        if let Some(filename) = file_path.file_name() {
            let thumb_path = file_path.parent().unwrap_or(&base_dir).join("thumbs").join(filename);
            let _ = std::fs::remove_file(&thumb_path);
        }
    }

    Ok(())
}

// ---- Tauri Commands ----

fn map_record_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<serde_json::Value> {
    Ok(serde_json::json!({
        "id": row.get::<_, String>(0)?,
        "type": row.get::<_, String>(1)?,
        "content": row.get::<_, String>(2)?,
        "source_app": row.get::<_, String>(3)?,
        "created_at": row.get::<_, String>(4)?,
    }))
}

#[tauri::command]
pub fn get_clipboard_records(
    app: AppHandle,
    search: Option<String>,
    limit: Option<u32>,
    record_type: Option<String>,
) -> Result<Vec<serde_json::Value>, String> {
    let state = app.state::<DbState>();
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let lim = limit.unwrap_or(200);

    let mut sql = String::from(
        "SELECT id, type, content, source_app, created_at FROM clipboard_records"
    );
    let mut param_values: Vec<String> = Vec::new();

    if let Some(ref rt) = record_type {
        sql.push_str(" WHERE type = ?");
        param_values.push(rt.clone());
    }

    if let Some(ref q) = search {
        let escaped = q.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_");
        if param_values.is_empty() {
            sql.push_str(" WHERE");
        } else {
            sql.push_str(" AND");
        }
        sql.push_str(" content LIKE '%' || ? || '%' ESCAPE '\\'");
        param_values.push(escaped);
    }

    sql.push_str(" ORDER BY created_at DESC LIMIT ?");
    param_values.push(lim.to_string());

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;

    let params: Vec<&dyn rusqlite::types::ToSql> = param_values
        .iter()
        .map(|s| s as &dyn rusqlite::types::ToSql)
        .collect();

    let rows = stmt
        .query_map(params.as_slice(), map_record_row)
        .map_err(|e| e.to_string())?;

    let mut records: Vec<serde_json::Value> = Vec::new();
    for row in rows {
        records.push(row.map_err(|e| e.to_string())?);
    }
    Ok(records)
}

#[tauri::command]
pub fn delete_clipboard_record(app: AppHandle, id: String) -> Result<(), String> {
    let image_content: Option<String> = {
        let state = app.state::<DbState>();
        let conn = state.conn.lock().map_err(|e| e.to_string())?;

        let record: Option<(String, String)> = conn
            .query_row(
                "SELECT type, content FROM clipboard_records WHERE id = ?1",
                params![id],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
            )
            .ok();

        conn.execute("DELETE FROM clipboard_records WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;

        let _ = app.emit("clipboard-deleted", &id);

        match record {
            Some((t, c)) if t == "image" => Some(c),
            _ => None,
        }
    };

    if let Some(content) = image_content {
        let file_path = get_storage_dir(&app).join(&content);
        let _ = std::fs::remove_file(&file_path);
        if let Some(filename) = file_path.file_name() {
            let thumb_path = file_path.parent().unwrap_or(std::path::Path::new("."))
                .join("thumbs").join(filename);
            let _ = std::fs::remove_file(&thumb_path);
        }
    }

    Ok(())
}

#[tauri::command]
pub fn get_phrase_groups(app: AppHandle) -> Result<Vec<serde_json::Value>, String> {
    let state = app.state::<DbState>();
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id, name, sort_order, created_at, updated_at FROM phrase_groups ORDER BY sort_order")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "name": row.get::<_, String>(1)?,
                "sort_order": row.get::<_, i32>(2)?,
                "created_at": row.get::<_, String>(3)?,
                "updated_at": row.get::<_, String>(4)?,
            }))
        })
        .map_err(|e| e.to_string())?;
    let mut groups = Vec::new();
    for row in rows {
        groups.push(row.map_err(|e| e.to_string())?);
    }
    Ok(groups)
}

#[tauri::command]
pub fn create_phrase_group(app: AppHandle, name: String) -> Result<serde_json::Value, String> {
    let state = app.state::<DbState>();
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO phrase_groups (id, name, sort_order, created_at, updated_at) VALUES (?1, ?2, 0, ?3, ?4)",
        params![id, name, &now, &now],
    )
    .map_err(|e| e.to_string())?;
    let _ = app.emit("phrase-groups-changed", ());
    Ok(serde_json::json!({
        "id": id,
        "name": name,
        "sort_order": 0,
        "created_at": now,
        "updated_at": now,
    }))
}

#[tauri::command]
pub fn update_phrase_group(app: AppHandle, id: String, name: String) -> Result<(), String> {
    let state = app.state::<DbState>();
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE phrase_groups SET name = ?1, updated_at = ?2 WHERE id = ?3",
        params![name, &now, id],
    )
    .map_err(|e| e.to_string())?;
    let _ = app.emit("phrase-groups-changed", ());
    Ok(())
}

#[tauri::command]
pub fn delete_phrase_group(app: AppHandle, id: String) -> Result<(), String> {
    let state = app.state::<DbState>();
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM phrases WHERE group_id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM phrase_groups WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    let _ = app.emit("phrase-groups-changed", ());
    Ok(())
}

#[tauri::command]
pub fn get_phrases(app: AppHandle, group_id: String) -> Result<Vec<serde_json::Value>, String> {
    let state = app.state::<DbState>();
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id, group_id, title, content, sort_order, created_at, updated_at FROM phrases WHERE group_id = ?1 ORDER BY sort_order")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![group_id], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "group_id": row.get::<_, String>(1)?,
                "title": row.get::<_, String>(2)?,
                "content": row.get::<_, String>(3)?,
                "sort_order": row.get::<_, i32>(4)?,
                "created_at": row.get::<_, String>(5)?,
                "updated_at": row.get::<_, String>(6)?,
            }))
        })
        .map_err(|e| e.to_string())?;
    let mut phrases = Vec::new();
    for row in rows {
        phrases.push(row.map_err(|e| e.to_string())?);
    }
    Ok(phrases)
}

#[tauri::command]
pub fn create_phrase(
    app: AppHandle,
    group_id: String,
    title: String,
    content: String,
) -> Result<serde_json::Value, String> {
    let state = app.state::<DbState>();
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO phrases (id, group_id, title, content, sort_order, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, 0, ?5, ?6)",
        params![id, group_id, title, content, &now, &now],
    )
    .map_err(|e| e.to_string())?;
    Ok(serde_json::json!({
        "id": id,
        "group_id": group_id,
        "title": title,
        "content": content,
        "sort_order": 0,
        "created_at": now,
        "updated_at": now,
    }))
}

#[tauri::command]
pub fn update_phrase(
    app: AppHandle,
    id: String,
    title: String,
    content: String,
) -> Result<(), String> {
    let state = app.state::<DbState>();
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE phrases SET title = ?1, content = ?2, updated_at = ?3 WHERE id = ?4",
        params![title, content, &now, id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn delete_phrase(app: AppHandle, id: String) -> Result<(), String> {
    let state = app.state::<DbState>();
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM phrases WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_translation_history(
    app: AppHandle,
    limit: Option<u32>,
) -> Result<Vec<serde_json::Value>, String> {
    let state = app.state::<DbState>();
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let l = limit.unwrap_or(100);
    let mut stmt = conn
        .prepare(
            "SELECT id, source_text, target_text, source_lang, target_lang, engine, created_at
             FROM translation_history ORDER BY created_at DESC LIMIT ?1",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![l], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "source_text": row.get::<_, String>(1)?,
                "target_text": row.get::<_, String>(2)?,
                "source_lang": row.get::<_, String>(3)?,
                "target_lang": row.get::<_, String>(4)?,
                "engine": row.get::<_, String>(5)?,
                "created_at": row.get::<_, String>(6)?,
            }))
        })
        .map_err(|e| e.to_string())?;
    let mut history = Vec::new();
    for row in rows {
        history.push(row.map_err(|e| e.to_string())?);
    }
    Ok(history)
}

#[tauri::command]
pub fn clear_translation_history(app: AppHandle) -> Result<(), String> {
    let state = app.state::<DbState>();
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM translation_history", [])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn clear_all_records(app: AppHandle) -> Result<(), String> {
    let state = app.state::<DbState>();
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM clipboard_records", [])
        .map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM phrases", [])
        .map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM phrase_groups", [])
        .map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM translation_history", [])
        .map_err(|e| e.to_string())?;
    let _ = app.emit("clipboard-update", ());
    let _ = app.emit("phrase-groups-changed", ());
    Ok(())
}

#[tauri::command]
pub fn get_setting(app: AppHandle, key: String) -> Result<String, String> {
    Ok(cached_setting(&app, &key))
}

pub fn get_setting_sync(app: &AppHandle, key: &str) -> Option<String> {
    let v = cached_setting(app, key);
    if v.is_empty() { None } else { Some(v) }
}

#[tauri::command]
pub fn get_all_settings(app: AppHandle) -> Result<std::collections::HashMap<String, String>, String> {
    {
        let cache = settings_cache().lock().map_err(|e| e.to_string())?;
        if !cache.is_empty() {
            return Ok(cache.clone());
        }
    }
    // Cache cold — populate from DB
    warm_settings_cache(&app);
    settings_cache().lock().map_err(|e| e.to_string()).map(|c| c.clone())
}

#[tauri::command]
pub fn get_image_base64(app: AppHandle, path: String) -> Result<String, String> {
    let mut base_dir = get_storage_dir(&app);
    base_dir.push(&path);

    let bytes = std::fs::read(&base_dir)
        .map_err(|e| format!("read image file: {}", e))?;

    use base64::Engine;
    Ok(base64::engine::general_purpose::STANDARD.encode(&bytes))
}

#[tauri::command]
pub fn get_image_thumbnail(app: AppHandle, path: String, max_size: u32) -> Result<String, String> {
    let base_dir = get_storage_dir(&app);
    let image_path = base_dir.join(&path);

    // Try pre-generated thumbnail first (saved during clipboard capture)
    let thumb_dir = image_path.parent().unwrap_or(&base_dir).join("thumbs");
    let filename = image_path.file_name().ok_or("invalid path")?;
    let thumb_path = thumb_dir.join(filename);

    let thumb_bytes = if thumb_path.exists() {
        std::fs::read(&thumb_path).map_err(|e| format!("read thumbnail: {}", e))?
    } else {
        // Fallback: generate thumbnail from full image
        let bytes = std::fs::read(&image_path)
            .map_err(|e| format!("read image file: {}", e))?;
        let img = image::load_from_memory(&bytes)
            .map_err(|e| format!("decode image: {}", e))?;
        let data = crate::clipboard::generate_thumbnail(&img, max_size)
            .map_err(|e| format!("generate thumbnail: {}", e))?;
        // Save for future use
        std::fs::create_dir_all(&thumb_dir).ok();
        let _ = std::fs::write(&thumb_path, &data);
        data
    };

    use base64::Engine;
    Ok(base64::engine::general_purpose::STANDARD.encode(&thumb_bytes))
}

#[tauri::command]
pub fn set_setting(app: AppHandle, key: String, value: String) -> Result<(), String> {
    if key == "storage_path" {
        return migrate_storage(&app, &value);
    }

    let state = app.state::<DbState>();
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = ?2",
        params![key, value],
    )
    .map_err(|e| e.to_string())?;

    update_cached_setting(&key, &value);

    if key == "translate_proxy" {
        crate::translator::invalidate_proxy_client();
    }

    let _ = app.emit("settings-changed", serde_json::json!({ &key: &value }));
    Ok(())
}

#[tauri::command]
pub fn set_settings_batch(app: AppHandle, settings: std::collections::HashMap<String, String>) -> Result<(), String> {
    let state = app.state::<DbState>();
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    for (key, value) in &settings {
        if key == "storage_path" {
            return migrate_storage(&app, value);
        }
    }
    let mut proxy_changed = false;
    for (key, value) in &settings {
        conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = ?2",
            params![key, value],
        )
        .map_err(|e| e.to_string())?;
        update_cached_setting(key, value);
        if key == "translate_proxy" {
            proxy_changed = true;
        }
    }
    if proxy_changed {
        crate::translator::invalidate_proxy_client();
    }

    let _ = app.emit("settings-changed", serde_json::json!(settings));
    Ok(())
}

fn migrate_storage(app: &AppHandle, new_path: &str) -> Result<(), String> {
    let custom_dir = PathBuf::from(new_path);
    std::fs::create_dir_all(&custom_dir).map_err(|e| format!("create dir: {}", e))?;
    let custom_db = custom_dir.join("data.db");

    // Collect all data from current DB
    let (settings, clipboard, phrases_data, phrase_groups_data, translations) = {
        let state = app.state::<DbState>();
        let conn = state.conn.lock().map_err(|e| e.to_string())?;

        let settings: Vec<(String, String)> = {
            let mut stmt = conn
                .prepare("SELECT key, value FROM settings")
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))
                .map_err(|e| e.to_string())?;
            rows.filter_map(|r| r.ok()).collect()
        };

        let clipboard: Vec<Vec<String>> = {
            let mut stmt = conn
                .prepare("SELECT id, type, content, source_app, created_at FROM clipboard_records")
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map([], |row| Ok(vec![
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                ]))
                .map_err(|e| e.to_string())?;
            rows.filter_map(|r| r.ok()).collect()
        };

        let phrase_groups_data: Vec<Vec<String>> = {
            let mut stmt = conn
                .prepare("SELECT id, name, sort_order, created_at, updated_at FROM phrase_groups")
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map([], |row| Ok(vec![
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i32>(2)?.to_string(),
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                ]))
                .map_err(|e| e.to_string())?;
            rows.filter_map(|r| r.ok()).collect()
        };

        let phrases_data: Vec<Vec<String>> = {
            let mut stmt = conn
                .prepare("SELECT id, group_id, title, content, sort_order, created_at, updated_at FROM phrases")
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map([], |row| Ok(vec![
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, i32>(4)?.to_string(),
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                ]))
                .map_err(|e| e.to_string())?;
            rows.filter_map(|r| r.ok()).collect()
        };

        let translations: Vec<Vec<String>> = {
            let mut stmt = conn
                .prepare("SELECT id, source_text, target_text, source_lang, target_lang, engine, created_at FROM translation_history")
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map([], |row| Ok(vec![
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                ]))
                .map_err(|e| e.to_string())?;
            rows.filter_map(|r| r.ok()).collect()
        };

        (settings, clipboard, phrases_data, phrase_groups_data, translations)
    };

    // Create new DB with schema at target location
    let new_conn = Connection::open(&custom_db).map_err(|e| format!("open new db: {}", e))?;

    new_conn
        .execute_batch(SCHEMA_SQL)
        .map_err(|e| format!("create schema: {}", e))?;

    // Copy all data to new DB
    {
        // Settings
        let mut stmt = new_conn
            .prepare("INSERT INTO settings (key, value) VALUES (?1, ?2)")
            .map_err(|e| e.to_string())?;
        for (k, v) in &settings {
            if k != "storage_path" && k != "shortcut_key" {
                stmt.execute(params![k, v]).map_err(|e| e.to_string())?;
            }
        }
        stmt.execute(params!["storage_path", new_path])
            .map_err(|e| e.to_string())?;
        stmt.execute(params!["shortcut_key", ""])
            .map_err(|e| e.to_string())?;
        drop(stmt);

        // Clipboard records
        if !clipboard.is_empty() {
            let mut stmt = new_conn
                .prepare("INSERT INTO clipboard_records (id, type, content, source_app, created_at) VALUES (?1, ?2, ?3, ?4, ?5)")
                .map_err(|e| e.to_string())?;
            for row in &clipboard {
                stmt.execute(params![row[0], row[1], row[2], row[3], row[4]])
                    .map_err(|e| e.to_string())?;
            }
        }

        // Phrase groups
        if !phrase_groups_data.is_empty() {
            let mut stmt = new_conn
                .prepare("INSERT INTO phrase_groups (id, name, sort_order, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)")
                .map_err(|e| e.to_string())?;
            for row in &phrase_groups_data {
                stmt.execute(params![row[0], row[1], row[2], row[3], row[4]])
                    .map_err(|e| e.to_string())?;
            }
        }

        // Phrases
        if !phrases_data.is_empty() {
            let mut stmt = new_conn
                .prepare("INSERT INTO phrases (id, group_id, title, content, sort_order, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)")
                .map_err(|e| e.to_string())?;
            for row in &phrases_data {
                stmt.execute(params![row[0], row[1], row[2], row[3], row[4], row[5], row[6]])
                    .map_err(|e| e.to_string())?;
            }
        }

        // Translation history
        if !translations.is_empty() {
            let mut stmt = new_conn
                .prepare("INSERT INTO translation_history (id, source_text, target_text, source_lang, target_lang, engine, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)")
                .map_err(|e| e.to_string())?;
            for row in &translations {
                stmt.execute(params![row[0], row[1], row[2], row[3], row[4], row[5], row[6]])
                    .map_err(|e| e.to_string())?;
            }
        }
    }

    // Copy images directory if it exists
    let old_storage_dir = get_storage_dir(app);
    let old_images = old_storage_dir.join("images");
    if old_images.exists() {
        let new_images = custom_dir.join("images");
        if let Err(e) = copy_dir_recursive(&old_images, &new_images) {
            log::warn!("Failed to copy images directory during migration: {}", e);
        }
    }

    // Update old DB's storage_path (for chain-following on restart) and switch connection
    {
        let state = app.state::<DbState>();
        let mut conn = state.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO settings (key, value) VALUES ('storage_path', ?1) ON CONFLICT(key) DO UPDATE SET value = ?1",
            params![new_path],
        )
        .map_err(|e| e.to_string())?;
        *conn = new_conn;
    }

    invalidate_storage_dir_cache();
    // Pre-populate the cache with the new path
    if let Some(cache) = STORAGE_DIR_CACHE.get() {
        if let Ok(mut c) = cache.lock() {
            *c = Some(custom_dir.clone());
        }
    }

    log::info!("Storage migrated to: {}", new_path);
    Ok(())
}

fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) -> Result<(), String> {
    std::fs::create_dir_all(dst).map_err(|e| format!("create dir: {}", e))?;
    let entries = std::fs::read_dir(src).map_err(|e| format!("read dir: {}", e))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("read entry: {}", e))?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)
                .map_err(|e| format!("copy file: {}", e))?;
        }
    }
    Ok(())
}

#[tauri::command]
pub fn get_storage_path(app: AppHandle) -> Result<String, String> {
    Ok(get_storage_dir(&app).to_string_lossy().to_string())
}

#[tauri::command]
pub fn ensure_thumbnail(app: AppHandle, path: String) -> Result<String, String> {
    let mut base = get_storage_dir(&app);
    base.push(&path);

    if !base.exists() {
        return Err("image file not found".to_string());
    }

    let filename = base.file_name().ok_or("invalid path")?.to_string_lossy().to_string();
    let mut thumb_dir = base.parent().ok_or("invalid path")?.to_path_buf();
    thumb_dir.push("thumbs");
    std::fs::create_dir_all(&thumb_dir).ok();
    let thumb_path = thumb_dir.join(&filename);

    if thumb_path.exists() {
        return Ok(thumb_path.to_string_lossy().to_string());
    }

    let bytes = std::fs::read(&base).map_err(|e| format!("read image: {}", e))?;
    let img = image::load_from_memory(&bytes).map_err(|e| format!("decode image: {}", e))?;

    let thumb_bytes = crate::clipboard::generate_thumbnail(&img, 200)
        .map_err(|e| format!("generate thumbnail: {}", e))?;
    std::fs::write(&thumb_path, &thumb_bytes).map_err(|e| format!("write thumbnail: {}", e))?;

    Ok(thumb_path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn select_storage_folder(app: AppHandle) -> Result<String, String> {
    use tauri_plugin_dialog::DialogExt;
    let (tx, rx) = std::sync::mpsc::channel();
    app.dialog().file().pick_folder(move |path| {
        let _ = tx.send(path);
    });
    let result = tokio::task::spawn_blocking(move || {
        rx.recv_timeout(std::time::Duration::from_secs(60))
    })
    .await
    .map_err(|e| format!("task error: {}", e))?;

    match result {
        Ok(Some(path)) => Ok(path.to_string()),
        Ok(None) => Err("cancelled".to_string()),
        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => Err("timeout".to_string()),
        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => Err("cancelled".to_string()),
    }
}
