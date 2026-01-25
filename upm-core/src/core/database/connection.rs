//! Connection management and health checks for the database pool.

// ============================================================================
// Imports
// ============================================================================

use std::str::FromStr;

use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

use super::{DataBase, PoolInfo};
use crate::types::errors::DataBaseError;

// ============================================================================
// Connection Management
// ============================================================================

impl DataBase {
    /// Корректно закрывает все соединения с базой данных.
    ///
    /// Ожидает завершения всех активных запросов и закрывает пул.
    /// После вызова этой функции DataBase больше нельзя использовать.
    ///
    /// # Примеры
    /// ```ignore
    /// let db = DataBase::new(...).await?;
    ///
    /// // Работаем с БД
    /// db.add_package(&package).await?;
    ///
    /// // Корректно закрываем
    /// db.close_pool_connection().await;
    /// ```
    pub async fn close_pool_connection(&self) {
        self.pool().close().await;
    }

    /// Пересоздаёт пул подключений к базе данных.
    ///
    /// Полезно когда:
    /// - Соединение с БД было потеряно
    /// - Пул "завис" или не отвечает
    /// - Нужно восстановить подключение после сбоя
    ///
    /// # Примеры
    /// ```ignore
    /// let mut db = DataBase::new(...).await?;
    ///
    /// // Работаем с БД...
    ///
    /// // Что-то пошло не так, пересоздаём пул
    /// db.recreate_new_pool().await?;
    ///
    /// // Продолжаем работу
    /// db.add_package(&package).await?;
    /// ```
    ///
    /// # Ошибки
    /// Возвращает ошибку если не удалось создать новый пул подключений.
    ///
    /// # Примечание
    /// Старый пул будет корректно закрыт перед созданием нового.
    pub async fn recreate_new_pool(&mut self) -> Result<(), DataBaseError> {
        // Закрываем старый пул
        self.pool().close().await;

        // Создаём новый пул с теми же параметрами
        let connect_options = SqliteConnectOptions::from_str(&format!(
            "sqlite://{}",
            self.database_path().display()
        ))?
        .create_if_missing(true);

        // Пересоздаём пул (используем те же настройки, что были)
        self.pool = SqlitePoolOptions::new()
            .max_connections(self.max_connections())
            .connect_with(connect_options)
            .await?;

        Ok(())
    }

    /// Проверяет здоровье базы данных.
    ///
    /// Выполняет простой запрос для проверки доступности и работоспособности.
    ///
    /// # Возвращает
    /// - `true` если база данных доступна и отвечает на запросы
    /// - `false` если база данных недоступна, закрыта или не отвечает
    ///
    /// # Примеры
    /// ```ignore
    /// if !db.check_pool_is_healthy().await {
    ///     eprintln!("Database is not responding!");
    ///     db.recreate_new_pool().await?;
    /// }
    /// ```
    pub async fn check_pool_is_healthy(&self) -> bool {
        const HEALTH_CHECK_SQL: &str = include_str!("../../sql/queries/health_check.sql");

        sqlx::query(HEALTH_CHECK_SQL)
            .fetch_one(self.pool())
            .await
            .is_ok()
    }

    /// Возвращает информацию о состоянии пула соединений.
    ///
    /// # Возвращает
    /// Структуру `PoolInfo` с информацией о:
    /// - Общем количестве соединений
    /// - Количестве простаивающих соединений
    /// - Статусе закрытия пула
    ///
    /// # Примеры
    /// ```ignore
    /// let info = db.get_pool_info();
    /// println!("Pool size: {}, idle: {}", info.size, info.idle_connections);
    /// ```
    pub fn get_pool_info(&self) -> PoolInfo {
        PoolInfo {
            size: self.pool().size(),
            idle_connections: self.pool().num_idle(),
            is_closed: self.pool().is_closed(),
        }
    }
}
