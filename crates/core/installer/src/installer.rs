use crate::installer::Installer;
use core::backend::ExtractedPackage;
use database::Database;
use nix::unistd::{Gid, Uid, chown};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub type Result<T> = std::result::Result<T, InstallerError>;

#[derive(Debug)]
pub enum InstallerError {
    IoError(std::io::Error),
    PathNotFoundError(std::io::Error),
    DatabaseError(database::Error),
}

impl From<std::io::Error> for InstallerError {
    fn from(e: std::io::Error) -> Self {
        InstallerError::Io(e)
    }
}

impl From<database::Error> for InstallerError {
    fn from(e: database::Error) -> Self {
        InstallerError::Database(e)
    }
}

pub struct InstallerManager {
    database: Arc<Mutex<Box<dyn Database>>>,
    root_dir: PathBuf,
    temp_dir: PathBuf,
}

impl Installer {
    pub fn new(
        database: Arc<Mutex<Box<dyn Database>>>,
        root_dir: PathBuf,
        temp_dir: PathBuf,
    ) -> Self {
        Self {
            database,
            root_dir,
            temp_dir,
        }
    }

    fn create_dir(
        path: &Path,
        mode: u32,
        uid: u32,
        gid: u32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                return Err(InstallerError::PathNotFoundError("Path not found".into()));
            }
        }

        fs::create_dir(path)?;
        fs::set_permissions(path, fs::Permissions::from_mode(mode))?;
        chown(path, Some(Uid::from_raw(uid)), Some(Gid::from_raw(gid)))?;

        Ok(())
    }
}

impl Installer for InstallerManager {
    fn install_package(&self, extracted: &ExtractedPackage) -> Result<()> {
        let mut database = self.database.lock()?;

        let package_info = database::PackageInfo {
            name: extracted.name.clone(),
            version: extracted.version.clone(),
            format: extracted.format.clone(),
        };
        database.add_package(&package_info)?;

        for file_entry in &extracted.files {
            let source_path = self.temp_dir.join(&file_entry.relative_path);
            let destination_path = self.root_dir.join(&file_entry.relative_path);

            path.ancestors().skip(1).rev().for_each(|path| {
                create_dir(
                    path,
                    extracted.files.permissions,
                    extracted.files.owner,
                    extracted.files.group,
                )?
            });

            fs::copy(&source, &dest)?;
            fs::set_permissions(
                dest,
                fs::Permissions::from_mode(extracted.files.permissions),
            )?;
            chown(
                dest,
                Some(Uid::from_raw(extracted.files.owner)),
                Some(Gid::from_raw(extracted.files.group)),
            )?;

            database.register_file(&extracted.files.relative_path, &dest)?;
        }

        Ok(())
    }
}
