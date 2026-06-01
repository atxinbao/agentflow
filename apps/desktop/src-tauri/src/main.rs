mod graph;
mod project_files;
mod project_workspace;

#[tauri::command]
fn load_workbench_snapshot() -> Result<agentflow_core::DesktopWorkbenchSnapshot, String> {
    let cwd = std::env::current_dir().map_err(|error| error.to_string())?;
    agentflow_core::read_desktop_workbench_snapshot(&cwd).map_err(|error| error.to_string())
}

#[tauri::command]
fn load_metrics_snapshot() -> Result<agentflow_core::LocalMetricsSnapshot, String> {
    let cwd = std::env::current_dir().map_err(|error| error.to_string())?;
    agentflow_core::read_local_metrics_snapshot(&cwd).map_err(|error| error.to_string())
}

#[tauri::command]
fn load_project_model_snapshot() -> Result<agentflow_core::LocalProjectModelSnapshot, String> {
    let cwd = std::env::current_dir().map_err(|error| error.to_string())?;
    agentflow_core::read_local_project_model_snapshot(&cwd).map_err(|error| error.to_string())
}

#[tauri::command]
fn load_project_milestone_issue_view_model_snapshot(
) -> Result<agentflow_core::ProjectMilestoneIssueViewModelSnapshot, String> {
    let cwd = std::env::current_dir().map_err(|error| error.to_string())?;
    agentflow_core::read_project_milestone_issue_view_model_snapshot(&cwd)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn load_search_snapshot(query: String) -> Result<agentflow_core::LocalSearchSnapshot, String> {
    let cwd = std::env::current_dir().map_err(|error| error.to_string())?;
    agentflow_core::read_local_search_snapshot(&cwd, &query).map_err(|error| error.to_string())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            load_workbench_snapshot,
            load_metrics_snapshot,
            load_project_model_snapshot,
            load_project_milestone_issue_view_model_snapshot,
            load_search_snapshot,
            graph::prepare_project_graph,
            graph::load_project_graph_status,
            graph::load_project_graph_manifest,
            graph::search_project_graph,
            graph::build_graph_context_pack,
            graph::load_graph_context_pack,
            graph::graph_preflight,
            graph::analyze_graph_impact,
            graph::check_graph_git_protection,
            project_files::load_project_files_snapshot,
            project_files::load_project_file_content,
            project_files::choose_existing_project_folder,
            project_workspace::prepare_local_project_workspace
        ])
        .run(tauri::generate_context!())
        .expect("run AgentFlow desktop workbench");
}
