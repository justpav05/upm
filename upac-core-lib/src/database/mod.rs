use crate::core::types::PackageInfo;

use toml::{ser, de};

use std::io;
use std::path::{Path, PathBuf};

pub type Result<T> = std::result::Result<T, Error>;

pub mod database;
pub mod index;
mod helpers;

#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    TomlError(String),
    NotFound,
    LockError,
    PathError(PathBuf),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::IoError(err)
    }
}

impl From<ser::Error> for Error {
    fn from(err: ser::Error) -> Self {
        Error::TomlError(err.to_string())
    }
}

impl From<de::Error> for Error {
    fn from(err: de::Error) -> Self {
        Error::TomlError(err.to_string())
    }
}

impl ToString for Error {
    fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}

pub trait Database: Send + Sync {
    fn add_package(&mut self, package: &PackageInfo) -> Result<()>;

    fn remove_package(&mut self, package_id: &str) -> Result<()>;

    fn get_package(&self, package_id: &str) -> Result<Option<PackageInfo>>;

    fn list_all_packages(&self) -> Result<Vec<PackageInfo>>;

    fn register_file(&mut self, package_id: &str, file_path: &Path) -> Result<()>;

    fn unregister_file(&mut self, file_path: &Path) -> Result<()>;

    fn get_files(&self, package_id: &str) -> Result<Vec<PathBuf>>;
}

pub trait Index {
    fn load(index_path: PathBuf) -> Result<Self>
    where
        Self: Sized;

    fn save(&self) -> Result<()>;

    fn insert(&mut self, name: &str, version: &str, format: &str);

    fn remove(&mut self, package_info: &str);

    fn get(&self, package_info: &str) -> Option<&PackageInfo>;

    fn search(&self, query: &str) -> Vec<&PackageInfo>;

    fn list_all(&self) -> Vec<&PackageInfo>;
}
