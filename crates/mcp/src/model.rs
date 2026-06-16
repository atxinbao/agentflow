use serde::{Deserialize, Serialize};

pub const MCP_PROVIDER_STATUS_VERSION: &str = "agentflow-mcp-provider.v1";
pub const MCP_REGISTRY_VERSION: &str = "agentflow-mcp-registry.v1";
pub const MCP_LAUNCH_REQUEST_VERSION: &str = "agentflow-mcp-launch-request.v1";
pub const MCP_LAUNCH_PLAN_VERSION: &str = "agentflow-mcp-launch-plan.v1";
pub const MCP_SESSION_SNAPSHOT_VERSION: &str = "agentflow-mcp-session.v1";
pub const MCP_LOG_CHUNK_VERSION: &str = "agentflow-mcp-log-chunk.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum McpProviderKind {
    Github,
    Gitlab,
    Codex,
    BrowserPreview,
}

impl McpProviderKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Github => "github",
            Self::Gitlab => "gitlab",
            Self::Codex => "codex",
            Self::BrowserPreview => "browser-preview",
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
    pub program: String,
    pub args: Vec<String>,
    pub stdin_path: Option<String>,
    pub output_path: Option<String>,
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
            program: program.into(),
            args: Vec::new(),
            stdin_path: None,
            output_path: None,
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
    pub launch_request_path: String,
    pub plan_path: String,
    pub log_path: Option<String>,
    pub branch_name: Option<String>,
    pub pid: Option<u32>,
    pub remote_session_id: Option<String>,
    pub pr_url: Option<String>,
    pub merge_state: Option<String>,
    pub note: Option<String>,
    pub last_error: Option<String>,
    pub created_at: u64,
    pub updated_at: u64,
}

impl McpSessionSnapshot {
    pub fn queued(request: &McpLaunchRequest, plan: &McpLaunchPlan, created_at: u64) -> Self {
        Self {
            version: MCP_SESSION_SNAPSHOT_VERSION.to_string(),
            provider: request.provider.clone(),
            issue_id: request.issue_id.clone(),
            project_id: request.project_id.clone(),
            run_id: request.run_id.clone(),
            session_id: plan.session_id.clone(),
            status: McpSessionStatus::Queued,
            launch_mode: plan.launch_mode.clone(),
            launch_request_path: request.launch_request_path.clone(),
            plan_path: format!(".agentflow/state/mcp/plans/{}.json", plan.session_id),
            log_path: plan.output_path.clone(),
            branch_name: request.branch_name.clone(),
            pid: None,
            remote_session_id: None,
            pr_url: None,
            merge_state: None,
            note: plan.note.clone(),
            last_error: None,
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
    use super::{McpCapability, McpProviderKind, McpProviderStatus};

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
}
