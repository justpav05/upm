//! High-level package management operations.
//!
//! Provides the main `PackageManager` struct that orchestrates all package operations
//! including installation, removal, updates, and snapshot management.

// ============================================================================
// Submodules
// ============================================================================

mod cache;
mod config;
mod manage_operation;
mod operations;
mod snapshots;

// ============================================================================
// Imports
// ============================================================================

use std::sync::Arc;

use crate::database::DataBase;
use crate::threadcoordination::ThreadCoordinator;

// ============================================================================
// Public API Re-exports
// ============================================================================

pub use self::cache::*;
pub use self::config::*;
pub use self::manage_operation::*;
pub use self::operations::*;
pub use self::snapshots::*;

// ============================================================================
// Core Types
// ============================================================================

/// Опции установки пакетов.
#[derive(Debug, Clone)]
pub struct InstallOptions {
    /// Конкретный бэкенд для использования
    pub backend: Option<String>,
    /// Стратегия разрешения зависимостей
    pub strategy: DependencyStrategy,
    /// Создавать ли снапшот системы перед установкой
    pub create_ostree_snapshot: bool,
}

impl Default for InstallOptions {
    fn default() -> Self {
        Self {
            backend: None,
            strategy: DependencyStrategy::Sat,
            create_ostree_snapshot: true,
        }
    }
}

/// Стратегия разрешения зависимостей.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DependencyStrategy {
    /// SAT solver (медленнее, но точнее)
    Sat,
    /// Greedy algorithm (быстрее, но может быть неоптимально)
    Greedy,
}

/// Опции удаления пакетов.
#[derive(Debug, Clone)]
pub struct RemoveOptions {
    /// Полное удаление (включая конфигурационные файлы)
    pub purge: bool,
    /// Удалять ли зависимости, которые больше не нужны
    pub remove_dependencies: bool,
}

impl Default for RemoveOptions {
    fn default() -> Self {
        Self {
            purge: false,
            remove_dependencies: true,
        }
    }
}

/// Результат операции с пакетами.
#[derive(Debug, Clone)]
pub struct OperationResult {
    /// Уникальный ID операции
    pub operation_id: String,
    /// Статус операции
    pub status: OperationStatus,
}

/// Статус операции.
#[derive(Debug, Clone)]
pub enum OperationStatus {
    /// Операция ожидает выполнения
    Pending,
    /// Операция выполняется
    Running {
        /// Прогресс (0-100)
        progress: u8,
        /// Текущий обрабатываемый пакет
        current_package: Option<String>,
    },
    /// Операция завершена успешно
    Completed {
        /// Количество успешно обработанных пакетов
        installed: usize,
        /// Количество неудачных
        failed: usize,
    },
    /// Операция провалилась
    Failed {
        /// Описание ошибки
        error: String,
    },
}

/// Снапшот системы (ostree).
#[derive(Debug, Clone)]
pub struct Snapshot {
    /// Уникальный ID снапшота
    pub id: String,
    /// Дата и время создания
    pub created: String,
    /// Описание снапшота
    pub description: String,
}

// ============================================================================
// Package Manager
// ============================================================================

/// Высокоуровневый менеджер пакетов.
///
/// Координирует работу между базой данных, координатором потоков,
/// и внешними бэкендами (apt, dnf, pacman и т.д.).
///
/// # Примеры
/// ```ignore
/// use upm_core::core::PackageManager;
/// use std::sync::Arc;
///
/// let coordinator = Arc::new(ThreadCoordinator::new(config).await?);
/// let database = Arc::new(DataBase::new(path, "packages.db", 10).await?);
///
/// let manager = PackageManager::new(coordinator, database);
///
/// // Установка пакетов
/// manager.install(vec!["nginx", "postgresql"], InstallOptions::default()).await?;
///
/// // Удаление пакетов
/// manager.remove(vec!["nginx"], RemoveOptions::default()).await?;
/// ```
pub struct PackageManager {
    /// Координатор параллельных операций
    coordinator: Arc<ThreadCoordinator>,
    /// База данных для хранения метаданных пакетов
    database: Arc<DataBase>,
}

impl PackageManager {
    /// Создаёт новый PackageManager.
    ///
    /// # Аргументы
    /// * `coordinator` - Координатор потоков для параллельной установки
    /// * `database` - База данных для хранения метаданных пакетов
    ///
    /// # Примеры
    /// ```ignore
    /// let manager = PackageManager::new(coordinator, database);
    /// ```
    pub fn new(coordinator: Arc<ThreadCoordinator>, database: Arc<DataBase>) -> Self {
        Self {
            coordinator,
            database,
        }
    }

    /// Возвращает ссылку на координатор потоков.
    pub fn coordinator(&self) -> &Arc<ThreadCoordinator> {
        &self.coordinator
    }

    /// Возвращает ссылку на базу данных.
    pub fn database(&self) -> &Arc<DataBase> {
        &self.database
    }
}
