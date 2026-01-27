//! Все enum'ы проекта UPM Core
//!
//! Этот файл содержит все перечисления (enum), используемые в проекте,
//! организованные по функциональным областям с чётким разделением.

// ============================================================================
// Database Errors - Ошибки при работе с базой данных
// ============================================================================

use thiserror::Error;

/// Ошибки при работе с базой данных
///
/// Охватывает все возможные ошибки при работе с SQLite базой данных:
/// - Проблемы с доступом к файлам и директориям
/// - Ошибки валидации и прав доступа
/// - Проблемы с подключением и таймауты
/// - Нарушения ограничений (constraints)
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

    #[error("Ошибка вхождения: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Неверное расширение файла базы данных для '{0}', должно быть '.db'")]
    InvalidDatabaseExtension(String),

    #[error("Неверные права доступа для файла базы данных '{0}'. Требуются права 600.")]
    IncorrectFilePermissions(String),

    #[error("Некорректный SQL-файл: {0}")]
    InvalidSqlFile(String),

    #[error("Валидация базы данных не прошла успешно: {0}")]
    DatabaseValidationError(String),

    #[error("Такой пакет уже существует: {0}")]
    UniqueConstraintViolated(String),

    #[error("Такой ключ уже существует: {0}")]
    ForeignKeyConstraintViolated(String),

    #[error("База данных не отвечает")]
    DatabaseTimeout,

    #[error("Ошибка подключения к базе данных: {0}")]
    ConnectionError(String),

    #[error("Пакет не найден: {0}")]
    PackageNotFound(String),

    #[error("Неизвестная ошибка базы данных")]
    UnknownDatabaseError,
}

// ============================================================================
// Package Errors - Ошибки при работе с пакетами
// ============================================================================

/// Ошибки при работе с пакетами
///
/// Общие ошибки операций с пакетами:
/// - Пакет не найден
/// - Ошибки ввода-вывода
/// - Проблемы с конфигурацией
/// - Неудачные операции установки/удаления
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

// ============================================================================
// Package Field Update - Type-safe обновление полей пакета
// ============================================================================

/// Enum для type-safe обновления отдельных полей пакета
///
/// Позволяет безопасно обновлять конкретное поле пакета в базе данных
/// без риска перепутать поля. Каждый вариант содержит значение нужного типа
/// и знает свой SQL-запрос для обновления.
///
/// # Примеры
/// ```ignore
/// let update = PackageFieldUpdate::Version("1.2.3".to_string());
/// database.update_package_field("nginx", update).await?;
/// ```
#[derive(Debug, Clone)]
pub enum PackageFieldUpdate {
    /// Обновить имя пакета
    Name(String),

    /// Обновить версию пакета
    Version(String),

    /// Обновить репозиторий пакета
    Repository(String),

    /// Обновить статус установки (true = установлен, false = не установлен)
    Installed(bool),

    /// Обновить описание пакета
    Description(Option<String>),

    /// Обновить лицензию пакета
    License(Option<String>),
}

// ============================================================================
// PackageFieldUpdate - Методы
// ============================================================================

impl PackageFieldUpdate {
    /// Возвращает SQL-запрос для обновления конкретного поля из SQL-файла
    ///
    /// Каждое поле имеет свой отдельный SQL-файл с запросом UPDATE.
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

    /// Привязывает значение к SQL-запросу
    ///
    /// Используется для безопасной подстановки значений в prepared statements.
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

// ============================================================================
// From/Into Implementations - Конвертация из sqlx ошибок
// ============================================================================

impl From<sqlx::Error> for DataBaseError {
    fn from(error: sqlx::Error) -> Self {
        match error {
            sqlx::Error::PoolTimedOut => DataBaseError::DatabaseTimeout,
            sqlx::Error::Io(io_err) => DataBaseError::IoError(io_err),
            sqlx::Error::Database(db_err) => {
                let sqlite_err = db_err.downcast_ref::<sqlx::sqlite::SqliteError>();
                sqlite_err.into()
            }
            sqlx::Error::Configuration(config_err) => {
                DataBaseError::ConnectionError(config_err.to_string())
            }
            sqlx::Error::Tls(tls_err) => DataBaseError::ConnectionError(tls_err.to_string()),
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

// ============================================================================
// Package Manager Operations - Enum'ы для операций с пакетами
// ============================================================================

/// Стратегия разрешения зависимостей
///
/// Определяет алгоритм, используемый для разрешения зависимостей пакетов
/// при установке. Разные стратегии имеют разные компромиссы между
/// скоростью работы и качеством решения.
///
/// # Варианты
///
/// ## SAT Solver
/// Использует SAT (Boolean Satisfiability) решатель для поиска оптимального
/// решения зависимостей. Гарантирует нахождение корректного решения если оно
/// существует, но может работать медленнее на больших графах зависимостей.
///
/// **Преимущества:**
/// - Находит оптимальное решение
/// - Гарантированно корректное разрешение
/// - Обнаруживает невозможность разрешения
///
/// **Недостатки:**
/// - Медленнее на больших графах
/// - Требует больше памяти
///
/// ## Greedy Algorithm
/// Использует жадный алгоритм для быстрого разрешения зависимостей.
/// Работает быстро, но может не найти оптимальное решение или вообще
/// не найти решение даже если оно существует.
///
/// **Преимущества:**
/// - Быстрая работа
/// - Малое потребление памяти
/// - Подходит для простых случаев
///
/// **Недостатки:**
/// - Может не найти оптимальное решение
/// - Может не найти решение вообще
///
/// # Рекомендации
/// - Используйте **SAT** для сложных зависимостей и важных установок
/// - Используйте **Greedy** для простых пакетов и быстрых операций
///
/// # Примеры
/// ```ignore
/// let options = InstallOptions {
///     strategy: DependencyStrategy::Sat,  // Точное разрешение
///     ..Default::default()
/// };
///
/// manager.install(vec!["complex-package"], options).await?;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DependencyStrategy {
    /// SAT solver (медленнее, но точнее и гарантирует корректность)
    Sat,

    /// Greedy algorithm (быстрее, но может быть неоптимально)
    Greedy,
}

/// Статус выполнения операции установки/удаления пакетов
///
/// Отслеживает текущее состояние операции с пакетами, включая прогресс,
/// ошибки и результаты выполнения. Используется для:
/// - Отображения прогресса в TUI/GUI
/// - Логирования операций
/// - Отслеживания асинхронных операций
/// - Диагностики проблем
///
/// # Жизненный цикл операции
///
/// ```text
/// Pending → Running → Completed (успех)
///                  └→ Failed (ошибка)
/// ```
///
/// # Варианты
///
/// ## Pending
/// Операция создана но ещё не началась. Находится в очереди на выполнение.
///
/// ## Running
/// Операция выполняется в данный момент. Содержит информацию о прогрессе
/// и текущем обрабатываемом пакете.
///
/// ## Completed
/// Операция завершена. Может быть частично успешной (некоторые пакеты
/// установились, некоторые нет) или полностью успешной.
///
/// ## Failed
/// Операция полностью провалилась. Содержит описание ошибки.
///
/// # Примеры
/// ```ignore
/// use upm_core::types::enums::OperationStatus;
///
/// // Проверка статуса операции
/// match operation.status {
///     OperationStatus::Running { progress, current_package } => {
///         println!("Installing... {}%", progress);
///         if let Some(pkg) = current_package {
///             println!("Current package: {}", pkg);
///         }
///     }
///     OperationStatus::Completed { installed, failed } => {
///         println!("Done! Installed: {}, Failed: {}", installed, failed);
///     }
///     OperationStatus::Failed { error } => {
///         eprintln!("Error: {}", error);
///     }
///     OperationStatus::Pending => {
///         println!("Waiting in queue...");
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub enum OperationStatus {
    /// Операция ожидает выполнения
    ///
    /// Операция создана и помещена в очередь, но ещё не началась.
    /// Обычно означает что другие операции выполняются в данный момент
    /// или система ещё не готова к началу выполнения.
    Pending,

    /// Операция выполняется
    ///
    /// Операция активно выполняется в данный момент.
    /// Содержит информацию о прогрессе и текущем состоянии.
    Running {
        /// Прогресс выполнения (0-100)
        ///
        /// Процент выполненной работы. 0 = только началась, 100 = почти готово.
        progress: u8,

        /// Текущий обрабатываемый пакет
        ///
        /// Имя пакета который устанавливается/удаляется в данный момент.
        /// None если операция между пакетами или выполняет общие действия.
        current_package: Option<String>,
    },

    /// Операция завершена успешно (полностью или частично)
    ///
    /// Операция закончила работу. Может быть:
    /// - Полностью успешной (failed = 0)
    /// - Частично успешной (failed > 0, но installed > 0)
    Completed {
        /// Количество успешно обработанных пакетов
        ///
        /// Число пакетов которые были успешно установлены/удалены.
        installed: usize,

        /// Количество неудачных пакетов
        ///
        /// Число пакетов для которых операция не удалась.
        /// Если 0 - операция полностью успешна.
        failed: usize,
    },

    /// Операция провалилась
    ///
    /// Операция завершилась с критической ошибкой.
    /// Обычно означает что ни один пакет не был обработан.
    Failed {
        /// Описание ошибки
        ///
        /// Человекочитаемое описание причины провала операции.
        error: String,
    },
}

// ============================================================================
// Package Operations - Операции пакетного менеджера
// ============================================================================

/// Тип операции, выполняемой пакетным менеджером
///
/// Определяет все возможные операции, которые может выполнять PackageManager.
/// Используется для:
/// - Логирования операций
/// - Отслеживания истории действий
/// - Определения типа текущей операции
/// - Формирования отчётов
///
/// # Категории операций
///
/// ## Операции с пакетами
/// - Install, Remove, Update, Upgrade - изменяют систему
///
/// ## Информационные операции
/// - Search, Info, List - только чтение, не изменяют систему
///
/// ## Операции с кэшем
/// - UpdateCache, CleanCache - работа с локальным кэшем репозиториев
///
/// ## Операции со снапшотами
/// - CreateSnapshot, Rollback, ListSnapshots - работа с OSTree
///
/// # Примеры
/// ```ignore
/// use upm_core::types::enums::PackageOperation;
///
/// let operation = PackageOperation::Install {
///     packages: vec!["nginx".to_string(), "postgresql".to_string()],
/// };
///
/// match operation {
///     PackageOperation::Install { packages } => {
///         println!("Installing {} packages", packages.len());
///     }
///     PackageOperation::Remove { packages, purge } => {
///         println!("Removing packages (purge: {})", purge);
///     }
///     _ => {}
/// }
/// ```
#[derive(Debug, Clone)]
pub enum PackageOperation {
    /// Установка одного или нескольких пакетов
    ///
    /// Устанавливает указанные пакеты вместе с их зависимостями.
    /// Если пакет уже установлен, операция может быть пропущена
    /// или выполнена переустановка в зависимости от опций.
    Install {
        /// Список имён пакетов для установки
        packages: Vec<String>,
    },

    /// Удаление одного или нескольких пакетов
    ///
    /// Удаляет указанные пакеты из системы. Может также удалить
    /// конфигурационные файлы если указан флаг purge.
    Remove {
        /// Список имён пакетов для удаления
        packages: Vec<String>,
        /// Удалять ли конфигурационные файлы (purge)
        purge: bool,
    },

    /// Обновление конкретных пакетов до новых версий
    ///
    /// Обновляет указанные пакеты до последних доступных версий
    /// из репозиториев.
    Update {
        /// Список имён пакетов для обновления
        packages: Vec<String>,
    },

    /// Обновление всех установленных пакетов
    ///
    /// Проверяет все установленные пакеты и обновляет те,
    /// для которых доступны новые версии.
    Upgrade,

    /// Поиск пакетов по имени или описанию
    ///
    /// Ищет пакеты в репозиториях по заданному запросу.
    /// Возвращает список найденных пакетов с метаданными.
    Search {
        /// Поисковый запрос (имя, часть имени, или ключевые слова)
        query: String,
    },

    /// Получение подробной информации о пакете
    ///
    /// Возвращает полную информацию о пакете: версию, зависимости,
    /// описание, размер и т.д.
    Info {
        /// Имя пакета
        package: String,
    },

    /// Получение списка пакетов
    ///
    /// Возвращает список пакетов согласно указанному фильтру.
    List {
        /// Тип списка (установленные, доступные, обновляемые)
        filter: ListFilter,
    },

    /// Обновление кэша репозиториев
    ///
    /// Скачивает актуальные списки пакетов из всех настроенных
    /// репозиториев. Необходимо выполнять периодически для получения
    /// информации о новых пакетах и обновлениях.
    UpdateCache,

    /// Очистка кэша загруженных пакетов
    ///
    /// Удаляет скачанные файлы пакетов из локального кэша для
    /// освобождения дискового пространства.
    CleanCache,

    /// Создание снапшота системы
    ///
    /// Создаёт точку восстановления системы через OSTree.
    /// Позволяет откатиться к текущему состоянию в случае проблем.
    CreateSnapshot {
        /// Описание снапшота
        description: String,
    },

    /// Откат к предыдущему снапшоту
    ///
    /// Восстанавливает систему до состояния, зафиксированного
    /// в указанном снапшоте.
    Rollback {
        /// ID снапшота для отката
        snapshot_id: String,
    },

    /// Получение списка всех снапшотов
    ///
    /// Возвращает список всех доступных точек восстановления системы.
    ListSnapshots,
}

/// Фильтр для операции List
///
/// Определяет какой список пакетов нужно получить.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListFilter {
    /// Только установленные пакеты
    Installed,

    /// Все доступные пакеты в репозиториях
    Available,

    /// Пакеты, для которых доступны обновления
    Upgradable,
}
