//! AgentFlow runtime role policy registry.
//!
//! This crate defines the machine-readable capability boundary for product and
//! runtime roles. It does not arbitrate actions, append events, rebuild
//! projections, or launch providers.

pub mod core;
pub mod model;
pub mod registry;
pub mod report;
pub mod validation;

pub use core::{core_role_policy_bundle, core_role_policy_registry, CORE_ROLE_POLICY_REF};
pub use model::{
    AgentRolePolicy, AgentRolePolicyBundle, AgentRolePolicyStatus, ApprovalGate, HandoffRule,
    ObjectScope, ObjectScopeKind, ProductAgentRole, ProductRoleBinding, RoleActionCapability,
    RoleActionMatrixEntry, RoleAliasBinding, RoleCapabilityMode, RoleEvidenceMatrixEntry,
    RoleObjectMatrixEntry, RolePolicyCompatibility, RoleToolScope, RuntimeAgentRole, ToolKind,
};
pub use registry::RolePolicyRegistry;
pub use report::{RoleCapabilityDecision, RolePolicyValidationError, RolePolicyValidationReport};
pub use validation::validate_role_policy_bundle;
