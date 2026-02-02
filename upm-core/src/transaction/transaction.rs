// ============================================================================
// Imports
// ============================================================================
use std::path::PathBuf;

use crate::types::Package;
// ============================================================================
// Transaction
// ============================================================================
pub struct Transaction {
    id: String,
    operation: String,
    package: Package,
    status: TransactionStatus,
    started_at: SystemTime,
    completed_at: Option<SystemTime>,
    steps: Vec<TransactionStep>,
    ostree_previous_commit: Option<String>,
    ostree_new_commit: Option<String>,
    pid: u32,
}

impl Transaction {
    pub fn new(operation: &str, package: Package) -> Self;
    pub fn add_step(&mut self, step: TransactionStep);
    pub fn update_step(&mut self, name: &str, status: StepStatus) -> Result<()>;
    pub fn mark_completed(&mut self);
    pub fn mark_failed(&mut self);
    pub fn duration(&self) -> Option<Duration>;

    fn id(&self) -> &str {
        &self.id
    }

    fn operation(&self) -> &str {
        &self.operation
    }

    fn package(&self) -> &Package {
        &self.package
    }

    fn status(&self) -> &TransactionStatus {
        &self.status
    }

    fn started_at(&self) -> &SystemTime {
        &self.started_at
    }

    fn completed_at(&self) -> &Option<SystemTime> {
        &self.completed_at
    }

    fn steps(&self) -> &Vec<TransactionStep> {
        &self.steps
    }

    fn steps(&self) -> &Vec<TransactionStep> {
        &self.steps
    }

    fn ostree_previous_commit(&self) -> &Option<String> {
        &self.ostree_previous_commit
    }

    fn ostree_new_commit(&self) -> &Option<String> {
        &self.ostree_new_commit
    }

    fn pid(&self) -> u32 {
        self.pid
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionStatus {
    InProgress,
    Completed,
    Failed,
    RolledBack,
}
