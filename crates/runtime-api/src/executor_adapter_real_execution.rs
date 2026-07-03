//! Executor adapter real execution closure.
//!
//! This module owns the v1.1.6 bridge from materialized Spec Issue authority to
//! executor handoff, evidence, diff boundary, result writeback, and controlled
//! failure lifecycle records. Executor sessions are observable transport; they
//! never become AgentFlow authority.

use agentflow_spec::{read_spec_issue, update_spec_issue_status, SpecIssue, SpecIssueStatus};
use agentflow_task_artifacts::{
    commit_task_run_writeback, create_task_run, write_task_changed_files,
    write_task_command_record, write_task_evidence, write_task_executor_closeout,
    write_task_run_checkpoint, write_task_validation_with_assessment, TaskChangedFile,
    TaskChangedFileSource, TaskCommandInput, TaskEvidence, TaskExecutorCloseout,
    TaskExecutorCoreRefs, TaskExecutorResultStatus, TaskExecutorWorkHandoff, TaskRun,
    TaskRunStatus, TASK_EXECUTOR_CLOSEOUT_VERSION,
};
use agentflow_workflow_core::{
    canonicalize_project_root, normalize_relative_path_string, normalize_relative_to_root,
    WorkflowFlowType,
};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

pub const EXECUTOR_HANDOFF_PACKAGE_VERSION: &str = "agentflow-executor-handoff-package.v1";
pub const EXECUTOR_DIFF_BOUNDARY_REPORT_VERSION: &str =
    "agentflow-executor-diff-boundary-report.v1";
pub const EXECUTOR_EVIDENCE_CAPTURE_VERSION: &str = "agentflow-executor-evidence-capture.v1";
pub const EXECUTOR_RESULT_WRITEBACK_VERSION: &str = "agentflow-executor-result-writeback.v1";
pub const EXECUTOR_LIFECYCLE_RECEIPT_VERSION: &str = "agentflow-executor-lifecycle-receipt.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutorHandoffRequest {
    pub issue_id: String,
    pub run_id: String,
    #[serde(default = "default_executor_adapter_id")]
    pub executor_adapter_id: String,
    #[serde(default = "default_executor_role")]
    pub executor_role: String,
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub branch_name: Option<String>,
    #[serde(default)]
    pub working_directory: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutorHandoffPackage {
    pub version: String,
    pub handoff_id: String,
    pub issue_id: String,
    pub run_id: String,
    pub project_id: Option<String>,
    pub source_issue_path: String,
    pub workflow_ref: String,
    pub required_agent_role: String,
    pub executor_adapter_id: String,
    pub executor_role: String,
    pub allowed_surface: Vec<String>,
    pub denied_surface: Vec<String>,
    pub expected_outputs: Vec<String>,
    pub evidence_policy: String,
    pub acceptance_criteria: Vec<String>,
    pub validation_commands: Vec<String>,
    pub writeback_target: String,
    pub authority_boundary: String,
    pub session_is_authority: bool,
    pub handoff_path: String,
    pub created_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutorDiffInputFile {
    pub path: String,
    pub change_type: String,
    #[serde(default)]
    pub insertions: usize,
    #[serde(default)]
    pub deletions: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutorDiffBoundaryRequest {
    pub issue_id: String,
    pub run_id: String,
    pub changed_files: Vec<ExecutorDiffInputFile>,
    #[serde(default)]
    pub base_commit: Option<String>,
    #[serde(default)]
    pub head_commit: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutorDiffBoundaryReport {
    pub version: String,
    pub issue_id: String,
    pub run_id: String,
    pub status: String,
    pub allowed_surface: Vec<String>,
    pub denied_surface: Vec<String>,
    pub changed_paths: Vec<String>,
    pub out_of_scope_paths: Vec<String>,
    pub denied_paths: Vec<String>,
    pub boundary_failures: Vec<String>,
    pub changed_files_path: String,
    pub report_path: String,
    pub checked_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutorCommandEvidenceInput {
    pub label: String,
    pub program: String,
    pub args: Vec<String>,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutorEvidenceCaptureRequest {
    pub issue_id: String,
    pub run_id: String,
    pub summary: String,
    pub commands: Vec<ExecutorCommandEvidenceInput>,
    #[serde(default)]
    pub boundary_failures: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutorEvidenceCaptureReport {
    pub version: String,
    pub issue_id: String,
    pub run_id: String,
    pub status: String,
    pub evidence_path: String,
    pub validation_path: String,
    pub command_paths: Vec<String>,
    pub changed_files_path: Option<String>,
    pub evidence: TaskEvidence,
    pub captured_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExecutorResultOutcome {
    Success,
    Failed,
    Blocked,
    Cancelled,
    TimedOut,
    NeedsFix,
}

impl ExecutorResultOutcome {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Failed => "failed",
            Self::Blocked => "blocked",
            Self::Cancelled => "cancelled",
            Self::TimedOut => "timed-out",
            Self::NeedsFix => "needs-fix",
        }
    }

    fn task_result_status(&self) -> TaskExecutorResultStatus {
        match self {
            Self::Success => TaskExecutorResultStatus::Accepted,
            Self::NeedsFix => TaskExecutorResultStatus::Deferred,
            Self::Failed | Self::Blocked | Self::Cancelled | Self::TimedOut => {
                TaskExecutorResultStatus::Failed
            }
        }
    }

    fn run_status(&self, can_writeback: bool) -> TaskRunStatus {
        match self {
            Self::Success if can_writeback => TaskRunStatus::Completed,
            Self::Cancelled => TaskRunStatus::Cancelled,
            Self::Failed | Self::Blocked | Self::TimedOut | Self::NeedsFix | Self::Success => {
                TaskRunStatus::Failed
            }
        }
    }

    fn issue_status(&self, can_writeback: bool) -> SpecIssueStatus {
        match self {
            Self::Success if can_writeback => SpecIssueStatus::Done,
            Self::Cancelled => SpecIssueStatus::Cancel,
            Self::Failed | Self::Blocked | Self::TimedOut | Self::NeedsFix | Self::Success => {
                SpecIssueStatus::Blocked
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutorResultWritebackRequest {
    pub issue_id: String,
    pub run_id: String,
    pub outcome: ExecutorResultOutcome,
    pub summary: String,
    #[serde(default)]
    pub artifacts: Vec<String>,
    #[serde(default)]
    pub logs: Vec<String>,
    #[serde(default)]
    pub failure_reason: Option<String>,
    #[serde(default)]
    pub continuation_request: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutorResultWritebackReport {
    pub version: String,
    pub issue_id: String,
    pub run_id: String,
    pub outcome: ExecutorResultOutcome,
    pub can_writeback: bool,
    pub blocked_reasons: Vec<String>,
    pub issue_status: String,
    pub run_status: String,
    pub evidence_path: String,
    pub boundary_report_path: String,
    pub closeout_path: String,
    pub writeback_path: String,
    pub generated_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExecutorLifecycleAction {
    Timeout,
    Cancel,
    Retry,
    Supersede,
}

impl ExecutorLifecycleAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Timeout => "timeout",
            Self::Cancel => "cancel",
            Self::Retry => "retry",
            Self::Supersede => "supersede",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutorLifecycleRequest {
    pub issue_id: String,
    pub run_id: String,
    pub action: ExecutorLifecycleAction,
    pub actor: String,
    pub reason: String,
    #[serde(default)]
    pub retry_run_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutorLifecycleReceipt {
    pub version: String,
    pub issue_id: String,
    pub run_id: String,
    pub action: ExecutorLifecycleAction,
    pub actor: String,
    pub reason: String,
    pub run_status: String,
    pub issue_status: String,
    pub retry_run_id: Option<String>,
    pub superseded_run_id: Option<String>,
    pub receipt_path: String,
    pub created_at: u64,
}

pub fn create_executor_handoff_package(
    project_root: impl AsRef<Path>,
    request: ExecutorHandoffRequest,
) -> Result<ExecutorHandoffPackage> {
    let root = canonicalize_project_root(project_root)?;
    let issue = read_spec_issue(&root, &request.issue_id)?;
    let branch_name = request
        .branch_name
        .clone()
        .or_else(|| Some(format!("agentflow/direct/{}", request.issue_id)));
    let _run = create_task_run(
        &root,
        &request.issue_id,
        &request.run_id,
        &issue.workflow_ref,
        branch_name,
    )?;
    let handoff_path = executor_handoff_path(&root, &request.issue_id, &request.run_id)?;
    let handoff_rel = normalize_relative_to_root(&root, &handoff_path)?;
    let package = build_handoff_package(&root, &issue, &request, handoff_rel)?;
    write_json(&handoff_path, &package)?;
    update_task_run_launch_refs(&root, &request.issue_id, &request.run_id, &package)?;
    let _ = update_spec_issue_status(&root, &request.issue_id, SpecIssueStatus::InProgress)?;
    let _ = write_task_run_checkpoint(
        &root,
        &request.issue_id,
        &request.run_id,
        WorkflowFlowType::Work,
        "handoff-prepared",
        &format!("handoff-{}", package.handoff_id),
        Some(package.handoff_id.clone()),
        "Executor handoff package prepared; executor session remains transport.",
    )?;
    Ok(package)
}

pub fn check_executor_diff_boundary(
    project_root: impl AsRef<Path>,
    request: ExecutorDiffBoundaryRequest,
) -> Result<ExecutorDiffBoundaryReport> {
    let root = canonicalize_project_root(project_root)?;
    let handoff = read_executor_handoff_package(&root, &request.issue_id, &request.run_id)?;
    let changed_files = request
        .changed_files
        .iter()
        .map(|file| TaskChangedFile {
            path: file.path.clone(),
            change_type: file.change_type.clone(),
            insertions: file.insertions,
            deletions: file.deletions,
            sources: vec![TaskChangedFileSource::WorkingTree],
        })
        .collect::<Vec<_>>();
    let changed_paths = changed_files
        .iter()
        .map(|file| file.path.clone())
        .collect::<Vec<_>>();
    let out_of_scope_paths = changed_paths
        .iter()
        .filter(|path| !path_allowed(path, &handoff.allowed_surface))
        .cloned()
        .collect::<Vec<_>>();
    let denied_paths = changed_paths
        .iter()
        .filter(|path| path_allowed(path, &handoff.denied_surface))
        .cloned()
        .collect::<Vec<_>>();
    let mut boundary_failures = Vec::new();
    for path in &out_of_scope_paths {
        boundary_failures.push(format!("out-of-scope diff path: {path}"));
    }
    for path in &denied_paths {
        boundary_failures.push(format!("denied diff path: {path}"));
    }
    let changed_record = write_task_changed_files(
        &root,
        &request.issue_id,
        &request.run_id,
        changed_files,
        request.base_commit,
        request.head_commit,
        None,
        digest_lines(&changed_paths),
        digest_lines(&changed_paths),
        digest_lines(&changed_paths),
    )?;
    let report_path = executor_boundary_report_path(&root, &request.issue_id, &request.run_id)?;
    let report = ExecutorDiffBoundaryReport {
        version: EXECUTOR_DIFF_BOUNDARY_REPORT_VERSION.to_string(),
        issue_id: request.issue_id,
        run_id: request.run_id,
        status: if boundary_failures.is_empty() {
            "passed"
        } else {
            "failed"
        }
        .to_string(),
        allowed_surface: handoff.allowed_surface,
        denied_surface: handoff.denied_surface,
        changed_paths,
        out_of_scope_paths,
        denied_paths,
        boundary_failures,
        changed_files_path: normalize_relative_to_root(
            &root,
            agentflow_task_artifacts::task_changed_files_path(
                &root,
                &changed_record.issue_id,
                &changed_record.run_id,
            )?,
        )?,
        report_path: normalize_relative_to_root(&root, &report_path)?,
        checked_at: unix_timestamp_seconds(),
    };
    write_json(&report_path, &report)?;
    Ok(report)
}

pub fn capture_executor_evidence(
    project_root: impl AsRef<Path>,
    request: ExecutorEvidenceCaptureRequest,
) -> Result<ExecutorEvidenceCaptureReport> {
    let root = canonicalize_project_root(project_root)?;
    let handoff = read_executor_handoff_package(&root, &request.issue_id, &request.run_id)?;
    if request.commands.is_empty() {
        anyhow::bail!("executor evidence capture requires at least one command record");
    }
    for command in request.commands {
        let _ = write_task_command_record(
            &root,
            &request.issue_id,
            &request.run_id,
            TaskCommandInput {
                label: command.label,
                program: command.program,
                args: command.args,
                exit_code: command.exit_code,
                stdout: command.stdout,
                stderr: command.stderr,
            },
        )?;
    }
    let boundary_failures = if request.boundary_failures.is_empty() {
        load_boundary_failures(&root, &request.issue_id, &request.run_id).unwrap_or_default()
    } else {
        request.boundary_failures
    };
    let validation = write_task_validation_with_assessment(
        &root,
        &request.issue_id,
        &request.run_id,
        boundary_failures,
    )?;
    let evidence = write_task_evidence(&root, &request.issue_id, &request.run_id, request.summary)?;
    let report_path = executor_evidence_capture_path(&root, &request.issue_id, &request.run_id)?;
    let report = ExecutorEvidenceCaptureReport {
        version: EXECUTOR_EVIDENCE_CAPTURE_VERSION.to_string(),
        issue_id: request.issue_id,
        run_id: request.run_id,
        status: evidence.status.clone(),
        evidence_path: normalize_relative_path_string(
            PathBuf::from(".agentflow")
                .join("tasks")
                .join(&evidence.issue_id)
                .join("evidence")
                .join("evidence.json"),
        )?,
        validation_path: evidence.validation_path.clone(),
        command_paths: evidence.command_paths.clone(),
        changed_files_path: validation.changed_files_path,
        evidence,
        captured_at: unix_timestamp_seconds(),
    };
    write_json(&report_path, &report)?;
    let _ = write_task_run_checkpoint(
        &root,
        &report.issue_id,
        &report.run_id,
        WorkflowFlowType::Work,
        "evidence-captured",
        &format!("evidence-{}", handoff.handoff_id),
        Some(handoff.handoff_id),
        "Executor command, validation, diff and evidence records captured.",
    )?;
    Ok(report)
}

pub fn write_executor_result_to_issue(
    project_root: impl AsRef<Path>,
    request: ExecutorResultWritebackRequest,
) -> Result<ExecutorResultWritebackReport> {
    let root = canonicalize_project_root(project_root)?;
    let handoff = read_executor_handoff_package(&root, &request.issue_id, &request.run_id)?;
    let boundary = read_boundary_report(&root, &request.issue_id, &request.run_id)?;
    let evidence = agentflow_task_artifacts::load_task_evidence(&root, &request.issue_id)
        .context("executor result writeback requires evidence")?;
    let evidence_ready = evidence.status == "passed";
    let boundary_ready = boundary.status == "passed";
    let can_writeback = matches!(request.outcome, ExecutorResultOutcome::Success)
        && evidence_ready
        && boundary_ready;
    let mut blocked_reasons = Vec::new();
    if !evidence_ready {
        blocked_reasons.push("missing-or-failed-evidence".to_string());
    }
    if !boundary_ready {
        blocked_reasons.push("diff-boundary-failed".to_string());
    }
    if !matches!(request.outcome, ExecutorResultOutcome::Success) {
        blocked_reasons.push(format!("executor-outcome-{}", request.outcome.as_str()));
    }
    let closeout = TaskExecutorCloseout {
        version: TASK_EXECUTOR_CLOSEOUT_VERSION.to_string(),
        issue_id: request.issue_id.clone(),
        run_id: request.run_id.clone(),
        work_handoff: TaskExecutorWorkHandoff {
            role: handoff.executor_role.clone(),
            skill: "executor-adapter-real-execution".to_string(),
            allowed_surface: handoff.allowed_surface.clone(),
            expected_outputs: handoff.expected_outputs.clone(),
            evidence_policy: handoff.evidence_policy.clone(),
            forbidden_scope: handoff.denied_surface.clone(),
        },
        changed_files: boundary
            .changed_paths
            .iter()
            .map(|path| TaskChangedFile {
                path: path.clone(),
                change_type: "modified".to_string(),
                insertions: 0,
                deletions: 0,
                sources: vec![TaskChangedFileSource::WorkingTree],
            })
            .collect(),
        logs: request.logs,
        artifacts: request.artifacts,
        evidence_refs: vec![evidence.run_path.clone(), evidence.validation_path.clone()],
        result_status: request.outcome.task_result_status(),
        failure_reason: request
            .failure_reason
            .or_else(|| (!blocked_reasons.is_empty()).then(|| blocked_reasons.join("; "))),
        continuation_request: request.continuation_request,
        normalized_core_refs: TaskExecutorCoreRefs {
            evidence_refs: vec![evidence.run_path, evidence.validation_path],
            artifact_refs: vec![boundary.report_path.clone()],
            decision_refs: vec![handoff.writeback_target.clone()],
        },
        can_writeback,
        generated_at: unix_timestamp_seconds(),
    };
    let _ = write_task_executor_closeout(&root, &request.issue_id, &closeout)?;
    let run_status = request.outcome.run_status(can_writeback);
    let issue_status = request.outcome.issue_status(can_writeback);
    let _ = commit_task_run_writeback(
        &root,
        &request.issue_id,
        &request.run_id,
        run_status.clone(),
        if can_writeback { "done" } else { "blocked" },
        closeout.failure_reason.clone(),
        None,
    )?;
    let issue = update_spec_issue_status(&root, &request.issue_id, issue_status.clone())?;
    let writeback_path = executor_writeback_report_path(&root, &request.issue_id, &request.run_id)?;
    let report = ExecutorResultWritebackReport {
        version: EXECUTOR_RESULT_WRITEBACK_VERSION.to_string(),
        issue_id: request.issue_id,
        run_id: request.run_id,
        outcome: request.outcome,
        can_writeback,
        blocked_reasons,
        issue_status: issue.status.as_str().to_string(),
        run_status: match run_status {
            TaskRunStatus::Queued => "queued",
            TaskRunStatus::InProgress => "in_progress",
            TaskRunStatus::Validating => "validating",
            TaskRunStatus::Completed => "completed",
            TaskRunStatus::Failed => "failed",
            TaskRunStatus::Cancelled => "cancelled",
        }
        .to_string(),
        evidence_path: normalize_relative_path_string(
            PathBuf::from(".agentflow")
                .join("tasks")
                .join(&issue.issue_id)
                .join("evidence")
                .join("evidence.json"),
        )?,
        boundary_report_path: boundary.report_path,
        closeout_path: normalize_relative_to_root(
            &root,
            agentflow_task_artifacts::task_executor_closeout_path(
                &root,
                &issue.issue_id,
                &closeout.run_id,
            )?,
        )?,
        writeback_path: normalize_relative_to_root(&root, &writeback_path)?,
        generated_at: unix_timestamp_seconds(),
    };
    write_json(&writeback_path, &report)?;
    Ok(report)
}

pub fn record_executor_lifecycle(
    project_root: impl AsRef<Path>,
    request: ExecutorLifecycleRequest,
) -> Result<ExecutorLifecycleReceipt> {
    let root = canonicalize_project_root(project_root)?;
    let (run_status, issue_status, retry_run_id, superseded_run_id) = match request.action {
        ExecutorLifecycleAction::Timeout => {
            (TaskRunStatus::Failed, SpecIssueStatus::Blocked, None, None)
        }
        ExecutorLifecycleAction::Cancel => (
            TaskRunStatus::Cancelled,
            SpecIssueStatus::Cancel,
            None,
            None,
        ),
        ExecutorLifecycleAction::Retry => {
            let retry_run_id = request
                .retry_run_id
                .clone()
                .unwrap_or_else(|| format!("{}-retry", request.run_id));
            let issue = read_spec_issue(&root, &request.issue_id)?;
            let _ = create_task_run(
                &root,
                &request.issue_id,
                &retry_run_id,
                &issue.workflow_ref,
                Some(format!("agentflow/direct/{}-retry", request.issue_id)),
            )?;
            (
                TaskRunStatus::Failed,
                SpecIssueStatus::Todo,
                Some(retry_run_id),
                Some(request.run_id.clone()),
            )
        }
        ExecutorLifecycleAction::Supersede => (
            TaskRunStatus::Failed,
            SpecIssueStatus::Todo,
            request.retry_run_id.clone(),
            Some(request.run_id.clone()),
        ),
    };
    let _ = commit_task_run_writeback(
        &root,
        &request.issue_id,
        &request.run_id,
        run_status.clone(),
        request.action.as_str(),
        Some(request.reason.clone()),
        None,
    )?;
    let issue = update_spec_issue_status(&root, &request.issue_id, issue_status.clone())?;
    let receipt_path = executor_lifecycle_receipt_path(
        &root,
        &request.issue_id,
        &request.run_id,
        request.action.as_str(),
    )?;
    let receipt = ExecutorLifecycleReceipt {
        version: EXECUTOR_LIFECYCLE_RECEIPT_VERSION.to_string(),
        issue_id: request.issue_id,
        run_id: request.run_id,
        action: request.action,
        actor: request.actor,
        reason: request.reason,
        run_status: run_status_label(&run_status).to_string(),
        issue_status: issue.status.as_str().to_string(),
        retry_run_id,
        superseded_run_id,
        receipt_path: normalize_relative_to_root(&root, &receipt_path)?,
        created_at: unix_timestamp_seconds(),
    };
    write_json(&receipt_path, &receipt)?;
    Ok(receipt)
}

fn build_handoff_package(
    root: &Path,
    issue: &SpecIssue,
    request: &ExecutorHandoffRequest,
    handoff_path: String,
) -> Result<ExecutorHandoffPackage> {
    let handoff_id = format!("handoff-{}-{}", issue.issue_id, request.run_id);
    Ok(ExecutorHandoffPackage {
        version: EXECUTOR_HANDOFF_PACKAGE_VERSION.to_string(),
        handoff_id,
        issue_id: issue.issue_id.clone(),
        run_id: request.run_id.clone(),
        project_id: issue.project_id.clone(),
        source_issue_path: issue.system.path.clone(),
        workflow_ref: issue.workflow_ref.clone(),
        required_agent_role: issue.required_agent_role.provider_role_alias().to_string(),
        executor_adapter_id: request.executor_adapter_id.clone(),
        executor_role: request.executor_role.clone(),
        allowed_surface: normalize_surface(root, &issue.allowed_paths)?,
        denied_surface: normalize_surface(root, &issue.forbidden_paths)?,
        expected_outputs: vec![
            issue.expected_outputs.task_run_dir.clone(),
            issue.expected_outputs.evidence_path.clone(),
            issue
                .expected_outputs
                .validation_result_path
                .clone()
                .unwrap_or_else(|| {
                    format!(
                        ".agentflow/tasks/{}/runs/{}/validation.json",
                        issue.issue_id, request.run_id
                    )
                }),
            issue
                .expected_outputs
                .closeout_proof_path
                .clone()
                .unwrap_or_else(|| {
                    format!(
                        ".agentflow/tasks/{}/runs/{}/review/executor-closeout.json",
                        issue.issue_id, request.run_id
                    )
                }),
        ],
        evidence_policy: "commands+validation+diff-boundary-required".to_string(),
        acceptance_criteria: vec![
            issue.summary.clone(),
            "Evidence pack must be ready.".to_string(),
            "Diff boundary must pass before writeback.".to_string(),
        ],
        validation_commands: issue.validation_commands.clone(),
        writeback_target: issue.system.path.clone(),
        authority_boundary: "executor-session-is-transport-spec-issue-is-authority".to_string(),
        session_is_authority: false,
        handoff_path,
        created_at: unix_timestamp_seconds(),
    })
}

fn update_task_run_launch_refs(
    root: &Path,
    issue_id: &str,
    run_id: &str,
    package: &ExecutorHandoffPackage,
) -> Result<TaskRun> {
    let run_path = agentflow_task_artifacts::task_run_dir(root, issue_id, run_id)?.join("run.json");
    let mut run: TaskRun = read_json(&run_path)?;
    let now = unix_timestamp_seconds();
    run.status = TaskRunStatus::InProgress;
    run.provider = Some(package.executor_adapter_id.clone());
    run.session_owner = Some(package.executor_role.clone());
    run.session_id = Some(package.handoff_id.clone());
    run.session_status = Some("handoff-prepared".to_string());
    run.working_directory = Some(root.display().to_string());
    run.workspace_root = Some(root.display().to_string());
    run.worktree_root = Some(root.display().to_string());
    run.runtime_root = Some(
        root.join(".agentflow")
            .join("tasks")
            .join(issue_id)
            .join("runs")
            .join(run_id)
            .display()
            .to_string(),
    );
    run.evidence_root = Some(
        root.join(".agentflow")
            .join("tasks")
            .join(issue_id)
            .join("evidence")
            .display()
            .to_string(),
    );
    run.launch_request_path = Some(package.handoff_path.clone());
    run.started_at.get_or_insert(now);
    run.last_heartbeat_at = Some(now);
    run.attempt_count = Some(1);
    run.retry_policy = Some("manual-retry-new-run".to_string());
    run.max_attempts = Some(2);
    run.retryable = Some(true);
    run.updated_at = now;
    write_json(&run_path, &run)?;
    Ok(run)
}

fn normalize_surface(root: &Path, paths: &[String]) -> Result<Vec<String>> {
    if paths.is_empty() {
        return Ok(vec!["docs/**".to_string()]);
    }
    paths
        .iter()
        .map(|path| {
            if path.starts_with("workspace://") || path.contains('*') {
                Ok(path.clone())
            } else {
                normalize_relative_path_string(PathBuf::from(path))
            }
        })
        .map(|value| value.or_else(|_| normalize_relative_to_root(root, root.join("docs"))))
        .collect()
}

fn path_allowed(path: &str, surface: &[String]) -> bool {
    surface.iter().any(|pattern| surface_matches(pattern, path))
}

fn surface_matches(pattern: &str, path: &str) -> bool {
    if pattern == "**" || pattern == "*" {
        return true;
    }
    if let Some(prefix) = pattern.strip_suffix("/**") {
        return path == prefix || path.starts_with(&format!("{prefix}/"));
    }
    if let Some(prefix) = pattern.strip_suffix("**") {
        return path.starts_with(prefix);
    }
    path == pattern || path.starts_with(&format!("{pattern}/"))
}

fn read_executor_handoff_package(
    root: &Path,
    issue_id: &str,
    run_id: &str,
) -> Result<ExecutorHandoffPackage> {
    read_json(&executor_handoff_path(root, issue_id, run_id)?)
}

fn read_boundary_report(
    root: &Path,
    issue_id: &str,
    run_id: &str,
) -> Result<ExecutorDiffBoundaryReport> {
    read_json(&executor_boundary_report_path(root, issue_id, run_id)?)
}

fn load_boundary_failures(root: &Path, issue_id: &str, run_id: &str) -> Result<Vec<String>> {
    Ok(read_boundary_report(root, issue_id, run_id)?.boundary_failures)
}

fn executor_handoff_path(root: &Path, issue_id: &str, run_id: &str) -> Result<PathBuf> {
    Ok(
        agentflow_task_artifacts::task_run_dir(root, issue_id, run_id)?
            .join("launch")
            .join("agent-request.json"),
    )
}

fn executor_boundary_report_path(root: &Path, issue_id: &str, run_id: &str) -> Result<PathBuf> {
    Ok(
        agentflow_task_artifacts::task_run_dir(root, issue_id, run_id)?
            .join("evidence")
            .join("diff-boundary.json"),
    )
}

fn executor_evidence_capture_path(root: &Path, issue_id: &str, run_id: &str) -> Result<PathBuf> {
    Ok(
        agentflow_task_artifacts::task_run_dir(root, issue_id, run_id)?
            .join("evidence")
            .join("capture.json"),
    )
}

fn executor_writeback_report_path(root: &Path, issue_id: &str, run_id: &str) -> Result<PathBuf> {
    Ok(
        agentflow_task_artifacts::task_run_dir(root, issue_id, run_id)?
            .join("review")
            .join("executor-writeback.json"),
    )
}

fn executor_lifecycle_receipt_path(
    root: &Path,
    issue_id: &str,
    run_id: &str,
    action: &str,
) -> Result<PathBuf> {
    Ok(
        agentflow_task_artifacts::task_run_dir(root, issue_id, run_id)?
            .join("lifecycle")
            .join(format!("{action}.json")),
    )
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

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T> {
    let payload = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    Ok(serde_json::from_str(&payload)?)
}

fn digest_lines(values: &[String]) -> String {
    let mut hasher = Sha256::new();
    for value in values {
        hasher.update(value.as_bytes());
        hasher.update(b"\n");
    }
    format!("{:x}", hasher.finalize())
}

fn run_status_label(status: &TaskRunStatus) -> &'static str {
    match status {
        TaskRunStatus::Queued => "queued",
        TaskRunStatus::InProgress => "in_progress",
        TaskRunStatus::Validating => "validating",
        TaskRunStatus::Completed => "completed",
        TaskRunStatus::Failed => "failed",
        TaskRunStatus::Cancelled => "cancelled",
    }
}

fn unix_timestamp_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn default_executor_adapter_id() -> String {
    "codex".to_string()
}

fn default_executor_role() -> String {
    "build-agent".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use agentflow_spec::{write_spec_issue, SpecIssueDraft};

    #[test]
    fn executor_handoff_and_writeback_close_issue_when_evidence_and_boundary_pass() {
        let dir = tempfile::tempdir().unwrap();
        seed_requirement(dir.path());
        let issue = seed_issue(dir.path(), "AF-TEST-001", vec!["docs/**".to_string()]);

        let handoff = create_executor_handoff_package(
            dir.path(),
            ExecutorHandoffRequest {
                issue_id: issue.issue_id.clone(),
                run_id: "run-001".to_string(),
                executor_adapter_id: "codex".to_string(),
                executor_role: "build-agent".to_string(),
                session_id: None,
                branch_name: None,
                working_directory: None,
            },
        )
        .unwrap();
        assert!(!handoff.session_is_authority);

        let boundary = check_executor_diff_boundary(
            dir.path(),
            ExecutorDiffBoundaryRequest {
                issue_id: issue.issue_id.clone(),
                run_id: "run-001".to_string(),
                changed_files: vec![ExecutorDiffInputFile {
                    path: "docs/requirements/example.md".to_string(),
                    change_type: "modified".to_string(),
                    insertions: 1,
                    deletions: 0,
                }],
                base_commit: None,
                head_commit: None,
            },
        )
        .unwrap();
        assert_eq!(boundary.status, "passed");

        let evidence = capture_executor_evidence(
            dir.path(),
            ExecutorEvidenceCaptureRequest {
                issue_id: issue.issue_id.clone(),
                run_id: "run-001".to_string(),
                summary: "validation passed".to_string(),
                commands: vec![ExecutorCommandEvidenceInput {
                    label: "unit".to_string(),
                    program: "cargo".to_string(),
                    args: vec!["test".to_string()],
                    exit_code: Some(0),
                    stdout: "ok".to_string(),
                    stderr: String::new(),
                }],
                boundary_failures: Vec::new(),
            },
        )
        .unwrap();
        assert_eq!(evidence.status, "passed");

        let writeback = write_executor_result_to_issue(
            dir.path(),
            ExecutorResultWritebackRequest {
                issue_id: issue.issue_id.clone(),
                run_id: "run-001".to_string(),
                outcome: ExecutorResultOutcome::Success,
                summary: "done".to_string(),
                artifacts: vec!["docs/requirements/example.md".to_string()],
                logs: vec!["validated".to_string()],
                failure_reason: None,
                continuation_request: None,
            },
        )
        .unwrap();
        assert!(writeback.can_writeback);
        assert_eq!(writeback.issue_status, "done");
    }

    #[test]
    fn out_of_scope_diff_blocks_writeback() {
        let dir = tempfile::tempdir().unwrap();
        seed_requirement(dir.path());
        let issue = seed_issue(dir.path(), "AF-TEST-002", vec!["docs/**".to_string()]);
        create_executor_handoff_package(
            dir.path(),
            ExecutorHandoffRequest {
                issue_id: issue.issue_id.clone(),
                run_id: "run-001".to_string(),
                executor_adapter_id: "codex".to_string(),
                executor_role: "build-agent".to_string(),
                session_id: None,
                branch_name: None,
                working_directory: None,
            },
        )
        .unwrap();
        let boundary = check_executor_diff_boundary(
            dir.path(),
            ExecutorDiffBoundaryRequest {
                issue_id: issue.issue_id.clone(),
                run_id: "run-001".to_string(),
                changed_files: vec![ExecutorDiffInputFile {
                    path: "src/main.rs".to_string(),
                    change_type: "modified".to_string(),
                    insertions: 1,
                    deletions: 0,
                }],
                base_commit: None,
                head_commit: None,
            },
        )
        .unwrap();
        assert_eq!(boundary.status, "failed");
        assert!(!boundary.boundary_failures.is_empty());
    }

    #[test]
    fn retry_creates_new_run_without_mutating_original_evidence() {
        let dir = tempfile::tempdir().unwrap();
        seed_requirement(dir.path());
        let issue = seed_issue(dir.path(), "AF-TEST-003", vec!["docs/**".to_string()]);
        create_executor_handoff_package(
            dir.path(),
            ExecutorHandoffRequest {
                issue_id: issue.issue_id.clone(),
                run_id: "run-001".to_string(),
                executor_adapter_id: "codex".to_string(),
                executor_role: "build-agent".to_string(),
                session_id: None,
                branch_name: None,
                working_directory: None,
            },
        )
        .unwrap();
        let receipt = record_executor_lifecycle(
            dir.path(),
            ExecutorLifecycleRequest {
                issue_id: issue.issue_id.clone(),
                run_id: "run-001".to_string(),
                action: ExecutorLifecycleAction::Retry,
                actor: "build-agent".to_string(),
                reason: "retry with fixed input".to_string(),
                retry_run_id: Some("run-002".to_string()),
            },
        )
        .unwrap();
        assert_eq!(receipt.retry_run_id.as_deref(), Some("run-002"));
        assert!(
            agentflow_task_artifacts::task_run_dir(dir.path(), &issue.issue_id, "run-002")
                .unwrap()
                .join("run.json")
                .is_file()
        );
    }

    fn seed_requirement(root: &Path) {
        let path = root.join("docs/requirements/example.md");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(
            path,
            "# Example Requirement\n\n## Summary\nSeed requirement for executor tests.\n",
        )
        .unwrap();
    }

    fn seed_issue(root: &Path, issue_id: &str, allowed_paths: Vec<String>) -> SpecIssue {
        let mut draft = SpecIssueDraft::new(issue_id);
        draft.title = Some("Executor test issue".to_string());
        draft.summary = Some("Executor test summary".to_string());
        draft.project_id = Some("project-test".to_string());
        draft.allowed_paths = allowed_paths;
        draft.forbidden_paths = vec!["secrets/**".to_string()];
        draft.validation_commands = vec!["cargo test".to_string()];
        let issue = agentflow_spec::issue_from_requirement(
            root,
            Path::new("docs/requirements/example.md"),
            draft,
        )
        .unwrap();
        write_spec_issue(root, &issue).unwrap();
        issue
    }
}
