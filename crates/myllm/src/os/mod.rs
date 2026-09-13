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

pub fn apply_float_chrome(ctx: &egui::Context) {
    #[cfg(target_os = "macos")]
    macos::apply_float_chrome();
    #[cfg(target_os = "windows")]
    windows::apply_tool_window(ctx);
    let _ = ctx;
}

pub fn set_accessory(hidden: bool) {
    #[cfg(target_os = "macos")]
    macos::set_accessory(hidden);
    let _ = hidden;
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
