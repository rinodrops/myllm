//! Shared library for My LLM. The GUI links this crate and does not reimplement providers.

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
