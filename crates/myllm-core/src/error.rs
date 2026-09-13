use std::fmt;
use std::io;
use std::path::PathBuf;

#[derive(Debug)]
pub enum Error {
    Config(String),
    Provider(String),
    Io(io::Error),
    Http(String),
    Toml(toml::de::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Config(msg) | Self::Provider(msg) | Self::Http(msg) => f.write_str(msg),
            Self::Io(err) => write!(f, "{err}"),
            Self::Toml(err) => write!(f, "invalid config.toml: {err}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<toml::de::Error> for Error {
    fn from(err: toml::de::Error) -> Self {
        Self::Toml(err)
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Self::Http(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub fn config_missing(path: &PathBuf) -> Error {
    Error::Config(format!("configuration file not found: {}", path.display()))
}
