mod commands;
mod project_files;
mod project_workspace;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::legacy_core::load_workbench_snapshot,
            commands::legacy_core::load_metrics_snapshot,
            commands::legacy_core::load_project_model_snapshot,
            commands::legacy_core::load_project_milestone_issue_view_model_snapshot,
            commands::legacy_core::load_search_snapshot,
            commands::graph::prepare_project_graph,
            commands::graph::load_project_graph_status,
            commands::graph::load_project_graph_manifest,
            commands::graph::search_project_graph,
            commands::graph::build_graph_context_pack,
            commands::graph::load_graph_context_pack,
            commands::graph::graph_preflight,
            commands::graph::analyze_graph_impact,
            commands::graph::check_graph_git_protection,
            commands::project_files::load_project_files_snapshot,
            commands::project_files::load_project_file_content,
            commands::project_files::load_project_directory_page,
            commands::project_files::search_project_files,
            commands::project_files::load_project_file_text_range,
            commands::project_files::choose_existing_project_folder,
            commands::project_workspace::prepare_local_project_workspace
        ])
        .run(tauri::generate_context!())
        .expect("run AgentFlow desktop workbench");
}
