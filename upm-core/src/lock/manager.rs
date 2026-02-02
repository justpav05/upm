use crate::operations::ActiveOperationsTracker;

pub struct LockManager {
    lock_file_path: PathBuf,
    operations_tracker: ActiveOperationsTracker,
}

impl LockManager {
    pub fn new(lock_file_path: PathBuf, tracker: ActiveOperationsTracker) -> Self;
    
    // Blocking acquire
    pub fn acquire_shared(&self) -> Result<SharedLock>;
    pub fn acquire_exclusive(&self) -> Result<ExclusiveLock>;
    
    // Non-blocking acquire
    pub fn try_acquire_shared(&self) -> Result<Option<SharedLock>>;
    pub fn try_acquire_exclusive(&self) -> Result<Option<ExclusiveLock>>;
    
    // Status
    pub fn is_locked(&self) -> bool;
    pub fn get_lock_info(&self) -> Result<Vec<LockInfo>>;
    
    // Internal
    fn show_waiting_message(&self) -> Result<()>;
}