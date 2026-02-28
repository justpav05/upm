// Imports
use super::Lockable;

use nix::fcntl::{Flock, FlockArg};

use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};
use std::io;

// Type alias for lock Result<T, LockError>
pub type Result<T> = std::result::Result<T, LockError>;

// Error enum for lock operations
#[derive(Debug)]
pub enum LockError {
    IoError(io::Error),
    Nix(nix::Error),
    SharedLockBusy(PathBuf),
    ExclusiveLockBusy(PathBuf),
}

// Convert io::Error to LockError
impl From<io::Error> for LockError {
    fn from(err: io::Error) -> Self {
        LockError::IoError(err)
    }
}

// Convert nix::errno::Errno to LockError
impl From<(File, nix::errno::Errno)> for LockError {
    fn from((_, err): (File, nix::errno::Errno)) -> Self {
        LockError::Nix(err.into())
    }
}

// Shared lock struct for file locking
pub struct SharedLock {
    _flock: Flock<File>,
}

// Acquire a shared lock on a file
impl Lockable for SharedLock {

	// Function for getting a shared lock on a file
    fn acquire(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                return Err(LockError::IoError(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Directory does not exist: {}", parent.display()),
                )));
            }
        }
        let lock_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;
        let flock = Flock::lock(lock_file, FlockArg::LockExclusive)?;
        Ok(Self { _flock: flock })
    }
}

// Exclusive lock struct for file locking
pub struct ExclusiveLock {
    _flock: Flock<File>,
}

// Acquire an exclusive lock on a file
impl Lockable for ExclusiveLock {

	// Function for getting an exclusive lock on a file
    fn acquire(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                return Err(LockError::IoError(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Directory does not exist: {}", parent.display()),
                )));
            }
        }
        let lock_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;
        let flock = Flock::lock(lock_file, FlockArg::LockExclusive)?;
        Ok(Self { _flock: flock })
    }
}
