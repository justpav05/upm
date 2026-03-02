// Imports
use crate::core::lock::Result;

use abi_stable::{
    sabi_trait,
    std_types::RString,
    StableAbi,
};

use std::path::Path;

// Mods
pub mod permission;
pub mod backend;
pub mod lock;
pub mod types;



#[sabi_trait]
pub trait Lockable: Sized {
    fn acquire(path: RString) -> RResult<Self, LockError>;
}
