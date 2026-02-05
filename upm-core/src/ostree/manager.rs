// ============================================================================
// Imports
// ============================================================================
use std::path::PathBuf;
use std::time::SystemTime;

use crate::types::{Error, Result};
// ============================================================================
// OStree manager
// ============================================================================
pub struct OStreeManager {
    repo_path: PathBuf,
    enabled: bool,
}

impl OStreeManager {
    pub fn new(config: OStreeConfig) -> Result<Self>;

    // Commit operations
    pub fn get_current_commit(&self) -> Result<String>;
    pub fn checkout_commit(&self, commit: &str) -> Result<PathBuf>;
    pub fn create_commit(&self, path: &Path, message: &str) -> Result<String>;
    pub fn deploy_commit(&self, commit: &str) -> Result<()>;

    // Rollback
    pub fn rollback_to_commit(&self, commit: &str) -> Result<()>;
    pub fn rollback_to_previous(&self) -> Result<()>;

    // Queries
    pub fn list_commits(&self) -> Result<Vec<CommitInfo>>;
    pub fn get_commit_info(&self, commit: &str) -> Result<CommitInfo>;
    pub fn list_deployments(&self) -> Result<Vec<DeploymentInfo>>;

    // Cleanup
    pub fn cleanup_old_deployments(&self, keep_count: u32) -> Result<()>;
    pub fn prune_repo(&self) -> Result<()>;

    // Internal
    fn verify_enabled(&self) -> Result<()>;
}
// ============================================================================
// OStree config
// ============================================================================
pub struct OStreeConfig {
    pub repo_path: PathBuf,
    pub enabled: bool,
    pub max_deployments: u32,
}
// ============================================================================
// Commit info
// ============================================================================
pub struct CommitInfo {
    pub hash: String,
    pub timestamp: SystemTime,
    pub message: String,
    pub parent: Option<String>,
}
// ============================================================================
// Deployment info
// ============================================================================
pub struct DeploymentInfo {
    pub commit: String,
    pub timestamp: SystemTime,
    pub is_current: bool,
}
