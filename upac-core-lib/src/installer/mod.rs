use crate::{PackageInfo, ExtractedPackage};
use crate::backup::OStreeError;
use crate::database;

use std::path::{Path, PathBuf};

pub mod installer;
mod helpers;

#[derive(Debug)]
pub enum InstallerError {
    IoError(std::io::Error),
    PathNotFoundError(std::io::Error),
    DatabaseError(database::DatabaseError),
    OStreeError(String),
}

pub type Result<T> = std::result::Result<T, InstallerError>;

impl From<std::io::Error> for InstallerError {
    fn from(err: std::io::Error) -> Self {
        InstallerError::IoError(err)
    }
}

impl From<database::DatabaseError> for InstallerError {
    fn from(err: database::DatabaseError) -> Self {
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

impl ToString for InstallerError {
    fn to_string(&self) -> String {
        format!("{:?}", self)
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

pub trait Install {
    fn install(&mut self, package: &ExtractedPackage) -> Result<()>;

    fn remove(&mut self, package_id: &str) -> Result<()>;

    fn list_files(&self, package_id: &str) -> Result<Vec<PathBuf>>;

    fn list_packages(&self) -> Result<Vec<PackageInfo>>;

    fn add_file(&mut self, package_id: &str, file_path: &Path) -> Result<()>;

    fn remove_file(&mut self, file_path: &Path) -> Result<()>;
}
