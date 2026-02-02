mod error;
mod file_entry;
mod metadata;
mod package;
mod scripts;

pub use error::{Error, Result};
pub use file_entry::FileEntry;
pub use metadata::PackageMetadata;
pub use package::{ExtractedPackage, Package, PackageInfo};
pub use scripts::{Script, Scripts};

// Re-export common types
pub use crate::lock::LockType;
pub use crate::repository::RepositoryType;
pub use crate::transaction::TransactionStatus;
