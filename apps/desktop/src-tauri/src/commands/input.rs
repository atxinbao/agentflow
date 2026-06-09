use tauri::AppHandle;

#[tauri::command]
pub(crate) fn prepare_input_workspace(
    project_root: String,
    app: AppHandle,
) -> Result<agentflow_input::model::InputSnapshot, String> {
    let snapshot = agentflow_input::prepare_input_workspace(&project_root)
        .map_err(|error| error.to_string())?;
    let _ = crate::commands::workflow_events::dispatch_workflow_events_for_app(&project_root, &app);
    Ok(snapshot)
}

#[tauri::command]
pub(crate) fn load_input_status(
    project_root: String,
) -> Result<agentflow_input::model::InputStatusSnapshot, String> {
    agentflow_input::load_input_status(project_root).map_err(|error| error.to_string())
}

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
