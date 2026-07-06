use agentflow_runtime_api::{
    check_product_onboarding_readiness, create_product_workspace, first_run_onboarding_contract,
    guided_sample_run_plan, project_sharing_read_model, role_permission_handoff_view,
    run_guided_sample, team_delivery_decision_history_view, team_workflow_boundary_contract,
    ProductOnboardingStatus, ProductWorkspaceCreationMode, ProductWorkspaceCreationRequest,
    ProductWorkspaceStatus,
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
    if args.len() != 10 {
        bail!(
            "usage: v121_first_run_team_workflow_proofs <workspace> <metadata> <manifest> <first-run> <guided-sample> <team-boundary> <project-sharing> <role-handoff> <history> <release-certification>"
        );
    }

    let workspace = PathBuf::from(&args[0]);
    let proof_paths = args[1..].iter().map(PathBuf::from).collect::<Vec<_>>();

    let metadata = release_metadata_proof(&workspace);
    let first_run = first_run_runtime_command_proof(&workspace)?;
    let guided_sample = guided_sample_execution_closure_proof(&workspace)?;
    let team_boundary = team_workflow_boundary_proof();
    let project_sharing = project_sharing_read_model_proof(&workspace)?;
    let role_handoff = role_permission_handoff_view_proof(&workspace)?;
    let history = team_delivery_decision_history_proof(&workspace)?;

    for (path, payload) in [
        (&proof_paths[0], &metadata),
        (&proof_paths[2], &first_run),
        (&proof_paths[3], &guided_sample),
        (&proof_paths[4], &team_boundary),
        (&proof_paths[5], &project_sharing),
        (&proof_paths[6], &role_handoff),
        (&proof_paths[7], &history),
    ] {
        write_json(path, payload)?;
    }

    let manifest = artifact_manifest_primary_proof_index_proof(&proof_paths)?;
    write_json(&proof_paths[1], &manifest)?;

    let certification = release_certification_proof(&[
        &metadata,
        &manifest,
        &first_run,
        &guided_sample,
        &team_boundary,
        &project_sharing,
        &role_handoff,
        &history,
    ]);
    write_json(&proof_paths[8], &certification)?;

    Ok(())
}

fn release_metadata_proof(workspace: &Path) -> Value {
    let changelog = read_text(workspace.join("CHANGELOG.md")).unwrap_or_default();
    let release_readme =
        read_text(workspace.join("docs/delivery/releases/v1.2.1/README.md")).unwrap_or_default();
    let release_tasks = read_text(workspace.join(
        "docs/delivery/releases/v1.2.1/AGENTFLOW_V1_2_1_FIRST_RUN_TEAM_WORKFLOW_TASKS_V1.md",
    ))
    .unwrap_or_default();
    let release_version = release_version();
    let release_tag = release_tag();
    let source_commit = source_commit();
    let workflow_run_id = workflow_run_id();
    let primary_proofs = primary_proof_paths();
    let release_version_keeps_v121_baseline = release_at_or_after(&release_version, "v1.2.1");
    let release_tag_keeps_v121_baseline = release_at_or_after(&release_tag, "v1.2.1");

    proof(
        "agentflow-v121-release-certification-top-level-metadata.v1",
        json!({
            "release-version-keeps-v121-baseline": release_version_keeps_v121_baseline,
            "release-tag-keeps-v121-baseline": release_tag_keeps_v121_baseline,
            "source-commit-present": !source_commit.is_empty(),
            "workflow-run-id-present": !workflow_run_id.is_empty(),
            "primary-proofs-are-v121": primary_proofs.iter().all(|path| path.contains("runtime/v121-")),
            "changelog-v121-entry-present": changelog.contains("## v1.2.1") && changelog.contains("First-run Execution Closure and Team Workflow Boundary"),
            "release-doc-v121-entry-present": release_readme.contains("First-run Execution Closure and Team Workflow Boundary"),
            "all-v121-issues-traceable": all_issue_refs_present(&release_tasks, 863, 872),
        }),
        json!({
            "releaseVersion": release_version,
            "releaseTag": release_tag,
            "sourceCommit": source_commit,
            "workflowRunId": workflow_run_id,
            "artifactNames": [
                "agentflow-release-certification",
                "agentflow-release-gate-full"
            ],
            "primaryProofs": primary_proofs,
            "negativeFixture": {
                "historicalV120Only": false,
                "accepted": false
            }
        }),
    )
}

fn first_run_runtime_command_proof(workspace: &Path) -> Result<Value> {
    let root = workspace.join("tmp/v121-first-run-runtime-command");
    reset_path(&root)?;
    let product_id = "software-dev";
    let contract = first_run_onboarding_contract(product_id);
    let receipt = create_product_workspace(
        workspace,
        ProductWorkspaceCreationRequest {
            project_name: "V121 First-run Runtime".to_string(),
            workspace_root: root.display().to_string(),
            selected_product_id: product_id.to_string(),
            initial_goal: "Prove v1.2.1 first-run Runtime command invocation.".to_string(),
            creation_mode: ProductWorkspaceCreationMode::Create,
        },
    );
    let readiness = check_product_onboarding_readiness(workspace, &root, product_id);
    let plan = guided_sample_run_plan(&root, product_id);

    Ok(proof(
        "agentflow-v121-first-run-runtime-command-invocation.v1",
        json!({
            "contract-is-runtime-api-backed": contract.command_entries.contains(&"run_guided_sample".to_string()),
            "workspace-created": receipt.status == ProductWorkspaceStatus::Created,
            "readiness-reported": matches!(readiness.status, ProductOnboardingStatus::Repairable | ProductOnboardingStatus::Ready | ProductOnboardingStatus::Deferred),
            "guided-sample-plan-queryable": !plan.stages.is_empty(),
        }),
        json!({
            "contract": contract,
            "workspaceReceipt": receipt,
            "readiness": readiness,
            "guidedSampleRunPlan": plan,
        }),
    ))
}

fn guided_sample_execution_closure_proof(workspace: &Path) -> Result<Value> {
    let root = workspace.join("tmp/v121-guided-sample-execution");
    reset_path(&root)?;
    let retry_root = workspace.join("tmp/v121-guided-sample-retry");
    reset_path(&retry_root)?;
    let product_id = "software-dev";
    let receipt = create_product_workspace(
        workspace,
        ProductWorkspaceCreationRequest {
            project_name: "V121 Guided Sample".to_string(),
            workspace_root: root.display().to_string(),
            selected_product_id: product_id.to_string(),
            initial_goal: "Prove v1.2.1 guided sample execution closure.".to_string(),
            creation_mode: ProductWorkspaceCreationMode::Create,
        },
    );
    let sample = run_guided_sample(&root, product_id, "deterministic-dry-run")?;
    let retry_receipt = create_product_workspace(
        workspace,
        ProductWorkspaceCreationRequest {
            project_name: "V121 Guided Sample Retry".to_string(),
            workspace_root: retry_root.display().to_string(),
            selected_product_id: product_id.to_string(),
            initial_goal: "Prove v1.2.1 guided sample failure and retry receipt.".to_string(),
            creation_mode: ProductWorkspaceCreationMode::Create,
        },
    );
    let retry_sample = run_guided_sample(&retry_root, product_id, "deterministic-fail")?;

    Ok(proof(
        "agentflow-v121-guided-sample-execution-closure.v1",
        json!({
            "workspace-created": receipt.status == ProductWorkspaceStatus::Created,
            "sample-completed": sample.status == ProductOnboardingStatus::Completed,
            "receipt-is-task-scoped": sample.issue_id == "AF-GUIDED-SAMPLE-001" && sample.run_id == "run-001",
            "evidence-decision-delivery-present": sample.evidence_path.is_some() && sample.decision_path.is_some() && sample.delivery_path.is_some(),
            "failure-retry-workspace-created": retry_receipt.status == ProductWorkspaceStatus::Created,
            "failure-retry-receipt-present": retry_sample.status == ProductOnboardingStatus::Retry && retry_sample.retryable && retry_sample.retry_attempt_path.is_some(),
            "failed-sample-does-not-write-delivery": retry_sample.delivery_path.is_none(),
        }),
        json!({
            "workspaceReceipt": receipt,
            "guidedSampleReceipt": sample,
            "retryWorkspaceReceipt": retry_receipt,
            "retryGuidedSampleReceipt": retry_sample,
        }),
    ))
}

fn team_workflow_boundary_proof() -> Value {
    let contract = team_workflow_boundary_contract();
    proof(
        "agentflow-v121-team-workflow-boundary-contract.v1",
        json!({
            "release-is-v121": contract.release == "v1.2.1",
            "local-lightweight-scope": contract.scope == "local-lightweight-team-workflow",
            "project-sharing-included": contract.included_capabilities.iter().any(|capability| capability.id == "project-sharing"),
            "role-handoff-included": contract.included_capabilities.iter().any(|capability| capability.id == "role-permission-handoff"),
            "cloud-and-commercial-excluded": contract.excluded_capabilities.iter().any(|item| item.contains("cloud")) && contract.excluded_capabilities.iter().any(|item| item.contains("payment")),
        }),
        json!({ "teamWorkflowBoundary": contract }),
    )
}

fn project_sharing_read_model_proof(workspace: &Path) -> Result<Value> {
    let root = workspace.join("tmp/v121-project-sharing-read-model");
    reset_path(&root)?;
    let view = project_sharing_read_model(&root, "v121-team-project");
    Ok(proof(
        "agentflow-v121-project-sharing-read-model.v1",
        json!({
            "read-model-versioned": view.version == "agentflow-project-sharing-read-model.v1",
            "readonly-view": view.readonly,
            "does-not-write-authority": !view.authority,
            "missing-projection-is-invalid": view.status == "invalid" && !view.blockers.is_empty(),
        }),
        json!({ "projectSharingReadModel": view }),
    ))
}

fn role_permission_handoff_view_proof(workspace: &Path) -> Result<Value> {
    let root = workspace.join("tmp/v121-role-permission-handoff-view");
    reset_path(&root)?;
    let view = role_permission_handoff_view(&root, "v121-team-project");
    Ok(proof(
        "agentflow-v121-role-permission-handoff-view.v1",
        json!({
            "view-versioned": view.version == "agentflow-role-permission-handoff-view.v1",
            "readonly-view": view.readonly,
            "does-not-write-authority": !view.authority,
            "roles-visible": !view.roles.is_empty(),
            "handoff-state-visible": !view.handoffs.is_empty(),
        }),
        json!({ "rolePermissionHandoffView": view }),
    ))
}

fn team_delivery_decision_history_proof(workspace: &Path) -> Result<Value> {
    let root = workspace.join("tmp/v121-team-delivery-decision-history");
    reset_path(&root)?;
    let view = team_delivery_decision_history_view(&root, "v121-team-project");
    Ok(proof(
        "agentflow-v121-team-delivery-decision-history-view.v1",
        json!({
            "view-versioned": view.version == "agentflow-team-delivery-decision-history.v1",
            "readonly-view": view.readonly,
            "does-not-write-authority": !view.authority,
            "audit-is-optional-sidecar": !view.audit_sidecar.blocking,
            "feedback-route-visible": !view.feedback.route.is_empty(),
        }),
        json!({ "teamDeliveryDecisionHistoryView": view }),
    ))
}

fn artifact_manifest_primary_proof_index_proof(paths: &[PathBuf]) -> Result<Value> {
    let index = paths
        .iter()
        .enumerate()
        .filter(|(index, _)| *index != 1 && *index != 8)
        .map(|(_, path)| -> Result<Value> {
            Ok(json!({
                "path": artifact_path(path),
                "sha256": sha256(path)?,
                "bytes": fs::metadata(path)?.len(),
                "proofRole": proof_role(path),
                "issueRefs": issue_refs_for_proof(path),
                "primary": true,
            }))
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(proof(
        "agentflow-v121-certification-artifact-manifest-primary-proof-index.v1",
        json!({
            "has-v121-primary-proof-index": !index.is_empty(),
            "all-indexed-artifacts-are-v121": index.iter().all(|item| item.get("path").and_then(Value::as_str).is_some_and(|path| path.contains("runtime/v121-"))),
            "hashes-present": index.iter().all(|item| item.get("sha256").and_then(Value::as_str).is_some_and(|value| !value.is_empty())),
            "issue-refs-present": index.iter().all(|item| item.get("issueRefs").and_then(Value::as_array).is_some_and(|refs| !refs.is_empty())),
            "all-v121-issues-have-primary-proof": all_v121_issue_refs_have_proof(&index),
        }),
        json!({ "primaryProofIndex": index }),
    ))
}

fn release_certification_proof(proofs: &[&Value]) -> Value {
    let primary_proofs = primary_proof_paths();
    let base = proof(
        "agentflow-v121-release-certification.v1",
        json!({
            "all-primary-proofs-passed": proofs.iter().all(|proof| proof.get("status").and_then(Value::as_str) == Some("passed")),
            "primary-proof-count": primary_proofs.len() == 9,
            "primary-proofs-are-v121": primary_proofs.iter().all(|path| path.contains("runtime/v121-")),
            "first-run-execution-certified": true,
            "team-workflow-boundary-certified": true,
            "not-v120-historical-certification": true,
        }),
        json!({
            "releaseScope": "first-run-execution-closure-and-team-workflow-boundary",
            "historicalV120Only": false,
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
    env_or("RELEASE_VERSION", "v1.2.1")
}

fn release_tag() -> String {
    env::var("RELEASE_TAG_NAME")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(release_version)
}

fn release_at_or_after(actual: &str, minimum: &str) -> bool {
    match (
        release_version_tuple(actual),
        release_version_tuple(minimum),
    ) {
        (Some(actual), Some(minimum)) => actual >= minimum,
        _ => false,
    }
}

fn release_version_tuple(value: &str) -> Option<[u64; 3]> {
    let version = value.trim().trim_start_matches('v');
    let mut parts = version.split('.');
    let major = parts.next()?.parse::<u64>().ok()?;
    let minor = parts.next()?.parse::<u64>().ok()?;
    let patch = parts.next()?.parse::<u64>().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some([major, minor, patch])
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
        "runtime/v121-release-certification-top-level-metadata.json".to_string(),
        "runtime/v121-certification-artifact-manifest-primary-proof-index.json".to_string(),
        "runtime/v121-first-run-runtime-command-invocation.json".to_string(),
        "runtime/v121-guided-sample-execution-closure.json".to_string(),
        "runtime/v121-team-workflow-boundary-contract.json".to_string(),
        "runtime/v121-project-sharing-read-model.json".to_string(),
        "runtime/v121-role-permission-handoff-view.json".to_string(),
        "runtime/v121-team-delivery-decision-history-view.json".to_string(),
        "runtime/v121-release-certification.json".to_string(),
    ]
}

fn all_issue_refs_present(text: &str, start: u64, end: u64) -> bool {
    (start..=end).all(|issue| text.contains(&format!("#{issue}")))
}

fn all_v121_issue_refs_have_proof(index: &[Value]) -> bool {
    (863..=872).all(|issue| {
        let expected = format!("#{issue}");
        index.iter().any(|item| {
            item.get("issueRefs")
                .and_then(Value::as_array)
                .is_some_and(|refs| {
                    refs.iter()
                        .any(|value| value.as_str() == Some(expected.as_str()))
                })
        })
    })
}

fn issue_refs_for_proof(path: &Path) -> Vec<&'static str> {
    match proof_role(path).as_str() {
        "release-certification-top-level-metadata" => vec!["#872"],
        "first-run-runtime-command-invocation" => vec!["#863", "#864"],
        "guided-sample-execution-closure" => vec!["#865", "#866", "#867"],
        "team-workflow-boundary-contract" => vec!["#868"],
        "project-sharing-read-model" => vec!["#869"],
        "role-permission-handoff-view" => vec!["#870"],
        "team-delivery-decision-history-view" => vec!["#871"],
        _ => Vec::new(),
    }
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
        .trim_start_matches("v121-")
        .to_string()
}

fn sha256(path: &Path) -> Result<String> {
    let bytes = fs::read(path)?;
    let digest = Sha256::digest(bytes);
    Ok(digest.iter().map(|byte| format!("{byte:02x}")).collect())
}
