use egui::{FontData, FontDefinitions, FontFamily};

pub fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    match load_cjk_font() {
        Some(bytes) => {
            let mut font_data = FontData::from_owned(bytes);
            font_data.tweak.y_offset = cjk_y_offset();
            fonts.font_data.insert("cjk".to_owned(), font_data.into());
            for family in [FontFamily::Proportional, FontFamily::Monospace] {
                if let Some(list) = fonts.families.get_mut(&family) {
                    list.push("cjk".to_owned());
                }
            }
        }
        None => eprintln!("Warning: no CJK font found — Japanese text will show as boxes."),
    }
    ctx.set_fonts(fonts);
}

fn cjk_y_offset() -> f32 {
    if cfg!(target_os = "linux") {
        0.0
    } else if cfg!(target_os = "windows") {
        1.0
    } else {
        3.0
    }
}

fn load_cjk_font() -> Option<Vec<u8>> {
    for path in cjk_font_candidates() {
        if let Ok(bytes) = std::fs::read(path) {
            return Some(bytes);
        }
    }
    None
}

fn cjk_font_candidates() -> &'static [&'static str] {
    if cfg!(target_os = "macos") {
        &[
            "/System/Library/Fonts/\u{30d2}\u{30e9}\u{30ae}\u{30ce}\u{89d2}\u{30b4}\u{30b7}\u{30c3}\u{30af} W3.ttc",
            "/System/Library/Fonts/\u{30d2}\u{30e9}\u{30ae}\u{30ce}\u{89d2}\u{30b4}\u{30b7}\u{30c3}\u{30af} W6.ttc",
            "/Library/Fonts/\u{30d2}\u{30e9}\u{30ae}\u{30ce}\u{89d2}\u{30b4}\u{30b7}\u{30c3}\u{30af} ProN W3.otf",
        ]
    } else if cfg!(target_os = "windows") {
        &[
            "C:\\Windows\\Fonts\\meiryo.ttc",
            "C:\\Windows\\Fonts\\YuGothM.ttc",
            "C:\\Windows\\Fonts\\msgothic.ttc",
        ]
    } else {
        &[
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/noto-cjk/NotoSansCJKjp-Regular.otf",
            "/usr/share/fonts/truetype/vlgothic/VL-Gothic-Regular.ttf",
        ]
    }
}
