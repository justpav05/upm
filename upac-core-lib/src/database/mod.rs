// Imports
use crate::core::types::PackageInfo;

use toml::{ser, de};

use std::io;
use std::path::{Path, PathBuf};

// Mods
pub mod database;
mod files;

// Enums for DatabaseError
#[derive(Debug)]
pub enum DatabaseError {
    IoError(std::io::Error),
    TomlError(String),
    NotFound,
    LockError,
    PathError(PathBuf),
}

// Result type for DatabaseError
pub type Result<T> = std::result::Result<T, DatabaseError>;

// Implementations for converting io::Error to DatabaseError
impl From<io::Error> for DatabaseError {
    fn from(err: io::Error) -> Self {
        DatabaseError::IoError(err)
    }
}

// Implementations for converting ser::Error to DatabaseError
impl From<ser::Error> for DatabaseError {
    fn from(err: ser::Error) -> Self {
        DatabaseError::TomlError(err.to_string())
    }
}

// Implementations for converting de::Error to DatabaseError
impl From<de::Error> for DatabaseError {
    fn from(err: de::Error) -> Self {
        DatabaseError::TomlError(err.to_string())
    }
}

// Implementations for converting DatabaseError to human-readable string
impl std::fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError(e) => write!(f, "IO error: {}", e),
            Self::TomlError(e) => write!(f, "TOML error: {}", e),
            Self::NotFound => write!(f, "Not found"),
            Self::LockError => write!(f, "Lock error"),
            Self::PathError(p) => write!(f, "Path error: {}", p.display()),
        }
    }
}

// Trait for package registry operations
pub trait PackageRegistry {
    fn add_package(&mut self, package: &PackageInfo) -> Result<()>;

    fn remove_package(&mut self, package_id: &str) -> Result<()>;

    fn get_package(&self, package_id: &str) -> Result<Option<PackageInfo>>;

    fn list_all_packages(&self) -> Result<Vec<PackageInfo>>;

}

// Trait for file registry operations
pub trait FileRegistry {
    fn register_file(&mut self, package_id: &str, file_path: &Path) -> Result<()>;

    fn unregister_file(&mut self, file_path: &Path) -> Result<()>;

    fn get_files(&self, package_id: &str) -> Result<Vec<PathBuf>>;
}
