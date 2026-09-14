use std::fs;
use std::path::PathBuf;

use eframe::egui::Pos2;
use serde::Deserialize;

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct WindowPos {
    pub x: f32,
    pub y: f32,
}

pub fn path() -> PathBuf {
    if let Ok(xdg) = std::env::var("XDG_STATE_HOME") {
        if !xdg.trim().is_empty() {
            return PathBuf::from(xdg).join("myllm").join("window.toml");
        }
    }
    #[cfg(windows)]
    {
        if let Some(base) = dirs::data_local_dir() {
            return base.join("myllm").join("window.toml");
        }
    }
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".local")
        .join("state")
        .join("myllm")
        .join("window.toml")
}

pub fn load() -> Option<Pos2> {
    let raw = fs::read_to_string(path()).ok()?;
    let pos: WindowPos = toml::from_str(&raw).ok()?;
    if !usable(pos.x, pos.y) {
        return None;
    }
    Some(Pos2::new(pos.x, pos.y))
}

pub fn save(pos: Pos2) {
    if !usable(pos.x, pos.y) {
        return;
    }
    let path = path();
    if let Some(dir) = path.parent() {
        let _ = fs::create_dir_all(dir);
    }
    let body = format!("x = {}\ny = {}\n", pos.x, pos.y);
    let _ = fs::write(path, body);
}

fn usable(x: f32, y: f32) -> bool {
    x.is_finite()
        && y.is_finite()
        && (-2000.0..8000.0).contains(&x)
        && (-2000.0..8000.0).contains(&y)
}

#[cfg(test)]
mod tests {
    use super::usable;

    #[test]
    fn rejects_off_screen_and_nan() {
        assert!(usable(100.0, 80.0));
        assert!(!usable(f32::NAN, 0.0));
        assert!(!usable(20_000.0, 0.0));
        assert!(!usable(-4000.0, 10.0));
    }
}
