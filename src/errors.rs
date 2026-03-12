use crate::lock::LockError;
use crate::database::DatabaseError;

use stabby::result::Result as StabbyResult;
use stabby::string::String as StabString;

use std::path::StripPrefixError;
use std::io;

#[repr(stabby)]
#[stabby::stabby]
#[derive(Debug)]
pub enum InstallerError {
    // File operations
    IoError(StabString),
    PathIsDir(StabString),
    PathIsFile(StabString),
    PathNotFound(StabString),

    // Lock file
    LockBusy,
    LockFailed,

    // Database
    DatabaseNotFound,
    DatabaseCorrupted(StabString),
    DatabaseReadFailed(StabString),
    DatabaseWriteFailed(StabString),

    // Installer
    PackageNotFound(StabString),
    PackageAlreadyInstalled(StabString),
    InstallFailed(StabString),
}

pub type InstallerResult<T> = Result<T, InstallerError>;
pub type InstallerStabbyResult<T> = StabbyResult<T, InstallerError>;

impl From<LockError> for InstallerError {
    fn from(err: LockError) -> Self {
        match err {
            LockError::IoError(_)           => Self::LockFailed(),
            LockError::Nix(_)               => Self::LockFailed(),
            LockError::SharedLockBusy(_)    => Self::LockBusy(),
            LockError::ExclusiveLockBusy(_) => Self::LockBusy(),
        }
    }
}

impl From<io::Error> for InstallerError {
    fn from(err: io::Error) -> Self {
        match err.kind() {
            io::ErrorKind::NotFound => Self::PathNotFound(err.to_string().into()),
            _                       => Self::IoError(err.to_string().into()),
        }
    }
}

impl From<nix::Error> for InstallerError {
    fn from(err: nix::Error) -> Self {
        Self::IoError(err.to_string().into())
    }
}

impl From<StripPrefixError> for InstallerError {
    fn from(err: StripPrefixError) -> Self {
        Self::PathNotFound(err.to_string().into())
    }
}

impl From<DatabaseError> for InstallerError {
    fn from(err: DatabaseError) -> Self {
        match err {
            DatabaseError::Io(err)       => Self::IoError(err.to_string().into()),
            DatabaseError::Toml(message) => Self::DatabaseWriteFailed(message.into()),
            DatabaseError::NotFound      => Self::DatabaseNotFound(),
            DatabaseError::Lock          => Self::LockFailed(),
            DatabaseError::Path(path)    => Self::PathNotFound(path.display().to_string().into()),
        }
    }
}
