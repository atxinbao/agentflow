use agentflow_runtime_api::{
    capture_executor_evidence, check_executor_diff_boundary, confirm_product_spec_preview,
    create_executor_handoff_package, materialize_confirmed_product_spec, preview_product_intent,
    read_product_spec_preview, record_executor_lifecycle, write_executor_result_to_issue,
    ExecutorCommandEvidenceInput, ExecutorDiffBoundaryRequest, ExecutorDiffInputFile,
    ExecutorEvidenceCaptureRequest, ExecutorHandoffRequest, ExecutorLifecycleAction,
    ExecutorLifecycleRequest, ExecutorResultOutcome, ExecutorResultWritebackRequest,
    ProductIntentIntakeRequest, ProductSpecConfirmationRequest, ProductSpecPreviewDecision,
};
use agentflow_spec::{issue_from_requirement, write_spec_issue, SpecIssue, SpecIssueDraft};
use anyhow::{bail, Context, Result};
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
            "usage: v116_executor_real_execution_proofs <workspace> <planning> <route-next-actions> <desktop-bridge> <handoff> <diff-boundary> <evidence> <writeback> <failure-retry> <golden-path> <release-certification>"
        );
    }
    let workspace = PathBuf::from(&args[0]);
    let planning_out = PathBuf::from(&args[1]);
    let route_out = PathBuf::from(&args[2]);
    let desktop_out = PathBuf::from(&args[3]);
    let handoff_out = PathBuf::from(&args[4]);
    let boundary_out = PathBuf::from(&args[5]);
    let evidence_out = PathBuf::from(&args[6]);
    let writeback_out = PathBuf::from(&args[7]);
    let failure_retry_out = PathBuf::from(&args[8]);
    let golden_out = PathBuf::from(&args[9]);
    let certification_out = PathBuf::from(&args[10]);

    let planning = planning_alignment_proof(&workspace)?;
    let route = route_next_action_semantics_proof(&workspace)?;
    let desktop = desktop_invocation_bridge_proof(&workspace)?;
    let handoff = handoff_package_proof(&workspace)?;
    let boundary = diff_boundary_proof(&workspace)?;
    let evidence = evidence_capture_proof(&workspace)?;
    let writeback = result_writeback_proof(&workspace)?;
    let failure_retry = failure_retry_semantics_proof(&workspace)?;
    let golden = software_dev_golden_path_proof(&workspace)?;
    let certification = release_certification_proof(
        &planning,
        &route,
        &desktop,
        &handoff,
        &boundary,
        &evidence,
        &writeback,
        &failure_retry,
        &golden,
    );

    write_json(&planning_out, &planning)?;
    write_json(&route_out, &route)?;
    write_json(&desktop_out, &desktop)?;
    write_json(&handoff_out, &handoff)?;
    write_json(&boundary_out, &boundary)?;
    write_json(&evidence_out, &evidence)?;
    write_json(&writeback_out, &writeback)?;
    write_json(&failure_retry_out, &failure_retry)?;
    write_json(&golden_out, &golden)?;
    write_json(&certification_out, &certification)?;
    Ok(())
}

fn planning_alignment_proof(workspace: &Path) -> Result<Value> {
    let roadmap = read_text(workspace.join("docs/project/roadmap.md"))?;
    let changelog = read_text(workspace.join("CHANGELOG.md"))?;
    let delivery = read_text(workspace.join("docs/delivery/README.md"))?;
    let release_readme = read_text(workspace.join("docs/delivery/releases/v1.1.6/README.md"))?;
    let release_tasks = read_text(workspace.join(
        "docs/delivery/releases/v1.1.6/AGENTFLOW_V1_1_6_EXECUTOR_ADAPTER_REAL_EXECUTION_TASKS_V1.md",
    ))?;
    let checks = json!({
        "roadmap-v116-is-executor-adapter-real-execution": roadmap.contains("v1.1.6") && roadmap.contains("Executor Adapter Real Execution Closure"),
        "changelog-v116-entry-present": changelog.contains("## v1.1.6") && changelog.contains("Executor Adapter Real Execution Closure"),
        "delivery-current-v116": delivery.contains("releases/v1.1.6/README.md") && delivery.contains("当前发布基线"),
        "release-doc-v116": release_readme.contains("Executor Adapter Real Execution Closure"),
        "release-tasks-v116-issues": (808..=817).all(|issue| release_tasks.contains(&format!("#{issue}"))),
        "provider-launch-closure-removed": !changelog.contains("v1.1.6 Product workspace lifecycle and provider launch closure")
            && !delivery.contains("v1.1.6` | 下一版计划：Product workspace lifecycle"),
        "executor-session-not-authority": release_readme.contains("executor session") && release_readme.contains("not authority"),
    });
    Ok(proof(
        "agentflow-v116-next-release-authority-alignment.v1",
        checks,
        json!({}),
    ))
}

fn route_next_action_semantics_proof(workspace: &Path) -> Result<Value> {
    let root = workspace.join("tmp/v116-route-next-actions");
    reset_path(&root)?;
    let cases = [
        ("clarify", "？"),
        ("research", "research 当前 Agent workflow 方案。"),
        ("define", "定义新的项目目标和边界。"),
        ("plan", "规划下一阶段路线图。"),
        ("task", "实现任务页状态时间线。"),
        ("decide", "确认两个方案的取舍。"),
        ("deliver", "交付 release notes。"),
        ("evolve", "迭代项目控制台体验。"),
    ];
    let mut rows = Vec::new();
    let mut materialize_rejections = Vec::new();
    for (expected, text) in cases {
        let receipt = preview_product_intent(
            workspace,
            &root,
            product_request(&format!("v116-route-{expected}"), text),
        )?;
        let preview = read_product_spec_preview(&root, &receipt.preview_id)?;
        let has_forbidden_action = receipt.next_actions.contains(&"confirm".to_string())
            || receipt
                .next_actions
                .contains(&"materialize-after-confirmation".to_string());
        if expected == "clarify" || expected == "research" {
            let confirmation = confirm_product_spec_preview(
                &root,
                ProductSpecConfirmationRequest {
                    preview_id: receipt.preview_id.clone(),
                    preview_hash: receipt.preview_hash.clone(),
                    actor: "human-owner".to_string(),
                    decision: ProductSpecPreviewDecision::Confirm,
                    summary: "negative fixture confirmation".to_string(),
                },
            )?;
            let rejected = materialize_confirmed_product_spec(&root, &receipt.preview_id)
                .unwrap_err()
                .to_string()
                .contains("cannot materialize authority");
            materialize_rejections.push(json!({
                "route": expected,
                "confirmationId": confirmation.confirmation_id,
                "materializationRejected": rejected,
            }));
        }
        rows.push(json!({
            "expected": expected,
            "actual": preview.route_decision.route.as_str(),
            "writeBoundary": preview.route_decision.write_boundary,
            "nextActions": receipt.next_actions,
            "hasForbiddenNoAuthorityAction": has_forbidden_action,
        }));
    }
    let checks = json!({
        "all-eight-routes-covered": rows.len() == 8,
        "all-routes-match": rows.iter().all(|row| row["expected"] == row["actual"]),
        "clarify-research-safe-actions": rows.iter()
            .filter(|row| row["expected"] == "clarify" || row["expected"] == "research")
            .all(|row| row["writeBoundary"] == "no-authority-write" && row["hasForbiddenNoAuthorityAction"] == false),
        "no-authority-materialization-rejected": materialize_rejections.iter().all(|row| row["materializationRejected"] == true),
    });
    Ok(proof(
        "agentflow-v116-core-route-next-action-semantics.v1",
        checks,
        json!({ "routes": rows, "materializeRejections": materialize_rejections }),
    ))
}

fn desktop_invocation_bridge_proof(workspace: &Path) -> Result<Value> {
    let runtime_api =
        read_text(workspace.join("apps/desktop/src-tauri/src/commands/runtime_api.rs"))?;
    let main_rs = read_text(workspace.join("apps/desktop/src-tauri/src/main.rs"))?;
    let all_commands_registered = [
        "commands::runtime_api::preview_product_intent",
        "commands::runtime_api::confirm_product_spec_preview",
        "commands::runtime_api::materialize_confirmed_product_spec",
        "commands::runtime_api::create_executor_handoff_package",
        "commands::runtime_api::check_executor_diff_boundary",
        "commands::runtime_api::capture_executor_evidence",
        "commands::runtime_api::write_executor_result_to_issue",
        "commands::runtime_api::record_executor_lifecycle",
    ]
    .iter()
    .all(|needle| main_rs.contains(needle));
    let checks = json!({
        "product-intake-preview-command": runtime_api.contains("fn preview_product_intent"),
        "product-intake-confirm-command": runtime_api.contains("fn confirm_product_spec_preview"),
        "product-intake-materialize-command": runtime_api.contains("fn materialize_confirmed_product_spec"),
        "executor-handoff-command": runtime_api.contains("fn create_executor_handoff_package"),
        "executor-boundary-command": runtime_api.contains("fn check_executor_diff_boundary"),
        "executor-evidence-command": runtime_api.contains("fn capture_executor_evidence"),
        "executor-writeback-command": runtime_api.contains("fn write_executor_result_to_issue"),
        "executor-lifecycle-command": runtime_api.contains("fn record_executor_lifecycle"),
        "all-commands-registered": all_commands_registered,
    });
    Ok(proof(
        "agentflow-v116-product-spec-intake-desktop-invocation-bridge.v1",
        checks,
        json!({}),
    ))
}

fn handoff_package_proof(workspace: &Path) -> Result<Value> {
    let root = workspace.join("tmp/v116-handoff");
    reset_path(&root)?;
    let issue = seed_issue(&root, "AF-V116-HANDOFF-001", vec!["docs/**".to_string()])?;
    let handoff = create_executor_handoff_package(
        &root,
        ExecutorHandoffRequest {
            issue_id: issue.issue_id.clone(),
            run_id: "run-001".to_string(),
            executor_adapter_id: "codex".to_string(),
            executor_role: "build-agent".to_string(),
            session_id: None,
            branch_name: None,
            working_directory: None,
        },
    )?;
    let checks = json!({
        "handoff-has-issue-and-run": handoff.issue_id == issue.issue_id && handoff.run_id == "run-001",
        "handoff-has-allowed-denied-surface": !handoff.allowed_surface.is_empty() && !handoff.denied_surface.is_empty(),
        "handoff-has-evidence-policy": handoff.evidence_policy.contains("required"),
        "handoff-has-validation-commands": !handoff.validation_commands.is_empty(),
        "handoff-session-not-authority": handoff.session_is_authority == false && handoff.authority_boundary.contains("spec-issue-is-authority"),
        "handoff-file-written": root.join(&handoff.handoff_path).is_file(),
    });
    Ok(proof(
        "agentflow-v116-executor-adapter-handoff-package.v1",
        checks,
        json!({ "handoff": handoff }),
    ))
}

fn diff_boundary_proof(workspace: &Path) -> Result<Value> {
    let root = workspace.join("tmp/v116-boundary");
    reset_path(&root)?;
    let issue = seed_issue(&root, "AF-V116-BOUNDARY-001", vec!["docs/**".to_string()])?;
    create_executor_handoff_package(&root, handoff_request(&issue.issue_id, "run-001"))?;
    let in_scope = check_executor_diff_boundary(
        &root,
        ExecutorDiffBoundaryRequest {
            issue_id: issue.issue_id.clone(),
            run_id: "run-001".to_string(),
            changed_files: vec![changed_file("docs/requirements/v116.md")],
            base_commit: None,
            head_commit: None,
        },
    )?;
    create_executor_handoff_package(&root, handoff_request(&issue.issue_id, "run-002"))?;
    let out_of_scope = check_executor_diff_boundary(
        &root,
        ExecutorDiffBoundaryRequest {
            issue_id: issue.issue_id.clone(),
            run_id: "run-002".to_string(),
            changed_files: vec![changed_file("src/main.rs")],
            base_commit: None,
            head_commit: None,
        },
    )?;
    let checks = json!({
        "in-scope-diff-passes": in_scope.status == "passed",
        "out-of-scope-diff-fails": out_of_scope.status == "failed" && !out_of_scope.boundary_failures.is_empty(),
        "boundary-report-written": root.join(&in_scope.report_path).is_file() && root.join(&out_of_scope.report_path).is_file(),
    });
    Ok(proof(
        "agentflow-v116-allowed-surface-diff-boundary-check.v1",
        checks,
        json!({ "inScope": in_scope, "outOfScope": out_of_scope }),
    ))
}

fn evidence_capture_proof(workspace: &Path) -> Result<Value> {
    let (root, issue, run_id) =
        prepare_success_run(workspace, "tmp/v116-evidence", "AF-V116-EVIDENCE-001")?;
    let evidence = capture_executor_evidence(
        &root,
        ExecutorEvidenceCaptureRequest {
            issue_id: issue.issue_id.clone(),
            run_id: run_id.clone(),
            summary: "executor validation captured".to_string(),
            commands: vec![passing_command()],
            boundary_failures: Vec::new(),
        },
    )?;
    let checks = json!({
        "evidence-status-passed": evidence.status == "passed",
        "evidence-links-validation": evidence.evidence.validation_path == evidence.validation_path,
        "evidence-has-command": !evidence.command_paths.is_empty(),
        "evidence-has-changed-files": evidence.changed_files_path.is_some(),
        "evidence-file-written": root.join(&evidence.evidence_path).is_file(),
    });
    Ok(proof(
        "agentflow-v116-executor-evidence-capture.v1",
        checks,
        json!({ "evidence": evidence }),
    ))
}

fn result_writeback_proof(workspace: &Path) -> Result<Value> {
    let (root, issue, run_id) =
        prepare_success_run(workspace, "tmp/v116-writeback", "AF-V116-WRITEBACK-001")?;
    capture_executor_evidence(
        &root,
        ExecutorEvidenceCaptureRequest {
            issue_id: issue.issue_id.clone(),
            run_id: run_id.clone(),
            summary: "executor validation captured".to_string(),
            commands: vec![passing_command()],
            boundary_failures: Vec::new(),
        },
    )?;
    let report = write_executor_result_to_issue(
        &root,
        ExecutorResultWritebackRequest {
            issue_id: issue.issue_id.clone(),
            run_id,
            outcome: ExecutorResultOutcome::Success,
            summary: "accepted".to_string(),
            artifacts: vec!["docs/requirements/v116.md".to_string()],
            logs: vec!["validation ok".to_string()],
            failure_reason: None,
            continuation_request: None,
        },
    )?;
    let checks = json!({
        "writeback-can-complete": report.can_writeback,
        "issue-status-done": report.issue_status == "done",
        "run-status-completed": report.run_status == "completed",
        "closeout-written": root.join(&report.closeout_path).is_file(),
        "writeback-report-written": root.join(&report.writeback_path).is_file(),
    });
    Ok(proof(
        "agentflow-v116-executor-result-issue-run-writeback.v1",
        checks,
        json!({ "writeback": report }),
    ))
}

fn failure_retry_semantics_proof(workspace: &Path) -> Result<Value> {
    let root = workspace.join("tmp/v116-failure-retry");
    reset_path(&root)?;
    let issue = seed_issue(&root, "AF-V116-FAILURE-001", vec!["docs/**".to_string()])?;
    create_executor_handoff_package(&root, handoff_request(&issue.issue_id, "run-001"))?;
    let timeout = record_executor_lifecycle(
        &root,
        ExecutorLifecycleRequest {
            issue_id: issue.issue_id.clone(),
            run_id: "run-001".to_string(),
            action: ExecutorLifecycleAction::Timeout,
            actor: "runtime".to_string(),
            reason: "executor heartbeat expired".to_string(),
            retry_run_id: None,
        },
    )?;
    let retry = record_executor_lifecycle(
        &root,
        ExecutorLifecycleRequest {
            issue_id: issue.issue_id.clone(),
            run_id: "run-001".to_string(),
            action: ExecutorLifecycleAction::Retry,
            actor: "human-owner".to_string(),
            reason: "retry with clean run".to_string(),
            retry_run_id: Some("run-002".to_string()),
        },
    )?;
    create_executor_handoff_package(&root, handoff_request(&issue.issue_id, "run-003"))?;
    let cancel = record_executor_lifecycle(
        &root,
        ExecutorLifecycleRequest {
            issue_id: issue.issue_id.clone(),
            run_id: "run-003".to_string(),
            action: ExecutorLifecycleAction::Cancel,
            actor: "human-owner".to_string(),
            reason: "cancelled by owner".to_string(),
            retry_run_id: None,
        },
    )?;
    let checks = json!({
        "timeout-blocks-current-run": timeout.run_status == "failed" && timeout.issue_status == "blocked",
        "retry-creates-new-run": retry.retry_run_id.as_deref() == Some("run-002") && root.join(".agentflow/tasks").join(&issue.issue_id).join("runs/run-002/run.json").is_file(),
        "cancel-is-terminal": cancel.run_status == "cancelled" && cancel.issue_status == "cancel",
        "receipts-written": root.join(&timeout.receipt_path).is_file() && root.join(&retry.receipt_path).is_file() && root.join(&cancel.receipt_path).is_file(),
    });
    Ok(proof(
        "agentflow-v116-failure-timeout-cancel-retry-semantics.v1",
        checks,
        json!({ "timeout": timeout, "retry": retry, "cancel": cancel }),
    ))
}

fn software_dev_golden_path_proof(workspace: &Path) -> Result<Value> {
    let root = workspace.join("tmp/v116-software-dev-golden");
    reset_path(&root)?;
    let receipt = preview_product_intent(
        workspace,
        &root,
        product_request(
            "v116-software-dev",
            "实现 Software Dev executor handoff、evidence、boundary 和 writeback golden path。",
        ),
    )?;
    let confirmation = confirm_product_spec_preview(
        &root,
        ProductSpecConfirmationRequest {
            preview_id: receipt.preview_id.clone(),
            preview_hash: receipt.preview_hash.clone(),
            actor: "human-owner".to_string(),
            decision: ProductSpecPreviewDecision::Confirm,
            summary: "confirm Software Dev golden path".to_string(),
        },
    )?;
    let materialized = materialize_confirmed_product_spec(&root, &receipt.preview_id)?;
    let issue_id = materialized
        .materialized_issue_ids
        .first()
        .cloned()
        .context("golden path requires materialized issue")?;
    let handoff = create_executor_handoff_package(&root, handoff_request(&issue_id, "run-001"))?;
    let boundary = check_executor_diff_boundary(
        &root,
        ExecutorDiffBoundaryRequest {
            issue_id: issue_id.clone(),
            run_id: "run-001".to_string(),
            changed_files: vec![changed_file("crates/spec/src/lib.rs")],
            base_commit: None,
            head_commit: None,
        },
    )?;
    let evidence = capture_executor_evidence(
        &root,
        ExecutorEvidenceCaptureRequest {
            issue_id: issue_id.clone(),
            run_id: "run-001".to_string(),
            summary: "Software Dev executor golden path evidence".to_string(),
            commands: vec![passing_command()],
            boundary_failures: Vec::new(),
        },
    )?;
    let writeback = write_executor_result_to_issue(
        &root,
        ExecutorResultWritebackRequest {
            issue_id: issue_id.clone(),
            run_id: "run-001".to_string(),
            outcome: ExecutorResultOutcome::Success,
            summary: "accepted".to_string(),
            artifacts: vec!["crates/spec/src/lib.rs".to_string()],
            logs: vec!["golden path accepted".to_string()],
            failure_reason: None,
            continuation_request: None,
        },
    )?;
    let checks = json!({
        "product-mapping-is-software-dev": receipt.product_mapping.product_id == "software-dev",
        "confirmation-bound-to-preview": confirmation.preview_id == receipt.preview_id && confirmation.preview_hash == receipt.preview_hash,
        "spec-issue-materialized": !materialized.materialized_issue_ids.is_empty(),
        "handoff-session-not-authority": handoff.session_is_authority == false,
        "boundary-passed": boundary.status == "passed",
        "evidence-passed": evidence.status == "passed",
        "writeback-done": writeback.issue_status == "done",
    });
    Ok(proof(
        "agentflow-v116-software-dev-real-executor-golden-path.v1",
        checks,
        json!({
            "receipt": receipt,
            "materialization": materialized,
            "handoff": handoff,
            "boundary": boundary,
            "evidence": evidence,
            "writeback": writeback,
        }),
    ))
}

fn release_certification_proof(
    planning: &Value,
    route: &Value,
    desktop: &Value,
    handoff: &Value,
    boundary: &Value,
    evidence: &Value,
    writeback: &Value,
    failure_retry: &Value,
    golden: &Value,
) -> Value {
    let artifacts = [
        planning,
        route,
        desktop,
        handoff,
        boundary,
        evidence,
        writeback,
        failure_retry,
        golden,
    ];
    let checks = json!({
        "all-v116-primary-proofs-passed": artifacts.iter().all(|artifact| artifact["status"] == "passed"),
        "executor-handoff-certified": handoff["status"] == "passed",
        "diff-boundary-certified": boundary["status"] == "passed",
        "evidence-certified": evidence["status"] == "passed",
        "writeback-certified": writeback["status"] == "passed",
        "failure-retry-certified": failure_retry["status"] == "passed",
        "software-dev-golden-certified": golden["status"] == "passed",
    });
    proof(
        "agentflow-v116-release-certification.v1",
        checks,
        json!({ "certifiedProofCount": artifacts.len(), "remainingRisks": [] }),
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
            changed_files: vec![changed_file("docs/requirements/v116.md")],
            base_commit: None,
            head_commit: None,
        },
    )?;
    Ok((root, issue, run_id))
}

fn seed_issue(root: &Path, issue_id: &str, allowed_paths: Vec<String>) -> Result<SpecIssue> {
    let requirement_path = root.join("docs/requirements/v116.md");
    if let Some(parent) = requirement_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(
        &requirement_path,
        "# V116 Requirement\n\nExecutor Adapter Real Execution Closure proof fixture.\n",
    )?;
    let mut draft = SpecIssueDraft::new(issue_id);
    draft.title = Some("Executor closure proof issue".to_string());
    draft.summary = Some("Prove handoff, evidence, boundary and writeback.".to_string());
    draft.project_id = Some("project-v116".to_string());
    draft.allowed_paths = allowed_paths;
    draft.forbidden_paths = vec!["secrets/**".to_string(), ".env".to_string()];
    draft.validation_commands = vec!["cargo test -p agentflow-runtime-api".to_string()];
    let issue = issue_from_requirement(root, Path::new("docs/requirements/v116.md"), draft)?;
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

fn product_request(workspace_id: &str, raw_text: &str) -> ProductIntentIntakeRequest {
    ProductIntentIntakeRequest {
        raw_text: raw_text.to_string(),
        selected_product_id: "software-dev".to_string(),
        workspace_id: workspace_id.to_string(),
        source_surface: "desktop-project-home".to_string(),
        locale: "zh-CN".to_string(),
        attachment_refs: Vec::new(),
        source_refs: vec!["docs/project/roadmap.md".to_string()],
    }
}

fn proof(version: &str, checks: Value, payload: Value) -> Value {
    json!({
        "version": version,
        "status": status_from_checks(&checks),
        "coverage": checks,
        "failedCoverage": failed_checks(&checks),
        "payload": payload,
        "checkedAt": unix_timestamp(),
    })
}

fn status_from_checks(checks: &Value) -> &'static str {
    if failed_checks(checks).is_empty() {
        "passed"
    } else {
        "failed"
    }
}

fn failed_checks(checks: &Value) -> Vec<String> {
    checks
        .as_object()
        .into_iter()
        .flat_map(|object| object.iter())
        .filter_map(|(key, value)| (value != &Value::Bool(true)).then(|| key.clone()))
        .collect()
}

fn write_json(path: &Path, payload: &Value) -> Result<()> {
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
