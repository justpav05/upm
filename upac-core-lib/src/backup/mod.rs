// Imports
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

// Structure for commit information
#[derive(Debug, Clone)]
pub struct CommitInfo {
    pub hash: String,
    pub timestamp: SystemTime,
    pub package_list: Vec<String>,
    pub description: String,
}

// Enum for OStree errors
#[derive(Debug)]
pub enum OStreeError {
    IoError(io::Error),
    NotFound(PathBuf),
    RepoPathError(PathBuf),
    CommitNotFound(String),
    OSTreeCommitFailed(String),
    OSTreeFailed(String),
}

// Type alias for Result<T, OStreeError>
pub type Result<T> = std::result::Result<T, OStreeError>;

// Implement From for io::Error to OStreeError
impl From<io::Error> for OStreeError {
    fn from(err: io::Error) -> Self {
        OStreeError::IoError(err)
    }
}

// Implement From for nix::Error to OStreeError
impl From<nix::Error> for OStreeError {
    fn from(err: nix::Error) -> Self {
        OStreeError::IoError(io::Error::other(err))
    }
}

// Implement From for glib::Error to OStreeError
impl From<glib::Error> for OStreeError {
    fn from(err: glib::Error) -> Self {
        OStreeError::OSTreeCommitFailed(err.to_string())
    }
}

// Implement From for database::DatabaseError to OStreeError
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

// Trait for commits and rollbacks
pub trait PackageRepo {
    fn commit(&self, files: Vec<PathBuf>, diff: &PackageDiff) -> Result<String>;

    fn delete(&self, ref_name: &str) -> Result<()>;

    fn rollback_to(&self, commit_hash: &str, root_dir: &Path) -> Result<()>;

    fn get_commit_info(&self, commit_hash: &str) -> Result<CommitInfo>;
}

// Trait for resolving refs and finding commits
pub trait OStreeRefCommitChange {
    fn resolve_ref(&self, ref_name: &str) -> Result<String>;

    fn find_ref(&self, commit_hash: &str) -> Result<Option<String>>;
}
