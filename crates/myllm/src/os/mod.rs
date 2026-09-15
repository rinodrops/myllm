use std::path::Path;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

pub fn supports_in_process_hotkeys() -> bool {
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    {
        true
    }
    #[cfg(target_os = "linux")]
    {
        linux::supports_in_process_hotkeys()
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        false
    }
}

pub fn apply_float_chrome(ctx: &egui::Context, frame: &eframe::Frame, visible: bool) {
    #[cfg(target_os = "macos")]
    if visible {
        macos::apply_float_chrome();
    }
    #[cfg(target_os = "windows")]
    {
        use raw_window_handle::HasWindowHandle;
        if let Ok(handle) = frame.window_handle() {
            windows::apply_tool_window_handle(handle, visible);
        }
    }
    let _ = (ctx, frame, visible);
}

pub fn acquire_instance() -> bool {
    #[cfg(target_os = "windows")]
    {
        windows::acquire_instance()
    }
    #[cfg(not(target_os = "windows"))]
    {
        true
    }
}

pub fn set_accessory(hidden: bool) {
    #[cfg(target_os = "macos")]
    macos::set_accessory(hidden);
    let _ = hidden;
}

pub fn show_startup_error(message: &str) {
    #[cfg(target_os = "windows")]
    windows::show_startup_error(message);
    #[cfg(not(target_os = "windows"))]
    let _ = message;
}

pub fn set_app_icon() {
    #[cfg(target_os = "macos")]
    macos::set_app_icon();
}

pub fn install_quit_watch() {
    #[cfg(target_os = "macos")]
    macos::install_quit_watch();
}

pub fn take_app_menu_quit() -> bool {
    #[cfg(target_os = "macos")]
    {
        macos::take_app_menu_quit()
    }
    #[cfg(not(target_os = "macos"))]
    {
        false
    }
}

pub fn local_hm() -> String {
    #[cfg(unix)]
    {
        unix_local_hm()
    }
    #[cfg(windows)]
    {
        windows::local_hm()
    }
    #[cfg(not(any(unix, windows)))]
    {
        String::new()
    }
}

#[cfg(unix)]
fn unix_local_hm() -> String {
    unsafe {
        let mut t = 0 as libc::time_t;
        libc::time(&mut t);
        let mut tm = std::mem::zeroed();
        if libc::localtime_r(&t, &mut tm).is_null() {
            return String::new();
        }
        let mut buf = [0u8; 8];
        let n = libc::strftime(
            buf.as_mut_ptr().cast(),
            buf.len(),
            b"%H:%M\0".as_ptr().cast(),
            &tm,
        );
        if n == 0 {
            String::new()
        } else {
            String::from_utf8_lossy(&buf[..n]).into_owned()
        }
    }
}

pub fn preferred_ui_langs() -> Vec<String> {
    #[cfg(target_os = "macos")]
    {
        macos::preferred_ui_langs()
    }
    #[cfg(target_os = "windows")]
    {
        windows::preferred_ui_langs()
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        for key in ["LC_ALL", "LC_MESSAGES", "LANG"] {
            if let Ok(val) = std::env::var(key) {
                if !val.trim().is_empty() {
                    return vec![val];
                }
            }
        }
        Vec::new()
    }
}

pub fn frontmost_pid() -> Option<u32> {
    #[cfg(target_os = "macos")]
    {
        macos::frontmost_pid()
    }
    #[cfg(target_os = "windows")]
    {
        windows::foreground_pid()
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        None
    }
}

pub fn current_pid() -> u32 {
    std::process::id()
}

pub fn capture_selection(source_pid: Option<u32>) -> String {
    #[cfg(target_os = "macos")]
    {
        macos::capture_selection(source_pid)
    }
    #[cfg(target_os = "windows")]
    {
        windows::capture_selection(source_pid)
    }
    #[cfg(target_os = "linux")]
    {
        linux::capture_selection()
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        read_clipboard()
    }
}

pub fn read_clipboard() -> String {
    arboard::Clipboard::new()
        .ok()
        .and_then(|mut c| c.get_text().ok())
        .unwrap_or_default()
}

pub fn write_clipboard(text: &str) -> Result<(), String> {
    arboard::Clipboard::new()
        .and_then(|mut c| c.set_text(text.to_string()))
        .map_err(|err| err.to_string())
}

pub fn open_path(path: &Path) -> Result<(), String> {
    let path = if path.is_file() {
        path.parent().unwrap_or(path)
    } else {
        path
    };
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(path)
            .spawn()
            .map(|_| ())
            .map_err(|err| err.to_string())
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(path)
            .spawn()
            .map(|_| ())
            .map_err(|err| err.to_string())
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        std::process::Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map(|_| ())
            .map_err(|err| err.to_string())
    }
}
