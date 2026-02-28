// Imports
use super::{PackageRepo, CommitInfo, Result, OStreeRefCommitChange};
use super::OStreeError;

use super::info::{parse_commit_description, parse_commit_package_list, parse_commit_timestamp};
use super::rollback::{collect_commit_files, collect_disk_files, checkout_to_root};
use super::commit::{build_mtree, write_commit};

use crate::core::permitions::set_permissions;
use crate::{PackageDiff};

use ostree::{Repo, RepoMode, RepoPruneFlags};
use ostree::gio::Cancellable;
use ostree::ObjectType;

use nix::unistd::{getuid, getgid};

use std::path::{PathBuf, Path};
use std::fs;

// Structure for repository management
pub struct OStreeRepo {
    repo_path: PathBuf,
    mode: RepoMode,
    repo: Repo,
}

// Original functions for repository operation
impl OStreeRepo {

	// Creating a new repository in a directory
	pub fn create(repo_path: PathBuf, mode: RepoMode) -> Result<Self> {
        if repo_path.exists() && fs::read_dir(&repo_path)?.next().is_some() {
            return Err(OStreeError::RepoPathError(repo_path.clone()));
        }

        fs::create_dir_all(&repo_path)?;

        let uid = getuid().as_raw() as u32;
        let gid = getgid().as_raw() as u32;

        set_permissions(&repo_path, 0o755, uid, gid)?;

        let repo = Repo::new_for_path(&repo_path);
        repo.create(mode, None::<&Cancellable>)?;

        Ok(Self { repo_path, mode, repo })
    }

    // Open an existing repository
    pub fn open(repo_path: PathBuf) -> Result<Self> {
        if !repo_path.exists() {
            return Err(OStreeError::NotFound(repo_path));
        }

        let repo = Repo::new_for_path(&repo_path);
        repo.open(None::<&Cancellable>)?;

        Ok(Self { repo_path, mode: repo.mode(), repo })
    }

    // Delete the repository
    pub fn delete(self) -> Result<()> {
        fs::remove_dir_all(&self.repo_path)?;
        Ok(())
    }

}

// PackageRepo implementation for OStreeRepo
impl PackageRepo for OStreeRepo {

	// Creating a new commit and writing it to the repository
    fn commit(&self, files: Vec<PathBuf>, diff: &PackageDiff) -> Result<String> {
        let mtree = build_mtree(&self.repo, &files)?;
        let commit_hash = write_commit(&self.repo, &mtree, diff)?;
        Ok(commit_hash)
    }

    // Removing a repository from a commit
    fn delete(&self, ref_name: &str) -> Result<()> {
        self.repo
            .resolve_rev(ref_name, false)
            .map_err(|e| OStreeError::CommitNotFound(e.to_string()))?;

        self.repo.set_ref_immediate(None, ref_name, None, None::<&Cancellable>)?;

        self.repo.prune(RepoPruneFlags::REFS_ONLY, -1, None::<&Cancellable>)?;

        Ok(())
    }

    // Rolling back the state of the repository and installed programs to a specific commit
    fn rollback_to(&self, commit_hash: &str, root_dir: &Path) -> Result<()> {
        let commit_files = collect_commit_files(&self.repo, commit_hash)?;
        let disk_files = collect_disk_files(root_dir)?;

        for relative_path in disk_files.difference(&commit_files) {
            let full_path = root_dir.join(relative_path);
            fs::remove_file(&full_path)?;
        }

        checkout_to_root(&self.repo, commit_hash, root_dir)?;

        Ok(())
    }

    // Getting human-readable information about a commit
    fn get_commit_info(&self, commit_hash: &str) -> Result<CommitInfo> {
        let variant = self.repo
            .load_variant(ObjectType::Commit, commit_hash)
            .map_err(|e| OStreeError::CommitNotFound(e.to_string()))?;

        let timestamp = parse_commit_timestamp(&variant)?;
        let description = parse_commit_description(&variant)?;
        let package_list = parse_commit_package_list(&self.repo, commit_hash)?;

        Ok(CommitInfo {
            hash: commit_hash.to_string(),
            timestamp,
            package_list,
            description,
        })
    }
}

// PackageRepo implementation for OStreeRefCommitChange
impl OStreeRefCommitChange for OStreeRepo {

	// Get commit hash for a given ref name
	fn resolve_ref(&self, ref_name: &str) -> Result<String> {
    	self.repo.resolve_rev(ref_name, true).map_err(|e: ostree::glib::Error| OStreeError::CommitNotFound(e.to_string()))?.ok_or_else(|| OStreeError::CommitNotFound(ref_name.to_string())).map(|s| s.to_string())
	}

	// Find a ref name for a given commit hash
    fn find_ref(&self, commit_hash: &str) -> Result<Option<String>> {
        let refs = self.repo.list_refs(None, None::<&Cancellable>).map_err(|err| OStreeError::OSTreeFailed(err.to_string()))?;

        for (ref_name, hash) in refs {
            if hash.to_string() == commit_hash {
                return Ok(Some(ref_name));
            }
        }

        Ok(None)
    }
}
