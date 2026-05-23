mod clipboard;
mod db;
mod paste;
mod preview_lock;
mod shortcut;
mod translator;
mod tray;

use std::collections::HashMap;
use std::sync::Mutex;
use serde::Serialize;
use tauri::Manager;
use uuid::Uuid;
use tauri_plugin_autostart::ManagerExt;

#[derive(Serialize)]
struct AppInfo {
    name: &'static str,
    version: &'static str,
    author: &'static str,
    copyright: &'static str,
}

struct PreviewImageStore(Mutex<HashMap<String, String>>);

#[tauri::command]
fn store_preview_image(state: tauri::State<'_, PreviewImageStore>, base64: String) -> String {
    let token = Uuid::new_v4().to_string();
    state.0.lock().unwrap().insert(token.clone(), base64);
    token
}

#[tauri::command]
fn fetch_preview_image(state: tauri::State<'_, PreviewImageStore>, token: String) -> Option<String> {
    state.0.lock().unwrap().remove(&token)
}

#[tauri::command]
fn apply_preview_backdrop(app: tauri::AppHandle, window_label: String) {
    #[cfg(target_os = "windows")]
    if let Some(window) = app.get_webview_window(&window_label) {
        apply_backdrop_effect(&window);
    }
    let _ = (app, window_label);
}

#[tauri::command]
fn get_app_info() -> AppInfo {
    AppInfo {
        name: "ViClip",
        version: env!("CARGO_PKG_VERSION"),
        author: "wwnetboy",
        copyright: "2025 wwnetboy. All rights reserved.",
    }
}

#[cfg(target_os = "windows")]
fn apply_backdrop_effect(window: &tauri::WebviewWindow) {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::Graphics::Dwm::{DwmSetWindowAttribute, DWMWA_SYSTEMBACKDROP_TYPE, DWMWA_WINDOW_CORNER_PREFERENCE};

    let hwnd = window.hwnd().unwrap_or_default();
    if hwnd.is_invalid() {
        return;
    }

    let hwnd = HWND(hwnd.0);

    let backdrop_type: i32 = 3;
    let result = unsafe {
        DwmSetWindowAttribute(
            hwnd,
            DWMWA_SYSTEMBACKDROP_TYPE,
            &backdrop_type as *const i32 as *const _,
            std::mem::size_of::<i32>() as u32,
        )
    };

    if let Err(e) = result {
        log::warn!("Failed to set DWM backdrop type: {:?}", e);
    }

    let corner_preference: i32 = 2; // DWMWCP_ROUND
    let result = unsafe {
        DwmSetWindowAttribute(
            hwnd,
            DWMWA_WINDOW_CORNER_PREFERENCE,
            &corner_preference as *const i32 as *const _,
            std::mem::size_of::<i32>() as u32,
        )
    };

    if let Err(e) = result {
        log::warn!("Failed to set DWM corner preference: {:?}", e);
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(PreviewImageStore(Mutex::new(HashMap::new())))
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
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            #[cfg(target_os = "windows")]
            {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.set_background_color(Some(tauri::window::Color(0, 0, 0, 0)));
                    apply_backdrop_effect(&window);
                }
            }

            let is_autostart = std::env::args().any(|a| a == "--hidden");

            db::init_db(app.handle())?;
            db::prune_old_records(app.handle()).ok();

            // Always start with light theme
            let _ = db::set_setting(app.handle().clone(), "theme".to_string(), "light".to_string());

            // Repair autostart registry entry to ensure --hidden arg is present
            let autostart = app.autolaunch();
            if autostart.is_enabled().unwrap_or(false) {
                let _ = autostart.enable();
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
                use tauri::WebviewWindowBuilder;
                use tauri::WebviewUrl;
                let radial = WebviewWindowBuilder::new(
                    app,
                    "radial-menu",
                    WebviewUrl::App("index.html?radial=1".into()),
                )
                .title("")
                .inner_size(300.0, 420.0)
                .decorations(false)
                .transparent(true)
                .always_on_top(true)
                .visible(false)
                .shadow(false)
                .skip_taskbar(true)
                .resizable(false)
                .build()?;
                let _ = radial.set_background_color(Some(tauri::window::Color(0, 0, 0, 0)));
                #[cfg(target_os = "windows")]
                apply_backdrop_effect(&radial);
                log::info!("Radial menu popup window created");
            }

            // Create toast notification window (bottom-right, always-on-top, transparent)
            {
                use tauri::WebviewWindowBuilder;
                use tauri::WebviewUrl;
                let toast = WebviewWindowBuilder::new(
                    app,
                    "toast",
                    WebviewUrl::App("index.html?toast=1".into()),
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
                .focused(false)
                .build()?;
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
                if !key.is_empty() {
                    if key.starts_with("Super+") {
                        shortcut::install_keyboard_hook();
                    } else if let Err(e) = shortcut::register_keyboard_shortcut(app.handle(), &key) {
                        log::warn!("Failed to register keyboard shortcut '{}': {}", key, e);
                    }
                }
            }

            // Show main window when not auto-started and minimize_to_tray is off
            if !is_autostart {
                let minimize_to_tray = db::get_setting(app.handle().clone(), "minimize_to_tray".to_string())
                    .map(|v| v == "1")
                    .unwrap_or(false);
                if !minimize_to_tray {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                    }
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
            store_preview_image,
            fetch_preview_image,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
