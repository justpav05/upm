//! Все трейты проекта UPM Core
//!
//! Этот файл содержит все трейты (traits), используемые в проекте,
//! организованные по функциональным областям с чётким разделением.

use sqlx::Sqlite;

// ============================================================================
// Database Traits - Трейты для работы с базой данных
// ============================================================================

/// Трейт для структур, поля которых можно автоматически биндить к SQL-запросам
///
/// Этот трейт предоставляет методы для разных типов SQL-операций:
/// - **INSERT**: биндит все поля в порядке структуры
/// - **UPDATE**: биндит все поля кроме id, затем id для WHERE clause
///
/// # Назначение
/// Упрощает работу с SQL-запросами, автоматически привязывая поля структуры
/// к параметрам prepared statements. Это обеспечивает:
/// - Безопасность от SQL-инъекций
/// - Автоматическую типизацию
/// - Меньше boilerplate кода
///
/// # Реализация
/// Трейт реализуется автоматически через макрос `impl_bindable_fields!`,
/// который генерирует код для биндинга полей на основе структуры.
///
/// # Примеры
/// ```ignore
/// use upm_core::types::traits::BindableFields;
///
/// // Автоматическая реализация через макрос
/// impl_bindable_fields!(Package, [
///     id: String,
///     name: String,
///     version: String,
/// ]);
///
/// // Использование
/// let package = Package { id: "pkg-1".to_string(), ... };
/// let query = sqlx::query("INSERT INTO packages VALUES (?, ?, ?)");
/// let bound_query = package.bind_to_insert_query(query);
/// ```
pub trait BindableFields {
    /// Возвращает список имён полей в том порядке, в котором они определены в структуре
    ///
    /// Используется для отладки и генерации динамических SQL-запросов.
    ///
    /// # Возвращает
    /// Статический массив строк с именами полей.
    fn field_names() -> &'static [&'static str];

    /// Биндит все поля структуры к SQL-запросу INSERT
    ///
    /// Порядок: все поля в порядке определения в структуре (id первым).
    ///
    /// # Аргументы
    /// * `query` - Prepared statement для INSERT запроса
    ///
    /// # Возвращает
    /// Query с привязанными значениями полей.
    ///
    /// # Пример SQL
    /// ```sql
    /// INSERT INTO packages (id, name, version, repository, installed, description, license)
    /// VALUES (?, ?, ?, ?, ?, ?, ?)
    /// ```
    ///
    /// # Примеры
    /// ```ignore
    /// let package = Package::new();
    /// let query = sqlx::query(INSERT_SQL);
    /// let bound = package.bind_to_insert_query(query);
    /// bound.execute(&pool).await?;
    /// ```
    fn bind_to_insert_query<'q>(
        &'q self,
        query: sqlx::query::Query<'q, Sqlite, sqlx::sqlite::SqliteArguments<'q>>,
    ) -> sqlx::query::Query<'q, Sqlite, sqlx::sqlite::SqliteArguments<'q>>;

    /// Биндит поля структуры к SQL-запросу UPDATE
    ///
    /// Порядок: все поля кроме id (для SET clause), затем id (для WHERE clause).
    ///
    /// # Аргументы
    /// * `query` - Prepared statement для UPDATE запроса
    ///
    /// # Возвращает
    /// Query с привязанными значениями полей.
    ///
    /// # Пример SQL
    /// ```sql
    /// UPDATE packages
    /// SET name = ?, version = ?, repository = ?, installed = ?, description = ?, license = ?
    /// WHERE id = ?
    /// ```
    ///
    /// # Примеры
    /// ```ignore
    /// let package = Package { id: "pkg-1".to_string(), name: "nginx".to_string(), ... };
    /// let query = sqlx::query(UPDATE_SQL);
    /// let bound = package.bind_to_update_query(query);
    /// bound.execute(&pool).await?;
    /// ```
    fn bind_to_update_query<'q>(
        &'q self,
        query: sqlx::query::Query<'q, Sqlite, sqlx::sqlite::SqliteArguments<'q>>,
    ) -> sqlx::query::Query<'q, Sqlite, sqlx::sqlite::SqliteArguments<'q>>;
}

// ============================================================================
// Macros - Макросы для автоматической реализации трейтов
// ============================================================================

/// Макрос для автоматической реализации трейта `BindableFields`
///
/// Генерирует имплементацию трейта для структуры, автоматически создавая
/// методы биндинга полей к SQL-запросам на основе списка полей.
///
/// # Требования
/// - **Первое поле ОБЯЗАТЕЛЬНО должно быть `id`** - используется как первичный ключ
/// - Все поля должны реализовывать `sqlx::Encode` и `sqlx::Type`
/// - Порядок полей должен соответствовать порядку столбцов в БД
///
/// # Синтаксис
/// ```ignore
/// impl_bindable_fields!(ИмяСтруктуры, [
///     id: ТипId,           // Обязательно первым!
///     поле1: Тип1,
///     поле2: Тип2,
///     // ...
/// ]);
/// ```
///
/// # Аргументы
/// * `$struct_name` - Имя структуры для которой генерируется impl
/// * `id: $id_type` - Тип поля id (обязательно первым)
/// * `$field: $field_type` - Список остальных полей и их типов
///
/// # Примеры
/// ```ignore
/// use upm_core::impl_bindable_fields;
/// use upm_core::types::traits::BindableFields;
///
/// #[derive(Debug)]
/// struct Package {
///     id: String,
///     name: String,
///     version: String,
///     repository: String,
///     state_of_instalation: bool,
///     description: Option<String>,
///     license: Option<String>,
/// }
///
/// // Автоматическая реализация трейта
/// impl_bindable_fields!(Package, [
///     id: String,                    // Обязательно первым!
///     name: String,
///     version: String,
///     repository: String,
///     state_of_instalation: bool,
///     description: Option<String>,
///     license: Option<String>,
/// ]);
///
/// // Теперь можно использовать методы трейта
/// let package = Package { /* ... */ };
/// let fields = Package::field_names(); // ["id", "name", "version", ...]
/// ```
///
/// # Что генерируется
/// Макрос создаёт impl блок с тремя методами:
/// 1. `field_names()` - возвращает список имён полей
/// 2. `bind_to_insert_query()` - биндит все поля по порядку
/// 3. `bind_to_update_query()` - биндит поля для UPDATE (без id, потом id)
///
/// # Ограничения
/// - Работает только со структурами с именованными полями
/// - Первое поле должно называться `id`
/// - Не поддерживает кортежные структуры
/// - Не поддерживает generic типы без дополнительных bounds
#[macro_export]
macro_rules! impl_bindable_fields {
    // Паттерн: первое поле это id, остальные поля
    ($struct_name:ty, [id: $id_type:ty, $($field:ident: $field_type:ty),+ $(,)?]) => {
        impl $crate::types::traits::BindableFields for $struct_name {
            fn field_names() -> &'static [&'static str] {
                &["id", $(stringify!($field)),+]
            }

            // Для INSERT: биндим все поля по порядку (id первым)
            fn bind_to_insert_query<'q>(
                &'q self,
                query: sqlx::query::Query<'q, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'q>>,
            ) -> sqlx::query::Query<'q, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'q>> {
                query
                    .bind(&self.id)
                    $(.bind(&self.$field))+
            }

            // Для UPDATE: биндим все поля кроме id, потом id для WHERE
            fn bind_to_update_query<'q>(
                &'q self,
                query: sqlx::query::Query<'q, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'q>>,
            ) -> sqlx::query::Query<'q, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'q>> {
                query
                    $(.bind(&self.$field))+  // Все поля кроме id
                    .bind(&self.id)          // id для WHERE clause
            }
        }
    };
}
