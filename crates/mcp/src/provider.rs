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
    path::{Path, PathBuf},
    process::{Child, Command, Output},
    thread,
    time::Duration,
};

const PROCESS_TERMINATION_GRACE_MILLIS: u64 = 1_500;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchIsolationBoundary {
    pub workspace_root: PathBuf,
    pub worktree_root: PathBuf,
    pub runtime_root: PathBuf,
    pub temp_root: PathBuf,
    pub cache_root: PathBuf,
    pub evidence_root: PathBuf,
    pub launch_request_path: PathBuf,
    pub prompt_path: PathBuf,
    pub context_pack_path: Option<PathBuf>,
    pub exit_proof_path: PathBuf,
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
    let runtime_root = launch_runtime_root(&workspace_root, &request.issue_id, &request.run_id)?;
    ensure_directory(&runtime_root)?;
    let temp_root = runtime_root.join("tmp");
    let cache_root = runtime_root.join("cache");
    let evidence_root = runtime_root.join("evidence");
    ensure_directory(&temp_root)?;
    ensure_directory(&cache_root)?;
    ensure_directory(&evidence_root)?;
    let worktree_root = if request.branch_name.is_some() && is_git_repository(&workspace_root) {
        prepare_git_run_worktree(
            &workspace_root,
            &runtime_root.join("worktree"),
            request.branch_name.as_deref(),
        )?
    } else {
        canonical_path_within_workspace(
            &workspace_root,
            &request.working_directory,
            "working directory",
        )?
    };
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
        runtime_root,
        temp_root,
        cache_root,
        evidence_root,
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

pub fn ensure_directory(path: &Path) -> Result<()> {
    fs::create_dir_all(path).with_context(|| format!("create {}", path.display()))
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
        "runtimeRoot": session.runtime_root,
        "tempRoot": session.temp_root,
        "cacheRoot": session.cache_root,
        "evidenceRoot": session.evidence_root,
        "permissionMode": session.permission_mode,
        "approvalPolicy": session.approval_policy,
        "sandboxMode": session.sandbox_mode,
        "supervisionMode": session.supervision_mode,
        "processGroupId": session.process_group_id,
        "terminalReason": session.governance_facts.terminal_reason,
        "retryable": session.governance_facts.retryable,
        "lastError": last_error,
    });
    fs::write(proof_path, serde_json::to_string_pretty(&proof)? + "\n")
        .with_context(|| format!("write {}", proof_path.display()))
}

pub fn spawn_exit_watcher(
    mut child: Child,
    project_root: PathBuf,
    proof_path: PathBuf,
    session_id: String,
    fallback_session: McpSessionSnapshot,
) {
    thread::spawn(move || {
        let waited = child.wait();
        let exited_at = unix_timestamp_seconds();
        let mut session =
            read_session_snapshot(&project_root, &session_id).unwrap_or(fallback_session);
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
        let final_status = if session.governance_facts.cancel_requested_at.is_some() {
            "cancelled".to_string()
        } else if session.governance_facts.timed_out_at.is_some() {
            "timeout".to_string()
        } else {
            status
        };
        session.exited_at = Some(exited_at);
        session.exit_code = exit_code;
        session.pid = None;
        session.process_group_id = None;
        session.updated_at = exited_at;
        if final_status == "cancelled" {
            session.status = McpSessionStatus::Cancelled;
            session
                .governance_facts
                .cancelled_at
                .get_or_insert(exited_at);
            session.governance_facts.retryable = false;
            session.governance_facts.terminal_reason = Some("cancelled".to_string());
        } else if final_status == "timeout" {
            session.status = McpSessionStatus::Interrupted;
            session.governance_facts.terminal_reason = Some("timeout".to_string());
        } else if matches!(
            session.status,
            McpSessionStatus::Claimed | McpSessionStatus::Starting | McpSessionStatus::Running
        ) {
            session.status = if exit_code == Some(0) {
                McpSessionStatus::Interrupted
            } else {
                McpSessionStatus::Failed
            };
        }
        let _ = write_session_snapshot(&project_root, &session);
        let _ = write_exit_proof(
            &proof_path,
            &session,
            &final_status,
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

fn launch_runtime_root(workspace_root: &Path, issue_id: &str, run_id: &str) -> Result<PathBuf> {
    join_relative_path(
        workspace_root,
        PathBuf::from(".agentflow")
            .join("tasks")
            .join(issue_id)
            .join("runs")
            .join(run_id)
            .join("runtime"),
    )
}

fn is_git_repository(path: &Path) -> bool {
    Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .current_dir(path)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn prepare_git_run_worktree(
    workspace_root: &Path,
    worktree_root: &Path,
    branch_name: Option<&str>,
) -> Result<PathBuf> {
    if is_git_repository(worktree_root) {
        return worktree_root
            .canonicalize()
            .with_context(|| format!("canonicalize {}", worktree_root.display()));
    }
    if worktree_root.exists() {
        fs::remove_dir_all(worktree_root)
            .with_context(|| format!("remove stale {}", worktree_root.display()))?;
    }
    ensure_parent_directory(worktree_root)?;
    let add = Command::new("git")
        .args([
            "worktree",
            "add",
            "--force",
            "--detach",
            worktree_root
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("invalid worktree path"))?,
            "HEAD",
        ])
        .current_dir(workspace_root)
        .output()
        .with_context(|| format!("git worktree add {}", worktree_root.display()))?;
    if !add.status.success() {
        anyhow::bail!(
            "git worktree add failed: {}",
            String::from_utf8_lossy(&add.stderr).trim()
        );
    }
    if let Some(branch_name) = branch_name {
        let switch = Command::new("git")
            .args(["switch", "-C", branch_name])
            .current_dir(worktree_root)
            .output()
            .with_context(|| {
                format!(
                    "git switch -C {} in {}",
                    branch_name,
                    worktree_root.display()
                )
            })?;
        if !switch.status.success() {
            anyhow::bail!(
                "git switch -C {} failed: {}",
                branch_name,
                String::from_utf8_lossy(&switch.stderr).trim()
            );
        }
    }
    worktree_root
        .canonicalize()
        .with_context(|| format!("canonicalize {}", worktree_root.display()))
}

pub fn terminate_process_group(process_group_id: Option<u32>, pid: Option<u32>) -> Result<bool> {
    if let Some(process_group_id) = process_group_id {
        match terminate_unix_process_group(process_group_id) {
            Ok(true) => return Ok(true),
            Ok(false) => {}
            Err(_) => {}
        }
    }
    if let Some(pid) = pid {
        return terminate_single_process(pid);
    }
    Ok(true)
}

fn terminate_unix_process_group(process_group_id: u32) -> Result<bool> {
    send_signal_to_process_group(process_group_id, "TERM")?;
    if wait_for_process_group_exit(process_group_id, PROCESS_TERMINATION_GRACE_MILLIS)? {
        return Ok(true);
    }
    send_signal_to_process_group(process_group_id, "KILL")?;
    wait_for_process_group_exit(process_group_id, PROCESS_TERMINATION_GRACE_MILLIS)
}

fn terminate_single_process(pid: u32) -> Result<bool> {
    let pid_text = pid.to_string();
    let term = Command::new("kill")
        .args(["-TERM", &pid_text])
        .output()
        .with_context(|| format!("kill -TERM {}", pid))?;
    if !term.status.success() {
        return Ok(false);
    }
    for _ in 0..15 {
        if !is_pid_alive(pid)? {
            return Ok(true);
        }
        thread::sleep(Duration::from_millis(100));
    }
    let kill = Command::new("kill")
        .args(["-KILL", &pid_text])
        .output()
        .with_context(|| format!("kill -KILL {}", pid))?;
    if !kill.status.success() {
        return Ok(false);
    }
    for _ in 0..15 {
        if !is_pid_alive(pid)? {
            return Ok(true);
        }
        thread::sleep(Duration::from_millis(100));
    }
    Ok(false)
}

fn send_signal_to_process_group(process_group_id: u32, signal: &str) -> Result<()> {
    let output = Command::new("kill")
        .args([&format!("-{signal}"), "--", &format!("-{process_group_id}")])
        .output()
        .with_context(|| format!("kill -{} -- -{}", signal, process_group_id))?;
    if !output.status.success() {
        anyhow::bail!(
            "kill -{} -- -{} failed: {}",
            signal,
            process_group_id,
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(())
}

fn wait_for_process_group_exit(process_group_id: u32, grace_millis: u64) -> Result<bool> {
    let max_checks = (grace_millis / 100).max(1);
    for _ in 0..max_checks {
        if !is_process_group_alive(process_group_id)? {
            return Ok(true);
        }
        thread::sleep(Duration::from_millis(100));
    }
    Ok(!is_process_group_alive(process_group_id)?)
}

fn is_process_group_alive(process_group_id: u32) -> Result<bool> {
    let output = Command::new("kill")
        .args(["-0", "--", &format!("-{process_group_id}")])
        .output()
        .with_context(|| format!("kill -0 -- -{}", process_group_id))?;
    Ok(output.status.success())
}

fn is_pid_alive(pid: u32) -> Result<bool> {
    let output = Command::new("kill")
        .args(["-0", &pid.to_string()])
        .output()
        .with_context(|| format!("kill -0 {}", pid))?;
    Ok(output.status.success())
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
    use std::{fs, process::Command};
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

    #[test]
    fn resolve_launch_isolation_boundary_prepares_runtime_dirs_for_git_workspace() {
        let dir = tempdir().unwrap();
        init_git_repo(dir.path());
        let request_path = dir
            .path()
            .join(".agentflow/tasks/AF-001/runs/run-001/launch/agent-request.json");
        fs::create_dir_all(request_path.parent().unwrap()).unwrap();
        fs::write(&request_path, "{\"task\":\"run\"}\n").unwrap();

        let mut request = McpLaunchRequest::new(
            "codex",
            "AF-001",
            "run-001",
            "build-agent",
            dir.path().display().to_string(),
            ".agentflow/tasks/AF-001/runs/run-001/launch/agent-request.json",
        );
        request.branch_name = Some("agentflow/direct/AF-001".to_string());

        let boundary =
            resolve_launch_isolation_boundary(dir.path(), &request, "codex-run-001", 1).unwrap();

        assert!(boundary.runtime_root.ends_with("runtime"));
        assert!(boundary.temp_root.is_dir());
        assert!(boundary.cache_root.is_dir());
        assert!(boundary.evidence_root.is_dir());
        assert!(boundary.worktree_root.is_dir());
        assert_ne!(boundary.worktree_root, dir.path());

        let branch = Command::new("git")
            .args(["branch", "--show-current"])
            .current_dir(&boundary.worktree_root)
            .output()
            .unwrap();
        assert!(branch.status.success());
        assert_eq!(
            String::from_utf8_lossy(&branch.stdout).trim(),
            "agentflow/direct/AF-001"
        );
    }

    fn init_git_repo(root: &Path) {
        let init = Command::new("git")
            .args(["init", "-b", "main"])
            .current_dir(root)
            .output()
            .unwrap();
        assert!(init.status.success());
        let config_name = Command::new("git")
            .args(["config", "user.name", "Codex"])
            .current_dir(root)
            .output()
            .unwrap();
        assert!(config_name.status.success());
        let config_email = Command::new("git")
            .args(["config", "user.email", "codex@example.com"])
            .current_dir(root)
            .output()
            .unwrap();
        assert!(config_email.status.success());
        fs::write(root.join("README.md"), "fixture\n").unwrap();
        let add = Command::new("git")
            .args(["add", "README.md"])
            .current_dir(root)
            .output()
            .unwrap();
        assert!(add.status.success());
        let commit = Command::new("git")
            .args(["commit", "-m", "initial fixture"])
            .current_dir(root)
            .output()
            .unwrap();
        assert!(commit.status.success());
    }
}
