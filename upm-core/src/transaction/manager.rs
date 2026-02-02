use crate::types::Package;

pub struct TransactionManager {
    transactions_dir: PathBuf,
    current_transaction: Option<Transaction>,
}

impl TransactionManager {
    pub fn new(transactions_dir: PathBuf) -> Self;

    pub fn begin_transaction(&mut self, operation: &str, package: &Package) -> Result<Transaction> {
    }

    pub fn commit_transaction(&mut self, txn: Transaction) -> Result<()> {}

    pub fn rollback_transaction(&mut self, txn: Transaction) -> Result<()> {}

    pub fn add_step(&mut self, step: TransactionStep) -> Result<()> {}

    pub fn update_step_status(&mut self, step_name: &str, status: StepStatus) -> Result<()> {}

    pub fn get_active_transactions(&self) -> Result<Vec<Transaction>> {}

    pub fn get_transaction(&self, id: &str) -> Result<Option<Transaction>> {}

    fn save_transaction(&self, txn: &Transaction) -> Result<()> {}

    fn load_transaction(&self, path: &Path) -> Result<Transaction> {}

    fn move_to_completed(&self, txn: &Transaction) -> Result<()> {}

    fn move_to_failed(&self, txn: &Transaction) -> Result<()> {}
}
