// ============================================================================
// Imports
// ============================================================================
use crate::database::DatabaseManager;
// ============================================================================
// Conflict detector
// ============================================================================
pub struct ConflictDetector {
    database_manager: DatabaseManager,
}

impl ConflictDetector {
    pub fn new(db_manager: DatabaseManager) -> Self;

    pub fn check_file_conflicts(&self, package: &PackageInfo) -> Result<Vec<Conflict>>;
    pub fn check_package_conflicts(&self, package: &PackageInfo) -> Result<Vec<Conflict>>;
    pub fn can_coexist(&self, pkg1: &PackageInfo, pkg2: &PackageInfo) -> bool;

    fn get_file_list(&self, package: &PackageInfo) -> Result<Vec<PathBuf>>;
    fn find_overlapping_files(&self, files1: &[PathBuf], files2: &[PathBuf]) -> Vec<PathBuf>;
}
// ============================================================================
// Conflict
// ============================================================================
pub struct Conflict {
    pub conflict_type: ConflictType,
    pub package1: String,
    pub package2: String,
    pub details: String,
    pub conflicting_files: Vec<PathBuf>,
}
// ============================================================================
// Conflict type
// ============================================================================
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictType {
    FileConflict,
    PackageConflict,
    DependencyConflict,
    CategoryConflict,
}
