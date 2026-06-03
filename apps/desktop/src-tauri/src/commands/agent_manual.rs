use agentflow_agent_manual::{
    load_agent_environment_status as load_status_at, model::AgentEnvironmentStatus,
    prepare_agent_working_manual as prepare_at, repair_agent_working_manual as repair_at,
    validate_agent_working_manual as validate_at,
};

#[tauri::command]
pub(crate) fn prepare_agent_working_manual(
    project_root: String,
) -> Result<AgentEnvironmentStatus, String> {
    prepare_at(project_root).map_err(|error| error.to_string())
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
) -> Result<AgentEnvironmentStatus, String> {
    repair_at(project_root).map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn validate_agent_working_manual(
    project_root: String,
) -> Result<AgentEnvironmentStatus, String> {
    validate_at(project_root).map_err(|error| error.to_string())
}
