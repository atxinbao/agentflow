use crate::{
    health::unix_timestamp_seconds,
    model::{
        McpLaunchPlan, McpLaunchRequest, McpLogChunk, McpProviderKind, McpProviderStatus,
        McpSessionSnapshot, McpSessionStatus,
    },
    storage::{read_session_snapshot, write_launch_plan, write_session_snapshot},
};
use agentflow_workflow_core::{canonicalize_project_root, join_relative_path};
use anyhow::{Context, Result};
use serde_json::json;
#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;
use std::{
    collections::BTreeMap,
    fs,
    path::Path,
    process::{Child, Command, Output},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchIsolationBoundary {
    pub workspace_root: std::path::PathBuf,
    pub worktree_root: std::path::PathBuf,
    pub launch_request_path: std::path::PathBuf,
    pub prompt_path: std::path::PathBuf,
    pub context_pack_path: Option<std::path::PathBuf>,
    pub exit_proof_path: std::path::PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandProbe {
    pub status_success: bool,
    pub stdout: String,
    pub stderr: String,
}

impl CommandProbe {
    pub fn from_output(output: Output) -> Self {
        Self {
            status_success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        }
    }

    pub fn combined_output(&self) -> String {
        format!("{}{}", self.stdout, self.stderr)
    }
}

pub fn run_command(
    project_root: impl AsRef<Path>,
    program: &str,
    args: &[&str],
) -> Result<CommandProbe> {
    let output = Command::new(program)
        .args(args)
        .current_dir(project_root.as_ref())
        .output()
        .with_context(|| format!("run {} {}", program, args.join(" ")))?;
    Ok(CommandProbe::from_output(output))
}

pub fn resolve_launch_isolation_boundary(
    project_root: impl AsRef<Path>,
    request: &McpLaunchRequest,
    session_id: &str,
    attempt_count: u32,
) -> Result<LaunchIsolationBoundary> {
    let workspace_root = canonicalize_project_root(project_root)?;
    let worktree_root = canonical_path_within_workspace(
        &workspace_root,
        &request.working_directory,
        "working directory",
    )?;
    let launch_request_path = existing_path_within_workspace(
        &workspace_root,
        &request.launch_request_path,
        "launch request",
    )?;
    let prompt_relative = request
        .prompt_path
        .as_deref()
        .unwrap_or(&request.launch_request_path);
    let prompt_path =
        existing_path_within_workspace(&workspace_root, prompt_relative, "prompt path")?;
    let context_pack_path = request
        .context_pack_path
        .as_deref()
        .map(|value| existing_path_within_workspace(&workspace_root, value, "context pack path"))
        .transpose()?;
    let exit_proof_path = join_relative_path(
        &workspace_root,
        &session_exit_proof_relative_path(session_id, attempt_count),
    )?;
    Ok(LaunchIsolationBoundary {
        workspace_root,
        worktree_root,
        launch_request_path,
        prompt_path,
        context_pack_path,
        exit_proof_path,
    })
}

pub fn ensure_parent_directory(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    Ok(())
}

pub fn relative_project_path(project_root: &Path, path: &Path) -> String {
    path.strip_prefix(project_root)
        .unwrap_or(path)
        .display()
        .to_string()
}

pub fn write_exit_proof(
    proof_path: &Path,
    session: &McpSessionSnapshot,
    status: &str,
    exit_code: Option<i32>,
    exited_at: u64,
    last_error: Option<&str>,
) -> Result<()> {
    ensure_parent_directory(proof_path)?;
    let proof = json!({
        "version": crate::model::MCP_SESSION_EXIT_PROOF_VERSION,
        "provider": session.provider,
        "issueId": session.issue_id,
        "projectId": session.project_id,
        "runId": session.run_id,
        "sessionId": session.session_id,
        "status": status,
        "exitCode": exit_code,
        "exitedAt": exited_at,
        "workingDirectory": session.working_directory,
        "workspaceRoot": session.workspace_root,
        "worktreeRoot": session.worktree_root,
        "permissionMode": session.permission_mode,
        "approvalPolicy": session.approval_policy,
        "sandboxMode": session.sandbox_mode,
        "supervisionMode": session.supervision_mode,
        "terminalReason": session.governance_facts.terminal_reason,
        "retryable": session.governance_facts.retryable,
        "lastError": last_error,
    });
    fs::write(proof_path, serde_json::to_string_pretty(&proof)? + "\n")
        .with_context(|| format!("write {}", proof_path.display()))
}

pub fn spawn_exit_watcher(
    mut child: Child,
    proof_path: std::path::PathBuf,
    mut session: McpSessionSnapshot,
) {
    std::thread::spawn(move || {
        let waited = child.wait();
        let exited_at = unix_timestamp_seconds();
        let (status, exit_code, last_error) = match waited {
            Ok(exit_status) => {
                #[cfg(unix)]
                let exit_code = exit_status
                    .code()
                    .or_else(|| exit_status.signal().map(|signal| 128 + signal));
                #[cfg(not(unix))]
                let exit_code = exit_status.code();
                let status = if exit_status.success() {
                    "exited"
                } else {
                    "failed"
                };
                (status.to_string(), exit_code, None)
            }
            Err(error) => ("wait-failed".to_string(), None, Some(error.to_string())),
        };
        session.exited_at = Some(exited_at);
        session.exit_code = exit_code;
        let _ = write_exit_proof(
            &proof_path,
            &session,
            &status,
            exit_code,
            exited_at,
            last_error.as_deref(),
        );
    });
}

fn canonical_path_within_workspace(
    workspace_root: &Path,
    value: &str,
    label: &str,
) -> Result<std::path::PathBuf> {
    let candidate = if Path::new(value).is_absolute() {
        Path::new(value).to_path_buf()
    } else {
        workspace_root.join(value)
    };
    let canonical = candidate
        .canonicalize()
        .with_context(|| format!("canonicalize {label} {}", candidate.display()))?;
    if !canonical.starts_with(workspace_root) {
        anyhow::bail!(
            "{label} escapes workspace boundary: {} is outside {}",
            canonical.display(),
            workspace_root.display()
        );
    }
    Ok(canonical)
}

fn existing_path_within_workspace(
    workspace_root: &Path,
    value: &str,
    label: &str,
) -> Result<std::path::PathBuf> {
    let canonical = canonical_path_within_workspace(workspace_root, value, label)?;
    if !canonical.exists() {
        anyhow::bail!("{label} does not exist: {}", canonical.display());
    }
    Ok(canonical)
}

fn session_exit_proof_relative_path(session_id: &str, attempt_count: u32) -> String {
    if attempt_count <= 1 {
        format!(".agentflow/state/mcp/sessions/{session_id}-exit.json")
    } else {
        format!(".agentflow/state/mcp/sessions/{session_id}-attempt-{attempt_count}-exit.json")
    }
}

pub trait McpAgentProvider: Send + Sync {
    fn provider_id(&self) -> &'static str;

    fn kind(&self) -> McpProviderKind;

    fn check_health(&self, project_root: &Path) -> McpProviderStatus;

    fn build_launch_plan(
        &self,
        project_root: &Path,
        request: &McpLaunchRequest,
    ) -> Result<McpLaunchPlan>;

    fn create_session(
        &self,
        project_root: &Path,
        request: &McpLaunchRequest,
    ) -> Result<McpSessionSnapshot> {
        let plan = self.build_launch_plan(project_root, request)?;
        let now = unix_timestamp_seconds();
        let session = McpSessionSnapshot::queued(request, &plan, now);
        write_launch_plan(project_root, &plan)?;
        write_session_snapshot(project_root, &session)?;
        Ok(session)
    }

    fn poll_session(&self, project_root: &Path, session_id: &str) -> Result<McpSessionSnapshot> {
        read_session_snapshot(project_root, session_id)
    }

    fn fetch_logs(
        &self,
        _project_root: &Path,
        session_id: &str,
        _cursor: Option<&str>,
    ) -> Result<McpLogChunk> {
        Ok(McpLogChunk::empty(self.provider_id(), session_id))
    }

    fn cancel_session(&self, project_root: &Path, session_id: &str) -> Result<McpSessionSnapshot> {
        let mut session = read_session_snapshot(project_root, session_id)?;
        session.status = McpSessionStatus::Cancelled;
        session.updated_at = unix_timestamp_seconds();
        write_session_snapshot(project_root, &session)?;
        Ok(session)
    }
}

#[derive(Default)]
pub struct McpProviderBridge {
    providers: BTreeMap<String, Box<dyn McpAgentProvider>>,
}

impl McpProviderBridge {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, provider: Box<dyn McpAgentProvider>) {
        self.providers
            .insert(provider.provider_id().to_string(), provider);
    }

    pub fn provider(&self, provider_id: &str) -> Option<&dyn McpAgentProvider> {
        self.providers.get(provider_id).map(Box::as_ref)
    }

    pub fn providers(&self) -> impl Iterator<Item = &dyn McpAgentProvider> {
        self.providers.values().map(Box::as_ref)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{McpCapability, McpLaunchMode, McpProviderStatusCode};
    use tempfile::tempdir;

    struct FakeProvider;

    impl McpAgentProvider for FakeProvider {
        fn provider_id(&self) -> &'static str {
            "fake"
        }

        fn kind(&self) -> McpProviderKind {
            McpProviderKind::Codex
        }

        fn check_health(&self, _project_root: &Path) -> McpProviderStatus {
            let mut status = McpProviderStatus::new(McpProviderKind::Codex, 1);
            status.provider = "fake".to_string();
            status.status = McpProviderStatusCode::Ready;
            status.capabilities = vec![McpCapability::new("launch", true)];
            status
        }

        fn build_launch_plan(
            &self,
            _project_root: &Path,
            request: &McpLaunchRequest,
        ) -> Result<McpLaunchPlan> {
            let mut plan = McpLaunchPlan::new(
                request.provider.clone(),
                "fake-run-001",
                request.issue_id.clone(),
                request.run_id.clone(),
                McpLaunchMode::CliExecPromptFile,
                request.working_directory.clone(),
                "fake-cli",
            );
            plan.stdin_path = Some(request.launch_request_path.clone());
            Ok(plan)
        }
    }

    #[test]
    fn provider_bridge_creates_and_cancels_session() {
        let dir = tempdir().unwrap();
        let provider = FakeProvider;
        let request = McpLaunchRequest::new(
            "fake",
            "AF-001",
            "run-001",
            "build-agent",
            dir.path().display().to_string(),
            ".agentflow/tasks/AF-001/runs/run-001/launch/agent-request.json",
        );
        let session = provider.create_session(dir.path(), &request).unwrap();
        assert_eq!(session.status, McpSessionStatus::Queued);
        assert_eq!(
            session.plan_path,
            ".agentflow/state/mcp/plans/fake-run-001.json"
        );

        let cancelled = provider
            .cancel_session(dir.path(), &session.session_id)
            .unwrap();
        assert_eq!(cancelled.status, McpSessionStatus::Cancelled);
    }
}
