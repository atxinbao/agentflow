use agentflow_agent_manual::{
    check_agentflow_workspace_ownership as check_ownership_at,
    load_agent_environment_status as load_status_at,
    model::{AgentEnvironmentStatus, WorkspaceOwnershipStatus},
    prepare_agent_working_manual_with_locale as prepare_at,
    repair_agent_working_manual_with_locale as repair_at,
    take_over_agentflow_workspace as take_over_at, validate_agent_working_manual as validate_at,
};

#[tauri::command]
pub(crate) fn prepare_agent_working_manual(
    project_root: String,
    app_locale: Option<String>,
) -> Result<AgentEnvironmentStatus, String> {
    prepare_at(project_root, app_locale).map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn load_agent_environment_status(
    project_root: String,
) -> Result<AgentEnvironmentStatus, String> {
    load_status_at(project_root).map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn repair_agent_working_manual(
    project_root: String,
    app_locale: Option<String>,
) -> Result<AgentEnvironmentStatus, String> {
    repair_at(project_root, app_locale).map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn validate_agent_working_manual(
    project_root: String,
) -> Result<AgentEnvironmentStatus, String> {
    validate_at(project_root).map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn load_agentflow_workspace_ownership(
    project_root: String,
) -> Result<WorkspaceOwnershipStatus, String> {
    check_ownership_at(project_root).map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn take_over_agentflow_workspace(
    project_root: String,
) -> Result<WorkspaceOwnershipStatus, String> {
    take_over_at(project_root).map_err(|error| error.to_string())
}
