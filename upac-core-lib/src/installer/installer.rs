use super::{InstallEvent, InstallerError, Install, Result};

use crate::{PackageRegistry, PackageRepo, PackageDiff, PackageInfo, ExtractedPackage};
use crate::core::permitions::set_permissions;
use crate::database::FileRegistry;

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;

pub struct Installer {
    registry: Box<dyn PackageRegistry>,
    file_registry: Box<dyn FileRegistry>,
    ostree_enabled: bool,
    root_dir: PathBuf,
    package_dir: PathBuf,
    temp_dir: PathBuf,
    event_tx: Sender<InstallEvent>,
}

impl Installer {
    pub fn new(
        registry: Box<dyn PackageRegistry>,
        file_registry: Box<dyn FileRegistry>,
        ostree_enabled: bool,
        root_dir: PathBuf,
        package_dir: PathBuf,
        temp_dir: PathBuf,
        event_tx: Sender<InstallEvent>,
    ) -> Self {
        Self {
            registry,
            file_registry,
            ostree_enabled,
            root_dir,
            package_dir,
            temp_dir,
            event_tx,
        }
    }

    fn emit(&self, event: InstallEvent) {
        let _ = self.event_tx.send(event);
    }

    fn fail(&self, package: &str, err: InstallerError) -> InstallerError {
        self.emit(InstallEvent::Failed {
            package: package.to_string(),
            reason: err.to_string(),
        });
        err
    }

    // fn commit(&self, diff: &PackageDiff) -> Result<()> {
    //     if let Some(ostree) = &self.ostree {
    //         let packages = self.registry.list_all_packages()?;
    //         let mut files = Vec::new();
    //         for package in &packages {
    //             files.extend(self.file_registry.get_files(&package.name)?);
    //         }
    //         let commit_hash = ostree.commit(files, diff)?;
    //         self.emit(InstallEvent::CommitCreated { commit_hash });
    //     }
    //     Ok(())
    // }
}

impl Install for Installer {
	fn install(&mut self, package: &ExtractedPackage) -> Result<()> {
    	let name = &package.name;

     	self.emit(InstallEvent::InstallStarted {
        	package: name.clone(),
         	total_files: package.files.len(),
     	});

      	let package_info = PackageInfo {
        	name: package.name.clone(),
        	version: package.version.clone(),
        	format: package.format.clone(),
       	};

       	self.registry.add_package(&package_info).map_err(|err| self.fail(name, err.into()))?;

        let total = package.files.len();
        for (index, file_entry) in package.files.iter().enumerate() {
        	let destination = self.root_dir.join(&file_entry.relative_path);

         	if !self.ostree_enabled {
            	let source_path = self.temp_dir.join(&file_entry.relative_path);
             	let package_path = self.package_dir.join(name).join(&file_entry.relative_path);

              	if let Some(parent) = package_path.parent() {
                	fs::create_dir_all(parent).map_err(|e| self.fail(name, e.into()))?;
               	}

                fs::copy(&source_path, &package_path).map_err(|e| self.fail(name, e.into()))?;

                set_permissions(&package_path, file_entry.permissions, file_entry.owner, file_entry.group).map_err(|err| self.fail(name, err.into()))?;
          	}

           self.file_registry.register_file(name, &destination).map_err(|e| self.fail(name, e.into()))?;

           self.emit(InstallEvent::FileInstalled {
           		path: destination,
             	current: index + 1,
             	total,
           });
        }

        self.emit(InstallEvent::InstallFinished { package: name.clone() });

        Ok(())
	}

	fn remove(&mut self, package_id: &str) -> Result<()> {
    	self.emit(InstallEvent::RemoveStarted { package: package_id.to_string() });

     	let files = self.file_registry.get_files(package_id).map_err(|err| self.fail(package_id, err.into()))?;

      	for file_path in &files {
        	if !self.ostree_enabled {
            // Без ostree — удаляем сами
            	if file_path.exists() {
                	fs::remove_file(file_path).map_err(|err| self.fail(package_id, err.into()))?;
             	}
         	}
          	// Если ostree включён — он сам уберёт файлы через rollback
        	// Мы только снимаем регистрацию из базы данных

        	self.file_registry.unregister_file(file_path).map_err(|err| self.fail(package_id, err.into()))?;

        	self.emit(InstallEvent::FileRemoved { path: file_path.clone() });
   		 }

    	self.registry.remove_package(package_id).map_err(|err| self.fail(package_id, err.into()))?;

    	self.emit(InstallEvent::RemoveFinished { package: package_id.to_string() });

    	Ok(())
	}

    fn list_files(&self, package_id: &str) -> Result<Vec<PathBuf>> {
        Ok(self.file_registry.get_files(package_id)?)
    }

    fn list_packages(&self) -> Result<Vec<PackageInfo>> {
        Ok(self.registry.list_all_packages()?)
    }

    fn add_file(&mut self, package_id: &str, file_path: &Path) -> Result<()> {
        self.file_registry.register_file(package_id, file_path)?;
        Ok(())
    }

    fn remove_file(&mut self, file_path: &Path) -> Result<()> {
        if !self.ostree_enabled && file_path.exists() {
            fs::remove_file(file_path)?;
        }
        self.file_registry.unregister_file(file_path)?;
        Ok(())
    }
}
