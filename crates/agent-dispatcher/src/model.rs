use agentflow_workflow_core::{WorkflowAgentRole, WorkflowSkillPack};
use serde::{Deserialize, Serialize};

pub const AGENT_SESSION_CREATED: &str = "agent.session.created";
pub const AGENT_LAUNCH_CLAIMED: &str = "agent.launch.claimed";
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
    pub runtime_role: WorkflowAgentRole,
    pub skill_pack: Option<WorkflowSkillPack>,
    pub created_event_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentDispatchRoleBinding {
    pub requested_role: String,
    pub runtime_role: WorkflowAgentRole,
    pub skill_pack: Option<WorkflowSkillPack>,
    pub provider_role: String,
}

impl AgentDispatchRoleBinding {
    pub fn resolve(requested_role: impl Into<String>) -> anyhow::Result<Self> {
        let requested_role = requested_role.into();
        let runtime_role = WorkflowAgentRole::parse_alias(&requested_role).ok_or_else(|| {
            anyhow::anyhow!("unsupported agent role for dispatcher: {requested_role}")
        })?;
        Ok(Self {
            requested_role,
            runtime_role,
            skill_pack: runtime_role.default_skill_pack(),
            provider_role: runtime_role
                .provider_role_alias()
                .unwrap_or(runtime_role.as_str())
                .to_string(),
        })
    }
}
