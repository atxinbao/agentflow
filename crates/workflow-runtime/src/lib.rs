//! AgentFlow workflow runtime.
//!
//! This crate advances workflow state by matching events to workflow
//! transitions, evaluating registered guards, executing registered actions, and
//! writing transition facts to the task event store.

pub mod locks;
pub mod model;
pub mod records;
pub mod runtime;
pub mod storage;

pub use agentflow_workflow_core::WorkflowFlowType;
pub use locks::{load_runtime_lock_snapshot, RuntimeLockSnapshot, RuntimeObjectLockRecord};
pub use model::{
    ActionExecution, ActionOutcome, GuardCheck, GuardOutcome, RuntimeContext,
    RuntimeHandoffBinding, RuntimeStateBinding, RuntimeTransition, RuntimeTransitionResult,
};
pub use records::{
    RuntimeAcceptedActionFact, RuntimeCommandFact, RuntimeCommandFactBundle,
    RuntimeCommandValidationFact, RuntimeDecisionFact, RuntimeProposalFact, RuntimeQueryHintFact,
    RUNTIME_ACCEPTED_ACTION_FACT_VERSION, RUNTIME_COMMAND_FACT_VERSION,
    RUNTIME_DECISION_FACT_VERSION, RUNTIME_PROPOSAL_FACT_VERSION,
};
pub use runtime::{
    apply_canonical_workflow_event, apply_workflow_event, find_transition, resolve_state_binding,
    resolve_transition_handoff, ActionRegistry, GuardRegistry, StaticActionRegistry,
    StaticGuardRegistry,
};
pub use storage::{
    load_runtime_accepted_action_fact, load_runtime_accepted_action_facts,
    load_runtime_command_bundle, load_runtime_command_fact, load_runtime_decision_fact,
    load_runtime_decision_facts, load_runtime_proposal_fact, load_runtime_proposal_facts,
    prepare_runtime_workspace, runtime_accepted_action_fact_path, runtime_command_fact_path,
    runtime_decision_fact_path, runtime_proposal_fact_path, write_runtime_accepted_action_fact,
    write_runtime_command_fact, write_runtime_decision_fact, write_runtime_proposal_fact,
};
