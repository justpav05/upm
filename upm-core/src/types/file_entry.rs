// ============================================================================
// Imports
// ============================================================================
use std::path::PathBuf;
// ============================================================================
// File entry
// ============================================================================
pub struct FileEntry {
    pub source_path: PathBuf,
    pub dest_path: PathBuf,
    pub permissions: u32,
    pub owner: String,
    pub group: String,
    pub checksum: Option<String>,
    pub size: u64,
}

impl FileEntry {
    pub fn new(source: PathBuf, dest: PathBuf) -> Self {}
    pub fn with_permissions(mut self, perms: u32) -> Self {}
    pub fn with_owner(mut self, owner: &str, group: &str) -> Self {}
}
