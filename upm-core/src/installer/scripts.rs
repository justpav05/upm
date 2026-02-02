use crate::types::Script;

pub struct ScriptRunner {
    timeout: Duration,
    sandbox_enabled: bool,
}

impl ScriptRunner {
    pub fn new() -> Self;
    pub fn with_timeout(timeout: Duration) -> Self;
    pub fn with_sandbox(sandbox: bool) -> Self;

    pub fn execute_script(&self, script: &Script, context: &ScriptContext) -> Result<ScriptOutput>;
    pub fn validate_script(&self, script: &Script) -> Result<()>;

    fn setup_environment(&self, context: &ScriptContext) -> HashMap<String, String>;
    fn run_with_timeout(
        &self,
        script: &Script,
        env: HashMap<String, String>,
    ) -> Result<ScriptOutput>;
    fn cleanup_after_script(&self) -> Result<()>;
}

pub struct ScriptContext {
    pub package_name: String,
    pub package_version: String,
    pub install_dir: PathBuf,
    pub temp_dir: PathBuf,
}

pub struct ScriptOutput {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub duration: Duration,
}
