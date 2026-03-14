use super::{Installer, InstallerState};

use crate::errors::{InstallerError, InstallerResult, InstallerStabbyResult};
use crate::types::ExtractedPackage;
use crate::database::{Database, PackageDatabase};

use stabby::str::Str as StabStr;
use stabby::result::Result as StabResult;

use nix::unistd::{chown, Uid, Gid};

use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::ffi::c_void;
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
        let file_metadata = fs::metadata(source_path)?;
        fs::set_permissions(dest_path, file_metadata.permissions())?;
        chown(
            dest_path,
            Some(Uid::from_raw(file_metadata.uid())),
            Some(Gid::from_raw(file_metadata.gid())),
        ).map_err(|err| InstallerError::Io(err.to_string().into()))?;
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
            return Err(InstallerError::Installer("Root path not found".into()));
        }

        if !temp_dir_path.exists() {
            self.set_state(InstallerState::Failed);
            return Err(InstallerError::Installer("Temp path not found".into()));
        }

        for file_path in package.file_list.iter().map(|string| PathBuf::from(string.as_str())) {
            if !file_path.exists() {
                self.set_state(InstallerState::Failed);
                return Err(InstallerError::Installer(
                    format!("File not found: {}", file_path.display()).into()
                ));
            }
        }

        for file_path in package.file_list.iter().map(|string| PathBuf::from(string.as_str())) {
            self.set_state(InstallerState::Copying);

            let dest_path = root_path.join(&file_path);
            let temp_file_path = temp_dir_path.join(&file_path);

            if temp_file_path.is_dir() {
                fs::create_dir_all(&dest_path)
                    .map_err(|err| InstallerError::Io(err.to_string().into()))?;
                self.copy_with_permissions(&temp_file_path, &dest_path)?;
            } else {
                fs::copy(&temp_file_path, &dest_path)
                    .map_err(|err| InstallerError::Io(err.to_string().into()))?;
                self.copy_with_permissions(&temp_file_path, &dest_path)?;
            }
        }

        self.set_state(InstallerState::Registering);
        self.database.add_package(&package).map_err(InstallerError::from)?;
        self.set_state(InstallerState::Success);

        Ok(())
    }

    fn remove(&mut self, package: &str) -> InstallerResult<()> {
        self.set_state(InstallerState::Preparing);

        let package_files_paths = self.database.get_package_files(package)
            .map_err(InstallerError::from)?;
        let root_path = PathBuf::from(&self.root_path);

        for file_path in package_files_paths {
            self.set_state(InstallerState::Deleting);

            let dest_dir = root_path.join(file_path);

            for entry in fs::read_dir(&dest_dir)
                .map_err(|err| InstallerError::Io(err.to_string().into()))?
            {
                let entry = entry.map_err(|err| InstallerError::Io(err.to_string().into()))?;
                let path = entry.path();

                if path.is_dir() {
                    fs::remove_dir_all(&path)
                        .map_err(|err| InstallerError::Io(err.to_string().into()))?;
                } else {
                    fs::remove_file(&path)
                        .map_err(|err| InstallerError::Io(err.to_string().into()))?;
                }
            }
        }

        self.set_state(InstallerState::Registering);
        self.database.remove_package(package).map_err(InstallerError::from)?;
        self.set_state(InstallerState::Success);

        Ok(())
    }
}

// Публичные extern "C" функции
#[no_mangle]
pub extern "C" fn upac_create(root_path: StabStr, temp_path: StabStr, database_path: StabStr) -> StabResult<*mut c_void, InstallerError> {
    let database = match PackageDatabase::new(PathBuf::from(database_path.as_str())) {
        Ok(database) => database,
        Err(err) => return Err(InstallerError::from(err)).into(),
    };

    let installer = Box::new(PackageInstaller::new(
        root_path.as_str().to_owned(),
        temp_path.as_str().to_owned(),
        Box::new(database),
    ));

    Ok(Box::into_raw(installer) as *mut c_void).into()
}

#[no_mangle]
pub extern "C" fn upac_install(installer: *mut c_void, package: ExtractedPackage) -> InstallerStabbyResult<()> {
    let installer = unsafe { &mut *(installer as *mut PackageInstaller) };
    installer.install(package).into()
}

#[no_mangle]
pub extern "C" fn upac_remove(installer: *mut c_void, package: StabStr) -> InstallerStabbyResult<()> {
    let installer = unsafe { &mut *(installer as *mut PackageInstaller) };
    installer.remove(package.as_str()).into()
}

#[no_mangle]
pub extern "C" fn upac_state(installer: *mut c_void) -> InstallerState {
    let installer = unsafe { &*(installer as *mut PackageInstaller) };
    *installer.state()
}

#[no_mangle]
pub extern "C" fn upac_free(installer: *mut c_void) {
    if !installer.is_null() {
        unsafe { drop(Box::from_raw(installer as *mut PackageInstaller)) };
    }
}
