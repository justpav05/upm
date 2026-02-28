// Imports
use super::{OStreeError, Result};

use ostree::gio::Cancellable;
use ostree::glib::VariantDict;
use ostree::Repo;

use std::time::{Duration, UNIX_EPOCH};
use std::time::SystemTime;

// Parse commit timestamp from a variant
pub(super) fn parse_commit_timestamp(variant: &ostree::glib::Variant) -> Result<SystemTime> {
    let secs = variant
        .child_value(5)
        .get()
        .ok_or_else(|| OStreeError::OSTreeFailed("Failed to read commit timestamp".into()))?;

    Ok(UNIX_EPOCH + Duration::from_secs(secs))
}

// Parse commit description from a variant
pub(super) fn parse_commit_description(variant: &ostree::glib::Variant) -> Result<String> {
    let subject = variant
        .child_value(3)
        .get()
        .ok_or_else(|| OStreeError::OSTreeFailed("Failed to read commit subject".into()))?;

    Ok(subject)
}

// Parse commit package list from a variant
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
