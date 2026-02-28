use crate::PackageDiff;
use crate::database;

use ostree::glib;

use std::io;
use std::path::{PathBuf, Path};
use std::time::SystemTime;

pub mod manager;
mod rollback;
mod commit;
mod info;

#[derive(Debug, Clone)]
pub struct CommitInfo {
    pub hash: String,
    pub timestamp: SystemTime,
    pub package_list: Vec<String>,
    pub description: String,
}

#[derive(Debug)]
pub enum OStreeError {
    IoError(io::Error),
    NotFound(PathBuf),
    RepoPathError(PathBuf),
    CommitNotFound(String),
    OSTreeCommitFailed(String),
    OSTreeFailed(String),
}

pub type Result<T> = std::result::Result<T, OStreeError>;

impl From<io::Error> for OStreeError {
    fn from(err: io::Error) -> Self {
        OStreeError::IoError(err)
    }
}

impl From<nix::Error> for OStreeError {
    fn from(err: nix::Error) -> Self {
        OStreeError::IoError(io::Error::other(err))
    }
}

impl From<glib::Error> for OStreeError {
    fn from(err: glib::Error) -> Self {
        OStreeError::OSTreeCommitFailed(err.to_string())
    }
}

impl From<database::DatabaseError> for OStreeError {
    fn from(err: database::DatabaseError) -> Self {
        match err {
            database::DatabaseError::IoError(err) => OStreeError::IoError(err),
            database::DatabaseError::TomlError(string) => OStreeError::OSTreeCommitFailed(string),
            database::DatabaseError::NotFound => OStreeError::OSTreeCommitFailed("Not found".to_string()),
            database::DatabaseError::LockError => OStreeError::OSTreeCommitFailed("Lock error".to_string()),
            database::DatabaseError::PathError(path_buf) => OStreeError::RepoPathError(path_buf),
        }
    }
}

pub trait PackageRepo {
    fn commit(&self, files: Vec<PathBuf>, diff: &PackageDiff) -> Result<String>;

    fn delete(&self, ref_name: &str) -> Result<()>;

    fn rollback_to(&self, commit_hash: &str, root_dir: &Path) -> Result<()>;

    fn get_commit_info(&self, commit_hash: &str) -> Result<CommitInfo>;
}
