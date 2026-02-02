use crate::lock::LockManager;
use crate::transaction::TransactionManager;
use crate::progress::ProgressReporter;
use crate::recovery::RecoveryManager;
use crate::operations::ActiveOperationsTracker;
use crate::installer::Installer;
use crate::repository::RepositoryManager;
use crate::dependency::DependencyResolver;
use crate::backend::BackendRegistry;
use crate::database::DatabaseManager;
use crate::ostree::OStreeManager;
use crate::config::Config;

/// Main package manager struct
pub struct PackageManager {
    config: Config,
    lock_manager: LockManager,
    operations_tracker: ActiveOperationsTracker,
    transaction_manager: TransactionManager,
    recovery_manager: RecoveryManager,
    installer: Installer,
    repository_manager: RepositoryManager,
    dependency_resolver: DependencyResolver,
    backend_registry: BackendRegistry,
    database_manager: DatabaseManager,
    ostree_manager: Option<OStreeManager>,
}

impl PackageManager {
    // Construction
    pub fn new() -> Result<Self>;
    pub fn with_config(config: Config) -> Result<Self>;
    
    // Initialization
    fn init(&mut self) -> Result<()>;
    fn load_backends(&mut self) -> Result<()>;
    
    // Package operations (Exclusive lock)
    pub fn install(&mut self, package: &str) -> Result<()>;
    pub fn install_with_options(&mut self, package: &str, options: InstallOptions) -> Result<()>;
    pub fn remove(&mut self, package: &str) -> Result<()>;
    pub fn remove_with_options(&mut self, package: &str, options: RemoveOptions) -> Result<()>;
    pub fn upgrade(&mut self, packages: Vec<String>) -> Result<()>;
    pub fn upgrade_all(&mut self) -> Result<()>;
    
    // Query operations (Shared lock)
    pub fn search(&self, query: &str) -> Result<Vec<PackageInfo>>;
    pub fn list_installed(&self) -> Result<Vec<PackageInfo>>;
    pub fn get_info(&self, package: &str) -> Result<PackageInfo>;
    pub fn list_files(&self, package: &str) -> Result<Vec<PathBuf>>;
    pub fn query_owner(&self, file: &Path) -> Result<Option<String>>;
    
    // Repository operations (Exclusive lock)
    pub fn repo_add(&mut self, repo: Repository) -> Result<()>;
    pub fn repo_remove(&mut self, name: &str) -> Result<()>;
    pub fn repo_update(&mut self) -> Result<()>;
    pub fn repo_update_specific(&mut self, name: &str) -> Result<()>;
    pub fn repo_list(&self) -> Result<Vec<RepositoryInfo>>;
    
    // Rollback operations (Exclusive lock)
    pub fn rollback(&mut self) -> Result<()>;
    pub fn rollback_to(&mut self, commit: &str) -> Result<()>;
    pub fn list_rollback_points(&self) -> Result<Vec<RollbackPoint>>;
    
    // Utility operations (No lock)
    pub fn doctor(&self) -> Result<DoctorReport>;
    pub fn doctor_fix(&mut self) -> Result<DoctorReport>;
    pub fn status(&self) -> Result<Vec<OperationInfo>>;
    pub fn clean_cache(&mut self) -> Result<()>;
    pub fn autoremove(&mut self) -> Result<Vec<String>>;
    
    // Static operations (No lock)
    pub fn get_progress(pid: u32) -> Result<Option<Progress>>;
    pub fn version() -> String;
}

// Options structs
pub struct InstallOptions {
    pub force: bool,
    pub no_deps: bool,
    pub download_only: bool,
    pub ostree: Option<bool>,  // Override global ostree setting
}

impl Default for InstallOptions;

pub struct RemoveOptions {
    pub recursive: bool,           // Remove dependencies
    pub keep_config: bool,         // Don't remove config files
    pub force: bool,
}

impl Default for RemoveOptions;