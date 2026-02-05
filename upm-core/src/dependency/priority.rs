// ============================================================================
// Imports
// ============================================================================
use crate::repository::RepositoryConfig;
// ============================================================================
// Priority manager
// ============================================================================
pub struct PriorityManager {
    config: RepositoryConfig,
}

impl PriorityManager {
    pub fn new(config: RepositoryConfig) -> Self;

    pub fn get_package_priority(&self, provider: &PackageProvider) -> u32;
    pub fn sort_by_priority(&self, providers: Vec<PackageProvider>) -> Vec<PackageProvider>;
    pub fn select_best_provider(&self, providers: Vec<PackageProvider>) -> Option<PackageProvider>;

    fn calculate_priority(&self, provider: &PackageProvider) -> u32;
    fn compare_providers(&self, a: &PackageProvider, b: &PackageProvider) -> Ordering;
}
