use std::io::Write;
use std::sync::atomic::Ordering;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_clipboard_manager::ClipboardExt;

use image::ImageEncoder;

fn hash_bytes(data: &[u8]) -> u64 {
    xxhash_rust::xxh3::xxh3_64(data)
}

fn encode_rgba_to_png(rgba: &[u8], w: u32, h: u32) -> Vec<u8> {
    let mut bytes = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut bytes);
    let _ = encoder.write_image(rgba, w, h, image::ExtendedColorType::Rgba8);
    bytes
}

fn save_thumbnail(image_dir: &std::path::Path, filename: &str, img: &image::DynamicImage) {
    let mut thumb_dir = image_dir.to_path_buf();
    thumb_dir.push("thumbs");
    std::fs::create_dir_all(&thumb_dir).ok();
    let thumb_path = thumb_dir.join(filename);
    if thumb_path.exists() { return; }
    let thumb_bytes = match generate_thumbnail(img, 200) {
        Ok(b) => b,
        Err(e) => {
            log::warn!("clipboard: generate thumbnail failed: {}", e);
            return;
        }
    };
    if !thumb_bytes.is_empty() {
        if let Ok(mut f) = std::fs::File::create(&thumb_path) {
            let _ = f.write_all(&thumb_bytes);
        }
    }
}

fn is_url(text: &str) -> bool {
    let lower = text.trim().to_lowercase();
    lower.starts_with("http://")
        || lower.starts_with("https://")
        || lower.starts_with("ftp://")
        || lower.starts_with("ftps://")
}

fn is_previewable_image_file(path: &str) -> bool {
    let lower = path.to_lowercase();
    lower.ends_with(".jpg")
        || lower.ends_with(".jpeg")
        || lower.ends_with(".png")
}

const IMAGE_PREVIEW_MAX_BYTES: u64 = 3 * 1024 * 1024;

/// Generate a thumbnail from a decoded image, returning PNG-encoded bytes.
/// The thumbnail is constrained to fit within max_size on its longest side.
pub fn generate_thumbnail(img: &image::DynamicImage, max_size: u32) -> image::ImageResult<Vec<u8>> {
    let (w, h) = (img.width(), img.height());
    let scale = if w > max_size || h > max_size {
        max_size as f32 / w.max(h) as f32
    } else {
        1.0
    };
    let thumb = if scale < 1.0 {
        img.resize(
            (w as f32 * scale) as u32,
            (h as f32 * scale) as u32,
            image::imageops::FilterType::Triangle,
        )
    } else {
        img.clone()
    };
    let mut buf = std::io::Cursor::new(Vec::new());
    thumb.write_to(&mut buf, image::ImageFormat::Png)?;
    Ok(buf.into_inner())
}

fn is_image_file(path: &str) -> bool {
    let lower = path.to_lowercase();
    lower.ends_with(".png")
        || lower.ends_with(".jpg")
        || lower.ends_with(".jpeg")
        || lower.ends_with(".gif")
        || lower.ends_with(".bmp")
        || lower.ends_with(".webp")
        || lower.ends_with(".ico")
}

/// Import an image file from disk into the storage directory.
/// Returns true if the file was imported as an image record.
fn import_image_file(app: &AppHandle, file_path: &str) -> bool {
    let file_size = std::fs::metadata(file_path)
        .map(|m| m.len())
        .unwrap_or(0);

    let should_import = is_previewable_image_file(file_path)
        .then(|| file_size < IMAGE_PREVIEW_MAX_BYTES)
        .unwrap_or(true);

    if !should_import {
        return false;
    }

    let img_bytes = match std::fs::read(file_path) {
        Ok(b) => b,
        Err(_) => return false,
    };

    let decoded = match image::load_from_memory(&img_bytes) {
        Ok(d) => d,
        Err(_) => return false,
    };

    let rgba = decoded.to_rgba8();
    let img_w = decoded.width();
    let img_h = decoded.height();

    let content_hash: u64 = hash_bytes(&rgba);
    let content_hash_str = format!("{:016x}", content_hash);
    let filename = format!("{}.png", content_hash_str);
    let relative = format!("images/{}", filename);

    let png_bytes = encode_rgba_to_png(&rgba, img_w, img_h);

    if png_bytes.is_empty() {
        return false;
    }

    let mut dir = crate::db::get_storage_dir(app);
    dir.push("images");
    std::fs::create_dir_all(&dir).ok();

    let out_path = dir.join(&filename);
    if !out_path.exists() {
        if let Ok(mut f) = std::fs::File::create(&out_path) {
            let _ = f.write_all(&png_bytes);
        }
    }

    crate::paste::cache_image(relative.clone(), rgba.to_vec(), img_w, img_h, png_bytes.clone());

    save_thumbnail(&dir, &filename, &decoded);

    *LAST_IMAGE_RELATIVE.lock().unwrap_or_else(|e| e.into_inner()) = relative.clone();

    insert_and_emit(app, "image", &relative);
    true
}

/// Lightweight: hash the raw clipboard image bytes (PNG or DIB) for stable dedup.
/// Raw clipboard bytes are deterministic across reads, unlike re-decoded RGBA.
#[cfg(target_os = "windows")]
fn get_clipboard_image_hash() -> u64 {
    use windows::Win32::System::DataExchange::*;
    use windows::Win32::System::Memory::*;
    use windows::Win32::Foundation::{HWND, HGLOBAL};

    unsafe {
        if OpenClipboard(HWND(std::ptr::null_mut())).is_err() {
            return 0;
        }

        let mut result = 0u64;

        // Hash PNG format if available (most stable)
        let png_format_name: Vec<u16> = "PNG\0".encode_utf16().collect();
        let cf_png = RegisterClipboardFormatW(windows::core::PCWSTR(png_format_name.as_ptr()));
        if cf_png != 0 && IsClipboardFormatAvailable(cf_png).is_ok() {
            if let Ok(handle) = GetClipboardData(cf_png) {
                let hglobal = HGLOBAL(handle.0);
                let size = GlobalSize(hglobal);
                if size > 0 {
                    let ptr = GlobalLock(hglobal);
                    if !ptr.is_null() {
                        let bytes = std::slice::from_raw_parts(ptr as *const u8, size);
                        result = hash_bytes(bytes);
                        let _ = GlobalUnlock(hglobal);
                        let _ = CloseClipboard();
                        return result;
                    }
                    let _ = GlobalUnlock(hglobal);
                }
            }
        }

        // Fallback: hash DIB header + first 256 pixels for stable fingerprint
        const CF_DIB_VAL: u32 = 8;
        if IsClipboardFormatAvailable(CF_DIB_VAL).is_ok() {
            if let Ok(handle) = GetClipboardData(CF_DIB_VAL) {
                let hglobal = HGLOBAL(handle.0);
                let size = GlobalSize(hglobal);
                if size >= 40 {
                    let ptr = GlobalLock(hglobal);
                    if !ptr.is_null() {
                        let src = ptr as *const u8;
                        // Hash DIB header (40 bytes) + first 1024 bytes of pixel data
                        let hash_len = (40 + 1024).min(size);
                        let bytes = std::slice::from_raw_parts(src, hash_len);
                        result = hash_bytes(bytes);
                    }
                    let _ = GlobalUnlock(hglobal);
                }
            }
        }

        let _ = CloseClipboard();
        result
    }
}

/// Direct Windows clipboard image read as a supplement to the clipboard plugin.
/// Returns decoded RGBA data + dimensions (only called when image hash changed).
#[cfg(target_os = "windows")]
fn read_clipboard_image_raw() -> Option<(Vec<u8>, u32, u32)> {
    use windows::Win32::System::DataExchange::*;
    use windows::Win32::System::Memory::*;
    use windows::Win32::Foundation::{HWND, HGLOBAL};

    const CF_DIB_VAL: u32 = 8;
    const CF_DIBV5_VAL: u32 = 17;

    unsafe {
        if OpenClipboard(HWND(std::ptr::null_mut())).is_err() {
            return None;
        }

        // Try PNG format first (lossless, preserves alpha)
        let png_format_name: Vec<u16> = "PNG\0".encode_utf16().collect();
        let cf_png = RegisterClipboardFormatW(windows::core::PCWSTR(png_format_name.as_ptr()));
        if cf_png != 0 && IsClipboardFormatAvailable(cf_png).is_ok() {
            if let Ok(handle) = GetClipboardData(cf_png) {
                let hglobal = HGLOBAL(handle.0);
                let size = GlobalSize(hglobal);
                if size > 0 {
                    let ptr = GlobalLock(hglobal);
                    if !ptr.is_null() {
                        let bytes = std::slice::from_raw_parts(ptr as *const u8, size).to_vec();
                        let _ = GlobalUnlock(hglobal);
                        let _ = CloseClipboard();
                        if let Ok(img) = image::load_from_memory(&bytes) {
                            let rgba = img.to_rgba8();
                            let (w, h) = (img.width(), img.height());
                            return Some((rgba.to_vec(), w, h));
                        }
                    }
                    let _ = GlobalUnlock(hglobal);
                }
            }
        }

        // Try CF_DIBV5 first, then CF_DIB
        for format in [CF_DIBV5_VAL, CF_DIB_VAL] {
            if IsClipboardFormatAvailable(format).is_err() {
                continue;
            }
            if let Ok(handle) = GetClipboardData(format) {
                let hglobal = HGLOBAL(handle.0);
                let size = GlobalSize(hglobal);
                if size >= 40 {
                    let ptr = GlobalLock(hglobal);
                    if !ptr.is_null() {
                        let header = ptr as *const u32;
                        let bi_size = *header;
                        let bpp = *((ptr as *const u8).add(14)) as u16;
                        let compression = *header.add(4);
                        let w = *header.add(1) as i32;
                        let h = (*header.add(2) as i32).abs();
                        if w > 0 && h > 0 && w < 20000 && h < 20000 {
                            let rgba = if bpp == 32 && compression == 0 {
                                let pixel_count = (w * h) as usize;
                                let src = (ptr as *const u8).add(bi_size as usize);
                                let mut rgba = vec![0u8; pixel_count * 4];
                                for i in 0..pixel_count {
                                    rgba[i * 4] = *src.add(i * 4 + 2);
                                    rgba[i * 4 + 1] = *src.add(i * 4 + 1);
                                    rgba[i * 4 + 2] = *src.add(i * 4);
                                    rgba[i * 4 + 3] = *src.add(i * 4 + 3);
                                }
                                rgba
                            } else {
                                let full = std::slice::from_raw_parts(ptr as *const u8, size).to_vec();
                                let _ = GlobalUnlock(hglobal);
                                let _ = CloseClipboard();
                                let pixel_offset = bi_size;
                                let file_size = 14 + size as u32;
                                let mut bmp: Vec<u8> = Vec::with_capacity(file_size as usize);
                                bmp.extend_from_slice(b"BM");
                                bmp.extend_from_slice(&file_size.to_le_bytes());
                                bmp.extend_from_slice(&0u32.to_le_bytes());
                                bmp.extend_from_slice(&(14u32 + pixel_offset).to_le_bytes());
                                bmp.extend_from_slice(&full);
                                if let Ok(img) = image::load_from_memory(&bmp) {
                                    let rgba = img.to_rgba8();
                                    return Some((rgba.to_vec(), img.width(), img.height()));
                                }
                                return None;
                            };
                            let _ = GlobalUnlock(hglobal);
                            let _ = CloseClipboard();
                            return Some((rgba, w as u32, h as u32));
                        }
                        let _ = GlobalUnlock(hglobal);
                    }
                }
            }
        }

        let _ = CloseClipboard();
    }
    None
}

#[cfg(target_os = "windows")]
fn read_clipboard_files() -> Option<Vec<String>> {
    use windows::Win32::System::DataExchange::*;
    use windows::Win32::UI::Shell::{DragQueryFileW, HDROP};
    use windows::Win32::Foundation::HWND;

    const CF_HDROP: u32 = 15;

    unsafe {
        if OpenClipboard(HWND(std::ptr::null_mut())).is_err() {
            return None;
        }

        if IsClipboardFormatAvailable(CF_HDROP).is_err() {
            let _ = CloseClipboard();
            return None;
        }

        let handle = match GetClipboardData(CF_HDROP) {
            Ok(h) => h,
            Err(_) => {
                let _ = CloseClipboard();
                return None;
            }
        };

        let hdrop = HDROP(handle.0);

        let count = DragQueryFileW(hdrop, 0xFFFFFFFF, None);
        if count == 0 {
            let _ = CloseClipboard();
            return None;
        }

        let mut paths = Vec::new();
        for i in 0..count {
            let len = DragQueryFileW(hdrop, i, None);
            if len == 0 {
                continue;
            }
            let mut buf = vec![0u16; (len as usize) + 1];
            DragQueryFileW(hdrop, i, Some(&mut buf));
            let end = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
            let path = String::from_utf16_lossy(&buf[..end]);
            if !path.is_empty() {
                paths.push(path);
            }
        }

        let _ = CloseClipboard();

        if paths.is_empty() {
            None
        } else {
            Some(paths)
        }
    }
}

/// Cached clipboard state, updated by the monitor and by paste functions.
/// When paste writes to the clipboard, it syncs these to prevent duplicate records.
pub static LAST_CLIPBOARD_TEXT: std::sync::Mutex<String> = std::sync::Mutex::new(String::new());
pub static LAST_CLIPBOARD_IMAGE_HASH: std::sync::Mutex<u64> = std::sync::Mutex::new(0);
#[cfg(target_os = "windows")]
pub static LAST_CLIPBOARD_FILES_KEY: std::sync::Mutex<String> = std::sync::Mutex::new(String::new());
/// Cached relative path of the last recorded image, avoids re-decoding on re-copy.
static LAST_IMAGE_RELATIVE: std::sync::Mutex<String> = std::sync::Mutex::new(String::new());

/// Windows clipboard sequence number — increments on every clipboard change,
/// even if content is identical. Used to detect re-copies of the same content.
#[cfg(target_os = "windows")]
static LAST_CLIPBOARD_SEQ: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);


#[cfg(target_os = "windows")]
fn get_clipboard_sequence() -> u32 {
    use windows::Win32::System::DataExchange::GetClipboardSequenceNumber;
    unsafe { GetClipboardSequenceNumber() }
}

/// Insert a new record into the DB and emit clipboard-update.
/// Skips insertion only if the most recent record has identical type and content
/// AND was created within the last 2 seconds (debounce window).
/// Re-copies after 2 seconds are treated as intentional and recorded normally.
fn insert_and_emit(app: &AppHandle, record_type: &str, content: &str) {
    let one_second_ago = chrono::Utc::now() - chrono::Duration::seconds(1);
    let cutoff = one_second_ago.to_rfc3339();

    let is_duplicate: bool = {
        let state = app.state::<crate::db::DbState>();
        let x = match state.conn.lock() {
            Ok(conn) => conn.query_row(
                "SELECT type, content, created_at FROM clipboard_records ORDER BY created_at DESC LIMIT 1",
                [],
                |row| {
                    let last_type: String = row.get(0)?;
                    let last_content: String = row.get(1)?;
                    let last_created: String = row.get(2)?;
                    Ok(last_type == record_type && last_content == content && last_created >= cutoff)
                },
            )
            .unwrap_or(false),
            Err(_) => false,
        };
        x
    };

    if is_duplicate {
        return;
    }

    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    {
        let state = app.state::<crate::db::DbState>();
        let _x = match state.conn.lock() {
            Ok(conn) => conn.execute(
                "INSERT INTO clipboard_records (id, type, content, source_app, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![id, record_type, content, "", &now],
            ).ok(),
            Err(_) => None,
        };
    }
    app.emit(
        "clipboard-update",
        serde_json::json!({
            "id": id,
            "type": record_type,
            "content": content,
            "source_app": "",
            "created_at": now,
        }),
    ).ok();
}

pub fn sync_monitor_cache(handle: &AppHandle) {
    if let Ok(text) = handle.clipboard().read_text() {
        *LAST_CLIPBOARD_TEXT.lock().unwrap_or_else(|e| e.into_inner()) = text.trim().to_string();
    }
    #[cfg(target_os = "windows")]
    {
        let h = get_clipboard_image_hash();
        if h != 0 {
            *LAST_CLIPBOARD_IMAGE_HASH.lock().unwrap_or_else(|e| e.into_inner()) = h;
        }
        if let Some(files) = read_clipboard_files() {
            *LAST_CLIPBOARD_FILES_KEY.lock().unwrap_or_else(|e| e.into_inner()) = files.join("|");
        }
        LAST_CLIPBOARD_SEQ.store(get_clipboard_sequence(), Ordering::SeqCst);
    }
}

fn save_monitor_image(
    handle: &AppHandle,
    rgba_vec: Vec<u8>,
    img_w: u32,
    img_h: u32,
) -> bool {
    let content_hash: u64 = hash_bytes(&rgba_vec);
    let content_hash_str = format!("{:016x}", content_hash);
    let filename = format!("{}.png", content_hash_str);
    let relative = format!("images/{}", filename);

    let png_bytes = encode_rgba_to_png(&rgba_vec, img_w, img_h);

    if png_bytes.is_empty() {
        return false;
    }

    let mut dir = crate::db::get_storage_dir(handle);
    dir.push("images");
    std::fs::create_dir_all(&dir).ok();

    let filepath = dir.join(&filename);

    if !filepath.exists() {
        if let Ok(mut f) = std::fs::File::create(&filepath) {
            let _ = f.write_all(&png_bytes);
        }
    }

    log::info!("clipboard: recorded image {}x{} hash={}", img_w, img_h, content_hash_str);

    crate::paste::cache_image(relative.clone(), rgba_vec.clone(), img_w, img_h, png_bytes.clone());

    // Generate thumbnail directly from RGBA buffer, avoids PNG encode→decode round-trip
    if let Some(rgba_img) = image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(img_w, img_h, rgba_vec) {
        save_thumbnail(&dir, &filename, &image::DynamicImage::ImageRgba8(rgba_img));
    }

    // Cache relative path for image re-copy optimization
    *LAST_IMAGE_RELATIVE.lock().unwrap_or_else(|e| e.into_inner()) = relative.clone();

    insert_and_emit(handle, "image", &relative);
    true
}

fn handle_monitor_text(handle: &AppHandle) {
    if let Ok(text) = handle.clipboard().read_text() {
        let text = text.trim().to_string();
        if !text.is_empty() && text != *LAST_CLIPBOARD_TEXT.lock().unwrap_or_else(|e| e.into_inner()) {
            *LAST_CLIPBOARD_TEXT.lock().unwrap_or_else(|e| e.into_inner()) = text.clone();
            let record_type = if is_url(&text) { "link" } else { "text" };
            insert_and_emit(handle, record_type, &text);
        } else if !text.is_empty() {
            let record_type = if is_url(&text) { "link" } else { "text" };
            insert_and_emit(handle, record_type, &text);
        }
    }
}

fn handle_monitor_files(handle: &AppHandle) {
    #[cfg(target_os = "windows")]
    {
        if let Some(files) = read_clipboard_files() {
            let key = files.join("|");
            {
                let mut cached = LAST_CLIPBOARD_FILES_KEY.lock().unwrap_or_else(|e| e.into_inner());
                if key == *cached {
                    for file_path in &files {
                        if file_path.trim().is_empty() { continue; }
                        if is_previewable_image_file(file_path) || is_image_file(file_path) {
                            import_image_file(handle, file_path);
                            continue;
                        }
                        insert_and_emit(handle, "file", file_path);
                    }
                    return;
                }
                *cached = key.clone();
            }

            for file_path in files {
                if file_path.trim().is_empty() {
                    continue;
                }
                if is_previewable_image_file(&file_path) || is_image_file(&file_path) {
                    if import_image_file(handle, &file_path) {
                        continue;
                    }
                    continue;
                }
                insert_and_emit(handle, "file", &file_path);
            }
        }
    }
}

pub fn start_monitor(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let handle = app.clone();

    {
        let initial_text = handle.clipboard().read_text()
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        *LAST_CLIPBOARD_TEXT.lock().unwrap_or_else(|e| e.into_inner()) = initial_text;
    }

    #[cfg(target_os = "windows")]
    {
        *LAST_CLIPBOARD_IMAGE_HASH.lock().unwrap_or_else(|e| e.into_inner()) = get_clipboard_image_hash();
    }

    #[cfg(target_os = "windows")]
    {
        let key = read_clipboard_files()
            .map(|files| files.join("|"))
            .unwrap_or_default();
        *LAST_CLIPBOARD_FILES_KEY.lock().unwrap_or_else(|e| e.into_inner()) = key;
        LAST_CLIPBOARD_SEQ.store(get_clipboard_sequence(), Ordering::SeqCst);
    }

    std::thread::spawn(move || {
        let mut poll_count: u32 = 0;
        loop {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        std::thread::sleep(std::time::Duration::from_millis(500));
        poll_count += 1;

        // Skip first 3 polls (1.5s) to avoid recording startup clipboard state
        if poll_count <= 3 {
            sync_monitor_cache(&handle);
            return;
        }

        if crate::paste::PASTING.load(std::sync::atomic::Ordering::SeqCst) {
            sync_monitor_cache(&handle);
            return;
        }

        // Detect clipboard changes via Windows sequence number.
        // This catches re-copies of identical content (e.g. same image twice).
        let seq_changed = {
            #[cfg(target_os = "windows")]
            {
                let current_seq = get_clipboard_sequence();
                let last_seq = LAST_CLIPBOARD_SEQ.load(Ordering::SeqCst);
                if current_seq != last_seq {
                    LAST_CLIPBOARD_SEQ.store(current_seq, Ordering::SeqCst);
                    true
                } else {
                    false
                }
            }
            #[cfg(not(target_os = "windows"))]
            { false }
        };

        if !seq_changed {
            return;
        }

        let mut image_recorded = false;

        let mut image_data: Option<(Vec<u8>, u32, u32)> = None;
        // Track whether the image hash stayed the same (re-copy of same image)
        let mut image_is_same = false;

        // Stable dedup: hash raw clipboard bytes (deterministic) rather than RGBA
        #[cfg(target_os = "windows")]
        {
            let raw_hash = get_clipboard_image_hash();
            let mut cached_hash = LAST_CLIPBOARD_IMAGE_HASH.lock().unwrap_or_else(|e| e.into_inner());
            if raw_hash != 0 && raw_hash != *cached_hash {
                *cached_hash = raw_hash;
                drop(cached_hash);
                if let Some((rgba, w, h)) = read_clipboard_image_raw() {
                    image_data = Some((rgba, w, h));
                }
            } else if raw_hash != 0 {
                // Sequence changed but image hash didn't — same image re-copied
                image_is_same = true;
                drop(cached_hash);
            } else {
                drop(cached_hash);
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            // Non-Windows: use plugin-based image read with RGBA hash
            if let Ok(image) = handle.clipboard().read_image() {
                let rgba = image.rgba();
                if !rgba.is_empty() && image.width() > 0 && image.height() > 0 {
                    let hash = hash_bytes(&rgba[..400.min(rgba.len())]);
                    let mut cached_hash = LAST_CLIPBOARD_IMAGE_HASH.lock().unwrap_or_else(|e| e.into_inner());
                    if hash != *cached_hash {
                        *cached_hash = hash;
                        image_data = Some((rgba.to_vec(), image.width(), image.height()));
                    }
                }
            }
        }

        if let Some((rgba_vec, img_w, img_h)) = image_data.take() {
            image_recorded = save_monitor_image(&handle, rgba_vec, img_w, img_h);
        }

        // Handle re-copy of same image: sequence changed but raw hash didn't.
        // Still insert a new chronological record pointing to the same file on disk.
        // Use cached relative path to avoid re-decoding the image from clipboard.
        if image_is_same {
            let relative = LAST_IMAGE_RELATIVE.lock().unwrap_or_else(|e| e.into_inner()).clone();
            if !relative.is_empty() {
                insert_and_emit(&handle, "image", &relative);
            }
            sync_monitor_cache(&handle);
        } else if image_recorded {
            if let Ok(text) = handle.clipboard().read_text() {
                *LAST_CLIPBOARD_TEXT.lock().unwrap_or_else(|e| e.into_inner()) = text.trim().to_string();
            }
            #[cfg(target_os = "windows")]
            {
                if let Some(files) = read_clipboard_files() {
                    *LAST_CLIPBOARD_FILES_KEY.lock().unwrap_or_else(|e| e.into_inner()) = files.join("|");
                }
            }
        } else {
            handle_monitor_text(&handle);
            handle_monitor_files(&handle);
        }
        }));
        if let Err(e) = result {
            let msg = if let Some(s) = e.downcast_ref::<String>() {
                s.clone()
            } else if let Some(s) = e.downcast_ref::<&str>() {
                s.to_string()
            } else {
                "unknown panic".to_string()
            };
            log::error!("clipboard monitor panic: {}. Restarting...", msg);
            // Reset poll count so we skip the first 2 polls after restart
            poll_count = 0;
            sync_monitor_cache(&handle);
        }
    }
    });

    Ok(())
}
