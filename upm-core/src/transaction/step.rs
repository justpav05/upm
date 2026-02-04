// ============================================================================
// Imports
// ============================================================================
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

use crate::types::Error;
// ============================================================================
// Transaction step
// ============================================================================
pub struct TransactionStep {
    name: String,
    status: StepStatus,
    timestamp: SystemTime,
    details: HashMap<String, String>,
}

impl TransactionStep {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            status: StepStatus::Pending,
            timestamp: SystemTime::now(),
            details: HashMap::new(),
        }
    }

    pub fn with_details(name: &str, details: HashMap<String, String>) -> Self {
        Self {
            name: name.to_string(),
            status: StepStatus::Pending,
            timestamp: SystemTime::now(),
            details: details,
        }
    }

    pub fn mark_completed(&mut self) {
        self.status = StepStatus::Completed;
    }

    pub fn mark_failed(&mut self) {
        self.status = StepStatus::Failed;
    }

    pub fn duration_since(&self) -> Result<Duration, Error> {
        SystemTime::now()
            .duration_since(self.timestamp)
            .map_err(|error| Error::TimeStampError(format!("{}", error)))
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn status(&self) -> &StepStatus {
        &self.status
    }

    pub fn timestamp(&self) -> &SystemTime {
        &self.timestamp
    }

    pub fn set_name(&mut self, name: &str) {
        self.name = String::from(name)
    }

    pub fn set_status(&mut self, status: StepStatus) {
        self.status = status
    }

    pub fn set_timestamp(&mut self, time: SystemTime) {
        self.timestamp = time
    }
}
// ============================================================================
// Step status
// ============================================================================
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}
