use sqlx::{self, error::DatabaseError};
use thiserror::Error;

//pub type Result<T> = std::result::Result<T, DataBaseError>;

#[derive(Debug, Error)]
pub enum PackageError {
    #[error("Package not found: {0}")]
    PackageNotFound(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Operation failed: {0}")]
    OperationFailed(String),
}

#[derive(Debug, Error)]
pub enum DataBaseError {
    #[error("Путь недоступен: {0}")]
    PathNotAccessible(String),

    #[error("База данных '{0}' уже существует")]
    DatabaseExists(String),

    #[error("Невозможность создать папку: {0}")]
    CannotCreateDirectory(std::io::Error),

    #[error("Недостаточно прав. Текущий UID: {0}. Требуется root (UID 0)")]
    InvalidPermissions(u32),

    #[error("Путь это файл: {0}")]
    PathIsFile(String),

    #[error("База данных повреждена: {0}")]
    DatabaseCorrupted(String),

    #[error("Ощибка вхождения: {0}")]
    IoError(std::io::Error),

    #[error("Неверное расширение файла базы данных для '{0}', должно быть '.db'")]
    InvalidDatabaseExtension(String),

    #[error("Неверные права доступа для файла базы данных '{0}'. Требуются права 600.")]
    IncorrectFilePermissions(String),

    #[error("Некорректный SQL-файл: {0}")]
    InvalidSqlFile(String),

    #[error("Валидация базы данных не прошла успншно: {0}")]
    DatabaseValidationError(String),

    #[error("Такой пакет уже существует: {0}")]
    UniqueConstraintViolated(String),

    #[error("Такой ключ уже существует: {0}")]
    ForeignKeyConstraintViolated(String),

    #[error("База данных не отвечает")]
    DatabaseTimeout,

    #[error("Неизвестная ошибка базы данных")]
    UnknownDatabaseError,
}

impl From<sqlx::Error> for DataBaseError {
    fn from(error: sqlx::Error) -> Self {
        match error {
            sqlx::Error::PoolTimedOut => DataBaseError::DatabaseTimeout,
            sqlx::Error::Io(io_err) => DataBaseError::IoError(io_err),
            sqlx::Error::Database(db_err) => {
                let sqlite_err = db_err.downcast_ref::<sqlx::sqlite::SqliteError>();
                sqlite_err.into()
            }
            _ => DataBaseError::UnknownDatabaseError,
        }
    }
}

impl From<&sqlx::sqlite::SqliteError> for DataBaseError {
    fn from(sqlite_err: &sqlx::sqlite::SqliteError) -> Self {
        use sqlx::error::DatabaseError as SqlxDatabaseError;

        let message = sqlite_err.message().to_string();
        let code = sqlite_err
            .code()
            .and_then(|code| code.parse::<u32>().ok())
            .unwrap_or(0);

        match code {
            1555 | 2067 => DataBaseError::UniqueConstraintViolated(message),
            787 => DataBaseError::ForeignKeyConstraintViolated(message),
            _ => DataBaseError::UnknownDatabaseError,
        }
    }
}
