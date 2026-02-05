// ============================================================================
// Imports
// ============================================================================
use nix::sys::stat::Mode;
use nix::unistd::{Gid, Uid, chown};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use tokio::fs;

use crate::types::error::PackageError;
use crate::types::package::operation::{InstallOptions, RemoveOptions};
use crate::types::package::{Package, PackageManager, UncompressedFile};

impl Default for InstallOptions {
    fn default() -> Self {
        Self {
            overwrite: false,
            keep_config: false,
            run_scripts: true,
            backup: true,
        }
    }
}

impl Default for RemoveOptions {
    fn default() -> Self {
        Self {
            purge: false,
            keep_config: true,
            run_scripts: true,
            remove_dependencies: true,
        }
    }
}

impl PackageManager {
    pub async fn install(
        &self,
        package: Package,
        options: InstallOptions,
    ) -> Result<(), Vec<PackageError>> {
        check_root_permissions()?;

        Ok(())
    }

    fn get_package_config_dirs(&self) -> Result<Vec<PathBuf>, PackageError> {
        let config = self.config.as_ref().ok_or_else(|| {
            PackageError::ConfigError("Configuration not initialized".to_string())
        })?;

        let dirs = vec![
            self.config.get_temp_dir(),
            self.config.get_cache_dir(),
            self.config.get_database_dir(),
        ];

        if let Some(missing_dir) = dirs.iter().find(|dir| !dir.exists()) {
            return Err(PackageError::IOError(format!(
                "Directory {} does not exist",
                missing_dir.display()
            )));
        }

        if let Some(not_dir) = dirs.iter().find(|d| !d.is_dir()) {
            return Err(PackageError::ConfigError(format!(
                "Not a directory: {}",
                not_dir.display()
            )));
        }

        Ok(dirs)
    }

    async fn create_directories(&self, source_file: &UncompressedFile) -> Result<(), PackageError> {
        let destination_path = source_file.get_destination()?;

        if destination_path.exists() {
            return Err(PackageError::PackageAlreadyExists(format!(
                "Directory already exists: {}",
                destination_path.display()
            )));
        }

        let parent_dirs = match destination_path.parent() {
            Some(parent_dirs) => parent_dirs,
            None => {
                return Err(PackageError::ConfigError(
                    "Invalid destination path".to_string(),
                ));
            }
        };

        for dir in parent_dirs.iter() {
            if dir.exists() {
                return Err(PackageError::IoError(format!(
                    "Failed to create directory: {}",
                    dir.display()
                )));
            }

            fs::create_dir(dir).await?;

            set_permissions(dir, source_file.get_permissions())?;
            set_owner_and_group(dir, source_file.get_uid(), source_file.get_gid())?;
        }

        Ok(())
    }
}

impl PackageManagerConfig {
    pub fn get_temp_dir(&self) -> &PathBuf {
        &self.temp_dir
    }

    pub fn get_cache_dir(&self) -> &PathBuf {
        &self.cache_dir
    }

    pub fn get_database_dir(&self) -> &PathBuf {
        &self.database_dir
    }
}

fn check_root_permissions() -> Result<(), PackageError> {
    match Uid::effective().is_root() {
        true => Ok(()),
        false => Err(PackageError::PermissionError(
            "This operation requires root privileges".to_string(),
        )),
    }
}

async fn execute_script(script: &InstallScript) -> Result<()> {}

async fn install_file(source_file: &UncompressedFile) -> Result<()> {
    let source_path = source_file.get_source_path()?;
    let destination_path = source_file.get_destination_path()?;

    if !source_path.exists() {
        return Err(PackageError::FileNotFound(format!(
            "Source file not found: {}",
            source_path.display()
        )));
    }

    let copied_bytes = fs::copy(source_path, destination_path).await?;
    if copied_bytes == 0 {
        return Err(PackageError::IoError(format!(
            "File {} copy failed",
            source_path.display()
        )));
    }

    set_permissions(destination_path, source_file.get_permissions())?;
    set_owner_and_group(
        destination_path,
        source_file.get_uid(),
        source_file.get_gid(),
    )?;

    let metadata = fs::metadata(destination_path).await?;
    if metadata.uid() != source_file.get_uid() || metadata.gid() != source_file.get_gid() {
        return Err(PackageError::PermissionError(format!(
            "Failed to set ownership {}",
            destination_path.display()
        )));
    }

    Ok(())
}

async fn set_permissions(path: &Path, mode: Mode) -> Result<(), PackageError> {
    let permissions = std::fs::Permissions::from_mode(mode.bits());
    fs::set_permissions(path, permissions).map_err(|error| {
        PackageError::IoError(format!(
            "Failed to set permissions for {}: expected mode: {:o}, got mode: {:o}",
            path.display(),
            mode,
            error.raw_os_error().unwrap_or(0)
        ))
    })?;

    let metadata = std::fs::metadata(path).map_err(|error| {
        PackageError::IoError(format!(
            "Failed to get metadata for {}: expected mode: {:o}, got mode: {:o}",
            path.display(),
            mode,
            error.raw_os_error().unwrap_or(0)
        ))
    })?;

    if metadata.permissions().mode() != mode {
        return Err(PackageError::IoError(format!(
            "Failed to set permissions for {}: expected mode: {:o}, got mode: {:o}",
            path.display(),
            metadata.permissions().mode(),
            mode.bits()
        )));
    }

    Ok(())
}

async fn set_owner_and_group(path: &Path, uid: Uid, gid: Gid) -> Result<(), PackageError> {
    chown(path, Some(uid), Some(gid)).map_err(|error| {
        PackageError::IoError(format!(
            "Failed to set ownership for {}: expected UID:{}, GID:{}",
            path.display(),
            uid,
            gid,
        ))
    })?;

    let metadata = fs::metadata(path).await?;
    if metadata.gid() != gid || metadata.uid() != uid {
        return Err(PackageError::IoError(format!(
            "Failed to set ownership for {}: expected UID:{}, GID:{}, got UID:{}, GID:{}",
            path.display(),
            uid,
            gid,
            metadata.uid(),
            metadata.gid()
        )));
    }

    Ok(())
}
