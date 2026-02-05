// ============================================================================
// Imports
// ============================================================================
use std::path::PathBuf;

use crate::database::index::PackageIndex;
use crate::types::Error;
use crate::types::{Package, PackageInfo};

pub type Result<T> = std::result::Result<T, Error>;
// ============================================================================
// Database manager
// ============================================================================
pub struct DatabaseManager {
    db_path: PathBuf,
    index: PackageIndex,
}

impl DatabaseManager {
    pub fn new(db_path: PathBuf) -> Result<Self> {}

    pub fn add_package(&mut self, package: &Package) -> Result<()> {}
    pub fn remove_package(&mut self, package_id: &str) -> Result<()> {}
    pub fn update_package(&mut self, package_id: &str, package: &Package) -> Result<()> {}
    pub fn get_package(&self, package_id: &str) -> Result<Option<Package>> {}

    pub fn list_all_packages(&self) -> Result<Vec<PackageInfo>> {}
    pub fn search_packages(&self, query: &str) -> Result<Vec<PackageInfo>> {}
    pub fn get_installed_files(&self, package_id: &str) -> Result<Vec<PathBuf>> {}
    pub fn find_package_by_file(&self, file: &Path) -> Result<Option<String>> {}

    pub fn register_file(&mut self, package_id: &str, file: &Path) -> Result<()> {}
    pub fn unregister_file(&mut self, file: &Path) -> Result<()> {}
    pub fn get_file_owner(&self, file: &Path) -> Result<Option<String>> {}

    fn create_package_entry(&self, package: &Package) -> Result<()> {}
    fn read_package_entry(&self, package_id: &str) -> Result<Package> {}
    fn update_index(&mut self) -> Result<()> {}
    fn rebuild_index(&mut self) -> Result<()> {}
}
