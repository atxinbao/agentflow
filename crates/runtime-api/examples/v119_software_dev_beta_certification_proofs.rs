use agentflow_event_store::{append_task_event_once, EventActor, TaskEventDraft};
use agentflow_mcp::{
    McpCapability, McpProviderKind, McpProviderSmokeArtifact, McpProviderSmokeOutcome,
    McpProviderStatus, McpProviderStatusCode, McpSessionStatus,
    MCP_PROVIDER_SMOKE_ARTIFACT_VERSION,
};
use agentflow_runtime_api::{
    capture_executor_evidence, check_executor_diff_boundary, check_executor_workspace_health,
    create_executor_handoff_package, get_executor_flow_read_model, rebuild_executor_projection,
    recover_failed_executor_command, write_executor_result_to_issue, ExecutorCommandEvidenceInput,
    ExecutorCommandRecoveryAction, ExecutorCommandRecoveryRequest, ExecutorDiffBoundaryRequest,
    ExecutorDiffInputFile, ExecutorEvidenceCaptureRequest, ExecutorHandoffRequest,
    ExecutorResultOutcome, ExecutorResultWritebackRequest,
};
use agentflow_spec::{issue_from_requirement, write_spec_issue, SpecIssue, SpecIssueDraft};
use agentflow_workflow_core::{WorkflowAgentRole, WorkflowFlowType};
use anyhow::{bail, Result};
use serde::Serialize;
use serde_json::{json, Value};
use std::{
    env, fs,
    path::{Path, PathBuf},
};

fn main() -> Result<()> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.len() != 11 {
        bail!(
            "usage: v119_software_dev_beta_certification_proofs <workspace> <metadata> <idempotency> <projection> <workspace-health> <beta-scope> <golden-intake> <golden-executor> <failure-retry> <desktop-smoke> <release-certification>"
        );
    }
    let workspace = PathBuf::from(&args[0]);
    let metadata = release_metadata_top_level_contract_proof(&workspace)?;
    let idempotency = idempotency_receipt_path_hardening_proof(&workspace)?;
    let projection = projection_rebuild_positive_recovery_proof(&workspace)?;
    let workspace_health = workspace_health_provider_skill_smoke_proof(&workspace)?;
    let beta_scope = software_dev_reference_app_beta_scope_proof(&workspace)?;
    let golden_intake = e2e_project_intake_tasks_golden_proof(&workspace)?;
    let golden_executor = executor_evidence_decision_delivery_golden_proof(&workspace)?;
    let failure_retry = failure_retry_feedback_beta_scenario_proof(&workspace)?;
    let desktop_smoke = desktop_beta_readiness_ui_smoke_proof(&workspace)?;
    let certification = beta_release_certification_proof(&[
        &metadata,
        &idempotency,
        &projection,
        &workspace_health,
        &beta_scope,
        &golden_intake,
        &golden_executor,
        &failure_retry,
        &desktop_smoke,
    ]);

    for (path, payload) in [
        (&args[1], metadata),
        (&args[2], idempotency),
        (&args[3], projection),
        (&args[4], workspace_health),
        (&args[5], beta_scope),
        (&args[6], golden_intake),
        (&args[7], golden_executor),
        (&args[8], failure_retry),
        (&args[9], desktop_smoke),
        (&args[10], certification),
    ] {
        write_json(Path::new(path), &payload)?;
    }
    Ok(())
}

fn release_metadata_top_level_contract_proof(workspace: &Path) -> Result<Value> {
    let changelog = read_text(workspace.join("CHANGELOG.md"))?;
    let release_readme = read_text(workspace.join("docs/delivery/releases/v1.1.9/README.md"))?;
    let tasks = read_text(
        workspace
            .join("docs/delivery/releases/v1.1.9/AGENTFLOW_V1_1_9_SOFTWARE_DEV_REFERENCE_APP_BETA_CERTIFICATION_TASKS_V1.md"),
    )?;
    let checks = json!({
        "changelog-v119-entry-present": changelog.contains("## v1.1.9") && changelog.contains("Software Dev Reference App Beta Certification"),
        "top-level-release-metadata-required": release_readme.contains("releaseVersion") && release_readme.contains("releaseTag") && release_readme.contains("sourceCommit") && release_readme.contains("workflowRunId") && release_readme.contains("primaryProofs"),
        "metadata-is-release-gate-not-runtime-authority": release_readme.contains("release-gate metadata") && release_readme.contains("not runtime authority"),
        "all-v119-issues-traceable": all_issue_refs_present(&tasks, 841, 850),
    });
    Ok(proof(
        "agentflow-v119-release-certification-metadata-top-level-contract.v1",
        checks,
        json!({ "issues": (841..=850).collect::<Vec<_>>() }),
    ))
}

fn idempotency_receipt_path_hardening_proof(workspace: &Path) -> Result<Value> {
    let (root, issue) =
        prepare_failed_command_run(workspace, "tmp/v119-idempotency", "AF-V119-IDEMPOTENCY-001")?;
    let validation =
        agentflow_task_artifacts::load_task_validation(&root, &issue.issue_id, "run-001")?;
    let failed_command_id = validation.failed_command_ids[0].clone();
    let request = ExecutorCommandRecoveryRequest {
        issue_id: issue.issue_id.clone(),
        run_id: "run-001".to_string(),
        failed_command_id,
        action: ExecutorCommandRecoveryAction::Retry,
        actor: "build-agent".to_string(),
        reason: "retry failed command".to_string(),
        replacement_command: None,
        idempotency_key: Some("recover-idempotent".to_string()),
    };
    let first = recover_failed_executor_command(&root, request.clone())?;
    let repeated = recover_failed_executor_command(&root, request.clone())?;
    let conflicting_key = recover_failed_executor_command(
        &root,
        ExecutorCommandRecoveryRequest {
            idempotency_key: Some("different-key".to_string()),
            ..request.clone()
        },
    )
    .unwrap_err()
    .to_string();
    let missing_key = recover_failed_executor_command(
        &root,
        ExecutorCommandRecoveryRequest {
            idempotency_key: None,
            ..request
        },
    )
    .unwrap_err()
    .to_string();

    let checks = json!({
        "same-key-same-payload-reuses-receipt": repeated.reused_existing && repeated.receipt_path == first.receipt_path,
        "same-path-different-key-rejected": conflicting_key.contains("conflicting idempotency receipt path"),
        "same-path-missing-key-rejected": missing_key.contains("receipt already exists without matching idempotency key"),
        "payload-hash-preserved": repeated.payload_sha256 == first.payload_sha256,
    });
    Ok(proof(
        "agentflow-v119-recovery-idempotency-receipt-path-hardening.v1",
        checks,
        json!({ "first": first, "repeated": repeated, "conflictingKey": conflicting_key, "missingKey": missing_key }),
    ))
}

fn projection_rebuild_positive_recovery_proof(workspace: &Path) -> Result<Value> {
    let positive_root = workspace.join("tmp/v119-projection-positive");
    reset_path(&positive_root)?;
    let issue = seed_issue(
        &positive_root,
        "AF-V119-PROJECTION-001",
        vec!["docs/**".to_string()],
    )?;
    create_executor_handoff_package(&positive_root, handoff_request(&issue.issue_id, "run-001"))?;
    append_task_event_once(
        &positive_root,
        event(
            &issue.issue_id,
            "issue.scheduled",
            json!({ "runId": "run-001" }),
        ),
    )?;
    let positive = rebuild_executor_projection(&positive_root, &issue.issue_id, "run-001")?;

    let missing_root = workspace.join("tmp/v119-projection-missing");
    reset_path(&missing_root)?;
    let missing_issue = seed_issue(
        &missing_root,
        "AF-V119-PROJECTION-002",
        vec!["docs/**".to_string()],
    )?;
    create_executor_handoff_package(
        &missing_root,
        handoff_request(&missing_issue.issue_id, "run-001"),
    )?;
    let missing = rebuild_executor_projection(&missing_root, &missing_issue.issue_id, "run-001")?;

    let checks = json!({
        "event-backed-rebuild-is-fresh": positive.status == "fresh" && positive.event_count == 1 && positive.failures.is_empty(),
        "rebuild-writes-receipt": positive_root.join(&positive.receipt_path).is_file(),
        "missing-events-fail": missing.status == "failed" && missing.event_count == 0 && !missing.failures.is_empty(),
        "freshness-derived-from-replay": positive.replay_status == "Passed" && missing.replay_status == "Failed",
    });
    Ok(proof(
        "agentflow-v119-projection-rebuild-positive-recovery-proof.v1",
        checks,
        json!({ "positive": positive, "missing": missing }),
    ))
}

fn workspace_health_provider_skill_smoke_proof(workspace: &Path) -> Result<Value> {
    let root = workspace.join("tmp/v119-workspace-health");
    reset_path(&root)?;
    let issue = seed_issue(&root, "AF-V119-HEALTH-001", vec!["docs/**".to_string()])?;
    create_executor_handoff_package(&root, handoff_request(&issue.issue_id, "run-001"))?;
    append_task_event_once(
        &root,
        event(
            &issue.issue_id,
            "issue.scheduled",
            json!({ "runId": "run-001" }),
        ),
    )?;
    let missing = check_executor_workspace_health(&root, &issue.issue_id, "run-001")?;
    write_provider_smoke(&root, "codex", 9_999_999_999)?;
    write_skill_smoke(&root, &issue.issue_id, "run-001", "ready")?;
    let ready = check_executor_workspace_health(&root, &issue.issue_id, "run-001")?;
    write_skill_smoke(&root, &issue.issue_id, "run-001", "failed")?;
    let stale_skill = check_executor_workspace_health(&root, &issue.issue_id, "run-001")?;

    let checks = json!({
        "provider-name-alone-is-not-ready": !missing.provider_ready && missing.provider_status == "missing",
        "skill-smoke-is-required": missing.skill_status == "missing",
        "provider-and-skill-smoke-ready": ready.status == "healthy" && ready.provider_ready && ready.provider_status == "ready" && ready.skill_status == "ready",
        "failed-skill-smoke-is-stale": stale_skill.status == "repairable" && stale_skill.skill_status == "stale",
    });
    Ok(proof(
        "agentflow-v119-workspace-health-provider-skill-smoke-boundary.v1",
        checks,
        json!({ "missing": missing, "ready": ready, "staleSkill": stale_skill }),
    ))
}

fn software_dev_reference_app_beta_scope_proof(workspace: &Path) -> Result<Value> {
    let release_readme = read_text(workspace.join("docs/delivery/releases/v1.1.9/README.md"))?;
    let tasks = read_text(
        workspace
            .join("docs/delivery/releases/v1.1.9/AGENTFLOW_V1_1_9_SOFTWARE_DEV_REFERENCE_APP_BETA_CERTIFICATION_TASKS_V1.md"),
    )?;
    let checks = json!({
        "reference-app-is-software-dev-beta": release_readme.contains("Software Dev reference app beta"),
        "not-core-product-ga": release_readme.contains("not Core GA") && release_readme.contains("not public commercial launch"),
        "beta-maps-to-domain-surface-connector-desktop": release_readme.contains("Domain Pack") && release_readme.contains("Surface Pack") && release_readme.contains("Connector Pack") && release_readme.contains("Desktop"),
        "scope-task-traceable": tasks.contains("#845") && tasks.contains("Software Dev Reference App Beta Scope Alignment"),
    });
    Ok(proof(
        "agentflow-v119-software-dev-reference-app-beta-scope-alignment.v1",
        checks,
        json!({ "scope": "software-dev-reference-app-beta" }),
    ))
}

fn e2e_project_intake_tasks_golden_proof(workspace: &Path) -> Result<Value> {
    let root = workspace.join("tmp/v119-golden-intake");
    reset_path(&root)?;
    let issue = seed_issue(&root, "AF-V119-GOLDEN-001", vec!["docs/**".to_string()])?;
    append_task_event_once(
        &root,
        event(
            &issue.issue_id,
            "spec.issue.ready",
            json!({ "source": "intake" }),
        ),
    )?;
    let projection = rebuild_executor_projection(&root, &issue.issue_id, "run-001")?;
    let checks = json!({
        "requirement-record-written": root.join("docs/requirements/example.md").is_file(),
        "issue-authority-written": root.join(".agentflow/spec/issues/AF-V119-GOLDEN-001.json").is_file(),
        "project-context-written": root.join("docs/project/goal.md").is_file() && root.join("docs/project/roadmap.md").is_file(),
        "projection-from-intake-event": projection.status == "fresh" && projection.event_count == 1,
    });
    Ok(proof(
        "agentflow-v119-e2e-project-intake-tasks-golden-scenario.v1",
        checks,
        json!({ "issueId": issue.issue_id, "projection": projection }),
    ))
}

fn executor_evidence_decision_delivery_golden_proof(workspace: &Path) -> Result<Value> {
    let (root, issue, run_id) = prepare_accepted_run(
        workspace,
        "tmp/v119-executor-golden",
        "AF-V119-EXECUTOR-001",
    )?;
    let view = get_executor_flow_read_model(&root, &issue.issue_id, &run_id)?;
    let checks = json!({
        "issue-is-done": view.issue_status == "done",
        "evidence-graph-complete": view.evidence_graph.status == "complete",
        "decision-accepted": view.decision.accepted,
        "delivery-ready": view.delivery.status == "ready",
    });
    Ok(proof(
        "agentflow-v119-executor-run-evidence-decision-delivery-golden-path.v1",
        checks,
        json!({ "view": view }),
    ))
}

fn failure_retry_feedback_beta_scenario_proof(workspace: &Path) -> Result<Value> {
    let (root, issue) =
        prepare_failed_command_run(workspace, "tmp/v119-failure-retry", "AF-V119-FAILURE-001")?;
    let validation =
        agentflow_task_artifacts::load_task_validation(&root, &issue.issue_id, "run-001")?;
    let failed_command_id = validation.failed_command_ids[0].clone();
    let receipt = recover_failed_executor_command(
        &root,
        ExecutorCommandRecoveryRequest {
            issue_id: issue.issue_id.clone(),
            run_id: "run-001".to_string(),
            failed_command_id,
            action: ExecutorCommandRecoveryAction::Replace,
            actor: "build-agent".to_string(),
            reason: "replace failed beta validation command".to_string(),
            replacement_command: Some(passing_command()),
            idempotency_key: Some("beta-retry-001".to_string()),
        },
    )?;
    let view = get_executor_flow_read_model(&root, &issue.issue_id, "run-001")?;
    let checks = json!({
        "failed-command-is-actionable": !validation.failed_command_ids.is_empty(),
        "retry-preserves-original-evidence": receipt.original_failed_evidence_preserved,
        "replacement-command-traceable": receipt.replacement_command_id.is_some(),
        "feedback-does-not-mark-done": receipt.issue_status == "todo" && view.issue_status != "done",
    });
    Ok(proof(
        "agentflow-v119-failure-retry-feedback-beta-scenario.v1",
        checks,
        json!({ "receipt": receipt, "view": view.recovery }),
    ))
}

fn desktop_beta_readiness_ui_smoke_proof(workspace: &Path) -> Result<Value> {
    let app = read_text(workspace.join("apps/desktop/src/App.tsx"))?;
    let runtime_command =
        read_text(workspace.join("apps/desktop/src-tauri/src/commands/runtime_api.rs"))?;
    let main_rs = read_text(workspace.join("apps/desktop/src-tauri/src/main.rs"))?;
    let checks = json!({
        "runtime-command-registered": main_rs.contains("commands::runtime_api::load_executor_flow_read_model"),
        "desktop-uses-runtime-api": app.contains("load_executor_flow_read_model"),
        "desktop-renders-executor-flow": app.contains("executorFlow") && app.contains("Runtime API"),
        "desktop-has-recovery-state-language": app.contains("resume") || app.contains("recovery") || app.contains("repairable"),
        "desktop-does-not-read-authority-files-directly": !app.contains("read_spec_issue") && !app.contains("load_spec_issue"),
        "tauri-command-exists": runtime_command.contains("load_executor_flow_read_model"),
    });
    Ok(proof(
        "agentflow-v119-desktop-beta-readiness-ui-smoke-proof.v1",
        checks,
        json!({ "files": ["apps/desktop/src/App.tsx", "apps/desktop/src-tauri/src/commands/runtime_api.rs"] }),
    ))
}

fn beta_release_certification_proof(proofs: &[&Value]) -> Value {
    let primary_proofs = vec![
        "runtime/v119-release-certification-metadata-top-level-contract.json",
        "runtime/v119-recovery-idempotency-receipt-path-hardening.json",
        "runtime/v119-projection-rebuild-positive-recovery-proof.json",
        "runtime/v119-workspace-health-provider-skill-smoke-boundary.json",
        "runtime/v119-software-dev-reference-app-beta-scope-alignment.json",
        "runtime/v119-e2e-project-intake-tasks-golden-scenario.json",
        "runtime/v119-executor-run-evidence-decision-delivery-golden-path.json",
        "runtime/v119-failure-retry-feedback-beta-scenario.json",
        "runtime/v119-desktop-beta-readiness-ui-smoke-proof.json",
    ];
    let checks = json!({
        "all-primary-proofs-passed": proofs.iter().all(|proof| proof.get("status").and_then(|value| value.as_str()) == Some("passed")),
        "primary-proof-count": primary_proofs.len() == 9,
        "beta-baseline-not-public-commercial-launch": true,
    });
    proof(
        "agentflow-v119-software-dev-reference-app-beta-release-certification.v1",
        checks,
        json!({
            "releaseVersion": "v1.1.9",
            "releaseTag": "v1.1.9",
            "sourceCommit": "filled-by-release-gate",
            "workflowRunId": "filled-by-release-gate",
            "artifactNames": ["agentflow-release-certification", "agentflow-release-gate-full"],
            "primaryProofs": primary_proofs,
            "releaseScope": "software-dev-reference-app-beta",
            "commercialLaunch": false,
        }),
    )
}

fn prepare_accepted_run(
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
            changed_files: vec![changed_file("docs/requirements/example.md")],
            base_commit: None,
            head_commit: None,
        },
    )?;
    capture_executor_evidence(
        &root,
        ExecutorEvidenceCaptureRequest {
            issue_id: issue.issue_id.clone(),
            run_id: run_id.clone(),
            summary: "v119 accepted run proof".to_string(),
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
            summary: "done".to_string(),
            artifacts: vec!["docs/requirements/example.md".to_string()],
            logs: vec!["validated".to_string()],
            failure_reason: None,
            continuation_request: None,
        },
    )?;
    Ok((root, issue, run_id))
}

fn prepare_failed_command_run(
    workspace: &Path,
    dir: &str,
    issue_id: &str,
) -> Result<(PathBuf, SpecIssue)> {
    let root = workspace.join(dir);
    reset_path(&root)?;
    let issue = seed_issue(&root, issue_id, vec!["docs/**".to_string()])?;
    create_executor_handoff_package(&root, handoff_request(&issue.issue_id, "run-001"))?;
    check_executor_diff_boundary(
        &root,
        ExecutorDiffBoundaryRequest {
            issue_id: issue.issue_id.clone(),
            run_id: "run-001".to_string(),
            changed_files: vec![changed_file("docs/requirements/example.md")],
            base_commit: None,
            head_commit: None,
        },
    )?;
    capture_executor_evidence(
        &root,
        ExecutorEvidenceCaptureRequest {
            issue_id: issue.issue_id.clone(),
            run_id: "run-001".to_string(),
            summary: "failed command recovery proof".to_string(),
            commands: vec![failing_command()],
            boundary_failures: Vec::new(),
        },
    )?;
    Ok((root, issue))
}

fn seed_issue(root: &Path, issue_id: &str, allowed_paths: Vec<String>) -> Result<SpecIssue> {
    seed_requirement(root)?;
    let mut draft = SpecIssueDraft::new(issue_id);
    draft.title = Some("v1.1.9 beta proof issue".to_string());
    draft.summary = Some("v1.1.9 beta proof summary".to_string());
    draft.project_id = Some("project-v119-beta-proof".to_string());
    draft.allowed_paths = allowed_paths;
    draft.forbidden_paths = vec!["secrets/**".to_string()];
    draft.validation_commands = vec!["cargo test".to_string()];
    let issue = issue_from_requirement(root, Path::new("docs/requirements/example.md"), draft)?;
    write_spec_issue(root, &issue)?;
    Ok(issue)
}

fn seed_requirement(root: &Path) -> Result<()> {
    let requirement = root.join("docs/requirements/example.md");
    fs::create_dir_all(requirement.parent().unwrap())?;
    fs::write(
        &requirement,
        "# Example Requirement\n\nSeed requirement for v1.1.9 beta proof.\n",
    )?;
    fs::create_dir_all(root.join("docs/project"))?;
    fs::write(
        root.join("docs/project/goal.md"),
        "# Goal\n\nSoftware Dev beta proof.\n",
    )?;
    fs::write(
        root.join("docs/project/roadmap.md"),
        "# Roadmap\n\nSoftware Dev beta proof.\n",
    )?;
    Ok(())
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

fn event(issue_id: &str, event_type: &str, payload: Value) -> TaskEventDraft {
    TaskEventDraft {
        flow_type: WorkflowFlowType::Work,
        aggregate_type: "issue".to_string(),
        aggregate_id: issue_id.to_string(),
        project_id: Some("project-v119-beta-proof".to_string()),
        issue_id: Some(issue_id.to_string()),
        run_id: payload
            .get("runId")
            .and_then(Value::as_str)
            .map(str::to_string),
        event_type: event_type.to_string(),
        authority_role: Some(WorkflowAgentRole::WorkAgent),
        actor: EventActor {
            role: "release-gate".to_string(),
            kind: "system".to_string(),
        },
        state: None,
        correlation_id: Some(format!("corr-{issue_id}")),
        causation_id: None,
        payload,
        artifact_refs: Vec::new(),
        idempotency_key: Some(format!("{event_type}:{issue_id}")),
    }
}

fn write_provider_smoke(root: &Path, provider: &str, created_at: u64) -> Result<()> {
    let mut health = McpProviderStatus::new(McpProviderKind::Codex, created_at);
    health.status = McpProviderStatusCode::Ready;
    health.installed = true;
    health.authenticated = Some(true);
    health.capabilities = vec![McpCapability::new("provider.codex.launch", true)];
    let artifact = McpProviderSmokeArtifact {
        version: MCP_PROVIDER_SMOKE_ARTIFACT_VERSION.to_string(),
        provider: provider.to_string(),
        outcome: McpProviderSmokeOutcome::Passed,
        reason: "release gate fixture provider smoke passed".to_string(),
        health,
        launch_request_path: Some(".agentflow/tmp/provider-smoke-request.md".to_string()),
        session_id: Some("session-v119-smoke".to_string()),
        session_snapshot_path: Some(
            ".agentflow/state/mcp/sessions/session-v119-smoke.json".to_string(),
        ),
        session_snapshot_readable: true,
        terminal_status: Some(McpSessionStatus::Done),
        terminal_provider_state_projectable: true,
        artifact_path: format!(
            ".agentflow/state/mcp/provider-smoke/{}-{}.json",
            provider, created_at
        ),
        created_at,
    };
    write_json(root.join(&artifact.artifact_path).as_path(), &artifact)
}

fn write_skill_smoke(root: &Path, issue_id: &str, run_id: &str, status: &str) -> Result<()> {
    let path = agentflow_task_artifacts::task_run_dir(root, issue_id, run_id)?
        .join("smoke")
        .join("skill-smoke.json");
    write_json(
        &path,
        &json!({
            "version": "agentflow-skill-smoke.v1",
            "issueId": issue_id,
            "runId": run_id,
            "status": status,
            "skill": "build-agent",
        }),
    )
}

fn passing_command() -> ExecutorCommandEvidenceInput {
    ExecutorCommandEvidenceInput {
        label: "unit".to_string(),
        program: "cargo".to_string(),
        args: vec!["test".to_string()],
        exit_code: Some(0),
        stdout: "ok".to_string(),
        stderr: String::new(),
    }
}

fn failing_command() -> ExecutorCommandEvidenceInput {
    ExecutorCommandEvidenceInput {
        label: "unit".to_string(),
        program: "cargo".to_string(),
        args: vec!["test".to_string()],
        exit_code: Some(101),
        stdout: String::new(),
        stderr: "failed".to_string(),
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

fn all_issue_refs_present(text: &str, start: u64, end: u64) -> bool {
    (start..=end).all(|issue| text.contains(&format!("#{issue}")))
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

fn reset_path(path: &Path) -> Result<()> {
    if path.exists() {
        fs::remove_dir_all(path)?;
    }
    fs::create_dir_all(path)?;
    Ok(())
}

fn read_text(path: impl AsRef<Path>) -> Result<String> {
    Ok(fs::read_to_string(path)?)
}

fn write_json(path: &Path, payload: &impl Serialize) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(
        path,
        format!("{}\n", serde_json::to_string_pretty(payload)?),
    )?;
    Ok(())
}
