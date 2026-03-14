// Imports
use serde::{Deserialize, Serialize};

use stabby::option::Option as StabOption;
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
    pub dependencies: StabVec<StabString>,

    pub pre_install: StabOption<StabString>,
    pub post_install: StabOption<StabString>,
    pub pre_remove: StabOption<StabString>,
    pub post_remove: StabOption<StabString>,
}

pub enum OSTreeOperation {
    Install,
    Remove,
    Update,
}

impl OSTreeOperation {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Install => "install",
            Self::Remove => "remove",
            Self::Update => "update",
        }
    }
}
