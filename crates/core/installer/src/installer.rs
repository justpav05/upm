use crate::Installer;
use crate::Result;
use core::backend::ExtractedPackage;
use core::types::PackageInfo;
use database::Database;
use nix::unistd::{chown, Gid, Uid};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

pub struct InstallerManager {
    database: Box<dyn Database>,
    root_dir: PathBuf,
    temp_dir: PathBuf,
}

impl InstallerManager {
    pub fn new(database: Box<dyn Database>, root_dir: PathBuf, temp_dir: PathBuf) -> Self {
        Self {
            database,
            root_dir,
            temp_dir,
        }
    }

    fn set_permissions(path: &Path, mode: u32, uid: u32, gid: u32) -> Result<()> {
        fs::set_permissions(path, fs::Permissions::from_mode(mode))?;
        chown(path, Some(Uid::from_raw(uid)), Some(Gid::from_raw(gid)))?;
        Ok(())
    }
}

impl Installer for InstallerManager {
    fn install_package(&mut self, extracted: &ExtractedPackage) -> Result<()> {
        let package_info = PackageInfo {
            name: extracted.name.clone(),
            version: extracted.version.clone(),
            format: extracted.format.clone(),
        };
        self.database.add_package(&package_info)?;

        for file_entry in &extracted.files {
            let source_path = self.temp_dir.join(&file_entry.relative_path);
            let destination_path = self.root_dir.join(&file_entry.relative_path);

            if let Some(parent) = destination_path.parent() {
                fs::create_dir_all(parent)?;
            }

            fs::copy(&source_path, &destination_path)?;
            Self::set_permissions(
                &destination_path,
                file_entry.permissions,
                file_entry.owner,
                file_entry.group,
            )?;

            self.database
                .register_file(&extracted.name, &destination_path)?;
        }

        Ok(())
    }

    fn remove_package(&mut self, package_name: &str) -> Result<()> {
        let files = self.database.get_files(package_name)?;

        for file_path in files {
            if file_path.exists() {
                fs::remove_file(&file_path)?;
            }
            self.database.unregister_file(&file_path)?;
        }

        self.database.remove_package(&package_name)?;

        Ok(())
    }

    fn list_package_files(&self, package_id: &str) -> Result<Option<Vec<PathBuf>>> {
        let files = self.database.get_files(package_id)?;

        if !files.is_empty() {
            Ok(Some(files))
        } else {
            Ok(None)
        }
    }

    fn add_file_to_package(&mut self, package_id: &str, file_path: &Path) -> Result<()> {
        self.database.register_file(package_id, file_path)?;
        Ok(())
    }

    fn remove_file_from_package(&mut self, file_path: &Path) -> Result<()> {
        if file_path.exists() {
            fs::remove_file(file_path)?;
        }
        self.database.unregister_file(file_path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::backend::Backend;
    use database::database::FileDatabase;
    use tempfile::tempdir;

    #[test]
    fn install_and_remove() {
        let temp = tempdir().unwrap();
        let db_path = temp.path().join("db");
        let root_path = temp.path().join("root");
        let temp_extract = temp.path().join("extract");

        std::fs::create_dir_all(&temp_extract).unwrap();

        let db = Box::new(FileDatabase::new(db_path).unwrap());
        let mut installer = InstallerManager::new(db, root_path.clone(), temp_extract.clone());

        let backend = core::mock::MockBackend;
        let extracted = backend
            .extract(Path::new("fake.mock"), &temp_extract)
            .unwrap();

        installer.install_package(&extracted).unwrap();
        assert!(root_path.join("usr/bin/test-app").exists());

        installer.remove_package("test-package").unwrap();
        assert!(!root_path.join("usr/bin/test-app").exists());
    }
}
