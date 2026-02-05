// ============================================================================
// Backend registry
// ============================================================================
pub struct BackendRegistry {
    backends: HashMap<String, Box<dyn Backend>>,
}

impl BackendRegistry {
    pub fn new() -> Self;

    pub fn register_backend(&mut self, backend: Box<dyn Backend>) -> Result<()>;
    pub fn unregister_backend(&mut self, name: &str) -> Result<()>;

    pub fn get_backend(&self, name: &str) -> Option<&Box<dyn Backend>>;
    pub fn get_backend_mut(&mut self, name: &str) -> Option<&mut Box<dyn Backend>>;
    pub fn detect_backend(&self, package_path: &Path) -> Option<&Box<dyn Backend>>;
    pub fn list_backends(&self) -> Vec<String>;
    pub fn get_backend_for_format(&self, format: &str) -> Option<&Box<dyn Backend>>;
}
