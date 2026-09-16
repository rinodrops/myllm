use std::mem::{size_of, zeroed};
use std::sync::atomic::{AtomicIsize, Ordering};
use std::time::Duration;

use raw_window_handle::{RawWindowHandle, WindowHandle};
use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS};
use windows_sys::Win32::System::Threading::CreateMutexW;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, VIRTUAL_KEY, VK_CONTROL,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetWindowLongPtrW, GetWindowThreadProcessId, IsWindowVisible, MessageBoxW,
    PostMessageW, SetLayeredWindowAttributes, SetWindowLongPtrW, SetWindowPos, ShowWindow,
    GWL_EXSTYLE, HWND_NOTOPMOST, HWND_TOPMOST, LWA_ALPHA, MB_ICONERROR, MB_OK, SWP_NOACTIVATE,
    SWP_NOMOVE, SWP_NOSIZE, SW_SHOWNA, WM_NULL, WS_EX_APPWINDOW, WS_EX_LAYERED, WS_EX_TOOLWINDOW,
    WS_EX_TRANSPARENT,
};

use super::read_clipboard;

const VK_C: VIRTUAL_KEY = 0x43;
const INSTANCE_MUTEX: &str = "Local\\jp.emotiongraphics.myllm";
static RESULT_HWND: AtomicIsize = AtomicIsize::new(0);

pub fn acquire_instance() -> bool {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    let wide: Vec<u16> = OsStr::new(INSTANCE_MUTEX)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    unsafe {
        let handle = CreateMutexW(std::ptr::null(), 1, wide.as_ptr());
        if handle.is_null() {
            return true;
        }
        if GetLastError() == ERROR_ALREADY_EXISTS {
            return false;
        }
        let _ = handle;
        true
    }
}

pub fn show_startup_error(message: &str) {
    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }
    let text = wide(message);
    let caption = wide("My LLM");
    unsafe {
        MessageBoxW(
            std::ptr::null_mut(),
            text.as_ptr(),
            caption.as_ptr(),
            MB_OK | MB_ICONERROR,
        );
    }
}

pub fn local_hm() -> String {
    use windows_sys::Win32::Foundation::SYSTEMTIME;
    use windows_sys::Win32::System::SystemInformation::GetLocalTime;
    unsafe {
        let mut st: SYSTEMTIME = zeroed();
        GetLocalTime(&mut st);
        format!("{:02}:{:02}", st.wHour, st.wMinute)
    }
}

pub fn preferred_ui_langs() -> Vec<String> {
    use windows_sys::Win32::Foundation::FALSE;
    use windows_sys::Win32::Globalization::GetUserPreferredUILanguages;
    const MUI_LANGUAGE_NAME: u32 = 0x08;
    unsafe {
        let mut num_langs: u32 = 0;
        let mut buf_size: u32 = 0;
        GetUserPreferredUILanguages(
            MUI_LANGUAGE_NAME,
            &mut num_langs,
            std::ptr::null_mut(),
            &mut buf_size,
        );
        if buf_size == 0 {
            return Vec::new();
        }
        let mut buf: Vec<u16> = vec![0u16; buf_size as usize];
        let ok = GetUserPreferredUILanguages(
            MUI_LANGUAGE_NAME,
            &mut num_langs,
            buf.as_mut_ptr(),
            &mut buf_size,
        );
        if ok == FALSE {
            return Vec::new();
        }
        buf.split(|&c| c == 0)
            .filter(|segment| !segment.is_empty())
            .map(|segment| String::from_utf16_lossy(segment))
            .collect()
    }
}

pub fn apply_tool_window_handle(handle: WindowHandle<'_>, visible: bool) {
    if let RawWindowHandle::Win32(win) = handle.as_raw() {
        let hwnd = win.hwnd.get() as HWND;
        RESULT_HWND.store(win.hwnd.get() as isize, Ordering::Relaxed);
        set_tool_style(hwnd, visible);
    }
}

pub fn wake_hidden_window() {
    let hwnd = RESULT_HWND.load(Ordering::Relaxed) as HWND;
    if hwnd.is_null() {
        return;
    }
    unsafe {
        if IsWindowVisible(hwnd) == 0 {
            ShowWindow(hwnd, SW_SHOWNA);
        }
        PostMessageW(hwnd, WM_NULL, 0, 0);
    }
}

fn set_tool_style(hwnd: HWND, visible: bool) {
    unsafe {
        let style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        let mut next = (style | WS_EX_TOOLWINDOW as isize | WS_EX_LAYERED as isize)
            & !(WS_EX_APPWINDOW as isize);
        if visible {
            next &= !(WS_EX_TRANSPARENT as isize);
        } else {
            next |= WS_EX_TRANSPARENT as isize;
        }
        if next != style {
            SetWindowLongPtrW(hwnd, GWL_EXSTYLE, next);
        }
        let alpha = if visible { 255u8 } else { 0 };
        SetLayeredWindowAttributes(hwnd, 0, alpha, LWA_ALPHA);
        let z = if visible {
            HWND_TOPMOST
        } else {
            HWND_NOTOPMOST
        };
        SetWindowPos(
            hwnd,
            z,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
        );
        if visible || IsWindowVisible(hwnd) == 0 {
            ShowWindow(hwnd, SW_SHOWNA);
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
    if source_pid
        .filter(|pid| *pid != std::process::id())
        .is_some()
    {
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
