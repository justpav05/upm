// ============================================================================
// Imports
// ============================================================================

use nix::sys::stat::Mode;
use nix::unistd::{Gid, Uid};
use url::Url;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Package {
    // Main information about package
    uuid: Uuid,
    name: String,
    // Type of package
    package_type: PackageType,

    version: String,
    architecture: String,
    repository: Url,
    backend: Option<BackendType>,
    // Addition information about package
    tags: Option<String>,
    license: Option<String>,
    description: Option<String>,

    download_size: u64,
    installed_size: u64,

    installed_at: Option<chrono::DateTime<chrono::Utc>>,
    updated_at: Option<chrono::DateTime<chrono::Utc>>,

    state_of_instalation: bool,
    dependencies: Vec<Package>,
}

#[derive(Debug, Clone)]
pub enum PackageType {
    Usual,
    Virtual,
    Meta,
    Group,
}

#[derive(Debug, Clone)]
pub enum PackageOperation {
    Install {
        /// Список имён пакетов для установки
        packages: Vec<Package>,
    },

    Remove {
        /// Список имён пакетов для удаления
        packages: Vec<Package>,
        /// Удалять ли конфигурационные файлы (purge)
        purge: bool,
    },

    Update {
        /// Список имён пакетов для обновления
        packages: Vec<Package>,
    },

    Upgrade,

    Search {
        query: String,
    },

    Info {
        /// Имя пакета
        package: Package,
    },

    List {
        /// Тип списка (установленные, доступные, обновляемые)
        filter: ListFilter,
    },

    UpdatePackageListCache,

    CleanPackageListCache,

    CleanPackageCache,

    CreateSnapshot {
        snapshot_uuid: Uuid,
        description: String,
    },

    Rollback {
        snapshot_uuid: Uuid,
    },

    ListSnapshots,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListFilter {
    Installed,
    Available,
    Upgradable,
}

pub struct UncompressedFile {
    source: PathBuf,
    destination: PathBuf,

    permissions: Mode,
    file_type: FileType,

    owner: Uid,
    group: Gid,
}

pub struct PackageScripts {
    pre_install: Option<Vec<PathBuf>>,

    post_install: Option<Vec<PathBuf>>,

    pre_remove: Option<Vec<PathBuf>>,

    post_remove: Option<Vec<PathBuf>>,
}

pub enum FileType {
    File,
    Config,
    Directory,
    ScriptFile,
}
