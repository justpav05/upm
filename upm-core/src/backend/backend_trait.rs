// ============================================================================
// Imports
// ============================================================================
use crate::types::{ExtractedPackage, PackageMetadata};
// ============================================================================
// Backend
// ============================================================================
pub trait Backend: Send + Sync {
    fn name(&self) -> &str;
    fn supported_formats(&self) -> Vec<String>;

    fn detect(&self, package_path: &Path) -> bool;

    fn extract(&self, package_path: &Path) -> Result<ExtractedPackage>;
    fn get_metadata(&self, package_path: &Path) -> Result<PackageMetadata>;
    fn validate(&self, package_path: &Path) -> Result<()>;

    fn verify_signature(&self, package_path: &Path) -> Result<bool> {
        Ok(true)
    }

    fn get_provides(&self, package_path: &Path) -> Result<Vec<String>> {
        Ok(vec![])
    }

    fn supports_delta_updates(&self) -> bool {
        false
    }
}
