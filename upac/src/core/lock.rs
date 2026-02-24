use nix::fcntl::{Flock, FlockArg};
use std::fs::{File, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};

pub type Result<T> = std::result::Result<T, LockError>;

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

impl From<(File, nix::errno::Errno)> for LockError {
    fn from((_, err): (File, nix::errno::Errno)) -> Self {
        LockError::Nix(err.into())
    }
}

pub struct SharedLock {
    _flock: Flock<File>,
}

impl SharedLock {
    pub fn acquire(path: &Path) -> Result<Self> {
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

        let flock = Flock::lock(lock_file, FlockArg::LockShared)?;

        Ok(Self { _flock: flock })
    }
}

pub struct ExclusiveLock {
    _flock: Flock<File>,
}

impl ExclusiveLock {
    pub fn acquire(path: &Path) -> Result<Self> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;
    use tempfile::NamedTempFile;

    #[test]
    fn shared_lock_acquire_and_release() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path();

        {
            let _lock = SharedLock::acquire(path).unwrap();
        }

        let _lock2 = SharedLock::acquire(path).unwrap();
    }

    #[test]
    fn exclusive_lock_acquire_and_release() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path();

        {
            let _lock = ExclusiveLock::acquire(path).unwrap();
        }

        let _lock2 = ExclusiveLock::acquire(path).unwrap();
    }

    #[test]
    fn multiple_shared_locks() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path().to_path_buf();

        let _lock1 = SharedLock::acquire(&path).unwrap();
        let _lock2 = SharedLock::acquire(&path).unwrap();
        let _lock3 = SharedLock::acquire(&path).unwrap();
    }

    #[test]
    fn exclusive_blocks_shared() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path().to_path_buf();
        let path_clone = path.clone();

        let _exclusive = ExclusiveLock::acquire(&path).unwrap();

        let handle = thread::spawn(move || SharedLock::acquire(&path_clone));

        thread::sleep(Duration::from_millis(100));

        drop(_exclusive);

        let result = handle.join().unwrap();
        assert!(result.is_ok());
    }

    #[test]
    fn shared_blocks_exclusive() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path().to_path_buf();
        let path_clone = path.clone();

        let _shared = SharedLock::acquire(&path).unwrap();

        let handle = thread::spawn(move || ExclusiveLock::acquire(&path_clone));

        thread::sleep(Duration::from_millis(100));

        drop(_shared);

        let result = handle.join().unwrap();
        assert!(result.is_ok());
    }

    #[test]
    fn test_nonexistent_path() {
        let result = SharedLock::acquire(Path::new("/nonexistent/path/lock.file"));
        assert!(result.is_err());
    }
}
