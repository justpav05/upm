use super::types::PackageMetadata;

use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::path::{Path, PathBuf};

pub type Result<T> = std::result::Result<T, BackendError>;

#[derive(Debug)]
pub enum BackendError {
    Io(std::io::Error),
    InvalidPackage(String),
    UnsupportedFormat(String),
}

impl From<std::io::Error> for BackendError {
    fn from(e: std::io::Error) -> Self {
        BackendError::Io(e)
    }
}

impl Display for BackendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BackendError::Io(e) => write!(f, "IO error: {e}"),
            BackendError::InvalidPackage(s) => write!(f, "Invalid package: {s}"),
            BackendError::UnsupportedFormat(s) => write!(f, "Unsupported format: {s}"),
        }
    }
}

pub struct ExtractedPackage {
    pub name: String,
    pub version: String,
    pub format: String,
    pub files: Vec<FileEntry>,
}
#[derive(Serialize, Deserialize)]
pub struct FileEntry {
    pub relative_path: PathBuf,
    pub permissions: u32,
    #[serde(default)]
    pub owner: u32,
    #[serde(default)]
    pub group: u32,
}

pub trait Backend: Send + Sync {
    fn name(&self) -> &str;
    fn supported_formats(&self) -> Vec<&str>;
    fn detect(&self, path: &Path) -> bool;
    fn extract(&self, path: &Path, temp_dir: &Path) -> Result<ExtractedPackage>;
    fn read_metadata(&self, path: &Path) -> Result<PackageMetadata>;
}
