use stabby::result::Result as StabbyResult;
use stabby::string::String as StabString;

use toml::{de, ser};

use ostree::glib;

use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::io;
use std::path::PathBuf;
use std::path::StripPrefixError;

#[derive(Debug)]
pub enum LockError {
    IoError(io::Error),
    Nix(nix::Error),
    SharedLockBusy(PathBuf),
    ExclusiveLockBusy(PathBuf),
}

impl From<io::Error> for LockError {
    fn from(err: io::Error) -> Self {
        LockError::IoError(err)
    }
}

impl From<nix::Error> for LockError {
    fn from(err: nix::Error) -> Self {
        LockError::Nix(err)
    }
}

impl<T> From<(T, nix::errno::Errno)> for LockError {
    fn from((_, err): (T, nix::errno::Errno)) -> Self {
        LockError::Nix(nix::Error::from(err))
    }
}

// Enums for config errors
#[repr(stabby)]
#[stabby::stabby]
pub enum ConfigError {
    IoError(StabString),
    ParseError(StabString),
    PathError(StabString),
}
// Implementations for io::Error to config errors
impl From<io::Error> for ConfigError {
    fn from(err: io::Error) -> Self {
        ConfigError::IoError(err.to_string().into())
    }
}

impl From<de::Error> for ConfigError {
    fn from(err: toml::de::Error) -> Self {
        ConfigError::ParseError(err.to_string().into())
    }
}

impl Debug for ConfigError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self}")
    }
}

impl Display for ConfigError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let msg = self.0.match_ref(
            |err| format!("IO error: {err}"),
            |inner| {
                inner.match_ref(
                    |msg| format!("Failed to parse config: {msg}"),
                    |path| format!("Invalid path: {path}"),
                )
            },
        );
        write!(formatter, "{msg}")
    }
}

#[derive(Debug)]
pub enum DatabaseError {
    Io(io::Error),
    Toml(String),
    NotFound,
    Lock,
    Path(PathBuf),
}

impl From<io::Error> for DatabaseError {
    fn from(err: io::Error) -> Self {
        DatabaseError::Io(err)
    }
}

impl From<ser::Error> for DatabaseError {
    fn from(err: ser::Error) -> Self {
        DatabaseError::Toml(err.to_string())
    }
}

impl From<de::Error> for DatabaseError {
    fn from(err: de::Error) -> Self {
        DatabaseError::Toml(err.to_string())
    }
}

impl From<LockError> for DatabaseError {
    fn from(_: LockError) -> Self {
        DatabaseError::Lock
    }
}

// Implementations for converting DatabaseError to human-readable string
impl Display for DatabaseError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => write!(formatter, "IO error: {}", err),
            Self::Toml(err) => write!(formatter, "TOML error: {}", err),
            Self::NotFound => write!(formatter, "Not found"),
            Self::Lock => write!(formatter, "Lock error"),
            Self::Path(path) => write!(formatter, "Path error: {}", path.display()),
        }
    }
}

#[repr(stabby)]
#[stabby::stabby]
pub enum InstallerError {
    Io(StabString),
    Lock(StabString),
    Database(StabString),
    Installer(StabString),
    Dependency(StabString),
}

impl From<io::Error> for InstallerError {
    fn from(err: io::Error) -> Self {
        Self::Io(err.to_string().into())
    }
}

impl From<nix::Error> for InstallerError {
    fn from(err: nix::Error) -> Self {
        Self::Io(err.to_string().into())
    }
}

impl From<StripPrefixError> for InstallerError {
    fn from(err: StripPrefixError) -> Self {
        Self::Io(err.to_string().into())
    }
}

impl From<glib::Error> for InstallerError {
    fn from(err: glib::Error) -> Self {
        Self::Installer(err.to_string().into())
    }
}

// Конвертации внутренних ошибок в стабильные
impl From<LockError> for InstallerError {
    fn from(err: LockError) -> Self {
        let msg = match err {
            LockError::IoError(e) => format!("IO error: {e}"),
            LockError::Nix(e) => format!("Nix error: {e}"),
            LockError::SharedLockBusy(p) => format!("Shared lock busy: {}", p.display()),
            LockError::ExclusiveLockBusy(p) => format!("Exclusive lock busy: {}", p.display()),
        };
        Self::Lock(msg.into())
    }
}

impl From<DatabaseError> for InstallerError {
    fn from(err: DatabaseError) -> Self {
        let msg = match err {
            DatabaseError::Io(err) => format!("IO error: {err}"),
            DatabaseError::Toml(err) => format!("TOML error: {err}"),
            DatabaseError::NotFound => "Not found".to_string(),
            DatabaseError::Lock => "Lock error".to_string(),
            DatabaseError::Path(path) => format!("Path error: {}", path.display()),
        };
        Self::Database(msg.into())
    }
}

#[repr(stabby)]
#[stabby::stabby]
pub enum OSTreeError {
    RepoNotFound(StabString),
    CommitFailed(StabString),
    RollbackFailed(StabString),
    RemoveFailed(StabString),
    Io(StabString),
}

impl From<std::io::Error> for OSTreeError {
    fn from(err: std::io::Error) -> Self {
        OSTreeError::Io(err.to_string().into())
    }
}

impl From<glib::Error> for OSTreeError {
    fn from(err: glib::Error) -> Self {
        OSTreeError::Io(err.to_string().into())
    }
}

impl From<LockError> for OSTreeError {
    fn from(err: LockError) -> Self {
        let msg = match err {
            LockError::IoError(err) => format!("IO error: {err}"),
            LockError::Nix(err) => format!("Nix error: {err}"),
            LockError::SharedLockBusy(path) => format!("Shared lock busy: {}", path.display()),
            LockError::ExclusiveLockBusy(path) => {
                format!("Exclusive lock busy: {}", path.display())
            }
        };
        OSTreeError::Io(msg.into())
    }
}

impl Display for OSTreeError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        let msg = self.match_ref(
            |msg| format!("Repository not found: {msg}"),
            |msg| format!("Commit failed: {msg}"),
            |msg| format!("Rollback failed: {msg}"),
            |msg| format!("Remove failed: {msg}"),
            |msg| format!("IO error: {msg}"),
        );
        write!(formatter, "{msg}")
    }
}

// Алиасы
pub type LockResult<T> = Result<T, LockError>;
pub type DatabaseResult<T> = Result<T, DatabaseError>;

pub type ConfigResult<T> = Result<T, ConfigError>;

pub type OSTreeResult<T> = Result<T, OSTreeError>;
pub type OSTreeStabbyResult<T> = StabbyResult<T, OSTreeError>;

pub type InstallerResult<T> = Result<T, InstallerError>;
pub type InstallerStabbyResult<T> = StabbyResult<T, InstallerError>;
