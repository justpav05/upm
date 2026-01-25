//! Database validation and integrity checks.

// ============================================================================
// Imports
// ============================================================================

use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use tokio::fs as async_fs;

use super::DataBase;
use crate::types::errors::DataBaseError;

// ============================================================================
// Validation & Integrity Checks
// ============================================================================

impl DataBase {
    /// Проверяет работоспособность пула через выполнение тестовой транзакции.
    ///
    /// Выполняет SQL-запрос в транзакции и откатывает её.
    /// Полезно для проверки синтаксиса SQL или доступности БД.
    ///
    /// # Аргументы
    /// * `sql` - SQL-запрос для выполнения
    ///
    /// # Примеры
    /// ```ignore
    /// db.check_pool_is_work("SELECT 1").await?;
    /// ```
    ///
    /// # Ошибки
    /// Возвращает ошибку если:
    /// - Не удалось начать транзакцию
    /// - SQL-запрос невалиден
    /// - БД недоступна
    pub async fn check_pool_is_work(&self, sql: &str) -> Result<(), DataBaseError> {
        let mut tx = self.pool().begin().await?;

        // Попытка выполнить транзакцию к базе данных
        sqlx::query(sql).execute(&mut *tx).await?;

        // Откат транзакции (не сохраняем изменения)
        tx.rollback().await?;

        Ok(())
    }

    /// Проверяет валидность пути к файлу базы данных.
    ///
    /// Проверяет:
    /// - Существование файла
    /// - Правильное расширение (.db)
    /// - Корректные права доступа (600 на Unix)
    ///
    /// # Аргументы
    /// * `database_path` - Путь к файлу базы данных
    ///
    /// # Примеры
    /// ```ignore
    /// DataBase::check_database_path_is_valid(Path::new("/var/lib/upm/packages.db")).await?;
    /// ```
    ///
    /// # Ошибки
    /// - `PathNotAccessible` - файл не существует
    /// - `InvalidDatabaseExtension` - неверное расширение файла
    /// - `IncorrectFilePermissions` - неверные права доступа (не 600)
    pub async fn check_database_path_is_valid(database_path: &Path) -> Result<(), DataBaseError> {
        // Проверка существования пути
        if !database_path.exists() {
            return Err(DataBaseError::PathNotAccessible(
                database_path.display().to_string(),
            ));
        }

        // Проверка расширения файла
        if database_path.extension().and_then(|s| s.to_str()) != Some("db") {
            return Err(DataBaseError::InvalidDatabaseExtension(
                database_path.display().to_string(),
            ));
        }

        // Проверка прав доступа (только на Unix)
        #[cfg(unix)]
        {
            let metadata = async_fs::metadata(database_path).await?;
            let mode = metadata.permissions().mode();

            // Проверка, что права доступа = 600 (rw-------)
            if (mode & 0o777) != 0o600 {
                return Err(DataBaseError::IncorrectFilePermissions(
                    database_path.display().to_string(),
                ));
            }
        }

        Ok(())
    }
}
