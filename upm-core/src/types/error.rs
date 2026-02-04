// ============================================================================
// Imports
// ============================================================================
use crate::Package;
use std::path::PathBuf;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;
// ============================================================================
// Errors
// ============================================================================
#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Process not found: {0}")]
    ProcessNotFound(u32),

    #[error("Path error: {0}")]
    PathError(PathBuf),

    #[error("Lock error: {0}")]
    LockError(String),

    #[error("Package not found: {0}")]
    PackageNotFoundInBase(Package),

    #[error("Package not found on disk {0}")]
    PackageNotFoundOnDisk(Package),

    #[error("Package already exists: {0}")]
    PackageAlreadyExists(Package),

    #[error("Dependency resolution error: {0}")]
    DependencyResolveError(String),

    #[error("Package conflict error: {0}")]
    PackageConflictError(String),

    #[error("Package files damaged {0}")]
    PackageFilesDamaged(Package),

    #[error("Incompatible architecture for package {0}")]
    IncompatibleArchitecture(Package),

    #[error("Checksum mismatch for package {0}")]
    ChecksumMismatch(Package),

    #[error("Backend error: {0}")]
    BackendError(String),

    #[error("Repository error: {0}")]
    RepositoryError(String),

    #[error("Transaction error: {0}")]
    TransactionError(String),

    #[error("OSTree error: {0}")]
    OSTreeError(String),

    #[error("Permission denied")]
    PermissionDenied,

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Time stamp error: {0}")]
    TimeStampError(String),

    #[error("Parse transaction error: {0}")]
    ParseTransactionError(String),

    #[error("Step not found: {0}")]
    StepNotFound(String),

    #[error("Add step error: {0}")]
    AddStepError(String),

    #[error("Update step error {0}")]
    UpdateStepError(String),

    #[error("{0}")]
    Other(String),
}
