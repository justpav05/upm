use super::toml::{read_toml, write_toml};
use super::PackageIndex;

use crate::{PackageInfo, DatabaseError};

use serde::{Deserialize, Serialize};

use std::collections::HashMap;
use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, DatabaseError>;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Index {
    index_path: PathBuf,
    packages: HashMap<String, PackageInfo>,
}

impl PackageIndex for Index {
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

    fn insert(&mut self, package: &PackageInfo) {
        self.packages.insert(package.name.clone(), package.clone());
    }

    fn remove(&mut self, package_id: &str) {
        self.packages.remove(package_id);
    }

    fn get(&self, package_id: &str) -> Option<&PackageInfo> {
        self.packages.get(package_id)
    }

    fn list_all(&self) -> Vec<&PackageInfo> {
        self.packages.values().collect()
    }
}
