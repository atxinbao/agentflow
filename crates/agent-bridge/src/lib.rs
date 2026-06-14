//! AgentFlow agent bridge.
//!
//! This crate consumes `agent.launch.requested` events and delegates provider
//! session creation to `agentflow-mcp`. It does not decide task state.

pub mod bridge;
pub mod model;

pub use bridge::AgentBridge;
pub use model::{
    AgentBridgeClaim, AGENT_SESSION_CREATED, AGENT_SESSION_DONE, AGENT_SESSION_FAILED,
    AGENT_SESSION_IN_REVIEW, AGENT_SESSION_RUNNING,
};
