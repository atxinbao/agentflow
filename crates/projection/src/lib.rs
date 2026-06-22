//! AgentFlow projection read model.
//!
//! This crate rebuilds task and project read models from `.agentflow/spec/**`
//! and `.agentflow/events/task-events.jsonl`. Desktop should read these
//! projections instead of runtime write layers.

pub mod model;
pub mod projector;
pub mod query;
pub mod storage;

pub use model::{
    CompletionDecisionIndex, CompletionDecisionIndexEntry, CompletionDecisionProjection,
    IssueStatusIndex, IssueStatusIndexEntry, ProjectBlockerSummary, ProjectBrainProjection,
    ProjectCompletionProjection, ProjectExternalReviewProjection, ProjectIssueLanes,
    ProjectProjection, ProjectReleaseProjection, ProjectionAcceptanceSubGateSummary,
    ProjectionAcceptanceSummary, ProjectionAcceptanceTraceabilitySummary, ProjectionAuditSummary,
    ProjectionDeliverySummary, ProjectionPhase, ProjectionPublicDelivery, ProjectionRuntimeSummary,
    ProjectionSessionSummary, ProjectionSummary, RequirementPreviewIndex,
    RequirementPreviewIndexEntry, RequirementPreviewProjection, SpecLoopActionProposalProjection,
    SpecLoopProjection, SpecLoopStageProjection, SpecLoopTraceabilityEdge, TaskProjection,
    TaskTimelineEvent, TaskTimelineItem, COMPLETION_DECISION_INDEX_VERSION,
    COMPLETION_DECISION_PROJECTION_VERSION, ISSUE_STATUS_INDEX_VERSION, PROJECT_PROJECTION_VERSION,
    REQUIREMENT_PREVIEW_INDEX_VERSION, REQUIREMENT_PREVIEW_PROJECTION_VERSION,
    SPEC_LOOP_PROJECTION_VERSION, TASK_PROJECTION_VERSION,
};
pub use projector::rebuild_projections;
pub use query::{
    get_audit_surface_view, get_delivery_package_view, get_project_home_view,
    get_requirement_intake_view, get_runtime_health_view, get_spec_loop_view,
    get_spec_preview_view, get_task_workbench_view, get_work_loop_run_view,
    get_work_loop_session_view, AuditSurfaceView, DeliveryPackageView, IssuePreviewItem,
    ProjectDependencyEdge, ProjectHomeView, ProjectRunSummary, ProjectionDefinitionVersions,
    ProjectionFreshness, RequirementIntakeView, RuntimeEventRow, RuntimeHealthView, SpecLoopView,
    SpecPreviewView, TaskWorkbenchView, ViewActionHint, WorkLoopEventView,
    WorkLoopEvidenceSummaryView, WorkLoopRunView, WorkLoopSessionView,
    PROJECTION_QUERY_SURFACE_VERSION,
};
pub use storage::{
    load_completion_decision_index, load_completion_decision_projection, load_issue_status_index,
    load_project_projection, load_requirement_preview_index, load_requirement_preview_projection,
    load_spec_loop_projection, load_task_projection, prepare_projection_workspace,
};
