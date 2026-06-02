//! Legacy compatibility module.
//!
//! This module belongs to the archived 2026-05 workflow/product-feature system.
//! It is kept only for compatibility and migration.
//! New AgentFlow requirements must not depend on this module unless a new
//! requirement explicitly re-authorizes it.

pub use super::archive_2026_05::{
    canonical_issue_status, canonical_issue_status_string, canonical_project_status,
    canonical_project_status_string, create_issue, create_milestone, create_project, create_team,
    read_issue_project_link_preview, read_local_project_model_snapshot,
    read_local_project_seed_preview, read_project_milestone_issue_view_model_snapshot,
    write_issue_project_link, write_local_project_seed, CreationPreview, CreationPreviewFile,
    CreationV1ContractPreview, CreationWriteSummary, IssueContext, IssueContract, IssueDraft,
    IssueProjectLink, IssueProjectLinkPreview, IssueProjectLinkWriteSummary, IssueStatus,
    LocalMilestone, LocalProject, LocalProjectIssueRef, LocalProjectModelSnapshot,
    LocalProjectSeedFile, LocalProjectSeedPreview, LocalProjectSeedWriteSummary, LocalTeam,
    LocalWorkspace, MilestoneDerivedProgress, MilestoneDraft, ProjectDraft,
    ProjectMilestoneIssueViewModelSnapshot, ProjectStatus, TeamCreationV1Preview, TeamDraft,
    V1Issue, V1Milestone, V1Project, V1TeamRef, V1View, V1ViewFilter, V1ViewSort, V1WorkspaceRef,
};
