use glib;

use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, OStreeError>;

#[derive(Debug)]
pub enum OStreeError {
    IoError(std::io::Error),
    NotFound(PathBuf),
    RepoPathError(PathBuf),
    CommitNotFound(String),
    OSTreeCommitFailed(String),
    OSTreeFailed(String),
}

impl From<std::io::Error> for OStreeError {
    fn from(err: std::io::Error) -> Self {
        OStreeError::IoError(err)
    }
}

impl From<nix::Error> for OStreeError {
    fn from(err: nix::Error) -> Self {
        OStreeError::IoError(std::io::Error::other(err))
    }
}

impl From<glib::Error> for OStreeError {
    fn from(err: glib::Error) -> Self {
        OStreeError::OSTreeCommitFailed(err.to_string())
    }
}

impl From<ostree::glib::Error> for OStreeError {
    fn from(err: ostree::glib::Error) -> Self {
        OStreeError::OSTreeCommitFailed(err.to_string())
    }
}

impl From<database::Error> for OStreeError {
    fn from(err: database::Error) -> Self {
        match err {
            database::Error::IoError(e) => OStreeError::IoError(e),
            database::Error::TomlError(s) => OStreeError::OSTreeCommitFailed(s),
            database::Error::NotFound => OStreeError::OSTreeCommitFailed("Not found".to_string()),
            database::Error::LockError => OStreeError::OSTreeCommitFailed("Lock error".to_string()),
            database::Error::PathError(p) => OStreeError::RepoPathError(p),
        }
    }
}
