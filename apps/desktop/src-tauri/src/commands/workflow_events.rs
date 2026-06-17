use agentflow_agent_dispatcher::{AgentDispatcher, AGENT_SESSION_CREATED};
use agentflow_event_store::{
    append_task_dead_letter, append_task_event_once, load_pending_task_events, load_task_events,
    mark_task_event_consumed, prepare_event_store, ContextPackFailedPayload,
    ContextPackReadyPayload, ContextPackRequestedPayload, EventActor, IssueReadyPayload,
    TaskEventDraft, CONSUMER_PANEL, EVENT_TYPE_PANEL_CONTEXT_PACK_FAILED,
    EVENT_TYPE_PANEL_CONTEXT_PACK_READY, EVENT_TYPE_PANEL_CONTEXT_PACK_REQUESTED,
    EVENT_TYPE_SPEC_ISSUE_READY,
};
use agentflow_panel::PanelStatus;
use agentflow_task_loop::AGENT_LAUNCH_REQUESTED;
use agentflow_workflow_core::{WorkflowAgentRole, WorkflowFlowType};
use serde::Serialize;
use std::path::Path;
use tauri::{AppHandle, Emitter};

const WORKFLOW_EVENT_DISPATCH_VERSION: &str = "workflow-event-dispatch.v1";
pub(crate) const AGENTFLOW_WORKFLOW_EVENTS_DISPATCHED_EVENT: &str =
    "agentflow-task-events-dispatched";

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WorkflowEventDispatchSummary {
    version: String,
    emitted_issue_ready_events: usize,
    pending_panel_events: usize,
    pending_build_agent_launch_events: usize,
    context_pack_requests: usize,
    context_pack_ready: usize,
    context_pack_failed: usize,
    build_agent_launch_sessions_created: usize,
    errors: Vec<String>,
}

impl WorkflowEventDispatchSummary {
    fn should_refresh_ui(&self) -> bool {
        self.context_pack_ready > 0
            || self.context_pack_failed > 0
            || self.build_agent_launch_sessions_created > 0
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct WorkflowEventsDispatchedEvent {
    version: String,
    project_root: String,
    pending_panel_events: usize,
    pending_build_agent_launch_events: usize,
    context_pack_requests: usize,
    context_pack_ready: usize,
    context_pack_failed: usize,
    build_agent_launch_sessions_created: usize,
    errors: Vec<String>,
}

#[tauri::command]
pub(crate) fn dispatch_workflow_events(
    project_root: String,
    app: AppHandle,
) -> Result<WorkflowEventDispatchSummary, String> {
    dispatch_workflow_events_for_app(project_root, &app)
}

pub(crate) fn dispatch_workflow_events_for_app(
    project_root: impl AsRef<Path>,
    app: &AppHandle,
) -> Result<WorkflowEventDispatchSummary, String> {
    dispatch_workflow_events_inner(project_root, Some(app))
}

fn dispatch_workflow_events_inner(
    project_root: impl AsRef<Path>,
    app: Option<&AppHandle>,
) -> Result<WorkflowEventDispatchSummary, String> {
    let root = project_root
        .as_ref()
        .canonicalize()
        .map_err(|error| format!("canonicalize {}: {error}", project_root.as_ref().display()))?;
    prepare_event_store(&root).map_err(|error| error.to_string())?;

    let mut summary = WorkflowEventDispatchSummary {
        version: WORKFLOW_EVENT_DISPATCH_VERSION.to_string(),
        ..WorkflowEventDispatchSummary::default()
    };

    let issue_status_index = agentflow_state::load_issue_status_index(&root).ok();
    let pending = load_pending_task_events(&root, CONSUMER_PANEL, &[EVENT_TYPE_SPEC_ISSUE_READY])
        .map_err(|error| error.to_string())?;
    summary.pending_panel_events = pending.len();
    if panel_ready_for_context_pack(&root) {
        for event in pending {
            let payload = match serde_json::from_value::<IssueReadyPayload>(event.payload.clone()) {
                Ok(payload) => payload,
                Err(error) => {
                    let message = format!("parse issue ready payload: {error}");
                    let _ = append_task_dead_letter(&root, CONSUMER_PANEL, &event, message.clone());
                    let _ = mark_task_event_consumed(&root, CONSUMER_PANEL, &event.event_id);
                    summary.errors.push(message);
                    summary.context_pack_failed += 1;
                    continue;
                }
            };
            if !event_ready_for_panel(&payload, issue_status_index.as_ref()) {
                mark_task_event_consumed(&root, CONSUMER_PANEL, &event.event_id)
                    .map_err(|error| error.to_string())?;
                continue;
            }

            let requested = ContextPackRequestedPayload {
                issue_id: payload.issue_id.clone(),
                context_pack_path: payload.context_pack_path.clone(),
            };
            if let Err(error) = append_context_pack_event(
                &root,
                EVENT_TYPE_PANEL_CONTEXT_PACK_REQUESTED,
                "panel",
                &payload.issue_id,
                payload.context_pack_path.clone(),
                format!(
                    "panel.context-pack.requested:{}:{}",
                    payload.issue_id, event.event_id
                ),
                requested,
            ) {
                summary.errors.push(error.to_string());
            } else {
                summary.context_pack_requests += 1;
            }

            match ensure_context_pack(&root, &payload) {
                Ok(context_pack_path) => {
                    let ready = ContextPackReadyPayload {
                        issue_id: payload.issue_id.clone(),
                        context_pack_path: context_pack_path.clone(),
                    };
                    append_context_pack_event(
                        &root,
                        EVENT_TYPE_PANEL_CONTEXT_PACK_READY,
                        "panel",
                        &payload.issue_id,
                        Some(context_pack_path),
                        format!(
                            "panel.context-pack.ready:{}:{}",
                            payload.issue_id, event.event_id
                        ),
                        ready,
                    )
                    .map_err(|error| error.to_string())?;
                    mark_task_event_consumed(&root, CONSUMER_PANEL, &event.event_id)
                        .map_err(|error| error.to_string())?;
                    summary.context_pack_ready += 1;
                }
                Err(error) => {
                    let message = error.to_string();
                    let failed = ContextPackFailedPayload {
                        issue_id: payload.issue_id.clone(),
                        context_pack_path: payload.context_pack_path.clone(),
                        error: message.clone(),
                    };
                    let _ = append_context_pack_event(
                        &root,
                        EVENT_TYPE_PANEL_CONTEXT_PACK_FAILED,
                        "panel",
                        &payload.issue_id,
                        payload.context_pack_path.clone(),
                        format!(
                            "panel.context-pack.failed:{}:{}",
                            payload.issue_id, event.event_id
                        ),
                        failed,
                    );
                    let _ = append_task_dead_letter(&root, CONSUMER_PANEL, &event, message.clone());
                    mark_task_event_consumed(&root, CONSUMER_PANEL, &event.event_id)
                        .map_err(|error| error.to_string())?;
                    summary.errors.push(message);
                    summary.context_pack_failed += 1;
                }
            }
        }
    }

    if let Err(error) = dispatch_build_agent_launch_events(&root, &mut summary) {
        summary.errors.push(error.to_string());
    }

    let _ = prepare_event_store(&root);
    if summary.should_refresh_ui() {
        if let Some(app) = app {
            let payload = WorkflowEventsDispatchedEvent {
                version: WORKFLOW_EVENT_DISPATCH_VERSION.to_string(),
                project_root: root.display().to_string(),
                pending_panel_events: summary.pending_panel_events,
                pending_build_agent_launch_events: summary.pending_build_agent_launch_events,
                context_pack_requests: summary.context_pack_requests,
                context_pack_ready: summary.context_pack_ready,
                context_pack_failed: summary.context_pack_failed,
                build_agent_launch_sessions_created: summary.build_agent_launch_sessions_created,
                errors: summary.errors.clone(),
            };
            let _ = app.emit(AGENTFLOW_WORKFLOW_EVENTS_DISPATCHED_EVENT, payload);
        }
    }
    Ok(summary)
}

fn panel_ready_for_context_pack(root: &Path) -> bool {
    agentflow_panel::load_project_panel_status(root)
        .map(|status| matches!(status.status, PanelStatus::Ready))
        .unwrap_or(false)
}

fn event_ready_for_panel(
    payload: &IssueReadyPayload,
    issue_status_index: Option<&agentflow_state::IssueStatusIndex>,
) -> bool {
    issue_status_index
        .and_then(|index| {
            index
                .issues
                .iter()
                .find(|entry| entry.issue_id == payload.issue_id)
        })
        .map(|entry| {
            matches!(
                entry.display_status.as_str(),
                "backlog" | "todo" | "blocked"
            )
        })
        .unwrap_or(true)
}

fn ensure_context_pack(root: &Path, payload: &IssueReadyPayload) -> anyhow::Result<String> {
    if let Some(path) = payload.context_pack_path.as_deref() {
        if root.join(path).is_file() {
            return Ok(path.to_string());
        }
    }

    agentflow_panel::build_panel_context_pack(
        root,
        "issue",
        Some(&payload.issue_id),
        &payload.title,
        &payload.objective,
        &payload.acceptance_criteria,
    )?;

    let context_pack_path = payload.context_pack_path.clone().unwrap_or_else(|| {
        format!(
            ".agentflow/panel/context-packs/{}.json",
            payload.issue_id.replace('/', "-")
        )
    });
    Ok(context_pack_path)
}

fn append_context_pack_event<T: Serialize>(
    root: &Path,
    event_type: &str,
    source: &str,
    issue_id: &str,
    subject_path: Option<String>,
    dedupe_key: String,
    payload: T,
) -> anyhow::Result<()> {
    append_task_event_once(
        root,
        TaskEventDraft {
            flow_type: WorkflowFlowType::Work,
            aggregate_type: "issue".to_string(),
            aggregate_id: issue_id.to_string(),
            project_id: None,
            issue_id: Some(issue_id.to_string()),
            run_id: None,
            event_type: event_type.to_string(),
            authority_role: Some(WorkflowAgentRole::WorkAgent),
            actor: EventActor {
                role: source.to_string(),
                kind: "system".to_string(),
            },
            state: None,
            correlation_id: Some(format!("corr-{issue_id}")),
            causation_id: None,
            payload: serde_json::to_value(payload)?,
            artifact_refs: subject_path.into_iter().collect(),
            idempotency_key: Some(dedupe_key),
        },
    )?;
    Ok(())
}

fn dispatch_build_agent_launch_events(
    root: &Path,
    summary: &mut WorkflowEventDispatchSummary,
) -> anyhow::Result<()> {
    summary.pending_build_agent_launch_events = pending_agent_launch_count(root)?;
    if summary.pending_build_agent_launch_events == 0 {
        return Ok(());
    }

    let dispatcher = AgentDispatcher::with_default_providers();
    loop {
        match dispatcher.claim_next_launch(root) {
            Ok(Some(_claim)) => {
                summary.build_agent_launch_sessions_created += 1;
            }
            Ok(None) => break,
            Err(error) => {
                summary
                    .errors
                    .push(format!("claim agent launch from task events: {error}"));
                break;
            }
        }
    }

    Ok(())
}

fn pending_agent_launch_count(root: &Path) -> anyhow::Result<usize> {
    let events = load_task_events(root)?;
    let claimed_runs = events
        .iter()
        .filter(|event| event.event_type == AGENT_SESSION_CREATED)
        .filter_map(|event| event.run_id.clone())
        .collect::<std::collections::BTreeSet<_>>();
    Ok(events
        .iter()
        .filter(|event| event.event_type == AGENT_LAUNCH_REQUESTED)
        .filter(|event| {
            event
                .run_id
                .as_deref()
                .is_some_and(|run_id| !claimed_runs.contains(run_id))
        })
        .count())
}

#[cfg(test)]
mod tests {
    use super::*;
    use agentflow_event_store::{append_task_event_once, EventActor, TaskEventDraft};
    use agentflow_state::{IssueStatusIndex, IssueStatusIndexEntry, WorkflowAuditStatus};
    use serde_json::json;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn issue_ready_filter_uses_state_display_status() {
        let payload = issue_ready_payload("iss-done", "todo");
        assert!(event_ready_for_panel(&payload, None));

        let issue_status_index = IssueStatusIndex {
            version: "state-issue-status-index.v1".to_string(),
            updated_at: 1,
            issues: vec![IssueStatusIndexEntry {
                issue_id: payload.issue_id.clone(),
                display_status: "done".to_string(),
                priority: "p2".to_string(),
                execution_risk: "normal".to_string(),
                latest_run_id: Some("run-001".to_string()),
                execute_status: Some("completed".to_string()),
                evidence_status: "ready".to_string(),
                delivery_status: "ready".to_string(),
                audit_status: WorkflowAuditStatus::NotRequested,
            }],
        };

        assert!(!event_ready_for_panel(&payload, Some(&issue_status_index)));
        let blocked_issue_status_index = IssueStatusIndex {
            version: "state-issue-status-index.v1".to_string(),
            updated_at: 1,
            issues: vec![IssueStatusIndexEntry {
                issue_id: payload.issue_id.clone(),
                display_status: "blocked".to_string(),
                priority: "p2".to_string(),
                execution_risk: "normal".to_string(),
                latest_run_id: None,
                execute_status: None,
                evidence_status: "missing".to_string(),
                delivery_status: "missing".to_string(),
                audit_status: WorkflowAuditStatus::NotRequested,
            }],
        };
        assert!(event_ready_for_panel(
            &payload,
            Some(&blocked_issue_status_index)
        ));
    }

    #[test]
    fn dispatch_generates_context_pack_for_ready_spec_issue_event() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("apps/desktop/src")).unwrap();
        fs::write(
            dir.path().join("apps/desktop/src/App.tsx"),
            "export function App() { return null; }\n",
        )
        .unwrap();
        agentflow_panel::prepare_project_panel(
            dir.path(),
            agentflow_panel::PanelPrepareMode::Blocking,
        )
        .unwrap();
        append_task_event_once(
            dir.path(),
            TaskEventDraft {
                flow_type: WorkflowFlowType::Work,
                aggregate_type: "issue".to_string(),
                aggregate_id: "iss-context".to_string(),
                project_id: None,
                issue_id: Some("iss-context".to_string()),
                run_id: None,
                event_type: EVENT_TYPE_SPEC_ISSUE_READY.to_string(),
                authority_role: Some(WorkflowAgentRole::WorkAgent),
                actor: EventActor {
                    role: "spec".to_string(),
                    kind: "system".to_string(),
                },
                state: None,
                correlation_id: Some("corr-iss-context".to_string()),
                causation_id: None,
                payload: serde_json::to_value(issue_ready_payload("iss-context", "todo")).unwrap(),
                artifact_refs: vec![".agentflow/spec/issues/iss-context.json".to_string()],
                idempotency_key: Some("issue-ready:iss-context".to_string()),
            },
        )
        .unwrap();

        let summary = dispatch_workflow_events_inner(dir.path(), None).unwrap();

        assert_eq!(summary.context_pack_ready, 1);
        assert!(dir
            .path()
            .join(".agentflow/panel/context-packs/iss-context.json")
            .is_file());
        let events = agentflow_event_store::load_task_events(dir.path()).unwrap();
        assert!(events
            .iter()
            .any(|event| event.event_type == EVENT_TYPE_SPEC_ISSUE_READY));
        assert!(events
            .iter()
            .any(|event| event.event_type == EVENT_TYPE_PANEL_CONTEXT_PACK_READY));
    }

    #[test]
    fn pending_agent_launch_count_ignores_already_claimed_runs() {
        use agentflow_event_store::{append_task_event_once, EventActor, TaskEventDraft};

        let dir = tempdir().unwrap();
        for run_id in ["run-001", "run-002"] {
            append_task_event_once(
                dir.path(),
                TaskEventDraft {
                    flow_type: WorkflowFlowType::Work,
                    aggregate_type: "issue".to_string(),
                    aggregate_id: format!("AF-{run_id}"),
                    project_id: Some("project-events".to_string()),
                    issue_id: Some(format!("AF-{run_id}")),
                    run_id: Some(run_id.to_string()),
                    event_type: AGENT_LAUNCH_REQUESTED.to_string(),
                    authority_role: Some(WorkflowAgentRole::WorkAgent),
                    actor: EventActor {
                        role: "test".to_string(),
                        kind: "system".to_string(),
                    },
                    state: None,
                    correlation_id: Some(format!("corr-{run_id}")),
                    causation_id: None,
                    payload: json!({ "runId": run_id }),
                    artifact_refs: Vec::new(),
                    idempotency_key: Some(format!("launch:{run_id}")),
                },
            )
            .unwrap();
        }
        append_task_event_once(
            dir.path(),
            TaskEventDraft {
                flow_type: WorkflowFlowType::Work,
                aggregate_type: "issue".to_string(),
                aggregate_id: "AF-run-001".to_string(),
                project_id: Some("project-events".to_string()),
                issue_id: Some("AF-run-001".to_string()),
                run_id: Some("run-001".to_string()),
                event_type: AGENT_SESSION_CREATED.to_string(),
                authority_role: Some(WorkflowAgentRole::WorkAgent),
                actor: EventActor {
                    role: "test".to_string(),
                    kind: "system".to_string(),
                },
                state: None,
                correlation_id: Some("corr-run-001".to_string()),
                causation_id: None,
                payload: json!({ "runId": "run-001" }),
                artifact_refs: Vec::new(),
                idempotency_key: Some("session:run-001".to_string()),
            },
        )
        .unwrap();

        assert_eq!(pending_agent_launch_count(dir.path()).unwrap(), 1);
    }

    fn issue_ready_payload(issue_id: &str, display_status: &str) -> IssueReadyPayload {
        IssueReadyPayload {
            issue_id: issue_id.to_string(),
            issue_path: format!(".agentflow/spec/issues/{issue_id}.json"),
            issue_category: "spec".to_string(),
            required_agent_role: "build-agent".to_string(),
            display_status: display_status.to_string(),
            title: "准备上下文包".to_string(),
            objective: "为任务生成 Panel Context Pack。".to_string(),
            acceptance_criteria: vec!["context pack exists".to_string()],
            context_pack_path: Some(format!(".agentflow/panel/context-packs/{issue_id}.json")),
        }
    }
}
