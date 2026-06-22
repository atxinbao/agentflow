use crate::{
    health::unix_timestamp_seconds,
    model::{
        McpLaunchRequest, McpProviderStatus, McpProviderStatusCode, McpSessionSnapshot,
        McpSessionStatus,
    },
    provider::McpAgentProvider,
    storage::{read_session_snapshot, write_session_snapshot},
};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};
use tempfile::NamedTempFile;

pub const MCP_PROVIDER_SMOKE_ARTIFACT_VERSION: &str = "agentflow-mcp-provider-smoke.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum McpProviderSmokeOutcome {
    Skipped,
    Passed,
    Failed,
}

impl McpProviderSmokeOutcome {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Skipped => "skipped",
            Self::Passed => "passed",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpProviderSmokeRequest {
    pub enabled: bool,
    pub provider: String,
    pub issue_id: String,
    pub run_id: String,
    pub agent_role: String,
    pub working_directory: String,
    pub launch_request_path: String,
    pub prompt_path: Option<String>,
    pub branch_name: Option<String>,
}

impl McpProviderSmokeRequest {
    pub fn new(
        provider: impl Into<String>,
        issue_id: impl Into<String>,
        run_id: impl Into<String>,
        working_directory: impl Into<String>,
        launch_request_path: impl Into<String>,
    ) -> Self {
        Self {
            enabled: false,
            provider: provider.into(),
            issue_id: issue_id.into(),
            run_id: run_id.into(),
            agent_role: "work-agent".to_string(),
            working_directory: working_directory.into(),
            launch_request_path: launch_request_path.into(),
            prompt_path: None,
            branch_name: None,
        }
    }

    pub fn into_launch_request(&self) -> McpLaunchRequest {
        let mut request = McpLaunchRequest::new(
            self.provider.clone(),
            self.issue_id.clone(),
            self.run_id.clone(),
            self.agent_role.clone(),
            self.working_directory.clone(),
            self.launch_request_path.clone(),
        );
        request.prompt_path = self.prompt_path.clone();
        request.branch_name = self.branch_name.clone();
        request
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpProviderSmokeArtifact {
    pub version: String,
    pub provider: String,
    pub outcome: McpProviderSmokeOutcome,
    pub reason: String,
    pub health: McpProviderStatus,
    pub launch_request_path: Option<String>,
    pub session_id: Option<String>,
    pub session_snapshot_path: Option<String>,
    pub session_snapshot_readable: bool,
    pub terminal_status: Option<McpSessionStatus>,
    pub terminal_provider_state_projectable: bool,
    pub artifact_path: String,
    pub created_at: u64,
}

pub fn run_provider_smoke_gate(
    project_root: impl AsRef<Path>,
    provider: &dyn McpAgentProvider,
    request: &McpProviderSmokeRequest,
) -> Result<McpProviderSmokeArtifact> {
    let root = canonical_project_root(project_root)?;
    let checked_at = unix_timestamp_seconds();
    let health = provider.check_health(&root);
    if !request.enabled {
        return write_provider_smoke_artifact(
            &root,
            McpProviderSmokeArtifact {
                version: MCP_PROVIDER_SMOKE_ARTIFACT_VERSION.to_string(),
                provider: request.provider.clone(),
                outcome: McpProviderSmokeOutcome::Skipped,
                reason: "PROVIDER_SMOKE=0".to_string(),
                health,
                launch_request_path: None,
                session_id: None,
                session_snapshot_path: None,
                session_snapshot_readable: false,
                terminal_status: None,
                terminal_provider_state_projectable: false,
                artifact_path: provider_smoke_artifact_relative_path(&request.provider, checked_at),
                created_at: checked_at,
            },
        );
    }

    if !matches!(health.status, McpProviderStatusCode::Ready) {
        return write_provider_smoke_artifact(
            &root,
            McpProviderSmokeArtifact {
                version: MCP_PROVIDER_SMOKE_ARTIFACT_VERSION.to_string(),
                provider: request.provider.clone(),
                outcome: McpProviderSmokeOutcome::Skipped,
                reason: format!("provider health is {}", health.status.as_str()),
                health,
                launch_request_path: None,
                session_id: None,
                session_snapshot_path: None,
                session_snapshot_readable: false,
                terminal_status: None,
                terminal_provider_state_projectable: false,
                artifact_path: provider_smoke_artifact_relative_path(&request.provider, checked_at),
                created_at: checked_at,
            },
        );
    }

    ensure_minimal_launch_request_file(&root, request)?;
    let launch_request = request.into_launch_request();
    let session = provider.create_session(&root, &launch_request)?;
    let snapshot = read_session_snapshot(&root, &session.session_id)?;
    let terminal = ensure_terminal_session(&root, provider, &snapshot)?;
    let session_snapshot_path = session_snapshot_relative_path(&terminal.session_id);
    let terminal_readable = read_session_snapshot(&root, &terminal.session_id).is_ok();
    let terminal_provider_state_projectable = terminal_readable
        && matches!(
            terminal.status,
            McpSessionStatus::Cancelled | McpSessionStatus::Done
        );

    write_provider_smoke_artifact(
        &root,
        McpProviderSmokeArtifact {
            version: MCP_PROVIDER_SMOKE_ARTIFACT_VERSION.to_string(),
            provider: request.provider.clone(),
            outcome: if terminal_provider_state_projectable {
                McpProviderSmokeOutcome::Passed
            } else {
                McpProviderSmokeOutcome::Failed
            },
            reason: if terminal_provider_state_projectable {
                "minimal launch, session snapshot, and terminal projection passed".to_string()
            } else {
                "terminal provider state is not projectable".to_string()
            },
            health,
            launch_request_path: Some(request.launch_request_path.clone()),
            session_id: Some(terminal.session_id.clone()),
            session_snapshot_path: Some(session_snapshot_path),
            session_snapshot_readable: terminal_readable,
            terminal_status: Some(terminal.status.clone()),
            terminal_provider_state_projectable,
            artifact_path: provider_smoke_artifact_relative_path(&request.provider, checked_at),
            created_at: checked_at,
        },
    )
}

fn ensure_terminal_session(
    root: &Path,
    provider: &dyn McpAgentProvider,
    session: &McpSessionSnapshot,
) -> Result<McpSessionSnapshot> {
    if matches!(
        session.status,
        McpSessionStatus::Cancelled | McpSessionStatus::Done | McpSessionStatus::Failed
    ) {
        write_session_snapshot(root, session)?;
        return Ok(session.clone());
    }
    provider.cancel_session(root, &session.session_id)
}

fn ensure_minimal_launch_request_file(
    root: &Path,
    request: &McpProviderSmokeRequest,
) -> Result<()> {
    let request_path = absolute_project_path(root, &request.launch_request_path);
    if request_path.is_file() {
        return Ok(());
    }
    write_text(
        &request_path,
        &format!(
            "# AgentFlow provider smoke\n\nProvider: {}\nIssue: {}\nRun: {}\n\nStart and exit cleanly. Do not modify source files.\n",
            request.provider, request.issue_id, request.run_id
        ),
    )
}

fn write_provider_smoke_artifact(
    root: &Path,
    artifact: McpProviderSmokeArtifact,
) -> Result<McpProviderSmokeArtifact> {
    let path = root.join(&artifact.artifact_path);
    write_json(&path, &artifact)?;
    Ok(artifact)
}

fn provider_smoke_artifact_relative_path(provider: &str, created_at: u64) -> String {
    format!(
        ".agentflow/state/mcp/provider-smoke/{}-{}.json",
        sanitize_id(provider),
        created_at
    )
}

fn session_snapshot_relative_path(session_id: &str) -> String {
    format!(
        ".agentflow/state/mcp/sessions/{}.json",
        sanitize_id(session_id)
    )
}

fn absolute_project_path(root: &Path, relative: &str) -> PathBuf {
    let path = Path::new(relative);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    }
}

fn canonical_project_root(project_root: impl AsRef<Path>) -> Result<PathBuf> {
    project_root
        .as_ref()
        .canonicalize()
        .with_context(|| format!("canonicalize {}", project_root.as_ref().display()))
}

fn sanitize_id(id: &str) -> String {
    id.chars()
        .map(|ch| match ch {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => ch,
        })
        .collect()
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    write_text(path, &(serde_json::to_string_pretty(value)? + "\n"))
}

fn write_text(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let parent = path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("{} has no parent directory", path.display()))?;
    let mut temp = NamedTempFile::new_in(parent)
        .with_context(|| format!("create temp file in {}", parent.display()))?;
    temp.write_all(content.as_bytes())
        .with_context(|| format!("write temp file for {}", path.display()))?;
    temp.flush()
        .with_context(|| format!("flush temp file for {}", path.display()))?;
    temp.as_file()
        .sync_all()
        .with_context(|| format!("sync temp file for {}", path.display()))?;
    temp.persist(path)
        .map_err(|error| error.error)
        .with_context(|| format!("persist {}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        run_provider_smoke_gate, McpProviderSmokeOutcome, McpProviderSmokeRequest,
        MCP_PROVIDER_SMOKE_ARTIFACT_VERSION,
    };
    use crate::{
        model::{
            McpLaunchMode, McpLaunchPlan, McpLaunchRequest, McpProviderKind, McpProviderStatus,
            McpProviderStatusCode,
        },
        provider::McpAgentProvider,
        storage::read_session_snapshot,
    };
    use anyhow::Result;
    use std::path::Path;

    struct FakeProvider {
        ready: bool,
    }

    impl McpAgentProvider for FakeProvider {
        fn provider_id(&self) -> &'static str {
            "codex"
        }

        fn kind(&self) -> McpProviderKind {
            McpProviderKind::Codex
        }

        fn check_health(&self, _project_root: &Path) -> McpProviderStatus {
            let mut status = McpProviderStatus::new(McpProviderKind::Codex, 1);
            status.status = if self.ready {
                McpProviderStatusCode::Ready
            } else {
                McpProviderStatusCode::Unauthenticated
            };
            status.installed = self.ready;
            status.authenticated = Some(self.ready);
            status
        }

        fn build_launch_plan(
            &self,
            _project_root: &Path,
            request: &McpLaunchRequest,
        ) -> Result<McpLaunchPlan> {
            let mut plan = McpLaunchPlan::new(
                self.provider_id(),
                format!("{}-session", request.run_id),
                request.issue_id.clone(),
                request.run_id.clone(),
                McpLaunchMode::CliExecStdin,
                request.working_directory.clone(),
                "codex",
            );
            plan.stdin_path = Some(request.launch_request_path.clone());
            plan.output_path = Some(format!(
                ".agentflow/state/mcp/sessions/{}-session.jsonl",
                request.run_id
            ));
            Ok(plan)
        }
    }

    #[test]
    fn provider_smoke_disabled_writes_clear_skip() {
        let dir = tempfile::tempdir().unwrap();
        let request = McpProviderSmokeRequest::new(
            "codex",
            "AF-SMOKE-001",
            "run-smoke-001",
            dir.path().display().to_string(),
            ".agentflow/tmp/provider-smoke-request.md",
        );

        let artifact = run_provider_smoke_gate(dir.path(), &FakeProvider { ready: true }, &request)
            .expect("smoke artifact");

        assert_eq!(artifact.version, MCP_PROVIDER_SMOKE_ARTIFACT_VERSION);
        assert_eq!(artifact.outcome, McpProviderSmokeOutcome::Skipped);
        assert_eq!(artifact.reason, "PROVIDER_SMOKE=0");
        assert!(dir.path().join(&artifact.artifact_path).is_file());
        assert!(artifact.session_id.is_none());
    }

    #[test]
    fn provider_smoke_skips_when_provider_is_not_ready() {
        let dir = tempfile::tempdir().unwrap();
        let mut request = McpProviderSmokeRequest::new(
            "codex",
            "AF-SMOKE-001",
            "run-smoke-001",
            dir.path().display().to_string(),
            ".agentflow/tmp/provider-smoke-request.md",
        );
        request.enabled = true;

        let artifact =
            run_provider_smoke_gate(dir.path(), &FakeProvider { ready: false }, &request)
                .expect("smoke artifact");

        assert_eq!(artifact.outcome, McpProviderSmokeOutcome::Skipped);
        assert_eq!(
            artifact.health.status,
            McpProviderStatusCode::Unauthenticated
        );
        assert!(artifact.reason.contains("unauthenticated"));
        assert!(artifact.session_id.is_none());
    }

    #[test]
    fn provider_smoke_creates_readable_terminal_session_snapshot() {
        let dir = tempfile::tempdir().unwrap();
        let mut request = McpProviderSmokeRequest::new(
            "codex",
            "AF-SMOKE-001",
            "run-smoke-001",
            dir.path().display().to_string(),
            ".agentflow/tmp/provider-smoke-request.md",
        );
        request.enabled = true;

        let artifact = run_provider_smoke_gate(dir.path(), &FakeProvider { ready: true }, &request)
            .expect("smoke artifact");

        assert_eq!(artifact.outcome, McpProviderSmokeOutcome::Passed);
        assert!(artifact.session_snapshot_readable);
        assert!(artifact.terminal_provider_state_projectable);
        let session_id = artifact.session_id.as_deref().expect("session id");
        let session = read_session_snapshot(dir.path(), session_id).expect("session snapshot");
        assert_eq!(session.status.as_str(), "cancelled");
    }
}
