// Imports
use serde::{Deserialize, Serialize};

use stabby::string::String as StabString;
use stabby::vec::Vec as StabVec;

// Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
	pub name: String,
    pub version: String,
    pub format: String,
    pub install_date: String,
}

#[stabby::stabby]
pub struct ExtractedPackage {
	pub name: StabString,
    pub version: StabString,
    pub format: StabString,
    pub file_list: StabVec<StabString>,
}
