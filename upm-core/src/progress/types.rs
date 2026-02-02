pub struct Progress {
    pub pid: u32,
    pub percentage: u8,
    pub stage: ProgressStage,
    pub message: String,
    pub current_file: Option<PathBuf>,
    pub bytes_processed: u64,
    pub total_bytes: u64,
    pub updated_at: SystemTime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProgressStage {
    Initializing,
    ResolvingDependencies,
    DownloadingPackages,
    ExtractingPackages,
    RunningPreInstall,
    InstallingFiles,
    RunningPostInstall,
    CreatingOSTreeCommit,
    Finalizing,
    Complete,
}
