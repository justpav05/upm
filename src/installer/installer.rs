use super::{InstallerState, Installer};

use crate::errors::{InstallerError, InstallerResult};
use crate::types::ExtractedPackage;
use crate::database::Database;

use nix::unistd::{chown, Uid, Gid};

use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::fs;

pub struct PackageInstaller {
	state: InstallerState,
	root_path: String,
	temp_path: String,
	database: Box<dyn Database>,
}

impl PackageInstaller {
	pub fn new(root_path: String, temp_path: String, database: Box<dyn Database>) -> Self {
		Self { state: InstallerState::Idle, root_path, temp_path, database }
	}

	pub fn state(&self) -> &InstallerState {
		&self.state
	}

	fn set_state(&mut self, state: InstallerState) {
		self.state = state;
	}

	fn copy_with_permissions(&mut self, source_path: &Path, dest_path: &Path) -> InstallerResult<()> {
		let file_metadata = fs::metadata(&source_path)?;

		fs::set_permissions(&dest_path, file_metadata.permissions())?;

		chown(
			&dest_path.to_path_buf(),
			Some(Uid::from_raw(file_metadata.uid())),
			Some(Gid::from_raw(file_metadata.gid())))?;

		Ok(())
	}
}

impl Installer for PackageInstaller {
	fn install(&mut self, package: ExtractedPackage) -> InstallerResult<()> {
		self.set_state(InstallerState::Preparing);

		let root_path = PathBuf::from(&self.root_path);
		let temp_dir_path = PathBuf::from(&self.temp_path);

    	if !root_path.exists() {
     		self.set_state(InstallerState::Failed);

    		return Err(InstallerError::PackageNotFound(self.root_path.clone()));
     	}

     	if !temp_dir_path.exists() {
      		self.set_state(InstallerState::Failed);

     		return Err(InstallerError::PackageNotFound(self.temp_path.clone()));
      	}

      	for file_path in &package.file_list {
     		if !file_path.exists() {
       			self.set_state(InstallerState::Failed);

        		return Err(InstallerError::PackageNotFound(file_path.to_string_lossy().into_owned()));
      		}
       	}

      	for file_path in &package.file_list {
       		self.set_state(InstallerState::Copying);

          	let dest_path = root_path.join(file_path);
           	let temp_file_path = temp_dir_path.join(file_path);

			if temp_file_path.is_dir() {
				fs::create_dir_all(&dest_path)?;
				self.copy_with_permissions(&temp_file_path, &dest_path)?;
        	} else {
            	fs::copy(&temp_file_path, &dest_path)?;
            	self.copy_with_permissions(&temp_file_path, &dest_path)?;
         	}
       }

       self.set_state(InstallerState::Registering);

       self.database.add_package(&package)?;

       self.set_state(InstallerState::Success);

       Ok(())
	}

	fn remove(&mut self, package: &str) -> InstallerResult<()> {
		self.set_state(InstallerState::Preparing);

		let package_files_paths = self.database.get_package_files(package)?;
		let root_path = PathBuf::from(&self.root_path);

     	for file_path in package_files_paths {
      		self.set_state(InstallerState::Deleting);

      		let dest_dir = root_path.join(file_path);

       		for entry in fs::read_dir(&dest_dir)? {
           		let entry = entry?;
            	let path = entry.path();

            	if path.is_dir() {
               		fs::remove_dir_all(&path)?;
            	} else {
               		fs::remove_file(&path)?;
            	}
        	}
      	}

      	self.set_state(InstallerState::Registering);

     	self.database.remove_package(package)?;

      	self.set_state(InstallerState::Success);

    	Ok(())
	}
}
