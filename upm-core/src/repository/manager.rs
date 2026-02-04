use crate::backend::BackendRegistry;
use crate::dependency::DependencyResolver;

pub struct RepositoryManager {
    config: RepositoryConfig,
    cache_manager: CacheManager,
    fetcher: MetadataFetcher,
    repositories: Vec<Repository>,
    backend_registry: BackendRegistry,
}

impl RepositoryManager {
    pub fn new(config: RepositoryConfig, backend_registry: BackendRegistry) -> Result<Self>;

    // Repository management
    pub fn add_repository(&mut self, repo: Repository) -> Result<()>;
    pub fn remove_repository(&mut self, name: &str) -> Result<()>;
    pub fn update_all(&mut self) -> Result<()>;
    pub fn update_repository(&mut self, name: &str) -> Result<()>;
    pub fn list_repositories(&self) -> Vec<RepositoryInfo>;

    // Package queries
    pub fn search_package(&self, name: &str) -> Result<Vec<PackageInfo>>;
    pub fn get_package_metadata(&self, name: &str) -> Result<PackageMetadata>;
    pub fn list_available_packages(&self) -> Result<Vec<PackageInfo>>;

    // Internal
    fn fetch_metadata(&self, repo: &Repository) -> Result<RepositoryMetadata>;
    fn update_provides_mapping(&mut self) -> Result<()>;
}

pub struct RepositoryInfo {
    pub name: String,
    pub url: String,
    pub repo_type: RepositoryType,
    pub enabled: bool,
    pub package_count: usize,
    pub last_updated: Option<SystemTime>,
}
