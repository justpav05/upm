pub struct ProgressReporter {
    progress_file: PathBuf,
    pid: u32,
    last_update: Instant,
    update_interval: Duration,
}

impl ProgressReporter {
    pub fn new(pid: u32) -> Self;
    pub fn with_interval(pid: u32, interval: Duration) -> Self;

    // Update progress
    pub fn update(&mut self, percentage: u8, message: &str) -> Result<()>;
    pub fn set_stage(&mut self, stage: ProgressStage) -> Result<()>;
    pub fn set_current_file(&mut self, file: &Path) -> Result<()>;
    pub fn set_bytes(&mut self, processed: u64, total: u64) -> Result<()>;
    pub fn finish(&mut self) -> Result<()>;

    // Static read (no instance needed)
    pub fn read_progress(pid: u32) -> Result<Option<Progress>>;

    // Internal
    fn should_update(&self) -> bool;
    fn write_progress(&self, progress: &Progress) -> Result<()>;
}
