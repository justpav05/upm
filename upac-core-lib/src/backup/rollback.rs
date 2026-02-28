// Imports
use super::{OStreeError, Result};

use ostree::{Repo, RepoFile, RepoCheckoutMode, RepoCheckoutOverwriteMode};
use ostree::gio::{File, FileInfo, FileQueryInfoFlags, Cancellable};
use ostree::prelude::{FileExt, FileEnumeratorExt, Cast};
use ostree::gio::FileType;

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::fs;

// Get files from commit
pub(super) fn collect_commit_files(repo: &Repo, commit_hash: &str) -> Result<HashSet<PathBuf>> {
    let (root, _) = repo.read_commit(commit_hash, None::<&Cancellable>)?;
    let mut files = HashSet::new();
    collect_tree_files(&root, Path::new(""), &mut files)?;
    Ok(files)
}

// Get files from disk
pub(super) fn collect_disk_files(root_dir: &Path) -> Result<HashSet<PathBuf>> {
    let mut files = HashSet::new();
    collect_dir_files(root_dir, Path::new(""), &mut files)?;
    Ok(files)
}

// Checkout commit to root directory
pub(super) fn checkout_to_root(repo: &Repo,
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

//A crutch, because I don't know how to convert the type otherwise
fn downcast_repo_file(file: File) -> Result<RepoFile> {
    file.downcast::<RepoFile>()
        .map_err(|_| OStreeError::OSTreeFailed("Failed to cast gio::File to RepoFile".into()))
}

// Read commit root file
fn read_commit_root(repo: &Repo, commit_hash: &str) -> Result<RepoFile> {
    let (root_file, _) = repo
        .read_commit(commit_hash, None::<&Cancellable>)?;

    downcast_repo_file(root_file)
}

// Get file info from RepoFile
fn repo_file_info(file: &RepoFile) -> Result<FileInfo> {
    let file_info = file.query_info(
        "standard::*,unix::*",
        FileQueryInfoFlags::NOFOLLOW_SYMLINKS,
        None::<&Cancellable>,
    )?;

    Ok(file_info)
}

// Collect files from a directory
fn collect_dir_files(dir: &Path, relative: &Path, files: &mut HashSet<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name();
        let rel_path = relative.join(&name);
        let full_path = dir.join(&name);

        if full_path.is_dir() {
            collect_dir_files(&full_path, &rel_path, files)?;
        } else {
            files.insert(rel_path);
        }
    }
    Ok(())
}

// Collect files from a virtual file mtree
fn collect_tree_files(dir: &File, current_path: &Path, files: &mut HashSet<PathBuf>) -> Result<()> {
    let enumerator = dir.enumerate_children(
        "standard::*",
        FileQueryInfoFlags::NOFOLLOW_SYMLINKS,
        None::<&Cancellable>,
    )?;

    while let Some(info) = enumerator.next_file(None::<&Cancellable>)? {
        let name = info.name();
        let child_path = current_path.join(&name);
        let child = dir.child(&name);

        if info.file_type() == FileType::Directory {
            collect_tree_files(&child, &child_path, files)?;
        } else {
            files.insert(child_path);
        }
    }

    Ok(())
}
