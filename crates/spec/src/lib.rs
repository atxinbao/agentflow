//! AgentFlow spec contract layer.
//!
//! This crate owns the new task-contract entrance:
//! `docs/requirements/<requirement-id>.md` -> `.agentflow/spec/**`.
//! It does not execute tasks and does not update runtime state.

pub mod model;
pub mod storage;

pub use model::{
    CompletionDecisionFacts, CompletionDecisionOutcome, CompletionDecisionRecord,
    CompletionDecisionRuntime, CompletionDecisionState, GoalDraftPreview, GoalDraftStatus,
    IssueContractDraftPreview, MilestoneDraftPreview, PlanDraftPreview, PlanDraftStatus,
    PreviewConfirmationRecord, ProjectBrainDocumentSet, ProjectBrainDocumentStatus,
    ProjectBrainSnapshot, ProjectBrainStatus, PublicDeliveryRecord, RequirementDocument,
    RequirementIntakeResult, RequirementIntentType, RequirementPreviewLifecycle,
    RequirementPreviewRuntime, SpecExpectedOutputs, SpecIssue, SpecIssueCategory, SpecIssueDraft,
    SpecIssueStatus, SpecPriority, SpecProject, SpecProjectDraft, SpecProjectStatus,
    SpecRequiredAgentRole, SpecSystemRecord, DEFAULT_WORKFLOW_REF,
};
pub use storage::{
    cancel_requirement_preview, confirm_goal_draft_preview, confirm_plan_draft_preview,
    issue_from_requirement, list_completion_decision_runtimes, list_requirement_preview_runtimes,
    list_spec_issues, list_spec_projects, materialize_spec_from_requirement_preview,
    prepare_spec_workspace, project_from_requirement, read_completion_decision_runtime,
    read_project_brain_document_set, read_project_brain_snapshot, read_requirement_preview_runtime,
    read_spec_issue, read_spec_project, record_completion_decision,
    requirement_preview_from_requirement, sync_completion_decision_runtimes,
    update_spec_issue_status, write_completion_decision_runtime, write_requirement_preview_runtime,
    write_spec_issue, write_spec_project, SpecWorkspaceSummary,
};
