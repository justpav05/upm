// Imports
use crate::core::lock::Result;

use std::path::Path;

// Mods
pub mod permitions;
pub mod backend;
pub mod lock;
pub mod types;

// Trait for file locking
pub trait Lockable: Sized {
    fn acquire(path: &Path) -> Result<Self>;
}
