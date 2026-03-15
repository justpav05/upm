// mod.rs
use upac_types::ExtractedPackage;
use upac_types::{InstallerError, InstallerResult, InstallerStabbyResult};

pub mod installer;
pub use installer::PackageInstaller;

#[repr(u8)]
#[stabby::stabby]
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum InstallerState {
    Idle,
    Preparing,
    Copying,
    Deleting,
    Registering,
    RollingBack,
    Success,
    Failed,
}

pub(crate) trait Installer {
    fn install(&mut self, package: ExtractedPackage) -> InstallerResult<()>;
    fn remove(&mut self, package: &str) -> InstallerResult<()>;
}
