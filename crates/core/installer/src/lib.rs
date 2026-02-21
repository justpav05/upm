use core::backend::ExtractedPackage;
use std::path::{Path, PathBuf};

pub mod installer;

pub type Result<T> = std::result::Result<T, InstallerError>;

#[derive(Debug)]
pub enum InstallerError {
    IoError(std::io::Error),
    PathNotFoundError(std::io::Error),
    DatabaseError(database::Error),
}

impl From<std::io::Error> for InstallerError {
    fn from(err: std::io::Error) -> Self {
        InstallerError::IoError(err)
    }
}

impl From<database::Error> for InstallerError {
    fn from(err: database::Error) -> Self {
        InstallerError::DatabaseError(err)
    }
}

impl From<nix::Error> for InstallerError {
    fn from(err: nix::Error) -> Self {
        InstallerError::IoError(std::io::Error::other(err))
    }
}

impl<T> From<std::sync::PoisonError<T>> for InstallerError {
    fn from(err: std::sync::PoisonError<T>) -> Self {
        InstallerError::IoError(std::io::Error::other(err.to_string()))
    }
}

pub trait Installer: Send + Sync {
    fn install_package(&mut self, package: &ExtractedPackage) -> Result<()>;

    fn remove_package(&mut self, package_id: &str) -> Result<()>;

    fn list_package_files(&self, package_id: &str) -> Result<Option<Vec<PathBuf>>>;

    fn add_file_to_package(&mut self, package_id: &str, file_path: &Path) -> Result<()>;

    fn remove_file_from_package(&mut self, file_path: &Path) -> Result<()>;
}
