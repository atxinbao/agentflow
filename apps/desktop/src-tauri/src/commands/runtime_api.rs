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

#[tauri::command]
pub(crate) fn create_product_workspace(
    product_source_root: String,
    request: agentflow_runtime_api::ProductWorkspaceCreationRequest,
) -> Result<agentflow_runtime_api::ProductWorkspaceCreationReceipt, String> {
    Ok(agentflow_runtime_api::create_product_workspace(
        product_source_root,
        request,
    ))
}

#[tauri::command]
pub(crate) fn load_product_workspace_projection(
    workspace_root: String,
) -> Result<agentflow_runtime_api::ProductWorkspaceProjection, String> {
    Ok(agentflow_runtime_api::load_product_workspace_projection(
        workspace_root,
    ))
}

#[tauri::command]
pub(crate) fn preview_product_intent(
    product_source_root: String,
    workspace_root: String,
    request: agentflow_runtime_api::ProductIntentIntakeRequest,
) -> Result<agentflow_runtime_api::ProductIntentIntakeReceipt, String> {
    agentflow_runtime_api::preview_product_intent(product_source_root, workspace_root, request)
        .map_err(|error| format!("preview product intent failed: {error}"))
}

#[tauri::command]
pub(crate) fn confirm_product_spec_preview(
    workspace_root: String,
    request: agentflow_runtime_api::ProductSpecConfirmationRequest,
) -> Result<agentflow_runtime_api::ProductSpecConfirmationRecord, String> {
    agentflow_runtime_api::confirm_product_spec_preview(workspace_root, request)
        .map_err(|error| format!("confirm product spec preview failed: {error}"))
}

#[tauri::command]
pub(crate) fn materialize_confirmed_product_spec(
    workspace_root: String,
    preview_id: String,
) -> Result<agentflow_runtime_api::ProductSpecMaterializationReport, String> {
    agentflow_runtime_api::materialize_confirmed_product_spec(workspace_root, &preview_id)
        .map_err(|error| format!("materialize confirmed product spec failed: {error}"))
}

#[cfg(test)]
mod tests {
    use super::{
        create_product_workspace, load_api_plane_manifest, load_product_workspace_projection,
        preview_product_intent,
    };
    use agentflow_runtime_api::{
        ProductIntentIntakeRequest, ProductWorkspaceCreationMode, ProductWorkspaceCreationRequest,
        ProductWorkspaceStatus,
    };
    use std::path::{Path, PathBuf};

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

    #[test]
    fn product_workspace_bridge_returns_receipt_and_projection() {
        let repo = workspace_root();
        let dir = tempfile::tempdir().expect("tempdir");
        let workspace = dir.path().join("workspace");
        let receipt = create_product_workspace(
            repo.to_string_lossy().to_string(),
            ProductWorkspaceCreationRequest {
                project_name: "Desktop Bridge Workspace".to_string(),
                workspace_root: workspace.to_string_lossy().to_string(),
                selected_product_id: "software-dev".to_string(),
                initial_goal: "Use Desktop to create a Product workspace.".to_string(),
                creation_mode: ProductWorkspaceCreationMode::Create,
            },
        )
        .expect("create product workspace");

        assert_eq!(receipt.status, ProductWorkspaceStatus::Created);
        assert_eq!(receipt.workspace_root_ref, "workspace://root");
        assert!(receipt
            .portable_paths
            .workspace_manifest
            .starts_with("workspace://"));

        let projection = load_product_workspace_projection(receipt.workspace_root)
            .expect("load product workspace projection");
        assert_eq!(projection.status, ProductWorkspaceStatus::Ready);
        assert!(projection
            .portable_paths
            .goal_doc
            .starts_with("workspace://"));
    }

    #[test]
    fn product_intake_bridge_creates_preview_only_receipt() {
        let repo = workspace_root();
        let dir = tempfile::tempdir().expect("tempdir");
        let receipt = preview_product_intent(
            repo.to_string_lossy().to_string(),
            dir.path().to_string_lossy().to_string(),
            ProductIntentIntakeRequest {
                raw_text: "把用户需求整理成目标、路线图和任务预览。".to_string(),
                selected_product_id: "software-dev".to_string(),
                workspace_id: "desktop-intake".to_string(),
                source_surface: "desktop-project-home".to_string(),
                locale: "zh-CN".to_string(),
                attachment_refs: Vec::new(),
                source_refs: Vec::new(),
            },
        )
        .expect("preview product intent");

        assert!(!receipt.writes_authority);
        assert!(Path::new(dir.path())
            .join(&receipt.preview_artifact_ref)
            .is_file());
    }

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .and_then(Path::parent)
            .expect("workspace root")
            .to_path_buf()
    }
}
