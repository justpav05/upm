pub struct CacheManager {
    cache_dir: PathBuf,
    max_size: u64,
}

impl CacheManager {
    pub fn new(cache_dir: PathBuf, max_size: u64) -> Self;

    // Cache operations
    pub fn get_cached_metadata(&self, repo_name: &str) -> Result<Option<RepositoryMetadata>>;
    pub fn cache_metadata(&self, repo_name: &str, metadata: &RepositoryMetadata) -> Result<()>;
    pub fn get_cached_package(&self, package: &str) -> Result<Option<PathBuf>>;
    pub fn cache_package(&self, package: &str, source: &Path) -> Result<()>;

    // Management
    pub fn clean_cache(&self) -> Result<()>;
    pub fn get_cache_size(&self) -> Result<u64>;
    pub fn is_cache_valid(&self, repo_name: &str, max_age: Duration) -> bool;
    pub fn remove_old_entries(&self, max_age: Duration) -> Result<usize>;

    // Internal
    fn ensure_space(&self, required: u64) -> Result<()>;
}
