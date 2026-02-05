// ============================================================================
// Imports
// ============================================================================
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::types::Package;
use crate::types::{Error, Result};
use crate::utils;
// ============================================================================
// Package index
// ============================================================================
pub struct PackageIndex {
    index_path: PathBuf,
    packages: HashMap<String, PackageIndexEntry>,
}

impl PackageIndex {
    pub fn load(index_path: PathBuf) -> Result<Self> {
        if index_path.exists() {
            let index: PackageIndex = utils::read_toml(&index_path)?;
            return Ok(index);
        }

        Ok(Self {
            index_path,
            packages: HashMap::new(),
        })
    }

    pub fn save(&self) -> Result<()> {
        if let Some(parent) = self.index_path.parent() {
            fs::create_dir(parent)?;
        }

        utils::write_toml_atomic(&self.index_path, &self)?;

        Ok(())
    }

    pub fn rebuild(database_path: &Path) -> Result<Self> {
        let mut packages = HashMap::new();
        let packages_dir = database_path.join("packages");

        if !packages_dir.exists() {
            return Ok(Self {
                index_path: database_path.join("index.toml"),
                packages,
            });
        }

        for dir in fs::read_dir(&packages_dir)? {
            let path = dir.map_err(|error| Error::IoError(error))?.path();
            if !path.is_dir() {
                continue;
            }

            let metadata_path = path.join("metadata.toml");
            if !metadata_path.exists() {
                return Err(Error::PackageFilesDamaged(format!(
                    "Not find metadata.toml in {}",
                    metadata_path.display()
                )));
            }

            let package: Package = utils::read_toml(&metadata_path)?;

            packages.insert(
                package.id.clone(),
                PackageIndexEntry::new(
                    package.id.clone(),
                    package.name,
                    package.version,
                    package.format,
                    package.installed_at.ok_or_else(|| {
                        Error::PackageFilesDamaged(format!(
                            "Invalid timestamp for package {}",
                            package.id
                        ))
                    })?,
                    package.size,
                ),
            );
        }

        Ok(Self {
            index_path: database_path.join("index.toml"),
            packages,
        })
    }

    pub fn add_entry(&mut self, id: &str, entry: PackageIndexEntry) {
        let id_lower_case = id.to_lowercase();
        self.packages.insert(id_lower_case, entry);
    }

    pub fn remove_entry(&mut self, id: &str) {
        let id_lower_case = id.to_lowercase();
        self.packages.remove(&id_lower_case);
    }

    pub fn get_entry(&self, id: &str) -> Option<&PackageIndexEntry> {
        let id_lower_case = id.to_lowercase();
        self.packages.get(&id_lower_case)
    }

    pub fn search(&self, query: &str) -> Vec<&PackageIndexEntry> {
        let query_lower_case = query.to_lowercase();

        self.packages
            .values()
            .filter(|entry| {
                entry.name.to_lowercase().contains(&query_lower_case)
                    || entry.id.to_lowercase().contains(&query_lower_case)
            })
            .collect()
    }

    pub fn list_all(&self) -> Vec<&PackageIndexEntry> {
        self.packages.values().collect()
    }
}
// ============================================================================
// Package index entry
// ============================================================================
pub struct PackageIndexEntry {
    pub id: String,
    pub name: String,
    pub version: String,
    pub format: String,
    pub installed_at: SystemTime,
    pub size: u64,
}

impl PackageIndexEntry {
    pub fn new(
        id: String,
        name: String,
        version: String,
        format: String,
        installed_at: SystemTime,
        size: u64,
    ) -> Self {
        Self {
            id,
            name,
            version,
            format,
            installed_at,
            size,
        }
    }
}
