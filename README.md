# upac

> ⚠️ Проект находится в активной разработке. API нестабильно.

**upac** — универсальный менеджер пакетов для Linux. Позволяет устанавливать, удалять и управлять пакетами различных форматов из единого инструмента, не привязываясь к конкретному дистрибутиву. Опционально интегрируется с [OSTree](https://ostreedev.github.io/ostree/) для атомарных снапшотов состояния системы — это позволяет откатить систему к любому предыдущему состоянию если что-то пошло не так.

---

## Возможности

- Установка и удаление пакетов из различных форматов
- Реестр установленных пакетов и их файлов
- Поддержка нескольких форматов пакетов через систему бэкендов
- Интеграция с OSTree: создание коммитов, откат системы к предыдущему состоянию
- Файловые блокировки для безопасной параллельной работы

---

## Доступность

| Пакетный менеджер | Статус |
|---|---|
| crates.io | 🔜 Скоро |
| AUR (Arch Linux) | 🔜 Скоро |
| APT (Debian/Ubuntu) | 🔜 Скоро |

---

## upac-core-lib

Данный репозиторий содержит **upac-core-lib** — библиотеку на Rust, которая реализует всю основную логику менеджера пакетов. Она предоставляет набор трейтов и структур для построения собственных инструментов поверх upac: CLI, графических интерфейсов или интеграций с другими системами.

### Архитектура

Библиотека разделена на независимые слои, каждый из которых отвечает за свою область:

```
core/         — базовые типы, трейт Backend, утилиты
database/     — реестр пакетов и файлов (PackageRegistry, FileRegistry)
index/        — индекс установленных пакетов (PackageIndex)
backup/       — интеграция с OSTree (PackageRepo)
installer/    — оркестратор установки/удаления (Install)
```

### Ключевые трейты

| Трейт | Описание |
|---|---|
| `Backend` | Формат пакета: извлечение, чтение метаданных |
| `PackageRegistry` | Добавление, удаление, поиск пакетов в БД |
| `FileRegistry` | Регистрация файлов пакета |
| `PackageIndex` | Быстрый индекс установленных пакетов |
| `PackageRepo` | Коммиты, откат, история через OSTree |
| `Install` | Установка и удаление пакетов |

### Использование

**Без OSTree (простая установка)**

```rust
use upac_core_lib::{Installer, Database, Install};

let (tx, _rx) = std::sync::mpsc::channel();

let db = Database::new("/var/lib/upac".into())?;
let mut installer = Installer::new(
    Box::new(db.clone()),  // PackageRegistry
    Box::new(db),          // FileRegistry
    false,                 // ostree_enabled
    "/".into(),            // root_dir
    "/var/lib/upac/packages".into(),
    "/tmp/upac".into(),
    tx,
);

installer.install(&extracted_package)?;
```

**С OSTree**

```rust
use upac_core_lib::{Installer, Database, Install, OStreeRepo, PackageRepo, PackageDiff};

// Открываем или создаём OSTree репозиторий
let ostree = OStreeRepo::open("/var/lib/upac/repo".into())?;

// Устанавливаем пакет
installer.install(&extracted_package)?;

// Собираем список всех файлов для коммита
let packages = installer.list_packages()?;
let mut files = Vec::new();
for pkg in &packages {
    files.extend(installer.list_files(&pkg.name)?);
}

// Создаём коммит
let diff = PackageDiff {
    added: vec![package_name.to_string()],
    removed: vec![],
    updated: vec![],
};
let commit_hash = ostree.commit(files, &diff)?;

// Откат к предыдущему состоянию
ostree.rollback_to(&commit_hash, "/".as_ref())?;
```

### Зависимости

| Крейт | Назначение |
|---|---|
| `ostree` | Интеграция с OSTree |
| `nix` | Unix: права доступа, блокировки |
| `serde` + `toml` | Сериализация базы данных |
| `regex` | Поиск пакетов |
| `tempfile` | Временные директории при установке |

---

## Требования

- Rust 1.83+
- Linux
- [OSTree](https://ostreedev.github.io/ostree/) (опционально, для бэкапов)
