//! AgentFlow agent dispatcher.
//!
//! This crate consumes `agent.launch.requested` events and delegates provider
//! session creation to `agentflow-mcp`. It does not decide task state.

pub mod dispatcher;
pub mod model;

pub use dispatcher::AgentDispatcher;
pub use model::{
    AgentDispatchProviderSelection, AgentDispatchRoleBinding, AgentDispatchSelectionStatus,
    AgentDispatcherClaim, AGENT_SESSION_CREATED, AGENT_SESSION_DONE, AGENT_SESSION_FAILED,
    AGENT_SESSION_IN_REVIEW, AGENT_SESSION_RUNNING,
};
