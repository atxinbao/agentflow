//! AgentFlow runtime action arbitration and write gate.
//!
//! This crate is the single acceptance gate before a write intent can become an
//! accepted runtime action. It composes action contract validation, role
//! policy, object state validation, dependency checks, evidence checks, and
//! object lock rules. It does not append events, rebuild projections, or start
//! providers.

pub mod arbitrator;
pub mod locks;
pub mod model;
pub mod reasons;
pub mod report;

pub use arbitrator::{
    arbitrate_action, build_accepted_action, check_object_lock, ActionArbitrator,
};
pub use locks::{default_lock_kind_for_object, LockDecision};
pub use model::{
    AcceptedAction, ArbitrationContext, ArbitrationDecision, ArbitrationDecisionStatus,
    ArbitrationRequest, DefinitionVersions, DependencyFact, EvidenceFact, HumanDecisionRequest,
    HumanDecisionResponseKind, ObjectLock, ObjectLockKind, ObjectLockPlan, ObjectRefKey, StateFact,
    ACTION_ARBITRATION_VERSION,
};
pub use reasons::{RejectionReason, RejectionReasonCode};
pub use report::RejectionExplanation;
