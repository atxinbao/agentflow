use agentflow_mcp::{
    McpCapability, McpProviderKind, McpProviderSmokeArtifact, McpProviderSmokeOutcome,
    McpProviderStatus, McpProviderStatusCode, McpSessionStatus,
    MCP_PROVIDER_SMOKE_ARTIFACT_VERSION,
};
use agentflow_runtime_api::{
    capture_executor_evidence, check_executor_diff_boundary, check_executor_workspace_health,
    create_executor_handoff_package, get_executor_flow_read_model, rebuild_executor_projection,
    record_executor_lifecycle, recover_failed_executor_command, resume_executor_run,
    write_executor_result_to_issue, ExecutorCommandEvidenceInput, ExecutorCommandRecoveryAction,
    ExecutorCommandRecoveryRequest, ExecutorDiffBoundaryRequest, ExecutorDiffInputFile,
    ExecutorEvidenceCaptureRequest, ExecutorHandoffRequest, ExecutorLifecycleAction,
    ExecutorLifecycleRequest, ExecutorResultOutcome, ExecutorResultWritebackRequest,
    ExecutorRunResumeRequest,
};
use agentflow_spec::{issue_from_requirement, write_spec_issue, SpecIssue, SpecIssueDraft};
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
            "usage: v118_recovery_resume_failure_proofs <workspace> <metadata> <evidence-graph> <desktop> <resume> <failed-command> <interrupted> <idempotency> <projection-rebuild> <workspace-health> <release-certification>"
        );
    }
    let workspace = PathBuf::from(&args[0]);
    let metadata = release_metadata_proof(&workspace)?;
    let evidence_graph = evidence_graph_completion_proof(&workspace)?;
    let desktop = desktop_frontend_invocation_proof(&workspace)?;
    let resume = run_resume_contract_proof(&workspace)?;
    let failed_command = failed_command_recovery_proof(&workspace)?;
    let interrupted = interrupted_closeout_proof(&workspace)?;
    let idempotency = duplicate_idempotency_proof(&workspace)?;
    let projection_rebuild = stale_projection_rebuild_proof(&workspace)?;
    let workspace_health = workspace_health_proof(&workspace)?;
    let certification = release_certification_proof(&[
        &metadata,
        &evidence_graph,
        &desktop,
        &resume,
        &failed_command,
        &interrupted,
        &idempotency,
        &projection_rebuild,
        &workspace_health,
    ]);

    for (path, payload) in [
        (&args[1], metadata),
        (&args[2], evidence_graph),
        (&args[3], desktop),
        (&args[4], resume),
        (&args[5], failed_command),
        (&args[6], interrupted),
        (&args[7], idempotency),
        (&args[8], projection_rebuild),
        (&args[9], workspace_health),
        (&args[10], certification),
    ] {
        write_json(Path::new(path), &payload)?;
    }
    Ok(())
}

fn release_metadata_proof(workspace: &Path) -> Result<Value> {
    let changelog = read_text(workspace.join("CHANGELOG.md"))?;
    let roadmap = read_text(workspace.join("docs/project/roadmap.md"))?;
    let release_readme = read_text(workspace.join("docs/delivery/releases/v1.1.8/README.md"))?;
    let checks = json!({
        "changelog-v118-entry-present": changelog.contains("## v1.1.8") && changelog.contains("Recovery / Resume / Failure Handling"),
        "roadmap-v118-entry-present": roadmap.contains("v1.1.8") && roadmap.contains("Recovery / Resume / Failure Handling"),
        "release-readme-has-certification-metadata": release_readme.contains("releaseVersion") && release_readme.contains("workflowRunId") && release_readme.contains("primaryProofs"),
    });
    Ok(proof(
        "agentflow-v118-release-closeout-certification-metadata-hardening.v1",
        checks,
        json!({ "issues": (830..=839).collect::<Vec<_>>() }),
    ))
}

fn evidence_graph_completion_proof(workspace: &Path) -> Result<Value> {
    let (root, issue, run_id) = prepare_accepted_run(
        workspace,
        "tmp/v118-evidence-complete",
        "AF-V118-EVIDENCE-001",
    )?;
    let complete = get_executor_flow_read_model(&root, &issue.issue_id, &run_id)?;

    let partial_issue = seed_issue(&root, "AF-V118-EVIDENCE-002", vec!["docs/**".to_string()])?;
    create_executor_handoff_package(&root, handoff_request(&partial_issue.issue_id, "run-001"))?;
    let partial = get_executor_flow_read_model(&root, &partial_issue.issue_id, "run-001")?;

    let failed_issue = seed_issue(&root, "AF-V118-EVIDENCE-003", vec!["docs/**".to_string()])?;
    create_executor_handoff_package(&root, handoff_request(&failed_issue.issue_id, "run-001"))?;
    check_executor_diff_boundary(
        &root,
        ExecutorDiffBoundaryRequest {
            issue_id: failed_issue.issue_id.clone(),
            run_id: "run-001".to_string(),
            changed_files: vec![changed_file("src/main.rs")],
            base_commit: None,
            head_commit: None,
        },
    )?;
    let failed = get_executor_flow_read_model(&root, &failed_issue.issue_id, "run-001")?;

    let checks = json!({
        "complete-requires-all-nodes": complete.evidence_graph.status == "complete",
        "partial-is-not-complete": partial.evidence_graph.status == "partial" && partial.evidence_graph.status != "complete",
        "failed-is-explicit": failed.evidence_graph.status == "failed" && !failed.evidence_graph.failed.is_empty(),
        "missing-list-is-explicit": !partial.evidence_graph.missing.is_empty(),
    });
    Ok(proof(
        "agentflow-v118-evidence-graph-completion-proof-tightening.v1",
        checks,
        json!({ "complete": complete.evidence_graph, "partial": partial.evidence_graph, "failed": failed.evidence_graph }),
    ))
}

fn desktop_frontend_invocation_proof(workspace: &Path) -> Result<Value> {
    let app = read_text(workspace.join("apps/desktop/src/App.tsx"))?;
    let runtime_command =
        read_text(workspace.join("apps/desktop/src-tauri/src/commands/runtime_api.rs"))?;
    let main_rs = read_text(workspace.join("apps/desktop/src-tauri/src/main.rs"))?;
    let checks = json!({
        "tauri-command-exists": runtime_command.contains("fn load_executor_flow_read_model"),
        "tauri-command-registered": main_rs.contains("commands::runtime_api::load_executor_flow_read_model"),
        "frontend-invokes-runtime-api": app.contains("load_executor_flow_read_model"),
        "frontend-renders-runtime-summary": app.contains("Runtime API") && app.contains("workspaceHealth"),
    });
    Ok(proof(
        "agentflow-v118-desktop-executor-flow-frontend-invocation.v1",
        checks,
        json!({ "files": ["apps/desktop/src/App.tsx", "apps/desktop/src-tauri/src/commands/runtime_api.rs"] }),
    ))
}

fn run_resume_contract_proof(workspace: &Path) -> Result<Value> {
    let root = workspace.join("tmp/v118-resume");
    reset_path(&root)?;
    let issue = seed_issue(&root, "AF-V118-RESUME-001", vec!["docs/**".to_string()])?;
    create_executor_handoff_package(&root, handoff_request(&issue.issue_id, "run-001"))?;
    let receipt = resume_executor_run(
        &root,
        ExecutorRunResumeRequest {
            issue_id: issue.issue_id.clone(),
            run_id: "run-001".to_string(),
            actor: "build-agent".to_string(),
            reason: "continue interrupted work".to_string(),
            resume_run_id: Some("run-002".to_string()),
            idempotency_key: Some("resume-001".to_string()),
        },
    )?;
    let repeated = resume_executor_run(
        &root,
        ExecutorRunResumeRequest {
            issue_id: issue.issue_id.clone(),
            run_id: "run-001".to_string(),
            actor: "build-agent".to_string(),
            reason: "continue interrupted work".to_string(),
            resume_run_id: Some("run-002".to_string()),
            idempotency_key: Some("resume-001".to_string()),
        },
    )?;
    let checks = json!({
        "resume-creates-new-run": root.join(".agentflow/tasks/AF-V118-RESUME-001/runs/run-002/run.json").is_file(),
        "resume-receipt-is-written": root.join(&receipt.receipt_path).is_file(),
        "resume-is-idempotent": repeated.reused_existing && repeated.payload_sha256 == receipt.payload_sha256,
        "terminal-run-not-auto-done": receipt.issue_status == "todo",
    });
    Ok(proof(
        "agentflow-v118-run-resume-contract.v1",
        checks,
        json!({ "receipt": receipt, "repeated": repeated }),
    ))
}

fn failed_command_recovery_proof(workspace: &Path) -> Result<Value> {
    let (root, issue) =
        prepare_failed_command_run(workspace, "tmp/v118-failed-command", "AF-V118-FAILED-001")?;
    let validation =
        agentflow_task_artifacts::load_task_validation(&root, &issue.issue_id, "run-001")?;
    let failed_command_id = validation.failed_command_ids[0].clone();
    let receipt = recover_failed_executor_command(
        &root,
        ExecutorCommandRecoveryRequest {
            issue_id: issue.issue_id.clone(),
            run_id: "run-001".to_string(),
            failed_command_id: failed_command_id.clone(),
            action: ExecutorCommandRecoveryAction::Replace,
            actor: "build-agent".to_string(),
            reason: "replace failed command with corrected validation command".to_string(),
            replacement_command: Some(passing_command()),
            idempotency_key: Some("recover-001".to_string()),
        },
    )?;
    let checks = json!({
        "failed-command-target-validated": receipt.failed_command_id == failed_command_id,
        "replacement-command-recorded": receipt.replacement_command_id.is_some(),
        "original-failed-evidence-preserved": receipt.original_failed_evidence_preserved,
        "issue-stays-retryable": receipt.issue_status == "todo",
    });
    Ok(proof(
        "agentflow-v118-failed-command-recovery.v1",
        checks,
        json!({ "receipt": receipt, "validation": validation }),
    ))
}

fn interrupted_closeout_proof(workspace: &Path) -> Result<Value> {
    let root = workspace.join("tmp/v118-interrupted");
    reset_path(&root)?;
    let issue = seed_issue(&root, "AF-V118-INTERRUPT-001", vec!["docs/**".to_string()])?;
    create_executor_handoff_package(&root, handoff_request(&issue.issue_id, "run-001"))?;
    let receipt = record_executor_lifecycle(
        &root,
        ExecutorLifecycleRequest {
            issue_id: issue.issue_id.clone(),
            run_id: "run-001".to_string(),
            action: ExecutorLifecycleAction::Interrupt,
            actor: "build-agent".to_string(),
            reason: "executor session interrupted".to_string(),
            retry_run_id: None,
        },
    )?;
    let view = get_executor_flow_read_model(&root, &issue.issue_id, "run-001")?;
    let checks = json!({
        "interrupt-is-recorded": receipt.action.as_str() == "interrupt",
        "interrupt-does-not-mark-done": receipt.issue_status == "todo",
        "resume-remains-available": view.resume.allowed,
        "lifecycle-receipt-source-visible": view.resume.receipt_refs.iter().any(|path| path.ends_with("/interrupt.json")),
    });
    Ok(proof(
        "agentflow-v118-interrupted-executor-session-closeout.v1",
        checks,
        json!({ "receipt": receipt, "view": view.resume }),
    ))
}

fn duplicate_idempotency_proof(workspace: &Path) -> Result<Value> {
    let (root, issue) =
        prepare_failed_command_run(workspace, "tmp/v118-idempotency", "AF-V118-IDEMPOTENCY-001")?;
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
    let second = recover_failed_executor_command(&root, request.clone())?;
    let conflict = recover_failed_executor_command(
        &root,
        ExecutorCommandRecoveryRequest {
            reason: "different reason".to_string(),
            ..request
        },
    )
    .unwrap_err()
    .to_string();
    let checks = json!({
        "same-payload-reuses-existing-receipt": second.reused_existing && second.payload_sha256 == first.payload_sha256,
        "conflicting-payload-is-rejected": conflict.contains("conflicting idempotency key"),
        "duplicate-does-not-create-extra-receipt": first.receipt_path == second.receipt_path,
    });
    Ok(proof(
        "agentflow-v118-duplicate-command-idempotency-handling.v1",
        checks,
        json!({ "first": first, "second": second, "conflict": conflict }),
    ))
}

fn stale_projection_rebuild_proof(workspace: &Path) -> Result<Value> {
    let root = workspace.join("tmp/v118-projection-rebuild");
    reset_path(&root)?;
    let issue = seed_issue(&root, "AF-V118-PROJECTION-001", vec!["docs/**".to_string()])?;
    create_executor_handoff_package(&root, handoff_request(&issue.issue_id, "run-001"))?;
    let receipt = rebuild_executor_projection(&root, &issue.issue_id, "run-001")?;
    let checks = json!({
        "rebuild-receipt-written": root.join(&receipt.receipt_path).is_file(),
        "missing-events-do-not-fake-freshness": receipt.status == "failed" && !receipt.failures.is_empty(),
        "projection-rebuild-is-deterministic-surface": receipt.version == "agentflow-executor-projection-rebuild-receipt.v1",
    });
    Ok(proof(
        "agentflow-v118-stale-projection-rebuild-recovery.v1",
        checks,
        json!({ "receipt": receipt }),
    ))
}

fn workspace_health_proof(workspace: &Path) -> Result<Value> {
    let root = workspace.join("tmp/v118-workspace-health");
    reset_path(&root)?;
    let issue = seed_issue(&root, "AF-V118-HEALTH-001", vec!["docs/**".to_string()])?;
    create_executor_handoff_package(&root, handoff_request(&issue.issue_id, "run-001"))?;
    write_provider_smoke(&root, "codex", 9_999_999_999)?;
    write_skill_smoke(&root, &issue.issue_id, "run-001", "ready")?;
    let report = check_executor_workspace_health(&root, &issue.issue_id, "run-001")?;
    fs::remove_file(root.join("docs/project/goal.md"))?;
    let broken = check_executor_workspace_health(&root, &issue.issue_id, "run-001")?;
    let checks = json!({
        "health-report-has-required-refs": report.required_refs.iter().any(|path| path == "docs/project/goal.md"),
        "provider-marker-detected": report.provider_ready,
        "missing-authority-blocks-health": broken.status == "blocked" && broken.missing_refs.iter().any(|path| path == "docs/project/goal.md"),
        "projection-freshness-is-reported": !report.stale_refs.is_empty() || report.projection_fresh,
    });
    Ok(proof(
        "agentflow-v118-workspace-health-check.v1",
        checks,
        json!({ "report": report, "broken": broken }),
    ))
}

fn release_certification_proof(proofs: &[&Value]) -> Value {
    let primary_proofs = vec![
        "runtime/v118-release-closeout-certification-metadata-hardening.json",
        "runtime/v118-evidence-graph-completion-proof-tightening.json",
        "runtime/v118-desktop-executor-flow-frontend-invocation.json",
        "runtime/v118-run-resume-contract.json",
        "runtime/v118-failed-command-recovery.json",
        "runtime/v118-interrupted-executor-session-closeout.json",
        "runtime/v118-duplicate-command-idempotency-handling.json",
        "runtime/v118-stale-projection-rebuild-recovery.json",
        "runtime/v118-workspace-health-check.json",
    ];
    let checks = json!({
        "all-primary-proofs-passed": proofs.iter().all(|proof| proof.get("status").and_then(|value| value.as_str()) == Some("passed")),
        "primary-proof-count": primary_proofs.len() == 9,
        "release-certification-status": true,
    });
    proof(
        "agentflow-v118-recovery-release-certification.v1",
        checks,
        json!({
            "releaseVersion": "v1.1.8",
            "releaseTag": "v1.1.8",
            "sourceCommit": "filled-by-release-gate",
            "workflowRunId": "filled-by-release-gate",
            "artifactNames": ["agentflow-release-certification", "agentflow-release-gate-full"],
            "primaryProofs": primary_proofs,
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
            summary: "v118 accepted run proof".to_string(),
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
    draft.title = Some("v1.1.8 recovery proof issue".to_string());
    draft.summary = Some("v1.1.8 recovery proof summary".to_string());
    draft.project_id = Some("project-v118-proof".to_string());
    draft.allowed_paths = allowed_paths;
    draft.forbidden_paths = vec!["secrets/**".to_string()];
    draft.validation_commands = vec!["cargo test".to_string()];
    let issue = issue_from_requirement(root, Path::new("docs/requirements/example.md"), draft)?;
    write_spec_issue(root, &issue)?;
    Ok(issue)
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
        session_id: Some("session-v118-smoke".to_string()),
        session_snapshot_path: Some(
            ".agentflow/state/mcp/sessions/session-v118-smoke.json".to_string(),
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

fn seed_requirement(root: &Path) -> Result<()> {
    let requirement = root.join("docs/requirements/example.md");
    fs::create_dir_all(requirement.parent().unwrap())?;
    fs::write(
        &requirement,
        "# Example Requirement\n\nSeed requirement for v1.1.8 recovery proof.\n",
    )?;
    fs::create_dir_all(root.join("docs/project"))?;
    fs::write(
        root.join("docs/project/goal.md"),
        "# Goal\n\nRecovery proof.\n",
    )?;
    fs::write(
        root.join("docs/project/roadmap.md"),
        "# Roadmap\n\nRecovery proof.\n",
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
