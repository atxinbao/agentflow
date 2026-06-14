//! AgentFlow spec contract layer.
//!
//! This crate owns the new task-contract entrance:
//! `docs/requirements/<requirement-id>.md` -> `.agentflow/spec/**`.
//! It does not execute tasks and does not update runtime state.

pub mod model;
pub mod storage;

pub use model::{
    PublicDeliveryRecord, RequirementDocument, SpecExpectedOutputs, SpecIssue, SpecIssueCategory,
    SpecIssueDraft, SpecIssueStatus, SpecPriority, SpecProject, SpecProjectDraft,
    SpecProjectStatus, SpecRequiredAgentRole, SpecSystemRecord, DEFAULT_WORKFLOW_REF,
};
pub use storage::{
    issue_from_requirement, list_spec_issues, list_spec_projects, prepare_spec_workspace,
    project_from_requirement, read_spec_issue, read_spec_project, write_spec_issue,
    write_spec_project, SpecWorkspaceSummary,
};
