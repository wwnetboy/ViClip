use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};
use std::ptr;

pub static PASTING: AtomicBool = AtomicBool::new(false);

#[cfg(target_os = "windows")]
static LAST_FOREGROUND_HWND: AtomicPtr<core::ffi::c_void> = AtomicPtr::new(ptr::null_mut());

#[cfg(target_os = "windows")]
pub fn save_foreground_window() {
    use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;
    unsafe {
        let hwnd = GetForegroundWindow();
        LAST_FOREGROUND_HWND.store(hwnd.0, Ordering::SeqCst);
    }
}

#[cfg(target_os = "macos")]
fn paste_cmd_v_macos() {
    extern "C" {
        fn CGEventCreateKeyboardEvent(source: *mut std::ffi::c_void, virtual_key: u16, key_down: bool) -> *mut std::ffi::c_void;
        fn CGEventSetFlags(event: *mut std::ffi::c_void, flags: u64);
        fn CGEventPost(tap_location: u32, event: *mut std::ffi::c_void);
        fn CFRelease(cf: *mut std::ffi::c_void);
    }

    // kCGHIDEventTap = 0
    // kVK_Command = 0x37, kVK_ANSI_V = 0x09
    // kCGEventFlagMaskCommand = 0x00100000

    unsafe {
        // Cmd key down — set command flag to indicate Cmd is held
        let cmd_down = CGEventCreateKeyboardEvent(std::ptr::null_mut(), 0x37, true);
        if !cmd_down.is_null() {
            CGEventSetFlags(cmd_down, 0x00100000); // kCGEventFlagMaskCommand
            CGEventPost(0, cmd_down);
            CFRelease(cmd_down);
            log::info!("[paste] Cmd down posted");
        }

        thread::sleep(Duration::from_millis(50));

        // V key down — keep command flag to indicate Cmd is still held
        let v_down = CGEventCreateKeyboardEvent(std::ptr::null_mut(), 0x09, true);
        if !v_down.is_null() {
            CGEventSetFlags(v_down, 0x00100000); // Cmd still held
            CGEventPost(0, v_down);
            CFRelease(v_down);
            log::info!("[paste] V down posted");
        }

        thread::sleep(Duration::from_millis(30));

        // V key up
        let v_up = CGEventCreateKeyboardEvent(std::ptr::null_mut(), 0x09, false);
        if !v_up.is_null() {
            CGEventPost(0, v_up);
            CFRelease(v_up);
        }

        thread::sleep(Duration::from_millis(20));

        // Cmd key up — no flags (Cmd released)
        let cmd_up = CGEventCreateKeyboardEvent(std::ptr::null_mut(), 0x37, false);
        if !cmd_up.is_null() {
            CGEventPost(0, cmd_up);
            CFRelease(cmd_up);
            log::info!("[paste] Cmd up posted");
        }

        log::info!("[paste] CGEventPost Cmd+V sequence completed");
    }
}

#[cfg(target_os = "macos")]
fn deactivate_app_macos() {
    // Use performSelectorOnMainThread to avoid linker issues with dispatch_get_main_queue.
    use cocoa::base::id;
    use objc::{class, msg_send, sel, sel_impl};

    unsafe {
        let ns_app: id = msg_send![class!(NSApplication), sharedApplication];
        // Create a selector for deactivate
        let sel = objc::runtime::Sel::register("deactivate");
        // performSelectorOnMainThread:withObject:waitUntilDone:
        let _: () = msg_send![ns_app, performSelectorOnMainThread:sel withObject:std::ptr::null_mut::<objc::runtime::Object>() waitUntilDone:true];
    }
}

#[cfg(target_os = "macos")]
fn wait_for_modifier_keys_release_macos() {
    extern "C" {
        fn CGEventSourceKeyState(state_id: i32, keycode: u16) -> bool;
    }
    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_millis(500);
    loop {
        let ctrl_down = unsafe { CGEventSourceKeyState(1, 0x3B) }; // kCGEventSourceStateHIDSystemState=1, kVK_Control=0x3B
        let alt_down = unsafe { CGEventSourceKeyState(1, 0x3A) };  // kVK_Option=0x3A
        if !ctrl_down && !alt_down {
            break;
        }
        if start.elapsed() > timeout {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    std::thread::sleep(std::time::Duration::from_millis(30));
}

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

struct CachedImage {
    rgba: Arc<Vec<u8>>,
    width: u32,
    height: u32,
    png_bytes: Arc<Vec<u8>>,
}

struct ImageCache {
    map: HashMap<String, CachedImage>,
    order: Vec<String>,
}

static IMAGE_CACHE: OnceLock<Mutex<ImageCache>> = OnceLock::new();

fn get_image_cache() -> &'static Mutex<ImageCache> {
    IMAGE_CACHE.get_or_init(|| Mutex::new(ImageCache {
        map: HashMap::new(),
        order: Vec::new(),
    }))
}

struct PasteGuard;

impl Drop for PasteGuard {
    fn drop(&mut self) {
        PASTING.store(false, Ordering::SeqCst);
    }
}

pub fn cache_image(path: String, rgba: Vec<u8>, width: u32, height: u32, png_bytes: Vec<u8>) {
    let mut cache = get_image_cache().lock().unwrap_or_else(|e| e.into_inner());
    // Evict oldest entries (deterministic insertion order)
    if cache.map.len() >= 10 {
        let evict_count = 5.min(cache.order.len());
        let evicted: Vec<String> = cache.order.drain(..evict_count).collect();
        for k in &evicted {
            cache.map.remove(k);
        }
    }
    cache.order.push(path.clone());
    cache.map.insert(path, CachedImage {
        rgba: Arc::new(rgba),
        width,
        height,
        png_bytes: Arc::new(png_bytes),
    });
}

use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_clipboard_manager::ClipboardExt;
use enigo::{Enigo, Keyboard, Key, Direction, Settings};
use std::thread;
use std::time::Duration;

#[derive(serde::Serialize, Clone)]
struct ToastPayload {
    copy_type: String,
    preview: String,
}

fn emit_toast(app: &AppHandle, copy_type: &str, preview: String) {
    let _ = app.emit("toast-show", ToastPayload {
        copy_type: copy_type.to_string(),
        preview,
    });
}

macro_rules! debug_log {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        {
            use std::io::Write;
            let msg = format!($($arg)*);
            if let Ok(mut f) = std::fs::OpenOptions::new()
                .append(true)
                .create(true)
                .open(std::env::temp_dir().join("viclip_debug.log"))
            {
                let _ = writeln!(f, "{} {}", chrono::Local::now().format("%H:%M:%S%.3f"), msg);
            }
        }
    };
}

fn paste_with_defocus(app: &AppHandle) -> Result<(), String> {
    debug_log!("paste_with_defocus started");

    #[cfg(target_os = "windows")]
    unsafe {
        use windows::Win32::UI::WindowsAndMessaging::AllowSetForegroundWindow;
        let _ = AllowSetForegroundWindow(0xFFFFFFFF);
    }

    // Hide radial popup if visible
    if let Some(radial) = app.get_webview_window("radial-menu") {
        let _ = radial.hide();
    }

    // Restore foreground to the app the user was working in BEFORE hiding our window.
    // This way Windows doesn't need to find a new foreground window when we hide ours.
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::Foundation::HWND;
        use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindow, SetForegroundWindow, GW_HWNDNEXT};
        let target = LAST_FOREGROUND_HWND.load(Ordering::SeqCst);
        let target = if !target.is_null() {
            HWND(target)
        } else {
            // Fallback: find the window behind ours in the Z-order
            let our_hwnd = unsafe { GetForegroundWindow() };
            let next = unsafe { GetWindow(our_hwnd, GW_HWNDNEXT).unwrap_or(HWND(std::ptr::null_mut())) };
            debug_log!("fallback foreground: our={:?} next={:?}", our_hwnd, next);
            next
        };
        if target.0 != std::ptr::null_mut() {
            debug_log!("SetForegroundWindow to {:?}", target);
            unsafe {
                let _ = SetForegroundWindow(target);
            }
            thread::sleep(Duration::from_millis(50));
        }
    }

    let window = app
        .get_webview_window("main")
        .ok_or("no window")?;

    debug_log!("hiding main window, visible before: {:?}", window.is_visible());
    window.hide().map_err(|e| e.to_string())?;
    debug_log!("main window hidden, visible after: {:?}", window.is_visible());

    #[cfg(target_os = "macos")]
    {
        // MUST dispatch to main thread — AppKit APIs are not thread-safe.
        // NSApp.deactivate must be called on the main thread.
        deactivate_app_macos();
        log::info!("[paste] app deactivated, waiting for app switch");
        thread::sleep(Duration::from_millis(200));
    }

    // Wait for user to release Ctrl/Alt from the radial menu gesture (Ctrl+Alt+RightClick).
    // If we send Ctrl+V while the physical Ctrl is still held, the simulated Ctrl release
    // can race with the physical release, causing the target app to receive a bare 'V'.
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_CONTROL, VK_MENU};
        let start = std::time::Instant::now();
        let timeout = Duration::from_millis(500);
        loop {
            let ctrl_up = unsafe { (GetAsyncKeyState(VK_CONTROL.0 as i32) as u16) & 0x8000 } == 0;
            let alt_up = unsafe { (GetAsyncKeyState(VK_MENU.0 as i32) as u16) & 0x8000 } == 0;
            if ctrl_up && alt_up {
                break;
            }
            if start.elapsed() > timeout {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        // Small extra settle time for foreground window
        thread::sleep(Duration::from_millis(30));
    }

    #[cfg(target_os = "macos")]
    {
        wait_for_modifier_keys_release_macos();
        log::info!("[paste] modifiers released, will simulate Cmd+V");
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        thread::sleep(Duration::from_millis(200));
    }

    #[cfg(not(target_os = "macos"))]
    let mut enigo = Enigo::new(&Settings::default()).map_err(|e| format!("enigo init: {}", e))?;

    #[cfg(target_os = "windows")]
    {
        enigo.key(Key::Control, Direction::Press).map_err(|e| e.to_string())?;
        thread::sleep(Duration::from_millis(30));
        enigo.key(Key::V, Direction::Click).map_err(|e| e.to_string())?;
        thread::sleep(Duration::from_millis(10));
        enigo.key(Key::Control, Direction::Release).map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "macos")]
    {
        log::info!("[paste] simulating Cmd+V via CGEventPost");
        paste_cmd_v_macos();
    }

    debug_log!("paste_with_defocus completed OK");
    log::info!("[paste] paste_with_defocus completed OK");
    Ok(())
}

#[cfg(target_os = "windows")]
fn write_image_to_clipboard(rgba: &[u8], w: u32, h: u32, png_bytes: &[u8]) -> Result<(), String> {
    use windows::Win32::Foundation::{HWND, HANDLE};
    use windows::Win32::System::DataExchange::*;
    use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};

    const CF_DIB: u32 = 8;

    unsafe {
        if OpenClipboard(HWND(std::ptr::null_mut())).is_err() {
            return Err("OpenClipboard failed".to_string());
        }
        let _ = EmptyClipboard();

        let dib_size = 40 + (w * h * 4) as usize;
        let hmem_dib = GlobalAlloc(GMEM_MOVEABLE, dib_size).map_err(|e| {
            let _ = CloseClipboard();
            format!("GlobalAlloc DIB failed: {}", e)
        })?;

        let ptr_dib = GlobalLock(hmem_dib);
        if ptr_dib.is_null() {
            let _ = CloseClipboard();
            return Err("GlobalLock DIB failed".to_string());
        }

        let bmi = ptr_dib as *mut u8;
        // Zero the DIB header to avoid garbage biCompression / biClrUsed etc.
        std::ptr::write_bytes(bmi, 0u8, 40);
        let bmi_header = std::slice::from_raw_parts_mut(bmi as *mut u32, 10);
        bmi_header[0] = 40;
        bmi_header[1] = w;
        bmi_header[2] = (-(h as i32)) as u32;
        *(((bmi as *mut u8).add(12)) as *mut u16) = 1;
        *(((bmi as *mut u8).add(14)) as *mut u16) = 32;
        *(((bmi as *mut u8).add(20)) as *mut u32) = w * h * 4;

        // Convert RGBA → BGRA (DIB expects BGRA pixel order)
        let pixel_offset = 40;
        let dst = (bmi as *mut u8).add(pixel_offset);
        let src = rgba.as_ptr();
        for i in 0..(w * h) as usize {
            *dst.add(i * 4) = *src.add(i * 4 + 2);       // B = R
            *dst.add(i * 4 + 1) = *src.add(i * 4 + 1);   // G = G
            *dst.add(i * 4 + 2) = *src.add(i * 4);       // R = B
            *dst.add(i * 4 + 3) = *src.add(i * 4 + 3);   // A = A
        }
        let _ = GlobalUnlock(hmem_dib);

        if SetClipboardData(CF_DIB, HANDLE(hmem_dib.0)).is_err() {
            let _ = CloseClipboard();
            return Err("SetClipboardData DIB failed".to_string());
        }

        let png_format_name: Vec<u16> = "PNG\0".encode_utf16().collect();
        let cf_png = RegisterClipboardFormatW(windows::core::PCWSTR(png_format_name.as_ptr()));
        if cf_png != 0 {
            let hmem_png = GlobalAlloc(GMEM_MOVEABLE, png_bytes.len()).map_err(|e| {
                let _ = CloseClipboard();
                format!("GlobalAlloc PNG failed: {}", e)
            })?;

            let ptr_png = GlobalLock(hmem_png);
            if ptr_png.is_null() {
                let _ = CloseClipboard();
                return Err("GlobalLock PNG failed".to_string());
            }

            std::ptr::copy_nonoverlapping(png_bytes.as_ptr(), ptr_png as *mut u8, png_bytes.len());
            let _ = GlobalUnlock(hmem_png);

            if SetClipboardData(cf_png, HANDLE(hmem_png.0)).is_err() {
                let _ = CloseClipboard();
                return Err("SetClipboardData PNG failed".to_string());
            }
        }

        let _ = CloseClipboard();
    }

    Ok(())
}

#[cfg(target_os = "macos")]
fn write_image_to_clipboard_macos(png_bytes: &[u8]) -> Result<(), String> {
    use objc::{class, msg_send, sel, sel_impl};
    use objc::runtime::Object;

    unsafe {
        let pasteboard: *mut Object = msg_send![class!(NSPasteboard), generalPasteboard];
        let _: () = msg_send![pasteboard, clearContents];

        // Create NSData from png bytes
        let ns_data: *mut Object = msg_send![class!(NSData), dataWithBytes:png_bytes.as_ptr() length:png_bytes.len()];

        // NSPasteboardTypePNG = @"public.png" (it's an NSString constant, not a class)
        let ns_png_type: *mut Object = msg_send![class!(NSString), stringWithUTF8String: b"public.png\0".as_ptr() as *const std::os::raw::c_char];
        let written: bool = msg_send![pasteboard, setData:ns_data forType:ns_png_type];

        if !written {
            return Err("NSPasteboard setData for PNG failed".to_string());
        }
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn write_files_to_clipboard(paths: &[String]) -> Result<(), String> {
    use windows::Win32::System::DataExchange::*;
    use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};
    use windows::Win32::UI::Shell::DROPFILES;
    use windows::Win32::Foundation::{HWND, HANDLE};

    const CF_HDROP: u32 = 15;

    let wide_paths: Vec<Vec<u16>> = paths.iter().map(|p| p.encode_utf16().chain(std::iter::once(0u16)).collect()).collect();
    let total_wide_len: usize = wide_paths.iter().map(|p| p.len()).sum();

    let dropfiles_size = std::mem::size_of::<DROPFILES>();
    let data_size = dropfiles_size + (total_wide_len + 1) * std::mem::size_of::<u16>();

    let mut data: Vec<u8> = vec![0u8; data_size];

    let df = data.as_mut_ptr() as *mut DROPFILES;
    unsafe {
        (*df).pFiles = dropfiles_size as u32;
        (*df).pt = windows::Win32::Foundation::POINT { x: 0, y: 0 };
        (*df).fNC = windows::Win32::Foundation::BOOL(0);
        (*df).fWide = windows::Win32::Foundation::BOOL(1);
    }

    let offset = dropfiles_size;
    let mut pos = offset;
    for wp in &wide_paths {
        let byte_len = wp.len() * std::mem::size_of::<u16>();
        data[pos..pos + byte_len].copy_from_slice(unsafe { std::slice::from_raw_parts(wp.as_ptr() as *const u8, byte_len) });
        pos += byte_len;
    }

    unsafe {
        if OpenClipboard(HWND(std::ptr::null_mut())).is_err() {
            return Err("OpenClipboard failed".to_string());
        }
        let _ = EmptyClipboard();

        let hmem = GlobalAlloc(GMEM_MOVEABLE, data_size).map_err(|e| {
            let _ = CloseClipboard();
            format!("GlobalAlloc failed: {}", e)
        })?;

        let ptr = GlobalLock(hmem);
        if ptr.is_null() {
            let _ = CloseClipboard();
            return Err("GlobalLock failed".to_string());
        }

        std::ptr::copy_nonoverlapping(data.as_ptr(), ptr as *mut u8, data_size);
        let _ = GlobalUnlock(hmem);

        if SetClipboardData(CF_HDROP, HANDLE(hmem.0)).is_err() {
            let _ = CloseClipboard();
            return Err("SetClipboardData failed".to_string());
        }

        let _ = CloseClipboard();
    }

    Ok(())
}

#[cfg(target_os = "macos")]
fn write_files_to_clipboard_macos(paths: &[String]) -> Result<(), String> {
    use objc::{class, msg_send, sel, sel_impl};
    use objc::runtime::Object;

    unsafe {
        let pasteboard: *mut Object = msg_send![class!(NSPasteboard), generalPasteboard];
        let _: () = msg_send![pasteboard, clearContents];

        let mut urls: Vec<*mut Object> = Vec::new();
        for path in paths {
            let ns_str: *mut Object = msg_send![class!(NSString), stringWithUTF8String:path.as_ptr() as *const std::os::raw::c_char];
            let url: *mut Object = msg_send![class!(NSURL), fileURLWithPath:ns_str];
            urls.push(url);
        }

        let ns_array: *mut Object = msg_send![class!(NSArray), arrayWithObjects:urls.as_ptr() count:urls.len()];
        let _: bool = msg_send![pasteboard, writeObjects:ns_array];
    }
    Ok(())
}

#[tauri::command]
pub fn paste_text(app: AppHandle, text: String) -> Result<(), String> {
    debug_log!("paste_text called, len={}", text.len());

    if PASTING.swap(true, Ordering::SeqCst) {
        debug_log!("paste_text PASTING was already true, skipping");
        return Ok(());
    }

    let preview: String = text.chars().take(60).collect();

    debug_log!("paste_text writing to clipboard...");
    if let Err(e) = app.clipboard().write_text(text) {
        debug_log!("paste_text write_text failed: {}", e);
        PASTING.store(false, Ordering::SeqCst);
        return Err(e.to_string());
    }
    debug_log!("paste_text write_text OK");

    // Emit toast notification to the toast window
    emit_toast(&app, "text", preview);

    // Sync monitor cache so the clipboard poller doesn't re-record our own paste
    crate::clipboard::sync_monitor_cache(&app);

    let handle = app.clone();
    debug_log!("paste_text spawning paste thread...");
    if std::thread::Builder::new().spawn(move || {
        let _guard = PasteGuard;
        paste_with_defocus(&handle).ok();
    }).is_err() {
        PASTING.store(false, Ordering::SeqCst);
    }

    debug_log!("paste_text returning Ok");
    Ok(())
}

#[tauri::command]
pub fn paste_image(app: AppHandle, path: String) -> Result<(), String> {
    if PASTING.swap(true, Ordering::SeqCst) {
        return Ok(());
    }

    let file_name = std::path::Path::new(&path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("image")
        .to_string();
    emit_toast(&app, "image", file_name);

    let handle = app.clone();
    if std::thread::Builder::new().spawn(move || {
        let _guard = PasteGuard;

        let (rgba, w, h, png) = {
            let cache = get_image_cache().lock().unwrap_or_else(|e| e.into_inner());
            if let Some(cached) = cache.map.get(&path) {
                (cached.rgba.clone(), cached.width, cached.height, cached.png_bytes.clone())
            } else {
                drop(cache);

                let mut base_dir = crate::db::get_storage_dir(&handle);
                base_dir.push(&path);

                let bytes = match std::fs::read(&base_dir) {
                    Ok(b) => b,
                    Err(e) => { log::error!("paste_image: read error: {}", e); return; }
                };

                let png_arc = Arc::new(bytes.clone());

                let (rgba, w, h) = {
                    use image::ImageDecoder;
                    let decoder = match image::codecs::png::PngDecoder::new(std::io::Cursor::new(&bytes)) {
                        Ok(d) => d,
                        Err(e) => { log::error!("paste_image: decode error: {}", e); return; }
                    };
                    let dims = decoder.dimensions();
                    let mut buf = vec![0; (dims.0 * dims.1 * 4) as usize];
                    if let Err(e) = decoder.read_image(&mut buf) {
                        log::error!("paste_image: read pixels error: {}", e); return;
                    }
                    (buf, dims.0, dims.1)
                };

                cache_image(path.clone(), rgba.clone(), w, h, bytes);
                (Arc::new(rgba), w, h, png_arc)
            }
        };

        #[cfg(target_os = "windows")]
        {
            if let Err(e) = write_image_to_clipboard(&rgba, w, h, &png) {
                log::error!("paste_image: write clipboard error: {}", e); return;
            }
        }

        #[cfg(target_os = "macos")]
        {
            if let Err(e) = write_image_to_clipboard_macos(&png) {
                log::error!("paste_image: write clipboard error: {}", e); return;
            }
        }
        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        {
            let tauri_img = tauri::image::Image::new_owned(rgba.to_vec(), w, h);
            if let Err(e) = handle.clipboard().write_image(&tauri_img) {
                log::error!("paste_image: write clipboard error: {}", e); return;
            }
        }

        crate::clipboard::sync_monitor_cache(&handle);
        paste_with_defocus(&handle).ok();
    }).is_err() {
        PASTING.store(false, Ordering::SeqCst);
    }

    Ok(())
}

#[tauri::command]
pub fn paste_file(app: AppHandle, path: String) -> Result<(), String> {
    if PASTING.swap(true, Ordering::SeqCst) {
        return Ok(());
    }

    // Verify the file still exists on disk before pasting
    let file_meta = std::fs::metadata(&path);
    if file_meta.is_err() {
        log::error!("paste_file: file not found: {}", path);
        PASTING.store(false, Ordering::SeqCst);
        return Err(format!("File not found: {}", path));
    }

    let file_name = std::path::Path::new(&path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(&path)
        .to_string();
    emit_toast(&app, "file", file_name);

    let handle = app.clone();
    if std::thread::Builder::new().spawn(move || {
        let _guard = PasteGuard;

        #[cfg(target_os = "windows")]
        {
            if let Err(e) = write_files_to_clipboard(&[path]) {
                log::error!("paste_file: write clipboard error: {}", e);
                return;
            }
        }

        #[cfg(target_os = "macos")]
        {
            if let Err(e) = write_files_to_clipboard_macos(&[path]) {
                log::error!("paste_file: write clipboard error: {}", e);
                return;
            }
        }
        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        {
            if let Err(e) = handle.clipboard().write_text(&path) {
                log::error!("paste_file: write clipboard error: {}", e);
                return;
            }
        }

        crate::clipboard::sync_monitor_cache(&handle);
        paste_with_defocus(&handle).ok();
    }).is_err() {
        PASTING.store(false, Ordering::SeqCst);
    }

    Ok(())
}
