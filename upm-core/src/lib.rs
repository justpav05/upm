//! Universal Package Manager Core Library

pub mod core;
pub mod types;

pub use crate::types::package::Package;
pub use core::manager::PackageManager;
pub use core::threadcoordination::ThreadCoordinator;

pub mod prelude {
    pub use crate::{PackageManager, ThreadCoordinator, types::package::Package};
}
