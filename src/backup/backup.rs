use super::{OSTree, OSTreeError, OSTreeOperation, OSTreeResult};

use crate::lock::{ExclusiveLock, Lock, SharedLock};

use ostree::gio::{Cancellable, File};
use ostree::prelude::Cast;
use ostree::{MutableTree, Repo};

use libc::AT_FDCWD;

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
