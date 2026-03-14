use crate::app::{AppResult, AppError};
use crate::{InstallOptions, RemoveOptions, SearchOptions, UpdateOptions, UpgradeOptions};

use upac_core_lib::{Backend, Database, Install, Installer, InstallerError, OStreeRepo, PackageDiff, PackageRegistry, PackageRepo, UpacConfig};

pub(crate) fn install(
    options: InstallOptions,
) -> impl FnOnce(
    &mut Installer,
    Option<&OStreeRepo>,
    &UpacConfig,
    &Database,
    &[Box<dyn Backend>],
) -> AppResult<()> {
    move |installer, ostree, config, database, backends| {
    	// Проверка существования пути пакета
        if !&options.package.exists() {
            return Err(AppError::CommandError(format!("File not found: {}", &options.package.display())));
        }

        // Выбор бекенда для пакета
        let backend = backends.iter().find(|backend| backend.detect(&options.package)).ok_or_else(|| AppError::CommandError(format!("Unsupported package format: {}", &options.package.display())))?;

        // Извлекаем пакет во временную директорию
        let extracted_package = backend.extract(&options.package, &config.temp_dir)?;

        // Устанавливаем
        installer.install(&extracted_package)?;

        // Если ostree включён — делаем коммит
        if config.ostree.enabled {
            let packages = installer.list_packages()?;

            let mut packages_files = Vec::new();
            for package in &packages {
                packages_files.extend(installer.list_files(&package.name)?);
            }

            let diff = PackageDiff {
                added:   vec![extracted_package.name.clone()],
                removed: vec![],
                updated: vec![],
            };

            ostree.ok_or_else(|| AppError::CommandError(String::from("OSTree not available")))?.commit(packages_files, &diff)?;
        }

        Ok(())
    }
}

pub(crate) fn remove(
    options: RemoveOptions,
) -> impl FnOnce(
    &mut Installer,
    Option<&OStreeRepo>,
    &UpacConfig,
    &Database,
    &[Box<dyn Backend>],
) -> AppResult<()> {
    move |installer, ostree, config, database, backends| {
        let package = database.get_package(&options.package).map_err(|err| AppError::CommandError(err.to_string()))?.ok_or_else(|| AppError::CommandError(format!("Package not found: {}", options.package)))?;

        installer.remove(&package.name)?;

        if config.ostree.enabled {
            let packages = installer.list_packages()?;

            let mut packages_files = Vec::new();
            for package in &packages {
                packages_files.extend(installer.list_files(&package.name)?);
            }

            let diff = PackageDiff {
                added:   vec![],
                removed: vec![options.package.clone()],
                updated: vec![],
            };

            ostree.ok_or_else(|| AppError::CommandError(String::from("OSTree not available")))?.commit(packages_files, &diff)?;
        }

        Ok(())
    }
}

pub(crate) fn update(
    options: UpdateOptions,
) -> impl FnOnce(
    &mut Installer,
    Option<&OStreeRepo>,
    &UpacConfig,
    &Database,
    &[Box<dyn Backend>],
) -> AppResult<()> {
    move |installer, ostree, config, database, backends| {
        // Проверяем что файл существует
        if !&options.package.exists() {
            return Err(AppError::CommandError(format!("File not found: {}", options.package.display())));
        }

        // Ищем подходящий бэкенд
        let backend = backends.iter().find(|backend| backend.detect(&options.package)).ok_or_else(|| AppError::CommandError(format!("Unsupported package format: {}", &options.package.display())))?;

        // Читаем метаданные нового пакета
        let extracted_package = backend.extract(&options.package, &config.temp_dir)?;

        // Проверяем что пакет вообще установлен
        let current_package = database.get_package(&extracted_package.name)?.ok_or_else(|| AppError::CommandError(format!("Package not installed: {}", extracted_package.name)))?;

        // Проверяем что новая версия отличается от текущей
        if current_package.version == extracted_package.version && !options.force {
            println!("Package {} is already at version {}", extracted_package.name, extracted_package.version);
            return Ok(());
        }

        installer.remove(&extracted_package.name)?;

        installer.install(&extracted_package)?;

        if config.ostree.enabled {
            let packages = installer.list_packages()?;

            let mut packages_files = Vec::new();
            for package in &packages {
                packages_files.extend(installer.list_files(&package.name)?);
            }

            let diff = PackageDiff {
                added:   vec![],
                removed: vec![],
                updated: vec![extracted.name.clone()],
            };

            ostree.ok_or_else(|| AppError::CommandError(String::from("OSTree not available")))?.commit(packages_files, &diff)?;
        }

        Ok(())
    }
}

pub(crate) fn upgrade(
    options: UpgradeOptions,
) -> impl FnOnce(
    &mut Installer,
    Option<&OStreeRepo>,
    &UpacConfig,
    &[Box<dyn Backend>],
) -> AppResult<()> {
    move |installer, _, _, _| {
        if options.check_only {
            let packages = installer.list_packages()?;

            if packages.is_empty() {
                println!("No packages installed.");
                return Ok(());
            }

            println!("Installed packages:");
            for package in &packages {
                println!("  {} ({})", package.name, package.version);
            }

            println!("\nRepository support is not yet implemented.");
            println!("Cannot check for updates without a configured repository.");

            return Ok(());
        }

        Err(AppError::CommandError(
            "Upgrade requires repository support which is not yet implemented.".to_string()
        ))
    }
}

pub(crate) fn search(
    options: SearchOptions,
) -> impl FnOnce(
    &mut Installer,
    Option<&OStreeRepo>,
    &UpacConfig,
    &Database,
    &[Box<dyn Backend>],
) -> AppResult<()> {
    move |_, _, _, database, _| {
        let mut results: Vec<_> = database.list_all_packages().iter().filter(|package_info| {
            if options.exact {
                package_info.name == options.query
            } else {
            	package_info.name.to_lowercase().contains(&options.query.to_lowercase())
            }
        }).collect();

        if results.is_empty() {
            println!("No packages found for: {}", options.query);
            return Ok(());
        }

        if let Some(limit) = options.limit {
            results.truncate(limit as usize);
        }

        for pkg in results {
            println!("{} ({})", pkg.name, pkg.version);
        }

        Ok(())
    }
}

pub(crate) fn show(
	package: &str,
) -> impl FnOnce(
    &mut Installer,
    Option<&OStreeRepo>,
    &UpacConfig,
    &Database,
    &[Box<dyn Backend>],
) -> AppResult<()> {
    move |installer, ostree, config, database, backends| {
        todo!()
    }
}

pub(crate) fn files(
	package: &str,
) -> impl FnOnce(
    &mut Installer,
    Option<&OStreeRepo>,
    &UpacConfig,
    &Database,
    &[Box<dyn Backend>],
) -> AppResult<()> {
    move |installer, ostree, config, database, backends| {
        todo!()
    }
}

pub(crate) fn deps(
	package: &str,
) -> impl FnOnce(
    &mut Installer,
    Option<&OStreeRepo>,
    &UpacConfig,
    &Database,
    &[Box<dyn Backend>],
) -> AppResult<()> {
    move |installer, ostree, config, database, backends| {
        todo!()
    }
}
