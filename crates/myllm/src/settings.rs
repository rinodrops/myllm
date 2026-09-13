use std::path::{Path, PathBuf};
use std::process::Command;

pub fn find_settings_binary() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    if let Some(dir) = exe.parent() {
        for name in ["settings", "Settings", "Settings.exe", "settings.exe"] {
            let candidate = dir.join(name);
            if is_executable(&candidate) {
                return Some(candidate);
            }
        }
    }
    search_path("settings").or_else(|| search_path("Settings"))
}

pub fn spawn_settings(config: &Path) -> Result<(), String> {
    let bin = find_settings_binary().ok_or_else(|| {
        "Settings binary not found next to myllm or on PATH".to_string()
    })?;
    Command::new(bin)
        .arg(config)
        .spawn()
        .map_err(|err| format!("failed to spawn Settings: {err}"))?;
    Ok(())
}

fn is_executable(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        path.metadata()
            .map(|m| m.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
    }
    #[cfg(not(unix))]
    {
        true
    }
}

fn search_path(name: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let candidate = dir.join(name);
        if is_executable(&candidate) {
            return Some(candidate);
        }
        #[cfg(windows)]
        {
            let exe = dir.join(format!("{name}.exe"));
            if is_executable(&exe) {
                return Some(exe);
            }
        }
    }
    None
}
