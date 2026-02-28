mod installer;
mod database;
mod backup;
mod index;
mod core;

// Публичное API для бекендов
pub use core::backend::{Backend, BackendError, ExtractedPackage, FileEntry};

// Публичное API для установщика
pub use installer::Install;
pub use installer::InstallEvent;
pub use installer::installer::Installer;

// Публичное API для базы данных
pub use database::{PackageRegistry, DatabaseError};
pub use database::database::Database;

// Публичное API для OSTree
pub use backup::{PackageRepo, CommitInfo};
pub use backup::manager::OStreeRepo;

// Типы
pub use core::types::{Package, PackageInfo, PackageMetadata, Dependency, PackageDiff};
