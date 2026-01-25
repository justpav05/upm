//! Example: Basic package installation and removal
//!
//! Demonstrates how to use PackageManager for basic operations.

use std::path::Path;
use std::sync::Arc;

use upm_core::core::database::DataBase;
use upm_core::core::package_manager::{InstallOptions, PackageManager, RemoveOptions};
use upm_core::core::threadcoordination::{ThreadCoordinator, ThreadPoolConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Инициализация логирования
    env_logger::init();

    println!("=== UPM Core Example: Basic Operations ===\n");

    // 1. Создаём ThreadCoordinator
    println!("1. Initializing ThreadCoordinator...");
    let coordinator = Arc::new(ThreadCoordinator::new(ThreadPoolConfig::default()).await?);

    // 2. Создаём DataBase (требует root права на Unix)
    println!("2. Initializing Database...");
    let database =
        Arc::new(DataBase::new(Path::new("/tmp/upm_test"), "packages.db".to_string(), 10).await?);

    // 3. Создаём PackageManager
    println!("3. Creating PackageManager...");
    let manager = PackageManager::new(coordinator, database);

    println!("\n=== Installing Packages ===\n");

    // 4. Устанавливаем пакеты
    let packages_to_install = vec!["nginx", "postgresql", "redis"];
    println!("Installing: {:?}", packages_to_install);

    let install_result = manager
        .install(packages_to_install, InstallOptions::default())
        .await?;

    println!("Operation ID: {}", install_result.operation_id);
    println!("Status: {:?}\n", install_result.status);

    // 5. Проверяем список установленных пакетов
    println!("=== Listing Installed Packages ===\n");
    let installed = manager.list_installed().await?;
    println!("Installed packages: {}", installed.len());
    for pkg in &installed {
        println!("  - {} v{} ({})", pkg.name, pkg.version, pkg.repository);
    }

    // 6. Проверяем статус конкретного пакета
    println!("\n=== Checking Package Status ===\n");
    let is_nginx_installed = manager.is_package_installed("nginx").await?;
    println!("Is nginx installed? {}", is_nginx_installed);

    if let Some(pkg) = manager.get_package_info("nginx").await? {
        println!("nginx info: {:?}", pkg);
    }

    // 7. Удаляем пакет
    println!("\n=== Removing Package ===\n");
    let remove_result = manager
        .remove(vec!["redis"], RemoveOptions::default())
        .await?;

    println!("Remove operation ID: {}", remove_result.operation_id);
    println!("Status: {:?}\n", remove_result.status);

    // 8. Проверяем снова
    let installed_after = manager.list_installed().await?;
    println!(
        "Installed packages after removal: {}",
        installed_after.len()
    );
    for pkg in &installed_after {
        println!("  - {} v{}", pkg.name, pkg.version);
    }

    // 9. Полное удаление с purge
    println!("\n=== Purging Package ===\n");
    let purge_result = manager
        .remove(
            vec!["nginx"],
            RemoveOptions {
                purge: true,
                remove_dependencies: false,
            },
        )
        .await?;

    println!("Purge operation ID: {}", purge_result.operation_id);
    println!("Status: {:?}\n", purge_result.status);

    println!("=== Example Completed ===");

    Ok(())
}
