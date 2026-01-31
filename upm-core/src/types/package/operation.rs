// ============================================================================
// Imports
// ============================================================================
use crate::Package;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct InstallOptions {
    overwrite: bool,
    keep_config: bool,
    run_scripts: bool,
    backup: bool,
}

#[derive(Debug, Clone)]
pub struct RemoveOptions {
    purge: bool,
    keep_config: bool,
    run_scripts: bool,
    remove_dependencies: bool,
}

pub struct OperationManager {
    operations: HashMap<Uuid, Operation>,
    active_operations: Vec<Uuid>,

    pending_queue: Vec<Uuid>,
}

pub struct Operation {
    uuid: Uuid,

    total_packages: Vec<Package>,
    completed_packages: Vec<Package>,
    current_package: Option<Package>,

    progress: OperationStep,

    created_at: DateTime<Utc>,
    started_at: Option<DateTime<Utc>>,
    completed_at: Option<DateTime<Utc>>,

    error: Option<String>,
}

pub enum OperationStep {
    ResolvingDependencies,
    DownloadingPackages,
    VerifyingPackages,
    InstallingPackages,
    UpdatingDatabase,
    CreatingSnapshot,
}
