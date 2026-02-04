pub struct MetadataFetcher {
    http_client: HttpClient,
    cache_dir: PathBuf,
}

impl MetadataFetcher {
    pub fn new(cache_dir: PathBuf) -> Self;

    // Fetching
    pub fn fetch_metadata(&self, repo: &Repository) -> Result<RepositoryMetadata>;
    pub fn download_package_list(&self, repo: &Repository) -> Result<Vec<PackageInfo>>;
    pub fn download_file(&self, url: &str, dest: &Path) -> Result<()>;
    pub fn download_file_with_progress<F>(&self, url: &str, dest: &Path, progress: F) -> Result<()>
    where
        F: Fn(u64, u64);

    // Parsing
    fn parse_apt_metadata(&self, data: &str) -> Result<RepositoryMetadata>;
    fn parse_rpm_metadata(&self, data: &str) -> Result<RepositoryMetadata>;
    fn parse_arch_metadata(&self, data: &str) -> Result<RepositoryMetadata>;

    // Verification
    fn verify_gpg_signature(&self, data: &[u8], signature: &[u8], key_url: &str) -> Result<bool>;
}

// HTTP client wrapper
struct HttpClient {
    client: reqwest::blocking::Client,
}
