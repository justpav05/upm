use sqlx::sqlite::{SqlitePool, SqliteConnectOptions, SqlitePoolOptions};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::{fs, io};
use libc;

use crate::errors::DbError;

pub struct Database {
    pool: SqlitePool,
    database_path: PathBuf,
}

impl Database {
    pub async fn new(database_dir_path: &Path, database_name: String, max_connection_for_pool: u32) -> Result<Self, DbError> {
        //log::info!("Initializing database at: {:?}", db_path);
        let uid = unsafe { libc::geteuid() };
        if uid != 0 {
            return Err(DbError::InvalidPermissions(uid));
        }

        let database_path = database_dir_path.join(&database_name);
        if database_path.exists() {
            return Err(DbError::DatabaseExists(database_name));
        }

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

    pub async fn close(&self) -> Result<()> {
        self.pool.close().await;
        //log::info!("Database closed");
        Ok(())
    }

    pub async fn init_schema(&self, schema_path: &Path) -> Result<(), DbError> {
        //log::info!("Initializing database schema from file: {:?}", schema_path);
        let sql_content = std::fs::read_to_string(schema_path)
            .map_err(DbError::IoError()?;

        Database::validate_sql(&sql_content)?;
        self.apply_sql(&sql_content).await?;

        //log::info!("Database schema initialized successfully");
        Ok(())
    }

    pub fn check_database_valid(database_path: &Path) -> Result<(), DbError> {
        if !database_path.exists() {
            return Err(DbError::PathNotAccessible(database_path.display().to_string()));
        }

        if database_path.extension().and_then(|s| s.to_str()) != Some("db") {
            return Err(DbError::InvalidDatabaseExtension(database_path.display().to_string()));
        }

        let metadata = fs::metadata(database_path).map_err(DbError::IoError)?;
        let mode = metadata.permissions().mode();

        if (mode & 0o600) != 0o600 {
            return Err(DbError::IncorrectFilePermissions(database_path.display().to_string()));
        }

        Ok(())
    }

    pub fn validate_sql(sql: &str) -> Result<(), DbError> {
        if !sql.contains("CREATE TABLE") {
            return Err(DbError::InvalidSqlFile("SQL-файл не содержит 'CREATE TABLE'. Файл может быть поврежден или пуст.".to_string()));
        }
        if !sql.contains("PRIMARY KEY") {
            return Err(DbError::InvalidSqlFile("В таблицах SQL-схемы отсутствует 'PRIMARY KEY'.".to_string()));
        }
        Ok(())
    }

    pub async fn apply_sql(&self, sql: &str) -> Result<(), DbError> {
        //log::info!("Applying SQL schema...");
        sqlx::query(sql)
            .execute(&self.pool)
            .await?;
        //log::info!("SQL schema applied successfully.");
        Ok(())
    }
}
