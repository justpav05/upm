use core::types::{PackageInfo, PackageMetadata};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    TomlError(String),
    NotFound,
    PathError(PathBuf),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IoError(e)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub(crate) fn read_toml<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let content = std::fs::read_to_string(path)?;
    toml::from_str(&content).map_err(|e| Error::TomlError(e.to_string()))
}

pub(crate) fn write_toml<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let content = toml::to_string_pretty(value).map_err(|e| Error::TomlError(e.to_string()))?;
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, content)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

pub mod database;
pub mod index;

pub trait Database: Send + Sync {
    // ========================================
    // CRD operations with packages
    // ========================================

    fn add_package(&mut self, package: &PackageInfo) -> Result<()>;

    fn remove_package(&mut self, package_id: &str) -> Result<()>;

    fn get_package(&self, package_id: &str) -> Result<Option<PackageInfo>>;

    fn list_all_packages(&self) -> Result<Vec<PackageInfo>>;

    // ========================================
    // File management
    // ========================================

    fn register_file(&mut self, package_id: &str, file_path: &Path) -> Result<()>;

    fn unregister_file(&mut self, file_path: &Path) -> Result<()>;

    fn get_files(&self, package_id: &str) -> Result<Vec<PathBuf>>;
}
