use whichlang::{detect_language, Lang};

pub fn detect_source(text: &str, default_source: &str) -> String {
    if text.trim().is_empty() {
        return default_source.to_string();
    }
    iso_code(detect_language(text)).to_string()
}

pub fn iso_code(lang: Lang) -> &'static str {
    match lang {
        Lang::Ara => "ar",
        Lang::Nld => "nl",
        Lang::Eng => "en",
        Lang::Fra => "fr",
        Lang::Deu => "de",
        Lang::Hin => "hi",
        Lang::Ita => "it",
        Lang::Jpn => "ja",
        Lang::Kor => "ko",
        Lang::Cmn => "zh",
        Lang::Por => "pt",
        Lang::Rus => "ru",
        Lang::Spa => "es",
        Lang::Swe => "sv",
        Lang::Tur => "tr",
        Lang::Vie => "vi",
    }
}

pub fn language_name(code: &str) -> String {
    match code {
        "ar" => "Arabic",
        "nl" => "Dutch",
        "en" => "English",
        "fr" => "French",
        "de" => "German",
        "hi" => "Hindi",
        "it" => "Italian",
        "ja" => "Japanese",
        "ko" => "Korean",
        "zh" => "Mandarin",
        "pt" => "Portuguese",
        "ru" => "Russian",
        "es" => "Spanish",
        "sv" => "Swedish",
        "tr" => "Turkish",
        "vi" => "Vietnamese",
        other => other,
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_text_uses_default_source() {
        assert_eq!(detect_source("   ", "ja"), "ja");
    }

    #[test]
    fn japanese_text_detects_ja() {
        assert_eq!(detect_source("これは日本語の文章です。", "en"), "ja");
    }
}
