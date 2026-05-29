use std::sync::atomic::{AtomicBool, AtomicIsize, AtomicPtr, Ordering};
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

pub(crate) static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();
#[cfg(target_os = "windows")]
static HOOK_HANDLE: AtomicPtr<core::ffi::c_void> = AtomicPtr::new(core::ptr::null_mut());
#[cfg(target_os = "windows")]
static KB_HOOK_HANDLE: AtomicPtr<core::ffi::c_void> = AtomicPtr::new(core::ptr::null_mut());

static TOGGLING: AtomicBool = AtomicBool::new(false);

pub(crate) static RADIAL_JUST_SHOWN: AtomicBool = AtomicBool::new(false);
pub(crate) static RADIAL_VISIBLE: AtomicBool = AtomicBool::new(false);
pub(crate) static RADIAL_HWND: AtomicIsize = AtomicIsize::new(0);

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
pub(crate) struct RadialMenuDownPayload {
    pub(crate) theme: String,
}

/// Show radial menu at current mouse cursor position.
/// Used as a keyboard shortcut alternative (Ctrl+Alt+V) when CGEventTap is unavailable.
pub fn show_radial_menu_at_cursor(app: &AppHandle) {
    if !RADIAL_MENU_ENABLED.load(Ordering::SeqCst) {
        return;
    }

    if let Some(window) = app.get_webview_window("radial-menu") {
        if let Some(main) = app.get_webview_window("main") {
            let _ = main.hide();
        }

        #[cfg(target_os = "macos")]
        {
            use cocoa::base::id;
            use cocoa::foundation::{NSPoint, NSRect};
            use objc::{class, msg_send, sel, sel_impl};
            let loc: NSPoint = unsafe { msg_send![class!(NSEvent), mouseLocation] };
            // Get the screen containing the cursor
            let screens: id = unsafe { msg_send![class!(NSScreen), screens] };
            let count: u64 = unsafe { msg_send![screens, count] };
            let mut screen_h: f64 = 0.0;
            for i in 0..count {
                let screen: id = unsafe { msg_send![screens, objectAtIndex:i] };
                let frame: NSRect = unsafe { msg_send![screen, frame] };
                // Check if cursor is on this screen
                if loc.x >= frame.origin.x && loc.x <= frame.origin.x + frame.size.width
                    && loc.y >= frame.origin.y && loc.y <= frame.origin.y + frame.size.height {
                    screen_h = frame.origin.y + frame.size.height;
                    break;
                }
            }
            if screen_h == 0.0 {
                let screen: id = unsafe { msg_send![class!(NSScreen), mainScreen] };
                let frame: NSRect = unsafe { msg_send![screen, frame] };
                screen_h = frame.origin.y + frame.size.height;
            }
            // Convert macOS bottom-left → top-left (Tauri LogicalPosition coords)
            let x = loc.x;
            let y = screen_h - loc.y;
            let _ = window.set_position(tauri::Position::Logical(
                tauri::LogicalPosition::new(x, y),
            ));
            log::info!("[radial] cursor=({:.0},{:.0}) screen_top={:.0} pos=({:.0},{:.0})",
                loc.x, loc.y, screen_h, x, y);
        }
        #[cfg(target_os = "windows")]
        {
            use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;
            let mut pt = windows::Win32::Foundation::POINT::default();
            unsafe { let _ = GetCursorPos(&mut pt); }
            let _ = window.set_position(tauri::Position::Physical(
                tauri::PhysicalPosition::new(pt.x, pt.y),
            ));
        }

        let theme = crate::db::get_setting(app.clone(), "theme".to_string())
            .unwrap_or_else(|_| "light".to_string());
        let _ = app.emit("radial-menu-down", RadialMenuDownPayload { theme });

        RADIAL_JUST_SHOWN.store(true, Ordering::SeqCst);
        RADIAL_HWND.store(1, Ordering::SeqCst);
        RADIAL_VISIBLE.store(true, Ordering::SeqCst);
        let _ = window.show();
        let _ = window.set_focus();
    }
}

#[tauri::command]
pub fn radial_menu_dismissed() {
    RADIAL_VISIBLE.store(false, Ordering::SeqCst);
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
                unsafe {
                    use windows::Win32::UI::WindowsAndMessaging::AllowSetForegroundWindow;
                    let _ = AllowSetForegroundWindow(0xFFFFFFFF);
                }
            }
            log::info!("[toggle_window] showing window");
            let _ = window.unminimize();
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
                        let hook_struct = &*(l_param.0 as *const MSLLHOOKSTRUCT);
                        let sx = hook_struct.pt.x;
                        let sy = hook_struct.pt.y;

                        crate::paste::save_foreground_window();

                        // Hide main window so its content doesn't bleed through backdrop blur
                        if let Some(main) = app.get_webview_window("main") {
                            let _ = main.hide();
                        }

                        // Allow this process to call SetForegroundWindow
                        unsafe {
                            use windows::Win32::UI::WindowsAndMessaging::AllowSetForegroundWindow;
                            let _ = AllowSetForegroundWindow(0xFFFFFFFF);
                        }

                        // Reposition window at mouse cursor (always, regardless of visibility)
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
                        RADIAL_HWND.store(window.hwnd().unwrap_or_default().0 as isize, Ordering::SeqCst);
                        RADIAL_VISIBLE.store(true, Ordering::SeqCst);
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                return LRESULT(1);
            }
        }

        if msg == WM_RBUTTONUP && RADIAL_JUST_SHOWN.swap(false, Ordering::SeqCst) {
            return LRESULT(1);
        }

        // When radial menu is visible, hide it if user clicks outside
        if RADIAL_VISIBLE.load(Ordering::SeqCst)
            && !RADIAL_JUST_SHOWN.load(Ordering::SeqCst)
            && (msg == WM_LBUTTONDOWN || msg == WM_RBUTTONDOWN || msg == WM_MBUTTONDOWN)
        {
            let hook_struct = &*(l_param.0 as *const MSLLHOOKSTRUCT);
            let hwnd_at_point = WindowFromPoint(hook_struct.pt);
            let root = GetAncestor(hwnd_at_point, GA_ROOT);
            let radial_hwnd = RADIAL_HWND.load(Ordering::SeqCst);
            if radial_hwnd != 0 && root.0 as isize != radial_hwnd {
                RADIAL_VISIBLE.store(false, Ordering::SeqCst);
                if let Some(app) = APP_HANDLE.get() {
                    if let Some(window) = app.get_webview_window("radial-menu") {
                        let _ = window.hide();
                        app.emit("radial-menu-dismissed", ()).ok();
                    }
                }
            }
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

    #[cfg(target_os = "macos")]
    {
        if let Ok(val) = crate::db::get_setting(app.clone(), "radial_menu_enabled".to_string()) {
            RADIAL_MENU_ENABLED.store(val == "1", Ordering::SeqCst);
        }

        APP_HANDLE.set(app.clone()).ok();

        // Mouse gesture (Shift+Cmd+Click) unavailable on macOS.
        // Both CGEventTap and NSEvent.addGlobalMonitor require complex setup.
        // Available alternatives:
        //   - Cmd+Shift+V keyboard shortcut
        //   - Tray right-click → 简约窗口 / Radial Menu
        log::info!("macOS mouse gesture not available — use Cmd+Shift+V or tray menu");
    }

    /* OLD CGEventTap code preserved for reference:

        let _app_handle = app.clone();
        std::thread::spawn(move || {
            // Use raw FFI for CGEventTap — more reliable than crate API
            extern "C" {
                fn CGEventTapCreate(
                    tap: i32,           // kCGHIDEventTap = 0
                    place: i32,         // kCGHeadInsertEventTap = 0
                    options: i32,       // kCGEventTapOptionDefault = 0
                    events_of_interest: u64,
                    callback: Option<
                        unsafe extern "C" fn(
                            proxy: *mut std::ffi::c_void,
                            event_type: u32,
                            event: *mut std::ffi::c_void,
                            user_info: *mut std::ffi::c_void,
                        ) -> *mut std::ffi::c_void,
                    >,
                    user_info: *mut std::ffi::c_void,
                ) -> *mut std::ffi::c_void;

                fn CFRunLoopGetCurrent() -> *mut std::ffi::c_void;
                fn CFMachPortCreateRunLoopSource(
                    allocator: *mut std::ffi::c_void,
                    port: *mut std::ffi::c_void,
                    order: i64,
                ) -> *mut std::ffi::c_void;
                fn CFRunLoopAddSource(
                    rl: *mut std::ffi::c_void,
                    source: *mut std::ffi::c_void,
                    mode: *mut std::ffi::c_void,
                );
                fn CGEventTapEnable(tap: *mut std::ffi::c_void, enable: bool);
                fn CFRunLoopRun();
            }

            // Mouse event types: kCGEventRightMouseDown=3, kCGEventRightMouseUp=4,
            // kCGEventLeftMouseDown=1, kCGEventOtherMouseDown=25
            let events_mask: u64 = (1u64 << 3) | (1u64 << 4) | (1u64 << 1) | (1u64 << 25);

            unsafe extern "C" fn tap_callback(
                _proxy: *mut std::ffi::c_void,
                event_type: u32,
                event: *mut std::ffi::c_void,
                _user_info: *mut std::ffi::c_void,
            ) -> *mut std::ffi::c_void {
                use core_graphics::geometry::CGPoint;

                if event_type == 3 {
                    // RightMouseDown — check modifier flags from the CGEvent
                    extern "C" {
                        fn CGEventGetFlags(event: *mut std::ffi::c_void) -> u64;
                        fn CGEventGetLocation(event: *mut std::ffi::c_void) -> CGPoint;
                    }

                    let flags = unsafe { CGEventGetFlags(event) };
                    let ctrl = (flags & (1 << 17)) != 0;   // kCGEventFlagMaskControl
                    let alt = (flags & (1 << 19)) != 0;    // kCGEventFlagMaskAlternate
                    let shift = (flags & (1 << 18)) != 0;  // kCGEventFlagMaskShift

                    if ctrl && alt && !shift {
                        if RADIAL_MENU_ENABLED.load(Ordering::SeqCst) {
                            if let Some(app) = APP_HANDLE.get() {
                                if let Some(window) = app.get_webview_window("radial-menu") {
                                    let loc = unsafe { CGEventGetLocation(event) };

                                    if let Some(main) = app.get_webview_window("main") {
                                        let _ = main.hide();
                                    }

                                    let _ = window.set_position(tauri::Position::Physical(
                                        tauri::PhysicalPosition::new(loc.x as i32, loc.y as i32),
                                    ));

                                    let theme = crate::db::get_setting(app.clone(), "theme".to_string())
                                        .unwrap_or_else(|_| "light".to_string());

                                    let _ = app.emit("radial-menu-down", RadialMenuDownPayload { theme });

                                    RADIAL_JUST_SHOWN.store(true, Ordering::SeqCst);
                                    RADIAL_HWND.store(1, Ordering::SeqCst); // non-zero sentinel for macOS
                                    RADIAL_VISIBLE.store(true, Ordering::SeqCst);
                                    let _ = window.show();
                                    let _ = window.set_focus();
                                }
                            }
                        }
                        return std::ptr::null_mut(); // consume event
                    }

                    if ctrl && shift && !alt {
                        if let Some(app) = APP_HANDLE.get() {
                            toggle_window(app);
                        }
                        return std::ptr::null_mut();
                    }
                }

                if event_type == 4 {
                    // RightMouseUp
                    if RADIAL_JUST_SHOWN.swap(false, Ordering::SeqCst) {
                        return std::ptr::null_mut();
                    }
                }

                // Dismiss radial on outside click
                if RADIAL_VISIBLE.load(Ordering::SeqCst)
                    && !RADIAL_JUST_SHOWN.load(Ordering::SeqCst)
                    && (event_type == 1 || event_type == 3 || event_type == 25)
                {
                    RADIAL_VISIBLE.store(false, Ordering::SeqCst);
                    if let Some(app) = APP_HANDLE.get() {
                        if let Some(window) = app.get_webview_window("radial-menu") {
                            let _ = window.hide();
                            app.emit("radial-menu-dismissed", ()).ok();
                        }
                    }
                }

                event // pass through
            }

            unsafe {
                let tap = CGEventTapCreate(0, 0, 0, events_mask, Some(tap_callback), std::ptr::null_mut());

                if tap.is_null() {
                    log::warn!("CGEventTapCreate failed — accessibility permission may not be granted");
                    return;
                }

                let run_loop_source = CFMachPortCreateRunLoopSource(std::ptr::null_mut(), tap, 0);
                let current_loop = CFRunLoopGetCurrent();
                CFRunLoopAddSource(current_loop, run_loop_source, std::ptr::null_mut());
                CGEventTapEnable(tap, true);
                log::info!("macOS CGEvent tap installed (Ctrl+Shift+RightClick / Ctrl+Alt+RightClick)");
                CFRunLoopRun();
            }
        });
        */
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

    // Toggle keyboard hook for Win-key shortcuts (Windows only)
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

    // Register the shortcut via global-shortcut plugin.
    // On Windows, Win+key combos are handled by the keyboard hook above,
    // so we skip plugin registration to avoid double-firing.
    // On macOS/Linux, Super/Cmd is a normal modifier — always register.
    #[cfg(not(target_os = "windows"))]
    {
        register_keyboard_shortcut(&app, &new_key)
            .map_err(|e| format!("Failed to register shortcut: {}", e))?;
    }
    #[cfg(target_os = "windows")]
    {
        if !new_key.starts_with("Super+") {
            register_keyboard_shortcut(&app, &new_key)
                .map_err(|e| format!("Failed to register shortcut: {}", e))?;
        }
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

