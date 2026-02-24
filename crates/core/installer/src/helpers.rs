use crate::Result;

use core::backend::ExtractedPackage;
use core::types::PackageDiff;

use nix::unistd::{chown, Gid, Uid};

use database::database::FileDatabase;
use database::Database;

use package_ostree::implement::OStreeManager;
use package_ostree::OSTreeRepo;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

pub(crate) fn set_permissions(path: &Path, mode: u32, uid: u32, gid: u32) -> Result<()> {
    fs::set_permissions(path, fs::Permissions::from_mode(mode))?;
    chown(path, Some(Uid::from_raw(uid)), Some(Gid::from_raw(gid)))?;
    Ok(())
}

pub(crate) fn stage_files(
    extracted: &ExtractedPackage,
    temp_dir: &Path,
    package_dir: &Path,
    root_dir: &Path,
    database: &mut Box<dyn Database>,
) -> Result<()> {
    for file_entry in &extracted.files {
        let source_path = temp_dir.join(&file_entry.relative_path);
        let package_path = package_dir
            .join(&extracted.name)
            .join(&file_entry.relative_path);

        if let Some(parent) = package_path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::copy(&source_path, &package_path)?;
        set_permissions(
            &package_path,
            file_entry.permissions,
            file_entry.owner,
            file_entry.group,
        )?;

        let destination_path = root_dir.join(&file_entry.relative_path);
        database.register_file(&extracted.name, &destination_path)?;
    }
    Ok(())
}

pub(crate) fn commit_installation(
    ostree: &OStreeManager,
    database: &FileDatabase,
    package_name: &str,
    root_dir: &Path,
) -> Result<String> {
    let diff = PackageDiff {
        added: vec![package_name.to_string()],
        removed: vec![],
        updated: vec![],
    };

    let commit_hash = ostree.create_commit(database, &diff, root_dir)?;

    Ok(commit_hash)
}
