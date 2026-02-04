use crate::types::PackageCategory;

pub struct Repository {
    pub name: String,
    pub url: String,
    pub repo_type: RepositoryType,
    pub category: PackageCategory,
    pub enabled: bool,
    pub priority: u32,
    pub gpg_check: bool,
    pub gpg_key_url: Option<String>,
    pub metadata: Option<RepositoryMetadata>,
}

impl Repository {
    pub fn new(name: &str, url: &str, repo_type: RepositoryType) -> Self;
    pub fn with_category(mut self, category: PackageCategory) -> Self;
    pub fn with_priority(mut self, priority: u32) -> Self;
    pub fn is_enabled(&self) -> bool;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RepositoryType {
    APT,
    RPM,
    Arch,
    AUR,
    Flatpak,
    Snap,
    AppImage,
}

pub struct RepositoryMetadata {
    pub packages: Vec<PackageInfo>,
    pub last_updated: SystemTime,
    pub total_size: u64,
}
