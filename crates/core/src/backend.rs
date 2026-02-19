use serde::{Deserialize, Serialize};
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

impl Drop for ExtractedPackage {
    fn drop(&mut self) {
        self.files.iter().for_each(|file| {
            let _ = std::fs::remove_file(&file.relative_path);
        });
    }
}

pub trait Backend: Send + Sync {
    fn name(&self) -> &str;
    fn supported_formats(&self) -> Vec<&str>;
    fn detect(&self, path: &Path) -> bool;
    fn extract(&self, path: &Path, temp_dir: &Path) -> Result<ExtractedPackage>;
}
