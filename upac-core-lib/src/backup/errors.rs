use crate::database;

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

// impl From<glib::Error> for OStreeError {
//     fn from(err: glib::Error) -> Self {
//         OStreeError::OSTreeCommitFailed(err.to_string())
//     }
// }

impl From<ostree::glib::Error> for OStreeError {
    fn from(err: ostree::glib::Error) -> Self {
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
