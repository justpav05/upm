use super::helpers::{ensure_directory, read_toml, write_toml};
use super::{Database, Error, Index, Result};
use super::index::PackageIndex;

use crate::core::lock::{ExclusiveLock, SharedLock};
use crate::core::types::PackageInfo;

use std::any::Any;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub struct FileDatabase {
    database_path: PathBuf,
    index: PackageIndex,
    file_map: HashMap<PathBuf, String>,
}

impl FileDatabase {
    pub fn new(database_path: PathBuf) -> Result<Self> {
        ensure_directory(&database_path)?;
        ensure_directory(&database_path.join("packages"))?;

        let index = PackageIndex::load(database_path.join("index.toml"))?;
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

impl Database for FileDatabase {
    fn add_package(&mut self, package_info: &PackageInfo) -> Result<()> {
        let lock_path = self.database_path.join("database.lock");
        let _lock = ExclusiveLock::acquire(&lock_path).map_err(|_| Error::LockError)?;

        let package_path_dir = self.database_path.join("packages").join(&package_info.name);
        ensure_directory(&package_path_dir)?;

        let metadata_path = package_path_dir.join("metadata.toml");
        write_toml(&metadata_path, package_info)?;

        let files_path = package_path_dir.join("files.list");
        write_toml(&files_path, package_info)?;

        self.index.insert(
            &package_info.name,
            &package_info.version,
            &package_info.format,
        );
        self.index.save()?;

        Ok(())
    }

    fn remove_package(&mut self, package_id: &str) -> Result<()> {
        let files = self.get_files(package_id)?;

        for file in files {
            self.unregister_file(&file)?;
        }

        let lock_path = self.database_path.join("database.lock");
        let _lock = ExclusiveLock::acquire(&lock_path).map_err(|_| Error::LockError)?;

        self.index.remove(package_id);
        self.index.save()?;

        let package_dir = self.database_path.join("packages").join(package_id);
        if package_dir.exists() {
            fs::remove_dir_all(&package_dir)?;
        }

        Ok(())
    }

    fn get_package(&self, package_id: &str) -> Result<Option<PackageInfo>> {
        let lock_path = self.database_path.join("database.lock");
        let _lock = SharedLock::acquire(&lock_path).map_err(|_| Error::LockError)?;

        let metadata_path = self
            .database_path
            .join("packages")
            .join(package_id)
            .join("metadata.toml");

        if !metadata_path.exists() {
            return Ok(None);
        }

        let package = read_toml(&metadata_path)?;

        Ok(Some(package))
    }

    fn list_all_packages(&self) -> Result<Vec<PackageInfo>> {
        let lock_path = self.database_path.join("database.lock");
        let _lock = SharedLock::acquire(&lock_path).map_err(|_| Error::LockError)?;

        let packages = self.index.list_all().into_iter().cloned().collect();
        Ok(packages)
    }

    fn register_file(&mut self, package_id: &str, file_path: &Path) -> Result<()> {
        let lock_path = self.database_path.join("database.lock");
        let _lock = SharedLock::acquire(&lock_path).map_err(|_| Error::LockError)?;

        self.file_map
            .insert(file_path.to_path_buf(), package_id.to_string());
        write_toml(&self.database_path.join("file_map.toml"), &self.file_map)?;

        let files_list = self
            .database_path
            .join("packages")
            .join(package_id)
            .join("files.list");
        let mut content = fs::read_to_string(&files_list).unwrap_or_default();

        content.push_str(&format!("{}\n", file_path.display()));
        fs::write(&files_list, content)?;

        Ok(())
    }

    fn unregister_file(&mut self, file_path: &Path) -> Result<()> {
        let lock_path = self.database_path.join("database.lock");
        let _lock = ExclusiveLock::acquire(&lock_path).map_err(|_| Error::LockError)?;

        if let Some(package_name) = self.file_map.remove(file_path).clone() {
            write_toml(&self.database_path.join("file_map.toml"), &self.file_map)?;

            let files_list = self
                .database_path
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
        }

        Ok(())
    }

    fn get_files(&self, package_id: &str) -> Result<Vec<PathBuf>> {
        let lock_path = self.database_path.join("database.lock");
        let _lock = SharedLock::acquire(&lock_path).map_err(|_| Error::LockError)?;

        let package_dir = self.database_path.join("packages").join(package_id);
        if !package_dir.exists() {
            return Ok(Vec::new());
        }

        let files_list_path = package_dir.join("files.list");
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

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::PackageInfo;
    use tempfile::tempdir;

    fn make_package() -> PackageInfo {
        PackageInfo {
            name: "firefox".to_string(),
            version: "120.0".to_string(),
            format: "deb".to_string(),
        }
    }

    #[test]
    fn add_and_get_package_from_database() {
        let dir = tempdir().unwrap();
        let mut db = FileDatabase::new(dir.path().to_path_buf()).unwrap();
        let package = make_package();

        db.add_package(&package).unwrap();

        let result = db.get_package("firefox").unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn remove_package_from_databse() {
        let dir = tempdir().unwrap();
        let mut db = FileDatabase::new(dir.path().to_path_buf()).unwrap();
        let package = make_package();

        db.add_package(&package).unwrap();
        db.remove_package("firefox").unwrap();

        let result = db.get_package("firefox").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn list_all_packages_from_database() {
        let dir = tempdir().unwrap();
        let mut db = FileDatabase::new(dir.path().to_path_buf()).unwrap();

        db.add_package(&make_package()).unwrap();

        let list = db.list_all_packages().unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "firefox");
    }

    #[test]
    fn register_and_get_files_from_package() {
        let dir = tempdir().unwrap();
        let mut db = FileDatabase::new(dir.path().to_path_buf()).unwrap();
        let package = make_package();

        db.add_package(&package).unwrap();
        db.register_file("firefox", Path::new("/usr/bin/firefox"))
            .unwrap();

        let files = db.get_files("firefox").unwrap();
        assert!(files.contains(&PathBuf::from("/usr/bin/firefox")));
    }

    #[test]
    fn unregister_file_from_package() {
        let dir = tempdir().unwrap();
        let mut db = FileDatabase::new(dir.path().to_path_buf()).unwrap();
        let package = make_package();

        db.add_package(&package).unwrap();
        db.register_file("firefox", Path::new("/usr/bin/firefox"))
            .unwrap();
        db.unregister_file(Path::new("/usr/bin/firefox")).unwrap();

        let files = db.get_files("firefox").unwrap();
        assert!(!files.contains(&PathBuf::from("/usr/bin/firefox")));
    }

    #[test]
    fn persist_after_reload() {
        let dir = tempdir().unwrap();

        {
            let mut db = FileDatabase::new(dir.path().to_path_buf()).unwrap();
            db.add_package(&make_package()).unwrap();
        }

        let db = FileDatabase::new(dir.path().to_path_buf()).unwrap();
        let result = db.get_package("firefox").unwrap();
        assert!(result.is_some());
    }
}
