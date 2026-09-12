use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::error::{config_missing, Error, Result};
use crate::lang::{detect_source, language_name};

pub const TEMPLATE: &str = include_str!("../../../config/config.toml");
pub const TRANSLATE_TASK: &str = "translate";

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Appearance {
    #[default]
    Dark,
    Light,
    System,
}

impl Appearance {
    pub fn parse(raw: Option<&str>) -> Self {
        match raw.map(|s| s.trim()) {
            Some("light") => Self::Light,
            Some("system") => Self::System,
            _ => Self::Dark,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TranslationEngine {
    #[default]
    TranslateGemma,
    Custom,
}

impl TranslationEngine {
    pub fn parse(raw: Option<&str>) -> Self {
        match raw.map(|s| s.trim()) {
            Some("custom") => Self::Custom,
            _ => Self::TranslateGemma,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::TranslateGemma => "translategemma",
            Self::Custom => "custom",
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct General {
    #[serde(default = "default_provider_name")]
    pub default_provider: String,
    #[serde(default = "default_true")]
    pub auto_copy: Option<bool>,
    pub appearance: Option<String>,
    #[serde(default = "default_keep_alive")]
    pub keep_alive: String,
}

impl Default for General {
    fn default() -> Self {
        Self {
            default_provider: default_provider_name(),
            auto_copy: default_true(),
            appearance: None,
            keep_alive: default_keep_alive(),
        }
    }
}

fn default_provider_name() -> String {
    "ollama".to_string()
}

fn default_true() -> Option<bool> {
    Some(true)
}

fn default_keep_alive() -> String {
    "5m".to_string()
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Provider {
    pub base_url: Option<String>,
    pub default_model: Option<String>,
    pub api_key: Option<String>,
    pub api_key_env: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Task {
    pub name: Option<String>,
    pub instruction: Option<String>,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub hotkey: Option<String>,
    pub auto_copy: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Translation {
    #[serde(default = "default_true_bool")]
    pub enabled: bool,
    pub engine: Option<String>,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub hotkey: Option<String>,
    pub auto_copy: Option<bool>,
    #[serde(default = "default_en")]
    pub default_source: String,
    #[serde(default = "default_ja")]
    pub default_target: String,
    #[serde(default = "default_en")]
    pub fallback_target: String,
    pub instruction: Option<String>,
}

impl Default for Translation {
    fn default() -> Self {
        Self {
            enabled: true,
            engine: None,
            provider: None,
            model: None,
            hotkey: None,
            auto_copy: None,
            default_source: default_en(),
            default_target: default_ja(),
            fallback_target: default_en(),
            instruction: None,
        }
    }
}

fn default_true_bool() -> bool {
    true
}

fn default_en() -> String {
    "en".to_string()
}

fn default_ja() -> String {
    "ja".to_string()
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub general: General,
    #[serde(default)]
    pub providers: BTreeMap<String, Provider>,
    #[serde(default)]
    pub tasks: BTreeMap<String, Task>,
    pub translation: Option<Translation>,
}

#[derive(Debug, Clone)]
pub struct ResolvedRun {
    pub id: String,
    pub display_name: String,
    pub prompt: String,
    pub provider: String,
    pub model: String,
    pub base_url: String,
    pub api_key: Option<String>,
    pub keep_alive: String,
    pub auto_copy: bool,
    pub hotkey: Option<String>,
}

impl Config {
    pub fn load_path(path: &Path) -> Result<Self> {
        let raw = fs::read_to_string(path).map_err(|err| {
            if err.kind() == std::io::ErrorKind::NotFound {
                config_missing(&path.to_path_buf())
            } else {
                Error::Io(err)
            }
        })?;
        Ok(toml::from_str(&raw)?)
    }

    pub fn load_or_bootstrap() -> Result<(Self, PathBuf, bool)> {
        let path = config_file_path();
        if path.exists() {
            return Ok((Self::load_path(&path)?, path, false));
        }
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }
        fs::write(&path, TEMPLATE)?;
        Ok((Self::load_path(&path)?, path, true))
    }

    pub fn appearance(&self) -> Appearance {
        Appearance::parse(self.general.appearance.as_deref())
    }

    pub fn translation(&self) -> Translation {
        self.translation.clone().unwrap_or_default()
    }

    pub fn translation_engine(&self) -> TranslationEngine {
        TranslationEngine::parse(self.translation().engine.as_deref())
    }

    pub fn resolve_task(&self, id: &str, input: &str) -> Result<ResolvedRun> {
        if id == TRANSLATE_TASK {
            return Err(Error::Config(
                "translate is not a [tasks] entry; use resolve_translation".into(),
            ));
        }
        let task = self
            .tasks
            .get(id)
            .ok_or_else(|| Error::Config(format!("task '{id}' not found in configuration")))?;
        let instruction = task.instruction.as_deref().unwrap_or("").trim();
        if instruction.is_empty() {
            return Err(Error::Config(format!(
                "task '{id}' has no instruction in configuration"
            )));
        }
        let provider = self.resolve_provider_name(task.provider.as_deref())?;
        let model = self.resolve_model(task.model.as_deref(), &provider)?;
        let spec = self.provider_spec(&provider)?;
        let prompt = format!("{instruction}\n\n{input}");
        Ok(ResolvedRun {
            id: id.to_string(),
            display_name: task
                .name
                .clone()
                .unwrap_or_else(|| title_case(id)),
            prompt,
            provider,
            model,
            base_url: spec.base_url,
            api_key: spec.api_key,
            keep_alive: self.general.keep_alive.clone(),
            auto_copy: resolve_auto_copy(task.auto_copy, self.general.auto_copy),
            hotkey: task.hotkey.clone(),
        })
    }

    pub fn resolve_translation(
        &self,
        input: &str,
        from: Option<&str>,
        to: Option<&str>,
    ) -> Result<ResolvedRun> {
        let tr = self.translation();
        if !tr.enabled {
            return Err(Error::Config("translation is disabled".into()));
        }
        let source = from
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(ToString::to_string)
            .unwrap_or_else(|| detect_source(input, &tr.default_source));
        let target = to
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(ToString::to_string)
            .unwrap_or_else(|| {
                if source == tr.default_source {
                    tr.default_target.clone()
                } else {
                    tr.fallback_target.clone()
                }
            });
        let prompt = match TranslationEngine::parse(tr.engine.as_deref()) {
            TranslationEngine::TranslateGemma => {
                translategemma_prompt(&source, &target, input)
            }
            TranslationEngine::Custom => {
                let instruction = tr.instruction.as_deref().unwrap_or("").trim();
                if instruction.is_empty() {
                    return Err(Error::Config(
                        "translation engine is custom but instruction is empty".into(),
                    ));
                }
                expand_custom_instruction(instruction, &source, &target, input)
            }
        };
        let provider = self.resolve_provider_name(tr.provider.as_deref())?;
        let model = self.resolve_model(tr.model.as_deref(), &provider)?;
        let spec = self.provider_spec(&provider)?;
        Ok(ResolvedRun {
            id: TRANSLATE_TASK.to_string(),
            display_name: "Translate".to_string(),
            prompt,
            provider,
            model,
            base_url: spec.base_url,
            api_key: spec.api_key,
            keep_alive: self.general.keep_alive.clone(),
            auto_copy: resolve_auto_copy(tr.auto_copy, self.general.auto_copy),
            hotkey: tr.hotkey.clone(),
        })
    }

    pub fn resolve_run(
        &self,
        task_id: &str,
        input: &str,
        from: Option<&str>,
        to: Option<&str>,
    ) -> Result<ResolvedRun> {
        if task_id == TRANSLATE_TASK {
            self.resolve_translation(input, from, to)
        } else {
            self.resolve_task(task_id, input)
        }
    }

    fn resolve_provider_name(&self, task_provider: Option<&str>) -> Result<String> {
        let name = task_provider
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or(self.general.default_provider.trim());
        if name.is_empty() {
            return Err(Error::Config("no provider specified".into()));
        }
        if !self.providers.contains_key(name) {
            return Err(Error::Config(format!(
                "provider '{name}' not found in configuration"
            )));
        }
        Ok(name.to_string())
    }

    fn resolve_model(&self, task_model: Option<&str>, provider: &str) -> Result<String> {
        if let Some(model) = task_model.map(str::trim).filter(|s| !s.is_empty()) {
            return Ok(model.to_string());
        }
        let Some(p) = self.providers.get(provider) else {
            return Err(Error::Config(format!(
                "provider '{provider}' not found in configuration"
            )));
        };
        p.default_model
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(ToString::to_string)
            .ok_or_else(|| {
                Error::Config(format!(
                    "no model specified and no default_model set for provider '{provider}'"
                ))
            })
    }

    fn provider_spec(&self, name: &str) -> Result<ProviderSpec> {
        let p = self.providers.get(name).ok_or_else(|| {
            Error::Config(format!("provider '{name}' not found in configuration"))
        })?;
        let base_url = p
            .base_url
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(ToString::to_string)
            .unwrap_or_else(|| default_base_url(name).to_string());
        Ok(ProviderSpec {
            base_url,
            api_key: resolve_api_key(p)?,
        })
    }
}

struct ProviderSpec {
    base_url: String,
    api_key: Option<String>,
}

pub fn config_file_path() -> PathBuf {
    if let Ok(p) = std::env::var("CONFIG_FILE") {
        if !p.trim().is_empty() {
            return PathBuf::from(p);
        }
    }
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        if !xdg.trim().is_empty() {
            return PathBuf::from(xdg).join("myllm").join("config.toml");
        }
    }
    #[cfg(windows)]
    {
        if let Some(base) = dirs::config_dir() {
            return base.join("myllm").join("config.toml");
        }
    }
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".config")
        .join("myllm")
        .join("config.toml")
}

pub fn resolve_auto_copy(section: Option<bool>, general: Option<bool>) -> bool {
    section.or(general).unwrap_or(true)
}

fn resolve_api_key(provider: &Provider) -> Result<Option<String>> {
    if let Some(key) = provider
        .api_key
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        return Ok(Some(key.to_string()));
    }
    let Some(env_name) = provider
        .api_key_env
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    else {
        return Ok(None);
    };
    if !env_name.starts_with("MYLLM_") {
        return Err(Error::Config(format!(
            "api_key_env '{env_name}' must start with MYLLM_"
        )));
    }
    match std::env::var(env_name) {
        Ok(value) if !value.is_empty() => Ok(Some(value)),
        _ => Err(Error::Config(format!(
            "API key not found. Set api_key or environment variable {env_name}"
        ))),
    }
}

fn default_base_url(provider: &str) -> &'static str {
    match provider {
        "openai" => "https://api.openai.com/v1",
        "anthropic" => "https://api.anthropic.com/v1",
        _ => "http://localhost:11434",
    }
}

fn title_case(id: &str) -> String {
    let mut chars = id.chars();
    match chars.next() {
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
        None => id.to_string(),
    }
}

pub fn translategemma_prompt(source: &str, target: &str, text: &str) -> String {
    let source_name = language_name(source);
    let target_name = language_name(target);
    format!(
        "You are a professional {source_name} ({source}) to {target_name} ({target}) translator. Your goal is to accurately convey the meaning and nuances of the original {source_name} text while adhering to {target_name} grammar, vocabulary, and cultural sensitivities.\n\
Produce only the {target_name} translation, without any additional explanations or commentary. Please translate the following {source_name} text into {target_name}:\n\
\n\
\n\
{text}"
    )
}

pub fn expand_custom_instruction(
    instruction: &str,
    source: &str,
    target: &str,
    text: &str,
) -> String {
    instruction
        .replace("{SOURCE_LANG}", &language_name(source))
        .replace("{SOURCE_CODE}", source)
        .replace("{TARGET_LANG}", &language_name(target))
        .replace("{TARGET_CODE}", target)
        .replace("{TEXT}", text)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(toml: &str) -> Config {
        toml::from_str(toml).expect("toml")
    }

    #[test]
    fn model_falls_back_to_provider_default() {
        let cfg = parse(
            r#"
[general]
default_provider = "ollama"

[providers.ollama]
base_url = "http://localhost:11434"
default_model = "llama3.2"

[tasks.polish]
instruction = "Fix this"
"#,
        );
        let run = cfg.resolve_task("polish", "hello").unwrap();
        assert_eq!(run.provider, "ollama");
        assert_eq!(run.model, "llama3.2");
        assert_eq!(run.prompt, "Fix this\n\nhello");
        assert!(run.auto_copy);
    }

    #[test]
    fn task_provider_and_model_override_defaults() {
        let cfg = parse(
            r#"
[general]
default_provider = "ollama"
auto_copy = true

[providers.ollama]
default_model = "llama3.2"

[providers.openai]
base_url = "https://api.openai.com/v1"
default_model = "gpt-4o"
api_key = "sk-test"

[tasks.polish]
provider = "openai"
model = "gpt-5.4"
auto_copy = false
instruction = "Fix"
"#,
        );
        let run = cfg.resolve_task("polish", "x").unwrap();
        assert_eq!(run.provider, "openai");
        assert_eq!(run.model, "gpt-5.4");
        assert!(!run.auto_copy);
        assert_eq!(run.api_key.as_deref(), Some("sk-test"));
    }

    #[test]
    fn missing_default_model_is_an_error() {
        let cfg = parse(
            r#"
[general]
default_provider = "ollama"

[providers.ollama]
base_url = "http://localhost:11434"

[tasks.polish]
instruction = "Fix"
"#,
        );
        let err = cfg.resolve_task("polish", "x").unwrap_err().to_string();
        assert!(err.contains("default_model"), "{err}");
    }

    #[test]
    fn auto_copy_general_false_without_task_override() {
        let cfg = parse(
            r#"
[general]
default_provider = "ollama"
auto_copy = false

[providers.ollama]
default_model = "llama3.2"

[tasks.polish]
instruction = "Fix"
"#,
        );
        assert!(!cfg.resolve_task("polish", "x").unwrap().auto_copy);
    }

    #[test]
    fn api_key_env_requires_myllm_prefix() {
        let cfg = parse(
            r#"
[general]
default_provider = "openai"

[providers.openai]
default_model = "gpt-4o"
api_key_env = "OPENAI_API_KEY"

[tasks.polish]
instruction = "Fix"
"#,
        );
        let err = cfg.resolve_task("polish", "x").unwrap_err().to_string();
        assert!(err.contains("MYLLM_"), "{err}");
    }

    fn translation_cfg() -> Config {
        parse(
            r#"
[general]
default_provider = "ollama"

[providers.ollama]
default_model = "llama3.2"

[translation]
enabled = true
engine = "translategemma"
provider = "ollama"
model = "translategemma:12b"
default_source = "en"
default_target = "ja"
fallback_target = "en"
instruction = "Custom {SOURCE_CODE}->{TARGET_CODE}: {TEXT}"
"#,
        )
    }

    #[test]
    fn translation_routes_default_source_to_default_target() {
        let run = translation_cfg()
            .resolve_translation("Hello world, this is English.", None, None)
            .unwrap();
        assert_eq!(run.id, "translate");
        assert_eq!(run.model, "translategemma:12b");
        assert!(run.prompt.contains("English (en) to Japanese (ja)"));
        assert!(
            run.prompt.contains("into Japanese:\n\n\nHello world, this is English."),
            "{}",
            run.prompt
        );
        assert!(!run.prompt.contains("Custom en"));
    }

    #[test]
    fn translation_routes_other_source_to_fallback() {
        let run = translation_cfg()
            .resolve_translation("これは日本語の文章です。", None, None)
            .unwrap();
        assert!(run.prompt.contains("Japanese (ja) to English (en)"));
    }

    #[test]
    fn translation_from_to_override_detection() {
        let run = translation_cfg()
            .resolve_translation("hello", Some("de"), Some("fr"))
            .unwrap();
        assert!(run.prompt.contains("German (de) to French (fr)"));
    }

    #[test]
    fn custom_engine_expands_variables_and_keeps_instruction() {
        let mut cfg = translation_cfg();
        cfg.translation.as_mut().unwrap().engine = Some("custom".into());
        let run = cfg
            .resolve_translation("bonjour", Some("fr"), Some("en"))
            .unwrap();
        assert_eq!(run.prompt, "Custom fr->en: bonjour");
    }

    #[test]
    fn missing_engine_defaults_to_translategemma() {
        let mut cfg = translation_cfg();
        cfg.translation.as_mut().unwrap().engine = None;
        assert_eq!(
            cfg.translation_engine(),
            TranslationEngine::TranslateGemma
        );
    }
}
