use crate::installer::InstallerManager;
use crate::events::InstallEvent;
use crate::Installer;
use crate::Result;

use core::backend::ExtractedPackage;

use package_ostree::implement::OStreeManager;

use database::Database;

use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;
use std::fs;

impl Installer for InstallerManager {
	fn new(
        database: Box<dyn Database>,
        root_dir: PathBuf,
        temp_dir: PathBuf,
        package_dir: PathBuf,
        ostree: OStreeManager,
        event_tx: Sender<InstallEvent>
    ) -> Self {
        Self { database, root_dir, temp_dir, package_dir, ostree, event_tx }
    }

    fn install_packages(&mut self, extracted_packages: Vec<&ExtractedPackage>, ostree_backup: bool) -> Result<()> {
        for extracted in extracted_packages {
            self.install_package(extracted, ostree_backup)?;
        }
        Ok(())
    }

    fn remove_packages(&mut self, packages_name: Vec<&str>, ostree_backup: bool) -> Result<()> {
        for package_name in packages_name {
            self.remove_package(package_name, ostree_backup)?;
        }
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
