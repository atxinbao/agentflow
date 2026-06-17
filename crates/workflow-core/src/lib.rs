//! AgentFlow workflow definition layer.
//!
//! This crate owns YAML workflow parsing and validation. It does not read issue
//! contracts, write runtime events, execute actions, or serve Desktop read
//! models.

pub mod model;
pub mod parser;
pub mod registry;
pub mod validation;

pub use model::{
    ActionDefinition, GuardDefinition, HandoffDefinition, StateDefinition, TransitionDefinition,
    WorkflowAgentRole, WorkflowDefinition, WorkflowHandoffMode, WorkflowMetadata,
    WorkflowSkillPack, WorkflowSpec, WorkflowStatePhase, AGENTFLOW_WORKFLOW_API_VERSION,
    TASK_WORKFLOW_KIND,
};
pub use parser::{
    load_workflow_yaml, parse_workflow_yaml, workflow_name_from_ref, workflow_path_for_ref,
};
pub use registry::WorkflowRegistry;
pub use validation::{
    validate_workflow, validate_workflow_with_registry, WorkflowValidationReport,
};
