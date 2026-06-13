pub mod browser;
pub mod codex;
pub mod error;
pub mod events;
pub mod github;
pub mod gitlab;
pub mod health;
pub mod model;
pub mod provider;
pub mod registry;
pub mod storage;

use agentflow_execute::mark_build_agent_launch_failed;
use agentflow_workflow_events::{
    append_event_once, BuildAgentSessionRunningPayload, WorkflowEventDraft,
    EVENT_TYPE_BUILD_AGENT_SESSION_RUNNING,
};
use anyhow::Result;
use std::path::Path;

pub use browser::browser_preview_status;
pub use codex::{check_codex_provider, CodexProvider};
pub use github::check_github_provider;
pub use gitlab::check_gitlab_provider;
pub use model::{
    McpCapability, McpLaunchMode, McpLaunchPlan, McpLaunchRequest, McpLogChunk, McpProviderKind,
    McpProviderStatus, McpProviderStatusCode, McpRegistry, McpRegistryEntry, McpSessionSnapshot,
    McpSessionStatus, MCP_LAUNCH_PLAN_VERSION, MCP_LAUNCH_REQUEST_VERSION, MCP_LOG_CHUNK_VERSION,
    MCP_PROVIDER_STATUS_VERSION, MCP_REGISTRY_VERSION, MCP_SESSION_SNAPSHOT_VERSION,
};
pub use provider::{run_command, CommandProbe, McpAgentProvider, McpProviderBridge};
pub use storage::{
    load_session_snapshots, prepare_mcp_workspace, read_launch_plan, read_provider_status,
    read_registry, read_session_snapshot, write_launch_plan, write_provider_status, write_registry,
    write_registry_for_statuses, write_session_snapshot,
};

pub fn default_provider_bridge() -> McpProviderBridge {
    let mut bridge = McpProviderBridge::new();
    bridge.register(Box::new(CodexProvider::new()));
    bridge
}

pub fn poll_session_snapshot(
    project_root: impl AsRef<Path>,
    session_id: &str,
) -> Result<McpSessionSnapshot> {
    let root = project_root.as_ref();
    let session = read_session_snapshot(root, session_id)?;
    let bridge = default_provider_bridge();
    if let Some(provider) = bridge.provider(&session.provider) {
        let updated = provider.poll_session(root, session_id)?;
        observe_session_transition(root, Some(&session), &updated)?;
        Ok(updated)
    } else {
        Ok(session)
    }
}

pub fn poll_session_snapshots(project_root: impl AsRef<Path>) -> Result<Vec<McpSessionSnapshot>> {
    let root = project_root.as_ref();
    let sessions = load_session_snapshots(root)?;
    let bridge = default_provider_bridge();
    let mut polled = Vec::with_capacity(sessions.len());
    for session in sessions {
        if let Some(provider) = bridge.provider(&session.provider) {
            match provider.poll_session(root, &session.session_id) {
                Ok(updated) => {
                    let _ = observe_session_transition(root, Some(&session), &updated);
                    polled.push(updated);
                }
                Err(_) => polled.push(session),
            }
        } else {
            polled.push(session);
        }
    }
    polled.sort_by(|left, right| {
        right
            .updated_at
            .cmp(&left.updated_at)
            .then_with(|| left.session_id.cmp(&right.session_id))
    });
    Ok(polled)
}

pub fn load_session_log_chunk(
    project_root: impl AsRef<Path>,
    session_id: &str,
    cursor: Option<&str>,
) -> Result<McpLogChunk> {
    let session = read_session_snapshot(project_root.as_ref(), session_id)?;
    let bridge = default_provider_bridge();
    if let Some(provider) = bridge.provider(&session.provider) {
        provider.fetch_logs(project_root.as_ref(), session_id, cursor)
    } else {
        Ok(McpLogChunk::empty(session.provider, session_id.to_string()))
    }
}

fn observe_session_transition(
    project_root: &Path,
    previous: Option<&McpSessionSnapshot>,
    updated: &McpSessionSnapshot,
) -> Result<()> {
    let previous_status = previous.map(|snapshot| snapshot.status.clone());
    if previous_status.as_ref() == Some(&updated.status) {
        return Ok(());
    }

    if matches!(updated.status, McpSessionStatus::Failed) {
        let _ = mark_build_agent_launch_failed(project_root, &updated.run_id);
    }

    if matches!(updated.status, McpSessionStatus::Running) {
        append_event_once(
            project_root,
            WorkflowEventDraft {
                event_type: EVENT_TYPE_BUILD_AGENT_SESSION_RUNNING.to_string(),
                source: "mcp".to_string(),
                subject_id: updated.issue_id.clone(),
                subject_path: Some(updated.launch_request_path.clone()),
                dedupe_key: format!("build-agent.session.running:{}", updated.run_id),
                payload: serde_json::to_value(BuildAgentSessionRunningPayload {
                    issue_id: updated.issue_id.clone(),
                    project_id: updated.project_id.clone(),
                    run_id: updated.run_id.clone(),
                    session_id: updated.session_id.clone(),
                    provider: updated.provider.clone(),
                    branch_name: updated.branch_name.clone(),
                    log_path: updated.log_path.clone(),
                })?,
            },
        )?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::observe_session_transition;
    use crate::model::{McpLaunchMode, McpSessionSnapshot, McpSessionStatus};
    use agentflow_workflow_events::{load_events, EVENT_TYPE_BUILD_AGENT_SESSION_RUNNING};
    use tempfile::tempdir;

    fn session(status: McpSessionStatus) -> McpSessionSnapshot {
        McpSessionSnapshot {
            version: crate::MCP_SESSION_SNAPSHOT_VERSION.to_string(),
            provider: "codex".to_string(),
            issue_id: "AF-001".to_string(),
            project_id: Some("proj-001".to_string()),
            run_id: "run-001".to_string(),
            session_id: "codex-run-001".to_string(),
            status,
            launch_mode: McpLaunchMode::CliExecStdin,
            launch_request_path:
                ".agentflow/execute/runs/run-001/launcher/build-agent-request.json".to_string(),
            plan_path: ".agentflow/state/mcp/plans/codex-run-001.json".to_string(),
            log_path: Some(".agentflow/state/mcp/sessions/codex-run-001.jsonl".to_string()),
            branch_name: Some("agentflow/proj-001/AF-001".to_string()),
            pid: Some(1),
            remote_session_id: None,
            pr_url: None,
            merge_state: None,
            note: None,
            last_error: None,
            created_at: 1,
            updated_at: 2,
        }
    }

    #[test]
    fn running_transition_appends_event_once() {
        let dir = tempdir().unwrap();
        observe_session_transition(
            dir.path(),
            Some(&session(McpSessionStatus::Starting)),
            &session(McpSessionStatus::Running),
        )
        .unwrap();
        observe_session_transition(
            dir.path(),
            Some(&session(McpSessionStatus::Starting)),
            &session(McpSessionStatus::Running),
        )
        .unwrap();

        let events = load_events(dir.path()).unwrap();
        let running_events = events
            .into_iter()
            .filter(|event| event.event_type == EVENT_TYPE_BUILD_AGENT_SESSION_RUNNING)
            .collect::<Vec<_>>();
        assert_eq!(running_events.len(), 1);
        assert_eq!(running_events[0].subject_id, "AF-001");
    }
}
