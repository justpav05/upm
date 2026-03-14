// Imports
use crate::types::{ExtractedPackage, Package};
use crate::errors::{DatabaseResult, DatabaseError};

use std::path::{Path, PathBuf};

// Mods
pub mod database;

pub use database::PackageDatabase;

// Trait for package registry operations
pub trait Database {
    fn add_package(&mut self, package: &ExtractedPackage) -> DatabaseResult<()>;
    fn remove_package(&mut self, package_id: &str) -> DatabaseResult<()>;
    fn get_package(&self, query: &str) -> DatabaseResult<Package>;
    fn get_package_files(&self, package_id: &str) -> DatabaseResult<Vec<PathBuf>>;
    fn add_file(&mut self, package_id: &str, file_path: &Path) -> DatabaseResult<()>;
    fn remove_file(&mut self, package_id: &str, file_path: &Path) -> DatabaseResult<()>;

}
