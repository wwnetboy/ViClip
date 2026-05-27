mod clipboard;
mod db;
mod paste;
mod preview_lock;
mod shortcut;
mod translator;
mod tray;
mod updater;

use serde::Serialize;
use tauri::Manager;
use tauri_plugin_autostart::ManagerExt;

#[derive(Serialize)]
struct AppInfo {
    name: &'static str,
    version: &'static str,
    author: &'static str,
    copyright: String,
}

#[tauri::command]
fn open_url(url: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/c", "start", "", &url])
            .spawn()
            .map_err(|e| format!("Failed to open URL: {}", e))?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&url)
            .spawn()
            .map_err(|e| format!("Failed to open URL: {}", e))?;
    }
    Ok(())
}

#[tauri::command]
fn open_file_location(path: String) -> Result<(), String> {
    let exists = std::path::Path::new(&path).exists();
    if !exists {
        return Err(format!("File not found: {}", path));
    }
    #[cfg(target_os = "windows")]
    {
        // Canonicalize to get absolute path with backslashes for explorer
        let abs = std::path::Path::new(&path)
            .canonicalize()
            .unwrap_or_else(|_| std::path::PathBuf::from(&path));
        std::process::Command::new("explorer")
            .args(["/select,", &abs.to_string_lossy()])
            .spawn()
            .map_err(|e| format!("Failed to open file location: {}", e))?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .args(["-R", &path])
            .spawn()
            .map_err(|e| format!("Failed to open file location: {}", e))?;
    }
    Ok(())
}

#[tauri::command]
fn apply_preview_backdrop(app: tauri::AppHandle, window_label: String) {
    #[cfg(target_os = "windows")]
    if let Some(window) = app.get_webview_window(&window_label) {
        apply_backdrop_effect(&window, true);
    }
    let _ = (app, window_label);
}

#[tauri::command]
fn get_app_info() -> AppInfo {
    AppInfo {
        name: "ViClip",
        version: env!("CARGO_PKG_VERSION"),
        author: "wwnetboy",
        copyright: "wwnetboy. All rights reserved.".to_string(),
    }
}

// Undocumented but stable API for acrylic blur on Windows 10.
// SetWindowCompositionAttribute is exported by user32.dll since Windows 10.
#[repr(C)]
struct AccentPolicy {
    accent_state: i32,
    accent_flags: i32,
    gradient_color: i32, // ABGR format
    animation_id: i32,
}

#[repr(C)]
struct WindowCompositionAttribData {
    attrib: i32,
    pv_data: *const AccentPolicy,
    cb_data: i32,
}

extern "system" {
    fn SetWindowCompositionAttribute(
        hwnd: isize,
        data: *const WindowCompositionAttribData,
    ) -> i32;
}

const WCA_ACCENT_POLICY: i32 = 19;
const ACCENT_ENABLE_BLURBEHIND: i32 = 3;

#[cfg(target_os = "windows")]
fn apply_backdrop_effect(window: &tauri::WebviewWindow, _use_win10_fallback: bool) -> bool {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::Graphics::Dwm::{
        DwmSetWindowAttribute,
        DWMWA_SYSTEMBACKDROP_TYPE, DWMWA_WINDOW_CORNER_PREFERENCE,
    };

    let hwnd = window.hwnd().unwrap_or_default();
    if hwnd.is_invalid() {
        return false;
    }

    let hwnd = HWND(hwnd.0);

    // Windows 11: use system backdrop (DWMSBT_TABBEDWINDOW = 3, Mica Alt)
    let backdrop_type: i32 = 3;
    let backdrop_result = unsafe {
        DwmSetWindowAttribute(
            hwnd,
            DWMWA_SYSTEMBACKDROP_TYPE,
            &backdrop_type as *const i32 as *const _,
            std::mem::size_of::<i32>() as u32,
        )
    };
    let is_win11 = backdrop_result.is_ok();

    // Always apply Win10 blur-behind for persistent transparency
    // (Mica Alt loses transparency on focus loss; blur-behind stays consistent)
    apply_win10_blur_behind(hwnd);

    // Rounded corners (Windows 11 only, fails silently on Windows 10)
    let corner_preference: i32 = 2; // DWMWCP_ROUND
    unsafe {
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_WINDOW_CORNER_PREFERENCE,
            &corner_preference as *const i32 as *const _,
            std::mem::size_of::<i32>() as u32,
        );
    }

    is_win11
}

#[cfg(target_os = "windows")]
fn apply_win10_blur_behind(hwnd: windows::Win32::Foundation::HWND) {
    let accent = AccentPolicy {
        accent_state: ACCENT_ENABLE_BLURBEHIND,
        accent_flags: 0,
        gradient_color: 0,
        animation_id: 0,
    };
    let data = WindowCompositionAttribData {
        attrib: WCA_ACCENT_POLICY,
        pv_data: &accent,
        cb_data: std::mem::size_of::<AccentPolicy>() as i32,
    };
    unsafe {
        if SetWindowCompositionAttribute(hwnd.0 as isize, &data) == 0 {
            log::warn!("SetWindowCompositionAttribute (Win10 blur-behind) failed");
        }
    }
}

/// Build a WebviewWindow with page-load retry logic and a timeout watchdog.
/// Returns the built window for further post-build customization.
fn build_window_with_retry<M: tauri::Manager<tauri::Wry>>(
    _app: &tauri::App,
    builder: tauri::WebviewWindowBuilder<'_, tauri::Wry, M>,
    label: &str,
) -> Result<tauri::WebviewWindow, Box<dyn std::error::Error>> {
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::Arc;

    const MAX_RETRIES: usize = 3;
    let retry_count = Arc::new(AtomicUsize::new(0));
    let page_loaded = Arc::new(AtomicBool::new(false));
    let retry_for_handler = retry_count.clone();
    let loaded_for_handler = page_loaded.clone();
    let retry_for_timeout = retry_count.clone();
    let loaded_for_timeout = page_loaded.clone();
    let label_for_log = label.to_string();

    let window = builder
        .on_page_load(move |window, payload| {
            use tauri::webview::PageLoadEvent;
            if let PageLoadEvent::Finished = payload.event() {
                let url = payload.url().to_string();
                if url.starts_with("chrome-error://") {
                    let attempt = retry_for_handler.fetch_add(1, Ordering::SeqCst);
                    if attempt < MAX_RETRIES {
                        log::warn!(
                            "{} window error page (attempt {}/{}); retrying after delay...",
                            label_for_log, attempt + 1, MAX_RETRIES
                        );
                        let w = window.clone();
                        std::thread::spawn(move || {
                            std::thread::sleep(std::time::Duration::from_secs(2));
                            let _ = w.reload();
                        });
                    } else {
                        log::error!(
                            "{} window failed to load after {} retries; giving up",
                            label_for_log, MAX_RETRIES
                        );
                    }
                } else {
                    loaded_for_handler.store(true, Ordering::SeqCst);
                }
            }
        })
        .build()?;

    // Timeout watchdog
    let w = window.clone();
    let lbl = label.to_string();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_secs(10));
        if !loaded_for_timeout.load(Ordering::SeqCst) {
            let attempt = retry_for_timeout.fetch_add(1, Ordering::SeqCst);
            if attempt < MAX_RETRIES {
                log::warn!(
                    "{} window load timed out (attempt {}/{}); reloading...",
                    lbl, attempt + 1, MAX_RETRIES
                );
                let _ = w.reload();
            } else {
                log::error!(
                    "{} window load timed out after {} retries; giving up",
                    lbl, MAX_RETRIES
                );
            }
        }
    });

    Ok(window)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_autostart::init(tauri_plugin_autostart::MacosLauncher::LaunchAgent, Some(vec!["--hidden"])))
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                        shortcut::toggle_window(app);
                    }
                })
                .build(),
        )
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let is_autostart = std::env::args().any(|a| a == "--hidden");

            // On auto-start (cold boot), delay to give WebView2 time to
            // initialize before creating any windows.
            if is_autostart {
                std::thread::sleep(std::time::Duration::from_secs(3));
            }

            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // Create main window programmatically (not from config) so it
            // is created AFTER asset protocol setup, avoiding a cold-boot
            // race where WebView2 navigates before the protocol is ready.
            let main_window = build_window_with_retry(
                app,
                tauri::WebviewWindowBuilder::new(
                    app,
                    "main",
                    tauri::WebviewUrl::App("index.html".into()),
                )
                .title("ViClip")
                .inner_size(520.0, 600.0)
                .min_inner_size(440.0, 420.0)
                .decorations(false)
                .transparent(true)
                .visible(false)
                .center()
                .shadow(false)
                .resizable(true),
                "main",
            )?;

            #[cfg(target_os = "windows")]
            {
                let window = &main_window;
                let _ = window.set_background_color(Some(tauri::window::Color(0, 0, 0, 0)));

        // Remove window border styles that create a 1px line on Win10.
        // These styles are added by Tauri for resizable undecorated windows.
        {
            use windows::Win32::Foundation::HWND;
            use windows::Win32::UI::WindowsAndMessaging::{
                GetWindowLongW, SetWindowLongW, SetWindowPos,
                GWL_STYLE, GWL_EXSTYLE,
                WS_THICKFRAME, WS_BORDER, WS_DLGFRAME,
                WS_EX_CLIENTEDGE, WS_EX_STATICEDGE, WS_EX_WINDOWEDGE,
                SWP_FRAMECHANGED, SWP_NOMOVE, SWP_NOZORDER, SWP_NOSIZE,
            };
            let hwnd = HWND(window.hwnd().unwrap().0);

            let style = unsafe { GetWindowLongW(hwnd, GWL_STYLE) };
            let remove_style = WS_THICKFRAME.0 as i32 | WS_BORDER.0 as i32 | WS_DLGFRAME.0 as i32;
            let new_style = style & !remove_style;
            if new_style != style {
                unsafe { SetWindowLongW(hwnd, GWL_STYLE, new_style) };
            }

            let ex_style = unsafe { GetWindowLongW(hwnd, GWL_EXSTYLE) };
            let remove_ex = WS_EX_CLIENTEDGE.0 as i32 | WS_EX_STATICEDGE.0 as i32 | WS_EX_WINDOWEDGE.0 as i32;
            let new_ex = ex_style & !remove_ex;
            if new_ex != ex_style {
                unsafe { SetWindowLongW(hwnd, GWL_EXSTYLE, new_ex) };
            }

            if new_style != style || new_ex != ex_style {
                unsafe {
                    let _ = SetWindowPos(
                        hwnd,
                        HWND(std::ptr::null_mut()),
                        0, 0, 0, 0,
                        SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_FRAMECHANGED,
                    );
                }
                log::info!("Removed window border styles from main window");
            }
        }

        // Always apply blur-behind for persistent transparency across
        // focus changes. Mica Alt is also attempted for Win11 rounded corners.
        // Rounded corners on Win10 are handled via CSS border-radius and
        // the inset box-shadow window border.
        apply_backdrop_effect(window, false);
            }

            db::init_db(app.handle())?;
            db::prune_old_records(app.handle()).ok();

            // Autostart: enable on first run (default ON), repair --hidden arg on existing installs.
            // Uses a DB marker so we don't re-enable after the user explicitly disables it.
            let autostart = app.autolaunch();
            let autostart_initialized = db::get_setting(app.handle().clone(), "autostart_initialized".to_string())
                .map(|v| v == "1")
                .unwrap_or(false);
            if autostart.is_enabled().unwrap_or(false) {
                // Repair existing autostart entry (ensure --hidden arg)
                if let Err(e) = autostart.enable() {
                    log::warn!("Failed to repair autostart entry: {}", e);
                }
            } else if !autostart_initialized {
                // First run: enable autostart by default
                // Only mark as initialized if enable() actually succeeds,
                // so we retry on the next launch if it fails silently.
                match autostart.enable() {
                    Ok(_) => {
                        let _ = db::set_setting(app.handle().clone(), "autostart_initialized".to_string(), "1".to_string());
                    }
                    Err(e) => {
                        log::error!("Failed to enable autostart: {}", e);
                    }
                }
            }

            // Periodic pruning every hour
            let prune_handle = app.handle().clone();
            std::thread::spawn(move || loop {
                std::thread::sleep(std::time::Duration::from_secs(3600));
                db::prune_old_records(&prune_handle).ok();
            });

            clipboard::start_monitor(app.handle())?;

            app.handle().manage(tray::TrayState { tray: std::sync::Mutex::new(None) });
            tray::create_tray(app.handle())?;

            shortcut::install_mouse_hook(app.handle());

            // Create hidden radial menu popup window
            {
                let radial = build_window_with_retry(
                    app,
                    tauri::WebviewWindowBuilder::new(
                        app,
                        "radial-menu",
                        tauri::WebviewUrl::App("index.html?radial=1".into()),
                    )
                    .title("")
                    .inner_size(300.0, 420.0)
                    .decorations(false)
                    .transparent(true)
                    .always_on_top(true)
                    .visible(false)
                    .shadow(false)
                    .skip_taskbar(true)
                    .resizable(false),
                    "radial-menu",
                )?;

                let _ = radial.set_background_color(Some(tauri::window::Color(0, 0, 0, 0)));
                #[cfg(target_os = "windows")]
                apply_backdrop_effect(&radial, true);

                log::info!("Radial menu popup window created");
            }

            // Create toast notification window (bottom-right, always-on-top, transparent)
            {
                let toast = build_window_with_retry(
                    app,
                    tauri::WebviewWindowBuilder::new(
                        app,
                        "toast",
                        tauri::WebviewUrl::App("index.html?toast=1".into()),
                    )
                    .title("")
                    .inner_size(320.0, 80.0)
                    .decorations(false)
                    .transparent(true)
                    .always_on_top(true)
                    .visible(true)
                    .shadow(false)
                    .skip_taskbar(true)
                    .resizable(false)
                    .focused(false),
                    "toast",
                )?;

                let _ = toast.set_background_color(Some(tauri::window::Color(0, 0, 0, 0)));
                let _ = toast.set_ignore_cursor_events(true);

                // Position at bottom-right of primary monitor, above the taskbar
                #[cfg(target_os = "windows")]
                {
                    use windows::Win32::UI::WindowsAndMessaging::{SystemParametersInfoW, SPI_GETWORKAREA, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS};
                    let mut rect = windows::Win32::Foundation::RECT::default();
                    let result = unsafe {
                        SystemParametersInfoW(
                            SPI_GETWORKAREA,
                            0,
                            Some(&mut rect as *mut _ as *mut core::ffi::c_void),
                            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
                        )
                    };
                    if result.is_ok() {
                        let scale = toast.scale_factor().unwrap_or(1.0);
                        let window_w = 320.0;
                        let window_h = 80.0;
                        let margin = 12.0;
                        let x = (rect.right as f64 / scale - window_w - margin) as f64;
                        let y = (rect.bottom as f64 / scale - window_h - margin) as f64;
                        let _ = toast.set_position(tauri::PhysicalPosition::new(x, y));
                    }
                }

                log::info!("Toast window created");
            }

            if let Ok(key) = db::get_setting(app.handle().clone(), "shortcut_key".to_string()) {
                let shortcut = if key.is_empty() { "Alt+V".to_string() } else { key };
                if shortcut.starts_with("Super+") {
                    shortcut::install_keyboard_hook();
                } else if let Err(e) = shortcut::register_keyboard_shortcut(app.handle(), &shortcut) {
                    log::warn!("Failed to register keyboard shortcut '{}': {}", shortcut, e);
                }
            }

            // Show main window when not auto-started and minimize_to_tray is off
            if !is_autostart {
                let minimize_to_tray = db::get_setting(app.handle().clone(), "minimize_to_tray".to_string())
                    .map(|v| v == "1")
                    .unwrap_or(false);
                if !minimize_to_tray {
                    let _ = main_window.show();
                    let _ = main_window.set_focus();
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            db::get_clipboard_records,
            db::delete_clipboard_record,
            db::get_phrase_groups,
            db::create_phrase_group,
            db::update_phrase_group,
            db::delete_phrase_group,
            db::get_phrases,
            db::create_phrase,
            db::update_phrase,
            db::delete_phrase,
            db::get_translation_history,
            db::clear_translation_history,
            db::clear_all_records,
            db::get_setting,
            db::get_all_settings,
            db::set_setting,
            db::set_settings_batch,
            paste::paste_text,
            paste::paste_image,
            paste::paste_file,
            db::get_image_base64,
            db::get_image_thumbnail,
            db::ensure_thumbnail,
            get_app_info,
            db::get_storage_path,
            db::select_storage_folder,
            translator::translate,
            shortcut::update_shortcut,
            shortcut::set_radial_menu_enabled,
            tray::update_tray_language,
            preview_lock::set_preview_aspect_ratio,
            apply_preview_backdrop,
            open_url,
            open_file_location,
            updater::check_update,
            updater::download_and_install_update,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
