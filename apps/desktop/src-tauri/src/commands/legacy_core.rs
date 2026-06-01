//! Transitional legacy read-model commands.
//!
//! These commands wrap agentflow-core legacy/transitional snapshots so the
//! current Desktop can keep rendering while the new workflow is being defined.
//!
//! Do not add new write flows here.

#[tauri::command]
pub(crate) fn load_workbench_snapshot() -> Result<agentflow_core::DesktopWorkbenchSnapshot, String>
{
    let cwd = std::env::current_dir().map_err(|error| error.to_string())?;
    agentflow_core::read_desktop_workbench_snapshot(&cwd).map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn load_metrics_snapshot() -> Result<agentflow_core::LocalMetricsSnapshot, String> {
    let cwd = std::env::current_dir().map_err(|error| error.to_string())?;
    agentflow_core::read_local_metrics_snapshot(&cwd).map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn load_project_model_snapshot(
) -> Result<agentflow_core::LocalProjectModelSnapshot, String> {
    let cwd = std::env::current_dir().map_err(|error| error.to_string())?;
    agentflow_core::read_local_project_model_snapshot(&cwd).map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn load_project_milestone_issue_view_model_snapshot(
) -> Result<agentflow_core::ProjectMilestoneIssueViewModelSnapshot, String> {
    let cwd = std::env::current_dir().map_err(|error| error.to_string())?;
    agentflow_core::read_project_milestone_issue_view_model_snapshot(&cwd)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn load_search_snapshot(
    query: String,
) -> Result<agentflow_core::LocalSearchSnapshot, String> {
    let cwd = std::env::current_dir().map_err(|error| error.to_string())?;
    agentflow_core::read_local_search_snapshot(&cwd, &query).map_err(|error| error.to_string())
}
