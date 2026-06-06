use super::model::{ProjectInitializationContext, ProjectInitializationSummary};
use agentflow_execute::{
    ExecuteChangedFile, ExecuteChangedFiles, ExecuteCommandRecord, ExecutePlan, ExecutePlanStep,
    ExecutePlanStepKind, ExecuteResult, ExecuteResultNext, ExecuteRun, ExecuteRunInput,
    ExecuteRunPaths, ExecuteRunStatus, ExecuteValidationResult, EXECUTE_COMMAND_VERSION,
    EXECUTE_PLAN_VERSION, EXECUTE_RESULT_VERSION, EXECUTE_RUN_VERSION,
};
use agentflow_input::issue::{
    AgentRole, DisplayStatus, InputIssue, InputIssueKind, InputIssueModel, InputIssueRelations,
    InputIssueStatus, InputPanelLink, InputPriority, InputRiskLevel, InputSystemRecord,
    IssueCategory,
};
use agentflow_input::project::{InputProject, InputProjectStatus};
use agentflow_output::{
    AuditCheckStatus, AuditChecks, AuditEvidenceMap, AuditFinding, AuditFindingSeverity,
    AuditFindings, AuditRequest, AuditRequestSource, AuditScope, AuditScopeRef, AuditStatus,
    AuditSummary, AuditTraceability, AuditTraceabilityItem, AuditTrigger, HumanAudit,
    OutputCommandEvidence, OutputEvidence, OutputEvidenceExecuteArtifacts, OutputEvidenceInput,
    OutputEvidencePanel, OutputManualProof, OutputPrMetadata, OutputReleaseDelivery,
    OutputReleaseDeliveryArtifacts, OutputValidationSummary, AUDIT_EVIDENCE_MAP_VERSION,
    AUDIT_FINDINGS_VERSION, AUDIT_REQUEST_VERSION, AUDIT_TRACEABILITY_VERSION,
    OUTPUT_AUDIT_VERSION, OUTPUT_EVIDENCE_VERSION, OUTPUT_PR_METADATA_VERSION,
    OUTPUT_RELEASE_DELIVERY_VERSION,
};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs,
    path::Path,
    process::{Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};

const INIT_STATUS_PATH: &str = ".agentflow/state/indexes/base-release-initialization.json";
const RECENT_CONTEXT_PATH: &str = ".agentflow/state/indexes/recent-project-context.json";
const GIT_CONTEXT_PATH: &str = ".agentflow/state/indexes/git-context.json";
const DEMO_SOURCE: &str = "agentflow-demo";
const DEMO_SPEC_ID: &str = "SPEC-DEMO-001";
const DEMO_PROJECT_ID: &str = "PROJ-DEMO-001";
const DEMO_DELIVERY_RUN_ID: &str = "DEL-DEMO-001";
const DEMO_AUDIT_ID: &str = "AUD-DEMO-001";

#[derive(Clone)]
struct DemoIssueSeed {
    issue_id: &'static str,
    title: &'static str,
    summary: &'static str,
    status: InputIssueStatus,
    display_status: DisplayStatus,
}

pub(crate) fn initialize_base_release_project(
    root: &Path,
) -> Result<ProjectInitializationSummary, String> {
    let git_context = collect_recent_git_context(root)?;
    let has_git_context = !git_context.is_empty();
    let has_real_issues = has_non_demo_issues(root);
    let has_any_issues = issue_json_count(root) > 0;
    let project_kind = if has_git_context || has_real_issues {
        "existing"
    } else {
        "new"
    };
    let mut warnings = Vec::new();
    let mut paths = Vec::new();

    if project_kind == "existing" {
        if has_git_context {
            write_git_context(root, &git_context)?;
            paths.extend([
                GIT_CONTEXT_PATH.to_string(),
                RECENT_CONTEXT_PATH.to_string(),
            ]);
        } else {
            warnings.push("未读取到最近 Git 提交；保留已有 .agentflow 数据。".to_string());
        }

        let summary = ProjectInitializationSummary {
            version: "base-release-initialization.v1".to_string(),
            project_kind: project_kind.to_string(),
            initialized: true,
            demo_data_created: false,
            git_context_loaded: has_git_context,
            recent_context_count: git_context.len(),
            demo_issue_count: 0,
            demo_delivery_count: 0,
            demo_audit_count: 0,
            message: if has_git_context {
                format!("已读取最近项目记录：{} 条。", git_context.len())
            } else {
                "项目已准备好。".to_string()
            },
            paths,
            warnings,
            recent_context: git_context,
        };
        write_initialization_summary(root, &summary)?;
        refresh_indexes(root)?;
        return Ok(summary);
    }

    let mut demo_data_created = false;
    let mut demo_issue_count = 0;
    let mut demo_delivery_count = 0;
    let mut demo_audit_count = 0;
    if !has_any_issues {
        write_demo_input(root)?;
        write_demo_execute_and_output(root)?;
        write_demo_audit(root)?;
        demo_data_created = true;
        demo_issue_count = demo_issue_seeds().len();
        demo_delivery_count = 1;
        demo_audit_count = 1;
        paths.extend([
            ".agentflow/input/issues/AF-DEMO-001.json".to_string(),
            ".agentflow/input/issues/AF-DEMO-002.json".to_string(),
            ".agentflow/input/issues/AF-DEMO-003.json".to_string(),
            ".agentflow/input/issues/AF-DEMO-004.json".to_string(),
            ".agentflow/input/issues/AF-DEMO-005.json".to_string(),
            format!(".agentflow/output/release/{DEMO_DELIVERY_RUN_ID}/delivery.json"),
            format!(".agentflow/output/audit/{DEMO_AUDIT_ID}/audit-report.md"),
        ]);
    }

    let summary = ProjectInitializationSummary {
        version: "base-release-initialization.v1".to_string(),
        project_kind: project_kind.to_string(),
        initialized: true,
        demo_data_created,
        git_context_loaded: false,
        recent_context_count: 0,
        demo_issue_count,
        demo_delivery_count,
        demo_audit_count,
        message: if demo_data_created {
            "这是一个新项目，AgentFlow 已生成示例任务和审计材料。".to_string()
        } else {
            "新项目已准备好。".to_string()
        },
        paths,
        warnings,
        recent_context: Vec::new(),
    };
    write_initialization_summary(root, &summary)?;
    refresh_indexes(root)?;
    Ok(summary)
}

pub(crate) fn load_project_initialization_status(
    root: &Path,
) -> Result<ProjectInitializationSummary, String> {
    let path = root.join(INIT_STATUS_PATH);
    if path.is_file() {
        return read_json(&path);
    }

    let git_context = read_json::<Value>(&root.join(RECENT_CONTEXT_PATH))
        .ok()
        .and_then(|value| {
            value
                .get("items")
                .and_then(Value::as_array)
                .map(|items| items.len())
        })
        .unwrap_or(0);
    Ok(ProjectInitializationSummary {
        version: "base-release-initialization.v1".to_string(),
        project_kind: if git_context > 0 { "existing" } else { "new" }.to_string(),
        initialized: false,
        demo_data_created: has_demo_issues(root),
        git_context_loaded: git_context > 0,
        recent_context_count: git_context,
        demo_issue_count: demo_issue_count(root),
        demo_delivery_count: usize::from(
            root.join(".agentflow/output/release")
                .join(DEMO_DELIVERY_RUN_ID)
                .join("delivery.json")
                .is_file(),
        ),
        demo_audit_count: usize::from(
            root.join(".agentflow/output/audit")
                .join(DEMO_AUDIT_ID)
                .join("audit.json")
                .is_file(),
        ),
        message: "初始化状态尚未登记，刷新项目后会补齐。".to_string(),
        paths: Vec::new(),
        warnings: Vec::new(),
        recent_context: Vec::new(),
    })
}

fn write_demo_input(root: &Path) -> Result<(), String> {
    let now = unix_timestamp_seconds();
    let spec_dir = root
        .join(".agentflow/input/specs/approved")
        .join(DEMO_SPEC_ID);
    ensure_directory(&spec_dir)?;
    write_text(
        &spec_dir.join("product.md"),
        "# AgentFlow 示例规格\n\n这是一份本地示例规格，用来帮助你理解需求确认后的样子。\n",
    )?;
    write_text(
        &spec_dir.join("tech.md"),
        "# 技术说明\n\n示例数据只写入 `.agentflow/**`，不会修改项目源码。\n",
    )?;
    write_json(
        &spec_dir.join("spec.json"),
        &json!({
            "version": "input-spec.v1",
            "specId": DEMO_SPEC_ID,
            "demo": true,
            "source": DEMO_SOURCE,
            "createdBy": DEMO_SOURCE,
            "title": "AgentFlow 示例规格",
            "summary": "本地示例规格，用来展示 SPEC 到 Issue 的流程。"
        }),
    )?;
    write_json(
        &spec_dir.join("approval.json"),
        &json!({
            "version": "input-spec-approval.v1",
            "specId": DEMO_SPEC_ID,
            "approved": true,
            "approvedBy": DEMO_SOURCE,
            "approvedAt": now,
            "demo": true,
            "source": DEMO_SOURCE
        }),
    )?;

    let issue_ids = demo_issue_seeds()
        .iter()
        .map(|seed| seed.issue_id.to_string())
        .collect::<Vec<_>>();
    let project = InputProject {
        version: "input-project.v1".to_string(),
        project_id: DEMO_PROJECT_ID.to_string(),
        source_spec_id: DEMO_SPEC_ID.to_string(),
        title: "AgentFlow 示例项目".to_string(),
        summary: "本地示例项目，用来展示任务、交付和审计工作区。".to_string(),
        objective: "帮助用户第一次打开新项目时理解 AgentFlow 流程。".to_string(),
        scope: vec!["只写入 .agentflow 示例数据".to_string()],
        non_goals: vec!["不修改用户源码".to_string(), "不调用模型".to_string()],
        success_criteria: vec!["任务页、交付页、审计页都有可读示例。".to_string()],
        issue_ids,
        status: InputProjectStatus::Active,
        panel: InputPanelLink::default(),
        system: InputSystemRecord {
            created_by: DEMO_SOURCE.to_string(),
            created_at: now,
            updated_at: now,
            path: format!(".agentflow/input/projects/{DEMO_PROJECT_ID}.json"),
            revision: 1,
        },
    };
    write_json_with_demo_marker(
        &root
            .join(".agentflow/input/projects")
            .join(format!("{DEMO_PROJECT_ID}.json")),
        &project,
    )?;

    for seed in demo_issue_seeds() {
        let issue = InputIssue {
            version: "input-issue.v1".to_string(),
            issue_id: seed.issue_id.to_string(),
            issue_model: InputIssueModel::Project,
            issue_category: IssueCategory::Spec,
            required_agent_role: AgentRole::BuildAgent,
            source_spec_id: DEMO_SPEC_ID.to_string(),
            project_id: Some(DEMO_PROJECT_ID.to_string()),
            title: seed.title.to_string(),
            summary: seed.summary.to_string(),
            kind: InputIssueKind::Feature,
            priority: InputPriority::Normal,
            status: seed.status.clone(),
            display_status: seed.display_status.clone(),
            risk_level: InputRiskLevel::Low,
            scope: vec!["示例任务范围只覆盖 .agentflow 演示数据。".to_string()],
            non_goals: vec![
                "不修改项目源码。".to_string(),
                "不执行真实 Codex 任务。".to_string(),
            ],
            acceptance_criteria: vec![
                "用户能在任务页看到这条示例任务。".to_string(),
                "任务能展示状态、风险、范围和验收标准。".to_string(),
            ],
            validation_hints: vec![
                "npm --prefix apps/desktop run build".to_string(),
                "cargo check".to_string(),
            ],
            relations: InputIssueRelations::default(),
            panel: InputPanelLink::default(),
            audit: None,
            system: InputSystemRecord {
                created_by: DEMO_SOURCE.to_string(),
                created_at: now,
                updated_at: now,
                path: format!(".agentflow/input/issues/{}.json", seed.issue_id),
                revision: 1,
            },
        };
        write_json_with_demo_marker(
            &root
                .join(".agentflow/input/issues")
                .join(format!("{}.json", seed.issue_id)),
            &issue,
        )?;
    }
    Ok(())
}

fn write_demo_execute_and_output(root: &Path) -> Result<(), String> {
    let now = unix_timestamp_seconds();
    let run_id = DEMO_DELIVERY_RUN_ID;
    let run_dir = root.join(".agentflow/execute/runs").join(run_id);
    ensure_directory(&run_dir.join("commands"))?;
    ensure_directory(&run_dir.join("checkpoints"))?;
    ensure_directory(&run_dir.join("patches"))?;
    ensure_directory(&run_dir.join("review"))?;

    let run = ExecuteRun {
        version: EXECUTE_RUN_VERSION.to_string(),
        run_id: run_id.to_string(),
        issue_id: "AF-DEMO-004".to_string(),
        source_spec_id: DEMO_SPEC_ID.to_string(),
        project_id: Some(DEMO_PROJECT_ID.to_string()),
        risk_level: "low".to_string(),
        status: ExecuteRunStatus::Completed,
        agent_role: "Build Agent".to_string(),
        created_by: DEMO_SOURCE.to_string(),
        created_at: now,
        updated_at: now,
        input: ExecuteRunInput {
            issue_path: ".agentflow/input/issues/AF-DEMO-004.json".to_string(),
            spec_path: format!(".agentflow/input/specs/approved/{DEMO_SPEC_ID}"),
            panel_snapshot_id: None,
            context_pack_id: None,
        },
        paths: ExecuteRunPaths {
            preflight: format!(".agentflow/execute/runs/{run_id}/preflight.json"),
            plan: format!(".agentflow/execute/runs/{run_id}/plan.json"),
            result: format!(".agentflow/execute/runs/{run_id}/result.json"),
            evidence: format!(".agentflow/output/evidence/{run_id}.json"),
        },
    };
    write_json_with_demo_marker(&run_dir.join("run.json"), &run)?;
    write_json(
        &run_dir.join("preflight.json"),
        &json!({
            "version": "execute-preflight.v1",
            "runId": run_id,
            "issueId": "AF-DEMO-004",
            "status": "ready",
            "demo": true,
            "source": DEMO_SOURCE
        }),
    )?;
    let plan = ExecutePlan {
        version: EXECUTE_PLAN_VERSION.to_string(),
        run_id: run_id.to_string(),
        issue_id: "AF-DEMO-004".to_string(),
        steps: vec![ExecutePlanStep {
            step_id: "demo-step-001".to_string(),
            kind: ExecutePlanStepKind::Review,
            target: Some(".agentflow/output".to_string()),
            command: None,
            summary: "阅读示例交付和证据链。".to_string(),
        }],
        allowed_write_paths: vec![".agentflow/".to_string()],
        allowed_commands: vec!["npm --prefix apps/desktop run build".to_string()],
    };
    write_json_with_demo_marker(&run_dir.join("plan.json"), &plan)?;
    write_json(
        &run_dir.join("checkpoints/chk-demo-001.json"),
        &json!({
            "version": "execute-checkpoint.v1",
            "checkpointId": "chk-demo-001",
            "runId": run_id,
            "createdAt": now,
            "demo": true,
            "source": DEMO_SOURCE
        }),
    )?;
    write_text(
        &run_dir.join("patches/worktree.diff"),
        "示例交付没有修改用户源码。\n",
    )?;
    write_json(
        &run_dir.join("patches/changed-files.json"),
        &ExecuteChangedFiles {
            version: "execute-changed-files.v1".to_string(),
            run_id: run_id.to_string(),
            files: vec![ExecuteChangedFile {
                path: ".agentflow/output/release/DEL-DEMO-001/delivery.json".to_string(),
                change_type: "created".to_string(),
                insertions: 1,
                deletions: 0,
            }],
        },
    )?;
    write_json(
        &run_dir.join("review/diff-summary.json"),
        &json!({
            "version": "execute-diff-summary.v1",
            "runId": run_id,
            "changedFiles": 1,
            "insertions": 1,
            "deletions": 0,
            "riskLevel": "low",
            "notes": ["示例交付只写入 .agentflow。"],
            "demo": true,
            "source": DEMO_SOURCE
        }),
    )?;
    let command = ExecuteCommandRecord {
        version: EXECUTE_COMMAND_VERSION.to_string(),
        command_id: "cmd-demo-001".to_string(),
        run_id: run_id.to_string(),
        label: "demo validation".to_string(),
        program: "agentflow-demo".to_string(),
        args: vec!["validate".to_string()],
        cwd: root.display().to_string(),
        source: DEMO_SOURCE.to_string(),
        started_at: now,
        finished_at: now,
        exit_code: Some(0),
        stdout_path: format!(".agentflow/execute/runs/{run_id}/commands/cmd-demo-001.stdout.txt"),
        stderr_path: format!(".agentflow/execute/runs/{run_id}/commands/cmd-demo-001.stderr.txt"),
    };
    write_json_with_demo_marker(&run_dir.join("commands/cmd-demo-001.json"), &command)?;
    write_text(
        &run_dir.join("commands/cmd-demo-001.stdout.txt"),
        "demo ok\n",
    )?;
    write_text(&run_dir.join("commands/cmd-demo-001.stderr.txt"), "")?;
    let result = ExecuteResult {
        version: EXECUTE_RESULT_VERSION.to_string(),
        run_id: run_id.to_string(),
        issue_id: "AF-DEMO-004".to_string(),
        status: ExecuteRunStatus::Completed,
        risk_level: "low".to_string(),
        changed_files: vec![".agentflow/output/release/DEL-DEMO-001/delivery.json".to_string()],
        commands: vec!["cmd-demo-001".to_string()],
        validation: ExecuteValidationResult {
            passed: true,
            evidence_path: format!(".agentflow/output/evidence/{run_id}.json"),
        },
        next: ExecuteResultNext {
            ready_for_delivery: true,
            needs_audit: true,
        },
    };
    write_json_with_demo_marker(&run_dir.join("result.json"), &result)?;

    let evidence = OutputEvidence {
        version: OUTPUT_EVIDENCE_VERSION.to_string(),
        run_id: run_id.to_string(),
        issue_id: "AF-DEMO-004".to_string(),
        source_spec_id: DEMO_SPEC_ID.to_string(),
        risk_level: "low".to_string(),
        completed_at: now,
        summary: "这是一次本地示例交付证据，用来展示 Codex 写回后会看到什么。".to_string(),
        input: OutputEvidenceInput {
            issue_path: ".agentflow/input/issues/AF-DEMO-004.json".to_string(),
            spec_path: format!(".agentflow/input/specs/approved/{DEMO_SPEC_ID}"),
        },
        panel: OutputEvidencePanel::default(),
        execute: OutputEvidenceExecuteArtifacts {
            run: format!(".agentflow/execute/runs/{run_id}/run.json"),
            preflight: format!(".agentflow/execute/runs/{run_id}/preflight.json"),
            plan: format!(".agentflow/execute/runs/{run_id}/plan.json"),
            result: format!(".agentflow/execute/runs/{run_id}/result.json"),
            checkpoint: Some(format!(
                ".agentflow/execute/runs/{run_id}/checkpoints/chk-demo-001.json"
            )),
            diff: Some(format!(
                ".agentflow/execute/runs/{run_id}/patches/worktree.diff"
            )),
            changed_files: Some(format!(
                ".agentflow/execute/runs/{run_id}/patches/changed-files.json"
            )),
            diff_summary: Some(format!(
                ".agentflow/execute/runs/{run_id}/review/diff-summary.json"
            )),
        },
        commands: vec![OutputCommandEvidence {
            command_id: "cmd-demo-001".to_string(),
            label: "demo validation".to_string(),
            exit_code: Some(0),
            record_path: format!(".agentflow/execute/runs/{run_id}/commands/cmd-demo-001.json"),
            stdout_path: Some(format!(
                ".agentflow/execute/runs/{run_id}/commands/cmd-demo-001.stdout.txt"
            )),
            stderr_path: Some(format!(
                ".agentflow/execute/runs/{run_id}/commands/cmd-demo-001.stderr.txt"
            )),
        }],
        validation: OutputValidationSummary {
            passed: true,
            failed_commands: Vec::new(),
            skipped: Vec::new(),
        },
        manual_proof: OutputManualProof {
            required: false,
            notes: vec!["这是 AgentFlow 本地示例证据。".to_string()],
            screenshots: Vec::new(),
            recordings: Vec::new(),
        },
    };
    write_json_with_demo_marker(
        &root
            .join(".agentflow/output/evidence")
            .join(format!("{run_id}.json")),
        &evidence,
    )?;
    write_demo_delivery(root, now)
}

fn write_demo_delivery(root: &Path, now: u64) -> Result<(), String> {
    let run_id = DEMO_DELIVERY_RUN_ID;
    let release_dir = root.join(".agentflow/output/release").join(run_id);
    ensure_directory(&release_dir)?;
    let delivery = OutputReleaseDelivery {
        version: OUTPUT_RELEASE_DELIVERY_VERSION.to_string(),
        run_id: run_id.to_string(),
        issue_id: "AF-DEMO-004".to_string(),
        source_spec_id: DEMO_SPEC_ID.to_string(),
        risk_level: "low".to_string(),
        status: "ready".to_string(),
        created_by: "Build Agent".to_string(),
        created_at: now,
        evidence_path: format!(".agentflow/output/evidence/{run_id}.json"),
        execute_result_path: format!(".agentflow/execute/runs/{run_id}/result.json"),
        diff_summary_path: Some(format!(
            ".agentflow/execute/runs/{run_id}/review/diff-summary.json"
        )),
        artifacts: OutputReleaseDeliveryArtifacts {
            pr_draft: format!(".agentflow/output/release/{run_id}/pr-draft.md"),
            pr_metadata: format!(".agentflow/output/release/{run_id}/pr-metadata.json"),
            review_checklist: format!(".agentflow/output/release/{run_id}/review-checklist.md"),
            changelog: format!(".agentflow/output/release/{run_id}/changelog.md"),
            release_note: format!(".agentflow/output/release/{run_id}/release-note.md"),
        },
    };
    write_json_with_demo_marker(&release_dir.join("delivery.json"), &delivery)?;
    write_json_with_demo_marker(
        &release_dir.join("pr-metadata.json"),
        &OutputPrMetadata {
            version: OUTPUT_PR_METADATA_VERSION.to_string(),
            run_id: run_id.to_string(),
            issue_id: "AF-DEMO-004".to_string(),
            source_spec_id: DEMO_SPEC_ID.to_string(),
            title: "示例交付：核对交付证据".to_string(),
            branch_name: None,
            remote_pr_url: None,
            status: "demo-only".to_string(),
            created_remote_pr: false,
        },
    )?;
    write_text(
        &release_dir.join("pr-draft.md"),
        "# 示例 PR 草稿\n\n这是本地示例，不会创建远程 PR。\n",
    )?;
    write_text(
        &release_dir.join("review-checklist.md"),
        "# 示例检查清单\n\n- [x] 证据存在\n- [x] 交付存在\n",
    )?;
    write_text(
        &release_dir.join("changelog.md"),
        "# 示例变更记录\n\n- 展示交付页结构。\n",
    )?;
    write_text(
        &release_dir.join("release-note.md"),
        "# 示例发布说明\n\n这是 AgentFlow 示例交付。\n",
    )
}

fn write_demo_audit(root: &Path) -> Result<(), String> {
    let now = unix_timestamp_seconds();
    let audit_id = DEMO_AUDIT_ID;
    let audit_dir = root.join(".agentflow/output/audit").join(audit_id);
    ensure_directory(&audit_dir)?;
    let request = AuditRequest {
        version: AUDIT_REQUEST_VERSION.to_string(),
        audit_id: audit_id.to_string(),
        trigger: AuditTrigger::ReleaseAuto,
        requested_by: DEMO_SOURCE.to_string(),
        requested_at: now,
        reason: "Release 已生成，AgentFlow 规则要求进行审计。".to_string(),
        source: Some(AuditRequestSource {
            kind: "release-delivery".to_string(),
            delivery_id: Some(DEMO_DELIVERY_RUN_ID.to_string()),
            run_id: Some(DEMO_DELIVERY_RUN_ID.to_string()),
            issue_id: Some("AF-DEMO-004".to_string()),
            spec_id: Some(DEMO_SPEC_ID.to_string()),
        }),
        scope: AuditScope {
            description: "审计示例交付、证据和任务追溯。".to_string(),
            refs: vec![
                AuditScopeRef {
                    kind: "spec".to_string(),
                    id: DEMO_SPEC_ID.to_string(),
                    path: format!(".agentflow/input/specs/approved/{DEMO_SPEC_ID}/"),
                },
                AuditScopeRef {
                    kind: "issue".to_string(),
                    id: "AF-DEMO-004".to_string(),
                    path: ".agentflow/input/issues/AF-DEMO-004.json".to_string(),
                },
                AuditScopeRef {
                    kind: "execute-run".to_string(),
                    id: DEMO_DELIVERY_RUN_ID.to_string(),
                    path: format!(".agentflow/execute/runs/{DEMO_DELIVERY_RUN_ID}/"),
                },
                AuditScopeRef {
                    kind: "evidence".to_string(),
                    id: DEMO_DELIVERY_RUN_ID.to_string(),
                    path: format!(".agentflow/output/evidence/{DEMO_DELIVERY_RUN_ID}.json"),
                },
                AuditScopeRef {
                    kind: "release-delivery".to_string(),
                    id: DEMO_DELIVERY_RUN_ID.to_string(),
                    path: format!(".agentflow/output/release/{DEMO_DELIVERY_RUN_ID}/delivery.json"),
                },
            ],
        },
    };
    let audit = HumanAudit {
        version: OUTPUT_AUDIT_VERSION.to_string(),
        audit_id: audit_id.to_string(),
        trigger: AuditTrigger::ReleaseAuto,
        requested_by: DEMO_SOURCE.to_string(),
        requested_at: now,
        source_delivery_id: Some(DEMO_DELIVERY_RUN_ID.to_string()),
        source_run_id: Some(DEMO_DELIVERY_RUN_ID.to_string()),
        source_issue_id: Some("AF-DEMO-004".to_string()),
        status: AuditStatus::PassedWithWarnings,
        summary: AuditSummary {
            checks: 7,
            passed: 6,
            warnings: 1,
            failed: 0,
            findings: 1,
        },
        checks: AuditChecks {
            checkpoint_exists: AuditCheckStatus::Passed,
            changed_files_recorded: AuditCheckStatus::Passed,
            allowed_write_paths_only: AuditCheckStatus::Passed,
            commands_recorded: AuditCheckStatus::Passed,
            high_risk_confirmed_if_needed: AuditCheckStatus::Passed,
            evidence_complete: AuditCheckStatus::Passed,
            release_delivery_complete: AuditCheckStatus::Warning,
        },
        paths: BTreeMap::from([
            (
                "auditRoot".to_string(),
                format!(".agentflow/output/audit/{audit_id}"),
            ),
            (
                "index".to_string(),
                ".agentflow/output/audit/index.json".to_string(),
            ),
        ]),
    };
    let findings = AuditFindings {
        version: AUDIT_FINDINGS_VERSION.to_string(),
        audit_id: audit_id.to_string(),
        findings: vec![AuditFinding {
            finding_id: "AF-DEMO-FINDING-001".to_string(),
            severity: AuditFindingSeverity::Low,
            category: "demo".to_string(),
            title: "示例审计只用于演示".to_string(),
            detail: "这条审计来自 AgentFlow 本地示例数据，不代表真实交付结论。".to_string(),
            evidence_path: format!(".agentflow/output/evidence/{DEMO_DELIVERY_RUN_ID}.json"),
            recommendation: "真实交付完成后等待 Agent 写入审计报告。".to_string(),
        }],
    };
    let evidence_map = AuditEvidenceMap {
        version: AUDIT_EVIDENCE_MAP_VERSION.to_string(),
        audit_id: audit_id.to_string(),
        inputs: [
            (
                "issue".to_string(),
                ".agentflow/input/issues/AF-DEMO-004.json".to_string(),
            ),
            (
                "evidence".to_string(),
                format!(".agentflow/output/evidence/{DEMO_DELIVERY_RUN_ID}.json"),
            ),
            (
                "delivery".to_string(),
                format!(".agentflow/output/release/{DEMO_DELIVERY_RUN_ID}/delivery.json"),
            ),
        ]
        .into_iter()
        .collect(),
    };
    let traceability = AuditTraceability {
        version: AUDIT_TRACEABILITY_VERSION.to_string(),
        audit_id: audit_id.to_string(),
        chain: vec![
            AuditTraceabilityItem {
                layer: "spec".to_string(),
                id: DEMO_SPEC_ID.to_string(),
                path: format!(".agentflow/input/specs/approved/{DEMO_SPEC_ID}/"),
            },
            AuditTraceabilityItem {
                layer: "issue".to_string(),
                id: "AF-DEMO-004".to_string(),
                path: ".agentflow/input/issues/AF-DEMO-004.json".to_string(),
            },
            AuditTraceabilityItem {
                layer: "delivery".to_string(),
                id: DEMO_DELIVERY_RUN_ID.to_string(),
                path: format!(".agentflow/output/release/{DEMO_DELIVERY_RUN_ID}/delivery.json"),
            },
        ],
    };

    write_json_with_demo_marker(&audit_dir.join("audit-request.json"), &request)?;
    write_json_with_demo_marker(&audit_dir.join("audit.json"), &audit)?;
    write_text(
        &audit_dir.join("audit-report.md"),
        "# 示例审计报告\n\n结论：通过但有提示。\n\n这是 AgentFlow 本地示例审计，用来展示审计报告、发现项和证据链。\n",
    )?;
    write_json_with_demo_marker(&audit_dir.join("findings.json"), &findings)?;
    write_text(
        &audit_dir.join("checklist.md"),
        "# 示例审计清单\n\n- [x] 证据链存在\n- [x] 交付记录存在\n",
    )?;
    write_json_with_demo_marker(&audit_dir.join("evidence-map.json"), &evidence_map)?;
    write_json_with_demo_marker(&audit_dir.join("traceability.json"), &traceability)?;
    Ok(())
}

fn write_git_context(root: &Path, contexts: &[ProjectInitializationContext]) -> Result<(), String> {
    let now = unix_timestamp_seconds();
    let context_value = json!({
        "version": "git-context.v1",
        "source": "local-git",
        "demo": false,
        "updatedAt": now,
        "count": contexts.len(),
        "items": contexts,
    });
    write_json(&root.join(GIT_CONTEXT_PATH), &context_value)?;
    write_json(&root.join(RECENT_CONTEXT_PATH), &context_value)
}

fn write_initialization_summary(
    root: &Path,
    summary: &ProjectInitializationSummary,
) -> Result<(), String> {
    write_json(&root.join(INIT_STATUS_PATH), summary)
}

fn refresh_indexes(root: &Path) -> Result<(), String> {
    agentflow_input::prepare_input_workspace(root)
        .map_err(|error| format!("refresh input after initialization: {error}"))?;
    agentflow_execute::prepare_execute_workspace(root)
        .map_err(|error| format!("refresh execute after initialization: {error}"))?;
    agentflow_output::prepare_output_workspace(root)
        .map_err(|error| format!("refresh output after initialization: {error}"))?;
    agentflow_state::refresh_state(root)
        .map_err(|error| format!("refresh state after initialization: {error}"))?;
    Ok(())
}

fn collect_recent_git_context(root: &Path) -> Result<Vec<ProjectInitializationContext>, String> {
    if !root.join(".git").is_dir() || !git_has_commits(root) {
        return Ok(Vec::new());
    }

    let Some(raw_log) = git_output(
        root,
        &[
            "log",
            "--max-count=10",
            "--date=iso-strict",
            "--pretty=format:%H%x1f%an%x1f%aI%x1f%s",
        ],
    ) else {
        return Ok(Vec::new());
    };
    let remote_url = git_output(root, &["remote", "get-url", "origin"]);

    Ok(raw_log
        .lines()
        .filter_map(|line| {
            let parts = line.split('\u{1f}').collect::<Vec<_>>();
            if parts.len() < 4 {
                return None;
            }
            let sha = parts[0].to_string();
            let title = parts[3].to_string();
            let changed_files = git_changed_files(root, &sha);
            Some(ProjectInitializationContext {
                id: sha.chars().take(12).collect(),
                title: title.clone(),
                summary: context_summary(&title, &changed_files),
                committed_at: Some(parts[2].to_string()),
                author: Some(parts[1].to_string()),
                changed_files,
                source_url: remote_url
                    .as_deref()
                    .and_then(|remote| github_commit_url(remote, &sha)),
            })
        })
        .collect())
}

fn context_summary(title: &str, changed_files: &[String]) -> String {
    let area = changed_files
        .iter()
        .filter_map(|path| path.split('/').next())
        .find(|part| !part.is_empty())
        .unwrap_or("项目");
    format!("最近合并或提交：{title}。影响范围：{area}。可承接需求：从这条记录继续整理下一步。")
}

fn git_changed_files(root: &Path, sha: &str) -> Vec<String> {
    git_output(root, &["show", "--pretty=format:", "--name-only", sha])
        .map(|raw| {
            raw.lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .take(20)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn github_commit_url(remote: &str, sha: &str) -> Option<String> {
    let normalized = remote
        .trim()
        .trim_end_matches(".git")
        .replace("git@github.com:", "https://github.com/");
    normalized
        .starts_with("https://github.com/")
        .then(|| format!("{normalized}/commit/{sha}"))
}

fn git_has_commits(root: &Path) -> bool {
    git_output(root, &["rev-parse", "--verify", "HEAD"]).is_some()
}

fn git_output(root: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .stdin(Stdio::null())
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|value| !value.is_empty())
}

fn has_non_demo_issues(root: &Path) -> bool {
    read_issue_values(root).into_iter().any(|value| {
        value
            .get("demo")
            .and_then(Value::as_bool)
            .map(|demo| !demo)
            .unwrap_or(true)
    })
}

fn has_demo_issues(root: &Path) -> bool {
    demo_issue_count(root) > 0
}

fn demo_issue_count(root: &Path) -> usize {
    read_issue_values(root)
        .into_iter()
        .filter(|value| value.get("source").and_then(Value::as_str) == Some(DEMO_SOURCE))
        .count()
}

fn issue_json_count(root: &Path) -> usize {
    root.join(".agentflow/input/issues")
        .read_dir()
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .filter(|entry| {
                    entry.path().extension().and_then(|value| value.to_str()) == Some("json")
                })
                .count()
        })
        .unwrap_or(0)
}

fn read_issue_values(root: &Path) -> Vec<Value> {
    let issue_dir = root.join(".agentflow/input/issues");
    let Ok(entries) = issue_dir.read_dir() else {
        return Vec::new();
    };
    entries
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().and_then(|value| value.to_str()) == Some("json"))
        .filter_map(|entry| read_json::<Value>(&entry.path()).ok())
        .collect()
}

fn demo_issue_seeds() -> Vec<DemoIssueSeed> {
    vec![
        DemoIssueSeed {
            issue_id: "AF-DEMO-001",
            title: "整理第一个需求",
            summary: "先把一个模糊想法整理成可以确认的需求。",
            status: InputIssueStatus::Planned,
            display_status: DisplayStatus::Backlog,
        },
        DemoIssueSeed {
            issue_id: "AF-DEMO-002",
            title: "生成 Codex 任务包",
            summary: "这个任务已经准备好，可以复制任务包交给 Codex。",
            status: InputIssueStatus::ReadyForExecute,
            display_status: DisplayStatus::Ready,
        },
        DemoIssueSeed {
            issue_id: "AF-DEMO-003",
            title: "等待 Codex 写回结果",
            summary: "任务包已经交给 Codex，等待写回交付结果。",
            status: InputIssueStatus::ReadyForExecute,
            display_status: DisplayStatus::InProgress,
        },
        DemoIssueSeed {
            issue_id: "AF-DEMO-004",
            title: "核对交付证据",
            summary: "Codex 已写回交付，需要检查验证结果和证据。",
            status: InputIssueStatus::ReadyForExecute,
            display_status: DisplayStatus::Review,
        },
        DemoIssueSeed {
            issue_id: "AF-DEMO-005",
            title: "完成一次审计",
            summary: "交付已经通过审计，可以作为完成样例。",
            status: InputIssueStatus::Done,
            display_status: DisplayStatus::Done,
        },
    ]
}

fn write_json_with_demo_marker<T: serde::Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let mut value =
        serde_json::to_value(value).map_err(|error| format!("serialize json: {error}"))?;
    if let Value::Object(map) = &mut value {
        map.insert("demo".to_string(), Value::Bool(true));
        map.insert("source".to_string(), Value::String(DEMO_SOURCE.to_string()));
        map.insert(
            "createdBy".to_string(),
            Value::String(DEMO_SOURCE.to_string()),
        );
    }
    write_json(path, &value)
}

fn write_json<T: serde::Serialize>(path: &Path, value: &T) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        ensure_directory(parent)?;
    }
    let content = serde_json::to_string_pretty(value)
        .map_err(|error| format!("serialize {}: {error}", path.display()))?
        + "\n";
    fs::write(path, content).map_err(|error| format!("write {}: {error}", path.display()))
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, String> {
    let raw =
        fs::read_to_string(path).map_err(|error| format!("read {}: {error}", path.display()))?;
    serde_json::from_str(&raw).map_err(|error| format!("parse {}: {error}", path.display()))
}

fn write_text(path: &Path, content: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        ensure_directory(parent)?;
    }
    fs::write(path, content).map_err(|error| format!("write {}: {error}", path.display()))
}

fn ensure_directory(path: &Path) -> Result<(), String> {
    if path.exists() {
        if path.is_dir() {
            return Ok(());
        }
        return Err(format!("{} exists but is not a directory", path.display()));
    }
    fs::create_dir_all(path).map_err(|error| format!("create {}: {error}", path.display()))
}

fn unix_timestamp_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}
