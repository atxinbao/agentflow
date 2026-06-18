//! AgentFlow workflow definition layer.
//!
//! This crate owns YAML workflow parsing and validation. It does not read issue
//! contracts, write runtime events, execute actions, or serve Desktop read
//! models.

pub mod identity;
pub mod model;
pub mod parser;
pub mod registry;
pub mod validation;

pub use identity::{
    canonicalize_project_root, join_relative_path, normalize_relative_path,
    normalize_relative_path_string, normalize_relative_to_root, validate_safe_local_id, IssueId,
    ProjectId, ReleaseId, RunId,
};
pub use model::{
    ActionDefinition, GuardDefinition, HandoffDefinition, StateDefinition, TransitionDefinition,
    WorkflowAgentRole, WorkflowDefinition, WorkflowFlowType, WorkflowHandoffMode, WorkflowMetadata,
    WorkflowSkillPack, WorkflowSpec, WorkflowStatePhase, AGENTFLOW_WORKFLOW_API_VERSION,
    TASK_WORKFLOW_KIND,
};
pub use parser::{
    load_workflow_yaml, parse_workflow_yaml, workflow_name_from_ref, workflow_path_for_ref,
};
pub use registry::{
    canonical_workflow, work_state_is_active, work_state_is_blocked, work_state_is_cancel,
    work_state_is_done, work_state_is_in_progress, work_state_is_in_review,
    work_state_is_ready_for_execution, work_state_is_terminal, work_state_is_todo,
    work_state_phase, WorkflowRegistry, AUDIT_STATE_BLOCKED, AUDIT_STATE_CANCEL,
    AUDIT_STATE_IN_PROGRESS, AUDIT_STATE_NEEDS_REPAIR, AUDIT_STATE_PASSED, AUDIT_STATE_PENDING,
    AUDIT_STATE_READY, DELIVERY_STATE_BLOCKED, DELIVERY_STATE_CANCEL, DELIVERY_STATE_IN_PROGRESS,
    DELIVERY_STATE_PENDING, DELIVERY_STATE_PUBLISHED, DELIVERY_STATE_READY,
    DELIVERY_STATE_RETURNED, WORK_STATE_BACKLOG, WORK_STATE_BLOCKED, WORK_STATE_CANCEL,
    WORK_STATE_DONE, WORK_STATE_IN_PROGRESS, WORK_STATE_IN_REVIEW, WORK_STATE_TODO,
};
pub use validation::{
    validate_workflow, validate_workflow_with_registry, WorkflowValidationReport,
};
