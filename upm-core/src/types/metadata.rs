// ============================================================================
// Package metadata
// ============================================================================
pub struct PackageMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub maintainer: String,
    pub homepage: Option<String>,
    pub license: String,
    pub dependencies: Vec<Dependency>,
    pub conflicts: Vec<String>,
    pub provides: Vec<String>,
    pub replaces: Vec<String>,
}
// ============================================================================
// Dependency
// ============================================================================
pub struct Dependency {
    pub name: String,
    pub version_constraint: Option<VersionConstraint>,
    pub is_optional: bool,
}
// ============================================================================
// Version constraint
// ============================================================================
pub struct VersionConstraint {
    pub operator: VersionOperator,
    pub version: String,
}
// ============================================================================
// Version operator
// ============================================================================
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionOperator {
    Equal,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
}
