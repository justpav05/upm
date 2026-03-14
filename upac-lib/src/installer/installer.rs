use super::{Installer, InstallerState};

use crate::database::{Database, PackageDatabase};
use crate::errors::{InstallerError, InstallerResult, InstallerStabbyResult};
use crate::types::ExtractedPackage;

use stabby::result::Result as StabResult;
use stabby::str::Str as StabStr;

use nix::unistd::{chown, Gid, Uid};

use std::ffi::c_void;
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

pub struct PackageInstaller {
    state: InstallerState,
    root_path: String,
    repo_path: String,
    temp_path: String,
    database: Box<dyn Database>,
}

impl PackageInstaller {
    pub fn new(
        root_path: String,
        repo_path: String,
        temp_path: String,
        database: Box<dyn Database>,
    ) -> InstallerResult<Self> {
        let repo_dir = PathBuf::from(&repo_path);

        let repo = ostree::Repo::new(&ostree::gio::File::for_path(&repo_dir));

        if repo_dir.exists() {
            repo.open(ostree::gio::Cancellable::NONE)?;
        } else {
            // Создаём новый
            fs::create_dir_all(&repo_dir)?;
            repo.create(ostree::RepoMode::BareUser, ostree::gio::Cancellable::NONE)?;
        }

        Ok(Self {
            state: InstallerState::Idle,
            root_path,
            repo_path,
            temp_path,
            database,
        })
    }

    pub fn state(&self) -> &InstallerState {
        &self.state
    }

    fn set_state(&mut self, state: InstallerState) {
        self.state = state;
    }

    fn copy_with_permissions(
        &mut self,
        source_path: &Path,
        dest_path: &Path,
    ) -> InstallerResult<()> {
        let file_metadata = fs::metadata(source_path)?;
        fs::set_permissions(dest_path, file_metadata.permissions())?;
        chown(
            dest_path,
            Some(Uid::from_raw(file_metadata.uid())),
            Some(Gid::from_raw(file_metadata.gid())),
        )
        .map_err(|err| InstallerError::Io(err.to_string().into()))?;
        Ok(())
    }
}

impl Installer for PackageInstaller {
    fn install(&mut self, package: ExtractedPackage) -> InstallerResult<()> {
        self.set_state(InstallerState::Preparing);

        let root_path = PathBuf::from(&self.root_path);
        let repo_path = PathBuf::from(&self.repo_path);
        let temp_dir_path = PathBuf::from(&self.temp_path);

        if !root_path.exists() {
            self.set_state(InstallerState::Failed);
            return Err(InstallerError::Installer("Root path not found".into()));
        }

        if !repo_path.exists() {
            self.set_state(InstallerState::Failed);
            return Err(InstallerError::Installer("Repo path not found".into()));
        }

        if !temp_dir_path.exists() {
            self.set_state(InstallerState::Failed);
            return Err(InstallerError::Installer("Temp path not found".into()));
        }

        for file_path in package
            .file_list
            .iter()
            .map(|string| PathBuf::from(string.as_str()))
        {
            if !file_path.exists() {
                self.set_state(InstallerState::Failed);
                return Err(InstallerError::Installer(
                    format!("File not found: {}", file_path.display()).into(),
                ));
            }
        }

        for file_path in package
            .file_list
            .iter()
            .map(|string| PathBuf::from(string.as_str()))
        {
            self.set_state(InstallerState::Copying);

            let temp_file_path = temp_dir_path.join(&file_path);
            let repo_file_path = repo_path.join(&file_path);
            let dest_path = root_path.join(&file_path);

            if temp_file_path.is_dir() {
                fs::create_dir_all(&repo_file_path)?;
                fs::create_dir_all(&dest_path)?;
                self.copy_with_permissions(&temp_file_path, &repo_file_path)?;
            } else {
                if let Some(parent) = repo_file_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                if let Some(parent) = dest_path.parent() {
                    fs::create_dir_all(parent)?;
                }

                fs::copy(&temp_file_path, &repo_file_path)?;
                self.copy_with_permissions(&temp_file_path, &repo_file_path)?;

                fs::hard_link(&repo_file_path, &dest_path)?;
            }
        }

        self.set_state(InstallerState::Registering);
        self.database
            .add_package(&package)
            .map_err(InstallerError::from)?;
        self.set_state(InstallerState::Success);

        Ok(())
    }

    fn remove(&mut self, package: &str) -> InstallerResult<()> {
        self.set_state(InstallerState::Preparing);

        let package_files_paths = self
            .database
            .get_package_files(package)
            .map_err(InstallerError::from)?;

        let root_path = PathBuf::from(&self.root_path);
        let repo_path = PathBuf::from(&self.repo_path);

        for file_path in package_files_paths {
            self.set_state(InstallerState::Deleting);

            let root_file = root_path.join(&file_path);
            let repo_file = repo_path.join(&file_path);

            if root_file.is_dir() {
                fs::remove_dir_all(&root_file)?;
            } else if root_file.exists() {
                fs::remove_file(&root_file)?;
            }

            if repo_file.is_dir() {
                fs::remove_dir_all(&repo_file)?;
            } else if repo_file.exists() {
                fs::remove_file(&repo_file)?;
            }
        }

        self.set_state(InstallerState::Registering);
        self.database
            .remove_package(package)
            .map_err(InstallerError::from)?;
        self.set_state(InstallerState::Success);

        Ok(())
    }
}

// Публичные extern "C" функции
#[no_mangle]
pub extern "C" fn upac_new(
    root_path: StabStr,
    repo_path: StabStr,
    temp_path: StabStr,
    database_path: StabStr,
) -> StabResult<*mut c_void, InstallerError> {
    let database = match PackageDatabase::new(PathBuf::from(database_path.as_str())) {
        Ok(database) => database,
        Err(err) => return Err(InstallerError::from(err)).into(),
    };

    let installer = Box::new(PackageInstaller::new(
        root_path.as_str().to_owned(),
        repo_path.as_str().to_owned(),
        temp_path.as_str().to_owned(),
        Box::new(database),
    ));

    Ok(Box::into_raw(installer) as *mut c_void).into()
}

#[no_mangle]
pub extern "C" fn upac_install(
    installer: *mut c_void,
    package: ExtractedPackage,
) -> InstallerStabbyResult<()> {
    let installer = unsafe { &mut *(installer as *mut PackageInstaller) };
    installer.install(package).into()
}

#[no_mangle]
pub extern "C" fn upac_remove(
    installer: *mut c_void,
    package: StabStr,
) -> InstallerStabbyResult<()> {
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
