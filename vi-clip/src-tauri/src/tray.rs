use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager};
use tauri::menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons};

pub struct TrayState {
    pub tray: Mutex<Option<tauri::tray::TrayIcon>>,
}

fn build_tray_menu(app: &AppHandle, lang: &str) -> Result<tauri::menu::Menu<tauri::Wry>, Box<dyn std::error::Error>> {
    let version = env!("CARGO_PKG_VERSION");
    let is_cn = !lang.starts_with("en");

    let (website_text, version_text, update_text, guide_text, feedback_text, prefs_text, quit_text) = if is_cn {
        (
            "ViClip官网",
            format!("版本 v{}", version),
            "检测更新",
            "使用指南",
            "意见反馈",
            "偏好设置",
            "退出应用",
        )
    } else {
        (
            "ViClip Website",
            format!("Version v{}", version),
            "Check for Updates",
            "User Guide",
            "Feedback",
            "Preferences",
            "Quit App",
        )
    };

    let website = MenuItemBuilder::with_id("website", website_text).build(app)?;
    let version_item = MenuItemBuilder::with_id("version", version_text)
        .enabled(false)
        .build(app)?;
    let update = MenuItemBuilder::with_id("check_update", update_text).build(app)?;
    let guide = MenuItemBuilder::with_id("guide", guide_text).build(app)?;
    let feedback = MenuItemBuilder::with_id("feedback", feedback_text).build(app)?;
    let sep1 = PredefinedMenuItem::separator(app)?;
    let sep2 = PredefinedMenuItem::separator(app)?;
    let sep3 = PredefinedMenuItem::separator(app)?;
    let prefs = MenuItemBuilder::with_id("preferences", prefs_text).build(app)?;
    let quit = MenuItemBuilder::with_id("quit", quit_text).build(app)?;

    MenuBuilder::new(app)
        .item(&prefs)
        .item(&sep1)
        .item(&website)
        .item(&version_item)
        .item(&update)
        .item(&sep2)
        .item(&guide)
        .item(&feedback)
        .item(&sep3)
        .item(&quit)
        .build()
        .map_err(Into::into)
}

pub fn create_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let lang = crate::db::get_setting_sync(app, "language").unwrap_or_else(|| "zh-CN".to_string());
    let menu = build_tray_menu(app, &lang)?;

    let icon_bytes = include_bytes!("../icons/icon.png");
    let img = image::load_from_memory(icon_bytes)
        .expect("Failed to decode tray icon")
        .into_rgba8();
    let (w, h) = img.dimensions();
    let icon = tauri::image::Image::new_owned(img.into_raw(), w, h);

    let tray = TrayIconBuilder::new()
        .icon(icon)
        .menu(&menu)
        .tooltip("ViClip")
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| {
            let menu_id = event.id().as_ref();
            match menu_id {
                "website" => {
                    let _ = open::that("https://github.com/wwnetboy/ViClip");
                }
                "check_update" => {
                    let app_handle = app.clone();
                    let is_cn = crate::db::get_setting_sync(&app_handle, "language")
                        .map(|l| !l.starts_with("en"))
                        .unwrap_or(true);
                    tauri::async_runtime::spawn(async move {
                        let (title, msg) = match crate::updater::check_update().await {
                            Ok(info) if info.has_update => {
                                let (t, m, ok_btn, cancel_btn, err_prefix) = if is_cn {
                                    ("ViClip 更新", format!("发现新版本 v{}（当前 v{}）。\n\n是否下载并安装？", info.latest_version, info.current_version), "下载", "取消", "下载失败")
                                } else {
                                    ("ViClip Update", format!("v{} is available (current v{}).\n\nDownload and install?", info.latest_version, info.current_version), "Download", "Cancel", "Download failed")
                                };
                                let dialog = app_handle.dialog();
                                dialog.message(m)
                                    .title(t)
                                    .buttons(MessageDialogButtons::OkCancelCustom(ok_btn.to_string(), cancel_btn.to_string()))
                                    .show(move |confirmed| {
                                        if confirmed {
                                            let handle = app_handle.clone();
                                            let url = info.download_url.clone();
                                            tauri::async_runtime::spawn(async move {
                                                if let Err(e) = crate::updater::download_and_install_update(url).await {
                                                    let d = handle.dialog();
                                                    d.message(format!("{}: {}", err_prefix, e)).title("ViClip").show(|_| {});
                                                }
                                            });
                                        }
                                    });
                                return;
                            }
                            Ok(_) => {
                                if is_cn {
                                    ("ViClip".to_string(), "已是最新版本".to_string())
                                } else {
                                    ("ViClip".to_string(), "You're up to date".to_string())
                                }
                            }
                            Err(e) => {
                                if is_cn {
                                    ("ViClip".to_string(), format!("检查更新失败：{}", e))
                                } else {
                                    ("ViClip".to_string(), format!("Update check failed: {}", e))
                                }
                            }
                        };
                        let dialog = app_handle.dialog();
                        dialog.message(msg).title(title).show(|_| {});
                    });
                }
                "guide" => {
                    let _ = open::that("https://github.com/wwnetboy/ViClip/wiki");
                }
                "feedback" => {
                    let _ = open::that("https://github.com/wwnetboy/ViClip/issues");
                }
                "preferences" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = app.emit("navigate-panel", "settings");
                        window.show().ok();
                        window.set_focus().ok();
                    }
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let tauri::tray::TrayIconEvent::Click { button, button_state, .. } = event {
                if button_state != tauri::tray::MouseButtonState::Down {
                    return;
                }
                if button == tauri::tray::MouseButton::Left {
                    let app = tray.app_handle();
                    if let Some(window) = app.get_webview_window("main") {
                        if window.is_visible().unwrap_or(false) {
                            window.hide().ok();
                        } else {
                            #[cfg(target_os = "windows")]
                            crate::paste::save_foreground_window();
                            window.show().ok();
                            window.set_focus().ok();
                        }
                    }
                }
            }
        })
        .build(app)?;

    let state = app.state::<TrayState>();
    *state.tray.lock().unwrap() = Some(tray);

    Ok(())
}

#[tauri::command]
pub fn update_tray_language(app: AppHandle) -> Result<(), String> {
    let lang = crate::db::get_setting_sync(&app, "language").unwrap_or_else(|| "zh-CN".to_string());
    let menu = build_tray_menu(&app, &lang).map_err(|e| e.to_string())?;

    let state = app.state::<TrayState>();
    let tray_guard = state.tray.lock().map_err(|e| e.to_string())?;
    if let Some(tray) = tray_guard.as_ref() {
        tray.set_menu(Some(menu)).map_err(|e| e.to_string())?;
    }

    Ok(())
}
