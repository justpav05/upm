use upac_types::OSTreeOperation;
use upac_types::{OSTreeError, OSTreeResult, OSTreeStabbyResult};

use std::path::Path;

pub mod backup;

pub(crate) trait OSTree {
    fn commit(
        &self,
        repo_path: &Path,
        parent_commit_hash: Option<&str>,
        operation: OSTreeOperation,
        packages: &[&str],
    ) -> OSTreeResult<String>;
    fn rollback(&self, commit_hash: &str) -> OSTreeResult<()>;
    fn remove(&self, package_id: &str) -> OSTreeResult<()>;
    fn list_commits(&self) -> OSTreeResult<Vec<String>>;
}
