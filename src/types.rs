// Imports
use serde::{Deserialize, Serialize};

use std::path::PathBuf;

// Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
	pub name: String,
    pub version: String,
    pub format: String,
    pub install_date: String,
}

pub struct ExtractedPackage {
	pub name: String,
    pub version: String,
    pub format: String,
    pub file_list: Vec<PathBuf>,
}
