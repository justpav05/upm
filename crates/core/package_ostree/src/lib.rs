use crate::errors::{OStreeError, Result};

use database::database::FileDatabase;

use std::time::SystemTime;

pub mod helpers;
mod errors;
mod implement;

#[derive(Debug, Clone)]
pub struct CommitInfo {
    pub hash: String,
    pub timestamp: SystemTime,
    pub package_list: Vec<String>,
    pub description: String,
}

pub trait OSTreeRepo {
    fn create_repo(&mut self, permissions: u32, uid: u32, gid: u32) -> Result<()>;

    fn delete_repo(&mut self) -> Result<()>;

    fn create_commit(&self, current_database: &FileDatabase) -> Result<String>;

    fn delete_commit(&self, commit_hash: &str) -> Result<()>;

    fn rollback_to(&self, commit_hash: &str) -> Result<()>;

    fn get_commit_info(&self, commit_hash: &str) -> Result<CommitInfo>;
}
