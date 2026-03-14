// Imports
use super::{ConfigError, ConfigResult, Config};

use toml::Value;

use stabby::string::String as StabString;
use stabby::result::Result as StabResult;

use std::path::PathBuf;
use std::ffi::c_void;
use std::fs;

// Default paths
const DEFAULT_PACKAGE_DIR: &str = "/var/lib/upac/packages";
const DEFAULT_CONFIG_PATH: &str = "/etc/upac/config.toml";
const DEFAULT_DATABASE_PATH: &str = "/var/lib/upac/db";
const DEFAULT_REPO_PATH: &str = "/var/lib/upac/repo";
const DEFAULT_TEMP_DIR: &str = "/tmp/upac";
const DEFAULT_ROOT_DIR: &str = "/";

// Config for OStree
#[stabby::stabby]
#[derive(Debug, Clone)]
pub struct OStreeConfig {
    pub enabled:   bool,
    pub repo_path: StabString,
}

// Config for Upac
#[stabby::stabby]
#[derive(Debug, Clone)]
pub struct UpacConfig {
    pub database_path: StabString,
    pub package_dir:   StabString,
    pub temp_dir:      StabString,
    pub root_dir:      StabString,
    pub ostree:        OStreeConfig,
}


// Implementation Default for OStreeConfig
impl Default for OStreeConfig {
    fn default() -> Self {
        Self {
            enabled:   false,
            repo_path: StabString::from(DEFAULT_REPO_PATH),
        }
    }
}

// Implementation Default for UpacConfig
impl Default for UpacConfig {
    fn default() -> Self {
        Self {
            database_path: StabString::from(DEFAULT_DATABASE_PATH),
            package_dir:   StabString::from(DEFAULT_PACKAGE_DIR),
            temp_dir:      StabString::from(DEFAULT_TEMP_DIR),
            root_dir:      StabString::from(DEFAULT_ROOT_DIR),
            ostree:        OStreeConfig::default(),
        }
    }
}

impl UpacConfig {
	fn get_str<'a>(value: &'a Value, key: &str) -> ConfigResult<&'a str> {
    	value[key].as_str().ok_or_else(|| ConfigError::ParseError(format!("missing field: {key}").into()))
	}

	fn get_nested_str<'a>(value: &'a Value, section: &str, key: &str) -> ConfigResult<&'a str> {
    	value[section][key].as_str().ok_or_else(|| ConfigError::ParseError(format!("missing field: {section}.{key}").into()))
	}
}

// Implementation Config for UpacConfig
impl Config for UpacConfig {
	// Function for loading config
	fn load() -> ConfigResult<Self> {
        let content = fs::read_to_string(DEFAULT_CONFIG_PATH).unwrap_or_default();

        if content.is_empty() {
            return Ok(Self::default_config());
        }

        let value: Value = content.parse()?;

        Ok(Self {
            database_path: Self::get_str(&value, "database_path")?.into(),
            package_dir:   Self::get_str(&value, "package_dir")?.into(),
            temp_dir:      Self::get_str(&value, "temp_dir")?.into(),
            root_dir:      Self::get_str(&value, "root_dir")?.into(),
            ostree: OStreeConfig {
                enabled:   value["ostree"]["enabled"].as_bool().unwrap_or(false),
                repo_path: Self::get_nested_str(&value, "ostree", "repo_path")?.into(),
            },
        })
    }

    // Get default config
    fn default_config() -> Self {
        Self::default()
    }

    // Function for validating config
    fn validate(&self) -> ConfigResult<()> {
    	let root_dir_path = PathBuf::from(self.root_dir.as_str());
        if !root_dir_path.exists() && !root_dir_path.is_dir() {
            return Err(ConfigError::PathError(self.root_dir.clone()));
        }

        let database_path = PathBuf::from(self.database_path.as_str());
        if !database_path.is_absolute() {
            return Err(ConfigError::PathError(self.database_path.clone()));
        }

        let package_dir_path = PathBuf::from(self.package_dir.as_str());
        if !package_dir_path.is_absolute() {
                return Err(ConfigError::PathError(self.package_dir.clone()));
            }

        let temp_dir_path = PathBuf::from(self.temp_dir.as_str());
        if !temp_dir_path.is_absolute() {
        	return Err(ConfigError::PathError(self.temp_dir.clone()));
        }

        let ostree_repo_path = PathBuf::from(self.ostree.repo_path.as_str());
        if self.ostree.enabled && !ostree_repo_path.is_absolute() {
            return Err(ConfigError::PathError(self.ostree.repo_path.clone()));
        }

        Ok(())
    }
}

#[no_mangle]
pub extern "C" fn upac_config_load() -> StabResult<*mut c_void, ConfigError> {
    match UpacConfig::load() {
        Ok(config) => Ok(Box::into_raw(Box::new(config)) as *mut c_void).into(),
        Err(err)   => Err(err).into(),
    }
}

#[no_mangle]
pub extern "C" fn upac_config_default() -> *mut c_void {
    Box::into_raw(Box::new(UpacConfig::default_config())) as *mut c_void
}

#[no_mangle]
pub extern "C" fn upac_config_free(config: *mut c_void) {
    if !config.is_null() {
        unsafe { drop(Box::from_raw(config as *mut UpacConfig)) };
    }
}
