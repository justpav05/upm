// ============================================================================
// Imports
// ============================================================================
use std::fs::OpenOptions;
use std::os::unix::io::AsRawFd;
use std::path::PathBuf;
use std::time::SystemTime;

use libc::{F_GETLK, fcntl};

use crate::lock::types::LockType::Shared;
use crate::lock::types::{ExclusiveLock, LockInfo, SharedLock};
use crate::operations::ActiveOperationsTracker;
use crate::types::{Error, Result};
// ============================================================================
// Lock manager
// ============================================================================
pub struct LockManager {
    lock_file_path: PathBuf,
    operations_tracker: ActiveOperationsTracker,
}

impl LockManager {
    pub fn new(lock_file_path: PathBuf, tracker: ActiveOperationsTracker) -> Self {
        Self {
            lock_file_path: lock_file_path,
            operations_tracker: tracker,
        }
    }

    pub fn acquire_shared(&self) -> Result<SharedLock> {
        let file_descriptor = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&self.lock_file_path)?;

        SharedLock::new(file_descriptor).map_err(Error::IoError)
    }

    pub fn acquire_exclusive(&self) -> Result<ExclusiveLock> {
        let file_descriptor = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&self.lock_file_path)?;

        ExclusiveLock::new(file_descriptor).map_err(Error::IoError)
    }

    pub fn try_acquire_shared(&self) -> Result<Option<SharedLock>> {
        let file_descriptor = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&self.lock_file_path)?;

        match SharedLock::try_new(file_descriptor) {
            Ok(lock) => Ok(Some(lock)),
            Err(_) => Ok(None),
        }
    }

    pub fn try_acquire_exclusive(&self) -> Result<Option<ExclusiveLock>> {
        let file_descriptor = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&self.lock_file_path)?;

        match ExclusiveLock::try_new(file_descriptor) {
            Ok(lock) => Ok(Some(lock)),
            Err(_) => Ok(None),
        }
    }

    pub fn is_locked(&self) -> bool {
        let file_descriptor = match OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&self.lock_file_path)
        {
            Ok(file_descriptor) => file_descriptor,
            Err(_) => return false,
        };

        #[repr(C)]
        struct Flock {
            l_type: i32,
            l_whence: i16,
            l_start: i64,
            l_len: i64,
            l_pid: i32,
        }

        let mut flock = Flock {
            l_type: libc::F_WRLCK,
            l_whence: 0,
            l_start: 0,
            l_len: 0,
            l_pid: 0,
        };

        let result = unsafe { fcntl(file_descriptor.as_raw_fd(), F_GETLK, &mut flock) };

        if result == -1 {
            return false;
        }

        flock.l_type != libc::F_WRLCK
    }

    pub fn get_lock_info(&self) -> Result<Vec<LockInfo>> {
        let file_descriptor = match OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&self.lock_file_path)
        {
            Ok(file_descriptor) => file_descriptor,
            Err(_) => return Ok(vec![]),
        };

        let mut locks = Vec::new();

        #[repr(C)]
        struct Flock {
            l_type: i32,
            l_whence: i16,
            l_start: i64,
            l_len: i64,
            l_pid: i32,
        }

        let mut flock = Flock {
            l_type: libc::F_RDLCK,
            l_whence: 0,
            l_start: 0,
            l_len: 0,
            l_pid: 0,
        };

        if unsafe { fcntl(file_descriptor.as_raw_fd(), F_GETLK, &mut flock) } != -1 {
            if flock.l_type == libc::F_RDLCK {
                let lock_info = LockInfo {
                    pid: flock.l_pid as u32,
                    lock_type: Shared,
                    operation: "shared_lock".to_string(), //TODO: Сделать получение нормальной операции через ActiveOperationTracker
                    package: None, //TODO: Сделать получение информации о операции с текущем пакетом ActiveOperationTracker
                    started_at: SystemTime::now(),
                };
                locks.push(lock_info);
            }
        }

        let mut flock = Flock {
            l_type: libc::F_WRLCK,
            l_whence: 0,
            l_start: 0,
            l_len: 0,
            l_pid: 0,
        };

        if unsafe { fcntl(file_descriptor.as_raw_fd(), F_GETLK, &mut flock) } != -1 {
            if flock.l_type == libc::F_WRLCK {
                let lock_info = LockInfo {
                    pid: flock.l_pid as u32,
                    lock_type: crate::lock::types::LockType::Exclusive,
                    operation: "exclusive_lock".to_string(), //TODO: Сделать получение нормальной операции через ActiveOperationTracker
                    package: None, //TODO: Сделать получение информации о операции с текущем пакетом ActiveOperationTracker
                    started_at: SystemTime::now(),
                };
                locks.push(lock_info);
            }
        }

        Ok(locks)
    }

    fn show_waiting_message(&self) -> Result<()> {
        let locks = self.operations_tracker.get_active_operations()?;
        if locks.is_empty() {
            return Err(Error::LockError((String::from("Lock file is empty"))));
        }

        let package_str = locks.package.as_deref().unwrap_or("");
        if package_str.is_empty() {
            return Err(Error::TransactionError(
                (format!("No package specified: {}", package_str)),
            ));
        }

        for lock in &locks {
            println!("   {} {}", lock.operation, package_str);
        }

        println!("   Waiting for lock...");

        Ok(())
    }
}
