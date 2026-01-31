//! Universal Package Manager Core Library

pub mod database;
pub mod package_manager;
pub mod threadcoordination;
pub mod types;

pub use crate::types::package::package::Package;
pub use threadcoordination::ThreadCoordinator;

pub mod prelude {
    pub use crate::{
        ThreadCoordinator, package_manager::PackageManager, types::database::DataBase,
        types::package::package::Package,
    };
}
