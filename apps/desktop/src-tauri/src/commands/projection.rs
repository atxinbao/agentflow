//! Task and project projection commands for Desktop.
//!
//! These commands expose event-driven read models only. They rebuild and read
//! `.agentflow/projections/**` plus `.agentflow/indexes/**`; they do not execute
//! issues, create delivery artifacts, or mutate user source files.

#[tauri::command]
pub(crate) fn rebuild_task_projections(
    project_root: String,
) -> Result<agentflow_projection::ProjectionSummary, String> {
    agentflow_projection::rebuild_projections(project_root).map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn load_projection_issue_status_index(
    project_root: String,
) -> Result<agentflow_projection::IssueStatusIndex, String> {
    agentflow_projection::load_issue_status_index(project_root).map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn load_task_projection(
    project_root: String,
    issue_id: String,
) -> Result<agentflow_projection::TaskProjection, String> {
    agentflow_projection::load_task_projection(project_root, &issue_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn load_project_projection(
    project_root: String,
    project_id: String,
) -> Result<agentflow_projection::ProjectProjection, String> {
    agentflow_projection::load_project_projection(project_root, &project_id)
        .map_err(|error| error.to_string())
}
