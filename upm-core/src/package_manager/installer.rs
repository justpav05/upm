pub struct Installer {
    database: Arc<DataBase>,
}

pub struct InstallOptions {
    pub overwrite: bool,   // Перезаписывать существующие файлы
    pub keep_config: bool, // Сохранять конфиги при обновлении
    pub run_scripts: bool, // Выполнять pre/post скрипты
    pub backup: bool,      // Создавать бэкапы
}

pub struct InstallResult {
    pub installed_files: Vec<PathBuf>,
    pub created_directories: Vec<PathBuf>,
    pub executed_scripts: Vec<String>,
    pub total_size: u64,
}

impl Installer {
    // Основной метод установки
    pub async fn install(
        &self,
        package: DownloadedPackage,  // От бэкенда
        options: InstallOptions
    ) -> Result<InstallResult, InstallerError>

    // Удаление пакета
    pub async fn remove(
        &self,
        package_name: &str,
        files: Vec<PathBuf>,        // Из БД
        scripts: PackageScripts,    // Из БД
        purge: bool
    ) -> Result<Vec<PathBuf>, InstallerError>

    // Приватные методы:
    fn check_root_permissions() -> Result<()>
    async fn create_directories(files: &[PackageFile]) -> Result<()>
    async fn execute_script(script: &InstallScript) -> Result<()>
    async fn install_files(files: &[PackageFile]) -> Result<()>
    async fn set_permissions(path: &Path, mode: u32) -> Result<()>
    async fn set_owner(path: &Path, uid: u32, gid: u32) -> Result<()>
}
