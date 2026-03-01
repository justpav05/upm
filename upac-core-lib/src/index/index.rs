// Imports
use super::toml::{read_toml, write_toml};
use super::PackageIndex;

use crate::{PackageInfo, DatabaseError};

use serde::{Deserialize, Serialize};

use std::collections::HashMap;
use std::path::PathBuf;

// Type alias for Result<T, DatabaseError>
pub type Result<T> = std::result::Result<T, DatabaseError>;

// Index struct for package indexing
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Index {
    index_path: PathBuf,
    packages: HashMap<String, PackageInfo>,
}

// Implement PackageIndex trait for Index
impl PackageIndex for Index {

	// Implement load method for Index
    fn load(index_path: PathBuf) -> Result<Self> {
        let packages = if index_path.exists() {
            read_toml(&index_path)?
        } else {
            HashMap::default()
        };

        Ok(Self {
            index_path,
            packages,
        })
    }

    // Implement save method for Index
    fn save(&self) -> Result<()> {
        write_toml(&self.index_path, &self.packages)
    }

    fn reload(&mut self) -> Result<()> {
        self.packages = if self.index_path.exists() {
            read_toml(&self.index_path)?
        } else {
            HashMap::default()
        };

        Ok(())
    }

    // Implement insert method for Index
    fn insert(&mut self, package: &PackageInfo) {
        self.packages.insert(package.name.clone(), package.clone());
    }

    // Implement remove method for Index
    fn remove(&mut self, package_id: &str) {
        self.packages.remove(package_id);
    }

    // Implement get method for Index
    fn get(&self, package_id: &str) -> Option<&PackageInfo> {
        self.packages.get(package_id)
    }

    // Implement list_all method for Index
    fn list_all(&self) -> Vec<&PackageInfo> {
        self.packages.values().collect()
    }
}
