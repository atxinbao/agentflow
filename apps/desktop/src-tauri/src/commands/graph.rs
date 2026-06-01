#[tauri::command]
pub(crate) fn prepare_project_graph(
    project_root: String,
) -> Result<agentflow_graph::GraphStatusSnapshot, String> {
    agentflow_graph::prepare_project_graph(
        project_root,
        agentflow_graph::GraphPrepareMode::Background,
    )
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn load_project_graph_status(
    project_root: String,
) -> Result<agentflow_graph::GraphStatusSnapshot, String> {
    agentflow_graph::load_project_graph_status(project_root).map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn load_project_graph_manifest(
    project_root: String,
) -> Result<agentflow_graph::GraphManifestSnapshot, String> {
    agentflow_graph::load_project_graph_manifest(project_root).map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn search_project_graph(
    project_root: String,
    query: String,
    limit: Option<usize>,
) -> Result<agentflow_graph::GraphSearchSnapshot, String> {
    agentflow_graph::search_project_graph(project_root, &query, limit)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn build_graph_context_pack(
    project_root: String,
    target_type: String,
    target_id: Option<String>,
    title: String,
    objective: String,
    acceptance_criteria: Option<Vec<String>>,
) -> Result<agentflow_graph::GraphContextPack, String> {
    let acceptance_criteria = acceptance_criteria.unwrap_or_default();
    agentflow_graph::build_context_pack(
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
pub(crate) fn load_graph_context_pack(
    project_root: String,
    target_id: String,
) -> Result<Option<agentflow_graph::GraphContextPack>, String> {
    agentflow_graph::load_context_pack(project_root, &target_id).map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn graph_preflight(
    project_root: String,
    target_type: String,
    target_id: Option<String>,
    title: String,
    objective: String,
    acceptance_criteria: Option<Vec<String>>,
) -> Result<agentflow_graph::GraphPreflightSnapshot, String> {
    let acceptance_criteria = acceptance_criteria.unwrap_or_default();
    agentflow_graph::preflight_graph_for_target(
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
pub(crate) fn analyze_graph_impact(
    project_root: String,
    changed_files: Option<Vec<String>>,
    target_files: Option<Vec<String>>,
    target_symbols: Option<Vec<String>>,
    query: Option<String>,
) -> Result<agentflow_graph::GraphImpactSnapshot, String> {
    agentflow_graph::analyze_graph_impact(
        project_root,
        &changed_files.unwrap_or_default(),
        &target_files.unwrap_or_default(),
        &target_symbols.unwrap_or_default(),
        query.as_deref(),
    )
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn check_graph_git_protection(
    project_root: String,
) -> Result<agentflow_graph::GraphProtectionSnapshot, String> {
    agentflow_graph::check_graph_git_protection(project_root).map_err(|error| error.to_string())
}
