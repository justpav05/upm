// ============================================================================
// Imports
// ============================================================================
use std::fs;
use std::path::Component;
use std::path::{Path, PathBuf};

use crate::types::{Error, Result};
// ============================================================================
// Utils path functions
// ============================================================================
pub fn validate_path(path: &Path) -> Result<()> {
    if !path.exists() {
        Err(Error::PathError(path.to_path_buf()))
    } else if !path.is_file() {
        Err(Error::PathError(path.to_path_buf()))
    } else {
        Ok(())
    }
}

pub fn sanitize_path(path: &Path) -> Result<PathBuf> {
    if path.as_os_str().is_empty() {
        return Err(Error::PathError(PathBuf::new()));
    }

    let mut result = PathBuf::new();
    let mut components = path.components().peekable();

    while let Some(component) = components.next() {
        match component {
            Component::RootDir => result.push("/"),

            Component::Normal(name) => {
                let name_str = name.to_string_lossy();

                if name_str.contains("..") || name_str.contains('/') || name_str.contains('\\') {
                    return Err(Error::PathError(path.to_path_buf()));
                }

                if name_str.len() > 255 {
                    return Err(Error::PathError(path.to_path_buf()));
                }

                result.push(name);
            }

            Component::CurDir => {
                continue;
            }

            Component::ParentDir => {
                if !result.pop() {
                    return Err(Error::PathError(path.to_path_buf()));
                }
            }

            Component::Prefix(_) => {
                return Err(Error::PathError(path.to_path_buf()));
            }
        }
    }

    if result.as_os_str().is_empty() {
        result.push(".");
    }

    Ok(result)
}

pub fn ensure_directory(path: &Path) -> Result<()> {
    if path.exists() {
        if path.is_dir() {
            Ok(())
        } else {
            Err(Error::PathError(path.to_path_buf()))
        }
    } else {
        fs::create_dir_all(path).map_err(Error::from)
    }
}

pub fn is_subpath(path: &Path, base: &Path) -> bool {
    match (path.canonicalize(), base.canonicalize()) {
        (Ok(canonical_path), Ok(canonical_base)) => canonical_path.starts_with(canonical_base),
        _ => false,
    }
}

pub fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB", "PB"];
    const THRESHOLD: f64 = 1024.0;

    if bytes == 0 {
        return "0 B".to_string();
    }

    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= THRESHOLD && unit_index < UNITS.len() - 1 {
        size /= THRESHOLD;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}
