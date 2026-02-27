use ostree::ffi::OstreeRemote;

use super::helpers::{stage_files, commit_installation};
use super::InstallEvent;
use super::InstallerError;
use super::Install;
use super::Result;

use crate::core::backend::ExtractedPackage;
use crate::core::types::PackageDiff;
use crate::core::types::PackageInfo;

use crate::database::database::Database;
	use crate::database::PackageDatabase;

use crate::backup::manager::OStreeRepo;
use crate::backup::PackageRepo;

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;

pub struct Installer {
    pub(crate) database: Box<dyn PackageDatabase>,
    pub(crate) file_database: Database,
    pub(crate) root_dir: PathBuf,
    pub(crate) package_dir: PathBuf,
    pub(crate) temp_dir: PathBuf,
    pub(crate) ostree: OStreeRepo,
    pub(crate) event_tx: Sender<InstallEvent>,
}

impl Installer {
	fn new(
       database: Box<dyn PackageDatabase>,
       file_database: Database,
       root_dir: PathBuf,
       temp_dir: PathBuf,
       package_dir: PathBuf,
       ostree: OStreeRepo,
       event_tx: Sender<InstallEvent>
   ) -> Self {
       Self { database, file_database, root_dir, temp_dir, package_dir, ostree, event_tx }
   }

	fn emit(&self, event: InstallEvent) {
    	let _ = self.event_tx.send(event);
    }

    pub(crate) fn fail(&self, package: &str, err: InstallerError) -> InstallerError {
           self.emit(InstallEvent::Failed {
               package: package.to_string(),
               reason: err.to_string(),
           });
           err
       }
}

impl Install for Installer {
	fn install_package(&mut self, extracted: &ExtractedPackage, ostree_backup: bool) -> Result<()> {
   		let name = &extracted.name;

        self.emit(InstallEvent::InstallStarted {
            package: name.clone(),
            total_files: extracted.files.len(),
        });

        let package_info = PackageInfo {
        	name: extracted.name.clone(),
         	version: extracted.version.clone(),
           	format: extracted.format.clone(),
        };

        self.database.add_package(&package_info).map_err(|e| self.fail(name, e.into()))?;

        stage_files(
        	extracted,
            &self.temp_dir,
            &self.package_dir,
            &self.root_dir,
            &self.event_tx,
            &mut self.database)
        .map_err(|err| self.fail(name, err))?;

        if self.temp_dir.exists() {
        	fs::remove_dir_all(&self.temp_dir).map_err(|err| self.fail(name, err.into()))?;
            fs::create_dir_all(&self.temp_dir).map_err(|err| self.fail(name, err.into()))?;
        }

        if ostree_backup {
       		let commit_hash = commit_installation(&self.ostree, &self.file_database, name, &self.root_dir)
             .map_err(|err| self.fail(name, err))?;

         	self.emit(InstallEvent::CommitCreated { commit_hash: commit_hash.clone() });
        }

        self.emit(InstallEvent::InstallFinished { package: extracted.name.clone() });

        Ok(())
    }

    fn remove_package(&mut self, package_name: &str, ostree_backup: bool) -> Result<()> {
   		self.emit(InstallEvent::RemoveStarted { package: package_name.to_string() });

    	let files = self.database.get_files(package_name).map_err(|err| self.fail(package_name, err.into()))?;

     	for file_path in files {
     		if file_path.exists() {
            	fs::remove_file(&file_path).map_err(|err| self.fail(package_name, err.into()))?;
         	}

         	self.database.unregister_file(&file_path).map_err(|err| self.fail(package_name, err.into()))?;

          	self.emit(InstallEvent::FileRemoved { path: file_path });
      	}

       self.database.remove_package(package_name).map_err(|err| self.fail(package_name, err.into()))?;

        if ostree_backup {
            let diff = PackageDiff {
                added: vec![],
                removed: vec![package_name.to_string()],
                updated: vec![],
            };

            let commit_hash = self.ostree.create_commit(&self.file_database, &diff, &self.root_dir).map_err(|err| self.fail(package_name, err.into()))?;

            self.emit(InstallEvent::CommitCreated { commit_hash });
        }

        Ok(())
    }

    fn install_packages<'a>(&mut self, extracted_packages: impl IntoIterator<Item = &'a ExtractedPackage>, ostree_backup: bool) -> Result<()> {
        for extracted in extracted_packages {
            self.install_package(extracted, ostree_backup)?;
        }
        Ok(())
    }

    fn remove_packages<'a>(&mut self, packages_name: impl IntoIterator<Item = &'a str>, ostree_backup: bool) -> Result<()> {
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
