use super::{OSTree, OSTreeError, OSTreeOperation, OSTreeResult, OSTreeStabbyResult};

use crate::lock::{ExclusiveLock, Lock, SharedLock};

use ostree::gio::{Cancellable, File};
use ostree::prelude::Cast;
use ostree::{MutableTree, Repo};

use stabby::result::Result as StabResult;
use stabby::str::Str as StabStr;
use stabby::string::String as StabString;
use stabby::vec::Vec as StabVec;

use libc::AT_FDCWD;

use std::ffi::c_void;
use std::path::{Path, PathBuf};

const OSTREE_LOCK_FILE_NAME: &str = "ostree.lock";

pub struct OSTreeManager {
    repo_path: PathBuf,
}

impl OSTreeManager {
    pub fn new(repo_path: PathBuf) -> Self {
        Self { repo_path }
    }
}

impl OSTree for OSTreeManager {
    fn commit(
        &self,
        repo_path: &Path,
        parent_commit_hash: Option<&str>,
        operation: OSTreeOperation,
        packages: &[&str],
    ) -> OSTreeResult<String> {
        let lock = ExclusiveLock::new(self.repo_path.join(OSTREE_LOCK_FILE_NAME));
        let _guard = lock.lock()?;

        let repo = Repo::new(&File::for_path(repo_path));
        repo.open(Cancellable::NONE)?;

        repo.prepare_transaction(Cancellable::NONE)?;

        let mtree = MutableTree::new();
        repo.write_directory_to_mtree(&File::for_path(repo_path), &mtree, None, Cancellable::NONE)?;

        let root = repo.write_mtree(&mtree, Cancellable::NONE)?;
        let root = root
            .downcast_ref::<ostree::RepoFile>()
            .ok_or_else(|| OSTreeError::CommitFailed("Failed to cast to RepoFile".into()))?;

        let subject = operation.as_str();
        let body = packages.join("\n");

        let commit_hash = repo.write_commit(
            parent_commit_hash,
            Some(subject),
            Some(&body),
            None,
            &root,
            Cancellable::NONE,
        )?;

        repo.commit_transaction(Cancellable::NONE)?;

        Ok(commit_hash.to_string())
    }

    fn rollback(&self, commit_hash: &str) -> OSTreeResult<()> {
        let lock = ExclusiveLock::new(self.repo_path.join(OSTREE_LOCK_FILE_NAME));
        let _guard = lock.lock()?;

        let repo = Repo::new(&File::for_path(&self.repo_path));
        repo.open(Cancellable::NONE)?;

        let options = ostree::RepoCheckoutAtOptions {
            overwrite_mode: ostree::RepoCheckoutOverwriteMode::UnionFiles,
            ..Default::default()
        };

        repo.checkout_at(
            Some(&options),
            AT_FDCWD,
            &self.repo_path,
            commit_hash,
            Cancellable::NONE,
        )?;

        Ok(())
    }

    fn remove(&self, commit_hash: &str) -> OSTreeResult<()> {
        let lock = ExclusiveLock::new(self.repo_path.join(OSTREE_LOCK_FILE_NAME));
        let _guard = lock.lock()?;

        let repo = Repo::new(&File::for_path(&self.repo_path));
        repo.open(Cancellable::NONE)?;

        let refs = repo.list_refs(None, Cancellable::NONE)?;
        for (ref_name, hash) in refs.iter() {
            if hash == commit_hash {
                repo.set_ref_immediate(None, ref_name, None, Cancellable::NONE)?;
            }
        }

        repo.prune(ostree::RepoPruneFlags::REFS_ONLY, -1, Cancellable::NONE)?;

        Ok(())
    }

    fn list_commits(&self) -> OSTreeResult<Vec<String>> {
        let lock = SharedLock::new(self.repo_path.join(OSTREE_LOCK_FILE_NAME));
        let _guard = lock.lock()?;

        let repo = Repo::new(&File::for_path(&self.repo_path));
        repo.open(Cancellable::NONE)?;

        let refs = repo.list_refs(None, Cancellable::NONE)?;
        let commits = refs.values().map(|hash| hash.to_string()).collect();

        Ok(commits)
    }
}

#[no_mangle]
pub extern "C" fn upac_create_ostree(repo_path: StabStr) -> StabResult<*mut c_void, OSTreeError> {
    let manager = OSTreeManager::new(PathBuf::from(repo_path.as_str()));

    Ok(Box::into_raw(Box::new(manager)) as *mut c_void).into()
}

#[no_mangle]
pub extern "C" fn upac_free_ostree(manager: *mut c_void) {
    if !manager.is_null() {
        unsafe { drop(Box::from_raw(manager as *mut OSTreeManager)) };
    }
}

#[no_mangle]
pub extern "C" fn upac_commit(
    manager: *mut c_void,
    repo_path: StabStr,
    parent_commit_hash: StabStr,
    operation: u8,
    packages: StabVec<StabString>,
) -> OSTreeStabbyResult<StabString> {
    let manager = unsafe { &*(manager as *mut OSTreeManager) };

    let parent = if parent_commit_hash.is_empty() {
        None
    } else {
        Some(parent_commit_hash.as_str())
    };

    let operation = match operation {
        0 => OSTreeOperation::Install,
        1 => OSTreeOperation::Remove,
        _ => OSTreeOperation::Update,
    };

    let packages: Vec<&str> = packages.iter().map(|string| string.as_str()).collect();

    manager
        .commit(
            &PathBuf::from(repo_path.as_str()),
            parent,
            operation,
            &packages,
        )
        .map(|hash| StabString::from(hash.as_str()))
        .into()
}

#[no_mangle]
pub extern "C" fn upac_rollback(
    manager: *mut c_void,
    commit_hash: StabStr,
) -> OSTreeStabbyResult<()> {
    let manager = unsafe { &*(manager as *mut OSTreeManager) };

    manager.rollback(commit_hash.as_str()).into()
}

#[no_mangle]
pub extern "C" fn upac_remove_commit(
    manager: *mut c_void,
    commit_hash: StabStr,
) -> OSTreeStabbyResult<()> {
    let manager = unsafe { &*(manager as *mut OSTreeManager) };

    manager.remove(commit_hash.as_str()).into()
}

#[no_mangle]
pub extern "C" fn upac_list_commits(
    manager: *mut c_void,
) -> OSTreeStabbyResult<StabVec<StabString>> {
    let manager = unsafe { &*(manager as *mut OSTreeManager) };

    manager
        .list_commits()
        .map(|commits| {
            commits
                .into_iter()
                .map(|string| StabString::from(string.as_str()))
                .collect::<StabVec<StabString>>()
        })
        .into()
}
