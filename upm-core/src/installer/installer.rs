use crate::backend::Backend;
use crate::progress::ProgressReporter;
use crate::transaction::TransactionManager;
use crate::types::{ExtractedPackage, Package};

pub struct Installer {
    fs_manager: FileSystemManager,
    permissions_manager: PermissionsManager,
    script_runner: ScriptRunner,
}

impl Installer {
    pub fn new() -> Self;

    // Main operations
    pub fn install_package(
        &self,
        package: &Package,
        backend: &dyn Backend,
        progress: &mut ProgressReporter,
        txn_manager: &mut TransactionManager,
    ) -> Result<()>;

    pub fn uninstall_package(
        &self,
        package: &Package,
        progress: &mut ProgressReporter,
        txn_manager: &mut TransactionManager,
    ) -> Result<()>;

    // Internal steps
    fn extract_package(&self, package: &Package, backend: &dyn Backend)
    -> Result<ExtractedPackage>;
    fn run_pre_install(&self, scripts: &Scripts) -> Result<()>;
    fn run_post_install(&self, scripts: &Scripts) -> Result<()>;
    fn install_files(&self, files: &[FileEntry], progress: &mut ProgressReporter) -> Result<()>;
    fn verify_installation(&self, package: &Package) -> Result<()>;
}
