//! Universal Package Manager Core Library
//! Полнофункциональная библиотека для управления пакетами

pub mod core {
    pub mod manager;
    pub mod thread_coordinator;
}

pub mod types {
    pub mod errors;
    pub mod package;
}

pub use core::manager::PackageManager;
pub use core::thread_coordinator::ThreadCoordinator;
pub use types::{Package, PackageInfo, Result};

pub mod prelude {
    pub use crate::{
        types::{Operation, Package, PackageInfo, Result},
        PackageManager, ThreadCoordinator,
    };
}
