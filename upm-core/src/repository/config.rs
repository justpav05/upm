pub struct RepositoryConfig {
    pub config_path: PathBuf,
    pub repositories: Vec<Repository>,
    pub native_priority: Vec<RepositoryType>,
    pub universal_priority: Vec<RepositoryType>,
    pub auto_update: bool,
    pub interactive_mode: bool,
    pub cache_dir: PathBuf,
    pub max_cache_size: u64,
}

impl RepositoryConfig {
    pub fn load() -> Result<Self>;
    pub fn load_from(path: &Path) -> Result<Self>;
    pub fn save(&self) -> Result<()>;
    pub fn save_to(&self, path: &Path) -> Result<()>;

    // Repository management
    pub fn add_repository(&mut self, repo: Repository) -> Result<()>;
    pub fn remove_repository(&mut self, name: &str) -> Result<()>;

    // Priority management
    pub fn set_native_priority(&mut self, order: Vec<RepositoryType>) -> Result<()>;
    pub fn set_universal_priority(&mut self, order: Vec<RepositoryType>) -> Result<()>;
    pub fn get_priority(&self, repo_type: RepositoryType, category: PackageCategory) -> u32;
}

impl Default for RepositoryConfig;
