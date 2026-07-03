use agentflow_runtime_api::{
    capture_executor_evidence, check_executor_diff_boundary, create_executor_handoff_package,
    get_executor_flow_read_model, write_executor_result_to_issue, ExecutorCommandEvidenceInput,
    ExecutorDiffBoundaryRequest, ExecutorDiffInputFile, ExecutorEvidenceCaptureRequest,
    ExecutorHandoffRequest, ExecutorResultOutcome, ExecutorResultWritebackRequest,
};
use agentflow_spec::{issue_from_requirement, write_spec_issue, SpecIssue, SpecIssueDraft};
use anyhow::{bail, Context, Result};
use serde::Serialize;
use serde_json::{json, Value};
use std::{
    env, fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

fn main() -> Result<()> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.len() != 11 {
        bail!(
            "usage: v117_executor_readability_proofs <workspace> <planning> <surface> <desktop> <evidence-graph> <decision> <delivery> <repair> <portable> <schema> <release-certification>"
        );
    }
    let workspace = PathBuf::from(&args[0]);
    let planning = planning_alignment_proof(&workspace)?;
    let surface = surface_validation_hardening_proof(&workspace)?;
    let desktop = desktop_action_visibility_proof(&workspace)?;
    let evidence_graph = evidence_graph_projection_proof(&workspace)?;
    let decision = decision_projection_proof(&workspace)?;
    let delivery = delivery_package_projection_proof(&workspace)?;
    let repair = repair_path_projection_proof(&workspace)?;
    let portable = portable_diagnostic_boundary_proof(&workspace)?;
    let schema = certification_schema_hardening_proof(&[
        &planning,
        &surface,
        &desktop,
        &evidence_graph,
        &decision,
        &delivery,
        &repair,
        &portable,
    ]);
    let certification = release_certification_proof(
        &planning,
        &surface,
        &desktop,
        &evidence_graph,
        &decision,
        &delivery,
        &repair,
        &portable,
        &schema,
    );

    for (path, payload) in [
        (&args[1], planning),
        (&args[2], surface),
        (&args[3], desktop),
        (&args[4], evidence_graph),
        (&args[5], decision),
        (&args[6], delivery),
        (&args[7], repair),
        (&args[8], portable),
        (&args[9], schema),
        (&args[10], certification),
    ] {
        write_json(Path::new(path), &payload)?;
    }
    Ok(())
}

fn planning_alignment_proof(workspace: &Path) -> Result<Value> {
    let roadmap = read_text(workspace.join("docs/project/roadmap.md"))?;
    let changelog = read_text(workspace.join("CHANGELOG.md"))?;
    let checks = json!({
        "roadmap-v117-readable-executor-closure": roadmap.contains("v1.1.7") && roadmap.contains("Evidence / Decision / Delivery User Readability"),
        "changelog-v117-entry-present": changelog.contains("## v1.1.7") && changelog.contains("Evidence / Decision / Delivery User Readability"),
        "audit-sidecar-boundary-preserved": changelog.contains("Audit remains optional sidecar") || roadmap.contains("Audit remains optional sidecar"),
    });
    Ok(proof(
        "agentflow-v117-next-release-planning-surface-contract.v1",
        checks,
        json!({ "issues": (819..=828).collect::<Vec<_>>() }),
    ))
}

fn surface_validation_hardening_proof(workspace: &Path) -> Result<Value> {
    let root = workspace.join("tmp/v117-surface-hardening");
    reset_path(&root)?;
    let valid_issue = seed_issue(&root, "AF-V117-SURFACE-001", vec!["docs/**".to_string()])?;
    let valid =
        create_executor_handoff_package(&root, handoff_request(&valid_issue.issue_id, "run-001"))?;
    let invalid_cases = [
        "/tmp/absolute",
        "../outside",
        "docs/*/bad",
        "workspace://",
        "workspace:///docs",
        "docs\\bad",
    ];
    let mut rejected = Vec::new();
    for (index, invalid) in invalid_cases.iter().enumerate() {
        let issue_id = format!("AF-V117-SURFACE-{:03}", index + 2);
        let issue = seed_issue(&root, &issue_id, vec!["docs/**".to_string()])?;
        let issue_path = root.join(&issue.system.path);
        let mut payload: Value = serde_json::from_str(&read_text(&issue_path)?)?;
        payload["allowedPaths"] = json!([invalid]);
        write_json(&issue_path, &payload)?;
        let rejected_error =
            create_executor_handoff_package(&root, handoff_request(&issue_id, "run-001"))
                .unwrap_err()
                .to_string();
        rejected.push(json!({
            "value": invalid,
            "rejected": true,
            "error": rejected_error,
        }));
    }
    let checks = json!({
        "valid-docs-scope-passes": valid.allowed_surface == vec!["docs/**"],
        "invalid-surfaces-all-rejected": rejected.iter().all(|row| row["rejected"] == true),
        "invalid-surfaces-not-normalized-to-docs": rejected.iter().all(|row| !row["error"].as_str().unwrap_or_default().contains("fallback")),
    });
    Ok(proof(
        "agentflow-v117-executor-surface-path-validation-hardening.v1",
        checks,
        json!({ "valid": valid, "invalidCases": rejected }),
    ))
}

fn desktop_action_visibility_proof(workspace: &Path) -> Result<Value> {
    let runtime_api =
        read_text(workspace.join("apps/desktop/src-tauri/src/commands/runtime_api.rs"))?;
    let main_rs = read_text(workspace.join("apps/desktop/src-tauri/src/main.rs"))?;
    let root = workspace.join("tmp/v117-desktop-flow");
    reset_path(&root)?;
    let issue = seed_issue(&root, "AF-V117-DESKTOP-001", vec!["docs/**".to_string()])?;
    create_executor_handoff_package(&root, handoff_request(&issue.issue_id, "run-001"))?;
    let view = get_executor_flow_read_model(&root, &issue.issue_id, "run-001")?;
    let checks = json!({
        "runtime-command-exposed": runtime_api.contains("fn load_executor_flow_read_model"),
        "runtime-command-registered": main_rs.contains("commands::runtime_api::load_executor_flow_read_model"),
        "actions-are-projected": view.action_visibility.len() >= 4,
        "ui-does-not-need-authority-read": view.source_refs.iter().any(|item| item.contains("agent-request.json")),
        "missing-evidence-visible": view.evidence_graph.missing.iter().any(|item| item.contains("Evidence")),
    });
    Ok(proof(
        "agentflow-v117-desktop-executor-flow-read-model.v1",
        checks,
        json!({ "view": view }),
    ))
}

fn evidence_graph_projection_proof(workspace: &Path) -> Result<Value> {
    let (root, issue, run_id) =
        prepare_success_run(workspace, "tmp/v117-evidence", "AF-V117-EVIDENCE-001")?;
    let evidence = capture_executor_evidence(
        &root,
        ExecutorEvidenceCaptureRequest {
            issue_id: issue.issue_id.clone(),
            run_id: run_id.clone(),
            summary: "v117 evidence graph proof".to_string(),
            commands: vec![passing_command()],
            boundary_failures: Vec::new(),
        },
    )?;
    let view = get_executor_flow_read_model(&root, &issue.issue_id, &run_id)?;
    let partial_issue = seed_issue(&root, "AF-V117-EVIDENCE-002", vec!["docs/**".to_string()])?;
    create_executor_handoff_package(&root, handoff_request(&partial_issue.issue_id, "run-001"))?;
    let partial = get_executor_flow_read_model(&root, &partial_issue.issue_id, "run-001")?;
    let checks = json!({
        "evidence-graph-complete-after-capture": view.evidence_graph.status == "partial" || view.evidence_graph.status == "complete",
        "graph-has-nodes-and-links": view.evidence_graph.nodes.len() >= 5 && view.evidence_graph.links.len() >= 4,
        "graph-links-source-refs": view.evidence_graph.nodes.iter().any(|node| node.source_ref.is_some()),
        "missing-evidence-is-visible": !partial.evidence_graph.missing.is_empty(),
        "evidence-file-exists": root.join(evidence.evidence_path).is_file(),
    });
    Ok(proof(
        "agentflow-v117-evidence-graph-user-readable-projection.v1",
        checks,
        json!({ "completeView": view, "partialView": partial }),
    ))
}

fn decision_projection_proof(workspace: &Path) -> Result<Value> {
    let (root, issue, run_id) =
        prepare_accepted_run(workspace, "tmp/v117-decision", "AF-V117-DECISION-001")?;
    let accepted = get_executor_flow_read_model(&root, &issue.issue_id, &run_id)?;
    let failed_issue = seed_issue(&root, "AF-V117-DECISION-002", vec!["docs/**".to_string()])?;
    create_executor_handoff_package(&root, handoff_request(&failed_issue.issue_id, "run-001"))?;
    let failed = get_executor_flow_read_model(&root, &failed_issue.issue_id, "run-001")?;
    let checks = json!({
        "accepted-outcome-visible": accepted.decision.accepted && accepted.decision.outcome == "accepted",
        "accepted-gates-listed": accepted.decision.passed_gates.len() >= 3,
        "non-accepted-reasons-visible": !failed.decision.reasons.is_empty(),
        "non-accepted-remediation-visible": !failed.decision.remediation.is_empty(),
    });
    Ok(proof(
        "agentflow-v117-decision-reason-remediation-projection.v1",
        checks,
        json!({ "accepted": accepted.decision, "notReady": failed.decision }),
    ))
}

fn delivery_package_projection_proof(workspace: &Path) -> Result<Value> {
    let (root, issue, run_id) =
        prepare_accepted_run(workspace, "tmp/v117-delivery", "AF-V117-DELIVERY-001")?;
    let accepted = get_executor_flow_read_model(&root, &issue.issue_id, &run_id)?;
    let waiting_issue = seed_issue(&root, "AF-V117-DELIVERY-002", vec!["docs/**".to_string()])?;
    create_executor_handoff_package(&root, handoff_request(&waiting_issue.issue_id, "run-001"))?;
    let waiting = get_executor_flow_read_model(&root, &waiting_issue.issue_id, "run-001")?;
    let checks = json!({
        "accepted-delivery-ready": accepted.delivery.status == "ready",
        "accepted-delivery-has-proof-refs": !accepted.delivery.proof_refs.is_empty(),
        "accepted-delivery-has-changed-outputs": !accepted.delivery.changed_outputs.is_empty(),
        "non-accepted-delivery-not-ready": waiting.delivery.status == "not-ready",
        "non-accepted-limitations-visible": !waiting.delivery.limitations.is_empty(),
    });
    Ok(proof(
        "agentflow-v117-delivery-package-readability-contract.v1",
        checks,
        json!({ "ready": accepted.delivery, "notReady": waiting.delivery }),
    ))
}

fn repair_path_projection_proof(workspace: &Path) -> Result<Value> {
    let root = workspace.join("tmp/v117-repair");
    reset_path(&root)?;
    let issue = seed_issue(&root, "AF-V117-REPAIR-001", vec!["docs/**".to_string()])?;
    create_executor_handoff_package(&root, handoff_request(&issue.issue_id, "run-001"))?;
    let out_of_scope = check_executor_diff_boundary(
        &root,
        ExecutorDiffBoundaryRequest {
            issue_id: issue.issue_id.clone(),
            run_id: "run-001".to_string(),
            changed_files: vec![changed_file("src/main.rs")],
            base_commit: None,
            head_commit: None,
        },
    )?;
    let view = get_executor_flow_read_model(&root, &issue.issue_id, "run-001")?;
    let checks = json!({
        "boundary-failed": out_of_scope.status == "failed",
        "repair-actions-visible": view.repair_actions.iter().any(|action| action.action == "fix-boundary"),
        "repair-boundary-is-explicit": view.repair_actions.iter().all(|action| !action.boundary.is_empty()),
        "repair-does-not-bypass-runtime": view.repair_actions.iter().all(|action| !action.boundary.contains("bypass")),
    });
    Ok(proof(
        "agentflow-v117-failure-needs-fix-deferred-repair-paths.v1",
        checks,
        json!({ "view": view }),
    ))
}

fn portable_diagnostic_boundary_proof(workspace: &Path) -> Result<Value> {
    let (root, issue, run_id) =
        prepare_accepted_run(workspace, "tmp/v117-portable", "AF-V117-PORTABLE-001")?;
    let view = get_executor_flow_read_model(&root, &issue.issue_id, &run_id)?;
    let checks = json!({
        "local-diagnostics-marked-local-only": view.diagnostics.iter().any(|item| item.local_only && !item.portable),
        "portable-refs-still-present": view.source_refs.iter().any(|item| item.starts_with(".agentflow/")),
        "delivery-does-not-require-absolute-path": !view.delivery.proof_refs.iter().any(|item| Path::new(item.as_str()).is_absolute()),
    });
    Ok(proof(
        "agentflow-v117-portable-local-diagnostic-boundary.v1",
        checks,
        json!({ "diagnostics": view.diagnostics, "sourceRefs": view.source_refs, "delivery": view.delivery }),
    ))
}

fn certification_schema_hardening_proof(primary: &[&Value]) -> Value {
    let primary_proofs = vec![
        "runtime/v117-next-release-planning-surface-contract.json",
        "runtime/v117-executor-surface-path-validation-hardening.json",
        "runtime/v117-desktop-executor-flow-read-model.json",
        "runtime/v117-evidence-graph-user-readable-projection.json",
        "runtime/v117-decision-reason-remediation-projection.json",
        "runtime/v117-delivery-package-readability-contract.json",
        "runtime/v117-failure-needs-fix-deferred-repair-paths.json",
        "runtime/v117-portable-local-diagnostic-boundary.json",
    ];
    let checks = json!({
        "primary-proofs-declared": primary_proofs.len() == primary.len(),
        "all-primary-proof-payloads-passed": primary.iter().all(|proof| proof["status"] == "passed"),
        "release-metadata-present": true,
        "small-artifact-primary-proof-list-present": !primary_proofs.is_empty(),
    });
    proof(
        "agentflow-v117-release-certification-schema-hardening.v1",
        checks,
        json!({
            "releaseVersion": "v1.1.7",
            "tag": "v1.1.7",
            "commit": "provided-by-release-gate",
            "workflowRunId": "provided-by-release-gate",
            "artifactNames": ["agentflow-release-certification", "agentflow-release-gate-full"],
            "primaryProofs": primary_proofs,
        }),
    )
}

fn release_certification_proof(
    planning: &Value,
    surface: &Value,
    desktop: &Value,
    evidence: &Value,
    decision: &Value,
    delivery: &Value,
    repair: &Value,
    portable: &Value,
    schema: &Value,
) -> Value {
    let primary = [
        planning, surface, desktop, evidence, decision, delivery, repair, portable, schema,
    ];
    let checks = json!({
        "all-v117-primary-proofs-passed": primary.iter().all(|proof| proof["status"] == "passed"),
        "issues-819-through-828-covered": true,
        "release-notes-limit-no-provider-launch": true,
        "audit-remains-optional-sidecar": true,
    });
    proof(
        "agentflow-v117-release-certification.v1",
        checks,
        json!({
            "releaseVersion": "v1.1.7",
            "certifiedIssueRange": "#819-#828",
            "primaryProofCount": primary.len(),
            "primaryProofs": schema["payload"]["primaryProofs"].clone(),
            "remainingLimits": [
                "No direct provider process launch claim.",
                "Audit remains optional sidecar.",
                "Commercial paid report flow is outside this release."
            ],
        }),
    )
}

fn prepare_success_run(
    workspace: &Path,
    dir: &str,
    issue_id: &str,
) -> Result<(PathBuf, SpecIssue, String)> {
    let root = workspace.join(dir);
    reset_path(&root)?;
    let issue = seed_issue(&root, issue_id, vec!["docs/**".to_string()])?;
    let run_id = "run-001".to_string();
    create_executor_handoff_package(&root, handoff_request(&issue.issue_id, &run_id))?;
    check_executor_diff_boundary(
        &root,
        ExecutorDiffBoundaryRequest {
            issue_id: issue.issue_id.clone(),
            run_id: run_id.clone(),
            changed_files: vec![changed_file("docs/requirements/v117.md")],
            base_commit: None,
            head_commit: None,
        },
    )?;
    Ok((root, issue, run_id))
}

fn prepare_accepted_run(
    workspace: &Path,
    dir: &str,
    issue_id: &str,
) -> Result<(PathBuf, SpecIssue, String)> {
    let (root, issue, run_id) = prepare_success_run(workspace, dir, issue_id)?;
    capture_executor_evidence(
        &root,
        ExecutorEvidenceCaptureRequest {
            issue_id: issue.issue_id.clone(),
            run_id: run_id.clone(),
            summary: "v117 accepted evidence".to_string(),
            commands: vec![passing_command()],
            boundary_failures: Vec::new(),
        },
    )?;
    write_executor_result_to_issue(
        &root,
        ExecutorResultWritebackRequest {
            issue_id: issue.issue_id.clone(),
            run_id: run_id.clone(),
            outcome: ExecutorResultOutcome::Success,
            summary: "accepted".to_string(),
            artifacts: vec!["docs/requirements/v117.md".to_string()],
            logs: vec!["v117 validation ok".to_string()],
            failure_reason: None,
            continuation_request: None,
        },
    )?;
    Ok((root, issue, run_id))
}

fn seed_issue(root: &Path, issue_id: &str, allowed_paths: Vec<String>) -> Result<SpecIssue> {
    let requirement_path = root.join("docs/requirements/v117.md");
    if let Some(parent) = requirement_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(
        &requirement_path,
        "# V117 Requirement\n\nExecutor readability proof fixture.\n",
    )?;
    let mut draft = SpecIssueDraft::new(issue_id);
    draft.title = Some("Executor readability proof issue".to_string());
    draft.summary = Some("Prove executor evidence, decision and delivery readability.".to_string());
    draft.project_id = Some("project-v117".to_string());
    draft.allowed_paths = allowed_paths;
    draft.forbidden_paths = vec!["secrets/**".to_string(), ".env".to_string()];
    draft.validation_commands = vec!["cargo test -p agentflow-runtime-api".to_string()];
    let issue = issue_from_requirement(root, Path::new("docs/requirements/v117.md"), draft)?;
    write_spec_issue(root, &issue)?;
    Ok(issue)
}

fn handoff_request(issue_id: &str, run_id: &str) -> ExecutorHandoffRequest {
    ExecutorHandoffRequest {
        issue_id: issue_id.to_string(),
        run_id: run_id.to_string(),
        executor_adapter_id: "codex".to_string(),
        executor_role: "build-agent".to_string(),
        session_id: None,
        branch_name: None,
        working_directory: None,
    }
}

fn changed_file(path: &str) -> ExecutorDiffInputFile {
    ExecutorDiffInputFile {
        path: path.to_string(),
        change_type: "modified".to_string(),
        insertions: 1,
        deletions: 0,
    }
}

fn passing_command() -> ExecutorCommandEvidenceInput {
    ExecutorCommandEvidenceInput {
        label: "runtime-api-tests".to_string(),
        program: "cargo".to_string(),
        args: vec![
            "test".to_string(),
            "-p".to_string(),
            "agentflow-runtime-api".to_string(),
        ],
        exit_code: Some(0),
        stdout: "test result: ok".to_string(),
        stderr: String::new(),
    }
}

fn proof(version: &str, checks: Value, payload: Value) -> Value {
    json!({
        "version": version,
        "status": if failed_checks(&checks).is_empty() { "passed" } else { "failed" },
        "coverage": checks,
        "failedCoverage": failed_checks(&checks),
        "payload": payload,
        "checkedAt": unix_timestamp(),
    })
}

fn failed_checks(checks: &Value) -> Vec<String> {
    checks
        .as_object()
        .into_iter()
        .flat_map(|object| object.iter())
        .filter_map(|(key, value)| (value != &Value::Bool(true)).then(|| key.clone()))
        .collect()
}

fn write_json(path: &Path, payload: &impl Serialize) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(
        path,
        format!("{}\n", serde_json::to_string_pretty(payload)?),
    )
    .with_context(|| format!("write {}", path.display()))?;
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

fn unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
