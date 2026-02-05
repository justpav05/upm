// ============================================================================
// Imports
// ============================================================================
use libc::getpid;
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};
use uuid::Uuid;

use crate::transaction::{StepStatus, TransactionStep};
use crate::types::Package;
use crate::types::{Error, Result};
// ============================================================================
// Transaction
// ============================================================================
pub struct Transaction {
    id: Uuid,
    operation: String,
    package: Package,
    status: TransactionStatus,
    started_at: SystemTime,
    completed_at: Option<SystemTime>,
    steps: Vec<TransactionStep>,
    pid: u32,
}

impl Transaction {
    pub fn new(operation: &str, package: Package) -> Self {
        Transaction {
            id: Uuid::new_v4(),
            operation: String::from(operation),
            package,
            status: TransactionStatus::InProgress,
            started_at: SystemTime::now(),
            completed_at: None,
            steps: Vec::new(),
            pid: unsafe { getpid() as u32 },
        }
    }
    pub fn add_step(&mut self, step: TransactionStep) {
        self.steps.push(step);
    }

    pub fn update_step(&mut self, name: &str, status: StepStatus) -> Result<()> {
        for step in &mut self.steps {
            if step.name() == name {
                step.set_status(status);
                step.set_timestamp(SystemTime::now());

                return Ok(());
            }
        }

        Err(Error::StepNotFound(String::from(name)))
    }

    pub fn mark_completed(&mut self) {
        self.completed_at = Some(SystemTime::now());
        self.status = TransactionStatus::Completed;
    }

    pub fn mark_failed(&mut self) {
        self.completed_at = Some(SystemTime::now());
        self.status = TransactionStatus::Failed;
    }

    pub fn duration(&self) -> Option<Duration> {
        match SystemTime::now().duration_since(self.started_at) {
            Ok(duration) => Some(duration),
            Err(_) => None,
        }
    }

    pub fn id(&self) -> String {
        String::from(self.id)
    }

    pub fn status(&self) -> TransactionStatus {
        self.status
    }

    pub fn steps(&self) -> &Vec<TransactionStep> {
        &self.steps
    }

    pub fn completed_at(&self) -> Option<SystemTime> {
        self.completed_at
    }

    pub fn set_status(&mut self, status: TransactionStatus) {
        self.status = status;
    }

    pub fn set_completed_at(&mut self, completed_at: Option<SystemTime>) {
        self.completed_at = completed_at;
    }
}
// ============================================================================
// Transaction status
// ============================================================================
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionStatus {
    InProgress,
    Completed,
    Failed,
    RolledBack,
}
