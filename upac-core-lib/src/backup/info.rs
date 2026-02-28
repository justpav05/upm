use super::{OStreeError, Result};

use ostree::gio::{File, FileInfo, FileQueryInfoFlags, Cancellable};
use ostree::prelude::{FileExt, Cast};
use ostree::glib::VariantDict;
use ostree::{Repo, RepoFile};

use std::time::{Duration, UNIX_EPOCH};
use std::time::SystemTime;

pub(super) fn parse_commit_timestamp(variant: &ostree::glib::Variant) -> Result<SystemTime> {
    let secs = variant
        .child_value(5)
        .get()
        .ok_or_else(|| OStreeError::OSTreeFailed("Failed to read commit timestamp".into()))?;

    Ok(UNIX_EPOCH + Duration::from_secs(secs))
}

pub(super) fn parse_commit_description(variant: &ostree::glib::Variant) -> Result<String> {
    let subject = variant
        .child_value(3)
        .get()
        .ok_or_else(|| OStreeError::OSTreeFailed("Failed to read commit subject".into()))?;

    Ok(subject)
}

pub(super) fn parse_commit_package_list(repo: &Repo, commit_hash: &str) -> Result<Vec<String>> {
    let metadata = repo
        .read_commit_detached_metadata(commit_hash, None::<&Cancellable>)?;

    let Some(metadata) = metadata else {
        return Ok(vec![]);
    };

    let dict = VariantDict::new(Some(&metadata));

    // lookup возвращает Result<Option<T>, _>
    let packages: Vec<String> = dict
        .lookup::<Vec<String>>("packages")
        .map_err(|e| OStreeError::OSTreeFailed(e.to_string()))?
        .unwrap_or_default();

    Ok(packages)
}

//A crutch, because I don't know how to convert the type otherwise
//Костыль, потому что по другому я не знаю, как преобразовать тип
fn downcast_repo_file(file: File) -> Result<RepoFile> {
    file.downcast::<RepoFile>()
        .map_err(|_| OStreeError::OSTreeFailed("Failed to cast gio::File to RepoFile".into()))
}

fn read_commit_root(repo: &Repo, commit_hash: &str) -> Result<RepoFile> {
    let (root_file, _) = repo
        .read_commit(commit_hash, None::<&Cancellable>)?;

    downcast_repo_file(root_file)
}

fn repo_file_info(file: &RepoFile) -> Result<FileInfo> {
    let file_info = file.query_info(
        "standard::*,unix::*",
        FileQueryInfoFlags::NOFOLLOW_SYMLINKS,
        None::<&Cancellable>,
    )?;

    Ok(file_info)
}
