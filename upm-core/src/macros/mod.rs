/// Макрос для автоматического биндинга полей структуры к SQL-запросу.
///
/// # Пример использования
/// ```ignore
/// bind_fields!(query, package, [id, name, version, repository, state_of_instalation, description, license])
/// ```
#[macro_export]
macro_rules! bind_fields {
    ($query:expr, $struct:expr, [$($field:ident),+ $(,)?]) => {
        {
            let mut query = $query;
            $(
                query = query.bind(&$struct.$field);
            )+
            query
        }
    };
}

/// Макрос для создания итератора по полям структуры в виде пар (имя, значение).
/// Полезно для динамической работы с полями.
///
/// # Пример
/// ```ignore
/// let fields = fields_iter!(package, {
///     id => &package.id,
///     name => &package.name,
///     version => &package.version,
/// });
/// ```
#[macro_export]
macro_rules! fields_iter {
    ($struct:expr, { $($field_name:ident => $field_value:expr),+ $(,)? }) => {
        vec![
            $(
                (stringify!($field_name), $field_value as &dyn std::fmt::Display),
            )+
        ]
    };
}
