use std::path::PathBuf;

pub struct ExtractedPackage {
    pub name: String,
    pub version: String,
    pub format: String,
    pub files: Vec<FileEntry>,
}

pub struct FileEntry {
    pub relative_path: PathBuf,
    pub permissions: u32,
    #[serde(default)]
    pub owner: u32,
    #[serde(default)]
    pub group: u32,
}

impl Drop for ExtractedPackage {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.temp_dir);
    }
}
