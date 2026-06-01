//! AgentFlow core module boundary.
//!
//! The current crate keeps archived workflow code available as a compatibility
//! layer while new AgentFlow requirements are rebuilt from `docs/requirements/`.

pub mod active;
pub mod legacy;
pub mod shared;

// Temporary compatibility export for legacy CLI and Desktop.
// Do not use from new requirements.
pub use legacy::*;
