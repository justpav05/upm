use url::Url;

#[derive(Debug, Clone)]
pub struct Package {
    id: String,
    name: String,
    repository: Url,
    info: PackageInfo,
    tags: Option<String>,
    state_of_instalation: bool,
    adition_info: Option<PackageAditionInfo>,
    dependencies: Option<PackageDependencies>,
}

#[derive(Debug, Clone)]
pub struct PackageInfo {
    version: String,
    backend: String,
    download_size: u64,
    installed_size: u64,
    architecture: String,
    maintainer: Option<String>,
    installed_at: Option<chrono::DateTime<chrono::Utc>>,
    updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone)]
pub struct PackageDependencies {
    dependencies: Vec<Package>,
    conflicts: Vec<Package>,
}

#[derive(Debug, Clone)]
pub struct PackageAditionInfo {
    description: Option<String>,
    license: Option<String>,
    homepage: Option<Url>,
}
