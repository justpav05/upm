use super::{OSTreeRepo, CommitInfo, Result};
use super::errors::OStreeError;
use super::helpers::{collect_files, build_mtree, write_commit, read_commit_root, repo_file_info, parse_commit_timestamp, parse_commit_description, parse_commit_package_list, checkout_to_root};

use crate::core::types::PackageDiff;
use crate::core::helpers::set_permissions;

use crate::database::database::FileDatabase;

use ostree::{Repo, RepoMode, RepoPruneFlags, RepoCheckoutMode, RepoCheckoutOverwriteMode};
use ostree::gio::{File, Cancellable};
use ostree::ObjectType;

use std::path::{Path, PathBuf};
use std::fs;

pub struct OStreeManager {
    repo_path: PathBuf,
    root_path: PathBuf,
    repo: Option<Repo>,
    mode: RepoMode,
}

impl OStreeManager {
    pub fn new(repo_path: PathBuf, root_path: PathBuf, mode: RepoMode) -> Self {
        Self {
            repo_path,
            root_path,
            repo: None,
            mode,
        }
    }
}

impl OSTreeRepo for OStreeManager {
    fn create_repo(&mut self, permissions: u32, uid: u32, gid: u32) -> Result<()> {
        if self.repo_path.exists() && fs::read_dir(&self.repo_path)?.next().is_some() {
            return Err(OStreeError::RepoPathError(self.repo_path.clone()));
        }

        fs::create_dir_all(&self.repo_path)?;
        set_permissions(&self.repo_path, permissions, uid, gid)?;

        let repo = Repo::new_for_path(&self.repo_path);

        repo.create(self.mode, None::<&Cancellable>)?;

        self.repo = Some(repo);

        Ok(())
    }

    fn delete_repo(&mut self) -> Result<()> {
        if !self.repo_path.exists() {
            return Err(OStreeError::NotFound(self.repo_path.clone()));
        }

        let repo = self.repo.as_ref()
            .ok_or_else(|| OStreeError::RepoPathError(self.repo_path.clone()))?;

        repo.open(None::<&Cancellable>)?;

        fs::remove_dir_all(&self.repo_path)?;

        Ok(())
    }

    fn create_commit(&self, database: &FileDatabase,  diff: &PackageDiff, root_dir: &Path) -> Result<String> {
        let repo = self.repo.as_ref()
            .ok_or_else(|| OStreeError::NotFound(self.repo_path.clone()))?;

        let files = collect_files(database)?;
        let mtree = build_mtree(repo, &files)?;
        let commit_hash = write_commit(repo, &mtree, diff)?;
        checkout_to_root(repo, &commit_hash, root_dir)?;
        Ok(commit_hash)
    }

    fn delete_commit(&self, commit_hash: &str) -> Result<()> {
        let repo = self.repo.as_ref()
            .ok_or_else(|| OStreeError::NotFound(self.repo_path.clone()))?;

        let _ = repo.traverse_commit(commit_hash, -1, None::<&Cancellable>)
            .map_err(|e| OStreeError::CommitNotFound(e.to_string()))?;

        repo.set_ref_immediate(None, commit_hash, None, None::<&Cancellable>)?;

        let (_, _pruned, _freed) = repo.prune(
            RepoPruneFlags::REFS_ONLY,
            -1,
            None::<&Cancellable>,
        )?;

        Ok(())
    }

    fn rollback_to(&self, commit_hash: &str) -> Result<()> {
        let repo = self.repo.as_ref()
            .ok_or_else(|| OStreeError::NotFound(self.repo_path.clone()))?;

        let root_file = read_commit_root(repo, commit_hash)?;
        let source_info = repo_file_info(&root_file)?;
        let destination = File::for_path(&self.root_path);

        repo.checkout_tree(
            RepoCheckoutMode::User,
            RepoCheckoutOverwriteMode::AddFiles,
            &destination,
            &root_file,
            &source_info,
            None::<&Cancellable>,
        )?;

        Ok(())
    }

    fn get_commit_info(&self, commit_hash: &str) -> Result<CommitInfo> {
        let repo = self.repo.as_ref()
            .ok_or_else(|| OStreeError::NotFound(self.repo_path.clone()))?;

        let variant = repo
            .load_variant(ObjectType::Commit, commit_hash)
            .map_err(|e| OStreeError::CommitNotFound(e.to_string()))?;

        let timestamp = parse_commit_timestamp(&variant)?;
        let description = parse_commit_description(&variant)?;
        let package_list = parse_commit_package_list(repo, commit_hash)?;

        Ok(CommitInfo {
            hash: commit_hash.to_string(),
            timestamp,
            package_list,
            description,
        })
    }
}
