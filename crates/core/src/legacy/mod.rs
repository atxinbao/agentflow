//! Legacy compatibility boundary.
//!
//! The archived 2026-05 workflow/product-feature implementation lives in
//! `archive_2026_05`. The files in this directory expose named compatibility
//! seams so new requirements can see which old domain a symbol belongs to
//! without treating legacy as the AgentFlow product core.

mod archive_2026_05;
pub mod team_project_milestone_issue;
pub mod workflow_control;
