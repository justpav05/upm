use crate::{errors::InstallerResult, types::ExtractedPackage};

mod installer;

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

pub trait Installer {
	fn install(&mut self, package: ExtractedPackage) -> InstallerResult<()>;
	fn remove(&mut self, package: &str) -> InstallerResult<()>;
}
