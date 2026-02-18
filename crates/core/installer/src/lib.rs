use core::types::PackageInfo;
use std::path::Path;

mod installer;

pub trait Installer: Send + Sync {
    fn install_package(&mut self, package: &PackageInfo) -> Result<()>;

    fn remove_package(&mut self, package_id: &str) -> Result<()>;

    fn list_package_files(&self, package_id: &str) -> Result<Option<PackageInfo>>;

    fn add_file_to_package(&self, package_id: &str, file_path: &Path) -> Result<()>;

    fn remove_file_from_package(&self, package_id: &str, file_path: &Path) -> Result<()>;
}
