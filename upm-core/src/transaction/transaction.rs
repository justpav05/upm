use crate::types::Package;

pub struct Transaction {
    pub id: String,
    pub operation: String,
    pub package: Package,
    pub status: TransactionStatus,
    pub started_at: SystemTime,
    pub completed_at: Option<SystemTime>,
    pub steps: Vec<TransactionStep>,
    pub ostree_previous_commit: Option<String>,
    pub ostree_new_commit: Option<String>,
    pub pid: u32,
}

impl Transaction {
    pub fn new(operation: &str, package: Package) -> Self;
    pub fn add_step(&mut self, step: TransactionStep);
    pub fn update_step(&mut self, name: &str, status: StepStatus) -> Result<()>;
    pub fn mark_completed(&mut self);
    pub fn mark_failed(&mut self);
    pub fn duration(&self) -> Option<Duration>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionStatus {
    InProgress,
    Completed,
    Failed,
    RolledBack,
}
