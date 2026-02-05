// ============================================================================
// Imports
// ============================================================================
use crate::backend::Backend;
// ============================================================================
// Virtual package manager
// ============================================================================
pub struct VirtualPackageManager {
    provides_mapping: HashMap<String, Vec<PackageProvider>>,
    backends: Vec<Box<dyn Backend>>,
}

impl VirtualPackageManager {
    pub fn new(backends: Vec<Box<dyn Backend>>) -> Self;

    // Registration
    pub fn register_provides(&mut self, package: &str, provides: Vec<String>) -> Result<()>;
    pub fn update_mapping_from_repos(&mut self, repos: &[Repository]) -> Result<()>;

    // Queries
    pub fn get_providers(&self, virtual_name: &str) -> Option<Vec<PackageProvider>>;
    pub fn is_virtual(&self, name: &str) -> bool;

    // Internal
    fn extract_provides_from_backend(
        &self,
        backend: &dyn Backend,
        pkg: &PackageMetadata,
    ) -> Vec<String>;
}
// ============================================================================
// Package provider
// ============================================================================
pub struct PackageProvider {
    pub package_name: String,
    pub repo_name: String,
    pub repo_type: RepositoryType,
    pub category: PackageCategory,
    pub version: String,
    pub priority: u32,
    pub provides: Vec<String>,
}
