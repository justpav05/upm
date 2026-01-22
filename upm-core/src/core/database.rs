use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use sqlx::{ConnectOptions, Connection, Sqlite};

use libc;

use crate::types::errors::DatabaseError;

pub struct Database {
    pool: SqlitePool,
    database_path: PathBuf,
}

impl Database {
    pub async fn new(
        database_dir_path: &Path,
        database_name: String,
        max_connections: u32,
    ) -> Result<Self, DatabaseError> {
        let uid = unsafe { libc::geteuid() };
        if uid != 0 {
            return Err(DatabaseError::InvalidPermissions(uid));
        }

        if !database_dir_path.exists() {
            return Err(DatabaseError::PathNotAccessible(
                database_dir_path.display().to_string(),
            ));
        }

        let database_path = database_dir_path.join(&database_name);

        let connect_options =
            SqliteConnectOptions::from_str(&format!("sqlite://{}", database_path.display()))?
                .create_if_missing(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(max_connections)
            .connect_with(connect_options)
            .await?;

        if !database_path.exists() {
            return Err(DatabaseError::DatabaseValidationError(
                database_path.display().to_string(),
            ));
        }

        Ok(Self {
            pool,
            database_path,
        })
    }

    pub async fn close(&self) {
        self.pool.close().await;
    }

    /// Initializes the database schema from a given SQL file.
    /// Requires an active connection.
    pub async fn init_schema(&self, schema_path: &Path) -> Result<(), DbError> {
        // This function now correctly uses `map_err` for `io::Error`
        let sql_content = std::fs::read_to_string(schema_path).map_err(DbError::IoError)?;

        Self::validate_sql(&sql_content)?;
        self.apply_sql(&sql_content).await?;

        Ok(())
    }

    pub async fn apply_sql(&self, sql: &str) -> Result<(), DbError> {
        if let Some(pool) = &self.pool {
            sqlx::query(sql).execute(pool).await?;
            Ok(())
        } else {
            Err(DbError::DatabaseNotConnected)
        }
    }

    pub fn validate_sql(sql: &str) -> Result<(), DbError> {
        if !sql.contains("CREATE TABLE") {
            return Err(DbError::InvalidSqlFile(
                "SQL-файл не содержит 'CREATE TABLE'. Файл может быть поврежден или пуст."
                    .to_string(),
            ));
        }
        if !sql.contains("PRIMARY KEY") {
            return Err(DbError::InvalidSqlFile(
                "В таблицах SQL-схемы отсутствует 'PRIMARY KEY'.".to_string(),
            ));
        }
        Ok(())
    }

    pub fn check_database_valid(database_path: &Path) -> Result<(), DbError> {
        if !database_path.exists() {
            return Err(DbError::PathNotAccessible(
                database_path.display().to_string(),
            ));
        }

        if database_path.extension().and_then(|s| s.to_str()) != Some("db") {
            return Err(DbError::InvalidDatabaseExtension(
                database_path.display().to_string(),
            ));
        }

        let metadata = fs::metadata(database_path).map_err(DbError::IoError)?;
        let mode = metadata.permissions().mode();

        if (mode & 0o600) != 0o600 {
            return Err(DbError::IncorrectFilePermissions(
                database_path.display().to_string(),
            ));
        }

        Ok(())
    }
}
