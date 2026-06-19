//! AgentFlow object state machine registry and lifecycle validation.
//!
//! This crate owns object lifecycle definitions for Runtime. It does not append
//! events, arbitrate live writes, rebuild projections, or execute agents.

pub mod core;
pub mod model;
pub mod registry;
pub mod report;
pub mod validation;

pub use core::{core_object_state_bundle, core_object_state_registry, CORE_OBJECT_STATE_REF};
pub use model::{
    ObjectStateDefinition, ObjectStateMachine, ObjectStateMachineBundle, ObjectStateMachineStatus,
    StateProjectionHints, StateTransitionDefinition, OBJECT_STATE_BUNDLE_VERSION,
};
pub use registry::ObjectStateMachineRegistry;
pub use report::{
    ObjectStateValidationError, ObjectStateValidationReport, ObjectStateValidationWarning,
    TransitionDecision,
};
pub use validation::validate_object_state_bundle;
