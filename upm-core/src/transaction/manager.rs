// ============================================================================
// Imports
// ============================================================================
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::transaction::transaction;
use crate::transaction::{StepStatus, TransactionStep};
use crate::transaction::{Transaction, TransactionStatus};
use crate::types::Error;
use crate::types::Package;
use crate::utils;

pub type Result<T> = std::result::Result<T, Error>;
// ============================================================================
// Transaction manager
// ============================================================================
pub struct TransactionManager {
    transactions_dir: PathBuf,
    current_transaction: Option<Transaction>,
}

impl TransactionManager {
    pub fn new(transactions_dir: PathBuf) -> Self {
        TransactionManager {
            transactions_dir,
            current_transaction: None,
        }
    }

    pub fn begin_transaction(&mut self, operation: &str, package: &Package) -> Result<Transaction> {
        let transaction = Transaction::new(operation, package.clone());

        self.current_transaction = Some(transaction.clone());
        self.save_transaction(&transaction)?;

        Ok(transaction)
    }

    pub fn commit_transaction(&mut self, mut transaction: Transaction) -> Result<()> {
        transaction.set_status(TransactionStatus::Completed);
        transaction.set_completed_at(Some(SystemTime::now()));

        self.move_to_completed(&transaction)?;

        if let Some(current_transaction) = &self.current_transaction {
            if current_transaction.id() == transaction.id() {
                self.current_transaction = None;
            }
        }

        Ok(())
    }

    // Не уверен насчёт правильной реализации, требуется доьавление определение типа step
    pub fn rollback_transaction(&mut self, transaction: Transaction) -> Result<()> {
        println!("⚠️  Rolling back transaction: {}", transaction.id());

        transaction.set_status(TransactionStatus::RolledBack);
        transaction.set_completed_at(Some(SystemTime::now()));

        for step in transaction.steps().iter().rev() {
            if step.status() != StepStatus::Completed {
                continue;
            }

            match step.name().as_str() {
                "install_file" => {
                    if let Some(path) = step.details().get("file_path") {
                        let file_path = PathBuf::from(path);
                        if file_path.exists() {
                            std::fs::remove_file(&file_path)?;
                            println!("   ↩️  Removed: {}", file_path.display());
                        }
                    }
                }
                "create_directory" => {
                    if let Some(path) = step.details().get("dir_path") {
                        let dir_path = PathBuf::from(path);
                        if dir_path.exists() {
                            std::fs::remove_dir(&dir_path)?;
                            println!("   ↩️  Removed directory: {}", dir_path.display());
                        }
                    }
                }
                "run_pre_install" => {
                    println!("   ↩️  Running pre_remove script");
                }
                _ => {
                    println!("   ⚠️  Don't know how to rollback step: {}", step.name());
                }
            }
        }

        if let Some(prev_commit) = &transaction.ostree_previous_commit {
            println!("   ↩️  Rolling back OSTree to: {}", prev_commit);
            // ostree_manager.rollback_to_commit(prev_commit)?;
            // TODO: Implement rollback logic for OSTree
        }

        self.move_to_failed(&transaction)?;

        self.current_transaction = None;

        println!("✓ Rollback complete");
        Ok(())
    }

    pub fn add_step(&mut self, step: TransactionStep) -> Result<()> {
        let transaction = self
            .current_transaction
            .as_mut()
            .ok_or(Error::AddStepError(format!("No active transaction")))?;

        self.add_step(step);
        self.save_transaction(transaction)?;

        Ok(())
    }

    pub fn update_step_status(&mut self, step_name: &str, status: StepStatus) -> Result<()> {
        let transaction = self
            .current_transaction
            .as_mut()
            .ok_or(Error::UpdateStepError(format!("No active transaction")))?;

        self.update_step(step_name, status);
        self.save_transaction(transaction)?;

        Ok(())
    }

    pub fn get_active_transactions(&self) -> Result<Vec<Transaction>> {
        let active_dir_path = self.transactions_dir.join("active");
        let mut transactions = Vec::new();

        if !active_dir_path.exists() {
            return Ok(transactions);
        }

        for dir_entry in fs::read_dir(active_dir_path).map_err(Error::IoError)? {
            if dir_entry.path().is_file()
                && dir_entry
                    .path()
                    .extension()
                    .map_or(false, |extention| extention == "toml")
            {
                transactions.push(self.load_transaction(&dir_entry.path())?);
            }
        }

        Ok(transactions)
    }

    pub fn get_transaction(&self, id: &str) -> Result<Option<Transaction>> {
        let transaction_filename = format!("{}.toml", id);

        let active_dir_path = self
            .transactions_dir
            .join("active")
            .join(&transaction_filename);
        if active_dir_path.exists() {
            return Ok(Some(self.load_transaction(&active_dir_path)?));
        }

        let completed_dir_path = self
            .transactions_dir
            .join("completed")
            .join(&transaction_filename);
        if completed_dir_path.exists() {
            return Ok(Some(self.load_transaction(&completed_dir_path)?));
        }

        let failed_dir_path = self
            .transactions_dir
            .join("failed")
            .join(&transaction_filename);
        if failed_dir_path.exists() {
            return Ok(Some(self.load_transaction(&failed_dir_path)?));
        }

        Ok(None)
    }

    fn save_transaction(&self, transaction: &Transaction) -> Result<()> {
        if !self.transactions_dir.exists() {
            return Err(Error::PathError(self.transactions_dir.clone()));
        }

        let directory = match transaction.status() {
            TransactionStatus::InProgress => self.transactions_dir.join("active"),
            TransactionStatus::Completed => self.transactions_dir.join("completed"),
            TransactionStatus::Failed | TransactionStatus::RolledBack => {
                self.transactions_dir.join("failed")
            }
        };

        let file_path = directory.join(format!("{}.toml", transaction.id()));

        utils::write_toml_atomic(&file_path, transaction)?;

        Ok(())
    }

    fn load_transaction(&self, path: &Path) -> Result<Transaction> {
        utils::read_toml(path)
    }

    fn move_to_completed(&self, transaction: &Transaction) -> Result<()> {
        let filename = format!("{}.toml", transaction.id());

        let old_path = self.transactions_dir.join("active").join(&filename);
        let new_path = self.transactions_dir.join("completed").join(&filename);

        if !old_path.exists() {
            return Err(Error::PathError(old_path));
        }

        fs::create_dir(new_path)?;

        fs::rename(&old_path, &new_path)?;

        Ok(())
    }

    fn move_to_failed(&self, transaction: &Transaction) -> Result<()> {
        let filename = format!("{}.toml", transaction.id());

        let old_path = self.transactions_dir.join("active").join(&filename);
        let new_path = self.transactions_dir.join("failed").join(&filename);

        if !old_path.exists() {
            return Err(Error::PathError(old_path));
        }

        fs::create_dir(new_path)?;

        fs::rename(&old_path, &new_path)?;

        Ok(())
    }
}
