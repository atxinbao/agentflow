use serde::{Deserialize, Serialize};

pub const AGENT_SESSION_CREATED: &str = "agent.session.created";
pub const AGENT_SESSION_RUNNING: &str = "agent.session.running";
pub const AGENT_SESSION_INTERRUPTED: &str = "agent.session.interrupted";
pub const AGENT_SESSION_RESUMED: &str = "agent.session.resumed";
pub const AGENT_SESSION_IN_REVIEW: &str = "agent.session.in_review";
pub const AGENT_SESSION_DONE: &str = "agent.session.completed";
pub const AGENT_SESSION_FAILED: &str = "agent.session.failed";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentDispatcherClaim {
    pub issue_id: String,
    pub run_id: String,
    pub provider: String,
    pub session_id: String,
    pub session_status: String,
    pub created_event_id: String,
}
