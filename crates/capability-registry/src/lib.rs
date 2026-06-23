//! AgentFlow worker / tool capability registry.
//!
//! This crate builds a read-only capability view for Command Surface and
//! runtime foundation checks. It aggregates static worker contracts, role tool
//! scopes, and MCP provider status. It does not launch providers, run smoke
//! checks, manage authentication, write authority files, or append events.

use agentflow_mcp::{
    provider_capability_profile, McpConnectorBoundary, McpProviderKind, McpProviderSmokeArtifact,
    McpProviderSmokeOutcome, McpProviderStatus, McpProviderStatusCode,
};
use agentflow_pack::{ConnectorCommandBoundary, PackConnectorDefinition};
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_smoke: Option<ProviderSmokeStatus>,
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
pub struct ProviderSmokeStatus {
    pub outcome: String,
    pub reason: String,
    pub artifact_path: String,
    pub terminal_provider_state_projectable: bool,
    pub created_at: u64,
}

impl From<&McpProviderSmokeArtifact> for ProviderSmokeStatus {
    fn from(value: &McpProviderSmokeArtifact) -> Self {
        Self {
            outcome: value.outcome.as_str().to_string(),
            reason: value.reason.clone(),
            artifact_path: value.artifact_path.clone(),
            terminal_provider_state_projectable: value.terminal_provider_state_projectable,
            created_at: value.created_at,
        }
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackConnectorCommandDecision {
    pub pack_id: String,
    pub connector_id: String,
    pub action_id: String,
    pub worker_id: String,
    pub command_type: String,
    pub required_capability: String,
    pub enabled: bool,
    pub health: WorkerHealth,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disabled_reason: Option<String>,
}

pub fn build_capability_registry(provider_statuses: &[McpProviderStatus]) -> CapabilityRegistry {
    build_capability_registry_with_smoke(provider_statuses, &[])
}

pub fn build_capability_registry_with_smoke(
    provider_statuses: &[McpProviderStatus],
    provider_smoke_artifacts: &[McpProviderSmokeArtifact],
) -> CapabilityRegistry {
    let statuses = provider_statuses
        .iter()
        .map(|status| (status.provider.clone(), status))
        .collect::<BTreeMap<_, _>>();
    let smoke = provider_smoke_artifacts
        .iter()
        .map(|artifact| (artifact.provider.clone(), artifact))
        .collect::<BTreeMap<_, _>>();
    CapabilityRegistry {
        version: CAPABILITY_REGISTRY_VERSION.to_string(),
        workers: vec![
            provider_worker(
                McpProviderKind::Codex,
                statuses.get("codex").copied(),
                smoke.get("codex").copied(),
            ),
            provider_worker(
                McpProviderKind::ClaudeCode,
                statuses.get("claude").copied(),
                smoke.get("claude").copied(),
            ),
            local_shell_validator_worker(),
            git_provider_worker(),
            provider_worker(
                McpProviderKind::Github,
                statuses.get("github").copied(),
                smoke.get("github").copied(),
            ),
            provider_worker(
                McpProviderKind::BrowserPreview,
                statuses.get("browser-preview").copied(),
                smoke.get("browser-preview").copied(),
            ),
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

pub fn evaluate_pack_connector_commands(
    registry: &CapabilityRegistry,
    definition: &PackConnectorDefinition,
) -> Vec<PackConnectorCommandDecision> {
    definition
        .connectors
        .iter()
        .flat_map(|connector| {
            connector.supported_actions.iter().map(|action| {
                let worker_id = connector.provider_type.worker_id();
                let decision = evaluate_command(registry, worker_id, &action.required_capability);
                let boundary_reason = connector_boundary_reason(&connector.command_boundary);
                let disabled_reason = decision.disabled_reason.or(boundary_reason);
                PackConnectorCommandDecision {
                    pack_id: definition.pack_id.clone(),
                    connector_id: connector.connector_id.clone(),
                    action_id: action.action_id.clone(),
                    worker_id: worker_id.to_string(),
                    command_type: action.command_type.clone(),
                    required_capability: action.required_capability.clone(),
                    enabled: disabled_reason.is_none() && decision.enabled,
                    health: decision.health,
                    disabled_reason,
                }
            })
        })
        .collect()
}

fn connector_boundary_reason(boundary: &ConnectorCommandBoundary) -> Option<String> {
    if !boundary.runtime_command_required {
        return Some("connector action must go through Runtime Command Surface".to_string());
    }
    if boundary.authority_write || boundary.output_authority {
        return Some("connector output must not write AgentFlow authority".to_string());
    }
    None
}

fn provider_worker(
    kind: McpProviderKind,
    status: Option<&McpProviderStatus>,
    smoke: Option<&McpProviderSmokeArtifact>,
) -> WorkerRegistryEntry {
    let provider = kind.as_str();
    let profile = provider_capability_profile(provider)
        .expect("built-in provider kinds must have capability profiles");
    let mut health = status
        .map(|status| WorkerHealth::from(status.status.clone()))
        .unwrap_or(WorkerHealth::Unknown);
    if smoke
        .map(|artifact| artifact.outcome == McpProviderSmokeOutcome::Failed)
        .unwrap_or(false)
    {
        health = WorkerHealth::Failed;
    }
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
    let smoke_failed = smoke
        .map(|artifact| artifact.outcome == McpProviderSmokeOutcome::Failed)
        .unwrap_or(false);
    let capabilities = profile
        .required_capabilities
        .iter()
        .chain(profile.degraded_capabilities.iter())
        .map(|capability_id| {
            let available = !smoke_failed
                &&
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
                } else if smoke_failed {
                    Some(format!("provider {provider} smoke gate failed"))
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
        disabled_reason: smoke
            .filter(|artifact| artifact.outcome == McpProviderSmokeOutcome::Failed)
            .map(|artifact| artifact.reason.clone())
            .or_else(|| status.and_then(|status| status.errors.first().cloned())),
        provider_smoke: smoke.map(ProviderSmokeStatus::from),
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
        provider_smoke: None,
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
    use super::{
        build_capability_registry, build_capability_registry_with_smoke, evaluate_command,
        evaluate_pack_connector_commands, WorkerHealth,
    };
    use agentflow_mcp::{
        McpCapability, McpProviderKind, McpProviderSmokeArtifact, McpProviderSmokeOutcome,
        McpProviderStatus, McpProviderStatusCode, MCP_PROVIDER_SMOKE_ARTIFACT_VERSION,
    };
    use agentflow_pack::software_dev_connector_definition;

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
        assert!(worker_ids.contains(&"browser-preview"));
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

    #[test]
    fn provider_smoke_failure_disables_provider_command() {
        let mut status = McpProviderStatus::new(McpProviderKind::Codex, 1);
        status.status = McpProviderStatusCode::Ready;
        status.capabilities = vec![McpCapability::new("launch", true)];
        let smoke = McpProviderSmokeArtifact {
            version: MCP_PROVIDER_SMOKE_ARTIFACT_VERSION.to_string(),
            provider: "codex".to_string(),
            outcome: McpProviderSmokeOutcome::Failed,
            reason: "terminal provider state is not projectable".to_string(),
            health: status.clone(),
            launch_request_path: Some(".agentflow/tmp/provider-smoke-request.md".to_string()),
            session_id: Some("session-smoke".to_string()),
            session_snapshot_path: Some(
                ".agentflow/state/mcp/sessions/session-smoke.json".to_string(),
            ),
            session_snapshot_readable: true,
            terminal_status: None,
            terminal_provider_state_projectable: false,
            artifact_path: ".agentflow/state/mcp/provider-smoke/codex-1.json".to_string(),
            created_at: 1,
        };
        let registry = build_capability_registry_with_smoke(&[status], &[smoke]);
        let worker = registry.worker("codex").unwrap();
        let decision = evaluate_command(&registry, "codex", "launch");

        assert_eq!(worker.health, WorkerHealth::Failed);
        assert_eq!(
            worker
                .provider_smoke
                .as_ref()
                .map(|smoke| smoke.outcome.as_str()),
            Some("failed")
        );
        assert!(!decision.enabled);
        assert_eq!(
            decision.disabled_reason,
            Some("provider codex smoke gate failed".to_string())
        );
    }

    #[test]
    fn pack_connector_requirements_map_to_command_availability() {
        let mut github = McpProviderStatus::new(McpProviderKind::Github, 1);
        github.status = McpProviderStatusCode::Ready;
        github.authenticated = Some(true);
        github.capabilities = vec![
            McpCapability::new("repo.read", true),
            McpCapability::new("pull_request.create", true),
        ];
        let registry = build_capability_registry(&[github]);
        let pack = software_dev_connector_definition();
        let decisions = evaluate_pack_connector_commands(&registry, &pack);

        let pr_create = decisions
            .iter()
            .find(|decision| decision.action_id == "github.pull-request.create")
            .unwrap();
        assert!(pr_create.enabled);
        assert_eq!(pr_create.worker_id, "github");
        assert_eq!(pr_create.required_capability, "pull_request.create");
    }

    #[test]
    fn provider_smoke_status_disables_pack_connector_command() {
        let mut status = McpProviderStatus::new(McpProviderKind::Codex, 1);
        status.status = McpProviderStatusCode::Ready;
        status.capabilities = vec![
            McpCapability::new("launch", true),
            McpCapability::new("build_agent.complete", true),
        ];
        let smoke = McpProviderSmokeArtifact {
            version: MCP_PROVIDER_SMOKE_ARTIFACT_VERSION.to_string(),
            provider: "codex".to_string(),
            outcome: McpProviderSmokeOutcome::Failed,
            reason: "terminal provider state is not projectable".to_string(),
            health: status.clone(),
            launch_request_path: Some(".agentflow/tmp/provider-smoke-request.md".to_string()),
            session_id: Some("session-smoke".to_string()),
            session_snapshot_path: Some(
                ".agentflow/state/mcp/sessions/session-smoke.json".to_string(),
            ),
            session_snapshot_readable: true,
            terminal_status: None,
            terminal_provider_state_projectable: false,
            artifact_path: ".agentflow/state/mcp/provider-smoke/codex-1.json".to_string(),
            created_at: 1,
        };
        let registry = build_capability_registry_with_smoke(&[status], &[smoke]);
        let pack = software_dev_connector_definition();
        let decisions = evaluate_pack_connector_commands(&registry, &pack);

        let launch = decisions
            .iter()
            .find(|decision| decision.action_id == "codex.launch")
            .unwrap();
        assert!(!launch.enabled);
        assert_eq!(launch.health, WorkerHealth::Failed);
        assert_eq!(
            launch.disabled_reason,
            Some("provider codex smoke gate failed".to_string())
        );
    }
}
