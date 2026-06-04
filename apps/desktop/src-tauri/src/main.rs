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
            commands::panel::prepare_project_panel,
            commands::panel::load_project_panel_status,
            commands::panel::load_project_panel_manifest,
            commands::panel::search_project_panel,
            commands::panel::build_panel_context_pack,
            commands::panel::load_panel_context_pack,
            commands::panel::panel_preflight,
            commands::panel::analyze_panel_impact,
            commands::panel::check_panel_git_protection,
            commands::goal_tree::load_goal_tree_snapshot,
            commands::goal_tree::validate_goal_tree,
            commands::input::prepare_input_workspace,
            commands::input::load_input_status,
            commands::input::load_input_manifest,
            commands::input::load_input_index,
            commands::input::load_input_snapshot,
            commands::input::validate_input,
            commands::agent_manual::prepare_agent_working_manual,
            commands::agent_manual::load_agent_environment_status,
            commands::agent_manual::repair_agent_working_manual,
            commands::agent_manual::validate_agent_working_manual,
            commands::agent_manual::load_agentflow_workspace_ownership,
            commands::agent_manual::take_over_agentflow_workspace,
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
