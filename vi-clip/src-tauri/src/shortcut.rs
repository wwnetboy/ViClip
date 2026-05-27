use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};
use std::sync::OnceLock;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::GlobalShortcutExt;

static RADIAL_MENU_ENABLED: AtomicBool = AtomicBool::new(true);

#[cfg(target_os = "windows")]
use windows::Win32::Foundation::*;
#[cfg(target_os = "windows")]
use windows::Win32::UI::Input::KeyboardAndMouse::*;
#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::*;

#[cfg(target_os = "windows")]
static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();
#[cfg(target_os = "windows")]
static HOOK_HANDLE: AtomicPtr<core::ffi::c_void> = AtomicPtr::new(core::ptr::null_mut());
#[cfg(target_os = "windows")]
static KB_HOOK_HANDLE: AtomicPtr<core::ffi::c_void> = AtomicPtr::new(core::ptr::null_mut());

static TOGGLING: AtomicBool = AtomicBool::new(false);

#[cfg(target_os = "windows")]
static RADIAL_JUST_SHOWN: AtomicBool = AtomicBool::new(false);

#[cfg(target_os = "windows")]
static WIN_KEY_DOWN: AtomicBool = AtomicBool::new(false);
#[cfg(target_os = "windows")]
static WIN_KEY_USED: AtomicBool = AtomicBool::new(false);

/// RAII guard that ensures TOGGLING is always reset, even on panic.
struct ToggleGuard;

impl Drop for ToggleGuard {
    fn drop(&mut self) {
        TOGGLING.store(false, Ordering::SeqCst);
    }
}

#[derive(serde::Serialize, Clone)]
struct RadialMenuDownPayload {
    theme: String,
}

pub fn toggle_window(app: &AppHandle) {
    if TOGGLING.swap(true, Ordering::SeqCst) {
        log::info!("[toggle_window] skipped (re-entrant)");
        return;
    }
    let _guard = ToggleGuard;

    if let Some(window) = app.get_webview_window("main") {
        let visible = window.is_visible().unwrap_or(false);
        log::info!("[toggle_window] visible={}", visible);

        if visible {
            log::info!("[toggle_window] hiding window");
            let _ = window.hide();
        } else {
            #[cfg(target_os = "windows")]
            {
                crate::paste::save_foreground_window();
                // Allow our own process (or any process) to call SetForegroundWindow.
                // The thread has temporary foreground permission from the hotkey / hook
                // input, so this ASFW call makes SetForegroundWindow bulletproof.
                unsafe {
                    use windows::Win32::UI::WindowsAndMessaging::AllowSetForegroundWindow;
                    let _ = AllowSetForegroundWindow(0xFFFFFFFF);
                }
            }

            log::info!("[toggle_window] showing window");
            let _ = window.show();
            let _ = window.set_focus();
        }
    } else {
        log::warn!("[toggle_window] main window not found");
    }
}

#[cfg(target_os = "windows")]
unsafe extern "system" fn mouse_hook_callback(
    n_code: i32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if n_code >= 0 {
        let msg = w_param.0 as u32;

        if msg == WM_RBUTTONDOWN {
            let ctrl = (GetAsyncKeyState(VK_CONTROL.0 as i32) as u16) & 0x8000 != 0;
            let shift = (GetAsyncKeyState(VK_SHIFT.0 as i32) as u16) & 0x8000 != 0;
            let alt = (GetAsyncKeyState(VK_MENU.0 as i32) as u16) & 0x8000 != 0;

            if ctrl && shift {
                if let Some(app) = APP_HANDLE.get() {
                    toggle_window(app);
                }
                return LRESULT(1);
            }

            if ctrl && alt && !shift {
                if !RADIAL_MENU_ENABLED.load(Ordering::SeqCst) {
                    let hook = HHOOK(HOOK_HANDLE.load(Ordering::SeqCst));
                    return unsafe { CallNextHookEx(hook, n_code, w_param, l_param) };
                }
                if let Some(app) = APP_HANDLE.get() {
                    if let Some(window) = app.get_webview_window("radial-menu") {
                        let visible = window.is_visible().unwrap_or(false);
                        if visible {
                            let _ = window.hide();
                        } else {
                            crate::paste::save_foreground_window();

                            // Allow this process to call SetForegroundWindow
                            unsafe {
                                use windows::Win32::UI::WindowsAndMessaging::AllowSetForegroundWindow;
                                let _ = AllowSetForegroundWindow(0xFFFFFFFF);
                            }

                            let hook_struct = &*(l_param.0 as *const MSLLHOOKSTRUCT);
                            let sx = hook_struct.pt.x;
                            let sy = hook_struct.pt.y;

                            // Position window at mouse bottom-right (like Windows context menu)
                            let _ = window.set_position(tauri::Position::Physical(
                                tauri::PhysicalPosition::new(sx, sy),
                            ));

                            let theme = crate::db::get_setting(app.clone(), "theme".to_string())
                                .unwrap_or_else(|_| "light".to_string());

                            log::info!("radial-menu-down: screen=({}, {}), theme={}", sx, sy, theme);
                            let _ = app.emit(
                                "radial-menu-down",
                                RadialMenuDownPayload { theme },
                            );

                            RADIAL_JUST_SHOWN.store(true, Ordering::SeqCst);
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                }
                return LRESULT(1);
            }
        }

        if msg == WM_RBUTTONUP && RADIAL_JUST_SHOWN.swap(false, Ordering::SeqCst) {
            return LRESULT(1);
        }
    }

    let hook = HHOOK(HOOK_HANDLE.load(Ordering::SeqCst));
    unsafe { CallNextHookEx(hook, n_code, w_param, l_param) }
}

#[cfg(target_os = "windows")]
unsafe extern "system" fn keyboard_hook_callback(
    n_code: i32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if n_code >= 0 {
        let vk_code = (l_param.0 as *const KBDLLHOOKSTRUCT).as_ref().map(|s| s.vkCode).unwrap_or(0);

        let is_win = vk_code == VK_LWIN.0 as u32 || vk_code == VK_RWIN.0 as u32;
        let is_v = vk_code == 0x56; // 'V'

        let is_keydown = w_param.0 == WM_KEYDOWN as usize || w_param.0 == WM_SYSKEYDOWN as usize;
        let is_keyup = w_param.0 == WM_KEYUP as usize || w_param.0 == WM_SYSKEYUP as usize;

        if is_keydown && is_win {
            WIN_KEY_DOWN.store(true, Ordering::SeqCst);
            WIN_KEY_USED.store(false, Ordering::SeqCst);
            // Pass through so other Win+key combos still work
        }

        if is_keydown && is_v && WIN_KEY_DOWN.load(Ordering::SeqCst) {
            WIN_KEY_USED.store(true, Ordering::SeqCst);
            if let Some(app) = APP_HANDLE.get() {
                toggle_window(app);
            }
            return LRESULT(1); // consume the V keydown
        }

        if is_keyup && is_win {
            let was_used = WIN_KEY_USED.swap(false, Ordering::SeqCst);
            WIN_KEY_DOWN.store(false, Ordering::SeqCst);
            if was_used {
                // Suppress the Start menu but inject a synthetic Win keyup
                // so the system doesn't think Win is stuck pressed.
                let inputs = [
                    INPUT {
                        r#type: INPUT_KEYBOARD,
                        Anonymous: INPUT_0 {
                            ki: KEYBDINPUT {
                                wVk: VK_LWIN,
                                wScan: 0,
                                dwFlags: KEYEVENTF_KEYUP,
                                time: 0,
                                dwExtraInfo: 0,
                            },
                        },
                    },
                ];
                SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
                return LRESULT(1);
            }
        }
    }

    let hook = HHOOK(KB_HOOK_HANDLE.load(Ordering::SeqCst));
    unsafe { CallNextHookEx(hook, n_code, w_param, l_param) }
}

pub fn install_mouse_hook(app: &AppHandle) {
    #[cfg(target_os = "windows")]
    {
        // Restore persisted radial menu enabled state
        if let Ok(val) = crate::db::get_setting(app.clone(), "radial_menu_enabled".to_string()) {
            RADIAL_MENU_ENABLED.store(val == "1", Ordering::SeqCst);
        }

        APP_HANDLE.set(app.clone()).ok();
        let hook = unsafe {
            SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_hook_callback), None, 0)
        };
        if let Ok(h) = hook {
            HOOK_HANDLE.store(h.0, Ordering::SeqCst);
            log::info!("Global mouse hook installed (Ctrl+Shift+RightClick / Ctrl+Alt+RightClick)");
        } else {
            log::warn!("Failed to install mouse hook");
        }
    }
}

#[cfg(target_os = "windows")]
pub fn install_keyboard_hook() {
    let hook = unsafe {
        SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_hook_callback), None, 0)
    };
    if let Ok(h) = hook {
        KB_HOOK_HANDLE.store(h.0, Ordering::SeqCst);
        log::info!("Win+V keyboard hook installed");
    } else {
        log::warn!("Failed to install Win+V keyboard hook");
    }
}

#[cfg(target_os = "windows")]
pub fn uninstall_keyboard_hook() {
    let handle = KB_HOOK_HANDLE.swap(core::ptr::null_mut(), Ordering::SeqCst);
    if !handle.is_null() {
        unsafe {
            let hhook = HHOOK(handle);
            let _ = UnhookWindowsHookEx(hhook);
        }
        log::info!("Win+V keyboard hook uninstalled");
    }
}

#[cfg(not(target_os = "windows"))]
pub fn install_keyboard_hook() {}

#[cfg(not(target_os = "windows"))]
pub fn uninstall_keyboard_hook() {}

pub fn register_keyboard_shortcut(
    app: &AppHandle,
    shortcut: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if shortcut.is_empty() {
        return Ok(());
    }
    app.global_shortcut().register(shortcut)?;
    Ok(())
}

pub fn unregister_keyboard_shortcut(
    app: &AppHandle,
    shortcut: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if shortcut.is_empty() {
        return Ok(());
    }
    let _ = app.global_shortcut().unregister(shortcut);
    Ok(())
}

#[tauri::command]
pub fn update_shortcut(
    app: AppHandle,
    old_shortcut: String,
    new_shortcut: String,
) -> Result<(), String> {
    let new_key = if new_shortcut.is_empty() { "Alt+V".to_string() } else { new_shortcut };

    if !old_shortcut.is_empty() && old_shortcut != new_key {
        let _ = unregister_keyboard_shortcut(&app, &old_shortcut);
    }

    // Toggle keyboard hook for Win-key shortcuts
    #[cfg(target_os = "windows")]
    {
        let old_is_win = old_shortcut.starts_with("Super+");
        let new_is_win = new_key.starts_with("Super+");
        if !old_is_win && new_is_win {
            install_keyboard_hook();
        } else if old_is_win && !new_is_win {
            uninstall_keyboard_hook();
        }
    }

    // Skip global-shortcut registration for Win-key combos — the keyboard hook handles them
    if !new_key.starts_with("Super+") {
        register_keyboard_shortcut(&app, &new_key)
            .map_err(|e| format!("Failed to register shortcut: {}", e))?;
    }
    Ok(())
}

#[tauri::command]
pub fn set_radial_menu_enabled(app: AppHandle, enabled: bool) -> Result<(), String> {
    RADIAL_MENU_ENABLED.store(enabled, Ordering::SeqCst);
    let state = app.state::<crate::db::DbState>();
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO settings (key, value) VALUES ('radial_menu_enabled', ?1) ON CONFLICT(key) DO UPDATE SET value = ?1",
        rusqlite::params![if enabled { "1" } else { "0" }],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

