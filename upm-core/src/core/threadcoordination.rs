// This suppresses all the unused crate warnings.
#![allow(unused)]
use std::sync::Arc;
use parking_lot::RwLock;
use std::collections::HashMap;

pub struct ThreadCoordinator {
    config: ThreadPoolConfig,
}

pub struct ThreadPoolConfig {
    pub packages_per_installer_thread: usize,
    pub max_installer_threads: usize,
}

impl Default for ThreadPoolConfig {
    fn default() -> Self {
        Self {
            packages_per_installer_thread: 5,
            max_installer_threads: num_cpus::get(),
        }
    }
}

impl ThreadCoordinator {
    pub async fn new(config: ThreadPoolConfig) -> anyhow::Result<Self> {
        log::info!("Initializing ThreadCoordinator");
        log::info!("  Max threads: {}", config.max_installer_threads);
        log::info!("  Packages per thread: {}", config.packages_per_installer_thread);

        Ok(Self { config })
    }

    pub async fn search_packages(&self, query: &str) -> anyhow::Result<Vec<crate::types::package::Package>> {
        log::debug!("Coordinator searching for: {}", query);
        Ok(vec![])
    }

    pub async fn get_package_info(&self, package_id: &str) -> anyhow::Result<crate::types::package::PackageInfo> {
        log::debug!("Coordinator getting info for: {}", package_id);
        Ok(crate::types::package::PackageInfo {
            id: package_id.to_string(),
            name: "Test".to_string(),
            version: "1.0".to_string(),
            description: None,
            category: Vec::new(),
            size_bytes: 0,
            license: None,
            homepage: None,
        })
    }

    pub async fn list_installed(&self) -> anyhow::Result<Vec<crate::types::package::Package>> {
        log::debug!("Coordinator listing installed");
        Ok(vec![])
    }

    pub async fn shutdown(&mut self) -> anyhow::Result<()> {
        log::info!("Shutting down ThreadCoordinator");
        Ok(())
    }
}
