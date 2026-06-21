use crate::model::{
    AgentDispatchProviderSelection, AgentDispatchRoleBinding, AgentDispatcherClaim,
    AGENT_LAUNCH_CLAIMED, AGENT_SESSION_CANCELLED, AGENT_SESSION_CREATED, AGENT_SESSION_DONE,
    AGENT_SESSION_FAILED, AGENT_SESSION_INTERRUPTED, AGENT_SESSION_IN_REVIEW,
    AGENT_SESSION_RESUMED, AGENT_SESSION_RUNNING,
};
use agentflow_event_store::{
    append_task_event_once, claim_task_event, load_task_events, release_task_claim,
    task_claim_is_active, EventActor, TaskEvent, TaskEventDraft,
};
use agentflow_mcp::{
    default_provider_bridge, find_session_snapshot_by_run, McpLaunchRequest, McpProviderBridge,
    McpSessionSnapshot, McpSessionStatus,
};
use agentflow_task_loop::{AgentLaunchPayload, AGENT_LAUNCH_REQUESTED};
use agentflow_workflow_core::WorkflowFlowType;
use anyhow::{Context, Result};
use serde_json::{json, Map, Value};
use std::{collections::BTreeMap, path::Path};

pub struct AgentDispatcher {
    providers: McpProviderBridge,
}

impl AgentDispatcher {
    pub fn new(providers: McpProviderBridge) -> Self {
        Self { providers }
    }

    pub fn with_default_providers() -> Self {
        Self::new(default_provider_bridge())
    }

    pub fn claim_next_launch(
        &self,
        project_root: impl AsRef<Path>,
    ) -> Result<Option<AgentDispatcherClaim>> {
        let root = project_root.as_ref();
        reconcile_terminal_session_leases(root)?;
        let Some((event, claim_event)) = claim_task_event(
            root,
            "agent-dispatcher",
            is_claimable_launch_request(root),
            |event, _events| self.build_launch_claim_draft(root, event),
        )?
        else {
            return Ok(None);
        };

        let payload: AgentLaunchPayload = serde_json::from_value(event.payload.clone())
            .with_context(|| format!("parse launch payload {}", event.event_id))?;
        let role_binding = AgentDispatchRoleBinding::resolve(payload.agent_role.clone())?;
        let selection = self.evaluate_provider_selection(root, &payload, &role_binding)?;
        if let Err(error) = selection.ensure_runnable() {
            append_launch_failed_event(root, &payload, &role_binding, &selection, &error, false)?;
            let _ = release_task_claim(
                root,
                &payload.run_id,
                Some(claim_event.event_id.as_str()),
                "provider-not-runnable",
            );
            return Err(error);
        }
        if let Some(existing_session) =
            recover_existing_session_snapshot(root, &payload, &payload.provider)?
        {
            let recovered_event = append_session_event(
                root,
                &payload,
                &role_binding,
                &selection,
                &existing_session,
                if had_prior_session_event(root, &payload.run_id)? {
                    AGENT_SESSION_RESUMED
                } else {
                    AGENT_SESSION_CREATED
                },
            )?;
            append_status_event(root, &payload, &selection, &existing_session)?;
            return Ok(Some(AgentDispatcherClaim {
                issue_id: payload.issue_id,
                run_id: payload.run_id,
                provider: existing_session.provider,
                session_id: existing_session.session_id,
                session_status: existing_session.status.as_str().to_string(),
                runtime_role: role_binding.runtime_role,
                skill_pack: role_binding.skill_pack,
                selection,
                created_event_id: recovered_event.event_id,
            }));
        }
        let request = launch_payload_to_mcp_request(&payload, &role_binding);
        let provider = self
            .providers
            .provider(&payload.provider)
            .ok_or_else(|| anyhow::anyhow!("{}", selection.selection_reason))?;
        let session = match provider.create_session(root, &request) {
            Ok(session) => session,
            Err(error) => {
                append_launch_failed_event(
                    root,
                    &payload,
                    &role_binding,
                    &selection,
                    &error,
                    true,
                )?;
                let _ = release_task_claim(
                    root,
                    &payload.run_id,
                    Some(claim_event.event_id.as_str()),
                    "provider-launch-failed",
                );
                return Err(error);
            }
        };
        let _created_event = append_session_event(
            root,
            &payload,
            &role_binding,
            &selection,
            &session,
            if had_prior_session_event(root, &payload.run_id)? {
                AGENT_SESSION_RESUMED
            } else {
                AGENT_SESSION_CREATED
            },
        )?;
        append_status_event(root, &payload, &selection, &session)?;

        Ok(Some(AgentDispatcherClaim {
            issue_id: payload.issue_id,
            run_id: payload.run_id,
            provider: session.provider,
            session_id: session.session_id,
            session_status: session.status.as_str().to_string(),
            runtime_role: role_binding.runtime_role,
            skill_pack: role_binding.skill_pack,
            selection,
            created_event_id: claim_event.event_id,
        }))
    }

    fn evaluate_provider_selection(
        &self,
        project_root: &Path,
        payload: &AgentLaunchPayload,
        role_binding: &AgentDispatchRoleBinding,
    ) -> Result<AgentDispatchProviderSelection> {
        let provider = self.providers.provider(&payload.provider);
        let provider_status = provider.map(|provider| provider.check_health(project_root));
        let selection = AgentDispatchProviderSelection::evaluate(
            &payload.provider,
            provider_status.as_ref(),
            role_binding,
        );
        Ok(selection)
    }

    fn build_launch_claim_draft(
        &self,
        project_root: &Path,
        event: &TaskEvent,
    ) -> Result<TaskEventDraft> {
        let payload: AgentLaunchPayload = serde_json::from_value(event.payload.clone())
            .with_context(|| format!("parse launch payload {}", event.event_id))?;
        let role_binding = AgentDispatchRoleBinding::resolve(payload.agent_role.clone())?;
        let selection = self.evaluate_provider_selection(project_root, &payload, &role_binding)?;
        Ok(TaskEventDraft {
            flow_type: WorkflowFlowType::Work,
            aggregate_type: "issue".to_string(),
            aggregate_id: payload.issue_id.clone(),
            project_id: payload.project_id.clone(),
            issue_id: Some(payload.issue_id.clone()),
            run_id: Some(payload.run_id.clone()),
            event_type: AGENT_LAUNCH_CLAIMED.to_string(),
            authority_role: Some(role_binding.runtime_role),
            actor: EventActor {
                role: "agent-dispatcher".to_string(),
                kind: "system".to_string(),
            },
            state: None,
            correlation_id: Some(format!("corr-{}", payload.issue_id)),
            causation_id: Some(event.event_id.clone()),
            payload: launch_claim_payload(&payload, &role_binding, &selection),
            artifact_refs: vec![payload.launch_request_path.clone()],
            idempotency_key: Some(format!(
                "{AGENT_LAUNCH_CLAIMED}:{}:{}",
                payload.issue_id, payload.run_id
            )),
        })
    }
}

fn unavailable_run_ids(root: &Path, events: &[TaskEvent]) -> BTreeMap<String, bool> {
    let mut state = BTreeMap::new();
    for event in events {
        let Some(run_id) = event.run_id.clone() else {
            continue;
        };
        match event.event_type.as_str() {
            AGENT_LAUNCH_CLAIMED => {
                let active = task_claim_is_active(root, &run_id).unwrap_or(false);
                state.insert(run_id, active);
            }
            AGENT_SESSION_CREATED
            | AGENT_SESSION_RESUMED
            | AGENT_SESSION_RUNNING
            | AGENT_SESSION_IN_REVIEW
            | AGENT_SESSION_DONE
            | AGENT_SESSION_CANCELLED => {
                state.insert(run_id, true);
            }
            AGENT_SESSION_INTERRUPTED | AGENT_SESSION_FAILED => {
                let retryable = event
                    .payload
                    .get("retryable")
                    .and_then(Value::as_bool)
                    .unwrap_or(true);
                state.insert(run_id, !retryable);
            }
            _ => {}
        }
    }
    state
}

fn had_prior_session_event(root: &Path, run_id: &str) -> Result<bool> {
    Ok(load_task_events(root)?.iter().any(|event| {
        matches!(
            event.event_type.as_str(),
            AGENT_SESSION_CREATED
                | AGENT_SESSION_RESUMED
                | AGENT_SESSION_RUNNING
                | AGENT_SESSION_INTERRUPTED
                | AGENT_SESSION_IN_REVIEW
                | AGENT_SESSION_DONE
                | AGENT_SESSION_FAILED
                | AGENT_SESSION_CANCELLED
        ) && event.run_id.as_deref() == Some(run_id)
    }))
}

fn reconcile_terminal_session_leases(root: &Path) -> Result<()> {
    let mut terminal_by_run = BTreeMap::new();
    for event in load_task_events(root)? {
        let Some(run_id) = event.run_id.clone() else {
            continue;
        };
        let reason = match event.event_type.as_str() {
            AGENT_SESSION_INTERRUPTED => Some("session-interrupted"),
            AGENT_SESSION_FAILED => Some("session-failed"),
            AGENT_SESSION_CANCELLED => Some("session-cancelled"),
            AGENT_SESSION_DONE => Some("session-completed"),
            AGENT_SESSION_CREATED
            | AGENT_SESSION_RESUMED
            | AGENT_SESSION_RUNNING
            | AGENT_SESSION_IN_REVIEW => None,
            _ => continue,
        };
        terminal_by_run.insert(run_id, reason);
    }

    for (run_id, reason) in terminal_by_run {
        if let Some(reason) = reason {
            let _ = release_task_claim(root, &run_id, None, reason)?;
        }
    }
    Ok(())
}

fn recover_existing_session_snapshot(
    root: &Path,
    payload: &AgentLaunchPayload,
    provider_id: &str,
) -> Result<Option<McpSessionSnapshot>> {
    let Some(session) = find_session_snapshot_by_run(root, &payload.run_id)? else {
        return Ok(None);
    };
    if session.issue_id != payload.issue_id || session.provider != provider_id {
        return Ok(None);
    }
    if matches!(
        session.status,
        McpSessionStatus::Failed | McpSessionStatus::Cancelled
    ) {
        return Ok(None);
    }
    Ok(Some(session))
}

fn is_claimable_launch_request<'a>(
    root: &'a Path,
) -> impl Fn(&TaskEvent, &[TaskEvent]) -> bool + 'a {
    move |event: &TaskEvent, events: &[TaskEvent]| {
        if event.event_type != AGENT_LAUNCH_REQUESTED {
            return false;
        }
        let unavailable_runs = unavailable_run_ids(root, events);
        event
            .run_id
            .as_deref()
            .is_some_and(|run_id| !unavailable_runs.get(run_id).copied().unwrap_or(false))
    }
}

fn launch_payload_to_mcp_request(
    payload: &AgentLaunchPayload,
    role_binding: &AgentDispatchRoleBinding,
) -> McpLaunchRequest {
    let mut request = McpLaunchRequest::new(
        payload.provider.clone(),
        payload.issue_id.clone(),
        payload.run_id.clone(),
        role_binding.provider_role.clone(),
        payload.working_directory.clone(),
        payload.launch_request_path.clone(),
    );
    request.project_id = payload.project_id.clone();
    request.prompt_path = Some(payload.launch_request_path.clone());
    request.context_pack_path = payload.context_pack_path.clone();
    request.branch_name = Some(payload.branch_name.clone());
    request.merge_mode = Some(payload.merge_mode.clone());
    request
}

fn launch_claim_payload(
    payload: &AgentLaunchPayload,
    role_binding: &AgentDispatchRoleBinding,
    selection: &AgentDispatchProviderSelection,
) -> Value {
    let mut body = Map::new();
    body.insert("provider".to_string(), json!(payload.provider));
    body.insert(
        "providerKind".to_string(),
        json!(selection.provider_kind.clone()),
    );
    body.insert(
        "providerStatus".to_string(),
        json!(selection.provider_status.clone()),
    );
    body.insert(
        "requestedRole".to_string(),
        json!(role_binding.requested_role.clone()),
    );
    body.insert(
        "runtimeRole".to_string(),
        json!(role_binding.runtime_role.as_str()),
    );
    body.insert(
        "skillPack".to_string(),
        json!(role_binding.skill_pack.map(|value| value.as_str())),
    );
    body.insert(
        "selectionStatus".to_string(),
        json!(selection.status.as_str()),
    );
    body.insert(
        "selectionReason".to_string(),
        json!(selection.selection_reason.clone()),
    );
    body.insert(
        "degradationReason".to_string(),
        json!(selection.degradation_reason.clone()),
    );
    body.insert(
        "requiredCapabilities".to_string(),
        json!(selection.required_capabilities.clone()),
    );
    body.insert(
        "degradedCapabilities".to_string(),
        json!(selection.degraded_capabilities.clone()),
    );
    body.insert(
        "missingRequiredCapabilities".to_string(),
        json!(selection.missing_required_capabilities.clone()),
    );
    body.insert(
        "missingDegradedCapabilities".to_string(),
        json!(selection.missing_degraded_capabilities.clone()),
    );
    body.insert("issueId".to_string(), json!(payload.issue_id.clone()));
    body.insert("projectId".to_string(), json!(payload.project_id.clone()));
    body.insert("runId".to_string(), json!(payload.run_id.clone()));
    body.insert("branchName".to_string(), json!(payload.branch_name.clone()));
    body.insert(
        "workingDirectory".to_string(),
        json!(payload.working_directory.clone()),
    );
    body.insert(
        "launchRequestPath".to_string(),
        json!(payload.launch_request_path.clone()),
    );
    body.insert("mergeMode".to_string(), json!(payload.merge_mode.clone()));
    Value::Object(body)
}

fn append_status_event(
    root: &Path,
    payload: &AgentLaunchPayload,
    selection: &AgentDispatchProviderSelection,
    session: &McpSessionSnapshot,
) -> Result<Option<TaskEvent>> {
    let event_type = match session.status {
        McpSessionStatus::Running | McpSessionStatus::Starting | McpSessionStatus::Claimed => {
            AGENT_SESSION_RUNNING
        }
        McpSessionStatus::Interrupted => AGENT_SESSION_INTERRUPTED,
        McpSessionStatus::InReview => AGENT_SESSION_IN_REVIEW,
        McpSessionStatus::Done => AGENT_SESSION_DONE,
        McpSessionStatus::Failed => AGENT_SESSION_FAILED,
        McpSessionStatus::Cancelled => AGENT_SESSION_CANCELLED,
        McpSessionStatus::Queued => return Ok(None),
    };
    let role_binding = AgentDispatchRoleBinding::resolve(payload.agent_role.clone())?;
    append_session_event(root, payload, &role_binding, selection, session, event_type).map(Some)
}

fn append_session_event(
    root: &Path,
    payload: &AgentLaunchPayload,
    role_binding: &AgentDispatchRoleBinding,
    selection: &AgentDispatchProviderSelection,
    session: &McpSessionSnapshot,
    event_type: &str,
) -> Result<TaskEvent> {
    let attempt_count = session.attempt_count.max(1);
    append_task_event_once(
        root,
        TaskEventDraft {
            flow_type: WorkflowFlowType::Work,
            aggregate_type: "issue".to_string(),
            aggregate_id: payload.issue_id.clone(),
            project_id: payload.project_id.clone(),
            issue_id: Some(payload.issue_id.clone()),
            run_id: Some(session.run_id.clone()),
            event_type: event_type.to_string(),
            authority_role: Some(role_binding.runtime_role),
            actor: EventActor {
                role: "agent-dispatcher".to_string(),
                kind: "system".to_string(),
            },
            state: None,
            correlation_id: Some(format!("corr-{}", payload.issue_id)),
            causation_id: None,
            payload: session_event_payload(role_binding, selection, session, attempt_count),
            artifact_refs: session_artifact_refs(session),
            idempotency_key: Some(format!(
                "{event_type}:{}:{}:attempt-{attempt_count}",
                session.issue_id, session.run_id
            )),
        },
    )
}

fn append_launch_failed_event(
    root: &Path,
    payload: &AgentLaunchPayload,
    role_binding: &AgentDispatchRoleBinding,
    selection: &AgentDispatchProviderSelection,
    error: &anyhow::Error,
    retryable: bool,
) -> Result<TaskEvent> {
    let mut failure_payload = launch_claim_payload(payload, role_binding, selection);
    if let Value::Object(ref mut body) = failure_payload {
        body.insert("sessionStatus".to_string(), json!("failed"));
        body.insert("lastError".to_string(), json!(error.to_string()));
        body.insert("retryable".to_string(), json!(retryable));
        body.insert(
            "terminalReason".to_string(),
            json!(if retryable {
                "provider-launch-failed"
            } else {
                "provider-not-runnable"
            }),
        );
    }
    append_task_event_once(
        root,
        TaskEventDraft {
            flow_type: WorkflowFlowType::Work,
            aggregate_type: "issue".to_string(),
            aggregate_id: payload.issue_id.clone(),
            project_id: payload.project_id.clone(),
            issue_id: Some(payload.issue_id.clone()),
            run_id: Some(payload.run_id.clone()),
            event_type: AGENT_SESSION_FAILED.to_string(),
            authority_role: Some(role_binding.runtime_role),
            actor: EventActor {
                role: "agent-dispatcher".to_string(),
                kind: "system".to_string(),
            },
            state: None,
            correlation_id: Some(format!("corr-{}", payload.issue_id)),
            causation_id: None,
            payload: failure_payload,
            artifact_refs: vec![payload.launch_request_path.clone()],
            idempotency_key: Some(format!(
                "{AGENT_SESSION_FAILED}:{}:{}:launch-create",
                payload.issue_id, payload.run_id
            )),
        },
    )
}

fn session_artifact_refs(session: &McpSessionSnapshot) -> Vec<String> {
    let mut refs = vec![session.plan_path.clone()];
    if let Some(log_path) = session.log_path.clone() {
        refs.push(log_path);
    }
    if let Some(last_message_path) = session.last_message_path.clone() {
        refs.push(last_message_path);
    }
    if let Some(exit_proof_path) = session.exit_proof_path.clone() {
        refs.push(exit_proof_path);
    }
    if let Some(merge_proof_path) = session.merge_proof_path.clone() {
        refs.push(merge_proof_path);
    }
    refs
}

fn session_event_payload(
    role_binding: &AgentDispatchRoleBinding,
    selection: &AgentDispatchProviderSelection,
    session: &McpSessionSnapshot,
    attempt_count: u32,
) -> Value {
    let mut payload = Map::new();
    payload.insert("provider".to_string(), json!(session.provider));
    payload.insert("providerKind".to_string(), json!(selection.provider_kind));
    payload.insert(
        "providerStatus".to_string(),
        json!(selection.provider_status),
    );
    payload.insert(
        "requestedRole".to_string(),
        json!(role_binding.requested_role),
    );
    payload.insert(
        "runtimeRole".to_string(),
        json!(role_binding.runtime_role.as_str()),
    );
    payload.insert(
        "skillPack".to_string(),
        json!(role_binding.skill_pack.map(|value| value.as_str())),
    );
    payload.insert(
        "supportedRoles".to_string(),
        json!(selection.supported_roles),
    );
    payload.insert(
        "supportedSkillPacks".to_string(),
        json!(selection.supported_skill_packs),
    );
    payload.insert(
        "selectionStatus".to_string(),
        json!(selection.status.as_str()),
    );
    payload.insert(
        "selectionReason".to_string(),
        json!(selection.selection_reason),
    );
    payload.insert(
        "degradationReason".to_string(),
        json!(selection.degradation_reason),
    );
    payload.insert(
        "requiredCapabilities".to_string(),
        json!(selection.required_capabilities),
    );
    payload.insert(
        "degradedCapabilities".to_string(),
        json!(selection.degraded_capabilities),
    );
    payload.insert(
        "missingRequiredCapabilities".to_string(),
        json!(selection.missing_required_capabilities),
    );
    payload.insert(
        "missingDegradedCapabilities".to_string(),
        json!(selection.missing_degraded_capabilities),
    );
    payload.insert("ownerId".to_string(), json!(session.owner_id));
    payload.insert("sessionId".to_string(), json!(session.session_id));
    payload.insert("sessionStatus".to_string(), json!(session.status.as_str()));
    payload.insert("attemptCount".to_string(), json!(attempt_count));
    payload.insert("issueId".to_string(), json!(session.issue_id));
    payload.insert("projectId".to_string(), json!(session.project_id));
    payload.insert("runId".to_string(), json!(session.run_id));
    payload.insert("branchName".to_string(), json!(session.branch_name));
    payload.insert(
        "workingDirectory".to_string(),
        json!(session.working_directory),
    );
    payload.insert("workspaceRoot".to_string(), json!(session.workspace_root));
    payload.insert("worktreeRoot".to_string(), json!(session.worktree_root));
    payload.insert("runtimeRoot".to_string(), json!(session.runtime_root));
    payload.insert("tempRoot".to_string(), json!(session.temp_root));
    payload.insert("cacheRoot".to_string(), json!(session.cache_root));
    payload.insert("evidenceRoot".to_string(), json!(session.evidence_root));
    payload.insert(
        "launchRequestPath".to_string(),
        json!(session.launch_request_path),
    );
    payload.insert("planPath".to_string(), json!(session.plan_path));
    payload.insert("logPath".to_string(), json!(session.log_path));
    payload.insert(
        "lastMessagePath".to_string(),
        json!(session.last_message_path),
    );
    payload.insert("exitProofPath".to_string(), json!(session.exit_proof_path));
    payload.insert(
        "mergeProofPath".to_string(),
        json!(session.merge_proof_path),
    );
    payload.insert("mergeState".to_string(), json!(session.merge_state));
    payload.insert("writebackState".to_string(), json!(session.writeback_state));
    payload.insert("recoveryReason".to_string(), json!(session.recovery_reason));
    payload.insert("lastError".to_string(), json!(session.last_error));
    payload.insert("permissionMode".to_string(), json!(session.permission_mode));
    payload.insert("approvalPolicy".to_string(), json!(session.approval_policy));
    payload.insert("sandboxMode".to_string(), json!(session.sandbox_mode));
    payload.insert(
        "supervisionMode".to_string(),
        json!(session.supervision_mode),
    );
    payload.insert("startedAt".to_string(), json!(session.started_at));
    payload.insert(
        "lastHeartbeatAt".to_string(),
        json!(session.last_heartbeat_at),
    );
    payload.insert(
        "governancePolicyVersion".to_string(),
        json!(session.governance_policy.version),
    );
    payload.insert(
        "claimPolicy".to_string(),
        json!(session.governance_policy.claim_policy),
    );
    payload.insert(
        "timeoutPolicy".to_string(),
        json!(session.governance_policy.timeout_policy),
    );
    payload.insert(
        "timeoutSeconds".to_string(),
        json!(session.governance_policy.timeout_seconds),
    );
    payload.insert(
        "timeoutAt".to_string(),
        json!(session.governance_facts.timeout_at),
    );
    payload.insert(
        "timedOutAt".to_string(),
        json!(session.governance_facts.timed_out_at),
    );
    payload.insert(
        "takeoverPolicy".to_string(),
        json!(session.governance_policy.takeover_policy),
    );
    payload.insert(
        "retryPolicy".to_string(),
        json!(session.governance_policy.retry_policy),
    );
    payload.insert(
        "maxAttempts".to_string(),
        json!(session.governance_policy.max_attempts),
    );
    payload.insert(
        "cancelPolicy".to_string(),
        json!(session.governance_policy.cancel_policy),
    );
    payload.insert(
        "cancelRequestedAt".to_string(),
        json!(session.governance_facts.cancel_requested_at),
    );
    payload.insert(
        "cancelledAt".to_string(),
        json!(session.governance_facts.cancelled_at),
    );
    payload.insert(
        "resumedFromAttempt".to_string(),
        json!(session.governance_facts.resumed_from_attempt),
    );
    payload.insert(
        "processGroupId".to_string(),
        json!(session.process_group_id),
    );
    payload.insert(
        "takeoverSessionId".to_string(),
        json!(session.governance_facts.takeover_session_id),
    );
    payload.insert(
        "terminalReason".to_string(),
        json!(session.governance_facts.terminal_reason),
    );
    payload.insert(
        "retryable".to_string(),
        json!(session.governance_facts.retryable),
    );
    payload.insert("exitedAt".to_string(), json!(session.exited_at));
    payload.insert("exitCode".to_string(), json!(session.exit_code));
    Value::Object(payload)
}

#[cfg(test)]
mod tests {
    use super::*;
    use agentflow_mcp::{
        write_launch_plan, write_session_snapshot, McpAgentProvider, McpLaunchMode, McpLaunchPlan,
        McpProviderKind, McpProviderStatus, McpProviderStatusCode, McpSessionSnapshot,
    };
    use agentflow_task_loop::TaskLoop;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };
    use std::{fs, path::Path, process::Command};
    use tempfile::tempdir;

    struct FakeProvider;

    struct FailOnceProvider {
        attempts: Arc<AtomicUsize>,
    }

    struct LimitedProvider;
    struct NoCreateProvider;

    impl McpAgentProvider for FakeProvider {
        fn provider_id(&self) -> &'static str {
            "codex"
        }

        fn kind(&self) -> McpProviderKind {
            McpProviderKind::Codex
        }

        fn check_health(&self, _project_root: &Path) -> McpProviderStatus {
            let mut status = McpProviderStatus::new(McpProviderKind::Codex, 1);
            status.provider = "codex".to_string();
            status.status = McpProviderStatusCode::Ready;
            status.capabilities = vec![
                agentflow_mcp::McpCapability::new("launch", true),
                agentflow_mcp::McpCapability::new("codex.exec", true),
                agentflow_mcp::McpCapability::new("session.poll", true),
                agentflow_mcp::McpCapability::new("session.logs", true),
                agentflow_mcp::McpCapability::new("session.cancel", true),
                agentflow_mcp::McpCapability::new("build_agent.complete", true),
            ];
            status
        }

        fn build_launch_plan(
            &self,
            _project_root: &Path,
            request: &McpLaunchRequest,
        ) -> Result<McpLaunchPlan> {
            let mut plan = McpLaunchPlan::new(
                "codex",
                format!("fake-{}", request.run_id),
                request.issue_id.clone(),
                request.run_id.clone(),
                McpLaunchMode::CliExecPromptFile,
                request.working_directory.clone(),
                "fake-agent",
            );
            plan.stdin_path = Some(request.launch_request_path.clone());
            Ok(plan)
        }
    }

    impl McpAgentProvider for FailOnceProvider {
        fn provider_id(&self) -> &'static str {
            "codex"
        }

        fn kind(&self) -> McpProviderKind {
            McpProviderKind::Codex
        }

        fn check_health(&self, _project_root: &Path) -> McpProviderStatus {
            let mut status = McpProviderStatus::new(McpProviderKind::Codex, 1);
            status.provider = "codex".to_string();
            status.status = McpProviderStatusCode::Ready;
            status.capabilities = vec![
                agentflow_mcp::McpCapability::new("launch", true),
                agentflow_mcp::McpCapability::new("codex.exec", true),
                agentflow_mcp::McpCapability::new("session.poll", true),
                agentflow_mcp::McpCapability::new("session.logs", true),
                agentflow_mcp::McpCapability::new("session.cancel", true),
                agentflow_mcp::McpCapability::new("build_agent.complete", true),
            ];
            status
        }

        fn build_launch_plan(
            &self,
            _project_root: &Path,
            request: &McpLaunchRequest,
        ) -> Result<McpLaunchPlan> {
            let mut plan = McpLaunchPlan::new(
                "codex",
                format!("fake-{}", request.run_id),
                request.issue_id.clone(),
                request.run_id.clone(),
                McpLaunchMode::CliExecPromptFile,
                request.working_directory.clone(),
                "fake-agent",
            );
            plan.stdin_path = Some(request.launch_request_path.clone());
            Ok(plan)
        }

        fn create_session(
            &self,
            project_root: &Path,
            request: &McpLaunchRequest,
        ) -> Result<McpSessionSnapshot> {
            if self.attempts.fetch_add(1, Ordering::SeqCst) == 0 {
                anyhow::bail!("create session failed");
            }
            let plan = self.build_launch_plan(project_root, request)?;
            let session = McpSessionSnapshot::queued(request, &plan, 1);
            write_launch_plan(project_root, &plan)?;
            write_session_snapshot(project_root, &session)?;
            Ok(session)
        }
    }

    impl McpAgentProvider for LimitedProvider {
        fn provider_id(&self) -> &'static str {
            "codex"
        }

        fn kind(&self) -> McpProviderKind {
            McpProviderKind::Codex
        }

        fn check_health(&self, _project_root: &Path) -> McpProviderStatus {
            let mut status = McpProviderStatus::new(McpProviderKind::Codex, 1);
            status.provider = "codex".to_string();
            status.status = McpProviderStatusCode::Ready;
            status.capabilities = vec![
                agentflow_mcp::McpCapability::new("launch", true),
                agentflow_mcp::McpCapability::new("codex.exec", true),
                agentflow_mcp::McpCapability::new("session.poll", true),
                agentflow_mcp::McpCapability::new("session.logs", true),
                agentflow_mcp::McpCapability::new("session.cancel", true),
                agentflow_mcp::McpCapability::new("build_agent.complete", false),
            ];
            status
        }

        fn build_launch_plan(
            &self,
            _project_root: &Path,
            request: &McpLaunchRequest,
        ) -> Result<McpLaunchPlan> {
            let mut plan = McpLaunchPlan::new(
                "codex",
                format!("limited-{}", request.run_id),
                request.issue_id.clone(),
                request.run_id.clone(),
                McpLaunchMode::CliExecPromptFile,
                request.working_directory.clone(),
                "fake-agent",
            );
            plan.stdin_path = Some(request.launch_request_path.clone());
            Ok(plan)
        }
    }

    impl McpAgentProvider for NoCreateProvider {
        fn provider_id(&self) -> &'static str {
            "codex"
        }

        fn kind(&self) -> McpProviderKind {
            McpProviderKind::Codex
        }

        fn check_health(&self, _project_root: &Path) -> McpProviderStatus {
            let mut status = McpProviderStatus::new(McpProviderKind::Codex, 1);
            status.provider = "codex".to_string();
            status.status = McpProviderStatusCode::Ready;
            status.capabilities = vec![
                agentflow_mcp::McpCapability::new("launch", true),
                agentflow_mcp::McpCapability::new("codex.exec", true),
                agentflow_mcp::McpCapability::new("session.poll", true),
                agentflow_mcp::McpCapability::new("session.logs", true),
                agentflow_mcp::McpCapability::new("session.cancel", true),
                agentflow_mcp::McpCapability::new("build_agent.complete", true),
            ];
            status
        }

        fn build_launch_plan(
            &self,
            _project_root: &Path,
            request: &McpLaunchRequest,
        ) -> Result<McpLaunchPlan> {
            let mut plan = McpLaunchPlan::new(
                "codex",
                format!("recovered-{}", request.run_id),
                request.issue_id.clone(),
                request.run_id.clone(),
                McpLaunchMode::CliExecPromptFile,
                request.working_directory.clone(),
                "fake-agent",
            );
            plan.stdin_path = Some(request.launch_request_path.clone());
            Ok(plan)
        }

        fn create_session(
            &self,
            _project_root: &Path,
            _request: &McpLaunchRequest,
        ) -> Result<McpSessionSnapshot> {
            anyhow::bail!("create_session should not run when snapshot is already durable")
        }
    }

    fn write_fixture(root: &Path) {
        let requirement = root.join("docs/requirements/034-test.md");
        fs::create_dir_all(requirement.parent().unwrap()).unwrap();
        fs::write(&requirement, "# 测试需求\n\n用于 agent-dispatcher 测试。\n").unwrap();
        Command::new("git")
            .arg("init")
            .arg("-b")
            .arg("main")
            .current_dir(root)
            .output()
            .unwrap();
        Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(root)
            .output()
            .unwrap();
        Command::new("git")
            .args(["config", "user.name", "AgentFlow Test"])
            .current_dir(root)
            .output()
            .unwrap();

        let mut issue = agentflow_spec::SpecIssueDraft::new("AF-DISPATCH-001");
        issue.project_id = Some("project-dispatcher".to_string());
        let mut issue = agentflow_spec::issue_from_requirement(root, &requirement, issue).unwrap();
        issue.allowed_paths = vec!["apps/desktop/src/**".to_string(), "docs/**".to_string()];
        issue.validation_commands = vec!["npm --prefix apps/desktop run build".to_string()];
        agentflow_spec::write_spec_issue(root, &issue).unwrap();

        let mut project = agentflow_spec::SpecProjectDraft::new("project-dispatcher");
        project.issue_ids = vec!["AF-DISPATCH-001".to_string()];
        let project =
            agentflow_spec::project_from_requirement(root, &requirement, project).unwrap();
        agentflow_spec::write_spec_project(root, &project).unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(root)
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "fixture"])
            .current_dir(root)
            .output()
            .unwrap();
    }

    fn commit_workspace(root: &Path, message: &str) {
        Command::new("git")
            .args(["add", "."])
            .current_dir(root)
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", message])
            .current_dir(root)
            .output()
            .unwrap();
    }

    #[test]
    fn dispatcher_claims_launch_and_writes_session_event() {
        let dir = tempdir().unwrap();
        write_fixture(dir.path());
        let loop_driver = TaskLoop::new("project-dispatcher");
        loop_driver
            .schedule_next_issue(dir.path())
            .unwrap()
            .unwrap();
        commit_workspace(dir.path(), "scheduled");
        loop_driver
            .request_agent_launch(dir.path(), "AF-DISPATCH-001", "codex")
            .unwrap();
        commit_workspace(dir.path(), "launch-request");
        let mut providers = McpProviderBridge::new();
        providers.register(Box::new(FakeProvider));

        let claim = AgentDispatcher::new(providers)
            .claim_next_launch(dir.path())
            .unwrap()
            .unwrap();

        assert_eq!(claim.issue_id, "AF-DISPATCH-001");
        assert_eq!(claim.provider, "codex");
        assert_eq!(claim.session_id, "fake-run-001");
        assert_eq!(claim.runtime_role.as_str(), "work-agent");
        assert_eq!(claim.selection.status.as_str(), "ready");
        assert_eq!(
            claim
                .skill_pack
                .as_ref()
                .map(|skill_pack| skill_pack.as_str()),
            Some("execution-skills")
        );
        let events = load_task_events(dir.path()).unwrap();
        assert!(events
            .iter()
            .any(|event| event.event_type == AGENT_LAUNCH_CLAIMED));
        assert!(events
            .iter()
            .any(|event| event.event_type == AGENT_SESSION_CREATED));
        let claimed = events
            .iter()
            .find(|event| event.event_type == AGENT_LAUNCH_CLAIMED)
            .unwrap();
        assert_eq!(claimed.payload["requestedRole"], "work-agent");
        assert_eq!(claimed.payload["runtimeRole"], "work-agent");
        assert_eq!(claimed.payload["selectionStatus"], "ready");
        let created = events
            .iter()
            .find(|event| event.event_type == AGENT_SESSION_CREATED)
            .unwrap();
        assert_eq!(created.payload["requestedRole"], "work-agent");
        assert_eq!(created.payload["runtimeRole"], "work-agent");
        assert_eq!(created.payload["selectionStatus"], "ready");
        let claimed_index = events
            .iter()
            .position(|event| event.event_type == AGENT_LAUNCH_CLAIMED)
            .unwrap();
        let created_index = events
            .iter()
            .position(|event| event.event_type == AGENT_SESSION_CREATED)
            .unwrap();
        assert!(claimed_index < created_index);
        let lease = agentflow_event_store::load_task_claim_lease(dir.path(), "run-001")
            .unwrap()
            .expect("expected claim lease");
        assert_eq!(lease.status.as_str(), "active");
        assert_eq!(lease.release_reason, None);
    }

    #[test]
    fn dispatcher_does_not_claim_same_run_twice() {
        let dir = tempdir().unwrap();
        write_fixture(dir.path());
        let loop_driver = TaskLoop::new("project-dispatcher");
        loop_driver
            .schedule_next_issue(dir.path())
            .unwrap()
            .unwrap();
        commit_workspace(dir.path(), "scheduled");
        loop_driver
            .request_agent_launch(dir.path(), "AF-DISPATCH-001", "codex")
            .unwrap();
        commit_workspace(dir.path(), "launch-request");
        let mut providers = McpProviderBridge::new();
        providers.register(Box::new(FakeProvider));
        let dispatcher = AgentDispatcher::new(providers);

        assert!(dispatcher.claim_next_launch(dir.path()).unwrap().is_some());
        assert!(dispatcher.claim_next_launch(dir.path()).unwrap().is_none());
    }

    #[test]
    fn dispatcher_reclaims_interrupted_run_as_resumed() {
        let dir = tempdir().unwrap();
        write_fixture(dir.path());
        let loop_driver = TaskLoop::new("project-dispatcher");
        loop_driver
            .schedule_next_issue(dir.path())
            .unwrap()
            .unwrap();
        commit_workspace(dir.path(), "scheduled");
        loop_driver
            .request_agent_launch(dir.path(), "AF-DISPATCH-001", "codex")
            .unwrap();
        commit_workspace(dir.path(), "launch-request");
        let mut providers = McpProviderBridge::new();
        providers.register(Box::new(FakeProvider));
        let dispatcher = AgentDispatcher::new(providers);

        let first_claim = dispatcher.claim_next_launch(dir.path()).unwrap().unwrap();
        append_task_event_once(
            dir.path(),
            TaskEventDraft {
                flow_type: WorkflowFlowType::Work,
                aggregate_type: "issue".to_string(),
                aggregate_id: "AF-DISPATCH-001".to_string(),
                project_id: Some("project-dispatcher".to_string()),
                issue_id: Some("AF-DISPATCH-001".to_string()),
                run_id: Some(first_claim.run_id.clone()),
                event_type: AGENT_SESSION_INTERRUPTED.to_string(),
                authority_role: Some(
                    AgentDispatchRoleBinding::resolve("build-agent")
                        .unwrap()
                        .runtime_role,
                ),
                actor: EventActor {
                    role: "agent-dispatcher".to_string(),
                    kind: "system".to_string(),
                },
                state: None,
                correlation_id: Some("corr-AF-DISPATCH-001".to_string()),
                causation_id: None,
                payload: json!({
                    "runId": first_claim.run_id,
                    "issueId": first_claim.issue_id,
                    "sessionId": first_claim.session_id,
                    "sessionStatus": "interrupted",
                }),
                artifact_refs: Vec::new(),
                idempotency_key: Some(format!(
                    "agent.session.interrupted:{}:{}",
                    first_claim.issue_id, first_claim.run_id
                )),
            },
        )
        .unwrap();
        commit_workspace(dir.path(), "session-interrupted");

        let resumed_claim = dispatcher.claim_next_launch(dir.path()).unwrap().unwrap();

        assert_eq!(resumed_claim.run_id, first_claim.run_id);
        assert_eq!(resumed_claim.runtime_role.as_str(), "work-agent");
        let events = load_task_events(dir.path()).unwrap();
        assert!(events
            .iter()
            .any(|event| event.event_type == AGENT_SESSION_RESUMED));
    }

    #[test]
    fn dispatcher_does_not_reclaim_cancelled_run() {
        let dir = tempdir().unwrap();
        write_fixture(dir.path());
        let loop_driver = TaskLoop::new("project-dispatcher");
        loop_driver
            .schedule_next_issue(dir.path())
            .unwrap()
            .unwrap();
        commit_workspace(dir.path(), "scheduled");
        loop_driver
            .request_agent_launch(dir.path(), "AF-DISPATCH-001", "codex")
            .unwrap();
        commit_workspace(dir.path(), "launch-request");
        let mut providers = McpProviderBridge::new();
        providers.register(Box::new(FakeProvider));
        let dispatcher = AgentDispatcher::new(providers);

        let first_claim = dispatcher.claim_next_launch(dir.path()).unwrap().unwrap();
        append_task_event_once(
            dir.path(),
            TaskEventDraft {
                flow_type: WorkflowFlowType::Work,
                aggregate_type: "issue".to_string(),
                aggregate_id: "AF-DISPATCH-001".to_string(),
                project_id: Some("project-dispatcher".to_string()),
                issue_id: Some("AF-DISPATCH-001".to_string()),
                run_id: Some(first_claim.run_id.clone()),
                event_type: AGENT_SESSION_INTERRUPTED.to_string(),
                authority_role: Some(
                    AgentDispatchRoleBinding::resolve("build-agent")
                        .unwrap()
                        .runtime_role,
                ),
                actor: EventActor {
                    role: "agent-dispatcher".to_string(),
                    kind: "system".to_string(),
                },
                state: None,
                correlation_id: Some("corr-AF-DISPATCH-001".to_string()),
                causation_id: None,
                payload: json!({
                    "runId": first_claim.run_id,
                    "issueId": first_claim.issue_id,
                    "sessionId": first_claim.session_id,
                    "sessionStatus": "interrupted",
                }),
                artifact_refs: Vec::new(),
                idempotency_key: Some(
                    "agent.session.interrupted:AF-DISPATCH-001:run-001".to_string(),
                ),
            },
        )
        .unwrap();
        commit_workspace(dir.path(), "session-interrupted");
        assert!(dispatcher.claim_next_launch(dir.path()).unwrap().is_some());

        append_task_event_once(
            dir.path(),
            TaskEventDraft {
                flow_type: WorkflowFlowType::Work,
                aggregate_type: "issue".to_string(),
                aggregate_id: "AF-DISPATCH-001".to_string(),
                project_id: Some("project-dispatcher".to_string()),
                issue_id: Some("AF-DISPATCH-001".to_string()),
                run_id: Some("run-001".to_string()),
                event_type: AGENT_SESSION_CANCELLED.to_string(),
                authority_role: Some(
                    AgentDispatchRoleBinding::resolve("build-agent")
                        .unwrap()
                        .runtime_role,
                ),
                actor: EventActor {
                    role: "agent-dispatcher".to_string(),
                    kind: "system".to_string(),
                },
                state: None,
                correlation_id: Some("corr-AF-DISPATCH-001".to_string()),
                causation_id: None,
                payload: json!({
                    "runId": "run-001",
                    "issueId": "AF-DISPATCH-001",
                    "sessionId": "fake-run-001",
                    "sessionStatus": "cancelled",
                }),
                artifact_refs: Vec::new(),
                idempotency_key: Some(
                    "agent.session.cancelled:AF-DISPATCH-001:run-001".to_string(),
                ),
            },
        )
        .unwrap();
        commit_workspace(dir.path(), "session-cancelled");

        assert!(dispatcher.claim_next_launch(dir.path()).unwrap().is_none());
    }

    #[test]
    fn dispatcher_records_failed_claim_before_retrying_same_run() {
        let dir = tempdir().unwrap();
        write_fixture(dir.path());
        let loop_driver = TaskLoop::new("project-dispatcher");
        loop_driver
            .schedule_next_issue(dir.path())
            .unwrap()
            .unwrap();
        commit_workspace(dir.path(), "scheduled");
        loop_driver
            .request_agent_launch(dir.path(), "AF-DISPATCH-001", "codex")
            .unwrap();
        commit_workspace(dir.path(), "launch-request");

        let attempts = Arc::new(AtomicUsize::new(0));
        let mut failing_providers = McpProviderBridge::new();
        failing_providers.register(Box::new(FailOnceProvider {
            attempts: Arc::clone(&attempts),
        }));
        let error = AgentDispatcher::new(failing_providers)
            .claim_next_launch(dir.path())
            .unwrap_err()
            .to_string();
        assert!(error.contains("create session failed"));

        let events = load_task_events(dir.path()).unwrap();
        assert!(events
            .iter()
            .any(|event| event.event_type == AGENT_LAUNCH_CLAIMED));
        assert!(events
            .iter()
            .any(|event| event.event_type == AGENT_SESSION_FAILED));
        let failed_lease = agentflow_event_store::load_task_claim_lease(dir.path(), "run-001")
            .unwrap()
            .expect("expected released lease after failed launch");
        assert_eq!(failed_lease.status.as_str(), "released");
        assert_eq!(
            failed_lease.release_reason.as_deref(),
            Some("provider-launch-failed")
        );

        let mut retry_providers = McpProviderBridge::new();
        retry_providers.register(Box::new(FailOnceProvider { attempts }));
        let claim = AgentDispatcher::new(retry_providers)
            .claim_next_launch(dir.path())
            .unwrap()
            .unwrap();

        assert_eq!(claim.run_id, "run-001");
        let events = load_task_events(dir.path()).unwrap();
        assert!(events
            .iter()
            .any(|event| event.event_type == AGENT_SESSION_RESUMED));
    }

    #[test]
    fn dispatcher_recovers_existing_session_snapshot_without_relaunching() {
        let dir = tempdir().unwrap();
        write_fixture(dir.path());
        let loop_driver = TaskLoop::new("project-dispatcher");
        loop_driver
            .schedule_next_issue(dir.path())
            .unwrap()
            .unwrap();
        commit_workspace(dir.path(), "scheduled");
        loop_driver
            .request_agent_launch(dir.path(), "AF-DISPATCH-001", "codex")
            .unwrap();
        commit_workspace(dir.path(), "launch-request");

        let request = McpLaunchRequest::new(
            "codex",
            "AF-DISPATCH-001",
            "run-001",
            "build-agent",
            dir.path().display().to_string(),
            ".agentflow/tasks/AF-DISPATCH-001/runs/run-001/launch/agent-request.json",
        );
        let provider = NoCreateProvider;
        let plan = provider.build_launch_plan(dir.path(), &request).unwrap();
        let mut session = McpSessionSnapshot::queued(&request, &plan, 1);
        session.status = McpSessionStatus::Running;
        session.updated_at = session.updated_at + 5;
        write_launch_plan(dir.path(), &plan).unwrap();
        write_session_snapshot(dir.path(), &session).unwrap();
        commit_workspace(dir.path(), "recovered-session");

        let mut providers = McpProviderBridge::new();
        providers.register(Box::new(NoCreateProvider));
        let claim = AgentDispatcher::new(providers)
            .claim_next_launch(dir.path())
            .unwrap()
            .unwrap();

        assert_eq!(claim.issue_id, "AF-DISPATCH-001");
        assert_eq!(claim.run_id, "run-001");
        assert_eq!(claim.session_id, session.session_id);
        let events = load_task_events(dir.path()).unwrap();
        assert!(events
            .iter()
            .any(|event| event.event_type == AGENT_SESSION_CREATED));
        assert!(events
            .iter()
            .any(|event| event.event_type == AGENT_SESSION_RUNNING));
        let lease = agentflow_event_store::load_task_claim_lease(dir.path(), "run-001")
            .unwrap()
            .expect("expected active lease after recovery");
        assert_eq!(lease.status.as_str(), "active");
        assert_eq!(lease.release_reason, None);
    }

    #[test]
    fn dispatcher_role_binding_maps_build_agent_to_work_agent() {
        let binding = AgentDispatchRoleBinding::resolve("build-agent").unwrap();

        assert_eq!(binding.runtime_role.as_str(), "work-agent");
        assert_eq!(binding.provider_role, "build-agent");
        assert_eq!(
            binding
                .skill_pack
                .as_ref()
                .map(|skill_pack| skill_pack.as_str()),
            Some("execution-skills")
        );
    }

    #[test]
    fn dispatcher_records_non_runnable_provider_failure_as_authoritative_event() {
        let dir = tempdir().unwrap();
        write_fixture(dir.path());
        let loop_driver = TaskLoop::new("project-dispatcher");
        loop_driver
            .schedule_next_issue(dir.path())
            .unwrap()
            .unwrap();
        commit_workspace(dir.path(), "scheduled");
        loop_driver
            .request_agent_launch(dir.path(), "AF-DISPATCH-001", "codex")
            .unwrap();
        commit_workspace(dir.path(), "launch-request");

        let mut providers = McpProviderBridge::new();
        providers.register(Box::new(LimitedProvider));
        let error = AgentDispatcher::new(providers)
            .claim_next_launch(dir.path())
            .unwrap_err()
            .to_string();

        assert!(error.contains("build_agent.complete"));
        let events = load_task_events(dir.path()).unwrap();
        let failed = events
            .iter()
            .find(|event| event.event_type == AGENT_SESSION_FAILED)
            .expect("expected failed session event");
        assert_eq!(failed.payload["retryable"], false);
        assert_eq!(failed.payload["terminalReason"], "provider-not-runnable");
    }
}
