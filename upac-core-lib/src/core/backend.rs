// Imports
use super::types::PackageMetadata;

use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::path::{Path, PathBuf};

// Alias for Result<T, BackendError>
pub type Result<T> = std::result::Result<T, BackendError>;

// Backend error enum
#[derive(Debug)]
pub enum BackendError {
    Io(std::io::Error),
    InvalidPackage(String),
    UnsupportedFormat(String),
}

// Convert std::io::Error to BackendError
impl From<std::io::Error> for BackendError {
    fn from(e: std::io::Error) -> Self {
        BackendError::Io(e)
    }
}

// Display implementation for BackendError
impl Display for BackendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BackendError::Io(err) => write!(f, "IO error: {err}"),
            BackendError::InvalidPackage(string) => write!(f, "Invalid package: {string}"),
            BackendError::UnsupportedFormat(string) => write!(f, "Unsupported format: {string}"),
        }
    }
}

// Structure for working with an unpacked package
pub struct ExtractedPackage {
    pub name: String,
    pub version: String,
    pub format: String,
    pub files: Vec<FileEntry>,
}

// File entry struct for working with package files
#[derive(Serialize, Deserialize)]
pub struct FileEntry {
    pub relative_path: PathBuf,
    pub permissions: u32,
    #[serde(default)]
    pub owner: u32,
    #[serde(default)]
    pub group: u32,
}

// Trait for backend implementations
pub trait Backend: Send + Sync {
    fn name(&self) -> &str;

    fn supported_formats(&self) -> Vec<&str>;

    fn detect(&self, path: &Path) -> bool;

    fn extract(&self, path: &Path, temp_dir: &Path) -> Result<ExtractedPackage>;

    fn read_metadata(&self, path: &Path) -> Result<PackageMetadata>;
}
