use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use tokio::fs as async_fs;

use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};

use crate::types::errors::DataBaseError;
use crate::types::package::Package;
use crate::types::traits::BindableFields;

pub struct DataBase {
    // Пул соединений с базой данных
    pool: SqlitePool,
    // Сохранение пути к базе данных
    database_path: PathBuf,
    // Максимальное количество соединений (для пересоздания пула)
    max_connections: u32,
}
/// Информация о состоянии пула соединений с базой данных.
#[derive(Debug, Clone)]
pub struct PoolInfo {
    /// Общее количество соединений в пуле
    pub size: u32,
    /// Количество простаивающих (неиспользуемых) соединений
    pub idle_connections: usize,
    /// Закрыт ли пул соединений
    pub is_closed: bool,
}

impl DataBase {
    pub async fn new(
        database_dir_path: &Path,
        database_name: String,
        max_connections: u32,
    ) -> Result<Self, DataBaseError> {
        #[cfg(unix)]
        {
            // Получение прав рут, проверка прав рут (только для Unix-систем)
            let uid = nix::unistd::Uid::effective();
            if !uid.is_root() {
                return Err(DataBaseError::InvalidPermissions(uid.as_raw()));
            // Проверка существования пути к базе данных
            } else if !database_dir_path.exists() {
                return Err(DataBaseError::PathNotAccessible(
                    database_dir_path.display().to_string(),
                ));
            }
        }

        // Получение финального пути базы данных
        let database_path = database_dir_path.join(&database_name);

        let connect_options =
            SqliteConnectOptions::from_str(&format!("sqlite://{}", database_path.display()))?
                .create_if_missing(true);
        // Получение пула базы данных
        let pool = SqlitePoolOptions::new()
            .max_connections(max_connections)
            .connect_with(connect_options)
            .await?;

        // Создаём структуру базы данных
        let db = Self {
            pool,
            database_path,
            max_connections,
        };

        // Автоматически инициализируем схему из SQL-файла
        const SCHEMA_SQL: &str = include_str!("../sql/schema.sql");
        sqlx::query(SCHEMA_SQL).execute(&db.pool).await?;

        // Возвращаем готовую базу данных со схемой
        Ok(db)
    }
    // Функция закрытия базы данных, очистка структуры
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
    /// db.close().await;
    /// ```
    pub async fn close_pool_connection(&self) {
        self.pool.close().await;
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
    /// db.recreate_pool().await?;
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
        self.pool.close().await;

        // Создаём новый пул с теми же параметрами
        let connect_options =
            SqliteConnectOptions::from_str(&format!("sqlite://{}", self.database_path.display()))?
                .create_if_missing(true);

        // Пересоздаём пул (используем те же настройки, что были)
        self.pool = SqlitePoolOptions::new()
            .max_connections(self.max_connections)
            .connect_with(connect_options)
            .await?;

        Ok(())
    }
    /// Проверяет здоровье базы данных.
    /// Выполняет простой запрос для проверки доступности и работоспособности.
    ///
    /// # Возвращает
    /// - `true` если база данных доступна и отвечает на запросы
    /// - `false` если база данных недоступна, закрыта или не отвечает
    pub async fn check_pool_is_healthy(&self) -> bool {
        const HEALTH_CHECK_SQL: &str = include_str!("../sql/queries/health_check.sql");

        sqlx::query(HEALTH_CHECK_SQL)
            .fetch_one(&self.pool)
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
    pub fn get_pool_info(&self) -> PoolInfo {
        PoolInfo {
            size: self.pool.size(),
            idle_connections: self.pool.num_idle(),
            is_closed: self.pool.is_closed(),
        }
    }

    pub async fn check_pool_is_work(&self, sql: &str) -> Result<(), DataBaseError> {
        let mut tx = self.pool.begin().await?;
        // Попытка выполнить транзакцию к базе данных, если она не проходит, то возращается ошибка
        sqlx::query(sql).execute(&mut *tx).await?;
        //Откат транзакции в случае неудачи
        tx.rollback().await?;

        Ok(())
    }

    pub async fn check_database_path_is_valid(database_path: &Path) -> Result<(), DataBaseError> {
        // Проверка того, что путь валидный
        if !database_path.exists() {
            return Err(DataBaseError::PathNotAccessible(
                database_path.display().to_string(),
            ));
        }
        // Проверка того, что файл имеет правильное расширение
        if database_path.extension().and_then(|s| s.to_str()) != Some("db") {
            return Err(DataBaseError::InvalidDatabaseExtension(
                database_path.display().to_string(),
            ));
        }

        let metadata = async_fs::metadata(database_path).await?;
        let mode = metadata.permissions().mode();
        // Проверка разрешений для доступа к базе данных
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

        // Автоматически биндим все поля через трейт BindableFields
        // SQL: INSERT INTO packages (id, name, version, repository, installed, description, license)
        package
            .bind_to_insert_query(sqlx::query(ADD_PACKAGE_SQL))
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn get_package_from_database_by_name(
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

    /// Изменяет состояние установки пакета в базе данных.
    ///
    /// # Аргументы
    /// * `package_name` - Имя пакета, состояние которого нужно изменить
    /// * `new_status` - Новое состояние установки (true = установлен, false = не установлен)
    ///
    /// # Примеры
    /// ```ignore
    /// // Отметить пакет как установленный после переустановки
    /// db.update_package_status("nginx", true).await?;
    ///
    /// // Отметить пакет как повреждённый/неустановленный
    /// db.update_package_status("nginx", false).await?;
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
        const UPDATE_STATUS_SQL: &str = include_str!("../sql/queries/update_package_status.sql");

        let result = sqlx::query(UPDATE_STATUS_SQL)
            .bind(new_status)
            .bind(package_name)
            .execute(&self.pool)
            .await?;
        // Проверяем, что пакет был найден и обновлён
        if result.rows_affected() == 0 {
            return Err(DataBaseError::PackageNotFound(package_name.to_string()));
        }

        Ok(())
    }
    /// Обновляет все поля пакета в базе данных.
    /// Автоматически использует трейт BindableFields для биндинга всех полей.
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
        const UPDATE_PACKAGE_SQL: &str = include_str!("../sql/queries/update_package.sql");

        // Автоматически биндим все поля через трейт BindableFields
        // SQL: UPDATE packages SET name = ?, version = ?, repository = ?,
        //      installed = ?, description = ?, license = ? WHERE id = ?
        let result = package
            .bind_to_update_query(sqlx::query(UPDATE_PACKAGE_SQL))
            .execute(&self.pool)
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
    /// Использует enum PackageFieldUpdate для type-safety.
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
        field_update: crate::types::package::PackageFieldUpdate,
    ) -> Result<(), DataBaseError> {
        // Получаем SQL-запрос из файла (через enum)
        let sql = field_update.sql_query();

        // Биндим значение и имя пакета из структуры
        let result = field_update
            .bind_value(sqlx::query(sql))
            .bind(&package.name)
            .execute(&self.pool)
            .await?;

        // Проверяем, что пакет был найден и обновлён
        if result.rows_affected() == 0 {
            return Err(DataBaseError::PackageNotFound(package.name.clone()));
        }

        Ok(())
    }
}
