use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub info: PackageInfo,
    pub metadata: PackageMetadata,
    pub dependencies: Vec<Dependency>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    pub format: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageMetadata {
    pub description: String,
    pub maintainer: String,
    pub homepage: Option<String>,
    pub license: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub version_req: Option<String>,
}
