pub mod browser;
pub mod claude;
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

use agentflow_event_store::{append_task_event_once, EventActor, TaskEventDraft};
use agentflow_workflow_core::{WorkflowAgentRole, WorkflowFlowType};
use anyhow::Result;
use serde_json::{json, Map, Value};
use std::path::Path;

pub use browser::browser_preview_status;
pub use claude::{check_claude_provider, ClaudeCodeProvider};
pub use codex::{check_codex_provider, CodexProvider};
pub use github::check_github_provider;
pub use gitlab::check_gitlab_provider;
pub use model::{
    provider_capability_profile, McpCapability, McpLaunchMode, McpLaunchPlan, McpLaunchRequest,
    McpLogChunk, McpProviderCapabilityProfile, McpProviderKind, McpProviderStatus,
    McpProviderStatusCode, McpRegistry, McpRegistryEntry, McpSessionGovernanceFacts,
    McpSessionGovernancePolicy, McpSessionSnapshot, McpSessionStatus, MCP_DEFAULT_MAX_ATTEMPTS,
    MCP_DEFAULT_SESSION_TIMEOUT_SECONDS, MCP_LAUNCH_PLAN_VERSION, MCP_LAUNCH_REQUEST_VERSION,
    MCP_LOG_CHUNK_VERSION, MCP_PROVIDER_CAPABILITY_PROFILE_VERSION, MCP_PROVIDER_STATUS_VERSION,
    MCP_REGISTRY_VERSION, MCP_SESSION_GOVERNANCE_POLICY_VERSION, MCP_SESSION_SNAPSHOT_VERSION,
};
pub use provider::{run_command, CommandProbe, McpAgentProvider, McpProviderBridge};
pub use storage::{
    load_session_snapshots, prepare_mcp_workspace, read_launch_plan, read_provider_status,
    read_registry, read_session_snapshot, write_launch_plan, write_provider_status, write_registry,
    write_registry_for_statuses, write_session_snapshot,
};

pub fn default_provider_bridge() -> McpProviderBridge {
    let mut bridge = McpProviderBridge::new();
    bridge.register(Box::new(ClaudeCodeProvider::new()));
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

    let event_type = match updated.status {
        McpSessionStatus::Running => Some("agent.session.running"),
        McpSessionStatus::Interrupted => Some("agent.session.interrupted"),
        McpSessionStatus::InReview => Some("agent.session.in_review"),
        McpSessionStatus::Done => Some("agent.session.completed"),
        McpSessionStatus::Failed => Some("agent.session.failed"),
        McpSessionStatus::Cancelled => Some("agent.session.cancelled"),
        McpSessionStatus::Queued | McpSessionStatus::Claimed | McpSessionStatus::Starting => None,
    };

    if let Some(event_type) = event_type {
        let attempt_count = updated.attempt_count.max(1);
        append_task_event_once(
            project_root,
            TaskEventDraft {
                flow_type: WorkflowFlowType::Work,
                aggregate_type: "issue".to_string(),
                aggregate_id: updated.issue_id.clone(),
                project_id: updated.project_id.clone(),
                issue_id: Some(updated.issue_id.clone()),
                run_id: Some(updated.run_id.clone()),
                event_type: event_type.to_string(),
                authority_role: Some(WorkflowAgentRole::WorkAgent),
                actor: EventActor {
                    role: "mcp".to_string(),
                    kind: "system".to_string(),
                },
                state: None,
                correlation_id: Some(format!("corr-{}", updated.issue_id)),
                causation_id: None,
                payload: session_transition_payload(updated, attempt_count),
                artifact_refs: session_artifact_refs(updated),
                idempotency_key: Some(format!(
                    "{event_type}:{}:{}:attempt-{attempt_count}",
                    updated.issue_id, updated.run_id
                )),
            },
        )?;
    }

    Ok(())
}

fn session_artifact_refs(session: &McpSessionSnapshot) -> Vec<String> {
    let mut refs = Vec::new();
    if let Some(log_path) = session.log_path.clone() {
        refs.push(log_path);
    }
    if let Some(last_message_path) = session.last_message_path.clone() {
        refs.push(last_message_path);
    }
    if let Some(merge_proof_path) = session.merge_proof_path.clone() {
        refs.push(merge_proof_path);
    }
    refs
}

fn session_transition_payload(updated: &McpSessionSnapshot, attempt_count: u32) -> Value {
    let mut payload = Map::new();
    payload.insert("issueId".to_string(), json!(updated.issue_id));
    payload.insert("projectId".to_string(), json!(updated.project_id));
    payload.insert("runId".to_string(), json!(updated.run_id));
    payload.insert("sessionId".to_string(), json!(updated.session_id));
    payload.insert("provider".to_string(), json!(updated.provider));
    payload.insert("branchName".to_string(), json!(updated.branch_name));
    payload.insert("attemptCount".to_string(), json!(attempt_count));
    payload.insert("logPath".to_string(), json!(updated.log_path));
    payload.insert(
        "lastMessagePath".to_string(),
        json!(updated.last_message_path),
    );
    payload.insert(
        "mergeProofPath".to_string(),
        json!(updated.merge_proof_path),
    );
    payload.insert("mergeState".to_string(), json!(updated.merge_state));
    payload.insert("writebackState".to_string(), json!(updated.writeback_state));
    payload.insert("recoveryReason".to_string(), json!(updated.recovery_reason));
    payload.insert("lastError".to_string(), json!(updated.last_error));
    payload.insert(
        "governancePolicyVersion".to_string(),
        json!(updated.governance_policy.version),
    );
    payload.insert(
        "claimPolicy".to_string(),
        json!(updated.governance_policy.claim_policy),
    );
    payload.insert(
        "timeoutPolicy".to_string(),
        json!(updated.governance_policy.timeout_policy),
    );
    payload.insert(
        "timeoutSeconds".to_string(),
        json!(updated.governance_policy.timeout_seconds),
    );
    payload.insert(
        "timeoutAt".to_string(),
        json!(updated.governance_facts.timeout_at),
    );
    payload.insert(
        "timedOutAt".to_string(),
        json!(updated.governance_facts.timed_out_at),
    );
    payload.insert(
        "takeoverPolicy".to_string(),
        json!(updated.governance_policy.takeover_policy),
    );
    payload.insert(
        "retryPolicy".to_string(),
        json!(updated.governance_policy.retry_policy),
    );
    payload.insert(
        "maxAttempts".to_string(),
        json!(updated.governance_policy.max_attempts),
    );
    payload.insert(
        "cancelPolicy".to_string(),
        json!(updated.governance_policy.cancel_policy),
    );
    payload.insert(
        "cancelRequestedAt".to_string(),
        json!(updated.governance_facts.cancel_requested_at),
    );
    payload.insert(
        "cancelledAt".to_string(),
        json!(updated.governance_facts.cancelled_at),
    );
    payload.insert(
        "resumedFromAttempt".to_string(),
        json!(updated.governance_facts.resumed_from_attempt),
    );
    payload.insert(
        "takeoverSessionId".to_string(),
        json!(updated.governance_facts.takeover_session_id),
    );
    payload.insert(
        "terminalReason".to_string(),
        json!(updated.governance_facts.terminal_reason),
    );
    payload.insert(
        "retryable".to_string(),
        json!(updated.governance_facts.retryable),
    );
    payload.insert("sessionStatus".to_string(), json!(updated.status.as_str()));
    payload.insert("status".to_string(), json!(updated.status.as_str()));
    Value::Object(payload)
}

#[cfg(test)]
mod tests {
    use super::observe_session_transition;
    use crate::model::{
        McpLaunchMode, McpSessionGovernanceFacts, McpSessionGovernancePolicy, McpSessionSnapshot,
        McpSessionStatus,
    };
    use agentflow_event_store::load_task_events;
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
            launch_request_path: ".agentflow/tasks/AF-001/runs/run-001/launch/agent-request.json"
                .to_string(),
            plan_path: ".agentflow/state/mcp/plans/codex-run-001.json".to_string(),
            log_path: Some(".agentflow/state/mcp/sessions/codex-run-001.jsonl".to_string()),
            branch_name: Some("agentflow/proj-001/AF-001".to_string()),
            attempt_count: 1,
            pid: Some(1),
            remote_session_id: None,
            pr_url: None,
            last_message_path: Some(
                ".agentflow/state/mcp/sessions/codex-run-001-last-message.txt".to_string(),
            ),
            merge_proof_path: None,
            merge_state: None,
            writeback_state: None,
            recovery_reason: None,
            note: None,
            last_error: None,
            governance_policy: McpSessionGovernancePolicy::default(),
            governance_facts: McpSessionGovernanceFacts::default(),
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

        let events = load_task_events(dir.path()).unwrap();
        let running_events = events
            .into_iter()
            .filter(|event| event.event_type == "agent.session.running")
            .collect::<Vec<_>>();
        assert_eq!(running_events.len(), 1);
        assert_eq!(running_events[0].issue_id.as_deref(), Some("AF-001"));
    }

    #[test]
    fn completed_transition_appends_session_completed_event() {
        let dir = tempdir().unwrap();
        observe_session_transition(
            dir.path(),
            Some(&session(McpSessionStatus::Running)),
            &session(McpSessionStatus::Done),
        )
        .unwrap();

        let events = load_task_events(dir.path()).unwrap();
        let completed = events
            .into_iter()
            .find(|event| event.event_type == "agent.session.completed")
            .expect("expected completed event");
        assert_eq!(completed.issue_id.as_deref(), Some("AF-001"));
        assert_eq!(completed.payload["status"], "done");
    }
}
