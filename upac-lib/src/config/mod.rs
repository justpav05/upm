// Imports
use upac_types::{ConfigError, ConfigResult};

use stabby::string::String as StabString;

// Mods
pub mod config;

// Trait for functions to load config
pub(crate) trait Config: Sized {
    fn load() -> ConfigResult<Self>;
    fn default_config() -> Self;
    fn validate(&self) -> ConfigResult<()>;
}
