//! Database layer for package management.
//!
//! Provides SQLite-based storage for package metadata with connection pooling,
//! health checks, and CRUD operations.

// ============================================================================
// Submodules
// ============================================================================

mod connection;
mod packages;
mod validation;

// ============================================================================
// Imports
// ============================================================================

use std::path::{Path, PathBuf};
use std::str::FromStr;

// ============================================================================
// Public API Re-exports
// ============================================================================

// Экспортируем методы из подмодулей (они будут impl DataBase в других файлах)
pub use self::connection::*;
pub use self::packages::*;
pub use self::validation::*;

// ============================================================================
// Database Initialization
// ============================================================================

impl DataBase {
    /// Создаёт новое подключение к базе данных.
    ///
    /// # Аргументы
    /// * `database_dir_path` - Путь к директории с базой данных
    /// * `database_name` - Имя файла базы данных (например, "packages.db")
    /// * `max_connections` - Максимальное количество соединений в пуле
    ///
    /// # Безопасность
    /// На Unix-системах требует root прав (UID 0).
    ///
    /// # Примеры
    /// ```ignore
    /// let db = DataBase::new(
    ///     Path::new("/var/lib/upm"),
    ///     "packages.db".to_string(),
    ///     10
    /// ).await?;
    /// ```
    ///
    /// # Ошибки
    /// - `InvalidPermissions` - недостаточно прав (не root)
    /// - `PathNotAccessible` - путь не существует
    /// - Ошибки подключения к SQLite
    pub async fn new(
        database_dir_path: &Path,
        database_name: String,
        max_connections: u32,
    ) -> Result<Self, DataBaseError> {
        #[cfg(unix)]
        {
            // Получение прав root, проверка прав root (только для Unix-систем)
            let uid = nix::unistd::Uid::effective();
            if !uid.is_root() {
                return Err(DataBaseError::InvalidPermissions(uid.as_raw()));
            }

            // Проверка существования пути к базе данных
            if !database_dir_path.exists() {
                return Err(DataBaseError::PathNotAccessible(
                    database_dir_path.display().to_string(),
                ));
            }
        }

        // Получение финального пути базы данных
        let database_path = database_dir_path.join(&database_name);
        //Создание файла базы данных, если она не существует и подключение к текущей
        let connect_options =
            SqliteConnectOptions::from_str(&format!("sqlite://{}", database_path.display()))?
                .create_if_missing(true);

        // Создание пула соединений
        let pool = SqlitePoolOptions::new()
            .max_connections(max_connections)
            .connect_with(connect_options)
            .await?;

        // Создаём структуру базы данных
        let database = Self {
            pool,
            database_path,
            max_connections,
        };

        // Автоматически инициализируем схему таблицы работы с паакетами из SQL-файла
        const SCHEMA_SQL: &str = include_str!("../sql/schema.sql");
        sqlx::query(SCHEMA_SQL).execute(&database.pool).await?;

        // Возвращаем готовую базу данных со схемой
        Ok(database)
    }

    /// Возвращает ссылку на пул соединений (для использования в других impl блоках)
    #[inline]
    pub(crate) fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Возвращает путь к базе данных
    #[inline]
    pub(crate) fn database_path(&self) -> &Path {
        &self.database_path
    }

    /// Возвращает максимальное количество соединений
    #[inline]
    pub(crate) fn max_connections(&self) -> u32 {
        self.max_connections
    }
}
