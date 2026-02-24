use core::backend::ExtractedPackage;

use package_ostree::errors::OStreeError;

use std::path::{Path, PathBuf};

mod helpers;
pub mod installer;

pub type Result<T> = std::result::Result<T, InstallerError>;

#[derive(Debug)]
pub enum InstallerError {
    IoError(std::io::Error),
    PathNotFoundError(std::io::Error),
    DatabaseError(database::Error),
    OStreeError(String),
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

impl From<OStreeError> for InstallerError {
    fn from(err: OStreeError) -> Self {
        match err {
            OStreeError::IoError(e) => InstallerError::IoError(e),
            OStreeError::NotFound(p) => InstallerError::PathNotFoundError(std::io::Error::other(p.display().to_string())),
            OStreeError::RepoPathError(p) => InstallerError::IoError(std::io::Error::other(format!("Repo path error: {}", p.display()))),
            OStreeError::CommitNotFound(s) => InstallerError::OStreeError(s),
            OStreeError::OSTreeCommitFailed(s) => InstallerError::OStreeError(s),
            OStreeError::OSTreeFailed(s) => InstallerError::OStreeError(s),
        }
    }
}


pub trait Installer {
    fn install_packages(&mut self, packages: Vec<&ExtractedPackage>) -> Result<()>;

    fn remove_packages(&mut self, packages: Vec<&str>) -> Result<()>;

    fn list_package_files(&self, package_id: &str) -> Result<Option<Vec<PathBuf>>>;

    fn add_file_to_package(&mut self, package_id: &str, file_path: &Path) -> Result<()>;

    fn remove_file_from_package(&mut self, file_path: &Path) -> Result<()>;
}
