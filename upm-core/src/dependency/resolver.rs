// ============================================================================
// Imports
// ============================================================================
use crate::database::DatabaseManager;
use crate::repository::RepositoryManager;
// ============================================================================
// Dependency resolver
// ============================================================================
pub struct DependencyResolver {
    virtual_pkg_manager: VirtualPackageManager,
    priority_manager: PriorityManager,
    conflict_detector: ConflictDetector,
    repository_manager: RepositoryManager,
    database_manager: DatabaseManager,
}

impl DependencyResolver {
    pub fn new(
        repo_manager: RepositoryManager,
        db_manager: DatabaseManager,
        priority_manager: PriorityManager,
    ) -> Self;

    pub fn resolve_dependencies(&self, package: &str) -> Result<DependencyGraph>;
    pub fn find_installation_plan(&self, package: &str) -> Result<InstallationPlan>;
    pub fn check_conflicts(&self, packages: &[PackageInfo]) -> Result<Vec<Conflict>>;

    fn build_dependency_tree(
        &self,
        package: &str,
        visited: &mut HashSet<String>,
    ) -> Result<DependencyNode>;
    fn select_best_provider(&self, virtual_name: &str) -> Result<PackageProvider>;
    fn resolve_version_conflict(&self, package: &str, versions: Vec<&str>) -> Result<&str>;
}
// ============================================================================
// Installation plan
// ============================================================================
pub struct InstallationPlan {
    pub packages_to_install: Vec<PackageInfo>,
    pub packages_to_remove: Vec<String>,
    pub packages_to_upgrade: Vec<PackageInfo>,
    pub total_download_size: u64,
    pub total_install_size: u64,
    pub conflicts: Vec<Conflict>,
}

impl InstallationPlan {
    pub fn is_valid(&self) -> bool;
    pub fn display_plan(&self) -> String;
}
