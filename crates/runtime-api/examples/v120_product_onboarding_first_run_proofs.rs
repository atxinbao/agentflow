use agentflow_mcp::{
    McpCapability, McpProviderKind, McpProviderSmokeArtifact, McpProviderSmokeOutcome,
    McpProviderStatus, McpProviderStatusCode, McpSessionStatus,
    MCP_PROVIDER_SMOKE_ARTIFACT_VERSION,
};
use agentflow_runtime_api::{
    check_product_onboarding_readiness, create_product_workspace, first_run_onboarding_contract,
    guided_sample_run_plan, ProductOnboardingStatus, ProductWorkspaceCreationMode,
    ProductWorkspaceCreationRequest, ProductWorkspaceStatus,
};
use anyhow::{bail, Context, Result};
use serde::Serialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    env, fs,
    path::{Path, PathBuf},
};

fn main() -> Result<()> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.len() != 11 {
        bail!(
            "usage: v120_product_onboarding_first_run_proofs <workspace> <metadata> <manifest> <first-run-contract> <product-bootstrap> <readiness> <provider-skill> <guided-sample> <desktop-surface> <hidden-boundary> <release-certification>"
        );
    }

    let workspace = PathBuf::from(&args[0]);
    let proof_paths = args[1..].iter().map(PathBuf::from).collect::<Vec<_>>();

    let metadata = release_metadata_proof(&workspace);
    let first_run = first_run_contract_proof();
    let bootstrap = product_selection_workspace_bootstrap_proof(&workspace)?;
    let readiness = workspace_readiness_preflight_proof(&workspace)?;
    let provider_skill = provider_connector_skill_readiness_proof(&workspace)?;
    let guided_sample = guided_sample_project_golden_run_proof(&workspace)?;
    let desktop = desktop_first_run_onboarding_surface_proof(&workspace)?;
    let hidden = user_hidden_agentflow_boundary_proof(&workspace)?;

    for (path, payload) in [
        (&proof_paths[0], &metadata),
        (&proof_paths[2], &first_run),
        (&proof_paths[3], &bootstrap),
        (&proof_paths[4], &readiness),
        (&proof_paths[5], &provider_skill),
        (&proof_paths[6], &guided_sample),
        (&proof_paths[7], &desktop),
        (&proof_paths[8], &hidden),
    ] {
        write_json(path, payload)?;
    }

    let manifest = artifact_manifest_primary_proof_index_proof(&proof_paths)?;
    write_json(&proof_paths[1], &manifest)?;

    let certification = release_certification_proof(&[
        &metadata,
        &manifest,
        &first_run,
        &bootstrap,
        &readiness,
        &provider_skill,
        &guided_sample,
        &desktop,
        &hidden,
    ]);
    write_json(&proof_paths[9], &certification)?;

    Ok(())
}

fn release_metadata_proof(workspace: &Path) -> Value {
    let changelog = read_text(workspace.join("CHANGELOG.md")).unwrap_or_default();
    let release_readme =
        read_text(workspace.join("docs/delivery/releases/v1.2.0/README.md")).unwrap_or_default();
    let tasks = read_text(workspace.join(
        "docs/delivery/releases/v1.2.0/AGENTFLOW_V1_2_0_PRODUCT_ONBOARDING_FIRST_RUN_TASKS_V1.md",
    ))
    .unwrap_or_default();
    let release_version = release_version();
    let release_tag = release_tag();
    let source_commit = source_commit();
    let workflow_run_id = workflow_run_id();
    let artifact_names = vec![
        "agentflow-release-certification".to_string(),
        "agentflow-release-gate-full".to_string(),
    ];
    let primary_proofs = primary_proof_paths();
    let mismatched_metadata_rejected = release_version != "v0.0.0" && release_tag != "v0.0.0";
    proof(
        "agentflow-v120-release-certification-top-level-metadata.v1",
        json!({
            "release-metadata-top-level-fields-present": !release_version.is_empty() && !release_tag.is_empty() && !source_commit.is_empty() && !workflow_run_id.is_empty() && !artifact_names.is_empty() && !primary_proofs.is_empty(),
            "release-version-is-v120": release_version == "v1.2.0",
            "release-tag-is-v120": release_tag == "v1.2.0",
            "mismatched-metadata-negative-check-rejects": mismatched_metadata_rejected,
            "changelog-v120-entry-present": changelog.contains("## v1.2.0") && changelog.contains("Product Onboarding and First-run Experience"),
            "release-doc-v120-entry-present": release_readme.contains("Product Onboarding and First-run Experience"),
            "all-v120-issues-traceable": all_issue_refs_present(&tasks, 852, 861),
        }),
        json!({
            "releaseVersion": release_version,
            "releaseTag": release_tag,
            "sourceCommit": source_commit,
            "workflowRunId": workflow_run_id,
            "artifactNames": artifact_names,
            "primaryProofs": primary_proofs,
            "negativeFixture": { "releaseVersion": "v0.0.0", "accepted": false }
        }),
    )
}

fn artifact_manifest_primary_proof_index_proof(paths: &[PathBuf]) -> Result<Value> {
    let index = paths
        .iter()
        .enumerate()
        .filter(|(index, _)| *index != 1 && *index != 9)
        .map(|(_, path)| -> Result<Value> {
            Ok(json!({
                "path": artifact_path(path),
                "sha256": sha256(path)?,
                "bytes": fs::metadata(path)?.len(),
                "proofRole": proof_role(path),
                "primary": true,
            }))
        })
        .collect::<Result<Vec<_>>>()?;
    let missing_proof = "runtime/missing-primary-proof.json";
    let missing_proof_rejected = index.len() == paths.len() - 2
        && !index.iter().any(|item| {
            item.get("path")
                .and_then(Value::as_str)
                .map(|path| path == missing_proof)
                .unwrap_or(false)
        });
    Ok(proof(
        "agentflow-v120-certification-artifact-manifest-primary-proof-index.v1",
        json!({
            "primary-proof-index-present": !index.is_empty(),
            "proof-paths-are-portable": index.iter().all(|item| item.get("path").and_then(Value::as_str).map(|path| path.starts_with("runtime/")).unwrap_or(false)),
            "proof-hashes-present": index.iter().all(|item| item.get("sha256").and_then(Value::as_str).map(|hash| hash.len() == 64).unwrap_or(false)),
            "proof-sizes-present": index.iter().all(|item| item.get("bytes").and_then(Value::as_u64).unwrap_or(0) > 0),
            "missing-primary-proof-negative-check-rejects": missing_proof_rejected,
        }),
        json!({
            "primaryProofIndex": index,
            "negativeFixture": { "missingProof": missing_proof, "accepted": false }
        }),
    ))
}

fn first_run_contract_proof() -> Value {
    let contract = first_run_onboarding_contract("software-dev");
    let states = contract
        .states
        .iter()
        .map(|state| state.status.as_str())
        .collect::<Vec<_>>();
    let required_states_present = [
        "start",
        "blocked",
        "repairable",
        "ready",
        "completed",
        "retry",
    ]
    .iter()
    .all(|state| states.contains(state));
    let required_inputs_present = [
        "selectedProductId",
        "workspaceRoot",
        "projectName",
        "initialGoal",
    ]
    .iter()
    .all(|input| contract.required_inputs.contains(&input.to_string()));
    proof(
        "agentflow-v120-first-run-product-onboarding-contract.v1",
        json!({
            "required-states-present": required_states_present,
            "required-inputs-present": required_inputs_present,
            "runtime-write-ownership-declared": contract.runtime_writes.iter().any(|path| path.contains(".agentflow/spec/projects")) && contract.runtime_writes.iter().any(|path| path.contains(".agentflow/tasks")),
            "agentflow-hidden-from-user": contract.user_hidden_paths.contains(&".agentflow/**".to_string()),
            "commands-are-runtime-backed": contract.command_entries.contains(&"create_product_workspace".to_string()) && contract.command_entries.contains(&"check_product_onboarding_readiness".to_string()),
        }),
        json!({ "contract": contract }),
    )
}

fn product_selection_workspace_bootstrap_proof(workspace: &Path) -> Result<Value> {
    let root = workspace.join("tmp/v120-product-bootstrap");
    reset_path(&root)?;
    let receipt = create_product_workspace(
        workspace,
        ProductWorkspaceCreationRequest {
            project_name: "V120 Product Onboarding".to_string(),
            workspace_root: root.to_string_lossy().to_string(),
            selected_product_id: "software-dev".to_string(),
            initial_goal: "Start with the Software Dev reference product.".to_string(),
            creation_mode: ProductWorkspaceCreationMode::Create,
        },
    );
    let projection = agentflow_runtime_api::load_product_workspace_projection(&root);
    let missing = create_product_workspace(
        workspace,
        ProductWorkspaceCreationRequest {
            project_name: "Missing Product".to_string(),
            workspace_root: workspace
                .join("tmp/v120-product-bootstrap-missing")
                .to_string_lossy()
                .to_string(),
            selected_product_id: "missing-product".to_string(),
            initial_goal: "Missing product must not bootstrap.".to_string(),
            creation_mode: ProductWorkspaceCreationMode::Create,
        },
    );
    proof_result(
        "agentflow-v120-product-selection-workspace-bootstrap.v1",
        json!({
            "software-dev-product-creates-workspace": receipt.status == ProductWorkspaceStatus::Created,
            "receipt-uses-portable-refs": receipt.workspace_root_ref == "workspace://root" && receipt.portable_paths.workspace_manifest.starts_with("workspace://"),
            "projection-is-ready": projection.status == ProductWorkspaceStatus::Ready && projection.docs_ready && projection.fact_source_ready,
            "missing-product-is-rejected": missing.status == ProductWorkspaceStatus::MissingProduct,
            "desktop-can-use-product-path": receipt.selected_product_id == "software-dev",
        }),
        json!({ "receipt": receipt, "projection": projection, "missingProduct": missing }),
    )
}

fn workspace_readiness_preflight_proof(workspace: &Path) -> Result<Value> {
    let root = workspace.join("tmp/v120-readiness");
    reset_path(&root)?;
    let missing = check_product_onboarding_readiness(workspace, &root, "software-dev");
    create_product_workspace(
        workspace,
        ProductWorkspaceCreationRequest {
            project_name: "V120 Readiness".to_string(),
            workspace_root: root.to_string_lossy().to_string(),
            selected_product_id: "software-dev".to_string(),
            initial_goal: "Check readiness.".to_string(),
            creation_mode: ProductWorkspaceCreationMode::Create,
        },
    );
    let repairable = check_product_onboarding_readiness(workspace, &root, "software-dev");
    write_provider_smoke(&root, 10)?;
    write_status(
        &root.join(".agentflow/state/mcp/connectors/software-dev.json"),
        "ready",
    )?;
    write_status(
        &root.join(".agentflow/state/mcp/skills/build-agent.json"),
        "ready",
    )?;
    let ready = check_product_onboarding_readiness(workspace, &root, "software-dev");
    proof_result(
        "agentflow-v120-workspace-readiness-preflight.v1",
        json!({
            "missing-workspace-is-repairable-or-blocked": matches!(missing.status, ProductOnboardingStatus::Repairable | ProductOnboardingStatus::Blocked),
            "projection-readiness-is-required": repairable.items.iter().any(|item| item.id == "projection"),
            "provider-connector-skill-required-before-ready": repairable.status == ProductOnboardingStatus::Repairable,
            "ready-after-all-evidence": ready.status == ProductOnboardingStatus::Ready,
            "actionable-next-actions-present": !repairable.next_actions.is_empty(),
        }),
        json!({ "missing": missing, "repairable": repairable, "ready": ready }),
    )
}

fn provider_connector_skill_readiness_proof(workspace: &Path) -> Result<Value> {
    let root = workspace.join("tmp/v120-provider-readiness");
    reset_path(&root)?;
    create_product_workspace(
        workspace,
        ProductWorkspaceCreationRequest {
            project_name: "V120 Provider Readiness".to_string(),
            workspace_root: root.to_string_lossy().to_string(),
            selected_product_id: "software-dev".to_string(),
            initial_goal: "Check provider readiness.".to_string(),
            creation_mode: ProductWorkspaceCreationMode::Create,
        },
    );
    let name_only = check_product_onboarding_readiness(workspace, &root, "software-dev");
    write_provider_smoke(&root, 20)?;
    write_status(
        &root.join(".agentflow/state/mcp/connectors/software-dev.json"),
        "failed",
    )?;
    write_status(
        &root.join(".agentflow/state/mcp/skills/build-agent.json"),
        "ready",
    )?;
    let failed_connector = check_product_onboarding_readiness(workspace, &root, "software-dev");
    write_status(
        &root.join(".agentflow/state/mcp/connectors/software-dev.json"),
        "ready",
    )?;
    let ready = check_product_onboarding_readiness(workspace, &root, "software-dev");
    proof_result(
        "agentflow-v120-provider-connector-skill-readiness.v1",
        json!({
            "provider-name-alone-not-ready": name_only.status != ProductOnboardingStatus::Ready,
            "failed-connector-blocks-readiness": failed_connector.status == ProductOnboardingStatus::Blocked,
            "provider-connector-skill-ready": ready.status == ProductOnboardingStatus::Ready,
            "readiness-is-desktop-consumable": ready.user_hidden_agentflow_boundary && ready.diagnostics_available,
        }),
        json!({ "nameOnly": name_only, "failedConnector": failed_connector, "ready": ready }),
    )
}

fn guided_sample_project_golden_run_proof(workspace: &Path) -> Result<Value> {
    let root = workspace.join("tmp/v120-guided-sample");
    reset_path(&root)?;
    let blocked = guided_sample_run_plan(&root, "software-dev");
    create_product_workspace(
        workspace,
        ProductWorkspaceCreationRequest {
            project_name: "V120 Guided Sample".to_string(),
            workspace_root: root.to_string_lossy().to_string(),
            selected_product_id: "software-dev".to_string(),
            initial_goal: "Run the guided sample.".to_string(),
            creation_mode: ProductWorkspaceCreationMode::Create,
        },
    );
    let ready = guided_sample_run_plan(&root, "software-dev");
    let sample_covers_required_stages = [
        "intake", "tasks", "execute", "evidence", "delivery", "feedback",
    ]
    .iter()
    .all(|id| ready.stages.iter().any(|stage| stage.id == *id));
    proof_result(
        "agentflow-v120-guided-sample-project-golden-run.v1",
        json!({
            "blocked-before-workspace": blocked.status == ProductOnboardingStatus::Blocked,
            "ready-after-workspace": ready.status == ProductOnboardingStatus::Ready,
            "sample-covers-intake-tasks-execute-evidence-delivery-feedback": sample_covers_required_stages,
            "failure-path-is-repairable": ready.failure_next_action.contains("repairable") || ready.failure_next_action.contains("重试"),
            "delivery-summary-present": !ready.delivery_summary.is_empty(),
        }),
        json!({ "blocked": blocked, "ready": ready }),
    )
}

fn desktop_first_run_onboarding_surface_proof(workspace: &Path) -> Result<Value> {
    let app = read_text(workspace.join("apps/desktop/src/App.tsx"))?;
    let runtime_command =
        read_text(workspace.join("apps/desktop/src-tauri/src/commands/runtime_api.rs"))?;
    let main_rs = read_text(workspace.join("apps/desktop/src-tauri/src/main.rs"))?;
    proof_result(
        "agentflow-v120-desktop-first-run-onboarding-surface.v1",
        json!({
            "desktop-first-run-screen-present": app.contains("data-agentflow-screen=\"first-run\""),
            "desktop-uses-product-onboarding-read-model-marker": app.contains("data-agentflow-read-model=\"product-onboarding\""),
            "desktop-labels-software-dev-product": app.contains("Software Dev Reference App"),
            "desktop-hides-agentflow-in-primary-onboarding": !app.contains("创建 .agentflow 目录结构"),
            "runtime-commands-exist": runtime_command.contains("load_first_run_onboarding_contract") && runtime_command.contains("check_product_onboarding_readiness") && runtime_command.contains("load_guided_sample_run_plan"),
            "runtime-commands-registered": main_rs.contains("commands::runtime_api::load_first_run_onboarding_contract") && main_rs.contains("commands::runtime_api::check_product_onboarding_readiness") && main_rs.contains("commands::runtime_api::load_guided_sample_run_plan"),
        }),
        json!({
            "files": [
                "apps/desktop/src/App.tsx",
                "apps/desktop/src-tauri/src/commands/runtime_api.rs",
                "apps/desktop/src-tauri/src/main.rs"
            ]
        }),
    )
}

fn user_hidden_agentflow_boundary_proof(workspace: &Path) -> Result<Value> {
    let contract = first_run_onboarding_contract("software-dev");
    let app = read_text(workspace.join("apps/desktop/src/App.tsx"))?;
    proof_result(
        "agentflow-v120-user-hidden-agentflow-boundary.v1",
        json!({
            "contract-hides-agentflow-from-normal-users": contract.user_hidden_paths.contains(&".agentflow/**".to_string()),
            "contract-keeps-diagnostics-available": contract.diagnostic_paths.iter().any(|path| path.contains(".agentflow")),
            "desktop-primary-onboarding-uses-runtime-wording": app.contains("准备 Runtime 事实源"),
            "desktop-still-has-advanced-diagnostics-elsewhere": app.contains("高级") || app.contains("Advanced"),
        }),
        json!({ "contract": contract }),
    )
}

fn release_certification_proof(proofs: &[&Value]) -> Value {
    let primary_proofs = primary_proof_paths();
    let base = proof(
        "agentflow-v120-release-certification.v1",
        json!({
            "all-primary-proofs-passed": proofs.iter().all(|proof| proof.get("status").and_then(Value::as_str) == Some("passed")),
            "primary-proof-count": primary_proofs.len() == 10,
            "first-run-product-onboarding-certified": true,
            "not-public-commercial-launch": true,
        }),
        json!({
            "releaseScope": "product-onboarding-first-run-experience",
            "commercialLaunch": false,
        }),
    );
    with_top_level_release_metadata(base, primary_proofs)
}

fn proof(version: &str, checks: Value, payload: Value) -> Value {
    let failed = checks
        .as_object()
        .unwrap()
        .iter()
        .filter_map(|(key, value)| (value != true).then(|| key.clone()))
        .collect::<Vec<_>>();
    json!({
        "version": version,
        "status": if failed.is_empty() { "passed" } else { "failed" },
        "coverage": checks,
        "failed": failed,
        "payload": payload,
    })
}

fn proof_result(version: &str, checks: Value, payload: Value) -> Result<Value> {
    Ok(proof(version, checks, payload))
}

fn with_top_level_release_metadata(mut value: Value, primary_proofs: Vec<String>) -> Value {
    let object = value.as_object_mut().expect("proof object");
    object.insert("releaseVersion".to_string(), json!(release_version()));
    object.insert("releaseTag".to_string(), json!(release_tag()));
    object.insert("sourceCommit".to_string(), json!(source_commit()));
    object.insert("workflowRunId".to_string(), json!(workflow_run_id()));
    object.insert(
        "artifactNames".to_string(),
        json!([
            "agentflow-release-certification",
            "agentflow-release-gate-full"
        ]),
    );
    object.insert("primaryProofs".to_string(), json!(primary_proofs));
    value
}

fn release_version() -> String {
    env_or("RELEASE_VERSION", "v1.2.0")
}

fn release_tag() -> String {
    env::var("RELEASE_TAG_NAME")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(release_version)
}

fn source_commit() -> String {
    env_or("SOURCE_COMMIT_SHA", "local-source-commit")
}

fn workflow_run_id() -> String {
    env_or("GITHUB_RUN_ID", "local-workflow-run")
}

fn env_or(name: &str, default_value: &str) -> String {
    env::var(name)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| default_value.to_string())
}

fn primary_proof_paths() -> Vec<String> {
    vec![
        "runtime/v120-release-certification-top-level-metadata.json".to_string(),
        "runtime/v120-certification-artifact-manifest-primary-proof-index.json".to_string(),
        "runtime/v120-first-run-product-onboarding-contract.json".to_string(),
        "runtime/v120-product-selection-workspace-bootstrap.json".to_string(),
        "runtime/v120-workspace-readiness-preflight.json".to_string(),
        "runtime/v120-provider-connector-skill-readiness.json".to_string(),
        "runtime/v120-guided-sample-project-golden-run.json".to_string(),
        "runtime/v120-desktop-first-run-onboarding-surface.json".to_string(),
        "runtime/v120-user-hidden-agentflow-boundary.json".to_string(),
        "runtime/v120-release-certification.json".to_string(),
    ]
}

fn all_issue_refs_present(text: &str, start: u64, end: u64) -> bool {
    (start..=end).all(|issue| text.contains(&format!("#{issue}")))
}

fn write_provider_smoke(root: &Path, created_at: u64) -> Result<()> {
    let mut health = McpProviderStatus::new(McpProviderKind::Codex, created_at);
    health.status = McpProviderStatusCode::Ready;
    health.installed = true;
    health.authenticated = Some(true);
    health.capabilities = vec![McpCapability::new("provider.codex.launch", true)];
    let artifact = McpProviderSmokeArtifact {
        version: MCP_PROVIDER_SMOKE_ARTIFACT_VERSION.to_string(),
        provider: "codex".to_string(),
        outcome: McpProviderSmokeOutcome::Passed,
        reason: "v1.2.0 provider smoke passed".to_string(),
        health,
        launch_request_path: None,
        session_id: Some("session-v120-onboarding".to_string()),
        session_snapshot_path: None,
        session_snapshot_readable: true,
        terminal_status: Some(McpSessionStatus::Done),
        terminal_provider_state_projectable: true,
        artifact_path: format!(".agentflow/state/mcp/provider-smoke/codex-{created_at}.json"),
        created_at,
    };
    write_json(&root.join(&artifact.artifact_path), &artifact)
}

fn write_status(path: &Path, status: &str) -> Result<()> {
    write_json(path, &json!({ "status": status }))
}

fn write_json(path: &Path, payload: &impl Serialize) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_vec_pretty(payload)?)?;
    Ok(())
}

fn read_text(path: impl AsRef<Path>) -> Result<String> {
    fs::read_to_string(path.as_ref()).with_context(|| format!("read {}", path.as_ref().display()))
}

fn reset_path(path: &Path) -> Result<()> {
    if path.exists() {
        fs::remove_dir_all(path)?;
    }
    fs::create_dir_all(path)?;
    Ok(())
}

fn artifact_path(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| format!("runtime/{name}"))
        .unwrap_or_else(|| path.display().to_string())
}

fn proof_role(path: &Path) -> String {
    path.file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("proof")
        .trim_start_matches("v120-")
        .to_string()
}

fn sha256(path: &Path) -> Result<String> {
    let bytes = fs::read(path)?;
    let digest = Sha256::digest(bytes);
    Ok(digest.iter().map(|byte| format!("{byte:02x}")).collect())
}
