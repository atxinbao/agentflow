//! Legacy compatibility boundary.
//!
//! The archived 2026-05 workflow/product-feature implementation lives in
//! `archive_2026_05`. The files in this directory expose named compatibility
//! seams so new requirements can see which old domain a symbol belongs to
//! without treating legacy as the AgentFlow product core.

pub mod archive_2026_05;
pub mod eligibility_lease;
pub mod evidence;
pub mod goal_protocol;
pub mod product_feature;
pub mod project_audit_docs_refresh;
pub mod project_closure;
pub mod run_verify_review;
pub mod saved_view;
pub mod sqlite_index;
pub mod team_project_milestone_issue;
pub mod workflow_control;

// Temporary compatibility export for archived CLI/Desktop callers.
// Do not use from new requirements.
pub use archive_2026_05::*;
