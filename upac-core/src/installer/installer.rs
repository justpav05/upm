use crate::helpers::{stage_files, commit_installation};
use crate::events::InstallEvent;
use crate::InstallerError;
use crate::Result;

use core::backend::ExtractedPackage;
use core::types::PackageDiff;
use core::types::PackageInfo;

use database::database::FileDatabase;
use database::Database;

use package_ostree::implement::OStreeManager;
use package_ostree::OSTreeRepo;

use std::fs;
use std::path::PathBuf;
use std::sync::mpsc::Sender;

pub struct InstallerManager {
    pub(crate) database: Box<dyn Database>,
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

    pub(crate) fn as_file_database(&self) -> Result<&FileDatabase> {
        self.database
            .as_any()
            .downcast_ref::<FileDatabase>()
            .ok_or_else(|| InstallerError::OStreeError("Database is not FileDatabase".into()))
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

        if ostree_backup {
        	let database = self.as_file_database().map_err(|err| self.fail(name, err))?;

      		let commit_hash = commit_installation(&self.ostree, database, name, &self.root_dir)
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

            let database = self.as_file_database().map_err(|err| self.fail(package_name, err))?;

            let commit_hash = self.ostree.create_commit(database, &diff, &self.root_dir).map_err(|err| self.fail(package_name, err.into()))?;

            self.emit(InstallEvent::CommitCreated { commit_hash });
        }

        Ok(())
    }
}
