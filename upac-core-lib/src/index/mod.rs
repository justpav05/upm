use crate::{PackageInfo, DatabaseError};

use std::path::PathBuf;

pub mod index;
mod toml;

pub type Result<T> = std::result::Result<T, DatabaseError>;

pub trait PackageIndex {
    fn load(index_path: PathBuf) -> Result<Self>; //where Self: Sized

    fn save(&self) -> Result<()>;

    fn reload(&mut self) -> Result<()>;

    fn insert(&mut self, package: &PackageInfo);

    fn remove(&mut self, package_id: &str);

    fn get(&self, package_id: &str) -> Option<&PackageInfo>;

    fn list_all(&self) -> Vec<&PackageInfo>;

}
