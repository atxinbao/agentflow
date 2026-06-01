//! Shared helpers for AgentFlow core.
//!
//! This module is reserved for utilities with no workflow/product stance:
//! filesystem paths, JSON I/O, Markdown helpers, IDs, and time helpers.
//! Legacy workflow concepts must not be added here.

pub mod fs_paths;
pub mod ids;
pub mod json_io;
pub mod markdown;
pub mod time;
