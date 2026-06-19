//! AgentFlow action contract registry and proposal validation.
//!
//! This crate defines what actions Runtime can understand structurally. It does
//! not decide permissions, append events, update projections, or start
//! provider sessions.

pub mod core;
pub mod model;
pub mod registry;
pub mod report;
pub mod validation;

pub use core::{
    core_action_contract_bundle, core_action_contract_registry, CORE_ACTION_CONTRACT_REF,
};
pub use model::{
    AcceptedRefKind, ActionApprovalHint, ActionCategory, ActionContract, ActionContractBundle,
    ActionDefinitionStatus, ActionEffect, ActionEffectKind, ActionExpectedEvent,
    ActionFieldDefinition, ActionFieldValueType, ActionIdempotencyPolicy, ActionInputSchema,
    ActionPrecondition, ActionPreconditionKind, ActionProposal, ActionRef, ActionSimulationHint,
    ActionSourceSurface, ActionTargetMode, ActionTypeDefinition, RequiredEvidenceDefinition,
};
pub use registry::ActionContractRegistry;
pub use report::{
    ActionContractValidationError, ActionContractValidationReport, ActionProposalValidationReport,
    ActionProposalValidationStatus,
};
pub use validation::{validate_action_contract_bundle, validate_action_proposal};
