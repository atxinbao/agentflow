//! AgentFlow worker / tool capability registry.
//!
//! This crate builds a read-only capability view for Command Surface and
//! runtime foundation checks. It aggregates static worker contracts, role tool
//! scopes, and MCP provider status. It does not launch providers, run smoke
//! checks, manage authentication, write authority files, or append events.

use agentflow_mcp::{
    provider_capability_profile, McpConnectorBoundary, McpProviderKind, McpProviderStatus,
    McpProviderStatusCode,
};
use agentflow_role_policy::ToolKind;
use agentflow_workflow_core::{WorkflowAgentRole, WorkflowSkillPack};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const CAPABILITY_REGISTRY_VERSION: &str = "agentflow-capability-registry.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WorkerKind {
    AgentProvider,
    Validator,
    Connector,
    RuntimeWorker,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WorkerHealth {
    Ready,
    Degraded,
    Unknown,
    Unavailable,
    Unauthenticated,
    PermissionDenied,
    Unsupported,
    Failed,
}

impl WorkerHealth {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Degraded => "degraded",
            Self::Unknown => "unknown",
            Self::Unavailable => "unavailable",
            Self::Unauthenticated => "unauthenticated",
            Self::PermissionDenied => "permission-denied",
            Self::Unsupported => "unsupported",
            Self::Failed => "failed",
        }
    }

    pub fn command_enabled(&self) -> bool {
        matches!(self, Self::Ready | Self::Degraded)
    }
}

impl From<McpProviderStatusCode> for WorkerHealth {
    fn from(value: McpProviderStatusCode) -> Self {
        match value {
            McpProviderStatusCode::Ready => Self::Ready,
            McpProviderStatusCode::Unavailable => Self::Unavailable,
            McpProviderStatusCode::Unauthenticated => Self::Unauthenticated,
            McpProviderStatusCode::PermissionDenied => Self::PermissionDenied,
            McpProviderStatusCode::Unsupported => Self::Unsupported,
            McpProviderStatusCode::Failed => Self::Failed,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CapabilityPolicy {
    Allowed,
    RequiresAuth,
    Disabled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerCapability {
    pub capability_id: String,
    pub label: String,
    pub command: String,
    pub required: bool,
    pub available: bool,
    pub requires_auth: bool,
    pub policy: CapabilityPolicy,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disabled_reason: Option<String>,
}

impl WorkerCapability {
    pub fn enabled(&self) -> bool {
        self.available && !matches!(self.policy, CapabilityPolicy::Disabled)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerRegistryEntry {
    pub worker_id: String,
    pub title: String,
    pub kind: WorkerKind,
    pub health: WorkerHealth,
    pub requires_auth: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disabled_reason: Option<String>,
    #[serde(default)]
    pub runtime_roles: Vec<String>,
    #[serde(default)]
    pub skill_packs: Vec<String>,
    #[serde(default)]
    pub tool_kinds: Vec<ToolKind>,
    #[serde(default)]
    pub capabilities: Vec<WorkerCapability>,
    pub boundary: WorkerBoundary,
}

impl WorkerRegistryEntry {
    pub fn capability_for_command(&self, command: &str) -> Option<&WorkerCapability> {
        self.capabilities
            .iter()
            .find(|capability| capability.command == command || capability.capability_id == command)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerBoundary {
    #[serde(default)]
    pub read_capabilities: Vec<String>,
    #[serde(default)]
    pub write_capabilities: Vec<String>,
    pub authority_write: bool,
    pub runtime_command_required: bool,
    #[serde(default)]
    pub output_channels: Vec<String>,
    pub failure_surface: String,
}

impl WorkerBoundary {
    pub fn connector(
        read_capabilities: Vec<String>,
        write_capabilities: Vec<String>,
        output_channels: Vec<String>,
    ) -> Self {
        Self {
            read_capabilities,
            write_capabilities,
            authority_write: false,
            runtime_command_required: true,
            output_channels,
            failure_surface: "capability-registry.disabled-reason".to_string(),
        }
    }

    pub fn runtime_worker(output_channels: Vec<String>) -> Self {
        Self {
            read_capabilities: Vec::new(),
            write_capabilities: Vec::new(),
            authority_write: false,
            runtime_command_required: true,
            output_channels,
            failure_surface: "capability-registry.disabled-reason".to_string(),
        }
    }
}

impl From<McpConnectorBoundary> for WorkerBoundary {
    fn from(value: McpConnectorBoundary) -> Self {
        Self {
            read_capabilities: value.read_capabilities,
            write_capabilities: value.write_capabilities,
            authority_write: value.authority_write,
            runtime_command_required: value.runtime_command_required,
            output_channels: value.output_channels,
            failure_surface: value.failure_surface,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CapabilityRegistry {
    pub version: String,
    pub workers: Vec<WorkerRegistryEntry>,
}

impl CapabilityRegistry {
    pub fn worker(&self, worker_id: &str) -> Option<&WorkerRegistryEntry> {
        self.workers
            .iter()
            .find(|worker| worker.worker_id == worker_id)
    }

    pub fn workers(&self) -> &[WorkerRegistryEntry] {
        &self.workers
    }

    pub fn commands(&self) -> Vec<CommandSurfaceDecision> {
        self.workers
            .iter()
            .flat_map(|worker| {
                worker
                    .capabilities
                    .iter()
                    .map(|capability| {
                        evaluate_command(self, &worker.worker_id, &capability.command)
                    })
                    .collect::<Vec<_>>()
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandSurfaceDecision {
    pub worker_id: String,
    pub command: String,
    pub enabled: bool,
    pub health: WorkerHealth,
    #[serde(default)]
    pub required_capabilities: Vec<String>,
    #[serde(default)]
    pub missing_capabilities: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disabled_reason: Option<String>,
}

pub fn build_capability_registry(provider_statuses: &[McpProviderStatus]) -> CapabilityRegistry {
    let statuses = provider_statuses
        .iter()
        .map(|status| (status.provider.clone(), status))
        .collect::<BTreeMap<_, _>>();
    CapabilityRegistry {
        version: CAPABILITY_REGISTRY_VERSION.to_string(),
        workers: vec![
            provider_worker(McpProviderKind::Codex, statuses.get("codex").copied()),
            provider_worker(McpProviderKind::ClaudeCode, statuses.get("claude").copied()),
            local_shell_validator_worker(),
            git_provider_worker(),
            provider_worker(McpProviderKind::Github, statuses.get("github").copied()),
            mcp_connector_worker(),
            audit_worker(),
        ],
    }
}

pub fn default_capability_registry() -> CapabilityRegistry {
    build_capability_registry(&[])
}

pub fn evaluate_command(
    registry: &CapabilityRegistry,
    worker_id: &str,
    command: &str,
) -> CommandSurfaceDecision {
    let Some(worker) = registry.worker(worker_id) else {
        return CommandSurfaceDecision {
            worker_id: worker_id.to_string(),
            command: command.to_string(),
            enabled: false,
            health: WorkerHealth::Unknown,
            required_capabilities: vec![command.to_string()],
            missing_capabilities: vec![command.to_string()],
            disabled_reason: Some(format!("worker {worker_id} is not registered")),
        };
    };
    let Some(capability) = worker.capability_for_command(command) else {
        return CommandSurfaceDecision {
            worker_id: worker_id.to_string(),
            command: command.to_string(),
            enabled: false,
            health: worker.health.clone(),
            required_capabilities: vec![command.to_string()],
            missing_capabilities: vec![command.to_string()],
            disabled_reason: Some(format!(
                "worker {worker_id} does not expose command {command}"
            )),
        };
    };

    let mut disabled_reason = capability.disabled_reason.clone();
    if disabled_reason.is_none() && !worker.health.command_enabled() {
        disabled_reason = Some(worker_disabled_reason(worker));
    }
    if disabled_reason.is_none() && !capability.enabled() {
        disabled_reason = Some(format!(
            "capability {} is not available",
            capability.capability_id
        ));
    }
    if disabled_reason.is_none() && capability.requires_auth && worker.requires_auth {
        disabled_reason = match worker.health {
            WorkerHealth::Unauthenticated => Some(format!(
                "worker {} requires authentication for {}",
                worker.worker_id, capability.command
            )),
            _ => None,
        };
    }
    let enabled =
        worker.health.command_enabled() && capability.enabled() && disabled_reason.is_none();
    CommandSurfaceDecision {
        worker_id: worker.worker_id.clone(),
        command: capability.command.clone(),
        enabled,
        health: worker.health.clone(),
        required_capabilities: vec![capability.capability_id.clone()],
        missing_capabilities: if enabled {
            Vec::new()
        } else {
            vec![capability.capability_id.clone()]
        },
        disabled_reason,
    }
}

fn provider_worker(
    kind: McpProviderKind,
    status: Option<&McpProviderStatus>,
) -> WorkerRegistryEntry {
    let provider = kind.as_str();
    let profile = provider_capability_profile(provider)
        .expect("built-in provider kinds must have capability profiles");
    let health = status
        .map(|status| WorkerHealth::from(status.status.clone()))
        .unwrap_or(WorkerHealth::Unknown);
    let requires_auth = matches!(kind, McpProviderKind::Github | McpProviderKind::Gitlab)
        || status.and_then(|status| status.authenticated).is_some();
    let available_capabilities = status
        .map(|status| {
            status
                .capabilities
                .iter()
                .filter(|capability| capability.available)
                .map(|capability| capability.name.clone())
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    let provider_known_but_unchecked = status.is_none();
    let capabilities = profile
        .required_capabilities
        .iter()
        .chain(profile.degraded_capabilities.iter())
        .map(|capability_id| {
            let available =
                !provider_known_but_unchecked && available_capabilities.contains(capability_id);
            capability(
                capability_id,
                capability_id,
                capability_id,
                profile.required_capabilities.contains(capability_id),
                available,
                requires_auth,
                if available {
                    CapabilityPolicy::Allowed
                } else if requires_auth {
                    CapabilityPolicy::RequiresAuth
                } else {
                    CapabilityPolicy::Disabled
                },
                if available {
                    None
                } else if provider_known_but_unchecked {
                    Some(format!("provider {provider} health has not been checked"))
                } else {
                    Some(format!(
                        "provider {provider} does not report capability {capability_id} as available"
                    ))
                },
            )
        })
        .collect::<Vec<_>>();

    WorkerRegistryEntry {
        worker_id: provider.to_string(),
        title: provider_title(&kind),
        kind: WorkerKind::AgentProvider,
        health,
        requires_auth,
        disabled_reason: status.and_then(|status| status.errors.first().cloned()),
        runtime_roles: profile
            .supported_roles
            .iter()
            .map(WorkflowAgentRole::as_str)
            .map(str::to_string)
            .collect(),
        skill_packs: profile
            .supported_skill_packs
            .iter()
            .map(WorkflowSkillPack::as_str)
            .map(str::to_string)
            .collect(),
        tool_kinds: provider_tool_kinds(&kind),
        capabilities,
        boundary: profile.connector_boundary.into(),
    }
}

fn local_shell_validator_worker() -> WorkerRegistryEntry {
    static_worker(
        "local-shell-validator",
        "Local Shell Validator",
        WorkerKind::Validator,
        vec![
            ToolKind::LocalBuild,
            ToolKind::LocalTest,
            ToolKind::BrowserSmoke,
            ToolKind::InspectDiff,
        ],
        vec![
            ("local.build", "Run local build", "validate.build"),
            ("local.test", "Run local tests", "validate.test"),
            (
                "browser.smoke",
                "Run browser smoke",
                "validate.browser-smoke",
            ),
            ("diff.inspect", "Inspect local diff", "validate.diff"),
        ],
        WorkerBoundary::runtime_worker(vec!["evidence".to_string()]),
    )
}

fn git_provider_worker() -> WorkerRegistryEntry {
    static_worker(
        "git-provider",
        "Git Provider",
        WorkerKind::Connector,
        vec![ToolKind::InspectDiff, ToolKind::InspectState],
        vec![
            ("git.status", "Read git status", "git.status"),
            ("git.branch", "Read current branch", "git.branch"),
            ("git.diff", "Read git diff", "git.diff"),
        ],
        WorkerBoundary::connector(
            vec![
                "git.status".to_string(),
                "git.branch".to_string(),
                "git.diff".to_string(),
            ],
            Vec::new(),
            vec!["context".to_string(), "external-fact".to_string()],
        ),
    )
}

fn mcp_connector_worker() -> WorkerRegistryEntry {
    static_worker(
        "mcp-connector",
        "MCP Connector",
        WorkerKind::Connector,
        vec![ToolKind::InspectState],
        vec![
            ("mcp.provider.list", "List MCP providers", "mcp.providers"),
            ("mcp.session.poll", "Poll MCP sessions", "mcp.sessions.poll"),
            (
                "mcp.session.logs",
                "Read MCP session logs",
                "mcp.sessions.logs",
            ),
        ],
        WorkerBoundary::connector(
            vec![
                "mcp.provider.list".to_string(),
                "mcp.session.poll".to_string(),
                "mcp.session.logs".to_string(),
            ],
            Vec::new(),
            vec![
                "context".to_string(),
                "evidence".to_string(),
                "external-fact".to_string(),
            ],
        ),
    )
}

fn audit_worker() -> WorkerRegistryEntry {
    static_worker(
        "audit-worker",
        "Audit Worker",
        WorkerKind::RuntimeWorker,
        vec![
            ToolKind::ReadEvidence,
            ToolKind::InspectState,
            ToolKind::GenerateReport,
        ],
        vec![
            ("audit.read", "Read audit state", "audit.read"),
            ("audit.report", "Generate audit report", "audit.report"),
            (
                "finding.propose",
                "Propose finding follow-up",
                "audit.finding.propose",
            ),
        ],
        WorkerBoundary::runtime_worker(vec!["evidence".to_string(), "external-fact".to_string()]),
    )
}

fn static_worker(
    worker_id: &str,
    title: &str,
    kind: WorkerKind,
    tool_kinds: Vec<ToolKind>,
    commands: Vec<(&str, &str, &str)>,
    boundary: WorkerBoundary,
) -> WorkerRegistryEntry {
    WorkerRegistryEntry {
        worker_id: worker_id.to_string(),
        title: title.to_string(),
        kind,
        health: WorkerHealth::Ready,
        requires_auth: false,
        disabled_reason: None,
        runtime_roles: Vec::new(),
        skill_packs: Vec::new(),
        tool_kinds,
        capabilities: commands
            .into_iter()
            .map(|(capability_id, label, command)| {
                capability(
                    capability_id,
                    label,
                    command,
                    true,
                    true,
                    false,
                    CapabilityPolicy::Allowed,
                    None,
                )
            })
            .collect(),
        boundary,
    }
}

fn capability(
    capability_id: &str,
    label: &str,
    command: &str,
    required: bool,
    available: bool,
    requires_auth: bool,
    policy: CapabilityPolicy,
    disabled_reason: Option<String>,
) -> WorkerCapability {
    WorkerCapability {
        capability_id: capability_id.to_string(),
        label: label.to_string(),
        command: command.to_string(),
        required,
        available,
        requires_auth,
        policy,
        disabled_reason,
    }
}

fn provider_title(kind: &McpProviderKind) -> String {
    match kind {
        McpProviderKind::Github => "GitHub Connector".to_string(),
        McpProviderKind::Gitlab => "GitLab Connector".to_string(),
        McpProviderKind::Codex => "Codex Provider".to_string(),
        McpProviderKind::ClaudeCode => "Claude Provider".to_string(),
        McpProviderKind::BrowserPreview => "Browser Preview Provider".to_string(),
    }
}

fn provider_tool_kinds(kind: &McpProviderKind) -> Vec<ToolKind> {
    match kind {
        McpProviderKind::Codex | McpProviderKind::ClaudeCode => vec![
            ToolKind::Filesystem,
            ToolKind::LocalBuild,
            ToolKind::LocalTest,
            ToolKind::InspectDiff,
        ],
        McpProviderKind::Github | McpProviderKind::Gitlab => {
            vec![ToolKind::InspectState, ToolKind::GenerateReport]
        }
        McpProviderKind::BrowserPreview => vec![ToolKind::BrowserSmoke],
    }
}

fn worker_disabled_reason(worker: &WorkerRegistryEntry) -> String {
    match worker.health {
        WorkerHealth::Unknown => format!("worker {} health has not been checked", worker.worker_id),
        WorkerHealth::Unauthenticated => {
            format!("worker {} requires authentication", worker.worker_id)
        }
        _ => format!(
            "worker {} is not ready: {}",
            worker.worker_id,
            worker.health.as_str()
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::{build_capability_registry, evaluate_command, WorkerHealth};
    use agentflow_mcp::{McpCapability, McpProviderKind, McpProviderStatus, McpProviderStatusCode};

    #[test]
    fn registry_lists_required_workers() {
        let registry = build_capability_registry(&[]);
        let worker_ids = registry
            .workers()
            .iter()
            .map(|worker| worker.worker_id.as_str())
            .collect::<Vec<_>>();

        assert!(worker_ids.contains(&"codex"));
        assert!(worker_ids.contains(&"claude"));
        assert!(worker_ids.contains(&"local-shell-validator"));
        assert!(worker_ids.contains(&"git-provider"));
        assert!(worker_ids.contains(&"github"));
        assert!(worker_ids.contains(&"mcp-connector"));
        assert!(worker_ids.contains(&"audit-worker"));
    }

    #[test]
    fn connector_boundaries_do_not_grant_authority_writes() {
        let registry = build_capability_registry(&[]);
        let git = registry.worker("git-provider").unwrap();
        let mcp = registry.worker("mcp-connector").unwrap();
        let github = registry.worker("github").unwrap();

        assert_eq!(git.kind, super::WorkerKind::Connector);
        assert!(!git.boundary.authority_write);
        assert!(git.boundary.runtime_command_required);
        assert!(git.boundary.write_capabilities.is_empty());
        assert!(git
            .boundary
            .output_channels
            .contains(&"external-fact".to_string()));

        assert!(!mcp.boundary.authority_write);
        assert!(mcp
            .boundary
            .read_capabilities
            .contains(&"mcp.session.poll".to_string()));

        assert!(!github.boundary.authority_write);
        assert!(github.boundary.runtime_command_required);
        assert!(github
            .boundary
            .write_capabilities
            .contains(&"pull_request.create".to_string()));
    }

    #[test]
    fn unchecked_provider_command_is_disabled_with_reason() {
        let registry = build_capability_registry(&[]);
        let decision = evaluate_command(&registry, "codex", "launch");

        assert!(!decision.enabled);
        assert_eq!(decision.health, WorkerHealth::Unknown);
        assert_eq!(decision.missing_capabilities, vec!["launch".to_string()]);
        assert_eq!(
            decision.disabled_reason,
            Some("provider codex health has not been checked".to_string())
        );
    }

    #[test]
    fn ready_provider_enables_reported_capability() {
        let mut status = McpProviderStatus::new(McpProviderKind::Codex, 1);
        status.status = McpProviderStatusCode::Ready;
        status.capabilities = vec![
            McpCapability::new("launch", true),
            McpCapability::new("codex.exec", true),
            McpCapability::new("session.poll", true),
            McpCapability::new("session.logs", true),
            McpCapability::new("session.cancel", true),
            McpCapability::new("build_agent.complete", true),
        ];
        let registry = build_capability_registry(&[status]);
        let decision = evaluate_command(&registry, "codex", "launch");

        assert!(decision.enabled);
        assert_eq!(decision.health, WorkerHealth::Ready);
        assert!(decision.missing_capabilities.is_empty());
        assert_eq!(decision.disabled_reason, None);
    }

    #[test]
    fn unauthenticated_connector_exposes_disabled_reason() {
        let mut status = McpProviderStatus::new(McpProviderKind::Github, 1);
        status.status = McpProviderStatusCode::Unauthenticated;
        status.authenticated = Some(false);
        status.capabilities = vec![McpCapability::new("repo.read", false)];
        let registry = build_capability_registry(&[status]);
        let decision = evaluate_command(&registry, "github", "repo.read");

        assert!(!decision.enabled);
        assert_eq!(decision.health, WorkerHealth::Unauthenticated);
        assert_eq!(
            decision.disabled_reason,
            Some("provider github does not report capability repo.read as available".to_string())
        );
    }

    #[test]
    fn static_local_validator_command_is_available() {
        let registry = build_capability_registry(&[]);
        let decision = evaluate_command(&registry, "local-shell-validator", "validate.test");

        assert!(decision.enabled);
        assert_eq!(decision.health, WorkerHealth::Ready);
        assert_eq!(decision.disabled_reason, None);
    }
}
