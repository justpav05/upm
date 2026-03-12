// Imports
use nix::fcntl::{Flock, FlockArg, FcntlArg, fcntl};
use nix::libc::{flock, F_WRLCK, F_UNLCK, SEEK_SET};

use std::fs::{File, OpenOptions};
use std::os::unix::io::AsRawFd;
use std::path::PathBuf;
use std::io;

// Type alias for lock Result<T, LockError>
pub type LockResult<T> = std::result::Result<T, LockError>;

// Error enum for lock operations
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

pub trait Lock {
	fn lock(&self) -> LockResult<Flock<File>>;
	fn is_lock(&self) -> LockResult<bool>;
}

// Shared lock struct for file locking
pub struct SharedLock {
	file_path: PathBuf,
}

impl SharedLock {
	pub fn new(file_path: PathBuf) -> Self {
		Self { file_path }
	}
}

// Acquire a shared lock on a file
impl Lock for SharedLock {
	fn lock(&self) -> LockResult<Flock<File>> {
		if let Some(parent) = self.file_path.parent() {
            if !parent.exists() {
                return Err(LockError::IoError(io::Error::new(io::ErrorKind::NotFound,
                format!("Directory does not exist: {}", parent.display()))));
            }
        }

        let lock_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&self.file_path)?;
        let lock_guard = Flock::lock(lock_file, FlockArg::LockShared)?;
        Ok(lock_guard)
	}

	fn is_lock(&self) -> LockResult<bool> {
		let file = File::open(&self.file_path)?;
		let file_descriptor = file.as_raw_fd();

		let mut lock = flock {
    		l_type:   F_WRLCK as i16,
      		l_whence: SEEK_SET as i16,
      		l_start:  0,
      		l_len:    0,
      		l_pid:    0,
		};

		fcntl(file_descriptor, FcntlArg::F_GETLK(&mut lock))?;
		Ok(lock.l_type != F_UNLCK as i16)
	}
}

// Exclusive lock struct for file locking
pub struct ExclusiveLock {
	file_path: PathBuf,
}

impl ExclusiveLock {
	pub fn new(file_path: PathBuf) -> Self {
		Self { file_path }
	}
}

impl Lock for ExclusiveLock {
	fn lock(&self) -> LockResult<Flock<File>> {
		if let Some(parent) = self.file_path.parent() {
            if !parent.exists() {
                return Err(LockError::IoError(io::Error::new(io::ErrorKind::NotFound,
                format!("Directory does not exist: {}", parent.display()))));
            }
        }

        let lock_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&self.file_path)?;
        let lock_guard = Flock::lock(lock_file, FlockArg::LockExclusive)?;
        Ok(lock_guard)
	}

	fn is_lock(&self) -> LockResult<bool> {
		let file = File::open(&self.file_path)?;
		let file_descriptor = file.as_raw_fd();

		let mut lock = flock {
    		l_type:   F_WRLCK as i16,
      		l_whence: SEEK_SET as i16,
      		l_start:  0,
      		l_len:    0,
      		l_pid:    0,
		};

		fcntl(file_descriptor, FcntlArg::F_GETLK(&mut lock))?;
		Ok(lock.l_type != F_UNLCK as i16)
	}
}
