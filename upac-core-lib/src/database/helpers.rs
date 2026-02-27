use super::{DatabaseError, Result};

use toml::{from_str, to_string_pretty};

use serde::de::DeserializeOwned;
use serde::Serialize;

use std::fs;
use std::path::Path;

pub(crate) fn read_toml<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let content = fs::read_to_string(path)?;
    Ok(from_str(&content)?)
}

pub(crate) fn write_toml<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let content = to_string_pretty(value)?;
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, content)?;
    fs::rename(&tmp, path)?;
    Ok(())
}

pub(crate) fn ensure_directory(path: &Path) -> Result<()> {
    if path.exists() {
        if path.is_dir() {
            Ok(())
        } else {
            Err(DatabaseError::PathError(path.to_path_buf()))
        }
    } else {
        fs::create_dir_all(path).map_err(DatabaseError::from)
    }
}
