// Imports
use super::types::PackageMetadata;

use abi_stable::std_types::{RStr, RString, RVec, RResult, RBoxError};
use abi_stable::sabi_trait;
use abi_stable::StableAbi;

use serde::{Deserialize, Serialize};
use std::fmt::Display;

// Alias for Result<T, BackendError>
pub type Result<T> = std::result::Result<T, BackendError>;

// Backend error enum
#[derive(StableAbi)]
#[repr(C)]
#[derive(Debug)]
pub enum BackendError {
    Io(RBoxError),
    InvalidPackage(RString),
    UnsupportedFormat(RString),
}

// Convert RBoxError to BackendError
impl From<RBoxError> for BackendError {
    fn from(err: RBoxError) -> Self {
        BackendError::Io(err)
    }
}

// Display implementation for BackendError
impl Display for BackendError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BackendError::Io(err) => write!(formatter, "IO error: {err}"),
            BackendError::InvalidPackage(string) => write!(formatter, "Invalid package: {string}"),
            BackendError::UnsupportedFormat(string) => write!(formatter, "Unsupported format: {string}"),
        }
    }
}

// Structure for working with an unpacked package
#[derive(StableAbi)]
#[repr(C)]
pub struct ExtractedPackage {
    pub name: RString,
    pub version: RString,
    pub format: RString,
    pub files: RVec<FileEntry>,
}

// File entry struct for working with package files
#[derive(StableAbi)]
#[repr(C)]
#[derive(Serialize, Deserialize)]
pub struct FileEntry {
    pub relative_path: RString,
    pub permissions: u32,
    #[serde(default)]
    pub owner: u32,
    #[serde(default)]
    pub group: u32,
}

// Trait for backend implementations
#[sabi_trait]
pub trait Backend: Send + Sync {

    fn name(&self) -> RStr<'static>;

    fn supported_formats(&self) -> RVec<RStr<'static>>;

    fn detect(&self, path: RStr<'_>) -> bool;

    fn extract(&self, path: RStr<'_>, temp_dir: RStr<'_>) -> RResult<ExtractedPackage, RString>;

    fn read_metadata(&self, path: RStr<'_>) -> RResult<PackageMetadata, RString>;
}
