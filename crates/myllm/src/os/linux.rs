use std::process::Command;

use super::read_clipboard;

pub fn supports_in_process_hotkeys() -> bool {
    match std::env::var("XDG_SESSION_TYPE") {
        Ok(kind) if kind.eq_ignore_ascii_case("wayland") => false,
        Ok(kind) if kind.eq_ignore_ascii_case("x11") => true,
        _ => std::env::var_os("WAYLAND_DISPLAY").is_none() && std::env::var_os("DISPLAY").is_some(),
    }
}

pub fn capture_selection() -> String {
    if let Some(text) = command_text("wl-paste", &["--primary", "--no-newline"]) {
        return text;
    }
    if let Some(text) = command_text("xclip", &["-o", "-selection", "primary"]) {
        return text;
    }
    if let Some(text) = command_text("xsel", &["-o", "-p"]) {
        return text;
    }
    read_clipboard()
}

fn command_text(bin: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(bin).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}
