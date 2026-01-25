//! Configuration management.
//!
//! Handles PackageManager configuration and settings.

// ============================================================================
// Imports
// ============================================================================

use super::PackageManager;
use crate::types::errors::PackageError;

/// Конфигурация PackageManager.
#[derive(Debug, Clone)]
pub struct ManagerConfig {
    /// Автоматически создавать снапшоты перед установкой
    pub auto_snapshot: bool,
    /// Максимальное количество одновременных установок
    pub max_parallel_installs: usize,
    /// Таймаут операции в секундах
    pub operation_timeout: u64,
}

impl Default for ManagerConfig {
    fn default() -> Self {
        Self {
            auto_snapshot: true,
            max_parallel_installs: 4,
            operation_timeout: 300,
        }
    }
}

impl PackageManager {
    /// Получает текущую конфигурацию.
    ///
    /// # Примеры
    /// ```ignore
    /// let config = manager.get_config();
    /// println!("Auto snapshot: {}", config.auto_snapshot);
    /// ```
    pub fn get_config(&self) -> ManagerConfig {
        // TODO: Хранить конфигурацию в структуре PackageManager
        ManagerConfig::default()
    }

    /// Устанавливает новую конфигурацию.
    ///
    /// # Примеры
    /// ```ignore
    /// let mut config = manager.get_config();
    /// config.auto_snapshot = false;
    /// manager.set_config(config)?;
    /// ```
    pub fn set_config(&mut self, config: ManagerConfig) -> Result<(), PackageError> {
        log::info!("Updating PackageManager configuration");

        // TODO: Сохранить конфигурацию
        Ok(())
    }
}
