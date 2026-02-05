// ============================================================================
// Mods declaration
// ============================================================================
mod conflict;
mod graph;
mod priority;
mod resolver;
mod virtual_pkg;
// ============================================================================
// Mods export
// ============================================================================
pub use conflict::{Conflict, ConflictDetector, ConflictType};
pub use graph::{DependencyGraph, DependencyNode};
pub use priority::PriorityManager;
pub use resolver::DependencyResolver;
pub use virtual_pkg::VirtualPackageManager;
