//! AgentFlow workflow runtime.
//!
//! This crate advances workflow state by matching events to workflow
//! transitions, evaluating registered guards, executing registered actions, and
//! writing transition facts to the task event store.

pub mod model;
pub mod runtime;

pub use model::{
    ActionExecution, ActionOutcome, GuardCheck, GuardOutcome, RuntimeContext, RuntimeTransition,
    RuntimeTransitionResult,
};
pub use runtime::{
    apply_workflow_event, find_transition, ActionRegistry, GuardRegistry, StaticActionRegistry,
    StaticGuardRegistry,
};
