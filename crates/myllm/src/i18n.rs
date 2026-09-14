//! Window and tray chrome strings.
//!
//! Schema labels stay in `schema.toml`. Translation `default_target` is unrelated.

#[derive(Debug, Clone, Copy)]
pub struct Strings {
    pub input: &'static str,
    pub output: &'static str,
    pub run: &'static str,
    pub copy: &'static str,
    pub open_window: &'static str,
    pub reload_config: &'static str,
    pub open_config: &'static str,
    pub settings: &'static str,
    pub quit: &'static str,
    pub config_created: &'static str,
    pub ok: &'static str,
    pub wayland_hint: &'static str,
    pub clipboard_empty: &'static str,
    pub config_reloaded: &'static str,
    pub copied: &'static str,
    pub opened_settings: &'static str,
    pub translate: &'static str,
    pub config_created_title: &'static str,
}

const EN: Strings = Strings {
    input: "Input",
    output: "Output",
    run: "Run",
    copy: "Copy",
    open_window: "Open Window",
    reload_config: "Reload Config",
    open_config: "Open Config Folder",
    settings: "Settings…",
    quit: "Quit My LLM",
    config_created: "A starter config was created at:",
    ok: "OK",
    wayland_hint: "On Wayland, assign a compositor shortcut to `myllm --task <id>`.",
    clipboard_empty: "No text selected and clipboard is empty.",
    config_reloaded: "Config reloaded",
    copied: "Copied",
    opened_settings: "Opened Settings",
    translate: "Translate",
    config_created_title: "Configuration created",
};

const JA: Strings = Strings {
    input: "入力",
    output: "出力",
    run: "実行",
    copy: "コピー",
    open_window: "ウィンドウを開く",
    reload_config: "設定を再読み込み",
    open_config: "設定フォルダを開く",
    settings: "設定…",
    quit: "My LLM を終了",
    config_created: "初期設定を作成しました:",
    ok: "OK",
    wayland_hint: "Wayland ではコンポジタのショートカットから `myllm --task <id>` を実行します。",
    clipboard_empty: "選択テキストもクリップボードも空です。",
    config_reloaded: "設定を再読み込みしました",
    copied: "コピーしました",
    opened_settings: "設定を開きました",
    translate: "翻訳",
    config_created_title: "設定ファイルを作成しました",
};

pub fn t(config_ui_lang: Option<&str>, os_langs: &[String]) -> &'static Strings {
    match resolve(config_ui_lang, os_langs) {
        "ja" => &JA,
        _ => &EN,
    }
}

pub fn resolve(config_ui_lang: Option<&str>, os_langs: &[String]) -> &'static str {
    if let Some(raw) = config_ui_lang {
        let trimmed = raw.trim();
        if !trimmed.is_empty() && !trimmed.eq_ignore_ascii_case("os") {
            return lang_code(trimmed).unwrap_or("en");
        }
    }
    for tag in os_langs {
        if let Some(code) = lang_code(tag) {
            return code;
        }
    }
    "en"
}

fn lang_code(tag: &str) -> Option<&'static str> {
    let tag = tag.trim().to_ascii_lowercase();
    if tag.is_empty() || tag == "c" || tag == "posix" {
        return None;
    }
    let primary = tag.split(|c| c == '-' || c == '_').next().unwrap_or(&tag);
    match primary {
        "ar" => Some("ar"),
        "nl" => Some("nl"),
        "en" => Some("en"),
        "fr" => Some("fr"),
        "de" => Some("de"),
        "hi" => Some("hi"),
        "it" => Some("it"),
        "ja" => Some("ja"),
        "ko" => Some("ko"),
        "zh" => Some("zh"),
        "pt" => Some("pt"),
        "ru" => Some("ru"),
        "es" => Some("es"),
        "sv" => Some("sv"),
        "tr" => Some("tr"),
        "vi" => Some("vi"),
        _ => None,
    }
}

pub fn format_hotkey(spec: &str) -> Option<String> {
    let mut ctrl = false;
    let mut alt = false;
    let mut shift = false;
    let mut meta = false;
    let mut key = None;
    for part in spec.split('+').map(str::trim).filter(|s| !s.is_empty()) {
        match part.to_ascii_lowercase().as_str() {
            "cmd" | "command" | "super" | "meta" | "win" | "windows" => meta = true,
            "ctrl" | "control" => ctrl = true,
            "opt" | "option" | "alt" => alt = true,
            "shift" => shift = true,
            other => key = Some(other.to_ascii_uppercase()),
        }
    }
    let key = key?;
    #[cfg(target_os = "macos")]
    {
        let mut out = String::new();
        if ctrl {
            out.push('⌃');
        }
        if alt {
            out.push('⌥');
        }
        if shift {
            out.push('⇧');
        }
        if meta {
            out.push('⌘');
        }
        out.push_str(&key);
        Some(out)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let mut parts: Vec<String> = Vec::new();
        if ctrl || meta {
            parts.push("Ctrl".into());
        }
        if alt {
            parts.push("Alt".into());
        }
        if shift {
            parts.push("Shift".into());
        }
        parts.push(key);
        Some(parts.join("+"))
    }
}

#[cfg(test)]
mod tests {
    use super::{format_hotkey, resolve};

    #[test]
    fn explicit_code_wins() {
        assert_eq!(resolve(Some("ja"), &["en-US".into()]), "ja");
        assert_eq!(resolve(Some("os"), &["ja-JP".into()]), "ja");
        assert_eq!(resolve(None, &["fr_FR".into()]), "fr");
        assert_eq!(resolve(Some("zz"), &[]), "en");
        assert_eq!(resolve(None, &[]), "en");
    }

    #[test]
    fn hotkey_symbols() {
        let text = format_hotkey("cmd+shift+p").unwrap();
        #[cfg(target_os = "macos")]
        assert_eq!(text, "⇧⌘P");
        #[cfg(not(target_os = "macos"))]
        assert_eq!(text, "Ctrl+Shift+P");
        let ctrl_cmd = format_hotkey("ctrl+cmd+b").unwrap();
        #[cfg(target_os = "macos")]
        assert_eq!(ctrl_cmd, "⌃⌘B");
        #[cfg(not(target_os = "macos"))]
        assert_eq!(ctrl_cmd, "Ctrl+B");
    }
}
