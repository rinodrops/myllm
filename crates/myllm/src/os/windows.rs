use std::mem::{size_of, zeroed};
use std::time::Duration;

use raw_window_handle::{RawWindowHandle, WindowHandle};
use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, VIRTUAL_KEY, VK_CONTROL,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetWindowLongPtrW, GetWindowThreadProcessId, SetWindowLongPtrW,
    GWL_EXSTYLE, WS_EX_APPWINDOW, WS_EX_TOOLWINDOW,
};

use super::read_clipboard;

const VK_C: VIRTUAL_KEY = 0x43;

pub fn local_hm() -> String {
    String::new()
}

pub fn apply_tool_window(_ctx: &egui::Context) {
    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd.is_null() {
        return;
    }
    set_tool_style(hwnd);
}

pub fn apply_tool_window_handle(handle: WindowHandle<'_>) {
    if let RawWindowHandle::Win32(win) = handle.as_raw() {
        set_tool_style(win.hwnd.get() as HWND);
    }
}

fn set_tool_style(hwnd: HWND) {
    unsafe {
        let style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        let next = (style | WS_EX_TOOLWINDOW as isize) & !(WS_EX_APPWINDOW as isize);
        if next != style {
            SetWindowLongPtrW(hwnd, GWL_EXSTYLE, next);
        }
    }
}

pub fn foreground_pid() -> Option<u32> {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.is_null() {
            return None;
        }
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, &mut pid);
        if pid == 0 {
            None
        } else {
            Some(pid)
        }
    }
}

pub fn capture_selection(source_pid: Option<u32>) -> String {
    if source_pid.filter(|pid| *pid != std::process::id()).is_some() {
        send_ctrl_c();
        std::thread::sleep(Duration::from_millis(250));
    }
    read_clipboard()
}

fn send_ctrl_c() {
    unsafe {
        let mut inputs: [INPUT; 4] = zeroed();
        inputs[0] = key(VK_CONTROL, false);
        inputs[1] = key(VK_C, false);
        inputs[2] = key(VK_C, true);
        inputs[3] = key(VK_CONTROL, true);
        SendInput(
            inputs.len() as u32,
            inputs.as_ptr(),
            size_of::<INPUT>() as i32,
        );
    }
}

fn key(code: VIRTUAL_KEY, up: bool) -> INPUT {
    let mut input: INPUT = unsafe { zeroed() };
    input.r#type = INPUT_KEYBOARD;
    input.Anonymous = INPUT_0 {
        ki: KEYBDINPUT {
            wVk: code,
            wScan: 0,
            dwFlags: if up { KEYEVENTF_KEYUP } else { 0 },
            time: 0,
            dwExtraInfo: 0,
        },
    };
    input
}
