// Imports
use toml::de;

use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use std::io;

// Mods
pub mod config;
mod files;

// Enums for config errors
#[derive(Debug)]
pub enum ConfigError {
    IoError(io::Error),
    ParseError(String),
    PathError(PathBuf),
}

// Alias for config errors
pub type Result<T> = std::result::Result<T, ConfigError>;

// Implementations for io::Error to config errors
impl From<io::Error> for ConfigError {
    fn from(err: io::Error) -> Self {
        ConfigError::IoError(err)
    }
}

// Implementations for toml::de::Error to config errors
impl From<de::Error> for ConfigError {
    fn from(err: de::Error) -> Self {
        ConfigError::ParseError(err.to_string())
    }
}

// Implementations for display config errors
impl Display for ConfigError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::IoError(err)    => write!(f, "IO error: {err}"),
            ConfigError::ParseError(msg) => write!(f, "Failed to parse config: {msg}"),
            ConfigError::PathError(path) => write!(f, "Invalid path: {}", path.display()),
        }
    }
}

// Trait for functions to load config
pub trait Config: Sized {

    fn load() -> Result<Self>;

    fn default_config() -> Self;

    fn validate(&self) -> Result<()>;
}
