// Mods
mod installer;
mod database;
mod backup;
mod index;
mod core;

// Public API for backends
pub use core::backend::{Backend, BackendError, ExtractedPackage, FileEntry};

// Public API for the installer
pub use installer::Install;
pub use installer::InstallEvent;
pub use installer::installer::Installer;

// Public API for the database
pub use database::{PackageRegistry, DatabaseError};
pub use database::database::Database;

// Public API for OSTree
pub use backup::{PackageRepo, CommitInfo, OStreeRefCommitChange};
pub use backup::manager::OStreeRepo;

// Public types
pub use core::types::{Package, PackageInfo, PackageMetadata, Dependency, PackageDiff};
