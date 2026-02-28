use crate::core::types::PackageInfo;

use toml::{ser, de};

use std::io;
use std::path::{Path, PathBuf};

pub mod database;
mod help;

#[derive(Debug)]
pub enum DatabaseError {
    IoError(std::io::Error),
    TomlError(String),
    NotFound,
    LockError,
    PathError(PathBuf),
}

pub type Result<T> = std::result::Result<T, DatabaseError>;

impl From<io::Error> for DatabaseError {
    fn from(err: io::Error) -> Self {
        DatabaseError::IoError(err)
    }
}

impl From<ser::Error> for DatabaseError {
    fn from(err: ser::Error) -> Self {
        DatabaseError::TomlError(err.to_string())
    }
}

impl From<de::Error> for DatabaseError {
    fn from(err: de::Error) -> Self {
        DatabaseError::TomlError(err.to_string())
    }
}

impl ToString for DatabaseError {
    fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}

pub trait PackageRegistry {
    fn add_package(&mut self, package: &PackageInfo) -> Result<()>;

    fn remove_package(&mut self, package_id: &str) -> Result<()>;

    fn get_package(&self, package_id: &str) -> Result<Option<PackageInfo>>;

    fn list_all_packages(&self) -> Result<Vec<PackageInfo>>;

}

pub trait FileRegistry {
    fn register_file(&mut self, package_id: &str, file_path: &Path) -> Result<()>;

    fn unregister_file(&mut self, file_path: &Path) -> Result<()>;

    fn get_files(&self, package_id: &str) -> Result<Vec<PathBuf>>;
}
