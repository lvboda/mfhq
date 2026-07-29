use std::thread::sleep;
use std::time::Duration;
use windows::Win32::Foundation::{HWND, LPARAM};
use windows::core::BOOL;
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetWindowTextW, IsWindowVisible, SW_RESTORE, SetForegroundWindow, ShowWindow,
};

struct Search {
    keys: Vec<String>,
    found: Vec<HWND>,
}

unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let search = unsafe { &mut *(lparam.0 as *mut Search) };
    unsafe {
        if !IsWindowVisible(hwnd).as_bool() {
            return BOOL(1);
        }
        let mut buf = [0u16; 512];
        let n = GetWindowTextW(hwnd, &mut buf);
        if n > 0 {
            let title = String::from_utf16_lossy(&buf[..n as usize]);
            let title = title.trim();
            if !title.is_empty() && search.keys.iter().any(|k| title.contains(k.as_str())) {
                search.found.push(hwnd);
            }
        }
    }
    BOOL(1)
}

pub fn find(keys: &[&str]) -> Vec<HWND> {
    let mut search = Search {
        keys: keys.iter().map(|k| k.to_string()).collect(),
        found: Vec::new(),
    };
    unsafe {
        let _ = EnumWindows(Some(enum_proc), LPARAM(&mut search as *mut Search as isize));
    }
    search.found
}

pub fn focus(hwnd: HWND) {
    unsafe {
        let _ = ShowWindow(hwnd, SW_RESTORE);
        sleep(Duration::from_millis(500));
        let _ = SetForegroundWindow(hwnd);
        sleep(Duration::from_millis(500));
    }
}

/// 对应 Python 的 focus_mofa_haqi_window。
pub fn focus_game(keys: &[&str]) -> bool {
    match find(keys).first() {
        Some(&hwnd) => {
            focus(hwnd);
            true
        }
        None => false,
    }
}
