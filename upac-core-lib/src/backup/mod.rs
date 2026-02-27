use crate::backup::errors::Result;
use crate::core::types::PackageDiff;
use crate::database::database::Database;

use std::time::SystemTime;
use std::path::Path;

pub mod manager;
pub mod errors;
mod helpers;

#[derive(Debug, Clone)]
pub struct CommitInfo {
    pub hash: String,
    pub timestamp: SystemTime,
    pub package_list: Vec<String>,
    pub description: String,
}

pub trait PackageRepo {
    fn create_repo(&mut self, permissions: u32, uid: u32, gid: u32) -> Result<()>;

    fn delete_repo(&mut self) -> Result<()>;

    fn create_commit(&self, current_database: &Database,  diff: &PackageDiff, root_dir: &Path) -> Result<String>;

    fn delete_commit(&self, commit_hash: &str) -> Result<()>;

    fn rollback_to(&self, commit_hash: &str) -> Result<()>;

    fn get_commit_info(&self, commit_hash: &str) -> Result<CommitInfo>;
}
