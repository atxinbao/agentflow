use crate::model::{
    AgentDispatchRoleBinding, AgentDispatcherClaim, AGENT_LAUNCH_CLAIMED, AGENT_SESSION_CREATED,
    AGENT_SESSION_DONE, AGENT_SESSION_FAILED, AGENT_SESSION_INTERRUPTED, AGENT_SESSION_IN_REVIEW,
    AGENT_SESSION_RESUMED, AGENT_SESSION_RUNNING,
};
use agentflow_event_store::{
    append_task_event_once, load_task_events, EventActor, TaskEvent, TaskEventDraft,
};
use agentflow_mcp::{
    default_provider_bridge, McpLaunchRequest, McpProviderBridge, McpSessionSnapshot,
    McpSessionStatus,
};
use agentflow_task_loop::{AgentLaunchPayload, AGENT_LAUNCH_REQUESTED};
use agentflow_workflow_core::WorkflowFlowType;
use anyhow::{Context, Result};
use serde_json::json;
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
        let events = load_task_events(root)?;
        let unavailable_runs = unavailable_run_ids(&events);
        let Some(event) = events.into_iter().find(|event| {
            event.event_type == AGENT_LAUNCH_REQUESTED
                && event
                    .run_id
                    .as_deref()
                    .is_some_and(|run_id| !unavailable_runs.get(run_id).copied().unwrap_or(false))
        }) else {
            return Ok(None);
        };

        let payload: AgentLaunchPayload = serde_json::from_value(event.payload.clone())
            .with_context(|| format!("parse launch payload {}", event.event_id))?;
        let role_binding = AgentDispatchRoleBinding::resolve(payload.agent_role.clone())?;
        let request = launch_payload_to_mcp_request(&payload, &role_binding);
        let provider = self
            .providers
            .provider(&payload.provider)
            .ok_or_else(|| anyhow::anyhow!("provider {} is not registered", payload.provider))?;
        let session = provider.create_session(root, &request)?;
        append_launch_claimed_event(root, &payload, &role_binding, &session)?;
        let created_event = append_session_event(
            root,
            &payload,
            &role_binding,
            &session,
            if had_prior_session_event(root, &payload.run_id)? {
                AGENT_SESSION_RESUMED
            } else {
                AGENT_SESSION_CREATED
            },
        )?;
        append_status_event(root, &payload, &session)?;

        Ok(Some(AgentDispatcherClaim {
            issue_id: payload.issue_id,
            run_id: payload.run_id,
            provider: session.provider,
            session_id: session.session_id,
            session_status: session.status.as_str().to_string(),
            runtime_role: role_binding.runtime_role,
            skill_pack: role_binding.skill_pack,
            created_event_id: created_event.event_id,
        }))
    }
}

fn unavailable_run_ids(events: &[TaskEvent]) -> BTreeMap<String, bool> {
    let mut state = BTreeMap::new();
    for event in events {
        let Some(run_id) = event.run_id.clone() else {
            continue;
        };
        match event.event_type.as_str() {
            AGENT_SESSION_CREATED
            | AGENT_SESSION_RESUMED
            | AGENT_SESSION_RUNNING
            | AGENT_SESSION_IN_REVIEW
            | AGENT_SESSION_DONE => {
                state.insert(run_id, true);
            }
            AGENT_SESSION_INTERRUPTED | AGENT_SESSION_FAILED => {
                state.insert(run_id, false);
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
        ) && event.run_id.as_deref() == Some(run_id)
    }))
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

fn append_status_event(
    root: &Path,
    payload: &AgentLaunchPayload,
    session: &McpSessionSnapshot,
) -> Result<Option<TaskEvent>> {
    let event_type = match session.status {
        McpSessionStatus::Running | McpSessionStatus::Starting | McpSessionStatus::Claimed => {
            AGENT_SESSION_RUNNING
        }
        McpSessionStatus::Interrupted => AGENT_SESSION_INTERRUPTED,
        McpSessionStatus::InReview => AGENT_SESSION_IN_REVIEW,
        McpSessionStatus::Done => AGENT_SESSION_DONE,
        McpSessionStatus::Failed | McpSessionStatus::Cancelled => AGENT_SESSION_FAILED,
        McpSessionStatus::Queued => return Ok(None),
    };
    let role_binding = AgentDispatchRoleBinding::resolve(payload.agent_role.clone())?;
    append_session_event(root, payload, &role_binding, session, event_type).map(Some)
}

fn append_session_event(
    root: &Path,
    payload: &AgentLaunchPayload,
    role_binding: &AgentDispatchRoleBinding,
    session: &McpSessionSnapshot,
    event_type: &str,
) -> Result<TaskEvent> {
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
            payload: json!({
                "provider": session.provider,
                "requestedRole": role_binding.requested_role,
                "runtimeRole": role_binding.runtime_role.as_str(),
                "skillPack": role_binding.skill_pack.map(|value| value.as_str()),
                "sessionId": session.session_id,
                "sessionStatus": session.status.as_str(),
                "issueId": session.issue_id,
                "projectId": session.project_id,
                "runId": session.run_id,
                "branchName": session.branch_name,
                "launchRequestPath": session.launch_request_path,
                "planPath": session.plan_path,
                "logPath": session.log_path,
            }),
            artifact_refs: session_artifact_refs(session),
            idempotency_key: Some(format!(
                "{event_type}:{}:{}",
                session.issue_id, session.run_id
            )),
        },
    )
}

fn append_launch_claimed_event(
    root: &Path,
    payload: &AgentLaunchPayload,
    role_binding: &AgentDispatchRoleBinding,
    session: &McpSessionSnapshot,
) -> Result<TaskEvent> {
    append_task_event_once(
        root,
        TaskEventDraft {
            flow_type: WorkflowFlowType::Work,
            aggregate_type: "issue".to_string(),
            aggregate_id: payload.issue_id.clone(),
            project_id: payload.project_id.clone(),
            issue_id: Some(payload.issue_id.clone()),
            run_id: Some(session.run_id.clone()),
            event_type: AGENT_LAUNCH_CLAIMED.to_string(),
            authority_role: Some(role_binding.runtime_role),
            actor: EventActor {
                role: "agent-dispatcher".to_string(),
                kind: "system".to_string(),
            },
            state: None,
            correlation_id: Some(format!("corr-{}", payload.issue_id)),
            causation_id: None,
            payload: json!({
                "provider": session.provider,
                "requestedRole": role_binding.requested_role,
                "runtimeRole": role_binding.runtime_role.as_str(),
                "skillPack": role_binding.skill_pack.map(|value| value.as_str()),
                "sessionId": session.session_id,
                "sessionStatus": session.status.as_str(),
                "issueId": session.issue_id,
                "projectId": session.project_id,
                "runId": session.run_id,
                "branchName": session.branch_name,
                "launchRequestPath": session.launch_request_path,
                "planPath": session.plan_path,
                "logPath": session.log_path,
            }),
            artifact_refs: session_artifact_refs(session),
            idempotency_key: Some(format!(
                "{AGENT_LAUNCH_CLAIMED}:{}:{}",
                session.issue_id, session.run_id
            )),
        },
    )
}

fn session_artifact_refs(session: &McpSessionSnapshot) -> Vec<String> {
    let mut refs = vec![session.plan_path.clone()];
    if let Some(log_path) = session.log_path.clone() {
        refs.push(log_path);
    }
    refs
}

#[cfg(test)]
mod tests {
    use super::*;
    use agentflow_mcp::{
        McpAgentProvider, McpLaunchMode, McpLaunchPlan, McpProviderKind, McpProviderStatus,
        McpProviderStatusCode,
    };
    use agentflow_task_loop::TaskLoop;
    use std::{fs, path::Path};
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
            status
        }

        fn build_launch_plan(
            &self,
            _project_root: &Path,
            request: &McpLaunchRequest,
        ) -> Result<McpLaunchPlan> {
            let mut plan = McpLaunchPlan::new(
                "fake",
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

    fn write_fixture(root: &Path) {
        let requirement = root.join("docs/requirements/034-test.md");
        fs::create_dir_all(requirement.parent().unwrap()).unwrap();
        fs::write(&requirement, "# 测试需求\n\n用于 agent-dispatcher 测试。\n").unwrap();

        let mut issue = agentflow_spec::SpecIssueDraft::new("AF-DISPATCH-001");
        issue.project_id = Some("project-dispatcher".to_string());
        let issue = agentflow_spec::issue_from_requirement(root, &requirement, issue).unwrap();
        agentflow_spec::write_spec_issue(root, &issue).unwrap();

        let mut project = agentflow_spec::SpecProjectDraft::new("project-dispatcher");
        project.issue_ids = vec!["AF-DISPATCH-001".to_string()];
        let project =
            agentflow_spec::project_from_requirement(root, &requirement, project).unwrap();
        agentflow_spec::write_spec_project(root, &project).unwrap();
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
        loop_driver
            .request_agent_launch(dir.path(), "AF-DISPATCH-001", "fake")
            .unwrap();
        let mut providers = McpProviderBridge::new();
        providers.register(Box::new(FakeProvider));

        let claim = AgentDispatcher::new(providers)
            .claim_next_launch(dir.path())
            .unwrap()
            .unwrap();

        assert_eq!(claim.issue_id, "AF-DISPATCH-001");
        assert_eq!(claim.provider, "fake");
        assert_eq!(claim.session_id, "fake-run-001");
        assert_eq!(claim.runtime_role.as_str(), "work-agent");
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
        assert_eq!(claimed.payload["requestedRole"], "build-agent");
        assert_eq!(claimed.payload["runtimeRole"], "work-agent");
        let created = events
            .iter()
            .find(|event| event.event_type == AGENT_SESSION_CREATED)
            .unwrap();
        assert_eq!(created.payload["requestedRole"], "build-agent");
        assert_eq!(created.payload["runtimeRole"], "work-agent");
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
        loop_driver
            .request_agent_launch(dir.path(), "AF-DISPATCH-001", "fake")
            .unwrap();
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
        loop_driver
            .request_agent_launch(dir.path(), "AF-DISPATCH-001", "fake")
            .unwrap();
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

        let resumed_claim = dispatcher.claim_next_launch(dir.path()).unwrap().unwrap();

        assert_eq!(resumed_claim.run_id, first_claim.run_id);
        assert_eq!(resumed_claim.runtime_role.as_str(), "work-agent");
        let events = load_task_events(dir.path()).unwrap();
        assert!(events
            .iter()
            .any(|event| event.event_type == AGENT_SESSION_RESUMED));
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
}
