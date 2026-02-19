use crate::backend::{Backend, ExtractedPackage, FileEntry, Result};
use std::path::{Path, PathBuf};

pub struct MockBackend;

impl Backend for MockBackend {
    fn name(&self) -> &str {
        "mock"
    }

    fn supported_formats(&self) -> Vec<&str> {
        vec!["mock"]
    }

    fn detect(&self, _path: &Path) -> bool {
        true
    }

    fn extract(&self, _path: &Path, temp_dir: &Path) -> Result<ExtractedPackage> {
        let test_file = temp_dir.join("usr").join("bin").join("test-app");
        std::fs::create_dir_all(test_file.parent().unwrap())?;
        std::fs::write(&test_file, "#!/bin/sh\necho test")?;

        Ok(ExtractedPackage {
            name: "test-package".to_string(),
            version: "1.0.0".to_string(),
            format: "mock".to_string(),
            files: vec![FileEntry {
                relative_path: PathBuf::from("usr/bin/test-app"),
                permissions: 0o755,
                owner: 0,
                group: 0,
            }],
        })
    }
}
