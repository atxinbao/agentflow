use crate::{
    health::unix_timestamp_seconds,
    model::{
        McpCapability, McpLaunchMode, McpLaunchPlan, McpLaunchRequest, McpProviderKind,
        McpProviderStatus, McpProviderStatusCode, McpSessionSnapshot, McpSessionStatus,
    },
    provider::{
        ensure_parent_directory, relative_project_path, resolve_launch_isolation_boundary,
        run_command, spawn_exit_watcher, terminate_process_group, write_exit_proof,
        McpAgentProvider,
    },
    storage::{read_session_snapshot, write_launch_plan, write_session_snapshot},
};
use agentflow_projection::load_task_projection;
use anyhow::{Context, Result};
use serde_json::Value;
#[cfg(unix)]
use std::os::unix::process::CommandExt;
use std::{
    fs::{self, File, OpenOptions},
    io::Read,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::SystemTime,
};

const CLI_FRESHNESS_PATHS: [&str; 9] = [
    "Cargo.toml",
    "Cargo.lock",
    "crates/cli/src",
    "crates/spec/src",
    "crates/task-loop/src",
    "crates/task-artifacts/src",
    "crates/panel/src",
    "crates/agent-manual/src",
    "crates/projection/src",
];

const CLAUDE_PROGRAM: &str = "claude";
const DEFAULT_CLAUDE_MODEL: &str = "sonnet";
const AGENTFLOW_CLAUDE_MODEL_ENV: &str = "AGENTFLOW_CLAUDE_MODEL";
const ANTHROPIC_MODEL_ENV: &str = "ANTHROPIC_MODEL";
const CLAUDE_PERMISSION_MODE: &str = "bypassPermissions";
const CLAUDE_STDIN_PROMPT: &str =
    "Read the complete AgentFlow task package from stdin and execute it exactly as written.";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaudeCodeProvider {
    program: String,
    permission_mode: String,
    model: Option<String>,
}

impl Default for ClaudeCodeProvider {
    fn default() -> Self {
        Self {
            program: CLAUDE_PROGRAM.to_string(),
            permission_mode: CLAUDE_PERMISSION_MODE.to_string(),
            model: Some(detect_claude_model()),
        }
    }
}

impl ClaudeCodeProvider {
    pub fn new() -> Self {
        Self::default()
    }

    fn session_id(&self, request: &McpLaunchRequest) -> String {
        format!("claude-{}", sanitize_id(&request.run_id))
    }

    fn session_log_path(&self, session_id: &str, attempt_count: u32) -> String {
        if attempt_count <= 1 {
            format!(".agentflow/state/mcp/sessions/{session_id}.jsonl")
        } else {
            format!(".agentflow/state/mcp/sessions/{session_id}-attempt-{attempt_count}.jsonl")
        }
    }
}

fn detect_claude_model() -> String {
    env_model_override().unwrap_or_else(|| DEFAULT_CLAUDE_MODEL.to_string())
}

fn env_model_override() -> Option<String> {
    [AGENTFLOW_CLAUDE_MODEL_ENV, ANTHROPIC_MODEL_ENV]
        .into_iter()
        .find_map(|name| std::env::var(name).ok())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

pub fn check_claude_provider(project_root: impl AsRef<Path>) -> McpProviderStatus {
    let root = project_root.as_ref();
    let mut status = McpProviderStatus::new(McpProviderKind::ClaudeCode, unix_timestamp_seconds());
    status.cli = Some(CLAUDE_PROGRAM.to_string());

    match run_command(root, CLAUDE_PROGRAM, &["--version"]) {
        Ok(version) if version.status_success => {
            status.installed = true;
        }
        Ok(version) => {
            status.status = McpProviderStatusCode::Unavailable;
            status.errors.push(format!(
                "claude --version failed: {}",
                version.combined_output().trim()
            ));
            status.capabilities = claude_capabilities(false, false, false, false, false);
            return status;
        }
        Err(error) => {
            status.status = McpProviderStatusCode::Unavailable;
            status.errors.push(error.to_string());
            status.capabilities = claude_capabilities(false, false, false, false, false);
            return status;
        }
    }

    match run_command(root, CLAUDE_PROGRAM, &["auth", "status"]) {
        Ok(auth) if auth.status_success => {
            status.authenticated = Some(true);
        }
        Ok(auth) => {
            status.status = McpProviderStatusCode::Unauthenticated;
            status.authenticated = Some(false);
            status.errors.push(format!(
                "claude auth status failed: {}",
                auth.combined_output().trim()
            ));
            status.capabilities = claude_capabilities(false, false, false, false, false);
            return status;
        }
        Err(error) => {
            status.status = McpProviderStatusCode::Unauthenticated;
            status.authenticated = Some(false);
            status.errors.push(error.to_string());
            status.capabilities = claude_capabilities(false, false, false, false, false);
            return status;
        }
    }

    let help = match run_command(root, CLAUDE_PROGRAM, &["--help"]) {
        Ok(help) if help.status_success => help.combined_output(),
        Ok(help) => {
            status.status = McpProviderStatusCode::Unsupported;
            status.errors.push(format!(
                "claude --help failed: {}",
                help.combined_output().trim()
            ));
            status.capabilities = claude_capabilities(false, false, false, false, false);
            return status;
        }
        Err(error) => {
            status.status = McpProviderStatusCode::Unsupported;
            status.errors.push(error.to_string());
            status.capabilities = claude_capabilities(false, false, false, false, false);
            return status;
        }
    };

    let supports_print = help.contains("claude -p") || help.contains("--print");
    let cli_supports_logs = help.contains("claude logs <id>") || help.contains("claude logs ");
    let cli_supports_cancel = help.contains("claude stop <id>") || help.contains("claude stop ");
    let cli_supports_poll =
        help.contains("claude agents --json") || help.contains("claude agents ");
    let supports_logs = true;
    let supports_cancel = true;
    let supports_poll = true;
    if !cli_supports_logs {
        status.warnings.push(
            "claude CLI native log export is unavailable; using local runtime logs".to_string(),
        );
    }
    if !cli_supports_cancel {
        status.warnings.push(
            "claude CLI native cancel is unavailable; using local process termination".to_string(),
        );
    }
    if !cli_supports_poll {
        status.warnings.push(
            "claude CLI native polling is unavailable; using local runtime supervision".to_string(),
        );
    }
    status.capabilities = claude_capabilities(
        supports_print,
        supports_poll,
        supports_logs,
        supports_cancel,
        false,
    );

    let newest_source = newest_source_mtime(root).ok().flatten();
    let mut build_agent_complete_supported = false;

    for candidate in agentflow_cli_candidates(root) {
        if let Some((source_path, source_modified)) = newest_source.as_ref() {
            if is_local_target_binary(root, &candidate)
                && file_modified(&candidate)
                    .map(|binary_modified| binary_modified < *source_modified)
                    .unwrap_or(false)
            {
                status.warnings.push(format!(
                    "{} is stale; rebuild before use because {} is newer",
                    candidate.display(),
                    source_path.display()
                ));
                continue;
            }
        }
        let program = candidate.to_string_lossy().to_string();
        match run_command(root, &program, &["build-agent", "complete", "--help"]) {
            Ok(help) if help.status_success => {
                build_agent_complete_supported = true;
                break;
            }
            Ok(help) => {
                status.warnings.push(format!(
                    "{} does not support build-agent complete: {}",
                    program,
                    help.combined_output().trim()
                ));
            }
            Err(error) => status.warnings.push(format!("{program}: {error}")),
        }
    }

    status.capabilities = claude_capabilities(
        supports_print,
        supports_poll,
        supports_logs,
        supports_cancel,
        build_agent_complete_supported,
    );
    status.status = if supports_print && supports_poll {
        McpProviderStatusCode::Ready
    } else {
        McpProviderStatusCode::Unsupported
    };
    if !supports_print {
        status
            .errors
            .push("claude CLI is missing print-mode support".to_string());
    }
    status
}

fn claude_capabilities(
    launch: bool,
    poll: bool,
    logs: bool,
    cancel: bool,
    complete: bool,
) -> Vec<McpCapability> {
    vec![
        McpCapability::with_detail("launch", launch, "claude print-mode launch"),
        McpCapability::with_detail("claude.print", launch, "claude -p is available"),
        McpCapability::with_detail("session.poll", poll, "claude session polling"),
        McpCapability::with_detail("session.logs", logs, "claude session log access"),
        McpCapability::with_detail("session.cancel", cancel, "claude session cancellation"),
        McpCapability::with_detail(
            "build_agent.complete",
            complete,
            "agentflow build-agent complete is available",
        ),
    ]
}

impl McpAgentProvider for ClaudeCodeProvider {
    fn provider_id(&self) -> &'static str {
        "claude"
    }

    fn kind(&self) -> McpProviderKind {
        McpProviderKind::ClaudeCode
    }

    fn check_health(&self, project_root: &Path) -> McpProviderStatus {
        check_claude_provider(project_root)
    }

    fn build_launch_plan(
        &self,
        _project_root: &Path,
        request: &McpLaunchRequest,
    ) -> Result<McpLaunchPlan> {
        let session_id = self.session_id(request);
        let log_path = self.session_log_path(&session_id, 1);
        let stdin_path = request
            .prompt_path
            .clone()
            .unwrap_or_else(|| request.launch_request_path.clone());
        let mut plan = McpLaunchPlan::new(
            self.provider_id(),
            session_id,
            request.issue_id.clone(),
            request.run_id.clone(),
            McpLaunchMode::CliExecStdin,
            request.working_directory.clone(),
            self.program.clone(),
        );
        plan.args = vec![
            "-p".to_string(),
            CLAUDE_STDIN_PROMPT.to_string(),
            "--output-format".to_string(),
            "stream-json".to_string(),
            "--verbose".to_string(),
            "--permission-mode".to_string(),
            self.permission_mode.clone(),
        ];
        if let Some(model) = self.model.as_ref() {
            plan.args.push("--model".to_string());
            plan.args.push(model.clone());
        }
        plan.stdin_path = Some(stdin_path);
        plan.output_path = Some(log_path);
        plan.permission_mode = Some(self.permission_mode.clone());
        plan.supervision_mode = Some("local-process-watch".to_string());
        plan.note = Some("prompt via stdin".to_string());
        Ok(plan)
    }

    fn create_session(
        &self,
        project_root: &Path,
        request: &McpLaunchRequest,
    ) -> Result<McpSessionSnapshot> {
        let mut plan = self.build_launch_plan(project_root, request)?;
        let mut attempt_count = 1;
        let mut recovery_reason = None;
        if let Ok(existing) = read_session_snapshot(project_root, &plan.session_id) {
            if matches!(existing.status, McpSessionStatus::Cancelled)
                || matches!(
                    existing.status,
                    McpSessionStatus::Failed | McpSessionStatus::Interrupted
                ) && !existing.governance_facts.retryable
            {
                return Ok(existing);
            }
            if !matches!(
                existing.status,
                McpSessionStatus::Failed | McpSessionStatus::Interrupted
            ) {
                return Ok(existing);
            }
            if existing.attempt_count >= existing.governance_policy.max_attempts.max(1) {
                anyhow::bail!(
                    "session {} exhausted retry policy at attempt {}",
                    existing.session_id,
                    existing.attempt_count
                );
            }
            attempt_count = existing.attempt_count.max(1).saturating_add(1);
            recovery_reason = Some(format!("retry after {} session", existing.status.as_str()));
        }
        let log_path = self.session_log_path(&plan.session_id, attempt_count);
        plan.output_path = Some(log_path.clone());
        let isolation = resolve_launch_isolation_boundary(
            project_root,
            request,
            &plan.session_id,
            attempt_count,
        )?;
        plan.workspace_root = Some(isolation.workspace_root.display().to_string());
        plan.worktree_root = Some(isolation.worktree_root.display().to_string());
        plan.runtime_root = Some(isolation.runtime_root.display().to_string());
        plan.temp_root = Some(isolation.temp_root.display().to_string());
        plan.cache_root = Some(isolation.cache_root.display().to_string());
        plan.evidence_root = Some(isolation.evidence_root.display().to_string());
        plan.exit_proof_path = Some(relative_project_path(
            &isolation.workspace_root,
            &isolation.exit_proof_path,
        ));
        plan.note = Some(format!("attempt {attempt_count}; prompt via stdin"));

        write_launch_plan(project_root, &plan)?;
        let stdin_path = isolation.prompt_path.clone();
        let log_path = isolation.workspace_root.join(
            plan.output_path
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("claude launch plan is missing outputPath"))?,
        );
        ensure_parent_directory(&log_path)?;

        let stdin = File::open(&stdin_path)
            .with_context(|| format!("open claude prompt {}", stdin_path.display()))?;
        let stdout = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&log_path)
            .with_context(|| format!("open claude log {}", log_path.display()))?;
        let stderr = stdout
            .try_clone()
            .with_context(|| format!("clone claude log {}", log_path.display()))?;
        let mut command = Command::new(&plan.program);
        command
            .args(&plan.args)
            .current_dir(&isolation.worktree_root)
            .env("TMPDIR", &isolation.temp_root)
            .env("TMP", &isolation.temp_root)
            .env("TEMP", &isolation.temp_root)
            .env("XDG_CACHE_HOME", &isolation.cache_root)
            .env("AGENTFLOW_RUN_WORKTREE_ROOT", &isolation.worktree_root)
            .env("AGENTFLOW_RUN_TEMP_ROOT", &isolation.temp_root)
            .env("AGENTFLOW_RUN_CACHE_ROOT", &isolation.cache_root)
            .env("AGENTFLOW_RUN_EVIDENCE_DIR", &isolation.evidence_root)
            .stdin(Stdio::from(stdin))
            .stdout(Stdio::from(stdout))
            .stderr(Stdio::from(stderr));
        #[cfg(unix)]
        command.process_group(0);

        let now = unix_timestamp_seconds();
        let mut session = McpSessionSnapshot::queued(request, &plan, now);
        session.status = McpSessionStatus::Starting;
        session.working_directory = isolation.worktree_root.display().to_string();
        session.workspace_root = Some(isolation.workspace_root.display().to_string());
        session.worktree_root = Some(isolation.worktree_root.display().to_string());
        session.runtime_root = Some(isolation.runtime_root.display().to_string());
        session.temp_root = Some(isolation.temp_root.display().to_string());
        session.cache_root = Some(isolation.cache_root.display().to_string());
        session.evidence_root = Some(isolation.evidence_root.display().to_string());
        session.log_path = plan.output_path.clone();
        session.attempt_count = attempt_count;
        session.recovery_reason = recovery_reason;
        session.exit_proof_path = plan.exit_proof_path.clone();
        session.merge_proof_path = None;
        session.merge_state = None;
        session.writeback_state = None;
        session.last_error = None;
        session.permission_mode = Some(self.permission_mode.clone());
        session.supervision_mode = Some("local-process-watch".to_string());
        session.governance_facts.timeout_at = Some(now + session.governance_policy.timeout_seconds);
        session.governance_facts.resumed_from_attempt =
            (attempt_count > 1).then_some(attempt_count.saturating_sub(1));
        session.governance_facts.takeover_session_id =
            (attempt_count > 1).then_some(session.session_id.clone());
        session.governance_facts.retryable =
            attempt_count < session.governance_policy.max_attempts.max(1);
        session.governance_facts.terminal_reason = None;
        session.governance_facts.timed_out_at = None;
        session.governance_facts.cancel_requested_at = None;
        session.governance_facts.cancelled_at = None;
        session.note = plan.note.clone();
        session.updated_at = now;
        let child = match command.spawn().with_context(|| {
            format!(
                "spawn claude session for issue {} run {}",
                request.issue_id, request.run_id
            )
        }) {
            Ok(child) => child,
            Err(error) => {
                session.status = McpSessionStatus::Failed;
                session.pid = None;
                session.last_error = Some(error.to_string());
                session.governance_facts.retryable = false;
                session.governance_facts.terminal_reason = Some("launch-failed".to_string());
                session.exited_at = Some(now);
                write_session_snapshot(project_root, &session)?;
                if let Some(exit_proof_path) = session.exit_proof_path.clone() {
                    let absolute_exit_proof = isolation.workspace_root.join(exit_proof_path);
                    write_exit_proof(
                        &absolute_exit_proof,
                        &session,
                        "launch-failed",
                        None,
                        now,
                        session.last_error.as_deref(),
                    )?;
                }
                return Ok(session);
            }
        };
        let pid = child.id();
        session.pid = Some(pid);
        session.process_group_id = Some(pid);
        write_session_snapshot(project_root, &session)?;
        if let Some(exit_proof_path) = session.exit_proof_path.clone() {
            let absolute_exit_proof = isolation.workspace_root.join(exit_proof_path);
            spawn_exit_watcher(
                child,
                isolation.workspace_root.clone(),
                absolute_exit_proof,
                session.session_id.clone(),
                session.clone(),
            );
        }
        Ok(session)
    }

    fn poll_session(&self, project_root: &Path, session_id: &str) -> Result<McpSessionSnapshot> {
        let mut session = read_session_snapshot(project_root, session_id)?;
        let projection = load_task_projection(project_root, &session.issue_id).ok();
        let merge_proof = load_merge_proof(project_root, &session.issue_id, &session.run_id)
            .ok()
            .flatten();
        let pid_alive = session.pid.map(is_pid_alive).transpose()?;
        let mut process_alive = pid_alive.unwrap_or(false);
        let issue_state = projection
            .as_ref()
            .map(|value| value.current_state.as_str());
        let now = unix_timestamp_seconds();

        if process_alive
            && session
                .governance_facts
                .timeout_at
                .is_some_and(|timeout_at| timeout_at <= now)
            && session.governance_facts.timed_out_at.is_none()
        {
            let terminated =
                terminate_process_group(session.process_group_id, session.pid).unwrap_or(false);
            process_alive = false;
            session.governance_facts.timed_out_at = Some(now);
            session.governance_facts.terminal_reason = Some("timeout".to_string());
            session.recovery_reason = Some("retry after timeout".to_string());
            if terminated {
                session.exited_at = Some(now);
            }
        }
        if let Some(exit_proof_path) = session.exit_proof_path.clone() {
            let absolute_exit_proof = absolute_project_path(project_root, &exit_proof_path);
            if absolute_exit_proof.is_file() {
                let proof_content = fs::read_to_string(&absolute_exit_proof)
                    .with_context(|| format!("read {}", absolute_exit_proof.display()))?;
                if let Ok(proof) = serde_json::from_str::<Value>(&proof_content) {
                    session.exited_at = proof.get("exitedAt").and_then(Value::as_u64);
                    session.exit_code = proof
                        .get("exitCode")
                        .and_then(Value::as_i64)
                        .map(|value| value as i32);
                    if session.governance_facts.terminal_reason.is_none() {
                        session.governance_facts.terminal_reason = proof
                            .get("status")
                            .and_then(Value::as_str)
                            .map(str::to_string);
                    }
                    if session.last_error.is_none() {
                        session.last_error = proof
                            .get("lastError")
                            .and_then(Value::as_str)
                            .map(str::to_string);
                    }
                }
            }
        }

        if let Some(proof) = merge_proof.as_ref() {
            session.pr_url = proof.pr_url.clone();
            session.merge_proof_path = Some(format!(
                ".agentflow/tasks/{}/runs/{}/review/closeout-proof.json",
                session.issue_id, session.run_id
            ));
            session.merge_state = Some(if proof.merged {
                if proof.issue_closed {
                    "awaiting-public-delivery".to_string()
                } else {
                    "awaiting-closeout".to_string()
                }
            } else {
                "awaiting-merge".to_string()
            });
        } else {
            session.merge_proof_path = None;
            session.merge_state = None;
        }
        session.writeback_state = derive_writeback_state(issue_state, merge_proof.as_ref());
        session.status = derive_session_status(
            session.status.clone(),
            issue_state,
            process_alive,
            merge_proof.as_ref(),
        );
        session.last_error = derive_session_error(
            issue_state,
            process_alive,
            &session.status,
            session.last_error.clone(),
        );
        if !process_alive
            && matches!(
                session.status,
                McpSessionStatus::InReview
                    | McpSessionStatus::Done
                    | McpSessionStatus::Failed
                    | McpSessionStatus::Interrupted
                    | McpSessionStatus::Cancelled
            )
        {
            session.pid = None;
        }
        session.governance_facts.retryable = session.attempt_count
            < session.governance_policy.max_attempts
            && !matches!(
                session.status,
                McpSessionStatus::Cancelled | McpSessionStatus::Done
            );
        session.updated_at = now;
        write_session_snapshot(project_root, &session)?;
        Ok(session)
    }

    fn fetch_logs(
        &self,
        project_root: &Path,
        session_id: &str,
        cursor: Option<&str>,
    ) -> Result<crate::model::McpLogChunk> {
        let session = read_session_snapshot(project_root, session_id)?;
        let Some(log_path) = session.log_path.as_deref() else {
            return Ok(crate::model::McpLogChunk::empty(
                self.provider_id(),
                session_id,
            ));
        };
        let absolute_log_path = absolute_project_path(project_root, log_path);
        if !absolute_log_path.is_file() {
            return Ok(crate::model::McpLogChunk::empty(
                self.provider_id(),
                session_id,
            ));
        }
        let mut bytes = Vec::new();
        File::open(&absolute_log_path)
            .with_context(|| format!("open claude log {}", absolute_log_path.display()))?
            .read_to_end(&mut bytes)
            .with_context(|| format!("read claude log {}", absolute_log_path.display()))?;
        let start = cursor
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(0)
            .min(bytes.len());
        let lines = String::from_utf8_lossy(&bytes[start..])
            .lines()
            .map(str::to_string)
            .collect();
        Ok(crate::model::McpLogChunk {
            version: crate::model::MCP_LOG_CHUNK_VERSION.to_string(),
            provider: self.provider_id().to_string(),
            session_id: session_id.to_string(),
            cursor: Some(bytes.len().to_string()),
            lines,
        })
    }

    fn cancel_session(&self, project_root: &Path, session_id: &str) -> Result<McpSessionSnapshot> {
        let mut session = read_session_snapshot(project_root, session_id)?;
        let terminated = terminate_process_group(session.process_group_id, session.pid)?;
        let now = unix_timestamp_seconds();
        session.status = McpSessionStatus::Cancelled;
        session.updated_at = now;
        session.last_error = None;
        session.governance_facts.cancel_requested_at = Some(now);
        if terminated {
            session.governance_facts.cancelled_at = Some(now);
        }
        session.governance_facts.retryable = false;
        session.governance_facts.terminal_reason = Some("cancelled".to_string());
        write_session_snapshot(project_root, &session)?;
        Ok(session)
    }
}

#[derive(Debug, Clone)]
struct MergeProofSummary {
    pr_url: Option<String>,
    merged: bool,
    issue_closed: bool,
    public_delivery_written: bool,
}

fn absolute_project_path(project_root: &Path, value: &str) -> PathBuf {
    let path = Path::new(value);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        project_root.join(path)
    }
}

fn load_merge_proof(
    project_root: &Path,
    issue_id: &str,
    run_id: &str,
) -> Result<Option<MergeProofSummary>> {
    let path = project_root.join(format!(
        ".agentflow/tasks/{issue_id}/runs/{run_id}/review/closeout-proof.json"
    ));
    if !path.is_file() {
        return Ok(None);
    }
    let proof: Value = serde_json::from_str(
        &fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?,
    )
    .with_context(|| format!("parse {}", path.display()))?;
    Ok(Some(MergeProofSummary {
        pr_url: proof
            .get("prUrl")
            .and_then(Value::as_str)
            .or_else(|| proof.get("remoteUrl").and_then(Value::as_str))
            .map(str::to_string),
        merged: proof
            .get("merged")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        issue_closed: proof
            .get("issueClosed")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        public_delivery_written: proof
            .get("publicDeliveryWritten")
            .and_then(Value::as_bool)
            .unwrap_or(false),
    }))
}

fn derive_writeback_state(
    issue_state: Option<&str>,
    merge_proof: Option<&MergeProofSummary>,
) -> Option<String> {
    match issue_state {
        Some("done") => Some("completed".to_string()),
        Some("in_review")
            if merge_proof.is_some_and(|proof| proof.merged && proof.issue_closed) =>
        {
            Some("awaiting-complete".to_string())
        }
        Some("blocked") => Some("failed".to_string()),
        Some("cancel") => Some("cancelled".to_string()),
        _ => None,
    }
}

fn derive_session_status(
    current: McpSessionStatus,
    issue_state: Option<&str>,
    process_alive: bool,
    merge_proof: Option<&MergeProofSummary>,
) -> McpSessionStatus {
    if let Some(proof) = merge_proof {
        if proof.merged && proof.issue_closed && proof.public_delivery_written {
            return McpSessionStatus::Done;
        }
        return McpSessionStatus::InReview;
    }

    match issue_state {
        Some("done") => return McpSessionStatus::Done,
        Some("in_review") => return McpSessionStatus::InReview,
        Some("cancel") => return McpSessionStatus::Cancelled,
        Some("blocked") => return McpSessionStatus::Failed,
        Some("in_progress") if process_alive => return McpSessionStatus::Running,
        Some("todo") if process_alive => return McpSessionStatus::Starting,
        _ => {}
    }

    if process_alive {
        return McpSessionStatus::Running;
    }

    if matches!(
        current,
        McpSessionStatus::Claimed | McpSessionStatus::Starting | McpSessionStatus::Running
    ) {
        McpSessionStatus::Interrupted
    } else {
        current
    }
}

fn derive_session_error(
    issue_state: Option<&str>,
    process_alive: bool,
    status: &McpSessionStatus,
    previous: Option<String>,
) -> Option<String> {
    match issue_state {
        Some("blocked") => Some("任务已阻断。".to_string()),
        _ if !process_alive && matches!(status, McpSessionStatus::Interrupted) => {
            Some("外部 Claude Code 会话已中断，任务还没有进入评审或完成状态。".to_string())
        }
        _ if matches!(status, McpSessionStatus::Done | McpSessionStatus::InReview) => None,
        _ => previous,
    }
}

fn is_pid_alive(pid: u32) -> Result<bool> {
    let output = Command::new("kill")
        .args(["-0", &pid.to_string()])
        .output()
        .with_context(|| format!("probe pid {pid}"))?;
    Ok(output.status.success())
}

fn agentflow_cli_candidates(project_root: &Path) -> Vec<PathBuf> {
    vec![
        project_root.join("target/debug/agentflow"),
        project_root.join("target/release/agentflow"),
        PathBuf::from("agentflow"),
    ]
}

fn newest_source_mtime(root: &Path) -> std::io::Result<Option<(PathBuf, SystemTime)>> {
    let mut newest = None;
    for relative in CLI_FRESHNESS_PATHS {
        collect_newest_mtime(&root.join(relative), &mut newest)?;
    }
    Ok(newest)
}

fn collect_newest_mtime(
    path: &Path,
    newest: &mut Option<(PathBuf, SystemTime)>,
) -> std::io::Result<()> {
    if !path.exists() {
        return Ok(());
    }
    let metadata = fs::metadata(path)?;
    if metadata.is_file() {
        let modified = metadata.modified()?;
        let replace = newest
            .as_ref()
            .map(|(_, current)| modified > *current)
            .unwrap_or(true);
        if replace {
            *newest = Some((path.to_path_buf(), modified));
        }
        return Ok(());
    }
    if metadata.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            collect_newest_mtime(&entry.path(), newest)?;
        }
    }
    Ok(())
}

fn file_modified(path: &Path) -> std::io::Result<SystemTime> {
    Ok(fs::metadata(path)?.modified()?)
}

fn is_local_target_binary(root: &Path, executable: &Path) -> bool {
    executable.starts_with(root.join("target"))
        && executable
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| name == "agentflow")
            .unwrap_or(false)
}

fn sanitize_id(id: &str) -> String {
    id.chars()
        .map(|ch| match ch {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => ch,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{agentflow_cli_candidates, is_local_target_binary, ClaudeCodeProvider};
    use crate::{model::McpLaunchRequest, provider::McpAgentProvider};
    use std::path::Path;

    #[test]
    fn prefers_debug_agentflow_before_release_binary() {
        let candidates = agentflow_cli_candidates(Path::new("/repo"));
        assert_eq!(candidates[0], Path::new("/repo/target/debug/agentflow"));
        assert_eq!(candidates[1], Path::new("/repo/target/release/agentflow"));
    }

    #[test]
    fn local_target_binary_detection_matches_workspace_targets() {
        assert!(is_local_target_binary(
            Path::new("/repo"),
            Path::new("/repo/target/debug/agentflow")
        ));
        assert!(!is_local_target_binary(
            Path::new("/repo"),
            Path::new("/usr/local/bin/agentflow")
        ));
    }

    #[test]
    fn claude_provider_builds_print_launch_plan() {
        let provider = ClaudeCodeProvider::new();
        let request = McpLaunchRequest::new(
            "claude",
            "AF-001",
            "run-001",
            "build-agent",
            "/repo",
            ".agentflow/tasks/AF-001/runs/run-001/launch/agent-request.json",
        );

        let plan = provider
            .build_launch_plan(Path::new("/repo"), &request)
            .unwrap();
        assert_eq!(plan.program, "claude");
        assert_eq!(plan.session_id, "claude-run-001");
        assert!(plan.args.iter().any(|arg| arg == "-p"));
        assert!(plan.args.iter().any(|arg| arg == "--output-format"));
        assert!(plan.args.iter().any(|arg| arg == "stream-json"));
        assert!(plan.args.iter().any(|arg| arg == "--verbose"));
        assert!(plan.args.iter().any(|arg| arg == "--permission-mode"));
        assert!(plan.args.iter().any(|arg| arg == "bypassPermissions"));
        assert!(plan.args.iter().any(|arg| arg == "--model"));
        assert!(plan.args.iter().any(|arg| arg == "sonnet"));
        assert_eq!(
            plan.stdin_path.as_deref(),
            Some(".agentflow/tasks/AF-001/runs/run-001/launch/agent-request.json")
        );
        assert_eq!(
            plan.output_path.as_deref(),
            Some(".agentflow/state/mcp/sessions/claude-run-001.jsonl")
        );
    }
}
