use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum InstallEvent {
    InstallStarted  { package: String, total_files: usize },
    FileInstalled   { path: PathBuf, current: usize, total: usize },
    CommitCreated   { commit_hash: String },
    InstallFinished { package: String },

    RemoveStarted   { package: String },
    FileRemoved     { path: PathBuf },
    RemoveFinished  { package: String },

    Failed          { package: String, reason: String },
}
