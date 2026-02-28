use super::{OStreeError, Result};

use crate::core::types::PackageDiff;

use ostree::gio::{File, Cancellable, InputStream};
use ostree::{MutableTree, Repo, RepoFile};
use ostree::prelude::{FileExt, Cast};

use std::path::{Path, PathBuf};
use std::ffi::OsStr;

pub(super) fn build_mtree(repo: &Repo, files: &[PathBuf]) -> Result<MutableTree> {
    let mtree = MutableTree::new();
    for path in files {
        insert_file_into_mtree(repo, &mtree, path)?;
    }

    Ok(mtree)
}

pub(super) fn write_commit(repo: &Repo, mtree: &MutableTree, diff: &PackageDiff) -> Result<String> {
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
//A crutch, because I don't know how to convert the type otherwise
//Костыль, потому что по другому я не знаю, как преобразовать тип
fn downcast_repo_file(file: File) -> Result<RepoFile> {
    file.downcast::<RepoFile>()
        .map_err(|_| OStreeError::OSTreeFailed("Failed to cast gio::File to RepoFile".into()))
}

fn checksum(repo: &Repo, file: &File) -> Result<String> {
    let stream = file.read(None::<&Cancellable>)?;
    let checksum = repo.write_content(None, &stream.upcast::<InputStream>(), 0, None::<&Cancellable>)?;

    Ok(checksum.to_string())
}

fn split_path<'a>(path: &'a Path) -> Result<(Vec<&'a OsStr>, &'a OsStr)> {
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

fn insert_file_into_mtree(repo: &Repo, mtree: &MutableTree, path: &Path) -> Result<()> {
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
