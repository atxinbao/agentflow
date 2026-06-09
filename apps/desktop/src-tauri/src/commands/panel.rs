use tauri::AppHandle;

#[tauri::command]
pub(crate) fn prepare_project_panel(
    project_root: String,
    app: AppHandle,
) -> Result<agentflow_panel::PanelStatusSnapshot, String> {
    let status = agentflow_panel::prepare_project_panel(
        &project_root,
        agentflow_panel::PanelPrepareMode::Background,
    )
    .map_err(|error| error.to_string())?;
    let _ = crate::commands::workflow_events::dispatch_workflow_events_for_app(&project_root, &app);
    Ok(status)
}

#[tauri::command]
pub(crate) fn load_project_panel_status(
    project_root: String,
) -> Result<agentflow_panel::PanelStatusSnapshot, String> {
    agentflow_panel::load_project_panel_status(project_root).map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn load_project_panel_manifest(
    project_root: String,
) -> Result<agentflow_panel::PanelManifestSnapshot, String> {
    agentflow_panel::load_project_panel_manifest(project_root).map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn search_project_panel(
    project_root: String,
    query: String,
    limit: Option<usize>,
) -> Result<agentflow_panel::PanelSearchSnapshot, String> {
    agentflow_panel::search_project_panel(project_root, &query, limit)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn build_panel_context_pack(
    project_root: String,
    target_type: String,
    target_id: Option<String>,
    title: String,
    objective: String,
    acceptance_criteria: Option<Vec<String>>,
) -> Result<agentflow_panel::PanelContextPack, String> {
    let acceptance_criteria = acceptance_criteria.unwrap_or_default();
    agentflow_panel::build_panel_context_pack(
        project_root,
        &target_type,
        target_id.as_deref(),
        &title,
        &objective,
        &acceptance_criteria,
    )
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn load_panel_context_pack(
    project_root: String,
    target_id: String,
) -> Result<Option<agentflow_panel::PanelContextPack>, String> {
    agentflow_panel::load_panel_context_pack(project_root, &target_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn panel_preflight(
    project_root: String,
    target_type: String,
    target_id: Option<String>,
    title: String,
    objective: String,
    acceptance_criteria: Option<Vec<String>>,
) -> Result<agentflow_panel::PanelPreflightSnapshot, String> {
    let acceptance_criteria = acceptance_criteria.unwrap_or_default();
    agentflow_panel::panel_preflight(
        project_root,
        &target_type,
        target_id.as_deref(),
        &title,
        &objective,
        &acceptance_criteria,
    )
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn analyze_panel_impact(
    project_root: String,
    changed_files: Option<Vec<String>>,
    target_files: Option<Vec<String>>,
    target_symbols: Option<Vec<String>>,
    query: Option<String>,
) -> Result<agentflow_panel::PanelImpactSnapshot, String> {
    agentflow_panel::analyze_panel_impact(
        project_root,
        &changed_files.unwrap_or_default(),
        &target_files.unwrap_or_default(),
        &target_symbols.unwrap_or_default(),
        query.as_deref(),
    )
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn check_panel_git_protection(
    project_root: String,
) -> Result<agentflow_panel::PanelProtectionSnapshot, String> {
    agentflow_panel::check_panel_git_protection(project_root).map_err(|error| error.to_string())
}
