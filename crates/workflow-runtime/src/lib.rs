//! AgentFlow workflow runtime.
//!
//! This crate advances workflow state by matching events to workflow
//! transitions, evaluating registered guards, executing registered actions, and
//! writing transition facts to the task event store.

pub mod model;
pub mod runtime;

pub use agentflow_workflow_core::WorkflowFlowType;
pub use model::{
    ActionExecution, ActionOutcome, GuardCheck, GuardOutcome, RuntimeContext,
    RuntimeHandoffBinding, RuntimeStateBinding, RuntimeTransition, RuntimeTransitionResult,
};
pub use runtime::{
    apply_canonical_workflow_event, apply_workflow_event, find_transition, resolve_state_binding,
    resolve_transition_handoff, ActionRegistry, GuardRegistry, StaticActionRegistry,
    StaticGuardRegistry,
};
