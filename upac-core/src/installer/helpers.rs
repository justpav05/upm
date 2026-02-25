use super::Result;
use super::events::InstallEvent;

use crate::core::backend::ExtractedPackage;
use crate::core::types::PackageDiff;
use crate::database::database::FileDatabase;
use crate::database::Database;
use crate::backup::implement::OStreeManager;
use crate::backup::OSTreeRepo;

use nix::unistd::{chown, Gid, Uid};

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::sync::mpsc::Sender;

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
    event_tx: &Sender<InstallEvent>,
    database: &mut Box<dyn Database>,
) -> Result<()> {
	let total_count_of_files = extracted.files.len();

    for (file_index, file_entry) in extracted.files.iter().enumerate() {
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

        let _ = event_tx.send(
        InstallEvent::FileInstalled {
        	path: destination_path,
         	current: file_index + 1,
            total: total_count_of_files,
        });
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
