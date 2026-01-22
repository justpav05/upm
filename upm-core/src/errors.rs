use sqlx;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
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
enum DbError {
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

    #[error("Ошибка базы данных: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("Неверное расширение файла базы данных для '{0}', должно быть '.db'")]
    InvalidDatabaseExtension(String),

    #[error("Неверные права доступа для файла базы данных '{0}'. Требуются права 600.")]
    IncorrectFilePermissions(String),
}
