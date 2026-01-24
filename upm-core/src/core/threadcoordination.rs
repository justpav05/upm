// This suppresses all the unused crate warnings.
#![allow(unused)]
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

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
        log::info!(
            "  Packages per thread: {}",
            config.packages_per_installer_thread
        );

        Ok(Self { config })
    }

    pub async fn search_packages(
        &self,
        query: &str,
    ) -> anyhow::Result<Vec<crate::types::package::Package>> {
        log::debug!("Coordinator searching for: {}", query);
        Ok(vec![])
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
