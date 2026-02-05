// ============================================================================
// Imports
// ============================================================================
use std::collections::{HashMap, HashSet};
// ============================================================================
// Dependency graph
// ============================================================================
pub struct DependencyGraph {
    pub nodes: HashMap<String, DependencyNode>,
    pub root: String,
}

impl DependencyGraph {
    pub fn new(root: String) -> Self;
    pub fn add_node(&mut self, name: String, node: DependencyNode);
    pub fn get_install_order(&self) -> Result<Vec<String>>;
    pub fn has_cycles(&self) -> bool;
    pub fn visualize(&self) -> String;

    // Internal
    fn topological_sort(&self) -> Result<Vec<String>>;
    fn detect_cycle(
        &self,
        node: &str,
        visited: &mut HashSet<String>,
        stack: &mut Vec<String>,
    ) -> bool;
}

pub struct DependencyNode {
    pub package_name: String,
    pub version: String,
    pub provider: PackageProvider,
    pub dependencies: Vec<String>,
    pub is_virtual: bool,
    pub is_optional: bool,
}
