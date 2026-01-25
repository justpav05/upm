use sqlx::Sqlite;

/// Трейт для структур, поля которых можно автоматически биндить к SQL-запросам.
///
/// Этот трейт предоставляет методы для разных типов SQL-операций:
/// - INSERT: биндит все поля в порядке структуры
/// - UPDATE: биндит все поля кроме id, затем id для WHERE clause
pub trait BindableFields {
    /// Возвращает список имён полей в том порядке, в котором они определены в структуре.
    fn field_names() -> &'static [&'static str];

    /// Биндит все поля структуры к SQL-запросу INSERT.
    ///
    /// Порядок: все поля в порядке определения в структуре.
    ///
    /// # Пример SQL
    /// ```sql
    /// INSERT INTO packages (id, name, version, repository, installed, description, license)
    /// VALUES (?, ?, ?, ?, ?, ?, ?)
    /// ```
    fn bind_to_insert_query<'q>(
        &'q self,
        query: sqlx::query::Query<'q, Sqlite, sqlx::sqlite::SqliteArguments<'q>>,
    ) -> sqlx::query::Query<'q, Sqlite, sqlx::sqlite::SqliteArguments<'q>>;

    /// Биндит поля структуры к SQL-запросу UPDATE.
    ///
    /// Порядок: все поля кроме id (для SET), затем id (для WHERE).
    ///
    /// # Пример SQL
    /// ```sql
    /// UPDATE packages
    /// SET name = ?, version = ?, repository = ?, installed = ?, description = ?, license = ?
    /// WHERE id = ?
    /// ```
    fn bind_to_update_query<'q>(
        &'q self,
        query: sqlx::query::Query<'q, Sqlite, sqlx::sqlite::SqliteArguments<'q>>,
    ) -> sqlx::query::Query<'q, Sqlite, sqlx::sqlite::SqliteArguments<'q>>;
}

/// Макрос для автоматической реализации трейта BindableFields.
///
/// # Важно!
/// Первое поле ДОЛЖНО быть `id` - оно используется как первичный ключ для UPDATE.
///
/// # Пример использования
/// ```ignore
/// impl_bindable_fields!(Package, [
///     id: String,                    // Обязательно первым!
///     name: String,
///     version: String,
///     repository: String,
///     state_of_instalation: bool,
///     description: Option<String>,
///     license: Option<String>,
/// ]);
/// ```
///
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
