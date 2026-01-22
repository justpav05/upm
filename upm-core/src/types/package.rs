use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub backends: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub size_bytes: u64,
    pub category: Vec<String>,
    pub description: Option<String>,
    pub license: Option<String>,
    pub homepage: Option<String>,
}
