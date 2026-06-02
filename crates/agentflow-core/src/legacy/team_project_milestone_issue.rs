//! Legacy compatibility module.
//!
//! This module belongs to the archived 2026-05 workflow/product-feature system.
//! It is kept only for compatibility and migration.
//! New AgentFlow requirements must not depend on this module unless a new
//! requirement explicitly re-authorizes it.

pub use super::archive_2026_05::{
    canonical_issue_status, canonical_issue_status_string, canonical_project_status,
    canonical_project_status_string, read_local_project_model_snapshot,
    read_project_milestone_issue_view_model_snapshot, IssueContext, IssueContract, IssueStatus,
    LocalMilestone, LocalProject, LocalProjectIssueRef, LocalProjectModelSnapshot, LocalTeam,
    LocalWorkspace, MilestoneDerivedProgress, ProjectMilestoneIssueViewModelSnapshot,
    ProjectStatus, V1Issue, V1Milestone, V1Project, V1TeamRef, V1View, V1ViewFilter, V1ViewSort,
    V1WorkspaceRef,
};
