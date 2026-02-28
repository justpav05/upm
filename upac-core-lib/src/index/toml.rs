use crate::DatabaseError;

use toml::{from_str, to_string_pretty};

use serde::de::DeserializeOwned;
use serde::Serialize;

use std::fs;
use std::path::Path;

pub type Result<T> = std::result::Result<T, DatabaseError>;

pub(super) fn read_toml<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let content = fs::read_to_string(path)?;
    Ok(from_str(&content)?)
}

pub(super) fn write_toml<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let content = to_string_pretty(value)?;
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, content)?;
    fs::rename(&tmp, path)?;
    Ok(())
}
