use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use sqlx::{ConnectOptions, Connection, Sqlite};

use libc;

use crate::types::errors::DataBaseError;
use crate::types::package::Package;

pub struct DataBase {
    pool: SqlitePool,
    database_path: PathBuf,
}

impl DataBase {
    pub async fn new(
        database_dir_path: &Path,
        database_name: String,
        max_connections: u32,
    ) -> Result<Self, DataBaseError> {
        let uid = unsafe { libc::geteuid() };
        if uid != 0 {
            return Err(DataBaseError::InvalidPermissions(uid));
        }

        if !database_dir_path.exists() {
            return Err(DataBaseError::PathNotAccessible(
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
            return Err(DataBaseError::DatabaseValidationError(
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

    pub async fn init_schema(&self, schema_path: &Path) -> Result<(), DataBaseError> {
        let sql_content = std::fs::read_to_string(schema_path).map_err(DataBaseError::IoError)?;

        self.validate_sql(&sql_content).await?;
        self.apply_sql_file(&sql_content).await?;

        Ok(())
    }

    pub async fn apply_sql_file(&self, sql: &str) -> Result<(), DataBaseError> {
        sqlx::query(sql).execute(&self.pool).await?;
        Ok(())
    }
}

impl DataBase {
    pub async fn validate_sql(&self, sql: &str) -> Result<(), DataBaseError> {
        let mut tx = self.pool.begin().await?;
        sqlx::query(sql).execute(&mut *tx).await?;
        tx.rollback().await?;
        Ok(())
    }

    pub fn check_database_valid(database_path: &Path) -> Result<(), DataBaseError> {
        if !database_path.exists() {
            return Err(DataBaseError::PathNotAccessible(
                database_path.display().to_string(),
            ));
        }

        if database_path.extension().and_then(|s| s.to_str()) != Some("db") {
            return Err(DataBaseError::InvalidDatabaseExtension(
                database_path.display().to_string(),
            ));
        }

        let metadata = fs::metadata(database_path).map_err(DataBaseError::IoError)?;
        let mode = metadata.permissions().mode();

        if (mode & 0o600) != 0o600 {
            return Err(DataBaseError::IncorrectFilePermissions(
                database_path.display().to_string(),
            ));
        }

        Ok(())
    }
}

impl DataBase {
    pub async fn add_package(&self, package: &Package) -> Result<(), DataBaseError> {
        sqlx::query(
            r#"
            INSERT INTO packages (id, name, version, repository, installed, description, license)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&package.id)
        .bind(&package.name)
        .bind(&package.version)
        .bind(&package.repository)
        .bind(package.state_of_instalation)
        .bind(&package.description)
        .bind(&package.license)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_package_by_name(
        &self,
        package_name: &str,
    ) -> Result<Option<Package>, DataBaseError> {
        const GET_PACKAGE_SQL: &str = include_str!("../sql/queries/get_package_by_name.sql");
        let package = sqlx::query_as::<_, Package>(GET_PACKAGE_SQL)
            .bind(package_name)
            .fetch_optional(&self.pool)
            .await?;
        Ok(package)
    }

    pub async fn check_package_exists(&self, package_name: &str) -> Result<bool, DataBaseError> {
        const CHECK_EXISTS_SQL: &str = include_str!("../sql/queries/check_package_exists.sql");
        let package_exists = sqlx::query_scalar(CHECK_EXISTS_SQL)
            .bind(package_name)
            .fetch_one(&self.pool)
            .await?;
        Ok(package_exists)
    }

    pub async fn get_package_status(&self, package_name: &str) -> Result<bool, DataBaseError> {
        const GET_STATUS_SQL: &str = include_str!("../sql/queries/get_package_status.sql");
        let installation_status = sqlx::query_scalar(GET_STATUS_SQL)
            .bind(package_name)
            .fetch_one(&self.pool)
            .await?;
        Ok(installation_status)
    }

    pub async fn delete_package(&self, package_name: &str) -> Result<(), DataBaseError> {
        const DELETE_PACKAGE_SQL: &str = include_str!("../sql/queries/delete_package.sql");
        sqlx::query(DELETE_PACKAGE_SQL)
            .bind(package_name)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
