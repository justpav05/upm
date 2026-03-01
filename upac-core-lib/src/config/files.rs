// Imports
use super::Result;

use toml::{from_str};

use serde::de::DeserializeOwned;

use std::fs;
use std::path::Path;

// Function to read a TOML file and deserialize its contents
pub(super) fn read_toml<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let content = fs::read_to_string(path)?;
    Ok(from_str(&content)?)
}
