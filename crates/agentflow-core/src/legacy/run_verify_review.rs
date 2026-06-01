//! Legacy compatibility module.
//!
//! This module belongs to the archived 2026-05 workflow/product-feature system.
//! It is kept only for compatibility and migration.
//! New AgentFlow requirements must not depend on this module unless a new
//! requirement explicitly re-authorizes it.

pub use super::archive_2026_05::{
    collect_context, plan_issue, review_issue, run_issue, verify_issue, write_context,
    write_project_summary, write_review_assistant, AgentRun, CommandRecord, ContextSummary,
    ControlledRunPlan, HumanGate, InitSummary, PlanSummary, ProjectSummaryResult,
    ReviewAssistantCheck, ReviewAssistantSummary, ReviewSummary, RunOutputs, RunSummary,
    ValidationSpec, ValidationSummary,
};
