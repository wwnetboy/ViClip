#[cfg(target_os = "windows")]
use std::collections::HashMap;
#[cfg(target_os = "windows")]
use std::sync::{Mutex, OnceLock};
#[cfg(target_os = "windows")]
use windows::Win32::Foundation::*;
#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::*;
#[cfg(target_os = "windows")]
use windows::Win32::UI::Shell::*;

#[cfg(target_os = "windows")]
struct AspectLock {
    ratio: f64,
    titlebar_h: i32,
}

#[cfg(target_os = "windows")]
static LOCKS: OnceLock<Mutex<HashMap<isize, AspectLock>>> = OnceLock::new();

#[cfg(target_os = "windows")]
fn locks() -> &'static Mutex<HashMap<isize, AspectLock>> {
    LOCKS.get_or_init(|| Mutex::new(HashMap::new()))
}

const SUBCLASS_ID: usize = 0x5669_436C_6970; // "ViClip"

#[cfg(target_os = "windows")]
unsafe extern "system" fn subclass_proc(
    hwnd: HWND,
    msg: u32,
    w_param: WPARAM,
    l_param: LPARAM,
    _subclass_id: usize,
    _ref_data: usize,
) -> LRESULT {
    if msg == WM_SIZING {
        if let Ok(locks) = locks().lock() {
            if let Some(lock) = locks.get(&(hwnd.0 as isize)) {
                let rect = &mut *(l_param.0 as *mut RECT);
                let edge = w_param.0 as u32;
                match edge {
                    // WMSZ_TOP (3) or WMSZ_BOTTOM (6): height-driven, adjust width
                    3 | 6 => {
                        let content_h = (rect.bottom - rect.top) - lock.titlebar_h;
                        let new_w = (content_h as f64 * lock.ratio) as i32;
                        let center_x = (rect.left + rect.right) / 2;
                        rect.left = center_x - new_w / 2;
                        rect.right = center_x + new_w / 2;
                    }
                    // WMSZ_TOPLEFT (4) or WMSZ_TOPRIGHT (5): width-driven, top adjusts
                    4 | 5 => {
                        let width = rect.right - rect.left;
                        let content_h = (width as f64 / lock.ratio) as i32;
                        let total_h = content_h + lock.titlebar_h;
                        rect.top = rect.bottom - total_h;
                    }
                    // Default: width-driven (LEFT=1, RIGHT=2, BOTTOMLEFT=7, BOTTOMRIGHT=8)
                    _ => {
                        let width = rect.right - rect.left;
                        let content_h = (width as f64 / lock.ratio) as i32;
                        let total_h = content_h + lock.titlebar_h;
                        rect.bottom = rect.top + total_h;
                    }
                }
                return LRESULT(1);
            }
        }
    }
    DefSubclassProc(hwnd, msg, w_param, l_param)
}

#[tauri::command]
pub fn set_preview_aspect_ratio(
    app: tauri::AppHandle,
    window_label: String,
    ratio: Option<f64>,
    titlebar_h_logical: i32,
) {
    #[cfg(not(target_os = "windows"))]
    {
        let _ = (app, window_label, ratio, titlebar_h_logical);
    }

    #[cfg(target_os = "windows")]
    {
        use tauri::Manager;
        if let Some(window) = app.get_webview_window(&window_label) {
            if let Ok(hwnd) = window.hwnd() {
                let hwnd_key = hwnd.0 as isize;

                if let Some(r) = ratio {
                    let scale = window.scale_factor().unwrap_or(1.0);
                    let titlebar_h = (titlebar_h_logical as f64 * scale) as i32;

                    let mut locks = locks().lock().unwrap();
                    if !locks.contains_key(&hwnd_key) {
                        unsafe {
                            let _ = SetWindowSubclass(
                                HWND(hwnd.0),
                                Some(subclass_proc),
                                SUBCLASS_ID,
                                0,
                            );
                        }
                    }
                    locks.insert(hwnd_key, AspectLock { ratio: r, titlebar_h });
                } else {
                    let mut locks = locks().lock().unwrap();
                    if locks.remove(&hwnd_key).is_some() {
                        unsafe {
                            let _ = RemoveWindowSubclass(
                                HWND(hwnd.0),
                                Some(subclass_proc),
                                SUBCLASS_ID,
                            );
                        }
                    }
                }
            }
        }
    }
}
