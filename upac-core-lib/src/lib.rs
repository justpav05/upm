mod core;
mod database;
mod installer;
mod backup;

// Публичное API для бекендов
pub use core::backend::{Backend, BackendError, ExtractedPackage, FileEntry};

// Публичное API для установщика
pub use installer::Installer;
pub use installer::installer::InstallerManager;
pub use installer::events::InstallEvent;

// Публичное API для базы данных
pub use database::{Database, Error as DatabaseError};
pub use database::database::FileDatabase;

// Публичное API для OSTree
pub use backup::{OSTreeRepo, CommitInfo};
pub use backup::implement::OStreeManager;

// Типы
pub use core::types::{Package, PackageInfo, PackageMetadata, Dependency, PackageDiff};
