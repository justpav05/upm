use super::helpers::{read_toml, write_toml};
use super::{DatabaseError, Search, Result};

use crate::core::types::PackageInfo;

use serde::{Deserialize, Serialize};

use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Index {
    index_path: PathBuf,
    packages: HashMap<String, PackageInfo>,
}

impl Index {
    fn rebuild(packages_dir: &Path) -> Result<Self> {
        let mut packages = HashMap::default();
        let index_path = packages_dir
            .parent()
            .unwrap_or(packages_dir)
            .join("index.toml");

        if packages_dir.exists() {
            for entry in std::fs::read_dir(packages_dir)? {
                let meta_path = entry?.path().join("metadata.toml");
                if let Ok(package) = read_toml::<PackageInfo>(&meta_path) {
                    packages.insert(package.name.clone(), package);
                }
            }
        } else {
            return Err(DatabaseError::NotFound);
        }

        Ok(Self {
            index_path,
            packages,
        })
    }
}

impl Search for Index {
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

    fn insert(&mut self, name: &str, version: &str, format: &str) {
        let package_info = PackageInfo {
            name: String::from(name),
            version: String::from(version),
            format: String::from(format),
        };

        self.packages
            .insert(package_info.name.clone(), package_info);
    }

    fn remove(&mut self, package_info: &str) {
        self.packages.remove(package_info);
    }

    fn get(&self, package_info: &str) -> Option<&PackageInfo> {
        self.packages.get(package_info)
    }

    fn search(&self, query: &str) -> Vec<&PackageInfo> {
        let lowercase_query = query.to_lowercase();
        self.packages
            .values()
            .filter(|package| package.name.to_lowercase().contains(&lowercase_query))
            .collect()
    }

    fn list_all(&self) -> Vec<&PackageInfo> {
        self.packages.values().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn insert_and_get_index() {
        let dir = tempdir().unwrap();
        let mut index = Index::load(dir.path().join("index.toml")).unwrap();

        index.insert("firefox", "120.0", "deb");

        let entry = index.get("firefox").unwrap();
        assert_eq!(entry.name, "firefox");
        assert_eq!(entry.version, "120.0");
    }

    #[test]
    fn remove_from_index() {
        let dir = tempdir().unwrap();
        let mut index = Index::load(dir.path().join("index.toml")).unwrap();

        index.insert("firefox", "120.0", "deb");
        index.remove("firefox");

        assert!(index.get("firefox").is_none());
    }

    #[test]
    fn save_and_reload_index() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("index.toml");

        let mut index = Index::load(path.clone()).unwrap();
        index.insert("firefox", "120.0", "deb");
        index.save().unwrap();

        let reloaded = Index::load(path).unwrap();
        assert!(reloaded.get("firefox").is_some());
    }

    #[test]
    fn search_in_index() {
        let dir = tempdir().unwrap();
        let mut index = Index::load(dir.path().join("index.toml")).unwrap();

        index.insert("firefox", "120.0", "deb");
        index.insert("vlc", "3.0", "deb");

        let results = index.search("fire");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "firefox");
    }
}
