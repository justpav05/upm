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

pub struct Dependency {
    pub name: String,
    pub version_constraint: Option<VersionConstraint>,
    pub is_optional: bool,
}

pub struct VersionConstraint {
    pub operator: VersionOperator,
    pub version: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionOperator {
    Equal,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
}
