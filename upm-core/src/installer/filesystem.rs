pub struct FileSystemManager {
    temp_dir: PathBuf,
}

impl FileSystemManager {
    pub fn new(temp_dir: PathBuf) -> Self;

    // Directory operations
    pub fn create_directory(&self, path: &Path, permissions: u32) -> Result<()>;
    pub fn create_directory_recursive(&self, path: &Path, permissions: u32) -> Result<()>;
    pub fn remove_directory(&self, path: &Path) -> Result<()>;
    pub fn remove_directory_recursive(&self, path: &Path) -> Result<()>;

    // File operations
    pub fn copy_file(&self, src: &Path, dst: &Path) -> Result<()>;
    pub fn copy_file_with_progress<F>(&self, src: &Path, dst: &Path, progress_fn: F) -> Result<()>
    where
        F: Fn(u64, u64);
    pub fn move_file(&self, src: &Path, dst: &Path) -> Result<()>;
    pub fn delete_file(&self, path: &Path) -> Result<()>;

    // Validation
    pub fn validate_path(&self, path: &Path) -> Result<()>;
    pub fn check_disk_space(&self, required: u64) -> Result<bool>;

    // Checksums
    pub fn calculate_checksum(&self, path: &Path) -> Result<String>;
    pub fn verify_checksum(&self, path: &Path, expected: &str) -> Result<bool>;
}
