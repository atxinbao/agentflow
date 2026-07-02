//! Runtime API read commands for Desktop Advanced diagnostics.

#[tauri::command]
pub(crate) fn load_api_plane_manifest() -> Result<agentflow_runtime_api::ApiPlaneManifest, String> {
    Ok(agentflow_runtime_api::api_plane_manifest())
}

#[tauri::command]
pub(crate) fn load_product_command_surface(
    project_root: String,
) -> Result<agentflow_runtime_api::ProductCommandSurfaceView, String> {
    agentflow_runtime_api::list_product_command_surface(project_root)
        .map_err(|error| format!("load product command surface failed: {error}"))
}

#[tauri::command]
pub(crate) fn dry_run_product_command(
    project_root: String,
    pack_id: String,
    command: String,
    target_object_id: Option<String>,
) -> Result<agentflow_runtime_api::PackCommandDryRunReport, String> {
    agentflow_runtime_api::dry_run_product_command(
        project_root,
        &pack_id,
        &command,
        target_object_id.as_deref(),
    )
    .map_err(|error| format!("dry run product command failed: {error}"))
}

#[tauri::command]
pub(crate) fn submit_product_command(
    project_root: String,
    request: agentflow_runtime_api::ProductCommandSubmitRequest,
) -> Result<agentflow_runtime_api::ProductCommandSubmitResponse, String> {
    agentflow_runtime_api::submit_product_command(project_root, request)
        .map_err(|error| format!("submit product command failed: {error}"))
}

#[cfg(test)]
mod tests {
    use super::load_api_plane_manifest;

    #[test]
    fn api_plane_manifest_command_exposes_required_planes() {
        let manifest = load_api_plane_manifest().expect("load api plane manifest");
        let categories = manifest
            .categories
            .iter()
            .map(|category| category.category.as_str())
            .collect::<Vec<_>>();

        assert!(categories.contains(&"runtime_commands"));
        assert!(categories.contains(&"projection_queries"));
        assert!(categories.contains(&"command_surface_actions"));
        assert!(categories.contains(&"connector_actions"));
        assert!(categories.contains(&"provider_actions"));
        assert!(categories.contains(&"audit_actions"));
        assert!(categories.contains(&"release_actions"));
    }
}
