use crate::lock::LockType;

pub struct OperationInfo {
    pub pid: u32,
    pub lock_type: LockType,
    pub operation: String,
    pub package: Option<String>,
    pub user: String,
    pub started_at: SystemTime,
    pub progress_file: PathBuf,
}

impl OperationInfo {
    pub fn new(pid: u32, lock_type: LockType, operation: &str) -> Self;
    pub fn with_package(mut self, package: &str) -> Self;
    pub fn elapsed(&self) -> Duration;
}
