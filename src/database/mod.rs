// Imports
use crate::types::{ExtractedPackage, Package};

use crate::lock::LockError;

use toml::{ser, de};

use std::fmt::{Formatter, Display, Result};
use std::path::{Path, PathBuf};
use std::io;

// Mods
pub mod database;

pub use database::PackageDatabase;

// Enums for DatabaseError
#[derive(Debug)]
pub enum DatabaseError {
    Io(std::io::Error),
    Toml(String),
    NotFound,
    Lock,
    Path(PathBuf),
}

// Result type for DatabaseError
pub type DatabaseResult<T> = std::result::Result<T, DatabaseError>;

// Implementations for converting io::Error to DatabaseError
impl From<io::Error> for DatabaseError {
    fn from(err: io::Error) -> Self {
        DatabaseError::Io(err)
    }
}

// Implementations for converting ser::Error to DatabaseError
impl From<ser::Error> for DatabaseError {
    fn from(err: ser::Error) -> Self {
        DatabaseError::Toml(err.to_string())
    }
}

// Implementations for converting de::Error to DatabaseError
impl From<de::Error> for DatabaseError {
    fn from(err: de::Error) -> Self {
        DatabaseError::Toml(err.to_string())
    }
}

impl From<LockError> for DatabaseError {
    fn from(_err: LockError) -> Self {
        DatabaseError::Lock
    }
}

// Implementations for converting DatabaseError to human-readable string
impl Display for DatabaseError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        match self {
            Self::Io(err) => write!(formatter, "IO error: {}", err),
            Self::Toml(err) => write!(formatter, "TOML error: {}", err),
            Self::NotFound => write!(formatter, "Not found"),
            Self::Lock => write!(formatter, "Lock error"),
            Self::Path(path) => write!(formatter, "Path error: {}", path.display()),
        }
    }
}

// Trait for package registry operations
pub trait Database {
    fn add_package(&mut self, package: &ExtractedPackage) -> DatabaseResult<()>;
    fn remove_package(&mut self, package_id: &str) -> DatabaseResult<()>;
    fn get_package(&self, query: &str) -> DatabaseResult<Package>;
    fn get_package_files(&self, package_id: &str) -> DatabaseResult<Vec<PathBuf>>;
    fn add_file(&mut self, package_id: &str, file_path: &Path) -> DatabaseResult<()>;
    fn remove_file(&mut self, package_id: &str, file_path: &Path) -> DatabaseResult<()>;

}
