//! AgentFlow core module boundary.
//!
//! This crate now retains archived 2026-05 workflow code only as historical
//! reference. Active task execution, projection, release, and audit flows must
//! use their dedicated crates instead of calling this archive.

pub mod legacy;
pub mod shared;
