//! Legacy compatibility module.
//!
//! This module belongs to the archived 2026-05 workflow/product-feature system.
//! It is kept only for compatibility and migration.
//! New AgentFlow requirements must not depend on this module unless a new
//! requirement explicitly re-authorizes it.

pub use super::archive_2026_05::{
    bootstrap_goal_protocol, check_goal_readiness, compile_goal_from_markdown, init_from_goal,
    require_goal_initialized, write_goal_next, GoalBootstrapSummary, GoalLoopCounts,
    GoalLoopIssueRef, GoalLoopNextSummary, GoalLoopSources, GoalLoopState, GoalReadinessCheck,
    GoalReadinessSummary, ProjectDefinition, ProjectDefinitionOutput, ProjectGoal,
};
