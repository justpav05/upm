// Imports
use super::{ConfigError, Result, Config};
use super::files::read_toml;

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

// Default config path
const DEFAULT_CONFIG_PATH: &str = "/etc/upac/config.toml";

// Config for Upac
#[derive(Debug, Serialize, Deserialize)]
pub struct UpacConfig {
    pub database_path: PathBuf,
    pub package_dir:   PathBuf,
    pub temp_dir:      PathBuf,
    pub root_dir:      PathBuf,
    pub ostree:        OStreeConfig,
}

// Config for OStree
#[derive(Debug, Serialize, Deserialize)]
pub struct OStreeConfig {
    pub enabled:   bool,
    pub repo_path: PathBuf,
}

// Implementation Default for OStreeConfig
impl Default for OStreeConfig {
    fn default() -> Self {
        Self {
            enabled:   false,
            repo_path: PathBuf::from("/var/lib/upac/repo"),
        }
    }
}

// Implementation Default for UpacConfig
impl Default for UpacConfig {
    fn default() -> Self {
        Self {
            database_path: PathBuf::from("/var/lib/upac/db"),
            package_dir:   PathBuf::from("/var/lib/upac/packages"),
            temp_dir:      PathBuf::from("/tmp/upac"),
            root_dir:      PathBuf::from("/"),
            ostree:        OStreeConfig::default(),
        }
    }
}

// Implementation Config for UpacConfig
impl Config for UpacConfig {
	// Function for loading config
    fn load() -> Result<Self> {
        let path = Path::new(DEFAULT_CONFIG_PATH);

        if !path.exists() {
            return Ok(Self::default_config());
        }

        let config: Self = read_toml(&path)?;

        config.validate()?;

        Ok(config)
    }

    // Get default config
    fn default_config() -> Self {
        Self::default()
    }

    // Function for validating config
    fn validate(&self) -> Result<()> {
        if !self.root_dir.exists() && !self.root_dir.is_dir() {
            return Err(ConfigError::PathError(self.root_dir.clone()));
        }

        if !self.database_path.is_absolute() {
            return Err(ConfigError::PathError(self.database_path.clone()));
        }

        if !self.package_dir.is_absolute() {
                return Err(ConfigError::PathError(self.package_dir.clone()));
            }

        if !self.temp_dir.is_absolute() {
        	return Err(ConfigError::PathError(self.temp_dir.clone()));
        }

        if self.ostree.enabled {
        	if self.ostree.repo_path == PathBuf::from("") {
            	return Err(ConfigError::PathError(self.ostree.repo_path.clone()));
            }
            if !self.ostree.repo_path.is_absolute() {
            	return Err(ConfigError::PathError(self.ostree.repo_path.clone()));
            }
        }

        if self.ostree.enabled && self.ostree.repo_path == PathBuf::from("") {
            return Err(ConfigError::PathError(self.ostree.repo_path.clone()));
        }

        Ok(())
    }
}
