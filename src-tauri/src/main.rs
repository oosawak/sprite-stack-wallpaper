#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            if let Some(win) = app.get_webview_window("main") {
                attach_to_desktop(&win);
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// ===== macOS: NSWindow をデスクトップレベルへ下げて壁紙化する =====
#[cfg(target_os = "macos")]
fn attach_to_desktop(win: &tauri::WebviewWindow) {
    use objc::{msg_send, sel, sel_impl};
    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        fn CGWindowLevelForKey(key: i32) -> i32;
    }
    let ns_win = win.ns_window().unwrap() as *mut objc::runtime::Object;
    unsafe {
        let level = CGWindowLevelForKey(2); // kCGDesktopWindowLevelKey
        let _: () = msg_send![ns_win, setLevel: level as isize];
        let _: () = msg_send![ns_win, setIgnoresMouseEvents: true];
        // canJoinAllSpaces(1) | stationary(16) | ignoresCycle(64)
        let _: () = msg_send![ns_win, setCollectionBehavior: 1usize | 16usize | 64usize];
        let _: () = msg_send![ns_win, orderBack: std::ptr::null_mut::<objc::runtime::Object>()];
    }
}

// ===== Windows: WorkerW を親にしてアイコンの背面へ入れる =====
#[cfg(target_os = "windows")]
fn attach_to_desktop(win: &tauri::WebviewWindow) {
    use std::ffi::c_void;
    type Hwnd = *mut c_void;
    extern "system" {
        fn FindWindowW(class: *const u16, name: *const u16) -> Hwnd;
        fn FindWindowExW(parent: Hwnd, child_after: Hwnd, class: *const u16, name: *const u16) -> Hwnd;
        fn SendMessageTimeoutW(hwnd: Hwnd, msg: u32, wparam: usize, lparam: isize, flags: u32, timeout: u32, result: *mut usize) -> isize;
        fn EnumWindows(cb: unsafe extern "system" fn(Hwnd, isize) -> i32, lparam: isize) -> i32;
        fn SetParent(child: Hwnd, parent: Hwnd) -> Hwnd;
        fn SetWindowPos(hwnd: Hwnd, after: Hwnd, x: i32, y: i32, cx: i32, cy: i32, flags: u32) -> i32;
        fn GetSystemMetrics(index: i32) -> i32;
    }
    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }
    static mut WORKER: *mut c_void = std::ptr::null_mut();
    unsafe extern "system" fn enum_cb(top: Hwnd, _: isize) -> i32 {
        let shell = FindWindowExW(top, std::ptr::null_mut(), wide("SHELLDLL_DefView").as_ptr(), std::ptr::null());
        if !shell.is_null() {
            let w = FindWindowExW(std::ptr::null_mut(), top, wide("WorkerW").as_ptr(), std::ptr::null());
            if !w.is_null() {
                WORKER = w;
            }
        }
        1
    }
    unsafe {
        let hwnd = win.hwnd().unwrap().0 as Hwnd;
        let progman = FindWindowW(wide("Progman").as_ptr(), std::ptr::null());
        let mut res: usize = 0;
        SendMessageTimeoutW(progman, 0x052C, 0, 0, 0, 1000, &mut res);
        EnumWindows(enum_cb, 0);
        if !WORKER.is_null() {
            SetParent(hwnd, WORKER);
            let (w, h) = (GetSystemMetrics(0), GetSystemMetrics(1));
            SetWindowPos(hwnd, std::ptr::null_mut(), 0, 0, w, h, 0x0010 | 0x0040);
        }
    }
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn attach_to_desktop(_win: &tauri::WebviewWindow) {}
