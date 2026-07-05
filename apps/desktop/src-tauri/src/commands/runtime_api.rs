//! Runtime API read commands for Desktop Advanced diagnostics.

use std::path::{Path, PathBuf};

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DesktopFirstRunOnboardingReceipt {
    pub(crate) version: String,
    pub(crate) invoked_commands: Vec<String>,
    pub(crate) product_source_root: String,
    pub(crate) workspace_receipt: agentflow_runtime_api::ProductWorkspaceCreationReceipt,
    pub(crate) readiness: agentflow_runtime_api::ProductOnboardingReadinessReport,
    pub(crate) guided_sample_run_plan: agentflow_runtime_api::ProductGuidedSampleRunPlan,
    pub(crate) guided_sample_run_receipt: agentflow_runtime_api::ProductGuidedSampleRunReceipt,
}

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
pub(crate) fn load_first_run_onboarding_contract(
    selected_product_id: String,
) -> Result<agentflow_runtime_api::ProductFirstRunOnboardingContract, String> {
    Ok(agentflow_runtime_api::first_run_onboarding_contract(
        selected_product_id,
    ))
}

#[tauri::command]
pub(crate) fn check_product_onboarding_readiness(
    product_source_root: String,
    workspace_root: String,
    selected_product_id: String,
) -> Result<agentflow_runtime_api::ProductOnboardingReadinessReport, String> {
    Ok(agentflow_runtime_api::check_product_onboarding_readiness(
        product_source_root,
        workspace_root,
        selected_product_id,
    ))
}

#[tauri::command]
pub(crate) fn load_guided_sample_run_plan(
    workspace_root: String,
    selected_product_id: String,
) -> Result<agentflow_runtime_api::ProductGuidedSampleRunPlan, String> {
    Ok(agentflow_runtime_api::guided_sample_run_plan(
        workspace_root,
        selected_product_id,
    ))
}

#[tauri::command]
pub(crate) fn run_guided_sample(
    workspace_root: String,
    selected_product_id: String,
    execution_mode: Option<String>,
) -> Result<agentflow_runtime_api::ProductGuidedSampleRunReceipt, String> {
    agentflow_runtime_api::run_guided_sample(
        workspace_root,
        selected_product_id,
        execution_mode.unwrap_or_else(|| "deterministic-dry-run".to_string()),
    )
    .map_err(|error| format!("run guided sample failed: {error}"))
}

#[tauri::command]
pub(crate) fn run_first_run_product_onboarding(
    project_root: String,
    project_name: String,
    selected_product_id: String,
    initial_goal: String,
    guided_sample_execution_mode: Option<String>,
) -> Result<DesktopFirstRunOnboardingReceipt, String> {
    let product_source_root = resolve_product_source_root(&project_root)
        .map_err(|error| format!("resolve product source root failed: {error}"))?;
    let product_source_root_string = product_source_root.to_string_lossy().to_string();
    let workspace_receipt = agentflow_runtime_api::create_product_workspace(
        product_source_root_string.clone(),
        agentflow_runtime_api::ProductWorkspaceCreationRequest {
            project_name,
            workspace_root: project_root.clone(),
            selected_product_id: selected_product_id.clone(),
            initial_goal,
            creation_mode: agentflow_runtime_api::ProductWorkspaceCreationMode::Recover,
        },
    );
    let readiness = agentflow_runtime_api::check_product_onboarding_readiness(
        product_source_root_string.clone(),
        project_root.clone(),
        selected_product_id.clone(),
    );
    let guided_sample_run_plan = agentflow_runtime_api::guided_sample_run_plan(
        project_root.clone(),
        selected_product_id.clone(),
    );
    let guided_sample_run_receipt = agentflow_runtime_api::run_guided_sample(
        project_root,
        selected_product_id,
        guided_sample_execution_mode.unwrap_or_else(|| "deterministic-dry-run".to_string()),
    )
    .map_err(|error| format!("run guided sample failed: {error}"))?;

    Ok(DesktopFirstRunOnboardingReceipt {
        version: "agentflow-desktop-first-run-onboarding-receipt.v1".to_string(),
        invoked_commands: vec![
            "create_product_workspace".to_string(),
            "check_product_onboarding_readiness".to_string(),
            "load_guided_sample_run_plan".to_string(),
            "run_guided_sample".to_string(),
        ],
        product_source_root: product_source_root_string,
        workspace_receipt,
        readiness,
        guided_sample_run_plan,
        guided_sample_run_receipt,
    })
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

#[tauri::command]
pub(crate) fn create_executor_handoff_package(
    project_root: String,
    request: agentflow_runtime_api::ExecutorHandoffRequest,
) -> Result<agentflow_runtime_api::ExecutorHandoffPackage, String> {
    agentflow_runtime_api::create_executor_handoff_package(project_root, request)
        .map_err(|error| format!("create executor handoff package failed: {error}"))
}

#[tauri::command]
pub(crate) fn check_executor_diff_boundary(
    project_root: String,
    request: agentflow_runtime_api::ExecutorDiffBoundaryRequest,
) -> Result<agentflow_runtime_api::ExecutorDiffBoundaryReport, String> {
    agentflow_runtime_api::check_executor_diff_boundary(project_root, request)
        .map_err(|error| format!("check executor diff boundary failed: {error}"))
}

#[tauri::command]
pub(crate) fn capture_executor_evidence(
    project_root: String,
    request: agentflow_runtime_api::ExecutorEvidenceCaptureRequest,
) -> Result<agentflow_runtime_api::ExecutorEvidenceCaptureReport, String> {
    agentflow_runtime_api::capture_executor_evidence(project_root, request)
        .map_err(|error| format!("capture executor evidence failed: {error}"))
}

#[tauri::command]
pub(crate) fn write_executor_result_to_issue(
    project_root: String,
    request: agentflow_runtime_api::ExecutorResultWritebackRequest,
) -> Result<agentflow_runtime_api::ExecutorResultWritebackReport, String> {
    agentflow_runtime_api::write_executor_result_to_issue(project_root, request)
        .map_err(|error| format!("write executor result to issue failed: {error}"))
}

#[tauri::command]
pub(crate) fn record_executor_lifecycle(
    project_root: String,
    request: agentflow_runtime_api::ExecutorLifecycleRequest,
) -> Result<agentflow_runtime_api::ExecutorLifecycleReceipt, String> {
    agentflow_runtime_api::record_executor_lifecycle(project_root, request)
        .map_err(|error| format!("record executor lifecycle failed: {error}"))
}

#[tauri::command]
pub(crate) fn resume_executor_run(
    project_root: String,
    request: agentflow_runtime_api::ExecutorRunResumeRequest,
) -> Result<agentflow_runtime_api::ExecutorRunResumeReceipt, String> {
    agentflow_runtime_api::resume_executor_run(project_root, request)
        .map_err(|error| format!("resume executor run failed: {error}"))
}

#[tauri::command]
pub(crate) fn recover_failed_executor_command(
    project_root: String,
    request: agentflow_runtime_api::ExecutorCommandRecoveryRequest,
) -> Result<agentflow_runtime_api::ExecutorCommandRecoveryReceipt, String> {
    agentflow_runtime_api::recover_failed_executor_command(project_root, request)
        .map_err(|error| format!("recover failed executor command failed: {error}"))
}

#[tauri::command]
pub(crate) fn rebuild_executor_projection(
    project_root: String,
    issue_id: String,
    run_id: String,
) -> Result<agentflow_runtime_api::ExecutorProjectionRebuildReceipt, String> {
    agentflow_runtime_api::rebuild_executor_projection(project_root, &issue_id, &run_id)
        .map_err(|error| format!("rebuild executor projection failed: {error}"))
}

#[tauri::command]
pub(crate) fn check_executor_workspace_health(
    project_root: String,
    issue_id: String,
    run_id: String,
) -> Result<agentflow_runtime_api::ExecutorWorkspaceHealthReport, String> {
    agentflow_runtime_api::check_executor_workspace_health(project_root, &issue_id, &run_id)
        .map_err(|error| format!("check executor workspace health failed: {error}"))
}

#[tauri::command]
pub(crate) fn load_executor_flow_read_model(
    project_root: String,
    issue_id: String,
    run_id: String,
) -> Result<agentflow_runtime_api::ExecutorFlowReadModel, String> {
    agentflow_runtime_api::get_executor_flow_read_model(project_root, &issue_id, &run_id)
        .map_err(|error| format!("load executor flow read model failed: {error}"))
}

fn resolve_product_source_root(workspace_root: &str) -> Result<PathBuf, String> {
    let candidates = [
        std::env::var("AGENTFLOW_PRODUCT_SOURCE_ROOT")
            .ok()
            .map(PathBuf::from),
        std::env::current_dir().ok(),
        repo_root_from_manifest_dir(),
        Some(PathBuf::from(workspace_root)),
    ];

    candidates
        .into_iter()
        .flatten()
        .find(|candidate| has_product_registry(candidate))
        .ok_or_else(|| {
            "no products/** registry found for Product Onboarding Runtime commands".to_string()
        })
}

fn repo_root_from_manifest_dir() -> Option<PathBuf> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .map(Path::to_path_buf)
}

fn has_product_registry(path: &Path) -> bool {
    path.join("products/software-dev/product.toml").is_file()
}

#[cfg(test)]
mod tests {
    use super::{
        check_product_onboarding_readiness, create_product_workspace, load_api_plane_manifest,
        load_first_run_onboarding_contract, load_guided_sample_run_plan,
        load_product_workspace_projection, preview_product_intent,
        run_first_run_product_onboarding, run_guided_sample,
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

    #[test]
    fn product_onboarding_bridge_exposes_contract_readiness_and_sample_plan() {
        let repo = workspace_root();
        let dir = tempfile::tempdir().expect("tempdir");
        let workspace = dir.path().join("workspace");

        let contract = load_first_run_onboarding_contract("software-dev".to_string())
            .expect("load first run contract");
        assert!(contract
            .user_hidden_paths
            .contains(&".agentflow/**".to_string()));
        assert!(contract
            .command_entries
            .contains(&"check_product_onboarding_readiness".to_string()));

        create_product_workspace(
            repo.to_string_lossy().to_string(),
            ProductWorkspaceCreationRequest {
                project_name: "Desktop Onboarding Workspace".to_string(),
                workspace_root: workspace.to_string_lossy().to_string(),
                selected_product_id: "software-dev".to_string(),
                initial_goal: "Check onboarding readiness.".to_string(),
                creation_mode: ProductWorkspaceCreationMode::Create,
            },
        )
        .expect("create product workspace");

        let readiness = check_product_onboarding_readiness(
            repo.to_string_lossy().to_string(),
            workspace.to_string_lossy().to_string(),
            "software-dev".to_string(),
        )
        .expect("check onboarding readiness");
        assert!(readiness.user_hidden_agentflow_boundary);
        assert!(readiness.diagnostics_available);

        let plan = load_guided_sample_run_plan(
            workspace.to_string_lossy().to_string(),
            "software-dev".to_string(),
        )
        .expect("load guided sample plan");
        assert!(plan
            .expected_trace
            .iter()
            .any(|item| item.contains("Delivery")));
        let run_receipt = run_guided_sample(
            workspace.to_string_lossy().to_string(),
            "software-dev".to_string(),
            Some("deterministic-dry-run".to_string()),
        )
        .expect("run guided sample");
        assert_eq!(run_receipt.result, "passed");
        assert_eq!(run_receipt.decision_result, "accepted");
        assert!(workspace.join(&run_receipt.receipt_path).is_file());
        assert!(run_receipt
            .evidence_path
            .as_ref()
            .is_some_and(|path| workspace.join(path).is_file()));
        assert!(run_receipt
            .decision_path
            .as_ref()
            .is_some_and(|path| workspace.join(path).is_file()));
        assert!(run_receipt
            .delivery_path
            .as_ref()
            .is_some_and(|path| workspace.join(path).is_file()));
    }

    #[test]
    fn first_run_onboarding_command_invokes_runtime_sequence() {
        let dir = tempfile::tempdir().expect("tempdir");
        let workspace = dir.path().join("workspace");

        let receipt = run_first_run_product_onboarding(
            workspace.to_string_lossy().to_string(),
            "Desktop First Run".to_string(),
            "software-dev".to_string(),
            "Create the first guided sample.".to_string(),
            None,
        )
        .expect("run first-run product onboarding");

        assert_eq!(
            receipt.version,
            "agentflow-desktop-first-run-onboarding-receipt.v1"
        );
        assert_eq!(
            receipt.invoked_commands,
            vec![
                "create_product_workspace".to_string(),
                "check_product_onboarding_readiness".to_string(),
                "load_guided_sample_run_plan".to_string(),
                "run_guided_sample".to_string()
            ]
        );
        assert!(receipt.workspace_receipt.writes_authority);
        assert!(workspace.join(".agentflow/workspace.json").is_file());
        assert!(receipt.readiness.user_hidden_agentflow_boundary);
        assert_eq!(
            receipt.guided_sample_run_plan.selected_product_id,
            "software-dev"
        );
        assert_eq!(receipt.guided_sample_run_receipt.result, "passed");
        assert_eq!(
            receipt.guided_sample_run_receipt.issue_id,
            "AF-GUIDED-SAMPLE-001"
        );
        assert_eq!(
            receipt.guided_sample_run_receipt.decision_result,
            "accepted"
        );
        assert!(receipt.guided_sample_run_receipt.delivery_path.is_some());
        assert!(workspace
            .join(&receipt.guided_sample_run_receipt.receipt_path)
            .is_file());
    }

    #[test]
    fn first_run_onboarding_command_preserves_retry_failure_state() {
        let dir = tempfile::tempdir().expect("tempdir");
        let workspace = dir.path().join("workspace");

        let receipt = run_first_run_product_onboarding(
            workspace.to_string_lossy().to_string(),
            "Desktop Failed First Run".to_string(),
            "software-dev".to_string(),
            "Show retry state for failed guided sample.".to_string(),
            Some("deterministic-fail".to_string()),
        )
        .expect("run failed first-run product onboarding");

        assert_eq!(receipt.guided_sample_run_receipt.result, "failed");
        assert_eq!(
            receipt.guided_sample_run_receipt.decision_result,
            "rejected"
        );
        assert!(receipt.guided_sample_run_receipt.delivery_path.is_none());
        assert!(receipt.guided_sample_run_receipt.retryable);
        assert!(receipt
            .guided_sample_run_receipt
            .retry_attempt_path
            .as_ref()
            .is_some_and(|path| workspace.join(path).is_file()));
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
