// ============================================================================
// Imports
// ============================================================================
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::SystemTime;

use crate::types::FileEntry;
use crate::types::PackageMetadata;
use crate::types::Scripts;
// ============================================================================
// Package
// ============================================================================
#[derive(Clone)]
pub struct Package {
    pub id: String,
    pub name: String,
    pub version: String,
    pub format: String,
    pub category: PackageCategory,
    pub metadata: PackageMetadata,
    pub files: Vec<FileEntry>,
    pub scripts: Scripts,
    pub installed_at: Option<SystemTime>,
}

impl Package {
    pub fn new(name: &str, version: &str, format: &str) -> Self;
    pub fn full_name(&self) -> String;
    pub fn is_installed(&self) -> bool;
}
// ============================================================================
// Package info
// ============================================================================
pub struct PackageInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub format: String,
    pub category: PackageCategory,
    pub description: String,
    pub size: u64,
    pub repository: String,
}
// ============================================================================
// Extracted package
// ============================================================================
pub struct ExtractedPackage {
    pub temp_path: PathBuf,
    pub files: Vec<FileEntry>,
    pub metadata: PackageMetadata,
    pub scripts: Scripts,
}

impl Drop for ExtractedPackage {
    fn drop(&mut self);
}
// ============================================================================
// Package category
// ============================================================================
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PackageCategory {
    Native,    // DEB, RPM, Arch, AUR
    Universal, // Flatpak, Snap, AppImage
}
