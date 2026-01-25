use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Default)]
pub struct Package {
    pub id: String,
    pub name: String,
    pub version: String,
    pub repository: String,
    #[sqlx(rename = "installed")]
    pub state_of_instalation: bool,
    pub description: Option<String>,
    pub license: Option<String>,
}

/// Enum для type-safe обновления отдельных полей пакета.
#[derive(Debug, Clone)]
pub enum PackageFieldUpdate {
    Name(String),
    Version(String),
    Repository(String),
    Installed(bool),
    Description(Option<String>),
    License(Option<String>),
}

impl PackageFieldUpdate {
    /// Возвращает SQL-запрос для обновления конкретного поля из SQL-файла
    pub fn sql_query(&self) -> &'static str {
        match self {
            Self::Name(_) => include_str!("../sql/queries/update_package_name.sql"),
            Self::Version(_) => include_str!("../sql/queries/update_package_version.sql"),
            Self::Repository(_) => include_str!("../sql/queries/update_package_repository.sql"),
            Self::Installed(_) => include_str!("../sql/queries/update_package_installed.sql"),
            Self::Description(_) => include_str!("../sql/queries/update_package_description.sql"),
            Self::License(_) => include_str!("../sql/queries/update_package_license.sql"),
        }
    }

    pub fn bind_value<'q>(
        &'q self,
        query: sqlx::query::Query<'q, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'q>>,
    ) -> sqlx::query::Query<'q, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'q>> {
        match self {
            Self::Name(v) => query.bind(v),
            Self::Version(v) => query.bind(v),
            Self::Repository(v) => query.bind(v),
            Self::Installed(v) => query.bind(v),
            Self::Description(v) => query.bind(v),
            Self::License(v) => query.bind(v),
        }
    }
}

impl Package {
    /// Очищает все поля пакета
    pub fn clear(&mut self) {
        self.id.clear();
        self.name.clear();
        self.version.clear();
        self.repository.clear();
        self.state_of_instalation = false;
        self.description = None;
        self.license = None;
    }

    /// Сбрасывает пакет к значениям по умолчанию
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Создаёт новый пустой пакет
    pub fn new() -> Self {
        Self::default()
    }
}

// Автоматическая реализация трейта BindableFields для Package
crate::impl_bindable_fields!(Package, [
    id: String,
    name: String,
    version: String,
    repository: String,
    state_of_instalation: bool,
    description: Option<String>,
    license: Option<String>,
]);
