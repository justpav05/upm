// Imports
use crate::{PackageInfo, ExtractedPackage};
use crate::backup::OStreeError;
use crate::database;

use std::path::{Path, PathBuf};

// Mods
pub mod installer;

// Enums for Installer Errors
#[derive(Debug)]
pub enum InstallerError {
    IoError(std::io::Error),
    PathNotFoundError(std::io::Error),
    DatabaseError(database::DatabaseError),
    OStreeError(String),
}

// Implement Install trait for Installer
pub type Result<T> = std::result::Result<T, InstallerError>;

// Implement for std::io::Error to InstallerError
impl From<std::io::Error> for InstallerError {
    fn from(err: std::io::Error) -> Self {
        InstallerError::IoError(err)
    }
}

// Implement for database::DatabaseError to InstallerError
impl From<database::DatabaseError> for InstallerError {
    fn from(err: database::DatabaseError) -> Self {
        InstallerError::DatabaseError(err)
    }
}

// Implement for nix::Error to InstallerError
impl From<nix::Error> for InstallerError {
    fn from(err: nix::Error) -> Self {
        InstallerError::IoError(std::io::Error::other(err))
    }
}

// Implement for std::sync::PoisonError to InstallerError
impl<T> From<std::sync::PoisonError<T>> for InstallerError {
    fn from(err: std::sync::PoisonError<T>) -> Self {
        InstallerError::IoError(std::io::Error::other(err.to_string()))
    }
}

// Implement for display InstallerError as human-readable string
impl std::fmt::Display for InstallerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError(err) => write!(f, "IO error: {}", err),
            Self::PathNotFoundError(err) => write!(f, "Path not found: {}", err),
            Self::DatabaseError(err) => write!(f, "Database error: {}", err),
            Self::OStreeError(err) => write!(f, "OStree error: {}", err),
        }
    }
}


// Implement for OStreeError to InstallerError
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

// An unfinished state machine for determining the state of an installation
#[derive(Debug, Clone)]
pub enum InstallEvent {
    InstallStarted  { package: String, total_files: usize },
    FileInstalled   { path: PathBuf, current: usize, total: usize },
    CommitCreated   { commit_hash: String },
    InstallFinished { package: String },

    RemoveStarted   { package: String },
    FileRemoved     { path: PathBuf },
    RemoveFinished  { package: String },

    Failed          { package: String, reason: String },
}

// Trait for the installer
pub trait Install {
    fn install(&mut self, package: &ExtractedPackage) -> Result<()>;

    fn remove(&mut self, package_id: &str) -> Result<()>;

    fn list_files(&self, package_id: &str) -> Result<Vec<PathBuf>>;

    fn list_packages(&self) -> Result<Vec<PackageInfo>>;

    fn add_file(&mut self, package_id: &str, file_path: &Path) -> Result<()>;

    fn remove_file(&mut self, file_path: &Path) -> Result<()>;
}
