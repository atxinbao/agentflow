//! Runtime API read commands for Desktop Advanced diagnostics.

#[tauri::command]
pub(crate) fn load_api_plane_manifest() -> Result<agentflow_runtime_api::ApiPlaneManifest, String> {
    Ok(agentflow_runtime_api::api_plane_manifest())
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
