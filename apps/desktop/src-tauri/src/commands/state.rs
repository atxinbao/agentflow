//! Workflow state commands expose the derived orchestration layer to Desktop.
//!
//! These commands never execute issues, trigger audits, call models, or write
//! user source files. Mutating commands are limited to `.agentflow/state/**`.

#[tauri::command]
pub(crate) fn prepare_state_workspace(
    project_root: String,
) -> Result<agentflow_state::StateStatusSnapshot, String> {
    agentflow_state::prepare_state_workspace(project_root).map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn refresh_state(
    project_root: String,
) -> Result<agentflow_state::StateStatusSnapshot, String> {
    agentflow_state::refresh_state(project_root).map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn load_state_status(
    project_root: String,
) -> Result<agentflow_state::StateStatusSnapshot, String> {
    agentflow_state::load_state_status(project_root).map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn load_state_manifest(
    project_root: String,
) -> Result<agentflow_state::StateManifest, String> {
    agentflow_state::load_state_manifest(project_root).map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn load_state_index(
    project_root: String,
) -> Result<agentflow_state::StateIndex, String> {
    agentflow_state::load_state_index(project_root).map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn load_issue_status_index(
    project_root: String,
) -> Result<agentflow_state::IssueStatusIndex, String> {
    agentflow_state::load_issue_status_index(project_root).map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn load_workflow_gates(
    project_root: String,
) -> Result<agentflow_state::WorkflowGateSnapshot, String> {
    agentflow_state::load_workflow_gates(project_root).map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn load_next_actions(
    project_root: String,
) -> Result<agentflow_state::WorkflowNextActionsSnapshot, String> {
    agentflow_state::load_next_actions(project_root).map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn load_blockers(
    project_root: String,
) -> Result<agentflow_state::WorkflowBlockersSnapshot, String> {
    agentflow_state::load_blockers(project_root).map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn load_state_timeline(
    project_root: String,
) -> Result<Vec<agentflow_state::StateTimelineEvent>, String> {
    agentflow_state::load_state_timeline(project_root).map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn append_state_event(
    project_root: String,
    draft: agentflow_state::StateTimelineEventDraft,
) -> Result<agentflow_state::StateTimelineEvent, String> {
    agentflow_state::append_state_event(project_root, draft).map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn load_state_session(
    project_root: String,
    session_id: String,
) -> Result<Option<agentflow_state::StateSession>, String> {
    agentflow_state::load_state_session(project_root, session_id).map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn update_state_session(
    project_root: String,
    update: agentflow_state::StateSessionUpdate,
) -> Result<agentflow_state::StateSession, String> {
    agentflow_state::update_state_session(project_root, update).map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn load_state_locks(
    project_root: String,
) -> Result<agentflow_state::StateLockSnapshot, String> {
    agentflow_state::load_state_locks(project_root).map_err(|error| error.to_string())
}
