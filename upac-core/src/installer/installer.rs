use super::helpers::{stage_files, commit_installation};
use super::events::InstallEvent;
use super::InstallerError;
use super::Result;

use crate::core::backend::ExtractedPackage;
use crate::core::types::PackageDiff;
use crate::core::types::PackageInfo;

use crate::database::database::FileDatabase;
use crate::database::Database;

use crate::backup::implement::OStreeManager;
use crate::backup::OSTreeRepo;

use std::fs;
use std::path::PathBuf;
use std::sync::mpsc::Sender;

pub struct InstallerManager {
    pub(crate) database: Box<dyn Database>,
    pub(crate) file_database: FileDatabase,
    pub(crate) root_dir: PathBuf,
    pub(crate) package_dir: PathBuf,
    pub(crate) temp_dir: PathBuf,
    pub(crate) ostree: OStreeManager,
    pub(crate) event_tx: Sender<InstallEvent>,
}

impl InstallerManager {
	fn emit(&self, event: InstallEvent) {
    	let _ = self.event_tx.send(event);
    }

    pub(crate) fn fail(&self, package: &str, err: InstallerError) -> InstallerError {
           self.emit(InstallEvent::Failed {
               package: package.to_string(),
               reason: err.to_string(),
           });
           err // ← возвращаем ошибку
       }

    pub(crate) fn install_package(&mut self, extracted: &ExtractedPackage, ostree_backup: bool) -> Result<()> {
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

    pub(crate) fn remove_package(&mut self, package_name: &str, ostree_backup: bool) -> Result<()> {
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
}
