use crate::{OStreeError, Result};

use core::types::PackageDiff;

use database::database::{FileDatabase};

use ostree::{MutableTree, Repo, RepoFile, RepoCheckoutMode, RepoCheckoutOverwriteMode};
use ostree::prelude::{FileExt, Cast};
use ostree::gio::{File, FileInfo, FileQueryInfoFlags, Cancellable, InputStream};
use ostree::glib::VariantDict;

use database::Database;

use nix::unistd::{Uid, Gid, chown};

use std::fs;
use std::ffi::OsStr;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, UNIX_EPOCH};
use std::time::SystemTime;

//A crutch, because I don't know how to convert the type otherwise
//Костыль, потому что по другому я не знаю, как преобразовать тип
pub(crate) fn downcast_repo_file(file: File) -> Result<RepoFile> {
    file.downcast::<RepoFile>()
        .map_err(|_| OStreeError::OSTreeFailed("Failed to cast gio::File to RepoFile".into()))
}

pub(crate) fn set_permissions(path: &Path, mode: u32, uid: u32, gid: u32) -> Result<()> {
    fs::set_permissions(path, fs::Permissions::from_mode(mode))?;
    chown(path, Some(Uid::from_raw(uid)), Some(Gid::from_raw(gid)))?;
    Ok(())
}

pub(crate) fn checksum(repo: &Repo, file: &File) -> Result<String> {
    let stream = file.read(None::<&Cancellable>)?;
    let checksum = repo.write_content(None, &stream.upcast::<InputStream>(), 0, None::<&Cancellable>)?;

    Ok(checksum.to_string())
}

pub(crate) fn split_path<'a>(path: &'a Path) -> Result<(Vec<&'a OsStr>, &'a OsStr)> {
    let file_name = path.file_name()
        .ok_or_else(|| OStreeError::OSTreeFailed("Path has no filename".into()))?;

    let parent_components = path
        .parent()
        .unwrap_or(Path::new(""))
        .components()
        .filter_map(|c| match c {
            std::path::Component::Normal(s) => Some(s),
            _ => None,
        })
        .collect();

    Ok((parent_components, file_name))
}

pub(crate) fn collect_files(database: &FileDatabase) -> Result<Vec<PathBuf>> {
    let packages = database.list_all_packages()?;
    let mut files = Vec::new();
    for package in &packages {
        files.extend(database.get_files(&package.name)?);
    }
    Ok(files)
}

pub(crate) fn insert_file_into_mtree(repo: &Repo, mtree: &MutableTree, path: &Path) -> Result<()> {
    let (parent_os, file_name_os) = split_path(path)?;

    let parent_str = parent_os
        .iter()
        .map(|s| s.to_str().ok_or_else(|| OStreeError::OSTreeFailed("Non-UTF8 path component".into())))
        .collect::<Result<Vec<&str>>>()?;

    let file_name = file_name_os
        .to_str()
        .ok_or_else(|| OStreeError::OSTreeFailed("Non-UTF8 filename".into()))?;

    let gio_file = File::for_path(path);
    let file_checksum = checksum(repo, &gio_file)?;

    let parent_tree = mtree.walk(&parent_str, 1)?;
    parent_tree.replace_file(file_name, &file_checksum)?;

    Ok(())
}

pub(crate) fn build_mtree(repo: &Repo, files: &[PathBuf]) -> Result<MutableTree> {
    let mtree = MutableTree::new();
    for path in files {
        insert_file_into_mtree(repo, &mtree, path)?;
    }

    Ok(mtree)
}

pub(crate) fn write_commit(repo: &Repo, mtree: &MutableTree, diff: &PackageDiff) -> Result<String> {
	let root_file = repo.write_mtree(mtree, None::<&Cancellable>)?;
	let root_file: RepoFile = downcast_repo_file(root_file)?;

	let description = diff.to_description();

    let commit_hash = repo.write_commit(
        None,
        Some(description.as_str()),
        None,
        None,
        &root_file,
        None::<&Cancellable>,
    )?;

    parse_diff_metadata(repo, &commit_hash, diff)?;

    Ok(commit_hash.to_string())
}

pub(crate) fn read_commit_root(repo: &Repo, commit_hash: &str) -> Result<RepoFile> {
    let (root_file, _) = repo
        .read_commit(commit_hash, None::<&Cancellable>)?;

    downcast_repo_file(root_file)
}

pub(crate) fn repo_file_info(file: &RepoFile) -> Result<FileInfo> {
    let file_info = file.query_info(
        "standard::*,unix::*",
        FileQueryInfoFlags::NOFOLLOW_SYMLINKS,
        None::<&Cancellable>,
    )?;

    Ok(file_info)
}

pub(crate) fn checkout_to_root(repo: &Repo,
    commit_hash: &str,
    root_dir: &Path,
) -> Result<()> {
    let root_file = read_commit_root(&repo, commit_hash)?;
    let source_info = repo_file_info(&root_file)?;
    let destination = File::for_path(root_dir);

    repo.checkout_tree(
        RepoCheckoutMode::None,
        RepoCheckoutOverwriteMode::UnionFiles,
        &destination,
        &root_file,
        &source_info,
        None::<&Cancellable>,
    )?;

    Ok(())
}

pub(crate) fn parse_commit_timestamp(variant: &ostree::glib::Variant) -> Result<SystemTime> {
    let secs = variant
        .child_value(5)
        .get()
        .ok_or_else(|| OStreeError::OSTreeFailed("Failed to read commit timestamp".into()))?;

    Ok(UNIX_EPOCH + Duration::from_secs(secs))
}

pub(crate) fn parse_commit_description(variant: &ostree::glib::Variant) -> Result<String> {
    let subject = variant
        .child_value(3)
        .get()
        .ok_or_else(|| OStreeError::OSTreeFailed("Failed to read commit subject".into()))?;

    Ok(subject)
}

pub(crate) fn parse_commit_package_list(repo: &Repo, commit_hash: &str) -> Result<Vec<String>> {
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

fn parse_diff_metadata(repo: &Repo, commit_hash: &str, diff: &PackageDiff) -> Result<()> {
    let all_packages: Vec<&str> = diff.added.iter()
        .chain(diff.updated.iter())
        .map(|s| s.as_str())
        .collect();

    let metadata = ostree::glib::VariantDict::new(None);

    metadata.insert("packages", &all_packages.as_slice());

    metadata.insert("added",    &diff.added.iter().map(|s| s.as_str()).collect::<Vec<_>>().as_slice());

    metadata.insert("removed",  &diff.removed.iter().map(|s| s.as_str()).collect::<Vec<_>>().as_slice());

    metadata.insert("updated",  &diff.updated.iter().map(|s| s.as_str()).collect::<Vec<_>>().as_slice());

    repo.write_commit_detached_metadata(
        commit_hash,
        Some(&metadata.end()),
        None::<&Cancellable>,
    )?;

    Ok(())
}
