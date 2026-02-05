// ============================================================================
// Imports
// ============================================================================
use libc::{LOCK_EX, LOCK_NB, LOCK_SH, LOCK_UN, flock};
use std::fs::File;
use std::os::unix::io::AsRawFd;
use std::time::SystemTime;
// ============================================================================
// Shared lock
// ============================================================================
pub struct SharedLock {
    file: File,
    pid: u32,
    started_at: SystemTime,
}

impl SharedLock {
    pub(super) fn new(file: File) -> Result<Self, std::io::Error> {
        if unsafe { flock(file.as_raw_fd(), LOCK_SH) } != 0 {
            return Err(std::io::Error::last_os_error());
        }

        Ok(Self {
            file,
            pid: std::process::id(),
            started_at: SystemTime::now(),
        })
    }

    pub(super) fn try_new(file: File) -> Result<Self, std::io::Error> {
        if unsafe { flock(file.as_raw_fd(), LOCK_SH | LOCK_NB) } != 0 {
            return Err(std::io::Error::last_os_error());
        }

        Ok(Self {
            file,
            pid: std::process::id(),
            started_at: SystemTime::now(),
        })
    }

    fn file(&self) -> &File {
        &self.file
    }

    fn pid(&self) -> u32 {
        self.pid
    }

    fn started_at(&self) -> SystemTime {
        self.started_at
    }
}

impl Drop for SharedLock {
    fn drop(&mut self) {
        unsafe {
            flock(self.file.as_raw_fd(), LOCK_UN);
        }
    }
}
// ============================================================================
// Exclusive lock
// ============================================================================
pub struct ExclusiveLock {
    file: File,
    pid: u32,
    started_at: SystemTime,
}

impl ExclusiveLock {
    pub(super) fn new(file: File) -> Result<Self, std::io::Error> {
        unsafe {
            if flock(file.as_raw_fd(), LOCK_EX) != 0 {
                return Err(std::io::Error::last_os_error());
            }
        }

        Ok(Self {
            file,
            pid: std::process::id(),
            started_at: SystemTime::now(),
        })
    }

    pub(super) fn try_new(file: File) -> Result<Self, std::io::Error> {
        unsafe {
            if flock(file.as_raw_fd(), LOCK_EX | LOCK_NB) != 0 {
                return Err(std::io::Error::last_os_error());
            }
        }

        Ok(Self {
            file,
            pid: std::process::id(),
            started_at: SystemTime::now(),
        })
    }

    fn file(&self) -> &File {
        &self.file
    }

    fn pid(&self) -> u32 {
        self.pid
    }

    fn started_at(&self) -> SystemTime {
        self.started_at
    }
}

impl Drop for ExclusiveLock {
    fn drop(&mut self) {
        unsafe {
            flock(self.file.as_raw_fd(), LOCK_UN);
        }
    }
}
// ============================================================================
// Lock type
// ============================================================================
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockType {
    Shared,
    Exclusive,
}
// ============================================================================
// Lock info
// ============================================================================
#[derive(Debug, Clone)]
pub struct LockInfo {
    pub pid: u32,
    pub lock_type: LockType,
    pub operation: String,
    pub package: Option<String>,
    pub started_at: SystemTime,
}
