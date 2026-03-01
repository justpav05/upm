// Imports
use super::{DatabaseError, PackageRegistry, PackageDatabase, FileRegistry, Result};
use super::files::{ensure_directory, read_toml, write_toml};

use crate::core::lock::{ExclusiveLock, SharedLock};
use crate::core::types::PackageInfo;
use crate::core::Lockable;

use crate::index::index::Index;
use crate::index::PackageIndex;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;

// Struct definition for database
#[derive(Debug, Clone)]
pub struct Database {
    database_path: PathBuf,
    index: Index,
    file_map: HashMap<PathBuf, String>,
}

// Implementation of Database struct own functions
impl Database {

	// Function to create a new database instance
    pub fn new(database_path: PathBuf) -> Result<Self> {
        ensure_directory(&database_path)?;
        ensure_directory(&database_path.join("packages"))?;

        let index = Index::load(database_path.join("index.toml"))?;
        let file_map = {
            let file_map_path = database_path.join("file_map.toml");
            if file_map_path.exists() {
                read_toml(&file_map_path)?
            } else {
                HashMap::default()
            }
        };

        Ok(Self {
            database_path,
            index,
            file_map,
        })
    }
}

// Implementation of FileRegistry trait for Database struct
impl FileRegistry for Database {

	// Function to register a file in the database
    fn register_file(&mut self, package_id: &str, file_path: &Path) -> Result<()> {
        let lock_path = self.database_path.join("database.lock");
        let _lock = ExclusiveLock::acquire(&lock_path).map_err(|_| DatabaseError::LockError)?;

        self.file_map.insert(file_path.to_path_buf(), package_id.to_string());
        write_toml(&self.database_path.join("file_map.toml"), &self.file_map)?;

        let files_list = self.database_path
            .join("packages")
            .join(package_id)
            .join("files.list");

        let mut content = fs::read_to_string(&files_list).unwrap_or_default();
        content.push_str(&format!("{}\n", file_path.display()));
        fs::write(&files_list, content)?;

        Ok(())
    }

    // Function to unregister a file from the database
    fn unregister_file(&mut self, file_path: &Path) -> Result<()> {
        let lock_path = self.database_path.join("database.lock");
        let _lock = ExclusiveLock::acquire(&lock_path).map_err(|_| DatabaseError::LockError)?;

        let Some(package_name) = self.file_map.remove(file_path) else {
            return Ok(());
        };

        write_toml(&self.database_path.join("file_map.toml"), &self.file_map)?;

        let files_list = self.database_path
            .join("packages")
            .join(&package_name)
            .join("files.list");

        if files_list.exists() {
            let content = fs::read_to_string(&files_list)?;
            let new_content: String = content
                .lines()
                .filter(|line| Path::new(line.trim()) != file_path)
                .map(|line| format!("{}\n", line))
                .collect();
            fs::write(&files_list, new_content)?;
        }

        Ok(())
    }

    // Function to get the list of files for a given package
    fn get_files(&self, package_id: &str) -> Result<Vec<PathBuf>> {
        let lock_path = self.database_path.join("database.lock");
        let _lock = SharedLock::acquire(&lock_path).map_err(|_| DatabaseError::LockError)?;

        let files_list_path = self.database_path
            .join("packages")
            .join(package_id)
            .join("files.list");

        if !files_list_path.exists() {
            return Ok(Vec::new());
        }

        let files_list = fs::read_to_string(&files_list_path)?;
        Ok(files_list
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| PathBuf::from(line.trim()))
            .collect())
    }
}

// Implementation of PackageRegistry trait for Database struct
impl PackageRegistry for Database {

	// Function to add a package to the database
    fn add_package(&mut self, package: &PackageInfo) -> Result<()> {
        let lock_path = self.database_path.join("database.lock");
        let _lock = ExclusiveLock::acquire(&lock_path).map_err(|_| DatabaseError::LockError)?;

        let package_dir = self.database_path.join("packages").join(&package.name);
        ensure_directory(&package_dir)?;

        let metadata_path = package_dir.join("metadata.toml");
        write_toml(&metadata_path, package)?;

        let package_info = PackageInfo {
            name: package.name.clone(),
            version: package.version.clone(),
            format: package.format.clone(),
        };

        self.index.insert(&package_info);
        self.index.save()?;

        Ok(())
    }

    // Function to remove a package from the database
    fn remove_package(&mut self, package_id: &str) -> Result<()> {
        let lock_path = self.database_path.join("database.lock");
        let _lock = ExclusiveLock::acquire(&lock_path).map_err(|_| DatabaseError::LockError)?;

        self.index.remove(package_id);
        self.index.save()?;

        let package_dir = self.database_path.join("packages").join(package_id);
        if package_dir.exists() {
            fs::remove_dir_all(&package_dir)?;
        }

        Ok(())
    }

    // Function to get a package from the database
    fn get_package(&self, package_id: &str) -> Result<Option<PackageInfo>> {
        let lock_path = self.database_path.join("database.lock");
        let _lock = SharedLock::acquire(&lock_path).map_err(|_| DatabaseError::LockError)?;

        let metadata_path = self.database_path
            .join("packages")
            .join(package_id)
            .join("metadata.toml");

        if !metadata_path.exists() {
            return Ok(None);
        }

        Ok(Some(read_toml(&metadata_path)?))
    }

    // Function to list all packages in the database
    fn list_all_packages(&self) -> Result<Vec<PackageInfo>> {
        let lock_path = self.database_path.join("database.lock");
        let _lock = SharedLock::acquire(&lock_path).map_err(|_| DatabaseError::LockError)?;

        Ok(self.index.list_all().into_iter().cloned().collect())
    }
}

impl PackageDatabase for Database {}
