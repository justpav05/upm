// ============================================================================
// Imports
// ============================================================================
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

use crate::types::{Error, Result};
// ============================================================================
// Utils toml functions
// ============================================================================
pub fn read_toml<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let mut file = fs::File::open(path).map_err(|error| Error::IoError(error))?;
    let mut contents = String::new();

    file.read_to_string(&mut contents)
        .map_err(|error| Error::IoError(error))?;

    toml::from_str(&contents).map_err(|error| Error::TomlError(error))
}

pub fn write_toml<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let content =
        toml::to_string_pretty(value).map_err(|error| Error::InvalidConfig(error.to_string()))?;
    let mut file = fs::File::create(path).map_err(|error| Error::IoError(error))?;

    file.write_all(content.as_bytes())
        .map_err(|error| Error::IoError(error))?;

    Ok(())
}

pub fn write_toml_atomic<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let content =
        toml::to_string_pretty(value).map_err(|error| Error::InvalidConfig(error.to_string()))?;

    let temp_path = path.with_extension("tmp");

    {
        let mut temp_file = fs::File::create(&temp_path).map_err(Error::from)?;
        temp_file
            .write_all(content.as_bytes())
            .map_err(Error::from)?;
        temp_file.sync_all().map_err(Error::from)?;
    }

    fs::rename(&temp_path, path).map_err(Error::from)?;

    Ok(())
}
