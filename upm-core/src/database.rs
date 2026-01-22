use sqlx::sqlite::{SqlitePool, SqliteConnectOptions, SqlitePoolOptions};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::{fs, io};
use libc;

pub struct Database {
    pool: SqlitePool,
    db_path: PathBuf,
}

#[derive(Debug)]
enum DbError {
    PathNotAccessible(String),
    DatabaseExists(String),
    CannotCreateDirectory(io::Error),
    InvalidPermissions(String),
    PathIsFile(String),
    DatabaseCorrupted(String),
    IoError(io::Error),
}

impl Database {
    pub async fn new(database_dir_path: &Path, database_name: String, max_connection_for_pool: u8) -> Result<Self> {
        //log::info!("Initializing database at: {:?}", db_path);
        if unsafe { libc::geteuid() } != 0 { return Err(DbError::InvalidPermissions(
            format!("Недостаточно прав. Текущий UID: {}. Требуется root (UID 0)", unsafe { libc::geteuid() })
            ));
        }

        let database_path = database_dir_path.join(database_name)
        if database_path.exists() { return Err(DbError::DatabaseExists(
            format!("Database {} aalready exists\n", database_name)
        )); }

        let connect_options = SqliteConnectOptions::from_str(
            &format!("sqlite://{}", database_path.display())
        )?
        .create_if_missing(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(max_connection_for_pool)
            .connect_with(connect_options)
            .await?;

        // log::info!("Database connected successfully");

        Ok(Self { pool, database_path })
    }

    pub fn check_database_valid(database_path: &Path) -> Result<(), DbError> {
        if !database_path.exists() { return Err(DbError::PathNotAccessible(
            format!("Path doesn`t exists: \n", database_path)
        )); }

        if database_path.extension().and_then(|string| string.to_str()) != Some("db") {
            return Err(DbError::PathNotAccessible(
                format!("Path incorrect!\n")
            ))
        }

        let metadata = fs::metadata(database_path).map_err(|e| DbError::IoError(e))?;
        let mod = metadata.permissions().mode();

        if (mode & 0o600) != 0o600 { return Err(DbError::InvalidPermissions(
            format!("Database acsses error!")
        )); }

        return Ok(());
    }

    pub async fn init_schema(&self) -> Result<()> {
        //log::info!("Initializing database schema");

        // Создать таблицу пакетов
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS packages (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                version TEXT NOT NULL,
                description TEXT,
                homepage TEXT,
                license TEXT,
                size_bytes INTEGER,
                download_url TEXT,
                installed BOOLEAN DEFAULT 0,
                installed_version TEXT,
                installed_time TIMESTAMP,
                repository TEXT,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Создать индексы
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_packages_name ON packages(name)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_packages_backend ON packages(backend)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_packages_installed ON packages(installed)")
            .execute(&self.pool)
            .await?;

        // Создать таблицу зависимостей
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS dependencies (
                id INTEGER PRIMARY KEY,
                package_id TEXT NOT NULL,
                dependency_id TEXT NOT NULL,
                version_constraint TEXT,
                is_optional BOOLEAN DEFAULT 0,
                FOREIGN KEY (package_id) REFERENCES packages(id),
                FOREIGN KEY (dependency_id) REFERENCES packages(id)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Создать таблицу операций
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS operations (
                id TEXT PRIMARY KEY,
                operation_type TEXT NOT NULL,
                packages TEXT NOT NULL,
                status TEXT NOT NULL,
                started_at TIMESTAMP,
                completed_at TIMESTAMP,
                error_message TEXT,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Создать таблицу снапшотов
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS snapshots (
                id TEXT PRIMARY KEY,
                commit_hash TEXT NOT NULL,
                description TEXT,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                size_bytes INTEGER,
                can_rollback BOOLEAN DEFAULT 1
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        log::info!("Database schema initialized successfully");

        Ok(())
    }

    /// Получить репозиторий пакетов
    pub fn packages(&self) -> PackageRepository {
        PackageRepository::new(self.pool.clone())
    }

    /// Получить репозиторий операций
    pub fn operations(&self) -> OperationRepository {
        OperationRepository::new(self.pool.clone())
    }

    /// Закрыть все подключения
    pub async fn close(&self) -> Result<()> {
        self.pool.close().await;
        log::info!("Database closed");
        Ok(())
    }
}

impl std::fmt::Display for DbError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DbError::PathNotAccessible(msg) => write!(f, "Путь недоступен: {}", msg),
            DbError::CannotCreateDirectory(e) => write!(f, "Не удалось создать директорию: {}", e),
            DbError::InvalidPermissions(msg) => write!(f, "Недостаточно прав: {}", msg),
            DbError::PathIsFile(msg) => write!(f, "Путь указывает на файл, а не директорию: {}", msg),
            DbError::DatabaseCorrupted(msg) => write!(f, "База данных повреждена: {}", msg),
            DbError::IoError(e) => write!(f, "Ошибка ввода-вывода: {}", e),
        }
    }
}
