use crate::lock::LockError;
use crate::database::DatabaseError;

use std::path::StripPrefixError;
use std::io;

#[derive(Debug)]
pub enum InstallerError {
    // File operations
    IoError(String),
    PathIsDir(String),
    PathIsFile(String),
    PathNotFound(String),

    // Lock file
    LockBusy,
    LockFailed,

    // Database
    DatabaseNotFound,
    DatabaseCorrupted(String),
    DatabaseReadFailed(String),
    DatabaseWriteFailed(String),

    // Installer
    PackageNotFound(String),
    PackageAlreadyInstalled(String),
    InstallFailed(String),
}

pub type InstallerResult<T> = Result<T, InstallerError>;

impl From<LockError> for InstallerError {
    fn from(err: LockError) -> Self {
        match err {
            LockError::IoError(_)  => Self::LockFailed,
            LockError::Nix(_) => Self::LockFailed,
            LockError::SharedLockBusy(_) => Self::LockBusy,
            LockError::ExclusiveLockBusy(_) => Self::LockBusy,
        }
    }
}

impl From<io::Error> for InstallerError {
    fn from(err: io::Error) -> Self {
        match err.kind() {
            io::ErrorKind::NotFound => Self::PathNotFound(err.to_string()),
            _ => Self::IoError(err.to_string()),
        }
    }
}

impl From<nix::Error> for InstallerError {
    fn from(err: nix::Error) -> Self {
        Self::IoError(err.to_string())
    }
}

impl From<StripPrefixError> for InstallerError {
    fn from(err: StripPrefixError) -> Self {
        Self::PathNotFound(err.to_string())
    }
}

impl From<DatabaseError> for InstallerError {
    fn from(err: DatabaseError) -> Self {
        match err {
            DatabaseError::Io(err) => Self::IoError(err.to_string()),
            DatabaseError::Toml(message) => Self::DatabaseWriteFailed(message),
            DatabaseError::NotFound => Self::DatabaseNotFound,
            DatabaseError::Lock => Self::LockFailed,
            DatabaseError::Path(path) => Self::PathNotFound(path.display().to_string()),
        }
    }
}
