//! Tauri command registration modules.
//!
//! Command names stay stable while implementation modules are split by product
//! boundary. Desktop remains read-only unless a requirement explicitly changes
//! that boundary.

pub(crate) mod goal_tree;
pub(crate) mod graph;
pub(crate) mod legacy_core;
pub(crate) mod project_files;
pub(crate) mod project_workspace;
