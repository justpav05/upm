use std::fs::File;
use std::time::SystemTime;

pub struct SharedLock {
    file: File,
    pid: u32,
    started_at: SystemTime,
}

impl Drop for SharedLock {
    fn drop(&mut self) {}
}

pub struct ExclusiveLock {
    file: File,
    pid: u32,
    started_at: SystemTime,
}

impl Drop for ExclusiveLock {
    fn drop(&mut self) {}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockType {
    Shared,
    Exclusive,
}

#[derive(Debug, Clone)]
pub struct LockInfo {
    pub pid: u32,
    pub lock_type: LockType,
    pub operation: String,
    pub package: Option<String>,
    pub started_at: SystemTime,
}
