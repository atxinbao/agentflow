use agentflow_workflow_core::{WorkflowAgentRole, WorkflowSkillPack};
use serde::{Deserialize, Serialize};

pub const MCP_PROVIDER_STATUS_VERSION: &str = "agentflow-mcp-provider.v1";
pub const MCP_REGISTRY_VERSION: &str = "agentflow-mcp-registry.v1";
pub const MCP_LAUNCH_REQUEST_VERSION: &str = "agentflow-mcp-launch-request.v1";
pub const MCP_LAUNCH_PLAN_VERSION: &str = "agentflow-mcp-launch-plan.v1";
pub const MCP_SESSION_SNAPSHOT_VERSION: &str = "agentflow-mcp-session.v1";
pub const MCP_SESSION_EXIT_PROOF_VERSION: &str = "agentflow-mcp-exit-proof.v1";
pub const MCP_LOG_CHUNK_VERSION: &str = "agentflow-mcp-log-chunk.v1";
pub const MCP_PROVIDER_CAPABILITY_PROFILE_VERSION: &str = "agentflow-mcp-capability-profile.v1";
pub const MCP_SESSION_GOVERNANCE_POLICY_VERSION: &str = "agentflow-mcp-session-policy.v1";
pub const MCP_CLOSEOUT_ATTESTATION_VERSION: &str = "agentflow-mcp-closeout-attestation.v1";
pub const MCP_DEFAULT_SESSION_TIMEOUT_SECONDS: u64 = 60 * 60;
pub const MCP_DEFAULT_MAX_ATTEMPTS: u32 = 3;

fn default_attempt_count() -> u32 {
    1
}

fn default_session_timeout_seconds() -> u64 {
    MCP_DEFAULT_SESSION_TIMEOUT_SECONDS
}

fn default_max_attempts() -> u32 {
    MCP_DEFAULT_MAX_ATTEMPTS
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpSessionGovernancePolicy {
    pub version: String,
    pub claim_policy: String,
    pub timeout_policy: String,
    #[serde(default = "default_session_timeout_seconds")]
    pub timeout_seconds: u64,
    pub takeover_policy: String,
    pub retry_policy: String,
    #[serde(default = "default_max_attempts")]
    pub max_attempts: u32,
    pub cancel_policy: String,
}

impl Default for McpSessionGovernancePolicy {
    fn default() -> Self {
        Self {
            version: MCP_SESSION_GOVERNANCE_POLICY_VERSION.to_string(),
            claim_policy: "single-active-session-per-run".to_string(),
            timeout_policy: "interrupt-and-recover".to_string(),
            timeout_seconds: MCP_DEFAULT_SESSION_TIMEOUT_SECONDS,
            takeover_policy: "resume-interrupted-or-failed-attempt".to_string(),
            retry_policy: "bounded-retry".to_string(),
            max_attempts: MCP_DEFAULT_MAX_ATTEMPTS,
            cancel_policy: "terminal-for-current-run".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpSessionGovernanceFacts {
    pub timeout_at: Option<u64>,
    pub timed_out_at: Option<u64>,
    pub cancel_requested_at: Option<u64>,
    pub cancelled_at: Option<u64>,
    pub resumed_from_attempt: Option<u32>,
    pub takeover_session_id: Option<String>,
    pub terminal_reason: Option<String>,
    pub retryable: bool,
}

impl Default for McpSessionGovernanceFacts {
    fn default() -> Self {
        Self {
            timeout_at: None,
            timed_out_at: None,
            cancel_requested_at: None,
            cancelled_at: None,
            resumed_from_attempt: None,
            takeover_session_id: None,
            terminal_reason: None,
            retryable: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum McpProviderKind {
    Github,
    Gitlab,
    Codex,
    ClaudeCode,
    BrowserPreview,
}

impl McpProviderKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Github => "github",
            Self::Gitlab => "gitlab",
            Self::Codex => "codex",
            Self::ClaudeCode => "claude",
            Self::BrowserPreview => "browser-preview",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim() {
            "github" => Some(Self::Github),
            "gitlab" => Some(Self::Gitlab),
            "codex" => Some(Self::Codex),
            "claude" => Some(Self::ClaudeCode),
            "browser-preview" => Some(Self::BrowserPreview),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum McpProviderStatusCode {
    Ready,
    Unavailable,
    Unauthenticated,
    PermissionDenied,
    Unsupported,
    Failed,
}

impl McpProviderStatusCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Unavailable => "unavailable",
            Self::Unauthenticated => "unauthenticated",
            Self::PermissionDenied => "permission-denied",
            Self::Unsupported => "unsupported",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpCapability {
    pub name: String,
    pub available: bool,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpCloseoutIssueAttestation {
    pub issue_ref: String,
    pub issue_url: Option<String>,
    pub closed: bool,
    pub closed_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpCloseoutAttestation {
    pub version: String,
    pub provider: String,
    pub review_ref: String,
    pub review_url: Option<String>,
    pub repository_full_name: Option<String>,
    pub source_branch: Option<String>,
    pub target_branch: Option<String>,
    pub base_sha: Option<String>,
    pub head_sha: Option<String>,
    pub merge_commit_sha: Option<String>,
    pub merged: bool,
    pub merged_at: Option<String>,
    pub issue_closed: bool,
    pub issues: Vec<McpCloseoutIssueAttestation>,
    pub queried_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpProviderCapabilityProfile {
    pub version: String,
    pub provider: String,
    pub kind: McpProviderKind,
    pub supported_roles: Vec<WorkflowAgentRole>,
    pub supported_skill_packs: Vec<WorkflowSkillPack>,
    pub required_capabilities: Vec<String>,
    pub degraded_capabilities: Vec<String>,
}

impl McpProviderCapabilityProfile {
    pub fn supports_role(&self, role: WorkflowAgentRole) -> bool {
        self.supported_roles.contains(&role)
    }

    pub fn supports_skill_pack(&self, skill_pack: WorkflowSkillPack) -> bool {
        self.supported_skill_packs.contains(&skill_pack)
    }
}

pub fn provider_capability_profile(provider: &str) -> Option<McpProviderCapabilityProfile> {
    let kind = McpProviderKind::parse(provider)?;
    Some(match kind {
        McpProviderKind::Github => McpProviderCapabilityProfile {
            version: MCP_PROVIDER_CAPABILITY_PROFILE_VERSION.to_string(),
            provider: provider.to_string(),
            kind,
            supported_roles: vec![WorkflowAgentRole::DeliveryAgent],
            supported_skill_packs: vec![WorkflowSkillPack::DeliverySkills],
            required_capabilities: vec!["repo.read".to_string(), "pull_request.create".to_string()],
            degraded_capabilities: vec![
                "pull_request.ready".to_string(),
                "pull_request.auto_merge".to_string(),
                "pull_request.merged_query".to_string(),
                "issue.close".to_string(),
                "issue.closed_query".to_string(),
            ],
        },
        McpProviderKind::Gitlab => McpProviderCapabilityProfile {
            version: MCP_PROVIDER_CAPABILITY_PROFILE_VERSION.to_string(),
            provider: provider.to_string(),
            kind,
            supported_roles: vec![WorkflowAgentRole::DeliveryAgent],
            supported_skill_packs: vec![WorkflowSkillPack::DeliverySkills],
            required_capabilities: vec![
                "repo.read".to_string(),
                "merge_request.create".to_string(),
            ],
            degraded_capabilities: vec![
                "merge_request.ready".to_string(),
                "merge_request.auto_merge".to_string(),
                "merge_request.merged_query".to_string(),
                "issue.close".to_string(),
                "issue.closed_query".to_string(),
            ],
        },
        McpProviderKind::Codex => McpProviderCapabilityProfile {
            version: MCP_PROVIDER_CAPABILITY_PROFILE_VERSION.to_string(),
            provider: provider.to_string(),
            kind,
            supported_roles: vec![
                WorkflowAgentRole::SpecAgent,
                WorkflowAgentRole::WorkAgent,
                WorkflowAgentRole::AuditAgent,
            ],
            supported_skill_packs: vec![
                WorkflowSkillPack::ContractSkills,
                WorkflowSkillPack::ExecutionSkills,
                WorkflowSkillPack::JudgmentSkills,
            ],
            required_capabilities: vec![
                "launch".to_string(),
                "codex.exec".to_string(),
                "session.poll".to_string(),
                "session.logs".to_string(),
                "session.cancel".to_string(),
                "build_agent.complete".to_string(),
            ],
            degraded_capabilities: Vec::new(),
        },
        McpProviderKind::ClaudeCode => McpProviderCapabilityProfile {
            version: MCP_PROVIDER_CAPABILITY_PROFILE_VERSION.to_string(),
            provider: provider.to_string(),
            kind,
            supported_roles: vec![
                WorkflowAgentRole::SpecAgent,
                WorkflowAgentRole::WorkAgent,
                WorkflowAgentRole::AuditAgent,
            ],
            supported_skill_packs: vec![
                WorkflowSkillPack::ContractSkills,
                WorkflowSkillPack::ExecutionSkills,
                WorkflowSkillPack::JudgmentSkills,
            ],
            required_capabilities: vec![
                "launch".to_string(),
                "claude.print".to_string(),
                "session.poll".to_string(),
                "session.logs".to_string(),
                "session.cancel".to_string(),
                "build_agent.complete".to_string(),
            ],
            degraded_capabilities: Vec::new(),
        },
        McpProviderKind::BrowserPreview => McpProviderCapabilityProfile {
            version: MCP_PROVIDER_CAPABILITY_PROFILE_VERSION.to_string(),
            provider: provider.to_string(),
            kind,
            supported_roles: vec![WorkflowAgentRole::Specialist],
            supported_skill_packs: vec![WorkflowSkillPack::SpecialistSkills],
            required_capabilities: vec!["browser_preview.smoke".to_string()],
            degraded_capabilities: vec![
                "browser_preview.dom_snapshot".to_string(),
                "browser_preview.console_logs".to_string(),
                "browser_preview.screenshot".to_string(),
            ],
        },
    })
}

impl McpCapability {
    pub fn new(name: impl Into<String>, available: bool) -> Self {
        Self {
            name: name.into(),
            available,
            detail: None,
        }
    }

    pub fn with_detail(
        name: impl Into<String>,
        available: bool,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            available,
            detail: Some(detail.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpProviderStatus {
    pub version: String,
    pub provider: String,
    pub kind: McpProviderKind,
    pub status: McpProviderStatusCode,
    pub capabilities: Vec<McpCapability>,
    pub cli: Option<String>,
    pub installed: bool,
    pub authenticated: Option<bool>,
    pub repo_permission_checked: bool,
    pub repo_permission: Option<String>,
    pub checked_at: u64,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl McpProviderStatus {
    pub fn new(kind: McpProviderKind, checked_at: u64) -> Self {
        Self {
            version: MCP_PROVIDER_STATUS_VERSION.to_string(),
            provider: kind.as_str().to_string(),
            kind,
            status: McpProviderStatusCode::Unavailable,
            capabilities: Vec::new(),
            cli: None,
            installed: false,
            authenticated: None,
            repo_permission_checked: false,
            repo_permission: None,
            checked_at,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn ready(&self) -> bool {
        matches!(self.status, McpProviderStatusCode::Ready)
    }

    pub fn capability(&self, name: &str) -> Option<&McpCapability> {
        self.capabilities
            .iter()
            .find(|capability| capability.name == name)
    }

    pub fn capability_available(&self, name: &str) -> bool {
        self.capability(name)
            .map(|capability| capability.available)
            .unwrap_or(false)
    }

    pub fn capability_profile(&self) -> Option<McpProviderCapabilityProfile> {
        provider_capability_profile(&self.provider)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpRegistryEntry {
    pub provider: String,
    pub kind: McpProviderKind,
    pub status: McpProviderStatusCode,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpRegistry {
    pub version: String,
    pub updated_at: u64,
    pub providers: Vec<McpRegistryEntry>,
}

impl McpRegistry {
    pub fn new(updated_at: u64) -> Self {
        Self {
            version: MCP_REGISTRY_VERSION.to_string(),
            updated_at,
            providers: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum McpLaunchMode {
    CliExecStdin,
    CliExecPromptFile,
    AppServerThread,
    McpRemoteSession,
}

impl McpLaunchMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CliExecStdin => "cli-exec-stdin",
            Self::CliExecPromptFile => "cli-exec-prompt-file",
            Self::AppServerThread => "app-server-thread",
            Self::McpRemoteSession => "mcp-remote-session",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum McpSessionStatus {
    Queued,
    Claimed,
    Starting,
    Running,
    Interrupted,
    InReview,
    Done,
    Failed,
    Cancelled,
}

impl McpSessionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Claimed => "claimed",
            Self::Starting => "starting",
            Self::Running => "running",
            Self::Interrupted => "interrupted",
            Self::InReview => "in-review",
            Self::Done => "done",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpLaunchRequest {
    pub version: String,
    pub provider: String,
    pub issue_id: String,
    pub project_id: Option<String>,
    pub run_id: String,
    pub agent_role: String,
    pub working_directory: String,
    pub launch_request_path: String,
    pub prompt_path: Option<String>,
    pub context_pack_path: Option<String>,
    pub branch_name: Option<String>,
    pub merge_mode: Option<String>,
}

impl McpLaunchRequest {
    pub fn new(
        provider: impl Into<String>,
        issue_id: impl Into<String>,
        run_id: impl Into<String>,
        agent_role: impl Into<String>,
        working_directory: impl Into<String>,
        launch_request_path: impl Into<String>,
    ) -> Self {
        Self {
            version: MCP_LAUNCH_REQUEST_VERSION.to_string(),
            provider: provider.into(),
            issue_id: issue_id.into(),
            project_id: None,
            run_id: run_id.into(),
            agent_role: agent_role.into(),
            working_directory: working_directory.into(),
            launch_request_path: launch_request_path.into(),
            prompt_path: None,
            context_pack_path: None,
            branch_name: None,
            merge_mode: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpLaunchPlan {
    pub version: String,
    pub provider: String,
    pub session_id: String,
    pub issue_id: String,
    pub run_id: String,
    pub launch_mode: McpLaunchMode,
    pub working_directory: String,
    #[serde(default)]
    pub workspace_root: Option<String>,
    #[serde(default)]
    pub worktree_root: Option<String>,
    #[serde(default)]
    pub runtime_root: Option<String>,
    #[serde(default)]
    pub temp_root: Option<String>,
    #[serde(default)]
    pub cache_root: Option<String>,
    #[serde(default)]
    pub evidence_root: Option<String>,
    pub program: String,
    pub args: Vec<String>,
    pub stdin_path: Option<String>,
    pub output_path: Option<String>,
    #[serde(default)]
    pub permission_mode: Option<String>,
    #[serde(default)]
    pub approval_policy: Option<String>,
    #[serde(default)]
    pub sandbox_mode: Option<String>,
    #[serde(default)]
    pub supervision_mode: Option<String>,
    #[serde(default)]
    pub exit_proof_path: Option<String>,
    pub note: Option<String>,
}

impl McpLaunchPlan {
    pub fn new(
        provider: impl Into<String>,
        session_id: impl Into<String>,
        issue_id: impl Into<String>,
        run_id: impl Into<String>,
        launch_mode: McpLaunchMode,
        working_directory: impl Into<String>,
        program: impl Into<String>,
    ) -> Self {
        Self {
            version: MCP_LAUNCH_PLAN_VERSION.to_string(),
            provider: provider.into(),
            session_id: session_id.into(),
            issue_id: issue_id.into(),
            run_id: run_id.into(),
            launch_mode,
            working_directory: working_directory.into(),
            workspace_root: None,
            worktree_root: None,
            runtime_root: None,
            temp_root: None,
            cache_root: None,
            evidence_root: None,
            program: program.into(),
            args: Vec::new(),
            stdin_path: None,
            output_path: None,
            permission_mode: None,
            approval_policy: None,
            sandbox_mode: None,
            supervision_mode: None,
            exit_proof_path: None,
            note: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpSessionSnapshot {
    pub version: String,
    pub provider: String,
    pub issue_id: String,
    pub project_id: Option<String>,
    pub run_id: String,
    pub session_id: String,
    pub status: McpSessionStatus,
    pub launch_mode: McpLaunchMode,
    pub working_directory: String,
    #[serde(default)]
    pub workspace_root: Option<String>,
    #[serde(default)]
    pub worktree_root: Option<String>,
    #[serde(default)]
    pub runtime_root: Option<String>,
    #[serde(default)]
    pub temp_root: Option<String>,
    #[serde(default)]
    pub cache_root: Option<String>,
    #[serde(default)]
    pub evidence_root: Option<String>,
    pub launch_request_path: String,
    pub plan_path: String,
    #[serde(default)]
    pub log_path: Option<String>,
    #[serde(default)]
    pub branch_name: Option<String>,
    #[serde(default = "default_attempt_count")]
    pub attempt_count: u32,
    #[serde(default)]
    pub pid: Option<u32>,
    #[serde(default)]
    pub process_group_id: Option<u32>,
    #[serde(default)]
    pub remote_session_id: Option<String>,
    #[serde(default)]
    pub pr_url: Option<String>,
    #[serde(default)]
    pub last_message_path: Option<String>,
    #[serde(default)]
    pub exit_proof_path: Option<String>,
    #[serde(default)]
    pub merge_proof_path: Option<String>,
    #[serde(default)]
    pub merge_state: Option<String>,
    #[serde(default)]
    pub writeback_state: Option<String>,
    #[serde(default)]
    pub recovery_reason: Option<String>,
    #[serde(default)]
    pub note: Option<String>,
    #[serde(default)]
    pub last_error: Option<String>,
    #[serde(default)]
    pub permission_mode: Option<String>,
    #[serde(default)]
    pub approval_policy: Option<String>,
    #[serde(default)]
    pub sandbox_mode: Option<String>,
    #[serde(default)]
    pub supervision_mode: Option<String>,
    #[serde(default)]
    pub exited_at: Option<u64>,
    #[serde(default)]
    pub exit_code: Option<i32>,
    #[serde(default)]
    pub governance_policy: McpSessionGovernancePolicy,
    #[serde(default)]
    pub governance_facts: McpSessionGovernanceFacts,
    pub created_at: u64,
    pub updated_at: u64,
}

impl McpSessionSnapshot {
    pub fn queued(request: &McpLaunchRequest, plan: &McpLaunchPlan, created_at: u64) -> Self {
        let governance_policy = McpSessionGovernancePolicy::default();
        let governance_facts = McpSessionGovernanceFacts {
            timeout_at: Some(created_at + governance_policy.timeout_seconds),
            retryable: true,
            ..Default::default()
        };
        Self {
            version: MCP_SESSION_SNAPSHOT_VERSION.to_string(),
            provider: request.provider.clone(),
            issue_id: request.issue_id.clone(),
            project_id: request.project_id.clone(),
            run_id: request.run_id.clone(),
            session_id: plan.session_id.clone(),
            status: McpSessionStatus::Queued,
            launch_mode: plan.launch_mode.clone(),
            working_directory: plan.working_directory.clone(),
            workspace_root: plan.workspace_root.clone(),
            worktree_root: plan.worktree_root.clone(),
            runtime_root: plan.runtime_root.clone(),
            temp_root: plan.temp_root.clone(),
            cache_root: plan.cache_root.clone(),
            evidence_root: plan.evidence_root.clone(),
            launch_request_path: request.launch_request_path.clone(),
            plan_path: format!(".agentflow/state/mcp/plans/{}.json", plan.session_id),
            log_path: plan.output_path.clone(),
            branch_name: request.branch_name.clone(),
            attempt_count: 1,
            pid: None,
            process_group_id: None,
            remote_session_id: None,
            pr_url: None,
            last_message_path: None,
            exit_proof_path: plan.exit_proof_path.clone(),
            merge_proof_path: None,
            merge_state: None,
            writeback_state: None,
            recovery_reason: None,
            note: plan.note.clone(),
            last_error: None,
            permission_mode: plan.permission_mode.clone(),
            approval_policy: plan.approval_policy.clone(),
            sandbox_mode: plan.sandbox_mode.clone(),
            supervision_mode: plan.supervision_mode.clone(),
            exited_at: None,
            exit_code: None,
            governance_policy,
            governance_facts,
            created_at,
            updated_at: created_at,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpLogChunk {
    pub version: String,
    pub provider: String,
    pub session_id: String,
    pub cursor: Option<String>,
    pub lines: Vec<String>,
}

impl McpLogChunk {
    pub fn empty(provider: impl Into<String>, session_id: impl Into<String>) -> Self {
        Self {
            version: MCP_LOG_CHUNK_VERSION.to_string(),
            provider: provider.into(),
            session_id: session_id.into(),
            cursor: None,
            lines: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{provider_capability_profile, McpCapability, McpProviderKind, McpProviderStatus};
    use agentflow_workflow_core::{WorkflowAgentRole, WorkflowSkillPack};

    #[test]
    fn provider_status_checks_capabilities_by_name() {
        let mut status = McpProviderStatus::new(McpProviderKind::Codex, 1);
        status.capabilities = vec![
            McpCapability::new("launch", true),
            McpCapability::new("build_agent.complete", false),
        ];

        assert!(status.capability_available("launch"));
        assert!(!status.capability_available("build_agent.complete"));
        assert!(!status.capability_available("missing"));
    }

    #[test]
    fn provider_capability_profile_maps_known_provider_roles_and_skills() {
        let codex = provider_capability_profile("codex").unwrap();
        assert!(codex.supports_role(WorkflowAgentRole::WorkAgent));
        assert!(codex.supports_skill_pack(WorkflowSkillPack::ExecutionSkills));
        assert_eq!(
            codex.required_capabilities,
            vec![
                "launch".to_string(),
                "codex.exec".to_string(),
                "session.poll".to_string(),
                "session.logs".to_string(),
                "session.cancel".to_string(),
                "build_agent.complete".to_string(),
            ]
        );
        assert!(codex.degraded_capabilities.is_empty());

        let github = provider_capability_profile("github").unwrap();
        assert!(github.supports_role(WorkflowAgentRole::DeliveryAgent));
        assert!(github.supports_skill_pack(WorkflowSkillPack::DeliverySkills));
        assert!(!github.supports_role(WorkflowAgentRole::WorkAgent));
        assert!(github
            .degraded_capabilities
            .contains(&"issue.close".to_string()));
        assert!(github
            .degraded_capabilities
            .contains(&"issue.closed_query".to_string()));

        let claude = provider_capability_profile("claude").unwrap();
        assert!(claude.supports_role(WorkflowAgentRole::WorkAgent));
        assert!(claude.supports_skill_pack(WorkflowSkillPack::ExecutionSkills));
        assert_eq!(
            claude.required_capabilities,
            vec![
                "launch".to_string(),
                "claude.print".to_string(),
                "session.poll".to_string(),
                "session.logs".to_string(),
                "session.cancel".to_string(),
                "build_agent.complete".to_string(),
            ]
        );
    }
}
