// ============================================================================
// Backend loader
// ============================================================================
pub struct BackendLoader {
    plugin_dir: PathBuf,
}

impl BackendLoader {
    pub fn new(plugin_dir: PathBuf) -> Self;

    // Loading
    pub fn load_backends(&self) -> Result<Vec<Box<dyn Backend>>>;
    pub fn load_backend(&self, path: &Path) -> Result<Box<dyn Backend>>;

    // Discovery
    fn discover_plugins(&self) -> Vec<PathBuf>;
    fn load_dynamic_library(&self, path: &Path) -> Result<Box<dyn Backend>>;
}
