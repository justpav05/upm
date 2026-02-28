// Imports
use serde::{Deserialize, Serialize};

// Types
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
    pub description: Option<String>,
    pub maintainer: Option<String>,
    pub homepage: Option<String>,
    pub license: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub version_req: Option<String>,
}
#[derive(Debug, Clone)]
pub struct PackageDiff {
    pub added: Vec<String>,
    pub removed: Vec<String>,
    pub updated: Vec<String>,
}

// Translate package diff to a human-readable description
impl PackageDiff {
    pub fn to_description(&self) -> String {
        let mut parts = Vec::new();
        if !self.added.is_empty() {
            parts.push(format!("added: {}", self.added.join(", ")));
        }
        if !self.removed.is_empty() {
            parts.push(format!("removed: {}", self.removed.join(", ")));
        }
        if !self.updated.is_empty() {
            parts.push(format!("updated: {}", self.updated.join(", ")));
        }
        parts.join("; ")
    }
}
