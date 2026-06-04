//! Tauri command registration modules.
//!
//! Command names stay stable while implementation modules are split by product
//! boundary. Desktop remains read-only unless a requirement explicitly changes
//! that boundary.

pub(crate) mod agent_manual;
pub(crate) mod execute;
pub(crate) mod goal_tree;
pub(crate) mod input;
pub(crate) mod legacy_core;
pub(crate) mod output;
pub(crate) mod panel;
pub(crate) mod project_files;
pub(crate) mod project_workspace;
pub(crate) mod state;
