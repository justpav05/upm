use super::InstallEvent;
use super::Result;

use crate::core::backend::ExtractedPackage;
use crate::core::helpers::set_permissions;
use crate::core::types::PackageDiff;

use crate::database::database::Database;
use crate::database::PackageDatabase;

use crate::backup::manager::OStreeRepo;
use crate::backup::PackageRepo;

use std::fs;
use std::path::Path;
use std::sync::mpsc::Sender;

pub(crate) fn stage_files(
    extracted: &ExtractedPackage,
    temp_dir: &Path,
    package_dir: &Path,
    root_dir: &Path,
    event_tx: &Sender<InstallEvent>,
    database: &mut Box<dyn PackageDatabase>,
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
    ostree: &OStreeRepo,
    database: &Database,
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
