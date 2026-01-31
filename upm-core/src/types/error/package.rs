// ============================================================================
// Imports
// ============================================================================
use thiserror::Error;

use crate::Package;

// ============================================================================
// Package Errors - Ошибки при работе с пакетами
// ============================================================================

/// Ошибки при работе с пакетами
///
/// Общие ошибки операций с пакетами:
/// - Пакет не найден
/// - Ошибки ввода-вывода
/// - Проблемы с конфигурацией
/// - Неудачные операции установки/удаления
#[derive(Debug, Error)]
pub enum PackageError {
    #[error("Package not found in base: {0}")]
    PackageNotFoundInBase(Package),

    #[error("Package not found on disk {0}")]
    PackageNotFoundOnDisk(Package),

    #[error("Package already exists: {0}")]
    PackageAlreadyExists(Package),

    #[error("Package files damaged {0}")]
    PackageFilesDamaged(Package),

    #[error("Checksum mismatch for package {0}")]
    ChecksumMismatch(Package),

    #[error("Dependency for package can`t be found {0}")]
    DependencyCanNotBeFound(Package),

    #[error("Dependency for package can`t be found {0}")]
    CyclicDependencies(Package),

    #[error("Incompatible architecture for package {0}")]
    IncompatibleArchitecture(Package),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
