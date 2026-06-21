use crate::{
    health::unix_timestamp_seconds,
    model::{McpLaunchPlan, McpProviderStatus, McpRegistry, McpSessionSnapshot},
    registry::build_registry,
};
use agentflow_task_artifacts::{
    sync_task_session, task_run_dir, TaskSessionMirror, TaskWorkSessionStatus,
};
use anyhow::{Context, Result};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};
use tempfile::NamedTempFile;

pub fn prepare_mcp_workspace(project_root: impl AsRef<Path>) -> Result<()> {
    let root = canonical_project_root(project_root)?;
    ensure_directory(&root.join(".agentflow/state/mcp/providers"))?;
    ensure_directory(&root.join(".agentflow/state/mcp/plans"))?;
    ensure_directory(&root.join(".agentflow/state/mcp/sessions"))?;
    Ok(())
}

pub fn write_provider_status(
    project_root: impl AsRef<Path>,
    status: &McpProviderStatus,
) -> Result<PathBuf> {
    let root = canonical_project_root(project_root)?;
    prepare_mcp_workspace(&root)?;
    let path = provider_status_path(&root, &status.provider);
    write_json(&path, status)?;
    Ok(path)
}

pub fn read_provider_status(
    project_root: impl AsRef<Path>,
    provider: &str,
) -> Result<McpProviderStatus> {
    let root = canonical_project_root(project_root)?;
    read_json(&provider_status_path(&root, provider))
}

pub fn write_registry(project_root: impl AsRef<Path>, registry: &McpRegistry) -> Result<PathBuf> {
    let root = canonical_project_root(project_root)?;
    prepare_mcp_workspace(&root)?;
    let path = root.join(".agentflow/state/mcp/registry.json");
    write_json(&path, registry)?;
    Ok(path)
}

pub fn read_registry(project_root: impl AsRef<Path>) -> Result<McpRegistry> {
    let root = canonical_project_root(project_root)?;
    read_json(&root.join(".agentflow/state/mcp/registry.json"))
}

pub fn write_registry_for_statuses(
    project_root: impl AsRef<Path>,
    statuses: &[McpProviderStatus],
) -> Result<McpRegistry> {
    let root = canonical_project_root(project_root)?;
    let registry = build_registry(statuses, unix_timestamp_seconds());
    write_registry(&root, &registry)?;
    Ok(registry)
}

pub fn write_launch_plan(project_root: impl AsRef<Path>, plan: &McpLaunchPlan) -> Result<PathBuf> {
    let root = canonical_project_root(project_root)?;
    prepare_mcp_workspace(&root)?;
    let path = launch_plan_path(&root, &plan.session_id);
    write_json(&path, plan)?;
    Ok(path)
}

pub fn read_launch_plan(project_root: impl AsRef<Path>, session_id: &str) -> Result<McpLaunchPlan> {
    let root = canonical_project_root(project_root)?;
    read_json(&launch_plan_path(&root, session_id))
}

pub fn write_session_snapshot(
    project_root: impl AsRef<Path>,
    session: &McpSessionSnapshot,
) -> Result<PathBuf> {
    let root = canonical_project_root(project_root)?;
    prepare_mcp_workspace(&root)?;
    let path = session_snapshot_path(&root, &session.session_id);
    write_json(&path, session)?;
    sync_task_session_if_present(&root, session)?;
    Ok(path)
}

pub fn read_session_snapshot(
    project_root: impl AsRef<Path>,
    session_id: &str,
) -> Result<McpSessionSnapshot> {
    let root = canonical_project_root(project_root)?;
    read_json(&session_snapshot_path(&root, session_id))
}

pub fn load_session_snapshots(project_root: impl AsRef<Path>) -> Result<Vec<McpSessionSnapshot>> {
    let root = canonical_project_root(project_root)?;
    prepare_mcp_workspace(&root)?;
    let mut sessions = Vec::new();
    for entry in fs::read_dir(root.join(".agentflow/state/mcp/sessions")).with_context(|| {
        format!(
            "read {}",
            root.join(".agentflow/state/mcp/sessions").display()
        )
    })? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        sessions.push(read_json::<McpSessionSnapshot>(&path)?);
    }
    sessions.sort_by(|left, right| {
        right
            .updated_at
            .cmp(&left.updated_at)
            .then_with(|| left.session_id.cmp(&right.session_id))
    });
    Ok(sessions)
}

pub fn find_session_snapshot_by_run(
    project_root: impl AsRef<Path>,
    run_id: &str,
) -> Result<Option<McpSessionSnapshot>> {
    Ok(load_session_snapshots(project_root)?
        .into_iter()
        .find(|session| session.run_id == run_id))
}

fn provider_status_path(root: &Path, provider: &str) -> PathBuf {
    root.join(".agentflow/state/mcp/providers")
        .join(format!("{}.json", sanitize_id(provider)))
}

fn launch_plan_path(root: &Path, session_id: &str) -> PathBuf {
    root.join(".agentflow/state/mcp/plans")
        .join(format!("{}.json", sanitize_id(session_id)))
}

fn session_snapshot_path(root: &Path, session_id: &str) -> PathBuf {
    root.join(".agentflow/state/mcp/sessions")
        .join(format!("{}.json", sanitize_id(session_id)))
}

fn sanitize_id(id: &str) -> String {
    id.chars()
        .map(|ch| match ch {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => ch,
        })
        .collect()
}

fn canonical_project_root(project_root: impl AsRef<Path>) -> Result<PathBuf> {
    project_root
        .as_ref()
        .canonicalize()
        .with_context(|| format!("canonicalize {}", project_root.as_ref().display()))
}

fn sync_task_session_if_present(root: &Path, session: &McpSessionSnapshot) -> Result<()> {
    let run_directory = task_run_dir(root, &session.issue_id, &session.run_id)?;
    if !run_directory.join("run.json").is_file() {
        return Ok(());
    }
    let mirror = TaskSessionMirror {
        provider: session.provider.clone(),
        session_owner: session.owner_id.clone(),
        session_id: session.session_id.clone(),
        status: task_session_status(&session.status),
        branch_name: session.branch_name.clone(),
        working_directory: session.working_directory.clone(),
        workspace_root: session.workspace_root.clone(),
        worktree_root: session.worktree_root.clone(),
        runtime_root: session.runtime_root.clone(),
        temp_root: session.temp_root.clone(),
        cache_root: session.cache_root.clone(),
        evidence_root: session.evidence_root.clone(),
        launch_request_path: session.launch_request_path.clone(),
        plan_path: session.plan_path.clone(),
        log_path: session.log_path.clone(),
        last_message_path: session.last_message_path.clone(),
        exit_proof_path: session.exit_proof_path.clone(),
        merge_proof_path: session.merge_proof_path.clone(),
        started_at: session.started_at,
        last_heartbeat_at: session.last_heartbeat_at,
        attempt_count: session.attempt_count,
        retry_policy: Some(session.governance_policy.retry_policy.clone()),
        max_attempts: Some(session.governance_policy.max_attempts),
        resumed_from_attempt: session.governance_facts.resumed_from_attempt,
        retryable: session.governance_facts.retryable,
        recovery_reason: session.recovery_reason.clone(),
        merge_state: session.merge_state.clone(),
        writeback_state: session.writeback_state.clone(),
        terminal_reason: session.governance_facts.terminal_reason.clone(),
        last_error: session.last_error.clone(),
        exited_at: session.exited_at,
        exit_code: session.exit_code,
        updated_at: session.updated_at,
    };
    let _ = sync_task_session(root, &session.issue_id, &session.run_id, &mirror)?;
    Ok(())
}

fn task_session_status(status: &crate::model::McpSessionStatus) -> TaskWorkSessionStatus {
    match status {
        crate::model::McpSessionStatus::Queued => TaskWorkSessionStatus::Queued,
        crate::model::McpSessionStatus::Claimed => TaskWorkSessionStatus::Claimed,
        crate::model::McpSessionStatus::Starting => TaskWorkSessionStatus::Starting,
        crate::model::McpSessionStatus::Running => TaskWorkSessionStatus::Running,
        crate::model::McpSessionStatus::InReview => TaskWorkSessionStatus::InReview,
        crate::model::McpSessionStatus::Done => TaskWorkSessionStatus::Done,
        crate::model::McpSessionStatus::Interrupted => TaskWorkSessionStatus::Interrupted,
        crate::model::McpSessionStatus::Failed => TaskWorkSessionStatus::Failed,
        crate::model::McpSessionStatus::Cancelled => TaskWorkSessionStatus::Cancelled,
    }
}

fn ensure_directory(path: &Path) -> Result<()> {
    fs::create_dir_all(path).with_context(|| format!("create {}", path.display()))
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

fn write_json<T: serde::Serialize>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        ensure_directory(parent)?;
    }
    atomic_write_text(path, &(serde_json::to_string_pretty(value)? + "\n"))
        .with_context(|| format!("write {}", path.display()))
}

fn atomic_write_text(path: &Path, content: &str) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("{} has no parent directory", path.display()))?;
    ensure_directory(parent)?;
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
    use super::*;
    use crate::model::{
        McpLaunchMode, McpLaunchPlan, McpProviderKind, McpProviderStatus, McpProviderStatusCode,
        McpSessionGovernanceFacts, McpSessionGovernancePolicy, McpSessionSnapshot,
        McpSessionStatus,
    };
    use tempfile::tempdir;

    #[test]
    fn writes_and_reads_provider_status() {
        let dir = tempdir().unwrap();
        let mut status = McpProviderStatus::new(McpProviderKind::Github, 100);
        status.status = McpProviderStatusCode::Ready;

        let path = write_provider_status(dir.path(), &status).unwrap();
        assert!(path.ends_with(".agentflow/state/mcp/providers/github.json"));

        let loaded = read_provider_status(dir.path(), "github").unwrap();
        assert_eq!(loaded, status);
    }

    #[test]
    fn writes_registry_for_provider_statuses() {
        let dir = tempdir().unwrap();
        let status = McpProviderStatus::new(McpProviderKind::Codex, 101);

        write_provider_status(dir.path(), &status).unwrap();
        let registry = write_registry_for_statuses(dir.path(), &[status]).unwrap();
        let loaded = read_registry(dir.path()).unwrap();

        assert_eq!(loaded, registry);
        assert_eq!(loaded.providers[0].provider, "codex");
    }

    #[test]
    fn writes_launch_plan_and_session_snapshot() {
        let dir = tempdir().unwrap();
        agentflow_task_artifacts::create_task_run(
            dir.path(),
            "AF-001",
            "run-001",
            "work-agent.issue-loop@v1",
            Some("agentflow/proj-001/AF-001".to_string()),
        )
        .unwrap();
        let mut plan = McpLaunchPlan::new(
            "codex",
            "codex-run-001",
            "AF-001",
            "run-001",
            McpLaunchMode::CliExecStdin,
            dir.path().display().to_string(),
            "codex",
        );
        plan.args = vec!["exec".to_string()];
        plan.stdin_path =
            Some(".agentflow/tasks/AF-001/runs/run-001/launch/agent-request.json".to_string());
        write_launch_plan(dir.path(), &plan).unwrap();

        let session = McpSessionSnapshot {
            version: "agentflow-mcp-session.v1".to_string(),
            provider: "codex".to_string(),
            issue_id: "AF-001".to_string(),
            project_id: Some("proj-001".to_string()),
            run_id: "run-001".to_string(),
            session_id: "codex-run-001".to_string(),
            owner_id: "work-agent".to_string(),
            status: McpSessionStatus::Queued,
            launch_mode: McpLaunchMode::CliExecStdin,
            working_directory: dir.path().display().to_string(),
            workspace_root: Some(dir.path().display().to_string()),
            worktree_root: Some(dir.path().display().to_string()),
            runtime_root: Some(
                dir.path()
                    .join(".agentflow/tasks/AF-001/runs/run-001/runtime")
                    .display()
                    .to_string(),
            ),
            temp_root: Some(
                dir.path()
                    .join(".agentflow/tasks/AF-001/runs/run-001/runtime/tmp")
                    .display()
                    .to_string(),
            ),
            cache_root: Some(
                dir.path()
                    .join(".agentflow/tasks/AF-001/runs/run-001/runtime/cache")
                    .display()
                    .to_string(),
            ),
            evidence_root: Some(
                dir.path()
                    .join(".agentflow/tasks/AF-001/runs/run-001/runtime/evidence")
                    .display()
                    .to_string(),
            ),
            launch_request_path: ".agentflow/tasks/AF-001/runs/run-001/launch/agent-request.json"
                .to_string(),
            plan_path: ".agentflow/state/mcp/plans/codex-run-001.json".to_string(),
            log_path: Some(".agentflow/state/mcp/sessions/codex-run-001.jsonl".to_string()),
            branch_name: Some("agentflow/proj-001/AF-001".to_string()),
            attempt_count: 1,
            pid: None,
            process_group_id: None,
            remote_session_id: None,
            pr_url: None,
            last_message_path: Some(
                ".agentflow/state/mcp/sessions/codex-run-001-last-message.txt".to_string(),
            ),
            exit_proof_path: Some(
                ".agentflow/state/mcp/sessions/codex-run-001-exit.json".to_string(),
            ),
            merge_proof_path: None,
            merge_state: None,
            writeback_state: None,
            recovery_reason: None,
            note: Some("test".to_string()),
            last_error: None,
            permission_mode: Some("never".to_string()),
            approval_policy: Some("never".to_string()),
            sandbox_mode: Some("workspace-write".to_string()),
            supervision_mode: Some("local-process-watch".to_string()),
            started_at: 1,
            last_heartbeat_at: 1,
            exited_at: None,
            exit_code: None,
            governance_policy: McpSessionGovernancePolicy::default(),
            governance_facts: McpSessionGovernanceFacts::default(),
            created_at: 1,
            updated_at: 1,
        };
        write_session_snapshot(dir.path(), &session).unwrap();

        assert_eq!(read_launch_plan(dir.path(), "codex-run-001").unwrap(), plan);
        assert_eq!(
            read_session_snapshot(dir.path(), "codex-run-001").unwrap(),
            session
        );
        let loaded = load_session_snapshots(dir.path()).unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].session_id, "codex-run-001");
        let mirrored_run =
            agentflow_task_artifacts::load_task_run(dir.path(), "AF-001", "run-001").unwrap();
        assert_eq!(mirrored_run.session_owner.as_deref(), Some("work-agent"));
        assert_eq!(mirrored_run.session_id.as_deref(), Some("codex-run-001"));
        assert_eq!(
            find_session_snapshot_by_run(dir.path(), "run-001")
                .unwrap()
                .expect("expected run lookup")
                .session_id,
            "codex-run-001"
        );
    }
}
