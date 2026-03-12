use crate::{errors::{InstallerResult, InstallerStabbyResult}, types::ExtractedPackage};

use stabby::str::Str as StabStr;

mod installer;

#[repr(u8)]
#[stabby::stabby]
#[derive(PartialEq, Eq)]
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

#[stabby::stabby]
pub trait Installer {
    extern "C" fn install(&mut self, package: ExtractedPackage) -> InstallerStabbyResult<()>;
    extern "C" fn remove(&mut self, package: StabStr) -> InstallerStabbyResult<()>;
}
