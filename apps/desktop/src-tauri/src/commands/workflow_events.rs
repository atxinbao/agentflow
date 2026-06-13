use agentflow_execute::{claim_build_agent_launch, mark_build_agent_launch_failed};
use agentflow_input::issue::DisplayStatus;
use agentflow_mcp::{
    default_provider_bridge, write_provider_status, write_registry_for_statuses, McpLaunchRequest,
};
use agentflow_panel::PanelStatus;
use agentflow_workflow_events::{
    append_dead_letter, append_event_once, load_pending_events, mark_event_consumed,
    prepare_events_workspace, BuildAgentLaunchClaimedPayload, BuildAgentLaunchRequestedPayload,
    ContextPackFailedPayload, ContextPackReadyPayload, ContextPackRequestedPayload,
    IssueReadyPayload, WorkflowEventDraft, CONSUMER_BUILD_AGENT, CONSUMER_PANEL,
    CONSUMER_PROVIDER_BRIDGE, EVENT_TYPE_BUILD_AGENT_LAUNCH_CLAIMED,
    EVENT_TYPE_BUILD_AGENT_LAUNCH_REQUESTED, EVENT_TYPE_INPUT_ISSUE_READY,
    EVENT_TYPE_PANEL_CONTEXT_PACK_FAILED, EVENT_TYPE_PANEL_CONTEXT_PACK_READY,
    EVENT_TYPE_PANEL_CONTEXT_PACK_REQUESTED,
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
    prepare_events_workspace(&root).map_err(|error| error.to_string())?;

    let mut summary = WorkflowEventDispatchSummary {
        version: WORKFLOW_EVENT_DISPATCH_VERSION.to_string(),
        ..WorkflowEventDispatchSummary::default()
    };

    let issue_status_index = agentflow_state::load_issue_status_index(&root).ok();
    let pending = load_pending_events(&root, CONSUMER_PANEL, &[EVENT_TYPE_INPUT_ISSUE_READY])
        .map_err(|error| error.to_string())?;
    summary.pending_panel_events = pending.len();
    if panel_ready_for_context_pack(&root) {
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
    }

    if let Err(error) = dispatch_build_agent_launch_events(&root, &mut summary) {
        summary.errors.push(error.to_string());
    }

    let _ = prepare_events_workspace(&root);
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

fn provider_ready_for_build_agent_launch(status: &agentflow_mcp::McpProviderStatus) -> bool {
    status.capability_available("launch")
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
                DisplayStatus::Backlog | DisplayStatus::Todo | DisplayStatus::Blocked
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

fn dispatch_build_agent_launch_events(
    root: &Path,
    summary: &mut WorkflowEventDispatchSummary,
) -> anyhow::Result<()> {
    let provider_bridge = default_provider_bridge();
    let provider = provider_bridge
        .provider("codex")
        .ok_or_else(|| anyhow::anyhow!("codex provider is not registered"))?;

    let provider_status = provider.check_health(root);
    write_provider_status(root, &provider_status)?;
    write_registry_for_statuses(root, &[provider_status.clone()])?;

    let pending = load_pending_events(
        root,
        CONSUMER_PROVIDER_BRIDGE,
        &[EVENT_TYPE_BUILD_AGENT_LAUNCH_REQUESTED],
    )?;
    summary.pending_build_agent_launch_events = pending.len();

    if !provider_ready_for_build_agent_launch(&provider_status) {
        if !pending.is_empty() {
            let launch_reason = provider_status
                .capability("launch")
                .and_then(|capability| capability.detail.clone())
                .unwrap_or_else(|| provider_status.status.as_str().to_string());
            summary.errors.push(format!(
                "build-agent launch pending: {} launch capability is unavailable ({})",
                provider.provider_id(),
                launch_reason
            ));
        }
        return Ok(());
    }

    for event in pending {
        let payload =
            match serde_json::from_value::<BuildAgentLaunchRequestedPayload>(event.payload.clone())
            {
                Ok(payload) => payload,
                Err(error) => {
                    let message = format!("parse build-agent launch payload: {error}");
                    let _ =
                        append_dead_letter(root, CONSUMER_PROVIDER_BRIDGE, &event, message.clone());
                    let _ = mark_event_consumed(root, CONSUMER_PROVIDER_BRIDGE, &event.event_id);
                    summary.errors.push(message);
                    continue;
                }
            };

        let request = match build_provider_launch_request(root, &payload) {
            Ok(request) => request,
            Err(error) => {
                let message = format!("build codex launch request {}: {error}", payload.issue_id);
                let _ = append_dead_letter(root, CONSUMER_PROVIDER_BRIDGE, &event, message.clone());
                let _ = mark_event_consumed(root, CONSUMER_PROVIDER_BRIDGE, &event.event_id);
                summary.errors.push(message);
                continue;
            }
        };

        match provider.create_session(root, &request) {
            Ok(session) => {
                if let Err(error) = claim_build_agent_launch(
                    root,
                    &payload.issue_id,
                    Some(&payload.project_id),
                    &payload.run_id,
                    payload.branch_name.clone(),
                    payload.launch_request_path.clone(),
                    event.event_id.clone(),
                ) {
                    summary.errors.push(format!(
                        "claim build-agent launch {}: {error}",
                        payload.issue_id
                    ));
                    continue;
                }
                if let Err(error) = append_event_once(
                    root,
                    WorkflowEventDraft {
                        event_type: EVENT_TYPE_BUILD_AGENT_LAUNCH_CLAIMED.to_string(),
                        source: "provider-bridge".to_string(),
                        subject_id: payload.issue_id.clone(),
                        subject_path: Some(payload.launch_request_path.clone()),
                        dedupe_key: format!(
                            "build-agent.launch.claimed:{}:{}",
                            payload.issue_id, payload.run_id
                        ),
                        payload: serde_json::to_value(BuildAgentLaunchClaimedPayload {
                            issue_id: payload.issue_id.clone(),
                            project_id: Some(payload.project_id.clone()),
                            run_id: payload.run_id.clone(),
                            session_id: session.session_id.clone(),
                            provider: session.provider.clone(),
                            branch_name: payload.branch_name.clone(),
                            launch_request_path: payload.launch_request_path.clone(),
                            log_path: session.log_path.clone(),
                        })?,
                    },
                ) {
                    summary.errors.push(format!(
                        "append claimed event {}: {error}",
                        payload.issue_id
                    ));
                    continue;
                }
                mark_event_consumed(root, CONSUMER_PROVIDER_BRIDGE, &event.event_id)?;
                mark_event_consumed(root, CONSUMER_BUILD_AGENT, &event.event_id)?;
                summary.build_agent_launch_sessions_created += 1;
            }
            Err(error) => {
                let _ = mark_build_agent_launch_failed(root, &payload.run_id);
                let message = format!("create provider session {}: {error}", payload.issue_id);
                let _ = append_dead_letter(root, CONSUMER_PROVIDER_BRIDGE, &event, message.clone());
                let _ = mark_event_consumed(root, CONSUMER_PROVIDER_BRIDGE, &event.event_id);
                summary.errors.push(message);
            }
        }
    }

    Ok(())
}

fn build_provider_launch_request(
    root: &Path,
    payload: &BuildAgentLaunchRequestedPayload,
) -> anyhow::Result<McpLaunchRequest> {
    let mut request = McpLaunchRequest::new(
        "codex",
        payload.issue_id.clone(),
        payload.run_id.clone(),
        "build-agent",
        root.display().to_string(),
        payload.launch_request_path.clone(),
    );
    request.project_id = Some(payload.project_id.clone());
    request.prompt_path = Some(payload.launch_request_path.clone());
    request.context_pack_path = Some(payload.context_pack_path.clone());
    request.branch_name = payload.branch_name.clone();
    if let Ok(issue) = agentflow_input::load_input_issue(root, &payload.issue_id) {
        request.agent_role = issue.required_agent_role.as_str().to_string();
        request.merge_mode = issue
            .execution_pipeline
            .as_ref()
            .and_then(|pipeline| pipeline.merge_modes.first().cloned());
    }
    Ok(request)
}

#[cfg(test)]
mod tests {
    use super::*;
    use agentflow_input::{
        issue::{
            AgentRole, InputIssue, InputIssueKind, InputIssueModel, InputIssueStatus,
            InputPriority, InputRiskLevel, InputSystemRecord, IssueCategory,
        },
        project::{InputProject, InputProjectStatus},
        spec_gate::{InputIssueGenerationMode, InputSpecApproval},
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
            status: InputIssueStatus::Todo,
            display_status: DisplayStatus::Todo,
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
            status: InputIssueStatus::Todo,
            display_status: DisplayStatus::Todo,
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

    #[test]
    fn dispatch_does_not_run_project_loop_for_backlog_project_issue_after_context_pack_ready() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("apps/desktop/src")).unwrap();
        fs::write(
            dir.path().join("apps/desktop/src/App.tsx"),
            "export function App() { return null; }\n",
        )
        .unwrap();

        agentflow_input::prepare_input_workspace(dir.path()).unwrap();
        write_approved_spec(dir.path());
        write_project_issue(dir.path());
        agentflow_input::prepare_input_workspace(dir.path()).unwrap();
        agentflow_panel::prepare_project_panel(dir.path(), PanelPrepareMode::Blocking).unwrap();

        let summary = dispatch_workflow_events_inner(dir.path(), None).unwrap();

        assert_eq!(summary.context_pack_ready, 1);
        let issue = agentflow_input::load_input_issue(dir.path(), "AF-EVENT-001").unwrap();
        assert_eq!(issue.status, InputIssueStatus::Backlog);
        assert!(dir
            .path()
            .join(".agentflow/panel/context-packs/AF-EVENT-001.json")
            .is_file());
        assert!(!dir
            .path()
            .join(".agentflow/state/loops/projects/proj-event.json")
            .is_file());
        assert!(!dir
            .path()
            .join(".agentflow/state/loops/issues/AF-EVENT-001.json")
            .is_file());
    }

    #[test]
    fn launch_dispatch_uses_launch_capability_instead_of_status_code() {
        let mut status =
            agentflow_mcp::McpProviderStatus::new(agentflow_mcp::McpProviderKind::Codex, 1);
        status.status = agentflow_mcp::McpProviderStatusCode::Unsupported;
        status.capabilities = vec![agentflow_mcp::McpCapability::new("launch", true)];

        assert!(provider_ready_for_build_agent_launch(&status));
    }

    fn write_approved_spec(root: &std::path::Path) {
        let spec_dir = root.join(".agentflow/input/specs/approved/spec-event");
        fs::create_dir_all(&spec_dir).unwrap();
        fs::write(spec_dir.join("product.md"), "# Product\n").unwrap();
        fs::write(spec_dir.join("tech.md"), "# Tech\n").unwrap();
        fs::write(spec_dir.join("spec.json"), "{}\n").unwrap();
        fs::write(
            spec_dir.join("approval.json"),
            serde_json::to_string_pretty(&InputSpecApproval {
                spec_id: "spec-event".to_string(),
                issue_generation_mode: InputIssueGenerationMode::Project,
                ..InputSpecApproval::default()
            })
            .unwrap(),
        )
        .unwrap();
    }

    fn write_project_issue(root: &std::path::Path) {
        let project = InputProject {
            project_id: "proj-event".to_string(),
            source_spec_id: "spec-event".to_string(),
            title: "事件驱动 Project Loop".to_string(),
            summary: "通过事件生成上下文包后推进任务。".to_string(),
            objective: "通过事件生成上下文包后推进任务。".to_string(),
            issue_ids: vec!["AF-EVENT-001".to_string()],
            status: InputProjectStatus::Planned,
            system: InputSystemRecord {
                created_by: "test".to_string(),
                created_at: 1,
                updated_at: 1,
                path: ".agentflow/input/projects/proj-event.json".to_string(),
                revision: 1,
            },
            ..InputProject::default()
        };
        let mut issue = InputIssue {
            issue_id: "AF-EVENT-001".to_string(),
            issue_model: InputIssueModel::Project,
            issue_category: IssueCategory::Spec,
            required_agent_role: AgentRole::BuildAgent,
            source_spec_id: "spec-event".to_string(),
            project_id: Some("proj-event".to_string()),
            title: "事件驱动任务".to_string(),
            summary: "验证 backlog 任务可以由事件链路推进到 todo。".to_string(),
            kind: InputIssueKind::Validation,
            priority: InputPriority::P2,
            status: InputIssueStatus::Backlog,
            display_status: DisplayStatus::Backlog,
            execution_risk: InputRiskLevel::Low,
            scope: vec!["apps/desktop/src/**".to_string()],
            acceptance_criteria: vec!["context pack exists".to_string()],
            validation_hints: vec!["npm --prefix apps/desktop run build".to_string()],
            system: InputSystemRecord {
                created_by: "test".to_string(),
                created_at: 1,
                updated_at: 1,
                path: ".agentflow/input/issues/AF-EVENT-001.json".to_string(),
                revision: 1,
            },
            ..InputIssue::default()
        };
        issue.normalize_execution_metadata();

        fs::write(
            root.join(".agentflow/input/projects/proj-event.json"),
            serde_json::to_string_pretty(&project).unwrap(),
        )
        .unwrap();
        fs::write(
            root.join(".agentflow/input/issues/AF-EVENT-001.json"),
            serde_json::to_string_pretty(&issue).unwrap(),
        )
        .unwrap();
    }
}
