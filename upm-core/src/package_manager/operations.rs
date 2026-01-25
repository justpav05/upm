//! Package installation and removal operations.
//!
//! Handles the core logic for installing and removing packages,
//! including database updates and status tracking.

// ============================================================================
// Imports
// ============================================================================

use std::sync::Arc;
use uuid::Uuid;

use crate::database::DataBase;
use crate::threadcoordination::ThreadCoordinator;
use crate::types::errors::{DataBaseError, PackageError};
use crate::types::package::Package;

use super::{InstallOptions, OperationResult, OperationStatus, PackageManager, RemoveOptions};

// ============================================================================
// Installation Operations
// ============================================================================

impl PackageManager {
    /// Устанавливает один или несколько пакетов.
    ///
    /// # Процесс установки:
    /// 1. Проверка существования пакетов в БД
    /// 2. Создание снапшота (если включено в опциях)
    /// 3. Установка пакетов (пока заглушка, позже через бэкенды)
    /// 4. Обновление статусов в БД
    ///
    /// # Аргументы
    /// * `package_names` - Список имён пакетов для установки
    /// * `options` - Опции установки (снапшоты, стратегия и т.д.)
    ///
    /// # Примеры
    /// ```ignore
    /// let result = manager.install(
    ///     vec!["nginx", "postgresql"],
    ///     InstallOptions::default()
    /// ).await?;
    ///
    /// match result.status {
    ///     OperationStatus::Completed { installed, failed } => {
    ///         println!("Installed: {}, Failed: {}", installed, failed);
    ///     }
    ///     _ => {}
    /// }
    /// ```
    ///
    /// # Ошибки
    /// Возвращает ошибку если:
    /// - Не удалось подключиться к БД
    /// - Пакет уже установлен
    /// - Ошибка при создании снапшота
    pub async fn install(
        &self,
        package_names: Vec<&str>,
        options: InstallOptions,
    ) -> Result<OperationResult, PackageError> {
        log::info!("Installing packages: {:?}", package_names);

        let operation_id = Uuid::new_v4().to_string();

        // Счётчики для результата
        let mut installed_count = 0;
        let mut failed_count = 0;
        let mut errors = Vec::new();

        // Создаём снапшот перед установкой (если включено)
        if options.create_ostree_snapshot {
            // TODO: Интеграция с ostree
            // self.create_snapshot().await?;
        }

        // Устанавливаем каждый пакет
        for package_name in package_names {
            match self.install_single_package(package_name, &options).await {
                Ok(_) => {
                    installed_count += 1;
                }
                Err(e) => {
                    failed_count += 1;
                    let error_msg = format!("Failed to install {}: {}", package_name, e);
                    errors.push(error_msg);
                }
            }
        }

        // Формируем результат операции
        let status = if failed_count == 0 {
            OperationStatus::Completed {
                installed: installed_count,
                failed: 0,
            }
        } else if installed_count == 0 {
            OperationStatus::Failed {
                error: errors.join("; "),
            }
        } else {
            OperationStatus::Completed {
                installed: installed_count,
                failed: failed_count,
            }
        };

        Ok(OperationResult {
            operation_id,
            status,
        })
    }

    /// Устанавливает один пакет (внутренний метод).
    ///
    /// # Логика:
    /// 1. Проверяем, существует ли пакет в БД
    /// 2. Если существует и уже установлен → ошибка
    /// 3. Если существует, но не установлен → обновляем статус
    /// 4. Если не существует → создаём новую запись
    ///
    /// # Аргументы
    /// * `package_name` - Имя пакета
    /// * `options` - Опции установки
    async fn install_single_package(
        &self,
        package_name: &str,
        options: &InstallOptions,
    ) -> Result<(), PackageError> {
        log::debug!("Installing single package: {}", package_name);

        // Проверяем существование пакета в БД
        let package_exists = self
            .database
            .check_package_exists_in_database(package_name)
            .await
            .map_err(|e| PackageError::OperationFailed(e.to_string()))?;

        if package_exists {
            // Пакет уже есть в БД, проверяем его статус
            let is_installed = self
                .database
                .get_package_status_from_database(package_name)
                .await
                .map_err(|e| PackageError::OperationFailed(e.to_string()))?;

            if is_installed {
                return Err(PackageError::OperationFailed(format!(
                    "Package '{}' is already installed",
                    package_name
                )));
            }

            // Пакет есть, но не установлен → обновляем статус
            self.database
                .update_package_status_in_database(package_name, true)
                .await
                .map_err(|e| PackageError::OperationFailed(e.to_string()))?;
        } else {
            // Пакета нет в БД → создаём новую запись

            // TODO: В будущем получать информацию из бэкенда
            // Пока создаём минимальную запись
            let package = Package {
                id: format!("{}-unknown", package_name),
                name: package_name.to_string(),
                version: "unknown".to_string(),
                repository: options
                    .backend
                    .clone()
                    .unwrap_or_else(|| "default".to_string()),
                state_of_instalation: true,
                description: None,
                license: None,
            };

            self.database
                .add_package(&package)
                .await
                .map_err(|e| PackageError::OperationFailed(e.to_string()))?;
        }

        // TODO: Реальная установка через бэкенд
        // let backend = self.backend_manager.detect_backend_for_package(package_name).await;
        // backend.install(package_name).await?;

        Ok(())
    }
}

// ============================================================================
// Removal Operations
// ============================================================================

impl PackageManager {
    /// Удаляет один или несколько пакетов.
    ///
    /// # Процесс удаления:
    /// 1. Проверка существования пакетов в БД
    /// 2. Создание снапшота (опционально)
    /// 3. Удаление пакетов (пока заглушка, позже через бэкенды)
    /// 4. Обновление БД (изменение статуса или полное удаление)
    ///
    /// # Аргументы
    /// * `package_names` - Список имён пакетов для удаления
    /// * `options` - Опции удаления (purge, зависимости и т.д.)
    ///
    /// # Примеры
    /// ```ignore
    /// // Обычное удаление (оставляет запись в БД)
    /// let result = manager.remove(
    ///     vec!["nginx"],
    ///     RemoveOptions::default()
    /// ).await?;
    ///
    /// // Полное удаление с очисткой конфигов (purge)
    /// let result = manager.remove(
    ///     vec!["nginx"],
    ///     RemoveOptions { purge: true, remove_dependencies: false }
    /// ).await?;
    /// ```
    ///
    /// # Ошибки
    /// Возвращает ошибку если:
    /// - Пакет не найден в БД
    /// - Пакет не установлен
    /// - Ошибка при обновлении БД
    pub async fn remove(
        &self,
        package_names: Vec<&str>,
        options: RemoveOptions,
    ) -> Result<OperationResult, PackageError> {
        log::info!("Removing packages: {:?}", package_names);

        let operation_id = Uuid::new_v4().to_string();

        let mut removed_count = 0;
        let mut failed_count = 0;
        let mut errors = Vec::new();

        // Удаляем каждый пакет
        for package_name in package_names {
            match self.remove_single_package(package_name, &options).await {
                Ok(_) => {
                    removed_count += 1;
                    log::info!("Successfully removed: {}", package_name);
                }
                Err(e) => {
                    failed_count += 1;
                    let error_msg = format!("Failed to remove {}: {}", package_name, e);
                    log::error!("{}", error_msg);
                    errors.push(error_msg);
                }
            }
        }

        // Формируем результат операции
        let status = if failed_count == 0 {
            OperationStatus::Completed {
                installed: removed_count, // используем то же поле для удалённых
                failed: 0,
            }
        } else if removed_count == 0 {
            OperationStatus::Failed {
                error: errors.join("; "),
            }
        } else {
            OperationStatus::Completed {
                installed: removed_count,
                failed: failed_count,
            }
        };

        Ok(OperationResult {
            operation_id,
            status,
        })
    }

    /// Удаляет один пакет (внутренний метод).
    ///
    /// # Логика:
    /// 1. Проверяем существование пакета в БД
    /// 2. Проверяем, что пакет установлен
    /// 3. Если purge = true → полностью удаляем из БД
    /// 4. Если purge = false → только меняем статус на "не установлен"
    ///
    /// # Аргументы
    /// * `package_name` - Имя пакета
    /// * `options` - Опции удаления
    async fn remove_single_package(
        &self,
        package_name: &str,
        options: &RemoveOptions,
    ) -> Result<(), PackageError> {
        log::debug!("Removing single package: {}", package_name);

        // Проверяем существование пакета в БД
        let package_exists = self
            .database
            .check_package_exists_in_database(package_name)
            .await
            .map_err(|e| PackageError::OperationFailed(e.to_string()))?;

        if !package_exists {
            return Err(PackageError::PackageNotFound(format!(
                "Package '{}' not found in database",
                package_name
            )));
        }

        // Проверяем статус установки
        let is_installed = self
            .database
            .get_package_status_from_database(package_name)
            .await
            .map_err(|e| PackageError::OperationFailed(e.to_string()))?;

        if !is_installed {
            return Err(PackageError::OperationFailed(format!(
                "Package '{}' is not installed",
                package_name
            )));
        }

        // TODO: Реальное удаление через бэкенд
        // let backend = self.backend_manager.detect_backend_for_package(package_name).await;
        // backend.remove(package_name, options.purge).await?;

        // Обновляем БД
        if options.purge {
            // Полное удаление из БД
            log::debug!("Purging package '{}' from database", package_name);
            self.database
                .delete_package_from_database(package_name)
                .await
                .map_err(|e| PackageError::OperationFailed(e.to_string()))?;
        } else {
            // Только меняем статус
            log::debug!("Marking package '{}' as uninstalled", package_name);
            self.database
                .update_package_status_in_database(package_name, false)
                .await
                .map_err(|e| PackageError::OperationFailed(e.to_string()))?;
        }

        log::info!("Package '{}' removed successfully", package_name);
        Ok(())
    }
}

// ============================================================================
// Helper Methods
// ============================================================================

impl PackageManager {
    /// Проверяет, установлен ли пакет.
    ///
    /// # Примеры
    /// ```ignore
    /// if manager.is_package_installed("nginx").await? {
    ///     println!("nginx is installed");
    /// }
    /// ```
    pub async fn is_package_installed(&self, package_name: &str) -> Result<bool, PackageError> {
        let exists = self
            .database
            .check_package_exists_in_database(package_name)
            .await
            .map_err(|e| PackageError::OperationFailed(e.to_string()))?;

        if !exists {
            return Ok(false);
        }

        self.database
            .get_package_status_from_database(package_name)
            .await
            .map_err(|e| PackageError::OperationFailed(e.to_string()))
    }

    /// Получает информацию о пакете из БД.
    ///
    /// # Примеры
    /// ```ignore
    /// match manager.get_package_info("nginx").await? {
    ///     Some(pkg) => println!("{} v{}", pkg.name, pkg.version),
    ///     None => println!("Package not found"),
    /// }
    /// ```
    pub async fn get_package_info(
        &self,
        package_name: &str,
    ) -> Result<Option<Package>, PackageError> {
        self.database
            .get_package_from_database_by_name(package_name)
            .await
            .map_err(|e| PackageError::OperationFailed(e.to_string()))
    }

    /// Список всех установленных пакетов.
    ///
    /// # Примеры
    /// ```ignore
    /// let installed = manager.list_installed().await?;
    /// for pkg in installed {
    ///     println!("{} v{}", pkg.name, pkg.version);
    /// }
    /// ```
    pub async fn list_installed(&self) -> Result<Vec<Package>, PackageError> {
        log::info!("Listing installed packages");

        self.database
            .list_installed_packages()
            .await
            .map_err(|e| PackageError::OperationFailed(e.to_string()))
    }
}
