#[tauri::command]
pub(crate) fn load_input_manifest(
    project_root: String,
) -> Result<agentflow_input::model::InputManifest, String> {
    agentflow_input::load_input_manifest(project_root).map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn load_input_index(
    project_root: String,
) -> Result<agentflow_input::model::InputIndex, String> {
    agentflow_input::load_input_index(project_root).map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn load_input_snapshot(
    project_root: String,
) -> Result<agentflow_input::model::InputSnapshot, String> {
    agentflow_input::load_input_snapshot(project_root).map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn validate_input(
    project_root: String,
) -> Result<agentflow_input::model::InputSnapshot, String> {
    agentflow_input::validate_input_workspace(project_root).map_err(|error| error.to_string())
}
