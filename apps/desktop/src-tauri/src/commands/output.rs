//! Output commands expose read-only delivery/evidence status to Desktop.
//!
//! Human Desktop UI must not use these commands to write output artifacts.

#[tauri::command]
pub(crate) fn load_output_status(
    project_root: String,
) -> Result<agentflow_output::OutputStatusSnapshot, String> {
    agentflow_output::load_output_status(project_root).map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn load_output_manifest(
    project_root: String,
) -> Result<agentflow_output::OutputManifest, String> {
    agentflow_output::load_output_manifest(project_root).map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn load_output_index(
    project_root: String,
) -> Result<agentflow_output::OutputIndex, String> {
    agentflow_output::load_output_index(project_root).map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn load_output_snapshot(
    project_root: String,
) -> Result<agentflow_output::OutputSnapshot, String> {
    agentflow_output::load_output_snapshot(project_root).map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn validate_output(
    project_root: String,
) -> Result<agentflow_output::OutputSnapshot, String> {
    agentflow_output::validate_output(project_root).map_err(|error| error.to_string())
}
