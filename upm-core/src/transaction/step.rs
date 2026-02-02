pub struct TransactionStep {
    pub name: String,
    pub status: StepStatus,
    pub timestamp: SystemTime,
    pub details: HashMap<String, String>,
}

impl TransactionStep {
    pub fn new(name: &str) -> Self {}

    pub fn with_details(name: &str, details: HashMap<String, String>) -> Self {}

    pub fn mark_completed(&mut self) {}

    pub fn mark_failed(&mut self) {}

    pub fn duration_since(&self) -> Duration {}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}
