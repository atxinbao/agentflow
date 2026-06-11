use agentflow_input::issue::DisplayStatus;
use agentflow_panel::PanelStatus;
use agentflow_workflow_events::{
    append_dead_letter, append_event_once, load_pending_events, mark_event_consumed,
    prepare_events_workspace, ContextPackFailedPayload, ContextPackReadyPayload,
    ContextPackRequestedPayload, IssueReadyPayload, WorkflowEventDraft, CONSUMER_PANEL,
    EVENT_TYPE_INPUT_ISSUE_READY, EVENT_TYPE_PANEL_CONTEXT_PACK_FAILED,
    EVENT_TYPE_PANEL_CONTEXT_PACK_READY, EVENT_TYPE_PANEL_CONTEXT_PACK_REQUESTED,
};
use serde::Serialize;
use std::path::Path;
use tauri::{AppHandle, Emitter};

const WORKFLOW_EVENT_DISPATCH_VERSION: &str = "workflow-event-dispatch.v1";
pub(crate) const AGENTFLOW_WORKFLOW_EVENTS_DISPATCHED_EVENT: &str =
    "agentflow-workflow-events-dispatched";

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WorkflowEventDispatchSummary {
    version: String,
    emitted_issue_ready_events: usize,
    pending_panel_events: usize,
    context_pack_requests: usize,
    context_pack_ready: usize,
    context_pack_failed: usize,
    errors: Vec<String>,
}

impl WorkflowEventDispatchSummary {
    fn should_refresh_ui(&self) -> bool {
        self.context_pack_ready > 0 || self.context_pack_failed > 0
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct WorkflowEventsDispatchedEvent {
    version: String,
    project_root: String,
    pending_panel_events: usize,
    context_pack_requests: usize,
    context_pack_ready: usize,
    context_pack_failed: usize,
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
    prepare_events_workspace(&root).map_err(|error| error.to_string())?;

    let mut summary = WorkflowEventDispatchSummary {
        version: WORKFLOW_EVENT_DISPATCH_VERSION.to_string(),
        ..WorkflowEventDispatchSummary::default()
    };

    let issue_status_index = agentflow_state::load_issue_status_index(&root).ok();
    let pending = load_pending_events(&root, CONSUMER_PANEL, &[EVENT_TYPE_INPUT_ISSUE_READY])
        .map_err(|error| error.to_string())?;
    summary.pending_panel_events = pending.len();
    if !panel_ready_for_context_pack(&root) {
        return Ok(summary);
    }

    for event in pending {
        let payload = match serde_json::from_value::<IssueReadyPayload>(event.payload.clone()) {
            Ok(payload) => payload,
            Err(error) => {
                let message = format!("parse issue ready payload: {error}");
                let _ = append_dead_letter(&root, CONSUMER_PANEL, &event, message.clone());
                let _ = mark_event_consumed(&root, CONSUMER_PANEL, &event.event_id);
                summary.errors.push(message);
                summary.context_pack_failed += 1;
                continue;
            }
        };
        if !event_ready_for_panel(&payload, issue_status_index.as_ref()) {
            mark_event_consumed(&root, CONSUMER_PANEL, &event.event_id)
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
                mark_event_consumed(&root, CONSUMER_PANEL, &event.event_id)
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
                let _ = append_dead_letter(&root, CONSUMER_PANEL, &event, message.clone());
                mark_event_consumed(&root, CONSUMER_PANEL, &event.event_id)
                    .map_err(|error| error.to_string())?;
                summary.errors.push(message);
                summary.context_pack_failed += 1;
            }
        }
    }

    let _ = prepare_events_workspace(&root);
    if summary.should_refresh_ui() {
        if let Some(app) = app {
            let payload = WorkflowEventsDispatchedEvent {
                version: WORKFLOW_EVENT_DISPATCH_VERSION.to_string(),
                project_root: root.display().to_string(),
                pending_panel_events: summary.pending_panel_events,
                context_pack_requests: summary.context_pack_requests,
                context_pack_ready: summary.context_pack_ready,
                context_pack_failed: summary.context_pack_failed,
                errors: summary.errors.clone(),
            };
            let _ = app.emit(AGENTFLOW_WORKFLOW_EVENTS_DISPATCHED_EVENT, payload);
        }
    }
    Ok(summary)
}

fn panel_ready_for_context_pack(root: &Path) -> bool {
    agentflow_panel::load_project_panel_status(root)
        .map(|status| matches!(status.status, PanelStatus::Ready | PanelStatus::Degraded))
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
                entry.display_status,
                DisplayStatus::Ready | DisplayStatus::Blocked
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
    append_event_once(
        root,
        WorkflowEventDraft {
            event_type: event_type.to_string(),
            source: source.to_string(),
            subject_id: issue_id.to_string(),
            subject_path,
            dedupe_key,
            payload: serde_json::to_value(payload)?,
        },
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use agentflow_input::issue::{
        AgentRole, InputIssue, InputIssueKind, InputIssueStatus, InputPriority, InputRiskLevel,
        InputSystemRecord, IssueCategory,
    };
    use agentflow_panel::PanelPrepareMode;
    use agentflow_state::{IssueStatusIndex, IssueStatusIndexEntry, WorkflowAuditStatus};
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn issue_ready_filter_uses_state_display_status() {
        let mut issue = InputIssue {
            issue_id: "iss-done".to_string(),
            issue_category: IssueCategory::Spec,
            required_agent_role: AgentRole::BuildAgent,
            status: InputIssueStatus::ReadyForExecute,
            display_status: DisplayStatus::Ready,
            context_pack_path: ".agentflow/panel/context-packs/iss-done.json".to_string(),
            ..InputIssue::default()
        };
        issue.normalize_execution_metadata();
        let payload = IssueReadyPayload {
            issue_id: issue.issue_id.clone(),
            issue_path: issue.issue_path.clone(),
            issue_category: issue.issue_category.as_str().to_string(),
            required_agent_role: issue.required_agent_role.as_str().to_string(),
            display_status: issue.display_status.as_str().to_string(),
            title: issue.title.clone(),
            objective: issue.summary.clone(),
            acceptance_criteria: issue.acceptance_criteria.clone(),
            context_pack_path: Some(issue.context_pack_path.clone()),
        };
        assert!(event_ready_for_panel(&payload, None));

        let issue_status_index = IssueStatusIndex {
            version: "state-issue-status-index.v1".to_string(),
            updated_at: 1,
            issues: vec![IssueStatusIndexEntry {
                issue_id: issue.issue_id.clone(),
                display_status: DisplayStatus::Done,
                priority: "p2".to_string(),
                execution_risk: "low".to_string(),
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
                issue_id: issue.issue_id.clone(),
                display_status: DisplayStatus::Blocked,
                priority: "p2".to_string(),
                execution_risk: "low".to_string(),
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
    fn dispatch_generates_context_pack_for_ready_spec_issue() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("apps/desktop/src")).unwrap();
        fs::write(
            dir.path().join("apps/desktop/src/App.tsx"),
            "export function App() { return null; }\n",
        )
        .unwrap();

        agentflow_input::prepare_input_workspace(dir.path()).unwrap();
        let mut issue = InputIssue {
            issue_id: "iss-context".to_string(),
            issue_category: IssueCategory::Spec,
            required_agent_role: AgentRole::BuildAgent,
            source_spec_id: "spec-context".to_string(),
            title: "准备上下文包".to_string(),
            summary: "为任务生成 Panel Context Pack。".to_string(),
            kind: InputIssueKind::Validation,
            priority: InputPriority::P1,
            status: InputIssueStatus::ReadyForExecute,
            display_status: DisplayStatus::Ready,
            execution_risk: InputRiskLevel::Medium,
            scope: vec!["apps/desktop/src/**".to_string()],
            acceptance_criteria: vec!["context pack exists".to_string()],
            validation_hints: vec!["npm --prefix apps/desktop run build".to_string()],
            system: InputSystemRecord {
                created_by: "test".to_string(),
                created_at: 1,
                updated_at: 1,
                path: ".agentflow/input/issues/iss-context.json".to_string(),
                revision: 1,
            },
            ..InputIssue::default()
        };
        issue.normalize_execution_metadata();
        fs::write(
            dir.path().join(".agentflow/input/issues/iss-context.json"),
            serde_json::to_string_pretty(&issue).unwrap(),
        )
        .unwrap();
        agentflow_input::prepare_input_workspace(dir.path()).unwrap();
        agentflow_panel::prepare_project_panel(dir.path(), PanelPrepareMode::Blocking).unwrap();

        let summary = dispatch_workflow_events_inner(dir.path(), None).unwrap();

        assert_eq!(summary.context_pack_ready, 1);
        assert!(dir
            .path()
            .join(".agentflow/panel/context-packs/iss-context.json")
            .is_file());
        let events = agentflow_workflow_events::load_events(dir.path()).unwrap();
        assert!(events
            .iter()
            .any(|event| event.event_type == EVENT_TYPE_INPUT_ISSUE_READY));
        assert!(events
            .iter()
            .any(|event| event.event_type == EVENT_TYPE_PANEL_CONTEXT_PACK_READY));
    }
}
