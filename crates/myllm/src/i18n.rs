//! Window and tray chrome strings.
//!
//! Schema labels stay in `schema.toml`. Translation `default_target` is unrelated.

use std::fmt::Display;

/// whichlang language codes accepted by `[general] ui_lang`.
pub const UI_LANGS: &[&str] = &[
    "ar", "nl", "en", "fr", "de", "hi", "it", "ja", "ko", "zh", "pt", "ru", "es", "sv", "tr", "vi",
];

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
    pub copied: &'static str,
    pub translate: &'static str,
    pub config_created_title: &'static str,
    pub hotkeys_unavailable_tmpl: &'static str,
    pub tray_unavailable_tmpl: &'static str,
    pub settings_not_found: &'static str,
    pub settings_spawn_tmpl: &'static str,
}

impl Strings {
    fn subst(tmpl: &str, arg: impl Display) -> String {
        tmpl.replace("{}", &arg.to_string())
    }

    pub fn hotkeys_unavailable(&self, err: impl Display) -> String {
        Self::subst(self.hotkeys_unavailable_tmpl, err)
    }

    pub fn tray_unavailable(&self, err: impl Display) -> String {
        Self::subst(self.tray_unavailable_tmpl, err)
    }

    pub fn settings_spawn(&self, err: impl Display) -> String {
        Self::subst(self.settings_spawn_tmpl, err)
    }
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
    copied: "Copied",
    translate: "Translate",
    config_created_title: "Configuration created",
    hotkeys_unavailable_tmpl: "hotkeys unavailable: {}",
    tray_unavailable_tmpl: "tray unavailable: {}",
    settings_not_found: "Settings binary not found next to myllm or on PATH",
    settings_spawn_tmpl: "failed to spawn Settings: {}",
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
    copied: "コピーしました",
    translate: "翻訳",
    config_created_title: "設定ファイルを作成しました",
    hotkeys_unavailable_tmpl: "ホットキーを使えません: {}",
    tray_unavailable_tmpl: "トレイを使えません: {}",
    settings_not_found: "Settings の実行ファイルが myllm の隣にも PATH 上にもありません",
    settings_spawn_tmpl: "Settings を起動できません: {}",
};

const AR: Strings = Strings {
    input: "الإدخال",
    output: "الإخراج",
    run: "تشغيل",
    copy: "نسخ",
    open_window: "فتح النافذة",
    reload_config: "إعادة تحميل الإعدادات",
    open_config: "فتح مجلد الإعدادات",
    settings: "الإعدادات…",
    quit: "إنهاء My LLM",
    config_created: "تم إنشاء ملف إعدادات أولي في:",
    ok: "موافق",
    wayland_hint: "في Wayland، عيّن اختصار المُركِّب إلى `myllm --task <id>`.",
    clipboard_empty: "لا يوجد نص محدد والحافظة فارغة.",
    copied: "تم النسخ",
    translate: "ترجمة",
    config_created_title: "تم إنشاء الإعدادات",
    hotkeys_unavailable_tmpl: "اختصارات لوحة المفاتيح غير متاحة: {}",
    tray_unavailable_tmpl: "صينية النظام غير متاحة: {}",
    settings_not_found: "تعذر العثور على برنامج Settings بجانب myllm أو في PATH",
    settings_spawn_tmpl: "تعذر تشغيل Settings: {}",
};

const NL: Strings = Strings {
    input: "Invoer",
    output: "Uitvoer",
    run: "Uitvoeren",
    copy: "Kopiëren",
    open_window: "Venster openen",
    reload_config: "Config herladen",
    open_config: "Configmap openen",
    settings: "Instellingen…",
    quit: "My LLM afsluiten",
    config_created: "Er is een startconfiguratie gemaakt op:",
    ok: "OK",
    wayland_hint: "Wijs op Wayland een compositor-sneltoets toe aan `myllm --task <id>`.",
    clipboard_empty: "Geen tekst geselecteerd en het klembord is leeg.",
    copied: "Gekopieerd",
    translate: "Vertalen",
    config_created_title: "Configuratie aangemaakt",
    hotkeys_unavailable_tmpl: "sneltoetsen niet beschikbaar: {}",
    tray_unavailable_tmpl: "systeemvak niet beschikbaar: {}",
    settings_not_found: "Settings-binair niet gevonden naast myllm of in PATH",
    settings_spawn_tmpl: "Settings starten mislukt: {}",
};

const FR: Strings = Strings {
    input: "Entrée",
    output: "Sortie",
    run: "Exécuter",
    copy: "Copier",
    open_window: "Ouvrir la fenêtre",
    reload_config: "Recharger la config",
    open_config: "Ouvrir le dossier de config",
    settings: "Préférences…",
    quit: "Quitter My LLM",
    config_created: "Un fichier de config de départ a été créé ici :",
    ok: "OK",
    wayland_hint: "Sous Wayland, assignez un raccourci du compositeur à `myllm --task <id>`.",
    clipboard_empty: "Aucun texte sélectionné et le presse-papiers est vide.",
    copied: "Copié",
    translate: "Traduire",
    config_created_title: "Configuration créée",
    hotkeys_unavailable_tmpl: "raccourcis clavier indisponibles : {}",
    tray_unavailable_tmpl: "zone de notification indisponible : {}",
    settings_not_found: "Binaire Settings introuvable à côté de myllm ou dans le PATH",
    settings_spawn_tmpl: "impossible de lancer Settings : {}",
};

const DE: Strings = Strings {
    input: "Eingabe",
    output: "Ausgabe",
    run: "Ausführen",
    copy: "Kopieren",
    open_window: "Fenster öffnen",
    reload_config: "Konfig neu laden",
    open_config: "Konfigurationsordner öffnen",
    settings: "Einstellungen…",
    quit: "My LLM beenden",
    config_created: "Eine Startkonfiguration wurde erstellt unter:",
    ok: "OK",
    wayland_hint:
        "Unter Wayland weisen Sie dem Compositor eine Verknüpfung für `myllm --task <id>` zu.",
    clipboard_empty: "Kein Text ausgewählt und die Zwischenablage ist leer.",
    copied: "Kopiert",
    translate: "Übersetzen",
    config_created_title: "Konfiguration erstellt",
    hotkeys_unavailable_tmpl: "Tastenkürzel nicht verfügbar: {}",
    tray_unavailable_tmpl: "Tray nicht verfügbar: {}",
    settings_not_found: "Settings-Programm neben myllm und im PATH nicht gefunden",
    settings_spawn_tmpl: "Settings konnte nicht gestartet werden: {}",
};

const HI: Strings = Strings {
    input: "इनपुट",
    output: "आउटपुट",
    run: "चलाएँ",
    copy: "कॉपी",
    open_window: "विंडो खोलें",
    reload_config: "कॉन्फ़िग फिर लोड करें",
    open_config: "कॉन्फ़िग फ़ोल्डर खोलें",
    settings: "सेटिंग्स…",
    quit: "My LLM बंद करें",
    config_created: "प्रारंभिक कॉन्फ़िग यहाँ बनाया गया:",
    ok: "ठीक है",
    wayland_hint: "Wayland पर कंपोज़िटर शॉर्टकट को `myllm --task <id>` पर सेट करें।",
    clipboard_empty: "कोई टेक्स्ट चयनित नहीं है और क्लिपबोर्ड खाली है।",
    copied: "कॉपी हो गया",
    translate: "अनुवाद",
    config_created_title: "कॉन्फ़िगरेशन बन गया",
    hotkeys_unavailable_tmpl: "हॉटकी उपलब्ध नहीं: {}",
    tray_unavailable_tmpl: "ट्रे उपलब्ध नहीं: {}",
    settings_not_found: "Settings बाइनरी myllm के पास या PATH में नहीं मिली",
    settings_spawn_tmpl: "Settings शुरू नहीं हो सका: {}",
};

const IT: Strings = Strings {
    input: "Input",
    output: "Output",
    run: "Esegui",
    copy: "Copia",
    open_window: "Apri finestra",
    reload_config: "Ricarica config",
    open_config: "Apri cartella config",
    settings: "Impostazioni…",
    quit: "Esci da My LLM",
    config_created: "È stato creato un file di configurazione iniziale in:",
    ok: "OK",
    wayland_hint: "Su Wayland, assegna una scorciatoia del compositor a `myllm --task <id>`.",
    clipboard_empty: "Nessun testo selezionato e gli appunti sono vuoti.",
    copied: "Copiato",
    translate: "Traduci",
    config_created_title: "Configurazione creata",
    hotkeys_unavailable_tmpl: "scorciatoie non disponibili: {}",
    tray_unavailable_tmpl: "tray non disponibile: {}",
    settings_not_found: "Binario Settings non trovato accanto a myllm né in PATH",
    settings_spawn_tmpl: "impossibile avviare Settings: {}",
};

const KO: Strings = Strings {
    input: "입력",
    output: "출력",
    run: "실행",
    copy: "복사",
    open_window: "창 열기",
    reload_config: "설정 다시 불러오기",
    open_config: "설정 폴더 열기",
    settings: "설정…",
    quit: "My LLM 종료",
    config_created: "시작 설정 파일을 만들었습니다:",
    ok: "확인",
    wayland_hint: "Wayland에서는 컴포지터 단축키로 `myllm --task <id>`를 실행하세요.",
    clipboard_empty: "선택한 텍스트가 없고 클립보드가 비어 있습니다.",
    copied: "복사했습니다",
    translate: "번역",
    config_created_title: "설정 파일을 만들었습니다",
    hotkeys_unavailable_tmpl: "단축키를 사용할 수 없습니다: {}",
    tray_unavailable_tmpl: "트레이를 사용할 수 없습니다: {}",
    settings_not_found: "Settings 실행 파일을 myllm 옆이나 PATH에서 찾을 수 없습니다",
    settings_spawn_tmpl: "Settings를 시작할 수 없습니다: {}",
};

const ZH: Strings = Strings {
    input: "输入",
    output: "输出",
    run: "运行",
    copy: "复制",
    open_window: "打开窗口",
    reload_config: "重新加载配置",
    open_config: "打开配置文件夹",
    settings: "设置…",
    quit: "退出 My LLM",
    config_created: "已在以下位置创建初始配置：",
    ok: "确定",
    wayland_hint: "在 Wayland 上，请将合成器快捷键指定为 `myllm --task <id>`。",
    clipboard_empty: "未选中文本且剪贴板为空。",
    copied: "已复制",
    translate: "翻译",
    config_created_title: "已创建配置文件",
    hotkeys_unavailable_tmpl: "快捷键不可用：{}",
    tray_unavailable_tmpl: "托盘不可用：{}",
    settings_not_found: "在 myllm 旁边和 PATH 中都找不到 Settings 程序",
    settings_spawn_tmpl: "无法启动 Settings：{}",
};

const PT: Strings = Strings {
    input: "Entrada",
    output: "Saída",
    run: "Executar",
    copy: "Copiar",
    open_window: "Abrir janela",
    reload_config: "Recarregar config",
    open_config: "Abrir pasta de config",
    settings: "Configurações…",
    quit: "Sair do My LLM",
    config_created: "Um arquivo de configuração inicial foi criado em:",
    ok: "OK",
    wayland_hint: "No Wayland, atribua um atalho do compositor a `myllm --task <id>`.",
    clipboard_empty: "Nenhum texto selecionado e a área de transferência está vazia.",
    copied: "Copiado",
    translate: "Traduzir",
    config_created_title: "Configuração criada",
    hotkeys_unavailable_tmpl: "atalhos indisponíveis: {}",
    tray_unavailable_tmpl: "bandeja indisponível: {}",
    settings_not_found: "Binário do Settings não encontrado ao lado do myllm nem no PATH",
    settings_spawn_tmpl: "falha ao iniciar o Settings: {}",
};

const RU: Strings = Strings {
    input: "Ввод",
    output: "Вывод",
    run: "Запуск",
    copy: "Копировать",
    open_window: "Открыть окно",
    reload_config: "Перезагрузить конфиг",
    open_config: "Открыть папку конфига",
    settings: "Настройки…",
    quit: "Выйти из My LLM",
    config_created: "Начальный файл конфигурации создан здесь:",
    ok: "ОК",
    wayland_hint: "В Wayland назначьте сочетание клавиш композитора на `myllm --task <id>`.",
    clipboard_empty: "Нет выделенного текста, буфер обмена пуст.",
    copied: "Скопировано",
    translate: "Перевод",
    config_created_title: "Конфигурация создана",
    hotkeys_unavailable_tmpl: "горячие клавиши недоступны: {}",
    tray_unavailable_tmpl: "лоток недоступен: {}",
    settings_not_found: "Бинарный файл Settings не найден рядом с myllm и в PATH",
    settings_spawn_tmpl: "не удалось запустить Settings: {}",
};

const ES: Strings = Strings {
    input: "Entrada",
    output: "Salida",
    run: "Ejecutar",
    copy: "Copiar",
    open_window: "Abrir ventana",
    reload_config: "Recargar config",
    open_config: "Abrir carpeta de config",
    settings: "Ajustes…",
    quit: "Salir de My LLM",
    config_created: "Se creó un archivo de configuración inicial en:",
    ok: "Aceptar",
    wayland_hint: "En Wayland, asigne un atajo del compositor a `myllm --task <id>`.",
    clipboard_empty: "No hay texto seleccionado y el portapapeles está vacío.",
    copied: "Copiado",
    translate: "Traducir",
    config_created_title: "Configuración creada",
    hotkeys_unavailable_tmpl: "atajos no disponibles: {}",
    tray_unavailable_tmpl: "bandeja no disponible: {}",
    settings_not_found: "No se encontró el binario de Settings junto a myllm ni en PATH",
    settings_spawn_tmpl: "no se pudo iniciar Settings: {}",
};

const SV: Strings = Strings {
    input: "Indata",
    output: "Utdata",
    run: "Kör",
    copy: "Kopiera",
    open_window: "Öppna fönster",
    reload_config: "Ladda om config",
    open_config: "Öppna config-mappen",
    settings: "Inställningar…",
    quit: "Avsluta My LLM",
    config_created: "En startkonfiguration skapades här:",
    ok: "OK",
    wayland_hint: "Tilldela en compositor-genväg till `myllm --task <id>` på Wayland.",
    clipboard_empty: "Ingen text är markerad och urklippet är tomt.",
    copied: "Kopierat",
    translate: "Översätt",
    config_created_title: "Konfiguration skapad",
    hotkeys_unavailable_tmpl: "snabbtangenter otillgängliga: {}",
    tray_unavailable_tmpl: "systemfältet otillgängligt: {}",
    settings_not_found: "Settings-binären hittades inte bredvid myllm eller i PATH",
    settings_spawn_tmpl: "kunde inte starta Settings: {}",
};

const TR: Strings = Strings {
    input: "Girdi",
    output: "Çıktı",
    run: "Çalıştır",
    copy: "Kopyala",
    open_window: "Pencereyi aç",
    reload_config: "Yapılandırmayı yenile",
    open_config: "Yapılandırma klasörünü aç",
    settings: "Ayarlar…",
    quit: "My LLM uygulamasından çık",
    config_created: "Başlangıç yapılandırması şurada oluşturuldu:",
    ok: "Tamam",
    wayland_hint: "Wayland'de compositor kısayolunu `myllm --task <id>` olarak atayın.",
    clipboard_empty: "Seçili metin yok ve pano boş.",
    copied: "Kopyalandı",
    translate: "Çevir",
    config_created_title: "Yapılandırma oluşturuldu",
    hotkeys_unavailable_tmpl: "kısayollar kullanılamıyor: {}",
    tray_unavailable_tmpl: "tepsi kullanılamıyor: {}",
    settings_not_found: "Settings ikilisi myllm'in yanında veya PATH'te bulunamadı",
    settings_spawn_tmpl: "Settings başlatılamadı: {}",
};

const VI: Strings = Strings {
    input: "Đầu vào",
    output: "Đầu ra",
    run: "Chạy",
    copy: "Sao chép",
    open_window: "Mở cửa sổ",
    reload_config: "Tải lại cấu hình",
    open_config: "Mở thư mục cấu hình",
    settings: "Cài đặt…",
    quit: "Thoát My LLM",
    config_created: "Đã tạo tệp cấu hình khởi đầu tại:",
    ok: "OK",
    wayland_hint: "Trên Wayland, gán phím tắt compositor cho `myllm --task <id>`.",
    clipboard_empty: "Không có văn bản được chọn và clipboard trống.",
    copied: "Đã sao chép",
    translate: "Dịch",
    config_created_title: "Đã tạo cấu hình",
    hotkeys_unavailable_tmpl: "phím tắt không khả dụng: {}",
    tray_unavailable_tmpl: "khay hệ thống không khả dụng: {}",
    settings_not_found: "Không tìm thấy file Settings cạnh myllm hoặc trên PATH",
    settings_spawn_tmpl: "không thể khởi chạy Settings: {}",
};

pub fn t(config_ui_lang: Option<&str>, os_langs: &[String]) -> &'static Strings {
    match resolve(config_ui_lang, os_langs) {
        "ar" => &AR,
        "nl" => &NL,
        "fr" => &FR,
        "de" => &DE,
        "hi" => &HI,
        "it" => &IT,
        "ja" => &JA,
        "ko" => &KO,
        "zh" => &ZH,
        "pt" => &PT,
        "ru" => &RU,
        "es" => &ES,
        "sv" => &SV,
        "tr" => &TR,
        "vi" => &VI,
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
    use super::{format_hotkey, resolve, t, UI_LANGS};

    #[test]
    fn explicit_code_wins() {
        assert_eq!(resolve(Some("ja"), &["en-US".into()]), "ja");
        assert_eq!(resolve(Some("os"), &["ja-JP".into()]), "ja");
        assert_eq!(resolve(None, &["fr_FR".into()]), "fr");
        assert_eq!(resolve(Some("zz"), &[]), "en");
        assert_eq!(resolve(None, &[]), "en");
    }

    #[test]
    fn chrome_tables_cover_whichlang() {
        let expected = [
            ("ar", "تشغيل"),
            ("nl", "Uitvoeren"),
            ("en", "Run"),
            ("fr", "Exécuter"),
            ("de", "Ausführen"),
            ("hi", "चलाएँ"),
            ("it", "Esegui"),
            ("ja", "実行"),
            ("ko", "실행"),
            ("zh", "运行"),
            ("pt", "Executar"),
            ("ru", "Запуск"),
            ("es", "Ejecutar"),
            ("sv", "Kör"),
            ("tr", "Çalıştır"),
            ("vi", "Chạy"),
        ];
        assert_eq!(expected.len(), UI_LANGS.len());
        for (code, run) in expected {
            assert_eq!(t(Some(code), &[]).run, run, "{code}");
            assert!(
                t(Some(code), &[]).hotkeys_unavailable_tmpl.contains("{}"),
                "{code}"
            );
        }
        assert_eq!(t(Some("fr"), &["ja-JP".into()]).copied, "Copié");
        assert_eq!(t(Some("zz"), &[]).copied, "Copied");
    }

    #[test]
    fn notice_templates_substitute() {
        assert_eq!(
            t(Some("ja"), &[]).hotkeys_unavailable("boom"),
            "ホットキーを使えません: boom"
        );
        assert_eq!(
            t(Some("en"), &[]).tray_unavailable("no icon"),
            "tray unavailable: no icon"
        );
        assert_eq!(
            t(Some("de"), &[]).settings_spawn("denied"),
            "Settings konnte nicht gestartet werden: denied"
        );
    }

    #[test]
    fn schema_localized_maps_have_all_langs() {
        let src = include_str!("../../../schema.toml");
        let maps = loc_maps(src);
        assert_eq!(maps.len(), 41, "expected 41 localized schema maps");
        for (i, map) in maps.iter().enumerate() {
            for code in UI_LANGS {
                let key = format!("{code} =");
                assert!(map.contains(&key), "map {i} missing {code}: {map}");
            }
        }
    }

    fn loc_maps(src: &str) -> Vec<&str> {
        let mut maps = Vec::new();
        let mut from = 0;
        while let Some(rel) = src[from..].find("{ en =") {
            let start = from + rel;
            let Some(end) = matching_brace(src, start) else {
                break;
            };
            maps.push(&src[start..=end]);
            from = end + 1;
        }
        maps
    }

    fn matching_brace(src: &str, start: usize) -> Option<usize> {
        let mut depth = 0;
        let mut in_str = false;
        let mut chars = src[start..].char_indices().peekable();
        while let Some((off, c)) = chars.next() {
            if in_str {
                if c == '\\' {
                    chars.next();
                    continue;
                }
                if c == '"' {
                    in_str = false;
                }
                continue;
            }
            match c {
                '"' => in_str = true,
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(start + off);
                    }
                }
                _ => {}
            }
        }
        None
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
