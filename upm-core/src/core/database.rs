use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use tokio::fs as async_fs;

use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};

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
        // Проверка прав root (только для Unix-систем)
        #[cfg(unix)]
        {
            let uid = nix::unistd::Uid::effective();
            if !uid.is_root() {
                return Err(DataBaseError::InvalidPermissions(uid.as_raw()));
            }
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

        Ok(Self {
            pool,
            database_path,
        })
    }

    pub async fn close(&self) {
        self.pool.close().await;
    }

    pub async fn init_schema(&self, schema_path: &Path) -> Result<(), DataBaseError> {
        let sql_content = async_fs::read_to_string(schema_path)
            .await
            .map_err(DataBaseError::IoError)?;

        self.validate_sql(&sql_content).await?;
        self.apply_sql_file(schema_path).await?;

        Ok(())
    }

    pub async fn apply_sql_file(&self, sql_path: &Path) -> Result<(), DataBaseError> {
        let sql = async_fs::read_to_string(sql_path)
            .await
            .map_err(DataBaseError::IoError)?;
        sqlx::query(&sql).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn revert_sql_file(&self, sql_path: &Path) -> Result<(), DataBaseError> {
        let sql = async_fs::read_to_string(sql_path)
            .await
            .map_err(DataBaseError::IoError)?;
        sqlx::query(&sql).execute(&self.pool).await?;
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

    pub async fn check_database_valid(database_path: &Path) -> Result<(), DataBaseError> {
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

        let metadata = async_fs::metadata(database_path).await?;
        let mode = metadata.permissions().mode();

        if (mode & 0o777) != 0o600 {
            return Err(DataBaseError::IncorrectFilePermissions(
                database_path.display().to_string(),
            ));
        }

        Ok(())
    }
}

impl DataBase {
    /// Добавляет пакет в базу данных.
    ///
    /// # Аргументы
    /// * `package` - Ссылка на пакет для добавления
    ///
    /// # Ошибки
    /// Возвращает ошибку если:
    /// - Пакет с таким именем уже существует (UniqueConstraintViolated)
    /// - Произошла ошибка при выполнении SQL-запроса
    pub async fn add_package(&self, package: &Package) -> Result<(), DataBaseError> {
        const ADD_PACKAGE_SQL: &str = include_str!("../sql/queries/add_package.sql");

        // Используем вспомогательную функцию для биндинга всех полей
        Self::bind_package_fields(sqlx::query(ADD_PACKAGE_SQL), package)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Вспомогательная функция для биндинга всех полей пакета к SQL-запросу.
    /// Это позволяет переиспользовать логику биндинга в разных методах.
    fn bind_package_fields<'q>(
        query: sqlx::query::Query<'q, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'q>>,
        package: &'q Package,
    ) -> sqlx::query::Query<'q, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'q>> {
        query
            .bind(&package.id)
            .bind(&package.name)
            .bind(&package.version)
            .bind(&package.repository)
            .bind(package.state_of_instalation)
            .bind(&package.description)
            .bind(&package.license)
    }

    pub async fn get_database_package_by_name(
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

    pub async fn check_package_exists_in_database(
        &self,
        package_name: &str,
    ) -> Result<bool, DataBaseError> {
        const CHECK_EXISTS_SQL: &str = include_str!("../sql/queries/check_package_exists.sql");
        let package_exists = sqlx::query_scalar(CHECK_EXISTS_SQL)
            .bind(package_name)
            .fetch_one(&self.pool)
            .await?;
        Ok(package_exists)
    }

    pub async fn get_package_status_from_database(
        &self,
        package_name: &str,
    ) -> Result<bool, DataBaseError> {
        const GET_STATUS_SQL: &str = include_str!("../sql/queries/get_package_status.sql");
        let installation_status = sqlx::query_scalar(GET_STATUS_SQL)
            .bind(package_name)
            .fetch_one(&self.pool)
            .await?;
        Ok(installation_status)
    }

    pub async fn delete_package_from_database(
        &self,
        package_name: &str,
    ) -> Result<(), DataBaseError> {
        const DELETE_PACKAGE_SQL: &str = include_str!("../sql/queries/delete_package.sql");
        sqlx::query(DELETE_PACKAGE_SQL)
            .bind(package_name)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
