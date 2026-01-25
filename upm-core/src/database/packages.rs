//! CRUD operations for package management.

// ============================================================================
// Imports
// ============================================================================

use super::DataBase;
use crate::types::errors::DataBaseError;
use crate::types::package::{Package, PackageFieldUpdate};
use crate::types::traits::BindableFields;

// ============================================================================
// Package CRUD Operations
// ============================================================================

impl DataBase {
    /// Добавляет пакет в базу данных.
    ///
    /// # Аргументы
    /// * `package` - Ссылка на пакет для добавления
    ///
    /// # Примеры
    /// ```ignore
    /// let package = Package {
    ///     id: "nginx-1.25".to_string(),
    ///     name: "nginx".to_string(),
    ///     version: "1.25.0".to_string(),
    ///     ..Default::default()
    /// };
    ///
    /// db.add_package(&package).await?;
    /// ```
    ///
    /// # Ошибки
    /// Возвращает ошибку если:
    /// - Пакет с таким именем уже существует (`UniqueConstraintViolated`)
    /// - Произошла ошибка при выполнении SQL-запроса
    pub async fn add_package(&self, package: &Package) -> Result<(), DataBaseError> {
        const ADD_PACKAGE_SQL: &str = include_str!("../../sql/queries/add_package.sql");

        package
            .bind_to_insert_query(sqlx::query(ADD_PACKAGE_SQL))
            .execute(self.pool())
            .await?;

        Ok(())
    }

    /// Получает пакет из базы данных по имени.
    ///
    /// # Аргументы
    /// * `package_name` - Имя пакета для поиска
    ///
    /// # Возвращает
    /// - `Some(Package)` если пакет найден
    /// - `None` если пакет не найден
    ///
    /// # Примеры
    /// ```ignore
    /// match db.get_package_from_database_by_name("nginx").await? {
    ///     Some(pkg) => println!("Found: {} v{}", pkg.name, pkg.version),
    ///     None => println!("Package not found"),
    /// }
    /// ```
    pub async fn get_package_from_database_by_name(
        &self,
        package_name: &str,
    ) -> Result<Option<Package>, DataBaseError> {
        const GET_PACKAGE_SQL: &str = include_str!("../../sql/queries/get_package_by_name.sql");

        let package = sqlx::query_as::<_, Package>(GET_PACKAGE_SQL)
            .bind(package_name)
            .fetch_optional(self.pool())
            .await?;

        Ok(package)
    }

    /// Проверяет существование пакета в базе данных.
    ///
    /// # Аргументы
    /// * `package_name` - Имя пакета для проверки
    ///
    /// # Возвращает
    /// - `true` если пакет существует
    /// - `false` если пакет не найден
    ///
    /// # Примеры
    /// ```ignore
    /// if db.check_package_exists_in_database("nginx").await? {
    ///     println!("Package already installed");
    /// }
    /// ```
    pub async fn check_package_exists_in_database(
        &self,
        package_name: &str,
    ) -> Result<bool, DataBaseError> {
        const CHECK_EXISTS_SQL: &str = include_str!("../../sql/queries/check_package_exists.sql");

        let package_exists = sqlx::query_scalar(CHECK_EXISTS_SQL)
            .bind(package_name)
            .fetch_one(self.pool())
            .await?;

        Ok(package_exists)
    }

    /// Получает статус установки пакета.
    ///
    /// # Аргументы
    /// * `package_name` - Имя пакета
    ///
    /// # Возвращает
    /// - `true` если пакет установлен
    /// - `false` если пакет не установлен или повреждён
    ///
    /// # Примеры
    /// ```ignore
    /// let is_installed = db.get_package_status_from_database("nginx").await?;
    /// ```
    pub async fn get_package_status_from_database(
        &self,
        package_name: &str,
    ) -> Result<bool, DataBaseError> {
        const GET_STATUS_SQL: &str = include_str!("../../sql/queries/get_package_status.sql");

        let installation_status = sqlx::query_scalar(GET_STATUS_SQL)
            .bind(package_name)
            .fetch_one(self.pool())
            .await?;

        Ok(installation_status)
    }

    /// Удаляет пакет из базы данных.
    ///
    /// # Аргументы
    /// * `package_name` - Имя пакета для удаления
    ///
    /// # Примеры
    /// ```ignore
    /// db.delete_package_from_database("nginx").await?;
    /// ```
    ///
    /// # Ошибки
    /// Возвращает ошибку при проблемах с выполнением SQL-запроса.
    pub async fn delete_package_from_database(
        &self,
        package_name: &str,
    ) -> Result<(), DataBaseError> {
        const DELETE_PACKAGE_SQL: &str = include_str!("../../sql/queries/delete_package.sql");

        sqlx::query(DELETE_PACKAGE_SQL)
            .bind(package_name)
            .execute(self.pool())
            .await?;

        Ok(())
    }

    /// Изменяет состояние установки пакета в базе данных.
    ///
    /// # Аргументы
    /// * `package_name` - Имя пакета, состояние которого нужно изменить
    /// * `new_status` - Новое состояние установки (true = установлен, false = не установлен)
    ///
    /// # Примеры
    /// ```ignore
    /// // Отметить пакет как установленный после переустановки
    /// db.update_package_status_in_database("nginx", true).await?;
    ///
    /// // Отметить пакет как повреждённый/неустановленный
    /// db.update_package_status_in_database("nginx", false).await?;
    /// ```
    ///
    /// # Ошибки
    /// Возвращает ошибку если:
    /// - Пакет с таким именем не существует в базе данных
    /// - Произошла ошибка при выполнении SQL-запроса
    pub async fn update_package_status_in_database(
        &self,
        package_name: &str,
        new_status: bool,
    ) -> Result<(), DataBaseError> {
        const UPDATE_STATUS_SQL: &str = include_str!("../../sql/queries/update_package_status.sql");

        let result = sqlx::query(UPDATE_STATUS_SQL)
            .bind(new_status)
            .bind(package_name)
            .execute(self.pool())
            .await?;

        // Проверяем, что пакет был найден и обновлён
        if result.rows_affected() == 0 {
            return Err(DataBaseError::PackageNotFound(package_name.to_string()));
        }

        Ok(())
    }

    /// Обновляет все поля пакета в базе данных.
    ///
    /// Автоматически использует трейт `BindableFields` для биндинга всех полей.
    ///
    /// # Аргументы
    /// * `package` - Ссылка на структуру пакета с обновлёнными данными
    ///
    /// # Примеры
    /// ```ignore
    /// // Получаем пакет из БД
    /// let mut package = db.get_package_from_database_by_name("nginx").await?.unwrap();
    ///
    /// // Изменяем нужные поля
    /// package.state_of_instalation = true;
    /// package.version = "1.25.0".to_string();
    /// package.description = Some("Updated description".to_string());
    ///
    /// // Обновляем все поля в БД одним вызовом
    /// db.update_package_in_database(&package).await?;
    /// ```
    ///
    /// # Ошибки
    /// Возвращает ошибку если:
    /// - Пакет с таким ID не существует в базе данных
    /// - Произошла ошибка при выполнении SQL-запроса
    ///
    /// # Как это работает
    /// Использует метод `bind_to_update_query()` из трейта BindableFields,
    /// который автоматически биндит все поля в правильном порядке:
    /// 1. Все поля кроме id (для SET clause)
    /// 2. Поле id (для WHERE clause)
    pub async fn update_package_in_database(&self, package: &Package) -> Result<(), DataBaseError> {
        const UPDATE_PACKAGE_SQL: &str = include_str!("../../sql/queries/update_package.sql");

        let result = package
            .bind_to_update_query(sqlx::query(UPDATE_PACKAGE_SQL))
            .execute(self.pool())
            .await?;

        // Проверяем, что пакет был найден и обновлён
        if result.rows_affected() == 0 {
            return Err(DataBaseError::PackageNotFound(package.id.clone()));
        }

        Ok(())
    }

    /// Обновляет конкретное поле пакета в базе данных (type-safe).
    ///
    /// Позволяет обновить одно поле, используя структуру Package.
    /// Использует enum `PackageFieldUpdate` для type-safety.
    ///
    /// # Аргументы
    /// * `package` - Ссылка на структуру пакета (заимствование)
    /// * `field_update` - Enum с полем и новым значением
    ///
    /// # Примеры
    /// ```ignore
    /// use upm_core::types::package::PackageFieldUpdate;
    ///
    /// let package = db.get_package_from_database_by_name("nginx").await?.unwrap();
    ///
    /// // Обновить версию
    /// db.update_package_field_in_database(
    ///     &package,
    ///     PackageFieldUpdate::Version("1.25.0".to_string())
    /// ).await?;
    ///
    /// // Можно продолжать использовать package
    /// println!("Обновили пакет: {}", package.name);
    ///
    /// // Обновить статус
    /// db.update_package_field_in_database(
    ///     &package,
    ///     PackageFieldUpdate::Installed(true)
    /// ).await?;
    ///
    /// // Обновить описание
    /// db.update_package_field_in_database(
    ///     &package,
    ///     PackageFieldUpdate::Description(Some("New description".to_string()))
    /// ).await?;
    /// ```
    ///
    /// # Ошибки
    /// Возвращает ошибку если:
    /// - Пакет с таким именем не существует в базе данных
    /// - Произошла ошибка при выполнении SQL-запроса
    pub async fn update_package_field_in_database(
        &self,
        package: &Package,
        field_update: PackageFieldUpdate,
    ) -> Result<(), DataBaseError> {
        let sql = field_update.sql_query();

        let result = field_update
            .bind_value(sqlx::query(sql))
            .bind(&package.name)
            .execute(self.pool())
            .await?;

        // Проверяем, что пакет был найден и обновлён
        if result.rows_affected() == 0 {
            return Err(DataBaseError::PackageNotFound(package.name.clone()));
        }

        Ok(())
    }

    /// Получает все пакеты из базы данных.
    ///
    /// # Примеры
    /// ```ignore
    /// let all_packages = db.list_all_packages().await?;
    /// for pkg in all_packages {
    ///     println!("{} v{}", pkg.name, pkg.version);
    /// }
    /// ```
    pub async fn list_all_packages(&self) -> Result<Vec<Package>, DataBaseError> {
        const LIST_ALL_SQL: &str = "SELECT * FROM packages ORDER BY name";

        let packages = sqlx::query_as::<_, Package>(LIST_ALL_SQL)
            .fetch_all(self.pool())
            .await?;

        Ok(packages)
    }

    /// Получает только установленные пакеты.
    ///
    /// # Примеры
    /// ```ignore
    /// let installed = db.list_installed_packages().await?;
    /// ```
    pub async fn list_installed_packages(&self) -> Result<Vec<Package>, DataBaseError> {
        const LIST_INSTALLED_SQL: &str =
            "SELECT * FROM packages WHERE installed = TRUE ORDER BY name";

        let packages = sqlx::query_as::<_, Package>(LIST_INSTALLED_SQL)
            .fetch_all(self.pool())
            .await?;

        Ok(packages)
    }

    /// Получает только доступные (не установленные) пакеты.
    ///
    /// # Примеры
    /// ```ignore
    /// let available = db.list_available_packages().await?;
    /// ```
    pub async fn list_available_packages(&self) -> Result<Vec<Package>, DataBaseError> {
        const LIST_AVAILABLE_SQL: &str =
            "SELECT * FROM packages WHERE installed = FALSE ORDER BY name";

        let packages = sqlx::query_as::<_, Package>(LIST_AVAILABLE_SQL)
            .fetch_all(self.pool())
            .await?;

        Ok(packages)
    }
}
