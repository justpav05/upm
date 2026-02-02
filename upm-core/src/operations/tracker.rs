use crate::lock::LockType;

pub struct ActiveOperationsTracker {
    file_path: PathBuf,
}

impl ActiveOperationsTracker {
    pub fn new(file_path: PathBuf) -> Self;

    // Registration
    pub fn register_operation(&self, info: OperationInfo) -> Result<()>;
    pub fn unregister_operation(&self, pid: u32) -> Result<()>;

    // Queries
    pub fn get_active_operations(&self) -> Result<Vec<OperationInfo>>;
    pub fn get_operation(&self, pid: u32) -> Result<Option<OperationInfo>>;

    // Cleanup
    pub fn cleanup_dead_operations(&self) -> Result<()>;

    // Internal
    fn load_operations(&self) -> Result<Vec<OperationInfo>>;
    fn save_operations(&self, ops: &[OperationInfo]) -> Result<()>;
}
