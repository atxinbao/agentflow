use agentflow_mcp::{provider_capability_profile, McpProviderStatus, McpProviderStatusCode};
use agentflow_workflow_core::{WorkflowAgentRole, WorkflowSkillPack};
use anyhow::Result;
use serde::{Deserialize, Serialize};

pub const AGENT_SESSION_CREATED: &str = "agent.session.created";
pub const AGENT_LAUNCH_CLAIMED: &str = "agent.launch.claimed";
pub const AGENT_SESSION_RUNNING: &str = "agent.session.running";
pub const AGENT_SESSION_INTERRUPTED: &str = "agent.session.interrupted";
pub const AGENT_SESSION_RESUMED: &str = "agent.session.resumed";
pub const AGENT_SESSION_IN_REVIEW: &str = "agent.session.in_review";
pub const AGENT_SESSION_DONE: &str = "agent.session.completed";
pub const AGENT_SESSION_FAILED: &str = "agent.session.failed";
pub const AGENT_SESSION_CANCELLED: &str = "agent.session.cancelled";

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
    pub selection: AgentDispatchProviderSelection,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AgentDispatchSelectionStatus {
    Ready,
    Degraded,
    Unsupported,
}

impl AgentDispatchSelectionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Degraded => "degraded",
            Self::Unsupported => "unsupported",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentDispatchProviderSelection {
    pub requested_provider: String,
    pub provider_kind: Option<String>,
    pub provider_status: Option<String>,
    pub requested_role: String,
    pub runtime_role: WorkflowAgentRole,
    pub skill_pack: Option<WorkflowSkillPack>,
    pub provider_role: String,
    pub supported_roles: Vec<String>,
    pub supported_skill_packs: Vec<String>,
    pub required_capabilities: Vec<String>,
    pub degraded_capabilities: Vec<String>,
    pub missing_required_capabilities: Vec<String>,
    pub missing_degraded_capabilities: Vec<String>,
    pub status: AgentDispatchSelectionStatus,
    pub selection_reason: String,
    pub degradation_reason: Option<String>,
}

impl AgentDispatchProviderSelection {
    pub fn evaluate(
        requested_provider: impl Into<String>,
        provider_status: Option<&McpProviderStatus>,
        role_binding: &AgentDispatchRoleBinding,
    ) -> Self {
        let requested_provider = requested_provider.into();
        let profile = provider_status
            .and_then(McpProviderStatus::capability_profile)
            .or_else(|| provider_capability_profile(&requested_provider));

        let Some(profile) = profile else {
            return Self {
                requested_provider: requested_provider.clone(),
                provider_kind: None,
                provider_status: None,
                requested_role: role_binding.requested_role.clone(),
                runtime_role: role_binding.runtime_role,
                skill_pack: role_binding.skill_pack,
                provider_role: role_binding.provider_role.clone(),
                supported_roles: Vec::new(),
                supported_skill_packs: Vec::new(),
                required_capabilities: Vec::new(),
                degraded_capabilities: Vec::new(),
                missing_required_capabilities: Vec::new(),
                missing_degraded_capabilities: Vec::new(),
                status: AgentDispatchSelectionStatus::Unsupported,
                selection_reason: format!(
                    "provider {} is unknown to the capability matrix",
                    requested_provider
                ),
                degradation_reason: None,
            };
        };

        let supported_roles = profile
            .supported_roles
            .iter()
            .map(|role| role.as_str().to_string())
            .collect::<Vec<_>>();
        let supported_skill_packs = profile
            .supported_skill_packs
            .iter()
            .map(|skill_pack| skill_pack.as_str().to_string())
            .collect::<Vec<_>>();
        let provider_status_code = provider_status.map(|status| status.status.as_str().to_string());
        let mut selection = Self {
            requested_provider: requested_provider.clone(),
            provider_kind: Some(profile.kind.as_str().to_string()),
            provider_status: provider_status_code,
            requested_role: role_binding.requested_role.clone(),
            runtime_role: role_binding.runtime_role,
            skill_pack: role_binding.skill_pack,
            provider_role: role_binding.provider_role.clone(),
            supported_roles,
            supported_skill_packs,
            required_capabilities: profile.required_capabilities.clone(),
            degraded_capabilities: profile.degraded_capabilities.clone(),
            missing_required_capabilities: Vec::new(),
            missing_degraded_capabilities: Vec::new(),
            status: AgentDispatchSelectionStatus::Ready,
            selection_reason: String::new(),
            degradation_reason: None,
        };

        if !profile.supports_role(role_binding.runtime_role) {
            selection.status = AgentDispatchSelectionStatus::Unsupported;
            selection.selection_reason = format!(
                "provider {} does not support runtime role {}",
                requested_provider,
                role_binding.runtime_role.as_str()
            );
            return selection;
        }

        if let Some(skill_pack) = role_binding.skill_pack {
            if !profile.supports_skill_pack(skill_pack) {
                selection.status = AgentDispatchSelectionStatus::Unsupported;
                selection.selection_reason = format!(
                    "provider {} does not support skill pack {}",
                    requested_provider,
                    skill_pack.as_str()
                );
                return selection;
            }
        }

        let Some(status) = provider_status else {
            selection.status = AgentDispatchSelectionStatus::Unsupported;
            selection.selection_reason = format!(
                "provider {} is known but not registered for dispatcher launch",
                requested_provider
            );
            return selection;
        };

        if !matches!(status.status, McpProviderStatusCode::Ready) {
            selection.status = AgentDispatchSelectionStatus::Unsupported;
            selection.selection_reason = format!(
                "provider {} is not launch-ready: {}",
                requested_provider,
                status.status.as_str()
            );
            return selection;
        }

        selection.missing_required_capabilities = selection
            .required_capabilities
            .iter()
            .filter(|capability| !status.capability_available(capability))
            .cloned()
            .collect();
        selection.missing_degraded_capabilities = selection
            .degraded_capabilities
            .iter()
            .filter(|capability| !status.capability_available(capability))
            .cloned()
            .collect();

        if !selection.missing_required_capabilities.is_empty() {
            selection.status = AgentDispatchSelectionStatus::Unsupported;
            selection.selection_reason = format!(
                "provider {} is missing required capabilities: {}",
                requested_provider,
                selection.missing_required_capabilities.join(", ")
            );
            return selection;
        }

        if !selection.missing_degraded_capabilities.is_empty() {
            selection.status = AgentDispatchSelectionStatus::Degraded;
            selection.selection_reason = format!(
                "provider {} supports {} with degraded capabilities",
                requested_provider,
                role_binding.runtime_role.as_str()
            );
            selection.degradation_reason = Some(format!(
                "missing degraded capabilities: {}",
                selection.missing_degraded_capabilities.join(", ")
            ));
            return selection;
        }

        selection.selection_reason = format!(
            "provider {} supports runtime role {} and required capabilities are ready",
            requested_provider,
            role_binding.runtime_role.as_str()
        );
        selection
    }

    pub fn ensure_runnable(&self) -> Result<()> {
        if !matches!(self.status, AgentDispatchSelectionStatus::Ready) {
            anyhow::bail!("{}", self.selection_reason);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AgentDispatchProviderSelection, AgentDispatchRoleBinding, AgentDispatchSelectionStatus,
    };
    use agentflow_mcp::{McpCapability, McpProviderKind, McpProviderStatus, McpProviderStatusCode};

    #[test]
    fn selection_marks_codex_without_runtime_closeout_as_unsupported() {
        let binding = AgentDispatchRoleBinding::resolve("build-agent").unwrap();
        let mut status = McpProviderStatus::new(McpProviderKind::Codex, 1);
        status.status = McpProviderStatusCode::Ready;
        status.capabilities = vec![
            McpCapability::new("launch", true),
            McpCapability::new("codex.exec", true),
            McpCapability::new("session.poll", true),
            McpCapability::new("session.logs", true),
            McpCapability::new("session.cancel", true),
            McpCapability::new("build_agent.complete", false),
        ];
        let selection = AgentDispatchProviderSelection::evaluate("codex", Some(&status), &binding);

        assert_eq!(selection.status, AgentDispatchSelectionStatus::Unsupported);
        assert_eq!(
            selection.missing_required_capabilities,
            vec!["build_agent.complete".to_string()]
        );
    }

    #[test]
    fn selection_rejects_provider_role_mismatch() {
        let binding = AgentDispatchRoleBinding::resolve("build-agent").unwrap();
        let mut status = McpProviderStatus::new(McpProviderKind::Github, 1);
        status.status = McpProviderStatusCode::Ready;
        status.capabilities = vec![
            McpCapability::new("repo.read", true),
            McpCapability::new("pull_request.create", true),
        ];
        let selection = AgentDispatchProviderSelection::evaluate("github", Some(&status), &binding);

        assert_eq!(selection.status, AgentDispatchSelectionStatus::Unsupported);
        assert!(selection
            .selection_reason
            .contains("does not support runtime role work-agent"));
    }
}
