// ============================================================================
// Imports
// ============================================================================
use crate::repository::RepositoryConfig;
use crate::ostree::OStreeConfig;
// ============================================================================
// Config
// ============================================================================
pub struct Config {
    pub config_path: PathBuf,
    pub data_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub lock_file: PathBuf,
    pub log_level: LogLevel,
    pub repository: RepositoryConfig,
    pub ostree: OStreeConfig,
}

impl Config {
    pub fn load() -> Result<Self>;
    pub fn load_from(path: &Path) -> Result<Self>;
    pub fn save(&self) -> Result<()>;

    // Default paths
    pub fn default_data_dir() -> PathBuf;
    pub fn default_cache_dir() -> PathBuf;
    pub fn default_config_path() -> PathBuf;
}

impl Default for Config;
// ============================================================================
// Log level
// ============================================================================
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}
