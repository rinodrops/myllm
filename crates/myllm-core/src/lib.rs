//! Shared library for My LLM. The GUI links this crate and does not reimplement providers.

pub mod config;
pub mod error;
pub mod lang;
pub mod provider;

pub use config::{
    config_file_path, Appearance, Config, EmptyWindowTask, ResolvedRun, TranslationEngine,
    TRANSLATE_TASK,
};
pub use error::{Error, Result};
pub use provider::stream_run;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
