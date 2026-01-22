//! Universal Package Manager Core Library
//! Полнофункциональная библиотека для управления пакетами

pub mod core {
    pub mod manager;
    pub mod threadcoordination;
}

pub mod types {
    pub mod errors;
    pub mod package;
}

pub use core::manager::PackageManager;
pub use core::threadcoordination::ThreadCoordinator;
pub use crate::types::package::{Package, PackageInfo};

pub mod prelude {
    pub use crate::{
        types::package::{Package, PackageInfo},
        PackageManager, ThreadCoordinator,
    };
}
