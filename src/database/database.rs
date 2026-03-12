// Imports
use super::{DatabaseError, Database, DatabaseResult};

use crate::lock::{Lock, ExclusiveLock, SharedLock};
use crate::types::{Package, ExtractedPackage};

use serde::{Serialize,Deserialize, de::DeserializeOwned};

use toml::{from_str, to_string_pretty};

use time::OffsetDateTime;

use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::fs;

const DATABASE_LOCK_FILE_NAME: &str = "database.lock";

const PACKAGE_DIR_NAME: &str = "packages";

const PACKAGES_MAP_FILE_NAME: &str = "packages_map.toml";
const FILES_TOML_FILE_NAME: &str = "files.toml";

// Struct definition for database
#[derive(Debug, Clone)]
pub struct PackageDatabase {
    database_path: PathBuf,
    packages_map: HashMap<String, Package>,
}

#[derive(Serialize, Deserialize)]
struct FileList {
    files: Vec<PathBuf>,
}

// Implementation of Database struct own functions
impl PackageDatabase {

	// Function to create a new database instance
    pub fn new(database_path: PathBuf) -> DatabaseResult<Self> {
        Self::ensure_directory(&database_path)?;
        Self::ensure_directory(&database_path.join(PACKAGE_DIR_NAME))?;

        let packages_map: HashMap<String, Package> = {
            let packages_map_file = database_path.join(PACKAGES_MAP_FILE_NAME);
            if packages_map_file.exists() {
                Self::read_toml(&packages_map_file)?
            } else {
                HashMap::default()
            }
        };

        Ok(Self { database_path, packages_map })
    }

    // Function to ensure a directory exists, creating it if necessary
    pub(super) fn ensure_directory(path: &Path) -> DatabaseResult<()> {
        if path.exists() {
            if path.is_dir() {
                Ok(())
            } else {
                Err(DatabaseError::Path(path.to_path_buf()))
            }
        } else {
            fs::create_dir_all(path).map_err(DatabaseError::from)
        }
    }

    // Function to read a TOML file and deserialize its contents
    pub(super) fn read_toml<T: DeserializeOwned>(path: &Path) -> DatabaseResult<T> {
        let content = fs::read_to_string(path)?;
        Ok(from_str(&content)?)
    }

    // Function to write a TOML file from a value
    pub(super) fn write_toml<T: Serialize>(path: &Path, value: &T) -> DatabaseResult<()> {
        let content = to_string_pretty(value)?;
        let tmp = path.with_extension("tmp");
        fs::write(&tmp, content)?;
        fs::rename(&tmp, path)?;
        Ok(())
    }
}

impl Database for PackageDatabase {
    fn add_package(&mut self, package: &ExtractedPackage) -> DatabaseResult<()> {
    	let lock = ExclusiveLock::new(self.database_path.join(DATABASE_LOCK_FILE_NAME));
    	let _guard = lock.lock()?;

    	if !self.database_path.exists() {
    		return Err(DatabaseError::Path(self.database_path.clone()))
    	}

    	if !self.database_path.join(PACKAGE_DIR_NAME).exists() {
    		return Err(DatabaseError::Path(self.database_path.clone()))
    	}

     	if !self.database_path.join(PACKAGES_MAP_FILE_NAME).exists() {
    		return Err(DatabaseError::Path(self.database_path.clone()))
    	}

     	let install_package_date = OffsetDateTime::now_utc().to_string();

     	let package_info = Package {
     		name: package.name.to_string().clone(),
     		version: package.version.to_string().clone(),
     		format: package.format.to_string().clone(),
     		install_date: install_package_date,
     	};

     	let file_list = FileList {
          files: package.file_list.iter().map(|string| PathBuf::from(string.as_str())).collect(),
      	};

     	self.packages_map.insert(package.name.to_string().clone(), package_info);

      	let package_dir_path = self.database_path.join(PACKAGE_DIR_NAME).join(&package.name.to_string());

      	Self::ensure_directory(&package_dir_path)?;

      	Self::write_toml(&package_dir_path.join(FILES_TOML_FILE_NAME), &file_list)?;
        Self::write_toml(&self.database_path.join(PACKAGES_MAP_FILE_NAME), &self.packages_map)?;

     	Ok(())
    }

    fn remove_package(&mut self, package_id: &str) -> DatabaseResult<()> {
   		let lock = ExclusiveLock::new(self.database_path.join(DATABASE_LOCK_FILE_NAME));
    	let _guard = lock.lock()?;

   		if !self.database_path.exists() {
   			return Err(DatabaseError::Path(self.database_path.clone()))
    	}

   		if !self.database_path.join(PACKAGE_DIR_NAME).exists() {
   			return Err(DatabaseError::Path(self.database_path.clone()))
    	}

    	if !self.database_path.join(PACKAGES_MAP_FILE_NAME).exists() {
   			return Err(DatabaseError::Path(self.database_path.clone()))
    	}

     	let package_dir_path = self.database_path.join(PACKAGE_DIR_NAME).join(&package_id);

      	if !package_dir_path.exists() {
      		return Err(DatabaseError::NotFound);
      	}

      	fs::remove_file(package_dir_path.join(FILES_TOML_FILE_NAME))?;
        fs::remove_dir(package_dir_path)?;

       	self.packages_map.remove(package_id);

        Self::write_toml(&self.database_path.join(PACKAGES_MAP_FILE_NAME), &self.packages_map)?;

      	Ok(())
    }

    fn get_package(&self, query: &str) -> DatabaseResult<Package> {
   		let lock = SharedLock::new(self.database_path.join(DATABASE_LOCK_FILE_NAME));
    	let _guard = lock.lock()?;

     	if !self.database_path.exists() {
   			return Err(DatabaseError::Path(self.database_path.clone()))
    	}

   		if !self.database_path.join(PACKAGE_DIR_NAME).exists() {
   			return Err(DatabaseError::Path(self.database_path.clone()))
    	}

     	self.packages_map.get(query).ok_or_else(|| DatabaseError::NotFound).cloned()
    }

    fn get_package_files(&self, package_id: &str) -> DatabaseResult<Vec<PathBuf>> {
    	let lock = SharedLock::new(self.database_path.join(DATABASE_LOCK_FILE_NAME));
   		let _guard = lock.lock()?;

     	let package_dir_path = self.database_path.join(PACKAGE_DIR_NAME).join(package_id);
      	let package_fils_file_path = package_dir_path.join(FILES_TOML_FILE_NAME);

     	if !self.database_path.exists() {
  			return Err(DatabaseError::Path(self.database_path.clone()))
     	}

  		if !self.database_path.join(PACKAGE_DIR_NAME).exists() {
  			return Err(DatabaseError::Path(self.database_path.clone()))
   		}

     	if !package_dir_path.exists() {
  			return Err(DatabaseError::Path(package_dir_path))
     	}

     	let file_list: FileList = Self::read_toml(&package_fils_file_path)?;

     	Ok(file_list.files)
    }

    fn add_file(&mut self, package_id: &str, file_path: &Path) -> DatabaseResult<()> {
   		let lock = ExclusiveLock::new(self.database_path.join(DATABASE_LOCK_FILE_NAME));
   		let _guard = lock.lock()?;

    	let package_dir_path = self.database_path.join(PACKAGE_DIR_NAME).join(package_id);
        let package_files_file_path = package_dir_path.join(FILES_TOML_FILE_NAME);

    	if !self.database_path.exists() {
  			return Err(DatabaseError::Path(self.database_path.clone()))
    	}

  		if !self.database_path.join(PACKAGE_DIR_NAME).exists() {
  			return Err(DatabaseError::Path(self.database_path.clone()))
   		}

     	if self.packages_map.get(package_id).is_none() {
             return Err(DatabaseError::NotFound);
        }

        let mut file_list: FileList = Self::read_toml(&package_files_file_path)?;
        file_list.files.push(file_path.to_path_buf());

        Self::write_toml(&package_files_file_path, &file_list)?;

        Ok(())
    }

    fn remove_file(&mut self, package_id: &str, file_path: &Path) -> DatabaseResult<()> {
   		let lock = ExclusiveLock::new(self.database_path.join(DATABASE_LOCK_FILE_NAME));
  		let _guard = lock.lock()?;

    	let package_dir_path = self.database_path.join(PACKAGE_DIR_NAME).join(package_id);
    	let package_files_file_path = package_dir_path.join(FILES_TOML_FILE_NAME);

   		if !self.database_path.exists() {
 			return Err(DatabaseError::Path(self.database_path.clone()))
    	}

 		if !self.database_path.join(PACKAGE_DIR_NAME).exists() {
 			return Err(DatabaseError::Path(self.database_path.clone()))
  		}

  	 	if self.packages_map.get(package_id).is_none() {
           return Err(DatabaseError::NotFound);
       	}

        let mut file_list: FileList = Self::read_toml(&package_files_file_path)?;
        let original_len = file_list.files.len();
        file_list.files.retain(|f| f != file_path);

        if file_list.files.len() == original_len {
        	return Err(DatabaseError::NotFound);
        }

        Self::write_toml(&package_files_file_path, &file_list)?;

      	Ok(())

    }
}
