use crate::model::{
    AgentDispatcherClaim, AGENT_SESSION_CREATED, AGENT_SESSION_DONE, AGENT_SESSION_FAILED,
    AGENT_SESSION_IN_REVIEW, AGENT_SESSION_RUNNING,
};
use agentflow_event_store::{
    append_task_event_once, load_task_events, EventActor, TaskEvent, TaskEventDraft,
};
use agentflow_mcp::{
    default_provider_bridge, McpLaunchRequest, McpProviderBridge, McpSessionSnapshot,
    McpSessionStatus,
};
use agentflow_task_loop::{AgentLaunchPayload, AGENT_LAUNCH_REQUESTED};
use anyhow::{Context, Result};
use serde_json::json;
use std::{collections::BTreeSet, path::Path};

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
        let claimed_runs = claimed_run_ids(&events);
        let Some(event) = events.into_iter().find(|event| {
            event.event_type == AGENT_LAUNCH_REQUESTED
                && event
                    .payload
                    .get("runId")
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|run_id| !claimed_runs.contains(run_id))
        }) else {
            return Ok(None);
        };

        let payload: AgentLaunchPayload = serde_json::from_value(event.payload.clone())
            .with_context(|| format!("parse launch payload {}", event.event_id))?;
        let request = launch_payload_to_mcp_request(&payload);
        let provider = self
            .providers
            .provider(&payload.provider)
            .ok_or_else(|| anyhow::anyhow!("provider {} is not registered", payload.provider))?;
        let session = provider.create_session(root, &request)?;
        let created_event = append_session_event(root, &payload, &session, AGENT_SESSION_CREATED)?;
        append_status_event(root, &payload, &session)?;

        Ok(Some(AgentDispatcherClaim {
            issue_id: payload.issue_id,
            run_id: payload.run_id,
            provider: session.provider,
            session_id: session.session_id,
            session_status: session.status.as_str().to_string(),
            created_event_id: created_event.event_id,
        }))
    }
}

fn claimed_run_ids(events: &[TaskEvent]) -> BTreeSet<String> {
    events
        .iter()
        .filter(|event| event.event_type == AGENT_SESSION_CREATED)
        .filter_map(|event| {
            event
                .payload
                .get("runId")
                .and_then(serde_json::Value::as_str)
                .map(str::to_string)
        })
        .collect()
}

fn launch_payload_to_mcp_request(payload: &AgentLaunchPayload) -> McpLaunchRequest {
    let mut request = McpLaunchRequest::new(
        payload.provider.clone(),
        payload.issue_id.clone(),
        payload.run_id.clone(),
        payload.agent_role.clone(),
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
        McpSessionStatus::InReview => AGENT_SESSION_IN_REVIEW,
        McpSessionStatus::Done => AGENT_SESSION_DONE,
        McpSessionStatus::Failed | McpSessionStatus::Cancelled => AGENT_SESSION_FAILED,
        McpSessionStatus::Queued => return Ok(None),
    };
    append_session_event(root, payload, session, event_type).map(Some)
}

fn append_session_event(
    root: &Path,
    payload: &AgentLaunchPayload,
    session: &McpSessionSnapshot,
    event_type: &str,
) -> Result<TaskEvent> {
    append_task_event_once(
        root,
        TaskEventDraft {
            aggregate_type: "issue".to_string(),
            aggregate_id: payload.issue_id.clone(),
            project_id: payload.project_id.clone(),
            issue_id: Some(payload.issue_id.clone()),
            event_type: event_type.to_string(),
            actor: EventActor {
                role: "agent-dispatcher".to_string(),
                kind: "system".to_string(),
            },
            state: None,
            correlation_id: Some(format!("corr-{}", payload.issue_id)),
            causation_id: None,
            payload: json!({
                "provider": session.provider,
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
            idempotency_key: Some(format!("{event_type}:{}", session.run_id)),
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
        let events = load_task_events(dir.path()).unwrap();
        assert!(events
            .iter()
            .any(|event| event.event_type == AGENT_SESSION_CREATED));
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
}
