use crate::{
    health::unix_timestamp_seconds,
    model::{McpLaunchPlan, McpProviderStatus, McpRegistry, McpSessionSnapshot},
    registry::build_registry,
};
use anyhow::{Context, Result};
use std::{
    fs,
    path::{Path, PathBuf},
};

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
    fs::write(path, serde_json::to_string_pretty(value)?)
        .with_context(|| format!("write {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        McpLaunchMode, McpLaunchPlan, McpProviderKind, McpProviderStatus, McpProviderStatusCode,
        McpSessionSnapshot, McpSessionStatus,
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
            status: McpSessionStatus::Queued,
            launch_mode: McpLaunchMode::CliExecStdin,
            launch_request_path: ".agentflow/tasks/AF-001/runs/run-001/launch/agent-request.json"
                .to_string(),
            plan_path: ".agentflow/state/mcp/plans/codex-run-001.json".to_string(),
            log_path: Some(
                ".agentflow/state/mcp/sessions/codex-run-001-last-message.txt".to_string(),
            ),
            branch_name: Some("agentflow/proj-001/AF-001".to_string()),
            pid: None,
            remote_session_id: None,
            pr_url: None,
            merge_state: None,
            note: Some("test".to_string()),
            last_error: None,
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
    }
}
