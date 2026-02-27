use crate::core::backend::ExtractedPackage;

use crate::database;

use crate::backup::errors::OStreeError;

use std::path::{Path, PathBuf};

pub mod installer;
mod helpers;

pub type Result<T> = std::result::Result<T, InstallerError>;

#[derive(Debug)]
pub enum InstallerError {
    IoError(std::io::Error),
    PathNotFoundError(std::io::Error),
    DatabaseError(database::DatabaseError),
    OStreeError(String),
}

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
    fn install_package(&mut self, package: &ExtractedPackage, ostree_backup: bool) -> Result<()>;

    fn remove_package(&mut self, package: &str, ostree_backup: bool) -> Result<()>;

    fn list_package_files(&self, package: &str) -> Result<Option<Vec<PathBuf>>>;

    fn add_file_to_package(&mut self, package: &str, path: &Path) -> Result<()>;

    fn remove_file_from_package(&mut self, path: &Path) -> Result<()>;

    fn install_packages<'a>(
        &mut self,
        packages: impl IntoIterator<Item = &'a ExtractedPackage>,
        ostree_backup: bool,
    ) -> Result<()> {
        packages.into_iter().try_for_each(|package| self.install_package(package, ostree_backup))
    }

    fn remove_packages<'a>(
        &mut self,
        packages: impl IntoIterator<Item = &'a str>,
        ostree_backup: bool,
    ) -> Result<()> {
        packages.into_iter().try_for_each(|package| self.remove_package(package, ostree_backup))
    }
}
