use crate::events::InstallEvent;
use crate::installer::InstallerManager;

use core::backend::ExtractedPackage;

use database::Database;

use package_ostree::errors::OStreeError;

use package_ostree::implement::OStreeManager;

use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;

mod events;
mod helpers;
pub mod implement;
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


pub trait Installer {
	fn new(database: Box<dyn Database>, root_dir: PathBuf, package_dir: PathBuf, temp_dir: PathBuf, ostree: OStreeManager, event_tx: Sender<InstallEvent>) -> InstallerManager;

    fn install_packages(&mut self, packages: Vec<&ExtractedPackage>, ostree_backup: bool) -> Result<()>;

    fn remove_packages(&mut self, packages: Vec<&str>, ostree_backup: bool) -> Result<()>;

    fn list_package_files(&self, package_id: &str) -> Result<Option<Vec<PathBuf>>>;

    fn add_file_to_package(&mut self, package_id: &str, file_path: &Path) -> Result<()>;

    fn remove_file_from_package(&mut self, file_path: &Path) -> Result<()>;
}
