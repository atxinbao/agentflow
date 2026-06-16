use crate::{
    health::unix_timestamp_seconds,
    model::{
        McpCapability, McpLaunchMode, McpLaunchPlan, McpLaunchRequest, McpProviderKind,
        McpProviderStatus, McpProviderStatusCode, McpSessionSnapshot, McpSessionStatus,
    },
    provider::{run_command, McpAgentProvider},
    storage::{read_session_snapshot, write_launch_plan, write_session_snapshot},
};
use agentflow_projection::load_task_projection;
use anyhow::{Context, Result};
use serde_json::Value;
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

const CODEX_PROGRAM: &str = "codex";
const DEFAULT_CODEX_MODEL: &str = "gpt-5.5";
const AGENTFLOW_CODEX_MODEL_ENV: &str = "AGENTFLOW_CODEX_MODEL";
const CODEX_MODEL_ENV: &str = "CODEX_MODEL";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodexProvider {
    program: String,
    sandbox: String,
    approval_policy: String,
    model: Option<String>,
}

impl Default for CodexProvider {
    fn default() -> Self {
        Self {
            program: CODEX_PROGRAM.to_string(),
            sandbox: "workspace-write".to_string(),
            approval_policy: "never".to_string(),
            model: Some(detect_codex_model()),
        }
    }
}

impl CodexProvider {
    pub fn new() -> Self {
        Self::default()
    }

    fn session_id(&self, request: &McpLaunchRequest) -> String {
        format!("codex-{}", sanitize_id(&request.run_id))
    }

    fn session_log_path(&self, session_id: &str) -> String {
        format!(".agentflow/state/mcp/sessions/{session_id}.jsonl")
    }

    fn session_last_message_path(&self, session_id: &str) -> String {
        format!(".agentflow/state/mcp/sessions/{session_id}-last-message.txt")
    }
}

fn detect_codex_model() -> String {
    env_model_override()
        .or_else(configured_codex_model)
        .unwrap_or_else(|| DEFAULT_CODEX_MODEL.to_string())
}

fn env_model_override() -> Option<String> {
    [AGENTFLOW_CODEX_MODEL_ENV, CODEX_MODEL_ENV]
        .into_iter()
        .find_map(|name| std::env::var(name).ok())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn configured_codex_model() -> Option<String> {
    let home = std::env::var_os("HOME")?;
    let path = Path::new(&home).join(".codex/config.toml");
    configured_codex_model_from_path(&path)
}

fn configured_codex_model_from_path(path: &Path) -> Option<String> {
    let content = fs::read_to_string(path).ok()?;
    configured_codex_model_from_content(&content)
}

fn configured_codex_model_from_content(content: &str) -> Option<String> {
    for raw_line in content.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with('[') {
            continue;
        }
        let (key, value) = line.split_once('=')?;
        if key.trim() != "model" {
            continue;
        }
        let value = value.trim();
        if let Some(stripped) = value
            .strip_prefix('"')
            .and_then(|value| value.strip_suffix('"'))
        {
            let model = stripped.trim();
            if !model.is_empty() {
                return Some(model.to_string());
            }
        }
    }
    None
}

pub fn check_codex_provider(project_root: impl AsRef<Path>) -> McpProviderStatus {
    let root = project_root.as_ref();
    let mut status = McpProviderStatus::new(McpProviderKind::Codex, unix_timestamp_seconds());
    let mut codex_exec_available = false;
    match run_command(root, CODEX_PROGRAM, &["exec", "--help"]) {
        Ok(help) if help.status_success => {
            status.installed = true;
            status.cli = Some(CODEX_PROGRAM.to_string());
            codex_exec_available = true;
            status.capabilities.push(McpCapability::with_detail(
                "launch",
                true,
                "codex exec launch is available",
            ));
            status.capabilities.push(McpCapability::with_detail(
                "codex.exec",
                true,
                "codex exec is available",
            ));
        }
        Ok(help) => {
            status.warnings.push(format!(
                "codex exec is unavailable: {}",
                help.combined_output().trim()
            ));
            status.capabilities.push(McpCapability::with_detail(
                "launch",
                false,
                "codex exec launch is unavailable",
            ));
            status.capabilities.push(McpCapability::with_detail(
                "codex.exec",
                false,
                "codex exec is unavailable",
            ));
        }
        Err(error) => {
            status.warnings.push(format!("{CODEX_PROGRAM}: {error}"));
            status.capabilities.push(McpCapability::with_detail(
                "launch",
                false,
                "codex exec launch is unavailable",
            ));
            status.capabilities.push(McpCapability::with_detail(
                "codex.exec",
                false,
                "codex exec is unavailable",
            ));
        }
    }

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
                status.capabilities.push(McpCapability::with_detail(
                    "build_agent.complete",
                    true,
                    "agentflow build-agent complete is available",
                ));
                break;
            }
            Ok(help) => {
                status.warnings.push(format!(
                    "{} does not support build-agent complete: {}",
                    program,
                    help.combined_output().trim()
                ));
            }
            Err(error) => {
                status.warnings.push(format!("{program}: {error}"));
            }
        }
    }

    if !build_agent_complete_supported {
        status.capabilities.push(McpCapability::with_detail(
            "build_agent.complete",
            false,
            "agentflow build-agent complete is unavailable",
        ));
    }

    status.status = if codex_exec_available {
        McpProviderStatusCode::Ready
    } else {
        McpProviderStatusCode::Unavailable
    };
    status
}

impl McpAgentProvider for CodexProvider {
    fn provider_id(&self) -> &'static str {
        "codex"
    }

    fn kind(&self) -> McpProviderKind {
        McpProviderKind::Codex
    }

    fn check_health(&self, project_root: &Path) -> McpProviderStatus {
        check_codex_provider(project_root)
    }

    fn build_launch_plan(
        &self,
        _project_root: &Path,
        request: &McpLaunchRequest,
    ) -> Result<McpLaunchPlan> {
        let session_id = self.session_id(request);
        let log_path = self.session_log_path(&session_id);
        let last_message_path = self.session_last_message_path(&session_id);
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
            "--ask-for-approval".to_string(),
            self.approval_policy.clone(),
            "exec".to_string(),
            "--ephemeral".to_string(),
            "--ignore-user-config".to_string(),
            "--ignore-rules".to_string(),
            "--cd".to_string(),
            request.working_directory.clone(),
            "--sandbox".to_string(),
            self.sandbox.clone(),
            "--json".to_string(),
            "--output-last-message".to_string(),
            last_message_path.clone(),
        ];
        if let Some(model) = self.model.as_ref() {
            plan.args.push("--model".to_string());
            plan.args.push(model.clone());
        }
        plan.args.push("-".to_string());
        plan.stdin_path = Some(stdin_path);
        plan.output_path = Some(log_path.clone());
        plan.note = Some(format!("last message path: {last_message_path}"));
        Ok(plan)
    }

    fn create_session(
        &self,
        project_root: &Path,
        request: &McpLaunchRequest,
    ) -> Result<McpSessionSnapshot> {
        let plan = self.build_launch_plan(project_root, request)?;
        if let Ok(existing) = read_session_snapshot(project_root, &plan.session_id) {
            if !matches!(
                existing.status,
                McpSessionStatus::Failed
                    | McpSessionStatus::Cancelled
                    | McpSessionStatus::Interrupted
            ) {
                return Ok(existing);
            }
        }

        write_launch_plan(project_root, &plan)?;
        let stdin_path = absolute_project_path(
            project_root,
            plan.stdin_path
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("codex launch plan is missing stdinPath"))?,
        );
        let log_path = absolute_project_path(
            project_root,
            plan.output_path
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("codex launch plan is missing outputPath"))?,
        );
        ensure_parent_directory(&log_path)?;

        let stdin = File::open(&stdin_path)
            .with_context(|| format!("open codex prompt {}", stdin_path.display()))?;
        let stdout = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&log_path)
            .with_context(|| format!("open codex log {}", log_path.display()))?;
        let stderr = stdout
            .try_clone()
            .with_context(|| format!("clone codex log {}", log_path.display()))?;
        let mut command = Command::new(&plan.program);
        command
            .args(&plan.args)
            .current_dir(project_root)
            .stdin(Stdio::from(stdin))
            .stdout(Stdio::from(stdout))
            .stderr(Stdio::from(stderr));
        let mut child = command.spawn().with_context(|| {
            format!(
                "spawn codex session for issue {} run {}",
                request.issue_id, request.run_id
            )
        })?;
        let pid = child.id();
        std::thread::spawn(move || {
            let _ = child.wait();
        });

        let now = unix_timestamp_seconds();
        let mut session = McpSessionSnapshot::queued(request, &plan, now);
        session.status = McpSessionStatus::Starting;
        session.pid = Some(pid);
        session.log_path = plan.output_path.clone();
        session.updated_at = now;
        write_session_snapshot(project_root, &session)?;
        Ok(session)
    }

    fn poll_session(&self, project_root: &Path, session_id: &str) -> Result<McpSessionSnapshot> {
        let mut session = read_session_snapshot(project_root, session_id)?;
        let projection = load_task_projection(project_root, &session.issue_id).ok();
        let merge_proof = load_merge_proof(project_root, &session.issue_id, &session.run_id)
            .ok()
            .flatten();
        let pid_alive = session.pid.map(is_pid_alive).transpose()?;
        let process_alive = pid_alive.unwrap_or(false);

        if let Some(proof) = merge_proof.as_ref() {
            session.pr_url = proof.remote_url.clone();
            session.merge_state = Some(if proof.merged {
                "merged".to_string()
            } else {
                "awaiting-merge".to_string()
            });
        }

        session.status = derive_session_status(
            session.status.clone(),
            projection
                .as_ref()
                .map(|value| value.current_state.as_str()),
            process_alive,
            merge_proof.as_ref(),
        );

        session.last_error = derive_session_error(
            projection
                .as_ref()
                .map(|value| value.current_state.as_str()),
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
        session.updated_at = unix_timestamp_seconds();
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
            .with_context(|| format!("open codex log {}", absolute_log_path.display()))?
            .read_to_end(&mut bytes)
            .with_context(|| format!("read codex log {}", absolute_log_path.display()))?;
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
}

#[derive(Debug, Clone)]
struct MergeProofSummary {
    remote_url: Option<String>,
    merged: bool,
}

fn absolute_project_path(project_root: &Path, value: &str) -> PathBuf {
    let path = Path::new(value);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        project_root.join(path)
    }
}

fn ensure_parent_directory(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    Ok(())
}

fn load_merge_proof(
    project_root: &Path,
    issue_id: &str,
    run_id: &str,
) -> Result<Option<MergeProofSummary>> {
    let path = project_root.join(format!(
        ".agentflow/tasks/{issue_id}/runs/{run_id}/review/merge-proof.json"
    ));
    if !path.is_file() {
        return Ok(None);
    }
    let proof: Value = serde_json::from_str(
        &fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?,
    )
    .with_context(|| format!("parse {}", path.display()))?;
    Ok(Some(MergeProofSummary {
        remote_url: proof
            .get("remoteUrl")
            .and_then(Value::as_str)
            .map(str::to_string),
        merged: proof
            .get("merged")
            .and_then(Value::as_bool)
            .unwrap_or(false),
    }))
}

fn derive_session_status(
    current: McpSessionStatus,
    issue_state: Option<&str>,
    process_alive: bool,
    merge_proof: Option<&MergeProofSummary>,
) -> McpSessionStatus {
    if let Some(proof) = merge_proof {
        if proof.merged {
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
            Some("外部执行会话已中断，任务还没有进入评审或完成状态。".to_string())
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
    use super::{
        agentflow_cli_candidates, configured_codex_model_from_content, is_local_target_binary,
        CodexProvider, DEFAULT_CODEX_MODEL,
    };
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
    fn codex_provider_builds_exec_launch_plan() {
        let provider = CodexProvider::new();
        let request = McpLaunchRequest::new(
            "codex",
            "AF-001",
            "run-001",
            "build-agent",
            "/repo",
            ".agentflow/tasks/AF-001/runs/run-001/launch/agent-request.json",
        );

        let plan = provider
            .build_launch_plan(Path::new("/repo"), &request)
            .unwrap();
        assert_eq!(plan.program, "codex");
        assert_eq!(plan.session_id, "codex-run-001");
        assert_eq!(plan.args[0], "--ask-for-approval");
        assert!(plan.args.iter().any(|arg| arg == "exec"));
        assert!(plan.args.iter().any(|arg| arg == "--ephemeral"));
        assert!(plan.args.iter().any(|arg| arg == "--ignore-user-config"));
        assert!(plan.args.iter().any(|arg| arg == "--ignore-rules"));
        assert!(plan.args.iter().any(|arg| arg == "--json"));
        assert!(plan.args.iter().any(|arg| arg == "--output-last-message"));
        assert!(plan.args.iter().any(|arg| arg == "--model"));
        assert!(plan.args.iter().any(|arg| arg == DEFAULT_CODEX_MODEL));
        assert_eq!(
            plan.stdin_path.as_deref(),
            Some(".agentflow/tasks/AF-001/runs/run-001/launch/agent-request.json")
        );
        assert_eq!(
            plan.output_path.as_deref(),
            Some(".agentflow/state/mcp/sessions/codex-run-001.jsonl")
        );
    }

    #[test]
    fn parses_model_from_codex_config_content() {
        let content = r#"
model = "gpt-5.5"
approval_policy = "never"
"#;
        assert_eq!(
            configured_codex_model_from_content(content).as_deref(),
            Some("gpt-5.5")
        );
    }

    #[test]
    fn ignores_non_top_level_model_content() {
        let content = r#"
[projects."/repo"]
trust_level = "trusted"

# model = "comment-only"
"#;
        assert_eq!(configured_codex_model_from_content(content), None);
    }

    #[test]
    fn claimed_launch_without_live_process_interrupts_session() {
        let status = super::derive_session_status(
            crate::model::McpSessionStatus::Running,
            Some("in_progress"),
            false,
            None,
        );
        assert_eq!(status, crate::model::McpSessionStatus::Interrupted);
    }

    #[test]
    fn queued_launch_without_live_process_interrupts_started_session() {
        let status = super::derive_session_status(
            crate::model::McpSessionStatus::Starting,
            Some("todo"),
            false,
            None,
        );
        assert_eq!(status, crate::model::McpSessionStatus::Interrupted);
    }
}
