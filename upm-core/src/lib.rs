//! Universal Package Manager Core Library
//! Полнофункциональная библиотека для управления пакетами

pub mod core {
    pub mod manager;
    pub mod thread_coordinator;
}

pub mod dependency {
    pub mod resolver;
}

pub mod types {
    pub mod package;
    pub mod operation;
    pub mod errors;
}

pub use core::manager::PackageManager;
pub use core::thread_coordinator::ThreadCoordinator;
pub use types::{Package, PackageInfo, Operation, Result};

pub mod prelude {
    pub use crate::{
        PackageManager,
        ThreadCoordinator,
        types::{Package, PackageInfo, Operation, Result},
    };
}
