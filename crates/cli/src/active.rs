//! Active CLI boundary.
//!
//! Active commands are narrow wrappers around the current workspace crates.
//! They must not call archived 2026-05 writers.

use agentflow_event_store::{
    append_task_event_once, EventActor, EventStateTransition, TaskEventDraft,
};
use agentflow_mcp::{
    find_session_snapshot_by_run, query_closeout_attestation, McpCloseoutAttestation,
    McpCloseoutIssueAttestation,
};
use agentflow_runtime_api::{
    assert_issue_mark_done_allowed, assert_issue_transition, assert_run_transition,
};
use agentflow_spec::{read_spec_issue, update_spec_issue_status, SpecIssueStatus};
use agentflow_task_artifacts::{
    load_task_changed_files, load_task_evidence, load_task_run, load_task_validation,
    task_changed_files_path, task_evidence_dir, task_run_dir, update_task_run_status,
    write_task_changed_files, write_task_command_record, write_task_evidence,
    write_task_evidence_gate_decision, write_task_validation_with_assessment, TaskChangedFile,
    TaskChangedFileSource, TaskCommandInput, TaskEvidence, TaskEvidenceEntry,
    TaskEvidenceEntryStatus, TaskEvidenceGateDecision, TaskRun, TaskRunStatus,
};
use agentflow_task_loop::{AgentLaunchPayload, TaskLoop, TaskLoopLaunch, AGENT_LAUNCH_REQUESTED};
use agentflow_workflow_core::{WorkflowAgentRole, WorkflowFlowType};
use anyhow::{Context, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};
use serde::Deserialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::SystemTime,
};

const CLI_FRESHNESS_PATHS: [&str; 11] = [
    "Cargo.toml",
    "Cargo.lock",
    "crates/cli/src",
    "crates/event-store/src",
    "crates/projection/src",
    "crates/release/src",
    "crates/spec/src",
    "crates/state/src",
    "crates/task-artifacts/src",
    "crates/task-loop/src",
    "crates/agent-dispatcher/src",
];

#[derive(Debug, Clone)]
pub(crate) struct BuildAgentStart {
    pub issue_id: String,
    pub run_id: String,
    pub stage: String,
    pub branch_name: Option<String>,
    pub project_id: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct BuildAgentCloseoutProof {
    pub issue_id: String,
    pub run_id: String,
    pub merged: bool,
    pub issue_closed: bool,
    pub proof_path: PathBuf,
}

#[derive(Debug, Clone)]
pub(crate) struct BuildAgentLaunchClaim {
    pub event_id: String,
    pub issue_id: String,
    pub run_id: String,
    pub branch_name: Option<String>,
    pub launch_request_path: PathBuf,
}

#[derive(Debug, Clone)]
pub(crate) struct BuildAgentReview {
    pub issue_id: String,
    pub run_id: String,
    pub run_status: String,
    pub validation_passed: bool,
    pub evidence_path: PathBuf,
}

#[derive(Debug, Clone)]
pub(crate) struct BuildAgentCompletionOutcome {
    pub issue_id: String,
    pub run_id: String,
    pub run_status: String,
    pub validation_passed: bool,
    pub evidence_path: PathBuf,
    pub closeout_proof_path: PathBuf,
    pub next_launch: Option<TaskLoopLaunch>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BuildAgentCompletionRequest {
    issue_id: String,
    #[serde(default)]
    run_id: Option<String>,
}

#[derive(Debug, Clone)]
struct TrustedChangedFilesSnapshot {
    files: Vec<TaskChangedFile>,
    base_commit: Option<String>,
    head_commit: Option<String>,
    tree_sha: Option<String>,
    working_tree_hash: String,
    patch_sha256: String,
    file_content_sha256: String,
}

pub(crate) fn complete_build_agent_issue_from_request(
    root: &Path,
    request_path: &Path,
) -> Result<BuildAgentCompletionOutcome> {
    assert_current_cli_is_fresh(root)?;
    let request = read_completion_request(request_path, "completion")?;
    let review = if let Some(run_id) = request
        .run_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        if let Some(review) = load_prepared_review_if_available(root, &request.issue_id, run_id)? {
            review
        } else {
            ensure_review_prepared(root, request.clone())?
        }
    } else {
        ensure_review_prepared(root, request.clone())?
    };
    let proof_path = closeout_proof_path(root, &review.issue_id, &review.run_id);
    let proof = if proof_path.is_file() {
        load_closeout_proof(root, &review.issue_id, &review.run_id)?
    } else {
        json!({})
    };
    let issue = read_spec_issue(root, &review.issue_id)?;
    let completed_project_id = issue.project_id.clone();
    let evidence = load_task_evidence(root, &review.issue_id)?;
    let gate_decision =
        build_evidence_gate_decision(root, &issue, &review.run_id, &evidence, &proof)?;
    write_task_evidence_gate_decision(root, &review.issue_id, &gate_decision)?;
    if !gate_decision.passed {
        let _ = agentflow_projection::rebuild_projections(root)?;
        agentflow_state::refresh_state(root)?;
        anyhow::bail!(
            "build agent completion requires evidence gate to pass for {} {}: {}",
            review.issue_id,
            review.run_id,
            gate_decision.blockers.join("; ")
        );
    }
    let _ = assert_issue_mark_done_allowed(root, &issue.issue_id, &issue.status)?;
    write_json(&proof_path, &proof)?;
    append_task_event_once(
        root,
        TaskEventDraft {
            flow_type: WorkflowFlowType::Work,
            aggregate_type: "issue".to_string(),
            aggregate_id: issue.issue_id.clone(),
            project_id: issue.project_id.clone(),
            issue_id: Some(issue.issue_id.clone()),
            run_id: Some(review.run_id.clone()),
            event_type: "issue.completed".to_string(),
            authority_role: Some(WorkflowAgentRole::System),
            actor: EventActor {
                role: "build-agent".to_string(),
                kind: "system".to_string(),
            },
            state: Some(EventStateTransition {
                from_state: "in_review".to_string(),
                to_state: "done".to_string(),
            }),
            correlation_id: Some(format!("corr-{}", issue.issue_id)),
            causation_id: None,
            payload: json!({
                "issueId": issue.issue_id,
                "projectId": issue.project_id,
                "runId": review.run_id,
                "evidencePath": task_evidence_path(&review.issue_id),
                "closeoutProofPath": relative_path(root, &proof_path),
                "provider": proof.get("provider").cloned().unwrap_or(serde_json::Value::Null),
                "mergeMode": proof.get("mergeMode").cloned().unwrap_or(serde_json::Value::Null),
                "repositoryFullName": proof.get("repositoryFullName").cloned().unwrap_or(serde_json::Value::Null),
                "remoteUrl": proof.get("remoteUrl").cloned().unwrap_or(serde_json::Value::Null),
                "prUrl": proof.get("prUrl").cloned().unwrap_or(serde_json::Value::Null),
                "sourceBranch": proof.get("sourceBranch").cloned().unwrap_or(serde_json::Value::Null),
                "targetBranch": proof.get("targetBranch").cloned().unwrap_or(serde_json::Value::Null),
                "baseSha": proof.get("baseSha").cloned().unwrap_or(serde_json::Value::Null),
                "headSha": proof.get("headSha").cloned().unwrap_or(serde_json::Value::Null),
                "mergeCommitSha": proof.get("mergeCommitSha").cloned().unwrap_or(serde_json::Value::Null),
                "issueClosed": proof.get("issueClosed").cloned().unwrap_or(serde_json::Value::Null),
                "issueClosedAt": proof.get("issueClosedAt").cloned().unwrap_or(serde_json::Value::Null),
                "evidenceHeadSha": proof.get("evidenceHeadSha").cloned().unwrap_or(serde_json::Value::Null),
                "evidenceHash": proof.get("evidenceHash").cloned().unwrap_or(serde_json::Value::Null),
            }),
            artifact_refs: vec![
                task_evidence_path(&review.issue_id),
                relative_path(root, &proof_path),
            ],
            idempotency_key: Some(format!(
                "issue.completed:{}:{}",
                issue.issue_id, review.run_id
            )),
        },
    )?;
    let _ = update_spec_issue_status(root, &issue.issue_id, SpecIssueStatus::Done)?;
    let _ = agentflow_projection::rebuild_projections(root)?;
    let next_launch = if let Some(project_id) = completed_project_id.as_deref() {
        TaskLoop::new(project_id)
            .tick(root, "codex")?
            .map(|tick| tick.launch)
    } else {
        None
    };
    if next_launch.is_some() {
        let _ = agentflow_projection::rebuild_projections(root)?;
    }
    agentflow_state::refresh_state(root)?;
    Ok(BuildAgentCompletionOutcome {
        issue_id: review.issue_id,
        run_id: review.run_id,
        run_status: review.run_status,
        validation_passed: review.validation_passed,
        evidence_path: evidence_path(root, &evidence),
        closeout_proof_path: proof_path,
        next_launch,
    })
}

fn load_prepared_review_if_available(
    root: &Path,
    issue_id: &str,
    run_id: &str,
) -> Result<Option<BuildAgentReview>> {
    let run = load_task_run(root, issue_id, run_id)?;
    if run.status != TaskRunStatus::Completed {
        return Ok(None);
    }
    let issue = read_spec_issue(root, issue_id)?;
    if issue.status != SpecIssueStatus::InReview {
        anyhow::bail!(
            "completed run {} requires issue {} to be in_review before completion, found {}",
            run_id,
            issue_id,
            issue.status.as_str()
        );
    }
    let evidence = load_task_evidence(root, issue_id)?;
    Ok(Some(BuildAgentReview {
        issue_id: issue_id.to_string(),
        run_id: run_id.to_string(),
        run_status: task_run_status_as_str(&run).to_string(),
        validation_passed: evidence.status == "passed",
        evidence_path: evidence_path(root, &evidence),
    }))
}

pub(crate) fn prepare_build_agent_review_from_request(
    root: &Path,
    request_path: &Path,
) -> Result<BuildAgentReview> {
    assert_current_cli_is_fresh(root)?;
    let request = read_completion_request(request_path, "review preparation")?;
    ensure_review_prepared(root, request)
}

pub(crate) fn start_build_agent_issue(root: &Path, issue_id: &str) -> Result<BuildAgentStart> {
    assert_current_cli_is_fresh(root)?;
    let issue_id = issue_id.trim();
    if issue_id.is_empty() {
        anyhow::bail!("build agent start requires issueId");
    }
    let tick = TaskLoop::start_issue(root, issue_id, "codex")?;
    let _ = agentflow_projection::rebuild_projections(root)?;
    agentflow_state::refresh_state(root)?;
    Ok(BuildAgentStart {
        issue_id: tick.launch.issue_id,
        run_id: tick.launch.run_id,
        stage: "in_progress".to_string(),
        branch_name: Some(tick.launch.branch_name),
        project_id: tick.launch.project_id,
    })
}

pub(crate) fn claim_next_build_agent_launch(root: &Path) -> Result<Option<BuildAgentLaunchClaim>> {
    assert_current_cli_is_fresh(root)?;
    claim_next_build_agent_launch_with_dispatcher(
        root,
        &agentflow_agent_dispatcher::AgentDispatcher::with_default_providers(),
    )
}

fn claim_next_build_agent_launch_with_dispatcher(
    root: &Path,
    dispatcher: &agentflow_agent_dispatcher::AgentDispatcher,
) -> Result<Option<BuildAgentLaunchClaim>> {
    let Some(claim) = dispatcher.claim_next_launch(root)? else {
        return Ok(None);
    };
    let payload = load_agent_launch_payload(root, &claim.run_id)?;
    let launch_request_path = root.join(&payload.launch_request_path);
    if !launch_request_path.is_file() {
        anyhow::bail!(
            "build agent launch request is missing: {}",
            launch_request_path.display()
        );
    }
    Ok(Some(BuildAgentLaunchClaim {
        event_id: claim.created_event_id,
        issue_id: claim.issue_id,
        run_id: claim.run_id,
        branch_name: Some(payload.branch_name),
        launch_request_path,
    }))
}

fn load_agent_launch_payload(root: &Path, run_id: &str) -> Result<AgentLaunchPayload> {
    let event = agentflow_event_store::load_task_events(root)?
        .into_iter()
        .find(|event| {
            event.event_type == AGENT_LAUNCH_REQUESTED && event.run_id.as_deref() == Some(run_id)
        })
        .ok_or_else(|| anyhow::anyhow!("missing agent launch request for run {run_id}"))?;
    serde_json::from_value(event.payload)
        .with_context(|| format!("parse agent launch payload {}", event.event_id))
}

pub(crate) fn write_build_agent_closeout_proof(
    root: &Path,
    issue_id: &str,
    run_id: &str,
    provider: &str,
    merge_mode: &str,
    remote_url: Option<String>,
    provider_issue_refs: Vec<String>,
    attestation_path: Option<&Path>,
) -> Result<BuildAgentCloseoutProof> {
    assert_current_cli_is_fresh(root)?;
    let issue =
        read_spec_issue(root, issue_id).with_context(|| format!("load spec issue {issue_id}"))?;
    let run = load_task_run(root, &issue.issue_id, run_id)?;
    if run.issue_id != issue.issue_id {
        anyhow::bail!(
            "closeout proof issueId mismatch: request {}, run {}",
            issue.issue_id,
            run.issue_id
        );
    }
    let attestation = if let Some(attestation_path) = attestation_path {
        read_closeout_attestation(attestation_path)?
    } else {
        let review_ref = remote_url.clone().ok_or_else(|| {
            anyhow::anyhow!("closeout proof requires provider review ref / remote-url")
        })?;
        query_closeout_attestation(root, provider, &review_ref, &provider_issue_refs)?
    };
    write_build_agent_closeout_proof_from_attestation(root, &issue, run_id, merge_mode, attestation)
}

fn write_build_agent_closeout_proof_from_attestation(
    root: &Path,
    issue: &agentflow_spec::SpecIssue,
    run_id: &str,
    merge_mode: &str,
    attestation: McpCloseoutAttestation,
) -> Result<BuildAgentCloseoutProof> {
    let current_issue = read_spec_issue(root, &issue.issue_id)?;
    if current_issue.status != SpecIssueStatus::InReview {
        anyhow::bail!(
            "closeout proof requires issue {} to be in_review, found {}",
            current_issue.issue_id,
            current_issue.status.as_str()
        );
    }
    let proof_path = closeout_proof_path(root, &issue.issue_id, run_id);
    let run = load_task_run(root, &issue.issue_id, run_id)?;
    let evidence = load_task_evidence(root, &issue.issue_id)?;
    let provider_name = attestation.provider.clone();
    let review_ref = attestation.review_ref.clone();
    let review_url = attestation
        .review_url
        .clone()
        .ok_or_else(|| anyhow::anyhow!("closeout proof requires pr_url"))?;
    let repository_full_name = attestation
        .repository_full_name
        .clone()
        .ok_or_else(|| anyhow::anyhow!("closeout proof requires repository_full_name"))?;
    let source_branch = attestation
        .source_branch
        .clone()
        .ok_or_else(|| anyhow::anyhow!("closeout proof requires source_branch"))?;
    let target_branch = attestation
        .target_branch
        .clone()
        .ok_or_else(|| anyhow::anyhow!("closeout proof requires target_branch"))?;
    let base_sha = attestation
        .base_sha
        .clone()
        .ok_or_else(|| anyhow::anyhow!("closeout proof requires base_sha"))?;
    let head_sha = attestation
        .head_sha
        .clone()
        .ok_or_else(|| anyhow::anyhow!("closeout proof requires head_sha"))?;
    let merged = attestation.merged;
    let merged_at = if merged {
        Some(
            attestation
                .merged_at
                .clone()
                .ok_or_else(|| anyhow::anyhow!("closeout proof requires merged_at"))?,
        )
    } else {
        attestation.merged_at.clone()
    };
    let merge_commit_sha = if merged {
        Some(
            attestation
                .merge_commit_sha
                .clone()
                .ok_or_else(|| anyhow::anyhow!("closeout proof requires merge_commit_sha"))?,
        )
    } else {
        attestation.merge_commit_sha.clone()
    };
    let issue_closed = attestation.issue_closed;
    let issues = attestation.issues.clone();
    let queried_at = attestation.queried_at;
    let closed_at = if issue_closed {
        Some(
            latest_closed_at(&issues)
                .ok_or_else(|| anyhow::anyhow!("closeout proof requires issue_closed_at"))?,
        )
    } else {
        latest_closed_at(&issues)
    };
    let provider_issue_url = issues
        .iter()
        .find_map(|attested_issue| attested_issue.issue_url.clone())
        .ok_or_else(|| anyhow::anyhow!("closeout proof requires provider_issue_url"))?;
    let evidence_head_sha = evidence
        .head_commit
        .clone()
        .ok_or_else(|| anyhow::anyhow!("closeout proof requires evidence head commit"))?;
    let evidence_hash = sha256_hex(&serde_json::to_vec(&evidence)?);
    ensure_attested_url_matches_repository(
        &provider_name,
        &repository_full_name,
        &review_url,
        CloseoutUrlKind::Review,
    )?;
    ensure_attested_url_matches_repository(
        &provider_name,
        &repository_full_name,
        &provider_issue_url,
        CloseoutUrlKind::Issue,
    )?;
    if run.branch_name.as_deref() != Some(source_branch.as_str()) {
        anyhow::bail!(
            "closeout proof source branch mismatch: run {}, attestation {}",
            run.branch_name.as_deref().unwrap_or("none"),
            source_branch
        );
    }
    if evidence_head_sha != head_sha {
        anyhow::bail!(
            "closeout proof head sha mismatch: evidence {}, attestation {}",
            evidence_head_sha,
            head_sha
        );
    }
    write_json(
        &proof_path,
        &json!({
            "version": "task-closeout-proof.v3",
            "issueId": issue.issue_id,
            "projectId": issue.project_id,
            "runId": run_id,
            "provider": &provider_name,
            "mergeMode": merge_mode,
            "repositoryFullName": &repository_full_name,
            "providerIssueUrl": &provider_issue_url,
            "reviewRef": &review_ref,
            "remoteUrl": &review_url,
            "prUrl": &review_url,
            "sourceBranch": &source_branch,
            "targetBranch": &target_branch,
            "baseSha": &base_sha,
            "headSha": &head_sha,
            "mergeCommitSha": &merge_commit_sha,
            "merged": merged,
            "mergedAt": &merged_at,
            "issueClosed": issue_closed,
            "issueClosedAt": &closed_at,
            "evidenceHeadSha": &evidence_head_sha,
            "evidenceHash": &evidence_hash,
            "issues": &issues,
            "queriedAt": queried_at,
            "publicDeliveryWritten": false,
        }),
    )?;
    append_task_event_once(
        root,
        TaskEventDraft {
            flow_type: WorkflowFlowType::Work,
            aggregate_type: "issue".to_string(),
            aggregate_id: issue.issue_id.clone(),
            project_id: issue.project_id.clone(),
            issue_id: Some(issue.issue_id.clone()),
            run_id: Some(run_id.to_string()),
            event_type: "issue.closeout.proof.recorded".to_string(),
            authority_role: Some(WorkflowAgentRole::WorkAgent),
            actor: EventActor {
                role: "build-agent".to_string(),
                kind: "system".to_string(),
            },
            state: None,
            correlation_id: Some(format!("corr-{}", issue.issue_id)),
            causation_id: None,
            payload: json!({
                "issueId": issue.issue_id,
                "projectId": issue.project_id,
                "runId": run_id,
                "provider": &provider_name,
                "mergeMode": merge_mode,
                "repositoryFullName": &repository_full_name,
                "providerIssueUrl": &provider_issue_url,
                "reviewRef": &review_ref,
                "remoteUrl": &review_url,
                "prUrl": &review_url,
                "sourceBranch": &source_branch,
                "targetBranch": &target_branch,
                "baseSha": &base_sha,
                "headSha": &head_sha,
                "mergeCommitSha": &merge_commit_sha,
                "merged": merged,
                "mergedAt": &merged_at,
                "issueClosed": issue_closed,
                "issueClosedAt": &closed_at,
                "evidenceHeadSha": &evidence_head_sha,
                "evidenceHash": &evidence_hash,
                "issues": &issues,
                "queriedAt": queried_at,
                "publicDeliveryWritten": false,
            }),
            artifact_refs: vec![relative_path(root, &proof_path)],
            idempotency_key: Some(format!(
                "issue.closeout-proof.recorded:{}:{run_id}",
                issue.issue_id
            )),
        },
    )?;
    let _ = agentflow_projection::rebuild_projections(root)?;
    agentflow_state::refresh_state(root)?;
    Ok(BuildAgentCloseoutProof {
        issue_id: issue.issue_id.clone(),
        run_id: run_id.to_string(),
        merged,
        issue_closed,
        proof_path,
    })
}

fn read_completion_request(
    request_path: &Path,
    label: &str,
) -> Result<BuildAgentCompletionRequest> {
    let raw = fs::read_to_string(request_path).with_context(|| {
        format!(
            "read build agent {label} request {}",
            request_path.display()
        )
    })?;
    serde_json::from_str(&raw).with_context(|| {
        format!(
            "parse build agent {label} request {}",
            request_path.display()
        )
    })
}

fn read_closeout_attestation(attestation_path: &Path) -> Result<McpCloseoutAttestation> {
    let raw = fs::read_to_string(attestation_path)
        .with_context(|| format!("read closeout attestation {}", attestation_path.display()))?;
    serde_json::from_str(&raw)
        .with_context(|| format!("parse closeout attestation {}", attestation_path.display()))
}

fn ensure_review_prepared(
    root: &Path,
    request: BuildAgentCompletionRequest,
) -> Result<BuildAgentReview> {
    let issue_id = request.issue_id.trim();
    if issue_id.is_empty() {
        anyhow::bail!("build agent review preparation requires issueId");
    }
    let run_id = request
        .run_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| anyhow::anyhow!("build agent review preparation requires runId"))?;

    let issue =
        read_spec_issue(root, issue_id).with_context(|| format!("load spec issue {issue_id}"))?;
    if issue.validation_commands.is_empty() {
        anyhow::bail!("spec issue {issue_id} is missing validation commands");
    }
    let run = load_task_run(root, issue_id, run_id)?;
    if run.issue_id != issue.issue_id {
        anyhow::bail!(
            "build agent review preparation issueId mismatch: request {}, run {}",
            issue.issue_id,
            run.issue_id
        );
    }
    let _ = assert_run_transition(&run.status, "runValidation")?;

    reset_review_artifacts(root, issue_id, run_id)?;
    update_task_run_status(root, issue_id, run_id, TaskRunStatus::Validating)?;
    for (index, command) in issue.validation_commands.iter().enumerate() {
        let result = run_validation_command(root, command)?;
        write_task_command_record(
            root,
            issue_id,
            run_id,
            TaskCommandInput {
                label: format!("validation-{:03}", index + 1),
                program: "sh".to_string(),
                args: vec!["-lc".to_string(), command.clone()],
                exit_code: Some(result.exit_code),
                stdout: result.stdout,
                stderr: result.stderr,
            },
        )?;
    }
    let changed_files = collect_trusted_changed_files(root, issue_id, run_id)?;
    write_task_changed_files(
        root,
        issue_id,
        run_id,
        changed_files.files.clone(),
        changed_files.base_commit.clone(),
        changed_files.head_commit.clone(),
        changed_files.tree_sha.clone(),
        changed_files.working_tree_hash.clone(),
        changed_files.patch_sha256.clone(),
        changed_files.file_content_sha256.clone(),
    )?;
    let boundary_failures = validate_changed_file_boundaries(&issue, &changed_files.files)?;
    let validation =
        write_task_validation_with_assessment(root, issue_id, run_id, boundary_failures.clone())?;
    let evidence = write_task_evidence(
        root,
        issue_id,
        run_id,
        format_validation_summary(&validation),
    )?;
    if validation.passed {
        if find_session_snapshot_by_run(root, run_id)?.is_none() {
            anyhow::bail!(
                "build agent review preparation requires session snapshot before run {} can enter completed",
                run_id
            );
        }
        let _ = assert_run_transition(&TaskRunStatus::Validating, "completeRun")?;
        update_task_run_status(root, issue_id, run_id, TaskRunStatus::Completed)?;
        let _ = assert_issue_transition(&issue.status, "submitDelivery")?;
        let _ = update_spec_issue_status(root, &issue.issue_id, SpecIssueStatus::InReview)?;
    } else {
        let _ = assert_run_transition(&TaskRunStatus::Validating, "failRun")?;
        update_task_run_status(root, issue_id, run_id, TaskRunStatus::Failed)?;
        append_validation_failed_event(root, &issue, run_id, &evidence)?;
        let _ = agentflow_projection::rebuild_projections(root)?;
        agentflow_state::refresh_state(root)?;
        anyhow::bail!(
            "build agent review preparation validation failed for {issue_id}: failedCommands={}, boundaryFailures={:?}",
            validation.failed_command_ids.len(),
            validation.boundary_failures
        );
    }

    let changed_files_record = load_task_changed_files(root, issue_id, run_id)?;

    append_task_event_once(
        root,
        TaskEventDraft {
            flow_type: WorkflowFlowType::Work,
            aggregate_type: "issue".to_string(),
            aggregate_id: issue.issue_id.clone(),
            project_id: issue.project_id.clone(),
            issue_id: Some(issue.issue_id.clone()),
            run_id: Some(run_id.to_string()),
            event_type: "issue.validation.passed".to_string(),
            authority_role: Some(WorkflowAgentRole::WorkAgent),
            actor: EventActor {
                role: "build-agent".to_string(),
                kind: "system".to_string(),
            },
            state: Some(EventStateTransition {
                from_state: "in_progress".to_string(),
                to_state: "in_review".to_string(),
            }),
            correlation_id: Some(format!("corr-{}", issue.issue_id)),
            causation_id: None,
            payload: json!({
                "issueId": issue.issue_id,
                "projectId": issue.project_id,
                "runId": run_id,
                "changedFiles": changed_files_record.files.iter().map(|file| {
                    json!({
                        "path": file.path.as_str(),
                        "changeType": file.change_type.as_str(),
                        "insertions": file.insertions,
                        "deletions": file.deletions,
                        "sources": file.sources.iter().map(|source| match source {
                            TaskChangedFileSource::Committed => "committed",
                            TaskChangedFileSource::WorkingTree => "working_tree",
                            TaskChangedFileSource::Untracked => "untracked",
                        }).collect::<Vec<_>>(),
                    })
                }).collect::<Vec<_>>(),
                "validationCommandCount": validation.command_ids.len(),
                "boundaryFailures": validation.boundary_failures,
                "changedFilesPath": relative_path(root, &task_changed_files_path(root, issue_id, run_id)?),
                "validationCommandHash": validation.validation_command_hash,
                "validationOutputHash": validation.validation_output_hash,
                "patchSha256": validation.patch_sha256,
                "fileContentSha256": validation.file_content_sha256,
                "treeSha": validation.tree_sha,
                "validationResultHash": validation.validation_result_hash,
                "evidencePath": task_evidence_path(issue_id),
            }),
            artifact_refs: vec![
                task_evidence_path(issue_id),
                relative_path(root, &task_changed_files_path(root, issue_id, run_id)?),
            ],
            idempotency_key: Some(format!(
                "issue.validation.passed:{}:{}",
                issue.issue_id, run_id
            )),
        },
    )?;
    let _ = agentflow_projection::rebuild_projections(root)?;
    agentflow_state::refresh_state(root)?;
    let run = load_task_run(root, issue_id, run_id)?;
    Ok(BuildAgentReview {
        issue_id: issue_id.to_string(),
        run_id: run_id.to_string(),
        run_status: task_run_status_as_str(&run).to_string(),
        validation_passed: evidence.status == "passed",
        evidence_path: evidence_path(root, &evidence),
    })
}

fn reset_review_artifacts(root: &Path, issue_id: &str, run_id: &str) -> Result<()> {
    let commands_dir = task_run_dir(root, issue_id, run_id)?.join("commands");
    if commands_dir.exists() {
        fs::remove_dir_all(&commands_dir)
            .with_context(|| format!("remove {}", commands_dir.display()))?;
    }
    for path in [
        task_run_dir(root, issue_id, run_id)?.join("validation.json"),
        task_changed_files_path(root, issue_id, run_id)?,
        task_evidence_dir(root, issue_id)?.join("evidence.json"),
    ] {
        if path.exists() {
            fs::remove_file(&path).with_context(|| format!("remove {}", path.display()))?;
        }
    }
    Ok(())
}

fn run_validation_command(root: &Path, command: &str) -> Result<CommandResult> {
    let output = Command::new("sh")
        .arg("-lc")
        .arg(command)
        .current_dir(root)
        .output()
        .with_context(|| format!("run validation command `{command}`"))?;
    Ok(CommandResult {
        exit_code: output.status.code().unwrap_or(1),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    })
}

fn collect_trusted_changed_files(
    root: &Path,
    issue_id: &str,
    run_id: &str,
) -> Result<TrustedChangedFilesSnapshot> {
    let run = load_task_run(root, issue_id, run_id)?;
    let head_commit = Some(git_stdout(root, &["rev-parse", "HEAD"])?);
    let base_commit = run
        .base_commit
        .clone()
        .or_else(|| resolve_base_commit(root).ok())
        .or_else(|| head_commit.clone());
    let tree_sha = git_stdout(root, &["rev-parse", "HEAD^{tree}"]).ok();
    let mut files = BTreeMap::new();
    if let (Some(base_commit), Some(head_commit)) = (base_commit.as_deref(), head_commit.as_deref())
    {
        let committed_stats = collect_diff_stats_between(root, base_commit, head_commit)?;
        for file in collect_diff_name_status_between(root, base_commit, head_commit)?.into_values()
        {
            let path = file.path.clone();
            merge_changed_file(
                &mut files,
                file,
                TaskChangedFileSource::Committed,
                committed_stats.get(&path).copied(),
            );
        }
    }
    if let Some(head_commit) = head_commit.as_deref() {
        let worktree_stats = collect_diff_stats_against_head(root, head_commit)?;
        for file in collect_diff_name_status_against_head(root, head_commit)?.into_values() {
            let path = file.path.clone();
            merge_changed_file(
                &mut files,
                file,
                TaskChangedFileSource::WorkingTree,
                worktree_stats.get(&path).copied(),
            );
        }
    }
    for file in collect_untracked_files(root)? {
        merge_changed_file(&mut files, file, TaskChangedFileSource::Untracked, None);
    }
    let files = files
        .into_values()
        .filter(|file| !file.path.starts_with(".agentflow/"))
        .collect::<Vec<_>>();
    let working_tree_hash = git_stdout(
        root,
        &["status", "--porcelain=v1", "--untracked-files=all", "-z"],
    )
    .map(|raw| sha256_hex(raw.as_bytes()))?;
    let patch_sha256 = collect_patch_sha(root, base_commit.as_deref(), head_commit.as_deref())?;
    let file_content_sha256 = collect_file_content_sha(root, &files)?;
    Ok(TrustedChangedFilesSnapshot {
        files,
        base_commit,
        head_commit,
        tree_sha,
        working_tree_hash,
        patch_sha256,
        file_content_sha256,
    })
}

fn collect_diff_name_status_between(
    root: &Path,
    base_commit: &str,
    head_commit: &str,
) -> Result<BTreeMap<String, TaskChangedFile>> {
    let raw = git_stdout(
        root,
        &[
            "diff",
            "--name-status",
            "--find-renames",
            &format!("{base_commit}..{head_commit}"),
            "--",
        ],
    )?;
    parse_diff_name_status(&raw)
}

fn collect_diff_name_status_against_head(
    root: &Path,
    head_commit: &str,
) -> Result<BTreeMap<String, TaskChangedFile>> {
    let raw = git_stdout(
        root,
        &["diff", "--name-status", "--find-renames", head_commit, "--"],
    )?;
    parse_diff_name_status(&raw)
}

fn parse_diff_name_status(raw: &str) -> Result<BTreeMap<String, TaskChangedFile>> {
    let mut files = BTreeMap::new();
    for line in raw.lines() {
        let Some(file) = parse_name_status_line(line) else {
            continue;
        };
        files.insert(file.path.clone(), file);
    }
    Ok(files)
}

fn collect_diff_stats_between(
    root: &Path,
    base_commit: &str,
    head_commit: &str,
) -> Result<HashMap<String, (usize, usize)>> {
    let raw = git_stdout(
        root,
        &[
            "diff",
            "--numstat",
            "--find-renames",
            &format!("{base_commit}..{head_commit}"),
            "--",
        ],
    )?;
    parse_diff_stats(&raw)
}

fn collect_diff_stats_against_head(
    root: &Path,
    head_commit: &str,
) -> Result<HashMap<String, (usize, usize)>> {
    let raw = git_stdout(
        root,
        &["diff", "--numstat", "--find-renames", head_commit, "--"],
    )?;
    parse_diff_stats(&raw)
}

fn parse_diff_stats(raw: &str) -> Result<HashMap<String, (usize, usize)>> {
    let mut stats = HashMap::new();
    for line in raw.lines() {
        let mut parts = line.splitn(3, '\t');
        let insertions = parts.next().unwrap_or_default();
        let deletions = parts.next().unwrap_or_default();
        let path = parts.next().unwrap_or_default();
        if path.is_empty() {
            continue;
        }
        let normalized = normalize_status_path(path);
        stats.insert(
            normalized,
            (
                insertions.parse::<usize>().unwrap_or_default(),
                deletions.parse::<usize>().unwrap_or_default(),
            ),
        );
    }
    Ok(stats)
}

fn collect_untracked_files(root: &Path) -> Result<Vec<TaskChangedFile>> {
    let raw = git_stdout(root, &["ls-files", "--others", "--exclude-standard"])?;
    Ok(raw
        .lines()
        .map(str::trim)
        .filter(|path| !path.is_empty())
        .map(|path| TaskChangedFile {
            path: normalize_status_path(path),
            change_type: "added".to_string(),
            insertions: 0,
            deletions: 0,
            sources: Vec::new(),
        })
        .collect())
}

fn merge_changed_file(
    files: &mut BTreeMap<String, TaskChangedFile>,
    file: TaskChangedFile,
    source: TaskChangedFileSource,
    stats: Option<(usize, usize)>,
) {
    let TaskChangedFile {
        path,
        change_type,
        insertions,
        deletions,
        sources: _,
    } = file;
    let entry = files
        .entry(path.clone())
        .or_insert_with(|| TaskChangedFile {
            path,
            change_type: change_type.clone(),
            insertions: 0,
            deletions: 0,
            sources: Vec::new(),
        });
    entry.change_type = merge_change_type(&entry.change_type, &change_type);
    if let Some((insertions, deletions)) = stats {
        entry.insertions += insertions;
        entry.deletions += deletions;
    } else {
        entry.insertions += insertions;
        entry.deletions += deletions;
    }
    let mut sources = entry.sources.iter().copied().collect::<BTreeSet<_>>();
    sources.insert(source);
    entry.sources = sources.into_iter().collect();
}

fn merge_change_type(current: &str, candidate: &str) -> String {
    let choose_rank = |change_type: &str| match change_type {
        "renamed" => 4,
        "deleted" => 3,
        "added" => 2,
        "modified" => 1,
        _ => 0,
    };
    if choose_rank(candidate) >= choose_rank(current) {
        candidate.to_string()
    } else {
        current.to_string()
    }
}

fn collect_patch_sha(
    root: &Path,
    base_commit: Option<&str>,
    head_commit: Option<&str>,
) -> Result<String> {
    let mut payload = Vec::new();
    if let (Some(base_commit), Some(head_commit)) = (base_commit, head_commit) {
        payload.extend_from_slice(
            git_stdout(
                root,
                &[
                    "diff",
                    "--binary",
                    &format!("{base_commit}..{head_commit}"),
                    "--",
                ],
            )?
            .as_bytes(),
        );
    }
    if let Some(head_commit) = head_commit {
        payload.extend_from_slice(b"\n--worktree--\n");
        payload.extend_from_slice(
            git_stdout(root, &["diff", "--binary", head_commit, "--"])?.as_bytes(),
        );
    }
    payload.extend_from_slice(b"\n--untracked--\n");
    for file in collect_untracked_files(root)? {
        payload.extend_from_slice(file.path.as_bytes());
        payload.extend_from_slice(b"\0");
        let path = root.join(&file.path);
        if path.is_file() {
            payload.extend_from_slice(&fs::read(path)?);
        }
        payload.extend_from_slice(b"\n");
    }
    Ok(sha256_hex(&payload))
}

fn collect_file_content_sha(root: &Path, files: &[TaskChangedFile]) -> Result<String> {
    let mut payload = Vec::new();
    let mut sorted = files.to_vec();
    sorted.sort_by(|left, right| left.path.cmp(&right.path));
    for file in sorted {
        payload.extend_from_slice(file.path.as_bytes());
        payload.extend_from_slice(b"\0");
        let path = root.join(&file.path);
        if path.is_file() {
            payload.extend_from_slice(&fs::read(path)?);
        } else {
            payload.extend_from_slice(b"__deleted__");
        }
        payload.extend_from_slice(b"\n");
    }
    Ok(sha256_hex(&payload))
}

fn parse_name_status_line(line: &str) -> Option<TaskChangedFile> {
    let mut parts = line.split('\t');
    let status = parts.next()?.trim();
    if status.is_empty() {
        return None;
    }
    let raw_path = if status.starts_with('R') || status.starts_with('C') {
        parts.nth(1)?
    } else {
        parts.next()?
    };
    if raw_path.is_empty() {
        return None;
    }
    Some(TaskChangedFile {
        path: normalize_status_path(raw_path),
        change_type: status_to_change_type(status),
        insertions: 0,
        deletions: 0,
        sources: Vec::new(),
    })
}

fn normalize_status_path(path: &str) -> String {
    path.rsplit(" -> ")
        .next()
        .unwrap_or(path)
        .rsplit(" => ")
        .next()
        .unwrap_or(path)
        .trim()
        .replace('\\', "/")
}

fn status_to_change_type(status: &str) -> String {
    let normalized = status.trim();
    if normalized.contains('R') {
        "renamed".to_string()
    } else if normalized.contains('D') {
        "deleted".to_string()
    } else if normalized.contains('A') || normalized == "??" {
        "added".to_string()
    } else {
        "modified".to_string()
    }
}

fn validate_changed_file_boundaries(
    issue: &agentflow_spec::SpecIssue,
    files: &[TaskChangedFile],
) -> Result<Vec<String>> {
    let allowed = build_globset(&issue.allowed_paths)?;
    let forbidden = build_globset(&issue.forbidden_paths)?;
    let mut failures = Vec::new();
    for file in files {
        if forbidden
            .as_ref()
            .is_some_and(|set| set.is_match(&file.path))
        {
            failures.push(format!("禁止路径变更：{}", file.path));
        }
        if let Some(allowed) = &allowed {
            if !allowed.is_match(&file.path) {
                failures.push(format!("超出允许路径：{}", file.path));
            }
        }
    }
    Ok(failures)
}

fn build_globset(patterns: &[String]) -> Result<Option<GlobSet>> {
    let active = patterns
        .iter()
        .map(|pattern| pattern.trim())
        .filter(|pattern| !pattern.is_empty())
        .collect::<Vec<_>>();
    if active.is_empty() {
        return Ok(None);
    }
    let mut builder = GlobSetBuilder::new();
    for pattern in active {
        builder
            .add(Glob::new(pattern).with_context(|| format!("invalid glob pattern `{pattern}`"))?);
    }
    Ok(Some(builder.build()?))
}

fn resolve_base_commit(root: &Path) -> Result<String> {
    for candidate in [
        vec!["merge-base", "HEAD", "@{upstream}"],
        vec!["merge-base", "HEAD", "origin/main"],
        vec!["merge-base", "HEAD", "main"],
    ] {
        if let Ok(commit) = git_stdout(root, &candidate) {
            return Ok(commit);
        }
    }
    anyhow::bail!("unable to resolve base commit")
}

fn git_stdout(root: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .with_context(|| format!("run git {}", args.join(" ")))?;
    if !output.status.success() {
        anyhow::bail!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .trim_end()
        .to_string())
}

fn format_validation_summary(
    validation: &agentflow_task_artifacts::TaskValidationRecord,
) -> String {
    if validation.passed {
        format!("验证命令 {} 条，全部通过。", validation.command_ids.len())
    } else if !validation.boundary_failures.is_empty() {
        format!(
            "验证未通过，{} 条命令已执行，边界校验失败 {} 项。",
            validation.command_ids.len(),
            validation.boundary_failures.len()
        )
    } else {
        format!(
            "验证未通过，{} 条命令中有 {} 条失败。",
            validation.command_ids.len(),
            validation.failed_command_ids.len()
        )
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[derive(Debug, Clone)]
struct CommandResult {
    exit_code: i32,
    stdout: String,
    stderr: String,
}

fn append_validation_failed_event(
    root: &Path,
    issue: &agentflow_spec::SpecIssue,
    run_id: &str,
    evidence: &TaskEvidence,
) -> Result<()> {
    append_task_event_once(
        root,
        TaskEventDraft {
            flow_type: WorkflowFlowType::Work,
            aggregate_type: "issue".to_string(),
            aggregate_id: issue.issue_id.clone(),
            project_id: issue.project_id.clone(),
            issue_id: Some(issue.issue_id.clone()),
            run_id: Some(run_id.to_string()),
            event_type: "issue.validation.failed".to_string(),
            authority_role: Some(WorkflowAgentRole::WorkAgent),
            actor: EventActor {
                role: "build-agent".to_string(),
                kind: "system".to_string(),
            },
            state: Some(EventStateTransition {
                from_state: "in_progress".to_string(),
                to_state: "blocked".to_string(),
            }),
            correlation_id: Some(format!("corr-{}", issue.issue_id)),
            causation_id: None,
            payload: json!({
                "issueId": issue.issue_id,
                "projectId": issue.project_id,
                "runId": run_id,
                "evidencePath": task_evidence_path(&issue.issue_id),
                "summary": evidence.summary,
            }),
            artifact_refs: vec![task_evidence_path(&issue.issue_id)],
            idempotency_key: Some(format!(
                "issue.validation.failed:{}:{}",
                issue.issue_id, run_id
            )),
        },
    )?;
    Ok(())
}

fn load_closeout_proof(root: &Path, issue_id: &str, run_id: &str) -> Result<serde_json::Value> {
    let path = closeout_proof_path(root, issue_id, run_id);
    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

fn build_evidence_gate_decision(
    root: &Path,
    issue: &agentflow_spec::SpecIssue,
    run_id: &str,
    evidence: &TaskEvidence,
    proof: &serde_json::Value,
) -> Result<TaskEvidenceGateDecision> {
    let validation = load_task_validation(root, &issue.issue_id, run_id)?;
    let expected_validation_path = resolve_output_path(
        issue.expected_outputs.validation_result_path.as_deref(),
        &issue.issue_id,
        run_id,
        "validation.json",
    );
    let expected_changed_files_path = resolve_output_path(
        issue.expected_outputs.changed_files_path.as_deref(),
        &issue.issue_id,
        run_id,
        "changed-files.json",
    );
    let expected_closeout_proof_path = resolve_output_path(
        issue.expected_outputs.closeout_proof_path.as_deref(),
        &issue.issue_id,
        run_id,
        "review/closeout-proof.json",
    );
    let proof_path = closeout_proof_path(root, &issue.issue_id, run_id);
    let proof_ref = expected_closeout_proof_path.clone();
    let merged = proof
        .get("merged")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    let issue_closed = proof
        .get("issueClosed")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    let pr_url = proof
        .get("prUrl")
        .and_then(serde_json::Value::as_str)
        .map(str::to_string);
    let manual_reason = proof
        .get("manualVerificationReason")
        .and_then(serde_json::Value::as_str)
        .map(str::to_string);
    let manual_risk = proof
        .get("manualVerificationRisk")
        .and_then(serde_json::Value::as_str)
        .map(str::to_string);

    let mut entries = evidence.entries.clone();
    entries.push(TaskEvidenceEntry {
        evidence_type: "prLink".to_string(),
        required: issue.expected_outputs.public_delivery_record.pr_or_mr_body,
        status: if pr_url.is_some() {
            TaskEvidenceEntryStatus::Ready
        } else {
            TaskEvidenceEntryStatus::Missing
        },
        summary: if pr_url.is_some() {
            "PR/MR 链接已记录。".to_string()
        } else {
            "缺少 PR/MR 链接。".to_string()
        },
        refs: pr_url.clone().into_iter().collect(),
        manual_reason: None,
        manual_risk: None,
    });
    entries.push(TaskEvidenceEntry {
        evidence_type: "mergeProof".to_string(),
        required: true,
        status: if proof_path.is_file() && merged && issue_closed {
            TaskEvidenceEntryStatus::Ready
        } else if proof_path.is_file() {
            TaskEvidenceEntryStatus::Failed
        } else {
            TaskEvidenceEntryStatus::Missing
        },
        summary: if proof_path.is_file() && merged && issue_closed {
            "合并证明已记录，PR/MR 已合并且 issue 已关闭。".to_string()
        } else if proof_path.is_file() {
            "合并证明已写入，但缺少 merged 或 issueClosed 事实。".to_string()
        } else {
            "缺少合并证明。".to_string()
        },
        refs: vec![proof_ref.clone()],
        manual_reason: None,
        manual_risk: None,
    });
    entries.push(TaskEvidenceEntry {
        evidence_type: "artifactSummary".to_string(),
        required: true,
        status: if proof_path.is_file()
            && (!issue.expected_outputs.public_delivery_record.pr_or_mr_body || pr_url.is_some())
        {
            TaskEvidenceEntryStatus::Ready
        } else if proof_path.is_file() {
            TaskEvidenceEntryStatus::Failed
        } else {
            TaskEvidenceEntryStatus::Missing
        },
        summary: if proof_path.is_file()
            && (!issue.expected_outputs.public_delivery_record.pr_or_mr_body || pr_url.is_some())
        {
            "交付摘要已通过 closeout proof 记录。".to_string()
        } else if proof_path.is_file() {
            "closeout proof 已存在，但交付摘要不完整。".to_string()
        } else {
            "缺少交付摘要。".to_string()
        },
        refs: vec![proof_ref.clone()],
        manual_reason: None,
        manual_risk: None,
    });
    if manual_reason.is_some() || manual_risk.is_some() {
        entries.push(TaskEvidenceEntry {
            evidence_type: "manualVerification".to_string(),
            required: false,
            status: if manual_reason.is_some() && manual_risk.is_some() {
                TaskEvidenceEntryStatus::Manual
            } else {
                TaskEvidenceEntryStatus::Failed
            },
            summary: if manual_reason.is_some() && manual_risk.is_some() {
                "人工验证原因与风险已记录。".to_string()
            } else {
                "人工验证只记录了部分信息，缺少原因或风险。".to_string()
            },
            refs: vec![proof_ref.clone()],
            manual_reason,
            manual_risk,
        });
    }

    let required_evidence_types = entries
        .iter()
        .filter(|entry| entry.required)
        .map(|entry| entry.evidence_type.clone())
        .collect::<Vec<_>>();
    let mut blockers = Vec::new();
    if evidence.validation_path != expected_validation_path {
        blockers.push(format!(
            "validation result path mismatch: expected {}, got {}",
            expected_validation_path, evidence.validation_path
        ));
    }
    if !root.join(&expected_validation_path).is_file() {
        blockers.push(format!(
            "missing validation result artifact: {}",
            expected_validation_path
        ));
    }
    if evidence.changed_files_path.as_deref() != Some(expected_changed_files_path.as_str()) {
        blockers.push(format!(
            "implementation summary path mismatch: expected {}, got {}",
            expected_changed_files_path,
            evidence
                .changed_files_path
                .clone()
                .unwrap_or_else(|| "<missing>".to_string())
        ));
    }
    if !root.join(&expected_changed_files_path).is_file() {
        blockers.push(format!(
            "missing implementation summary artifact: {}",
            expected_changed_files_path
        ));
    }
    if issue.expected_outputs.evidence_path != task_evidence_path(&issue.issue_id) {
        blockers.push(format!(
            "evidence path mismatch: expected {}, got {}",
            issue.expected_outputs.evidence_path,
            task_evidence_path(&issue.issue_id)
        ));
    }
    if !root.join(task_evidence_path(&issue.issue_id)).is_file() {
        blockers.push(format!(
            "missing evidence artifact: {}",
            task_evidence_path(&issue.issue_id)
        ));
    }
    if !validation.passed {
        blockers.push("failed validation cannot count as passing evidence".to_string());
    }
    if let Some(entry) = entries
        .iter()
        .find(|entry| entry.evidence_type == "manualVerification")
    {
        if entry.status == TaskEvidenceEntryStatus::Failed {
            blockers.push("manual verification requires both reason and risk".to_string());
        }
    }
    for entry in entries.iter().filter(|entry| entry.required) {
        if entry.status != TaskEvidenceEntryStatus::Ready {
            blockers.push(format!(
                "required evidence `{}` is {}",
                entry.evidence_type,
                evidence_entry_status_as_str(&entry.status)
            ));
        }
    }

    Ok(TaskEvidenceGateDecision {
        version: String::new(),
        issue_id: issue.issue_id.clone(),
        run_id: run_id.to_string(),
        passed: blockers.is_empty(),
        validation_passed: validation.passed,
        required_evidence_types,
        blockers,
        entries,
        checked_at: current_unix_timestamp(),
    })
}

fn resolve_output_path(
    configured: Option<&str>,
    issue_id: &str,
    run_id: &str,
    fallback_suffix: &str,
) -> String {
    configured
        .map(|value| value.replace("<run-id>", run_id))
        .unwrap_or_else(|| format!(".agentflow/tasks/{issue_id}/runs/{run_id}/{fallback_suffix}"))
}

fn current_unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn evidence_entry_status_as_str(status: &TaskEvidenceEntryStatus) -> &'static str {
    match status {
        TaskEvidenceEntryStatus::Ready => "ready",
        TaskEvidenceEntryStatus::Missing => "missing",
        TaskEvidenceEntryStatus::Failed => "failed",
        TaskEvidenceEntryStatus::Manual => "manual",
    }
}

fn closeout_proof_path(root: &Path, issue_id: &str, run_id: &str) -> PathBuf {
    task_run_dir(root, issue_id, run_id)
        .expect("validated task run dir")
        .join("review/closeout-proof.json")
}

fn latest_closed_at(issues: &[McpCloseoutIssueAttestation]) -> Option<String> {
    issues
        .iter()
        .filter_map(|issue| issue.closed_at.clone())
        .max()
}

#[derive(Debug, Clone, Copy)]
enum CloseoutUrlKind {
    Review,
    Issue,
}

fn ensure_attested_url_matches_repository(
    provider: &str,
    repository_full_name: &str,
    url: &str,
    kind: CloseoutUrlKind,
) -> Result<()> {
    let (expected_segment, label) = match (provider, kind) {
        ("github", CloseoutUrlKind::Review) => (
            format!("https://github.com/{repository_full_name}/pull/"),
            "pr_url",
        ),
        ("github", CloseoutUrlKind::Issue) => (
            format!("https://github.com/{repository_full_name}/issues/"),
            "provider_issue_url",
        ),
        ("gitlab", CloseoutUrlKind::Review) => (
            format!("/{repository_full_name}/-/merge_requests/"),
            "pr_url",
        ),
        ("gitlab", CloseoutUrlKind::Issue) => (
            format!("/{repository_full_name}/-/issues/"),
            "provider_issue_url",
        ),
        _ => return Ok(()),
    };
    if !url.contains(&expected_segment) {
        anyhow::bail!(
            "closeout proof {} repository mismatch: expected {}, got {}",
            label,
            repository_full_name,
            url
        );
    }
    Ok(())
}

fn task_evidence_path(issue_id: &str) -> String {
    format!(".agentflow/tasks/{issue_id}/evidence/evidence.json")
}

fn evidence_path(root: &Path, evidence: &TaskEvidence) -> PathBuf {
    task_evidence_dir(root, &evidence.issue_id)
        .expect("validated task evidence dir")
        .join("evidence.json")
}

fn relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string()
}

fn write_json(path: &Path, value: &serde_json::Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(path, serde_json::to_string_pretty(value)? + "\n")
        .with_context(|| format!("write {}", path.display()))
}

fn task_run_status_as_str(run: &TaskRun) -> &'static str {
    match run.status {
        TaskRunStatus::Queued => "queued",
        TaskRunStatus::InProgress => "in_progress",
        TaskRunStatus::Validating => "validating",
        TaskRunStatus::Completed => "completed",
        TaskRunStatus::Failed => "failed",
        TaskRunStatus::Cancelled => "cancelled",
    }
}

fn assert_current_cli_is_fresh(root: &Path) -> Result<()> {
    let current_exe = std::env::current_exe().context("locate current agentflow CLI binary")?;
    if !is_local_target_binary(root, &current_exe) {
        return Ok(());
    }
    let binary_modified = file_modified(&current_exe)?;
    if let Some((newest_path, newest_modified)) = newest_source_mtime(root)? {
        if binary_is_stale(binary_modified, newest_modified) {
            anyhow::bail!(
                "current AgentFlow CLI is stale: {} is older than {}. Run `{}` before `build-agent complete`.",
                current_exe.display(),
                newest_path.display(),
                rebuild_hint(&current_exe)
            );
        }
    }
    Ok(())
}

fn newest_source_mtime(root: &Path) -> Result<Option<(PathBuf, SystemTime)>> {
    let mut newest = None;
    for relative in CLI_FRESHNESS_PATHS {
        collect_newest_mtime(&root.join(relative), &mut newest)?;
    }
    Ok(newest)
}

fn collect_newest_mtime(path: &Path, newest: &mut Option<(PathBuf, SystemTime)>) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }
    let metadata = fs::metadata(path)?;
    if metadata.is_file() {
        let modified = metadata.modified()?;
        let replace = newest
            .as_ref()
            .map(|(_, current)| modified > *current)
            .unwrap_or(true);
        if replace {
            *newest = Some((path.to_path_buf(), modified));
        }
        return Ok(());
    }
    if metadata.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            collect_newest_mtime(&entry.path(), newest)?;
        }
    }
    Ok(())
}

fn file_modified(path: &Path) -> Result<SystemTime> {
    Ok(fs::metadata(path)?.modified()?)
}

fn is_local_target_binary(root: &Path, executable: &Path) -> bool {
    executable.starts_with(root.join("target"))
        && executable
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| name == "agentflow")
            .unwrap_or(false)
}

fn rebuild_hint(executable: &Path) -> &'static str {
    if executable
        .components()
        .any(|component| component.as_os_str() == "release")
    {
        "cargo build --release --bin agentflow"
    } else {
        "cargo build --bin agentflow"
    }
}

fn binary_is_stale(binary_modified: SystemTime, newest_source_modified: SystemTime) -> bool {
    newest_source_modified > binary_modified
}

#[cfg(test)]
mod tests {
    use super::{
        binary_is_stale, claim_next_build_agent_launch_with_dispatcher,
        complete_build_agent_issue_from_request, is_local_target_binary, load_closeout_proof,
        prepare_build_agent_review_from_request, rebuild_hint, start_build_agent_issue,
        write_build_agent_closeout_proof_from_attestation,
    };
    use agentflow_mcp::{
        write_session_snapshot, McpAgentProvider, McpCloseoutAttestation,
        McpCloseoutIssueAttestation, McpLaunchMode, McpLaunchPlan, McpLaunchRequest,
        McpProviderBridge, McpProviderKind, McpProviderStatus, McpProviderStatusCode,
        McpSessionGovernanceFacts, McpSessionGovernancePolicy, McpSessionSnapshot,
        McpSessionStatus, MCP_CLOSEOUT_ATTESTATION_VERSION,
    };
    use agentflow_task_artifacts::{
        load_task_changed_files, load_task_evidence, load_task_evidence_gate_decision,
        load_task_run, TaskChangedFileSource, TaskRunStatus,
    };
    use anyhow::Result;
    use std::{
        fs,
        path::{Path, PathBuf},
        process::Command,
        time::{Duration, UNIX_EPOCH},
    };
    use tempfile::tempdir;

    struct FakeProvider;

    impl McpAgentProvider for FakeProvider {
        fn provider_id(&self) -> &'static str {
            "codex"
        }

        fn kind(&self) -> McpProviderKind {
            McpProviderKind::Codex
        }

        fn check_health(&self, _project_root: &Path) -> McpProviderStatus {
            let mut status = McpProviderStatus::new(McpProviderKind::Codex, 1);
            status.provider = "codex".to_string();
            status.status = McpProviderStatusCode::Ready;
            status.capabilities = vec![
                agentflow_mcp::McpCapability::new("launch", true),
                agentflow_mcp::McpCapability::new("codex.exec", true),
                agentflow_mcp::McpCapability::new("session.poll", true),
                agentflow_mcp::McpCapability::new("session.logs", true),
                agentflow_mcp::McpCapability::new("session.cancel", true),
                agentflow_mcp::McpCapability::new("build_agent.complete", true),
            ];
            status
        }

        fn build_launch_plan(
            &self,
            _project_root: &Path,
            request: &McpLaunchRequest,
        ) -> Result<McpLaunchPlan> {
            let mut plan = McpLaunchPlan::new(
                "codex",
                format!("fake-{}", request.run_id),
                request.issue_id.clone(),
                request.run_id.clone(),
                McpLaunchMode::CliExecPromptFile,
                request.working_directory.clone(),
                "fake-agent",
            );
            plan.stdin_path = Some(request.launch_request_path.clone());
            Ok(plan)
        }
    }

    #[test]
    fn detects_local_target_binaries_only() {
        assert!(is_local_target_binary(
            Path::new("/repo"),
            Path::new("/repo/target/debug/agentflow")
        ));
        assert!(is_local_target_binary(
            Path::new("/repo"),
            Path::new("/repo/target/release/agentflow")
        ));
        assert!(!is_local_target_binary(
            Path::new("/repo"),
            Path::new("/usr/local/bin/agentflow")
        ));
    }

    #[test]
    fn stale_check_requires_newer_sources() {
        let binary = UNIX_EPOCH + Duration::from_secs(10);
        let source = UNIX_EPOCH + Duration::from_secs(11);
        assert!(binary_is_stale(binary, source));
        assert!(!binary_is_stale(source, binary));
    }

    #[test]
    fn rebuild_hint_matches_binary_profile() {
        assert_eq!(
            rebuild_hint(Path::new("/repo/target/debug/agentflow")),
            "cargo build --bin agentflow"
        );
        assert_eq!(
            rebuild_hint(Path::new("/repo/target/release/agentflow")),
            "cargo build --release --bin agentflow"
        );
    }

    fn write_running_session_snapshot(
        root: &Path,
        issue_id: &str,
        run_id: &str,
        branch_name: &str,
    ) {
        let session = McpSessionSnapshot {
            version: "agentflow-mcp-session.v1".to_string(),
            provider: "codex".to_string(),
            issue_id: issue_id.to_string(),
            project_id: Some("proj-001".to_string()),
            run_id: run_id.to_string(),
            session_id: format!("codex-{run_id}"),
            status: McpSessionStatus::Running,
            launch_mode: McpLaunchMode::CliExecPromptFile,
            working_directory: root.display().to_string(),
            workspace_root: Some(root.display().to_string()),
            worktree_root: Some(root.display().to_string()),
            runtime_root: Some(
                root.join(format!(".agentflow/tasks/{issue_id}/runs/{run_id}/runtime"))
                    .display()
                    .to_string(),
            ),
            temp_root: None,
            cache_root: None,
            evidence_root: Some(
                root.join(format!(".agentflow/tasks/{issue_id}/evidence"))
                    .display()
                    .to_string(),
            ),
            launch_request_path: format!(
                ".agentflow/tasks/{issue_id}/runs/{run_id}/launch/agent-request.json"
            ),
            plan_path: format!(".agentflow/state/mcp/plans/codex-{run_id}.json"),
            log_path: None,
            branch_name: Some(branch_name.to_string()),
            attempt_count: 1,
            pid: None,
            process_group_id: None,
            remote_session_id: Some(format!("remote-{run_id}")),
            pr_url: None,
            last_message_path: None,
            exit_proof_path: None,
            merge_proof_path: None,
            merge_state: None,
            writeback_state: None,
            recovery_reason: None,
            note: Some("test-running-session".to_string()),
            last_error: None,
            permission_mode: Some("never".to_string()),
            approval_policy: Some("never".to_string()),
            sandbox_mode: Some("workspace-write".to_string()),
            supervision_mode: Some("local-process-watch".to_string()),
            exited_at: None,
            exit_code: None,
            governance_policy: McpSessionGovernancePolicy::default(),
            governance_facts: McpSessionGovernanceFacts::default(),
            created_at: 1,
            updated_at: 1,
        };
        write_session_snapshot(root, &session).unwrap();
    }

    #[test]
    fn claim_next_build_agent_launch_consumes_pending_event() {
        let dir = tempdir().unwrap();
        let requirement = dir.path().join("docs/requirements/034-claim-test.md");
        fs::create_dir_all(requirement.parent().unwrap()).unwrap();
        fs::write(
            &requirement,
            "# Claim Test\n\n验证 CLI claim 走 AgentDispatcher。\n",
        )
        .unwrap();
        let mut issue = agentflow_spec::SpecIssueDraft::new("AF-001");
        issue.project_id = Some("proj-001".to_string());
        issue.allowed_paths = vec!["src/**".to_string()];
        issue.validation_commands = vec!["test -f src/lib.rs".to_string()];
        let issue =
            agentflow_spec::issue_from_requirement(dir.path(), &requirement, issue).unwrap();
        agentflow_spec::write_spec_issue(dir.path(), &issue).unwrap();
        let mut project = agentflow_spec::SpecProjectDraft::new("proj-001");
        project.issue_ids = vec!["AF-001".to_string()];
        let project =
            agentflow_spec::project_from_requirement(dir.path(), &requirement, project).unwrap();
        agentflow_spec::write_spec_project(dir.path(), &project).unwrap();
        write_file(
            dir.path().join("src/lib.rs"),
            "pub fn claim_fixture() -> &'static str { \"ok\" }\n",
        );
        init_git_repo(dir.path());
        let loop_driver = agentflow_task_loop::TaskLoop::new("proj-001");
        loop_driver
            .schedule_next_issue(dir.path())
            .unwrap()
            .unwrap();
        let launch = loop_driver
            .request_agent_launch(dir.path(), "AF-001", "codex")
            .unwrap();
        let mut providers = McpProviderBridge::new();
        providers.register(Box::new(FakeProvider));
        let dispatcher = agentflow_agent_dispatcher::AgentDispatcher::new(providers);

        let claim = claim_next_build_agent_launch_with_dispatcher(dir.path(), &dispatcher)
            .unwrap()
            .expect("expected launch claim");
        assert_eq!(claim.issue_id, "AF-001");
        assert_eq!(claim.run_id, "run-001");
        assert_eq!(
            claim.branch_name.as_deref(),
            Some("agentflow/proj-001/AF-001")
        );
        assert_eq!(
            claim.launch_request_path,
            dir.path().join(&launch.launch_request_path)
        );
        let events = agentflow_event_store::load_task_events(dir.path()).unwrap();
        assert!(events
            .iter()
            .any(|event| event.event_type == agentflow_agent_dispatcher::AGENT_SESSION_CREATED));
        assert!(
            claim_next_build_agent_launch_with_dispatcher(dir.path(), &dispatcher)
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn build_agent_start_uses_spec_task_loop_issue() {
        let dir = tempdir().unwrap();
        let requirement = dir.path().join("docs/requirements/034-start-test.md");
        fs::create_dir_all(requirement.parent().unwrap()).unwrap();
        fs::write(
            &requirement,
            "# Start Test\n\n验证 CLI start 只依赖 spec/task-loop。\n",
        )
        .unwrap();
        let mut issue = agentflow_spec::SpecIssueDraft::new("AF-START-001");
        issue.project_id = Some("proj-start".to_string());
        issue.allowed_paths = vec!["src/**".to_string()];
        issue.validation_commands = vec!["test -f src/lib.rs".to_string()];
        let issue =
            agentflow_spec::issue_from_requirement(dir.path(), &requirement, issue).unwrap();
        agentflow_spec::write_spec_issue(dir.path(), &issue).unwrap();
        let mut project = agentflow_spec::SpecProjectDraft::new("proj-start");
        project.issue_ids = vec!["AF-START-001".to_string()];
        let project =
            agentflow_spec::project_from_requirement(dir.path(), &requirement, project).unwrap();
        agentflow_spec::write_spec_project(dir.path(), &project).unwrap();
        write_file(
            dir.path().join("src/lib.rs"),
            "pub fn start_fixture() -> &'static str { \"ok\" }\n",
        );
        init_git_repo(dir.path());

        let started = start_build_agent_issue(dir.path(), "AF-START-001").unwrap();
        let run = load_task_run(dir.path(), "AF-START-001", &started.run_id).unwrap();

        assert_eq!(started.issue_id, "AF-START-001");
        assert_eq!(started.run_id, "run-001");
        assert_eq!(started.stage, "in_progress");
        assert_eq!(started.project_id.as_deref(), Some("proj-start"));
        assert_eq!(
            started.branch_name.as_deref(),
            Some("agentflow/proj-start/AF-START-001")
        );
        assert!(dir
            .path()
            .join(".agentflow/tasks/AF-START-001/runs/run-001/launch/agent-request.json")
            .is_file());
        assert!(dir
            .path()
            .join(".agentflow/spec/issues/AF-START-001.json")
            .is_file());
        assert_eq!(run.status, TaskRunStatus::InProgress);
    }

    #[test]
    fn build_agent_review_merge_and_complete_use_task_events() {
        let dir = tempdir().unwrap();
        write_spec_project_fixture(dir.path(), "proj-001", "AF-001");
        write_file(
            dir.path().join("src/lib.rs"),
            "pub fn status() -> &'static str { \"before\" }\n",
        );
        init_git_repo(dir.path());
        let started = start_build_agent_issue(dir.path(), "AF-001").unwrap();
        write_running_session_snapshot(
            dir.path(),
            "AF-001",
            &started.run_id,
            started.branch_name.as_deref().unwrap(),
        );
        write_file(
            dir.path().join("src/lib.rs"),
            "pub fn status() -> &'static str { \"after\" }\n",
        );

        let request_path = write_completion_request(dir.path(), "AF-001", &started.run_id);
        let prepared = prepare_build_agent_review_from_request(dir.path(), &request_path).unwrap();
        assert_eq!(prepared.issue_id, "AF-001");
        assert_eq!(prepared.run_status, "completed");
        assert!(prepared.validation_passed);
        assert!(prepared.evidence_path.is_file());
        let changed_files = load_task_changed_files(dir.path(), "AF-001", &started.run_id).unwrap();
        assert_eq!(changed_files.files.len(), 1);
        assert_eq!(changed_files.files[0].path, "src/lib.rs");
        let evidence = load_task_evidence(dir.path(), "AF-001").unwrap();
        assert!(evidence.validation_command_hash.is_some());
        assert!(evidence.validation_output_hash.is_some());
        assert!(evidence.patch_sha256.is_some());
        assert!(evidence.file_content_sha256.is_some());
        assert!(evidence.tree_sha.is_some());
        assert!(evidence.command_hash.is_some());
        assert!(evidence.changed_file_hash.is_some());
        assert!(evidence.validation_result_hash.is_some());
        let in_review_projection =
            agentflow_projection::load_task_projection(dir.path(), "AF-001").unwrap();
        assert_eq!(in_review_projection.current_state, "in_review");
        assert!(!dir.path().join(".agentflow/output").exists());
        assert!(!dir.path().join(".agentflow/execute").exists());
        assert!(!dir.path().join(".agentflow/input").exists());

        let issue = agentflow_spec::read_spec_issue(dir.path(), "AF-001").unwrap();
        write_build_agent_closeout_proof_from_attestation(
            dir.path(),
            &issue,
            &started.run_id,
            "auto-merge-if-eligible",
            trusted_closeout(
                dir.path(),
                "github",
                "https://github.com/atxinbao/agentflow/pull/1",
                started.branch_name.as_deref().unwrap(),
                true,
                "1",
                true,
                Some("2026-06-19T11:20:01Z"),
            ),
        )
        .unwrap();
        let still_review_projection =
            agentflow_projection::load_task_projection(dir.path(), "AF-001").unwrap();
        assert_eq!(still_review_projection.current_state, "in_review");
        let in_review_issue = agentflow_spec::read_spec_issue(dir.path(), "AF-001").unwrap();
        assert_eq!(
            in_review_issue.status,
            agentflow_spec::SpecIssueStatus::InReview
        );

        let outcome = complete_build_agent_issue_from_request(dir.path(), &request_path).unwrap();
        assert_eq!(outcome.issue_id, "AF-001");
        assert_eq!(outcome.run_status, "completed");
        assert!(outcome.next_launch.is_none());
        let gate = load_task_evidence_gate_decision(dir.path(), "AF-001").unwrap();
        assert!(gate.passed);
        assert!(gate.validation_passed);
        assert!(gate
            .required_evidence_types
            .iter()
            .any(|item| item == "verificationLog"));
        assert!(gate
            .required_evidence_types
            .iter()
            .any(|item| item == "artifactSummary"));
        assert!(gate
            .required_evidence_types
            .iter()
            .any(|item| item == "implementationSummary"));
        assert_eq!(
            std::fs::canonicalize(&outcome.closeout_proof_path).unwrap(),
            std::fs::canonicalize(
                dir.path()
                    .join(".agentflow/tasks/AF-001/runs/run-001/review/closeout-proof.json")
            )
            .unwrap()
        );
        assert!(outcome.closeout_proof_path.is_file());
        let done_projection =
            agentflow_projection::load_task_projection(dir.path(), "AF-001").unwrap();
        assert_eq!(done_projection.current_state, "done");
        let done_issue = agentflow_spec::read_spec_issue(dir.path(), "AF-001").unwrap();
        assert_eq!(done_issue.status, agentflow_spec::SpecIssueStatus::Done);
        assert_eq!(
            done_projection.public_delivery.pr_url.as_deref(),
            Some("https://github.com/atxinbao/agentflow/pull/1")
        );
        assert_eq!(
            done_projection.public_delivery.changelog_path.as_deref(),
            None
        );
        assert_eq!(
            done_projection.public_delivery.release_notes_url.as_deref(),
            None
        );
        assert_eq!(done_projection.delivery.status, "ready");
        let proof = load_closeout_proof(dir.path(), "AF-001", &started.run_id).unwrap();
        assert!(!proof["publicDeliveryWritten"].as_bool().unwrap_or(false));
        assert_eq!(proof.get("changelogPath"), None);
        assert_eq!(proof.get("releaseNotesPath"), None);
        let events = agentflow_event_store::replay_task_events(
            dir.path(),
            agentflow_event_store::ReplayFilter::issue("AF-001"),
        )
        .unwrap();
        assert!(events
            .iter()
            .any(|event| event.event_type == "issue.validation.passed"));
        assert!(events
            .iter()
            .any(|event| event.event_type == "issue.closeout.proof.recorded"));
        assert!(events
            .iter()
            .any(|event| event.event_type == "issue.completed"));
        assert!(!dir.path().join(".agentflow/output").exists());
        assert!(!dir.path().join(".agentflow/execute").exists());
        assert!(!dir.path().join(".agentflow/input").exists());
    }

    #[test]
    fn build_agent_review_requires_session_snapshot_before_completion() {
        let dir = tempdir().unwrap();
        write_spec_project_fixture(dir.path(), "proj-001", "AF-001");
        write_file(
            dir.path().join("src/lib.rs"),
            "pub fn status() -> &'static str { \"before\" }\n",
        );
        init_git_repo(dir.path());
        let started = start_build_agent_issue(dir.path(), "AF-001").unwrap();
        write_file(
            dir.path().join("src/lib.rs"),
            "pub fn status() -> &'static str { \"after\" }\n",
        );

        let request_path = write_completion_request(dir.path(), "AF-001", &started.run_id);
        let err = prepare_build_agent_review_from_request(dir.path(), &request_path).unwrap_err();

        assert!(err.to_string().contains("session snapshot"));
    }

    #[test]
    fn closeout_proof_requires_issue_already_in_review() {
        let dir = tempdir().unwrap();
        write_spec_project_fixture(dir.path(), "proj-001", "AF-001");
        write_file(
            dir.path().join("src/lib.rs"),
            "pub fn status() -> &'static str { \"before\" }\n",
        );
        init_git_repo(dir.path());
        let started = start_build_agent_issue(dir.path(), "AF-001").unwrap();
        write_running_session_snapshot(
            dir.path(),
            "AF-001",
            &started.run_id,
            started.branch_name.as_deref().unwrap(),
        );
        let issue = agentflow_spec::read_spec_issue(dir.path(), "AF-001").unwrap();

        let err = write_build_agent_closeout_proof_from_attestation(
            dir.path(),
            &issue,
            &started.run_id,
            "auto-merge-if-eligible",
            trusted_closeout(
                dir.path(),
                "github",
                "https://github.com/atxinbao/agentflow/pull/8",
                started.branch_name.as_deref().unwrap(),
                true,
                "8",
                true,
                Some("2026-06-19T11:20:08Z"),
            ),
        )
        .unwrap_err();

        assert!(err.to_string().contains("to be in_review"));
    }

    #[test]
    fn build_agent_complete_launches_next_project_issue() {
        let dir = tempdir().unwrap();
        write_two_issue_project_fixture(dir.path(), "proj-chain", "AF-CHAIN-001", "AF-CHAIN-002");
        write_file(
            dir.path().join("src/lib.rs"),
            "pub fn chain() -> &'static str { \"before\" }\n",
        );
        init_git_repo(dir.path());
        let started = start_build_agent_issue(dir.path(), "AF-CHAIN-001").unwrap();
        write_running_session_snapshot(
            dir.path(),
            "AF-CHAIN-001",
            &started.run_id,
            started.branch_name.as_deref().unwrap(),
        );
        write_file(
            dir.path().join("src/lib.rs"),
            "pub fn chain() -> &'static str { \"after\" }\n",
        );
        run_git(dir.path(), &["add", "src/lib.rs"]);
        run_git(dir.path(), &["commit", "-m", "complete first issue"]);
        let request_path = write_completion_request(dir.path(), "AF-CHAIN-001", &started.run_id);
        prepare_build_agent_review_from_request(dir.path(), &request_path).unwrap();
        let issue = agentflow_spec::read_spec_issue(dir.path(), "AF-CHAIN-001").unwrap();
        write_build_agent_closeout_proof_from_attestation(
            dir.path(),
            &issue,
            &started.run_id,
            "auto-merge-if-eligible",
            trusted_closeout(
                dir.path(),
                "github",
                "https://github.com/atxinbao/agentflow/pull/2",
                started.branch_name.as_deref().unwrap(),
                true,
                "2",
                true,
                Some("2026-06-19T11:20:02Z"),
            ),
        )
        .unwrap();

        let outcome = complete_build_agent_issue_from_request(dir.path(), &request_path).unwrap();

        let next_launch = outcome.next_launch.expect("next issue launch");
        assert_eq!(next_launch.issue_id, "AF-CHAIN-002");
        assert_eq!(next_launch.run_id, "run-001");
        let next_projection =
            agentflow_projection::load_task_projection(dir.path(), "AF-CHAIN-002").unwrap();
        assert_eq!(next_projection.current_state, "in_progress");
        assert_eq!(next_projection.latest_run_id.as_deref(), Some("run-001"));
        let next_issue = agentflow_spec::read_spec_issue(dir.path(), "AF-CHAIN-002").unwrap();
        assert_eq!(
            next_issue.status,
            agentflow_spec::SpecIssueStatus::InProgress
        );
        assert!(dir.path().join(next_launch.launch_request_path).is_file());
    }

    #[test]
    fn build_agent_review_provenance_keeps_committed_and_untracked_changes() {
        let dir = tempdir().unwrap();
        write_spec_project_fixture(dir.path(), "proj-001", "AF-001");
        write_file(
            dir.path().join("src/lib.rs"),
            "pub fn status() -> &'static str { \"before\" }\n",
        );
        init_git_repo(dir.path());
        let started = start_build_agent_issue(dir.path(), "AF-001").unwrap();
        write_running_session_snapshot(
            dir.path(),
            "AF-001",
            &started.run_id,
            started.branch_name.as_deref().unwrap(),
        );

        write_file(
            dir.path().join("src/lib.rs"),
            "pub fn status() -> &'static str { \"after-commit\" }\n",
        );
        run_git(dir.path(), &["add", "src/lib.rs"]);
        run_git(dir.path(), &["commit", "-m", "committed change"]);
        write_file(
            dir.path().join("src/lib.rs"),
            "pub fn status() -> &'static str { \"after-worktree\" }\n",
        );
        write_file(
            dir.path().join("src/helper.rs"),
            "pub fn helper() -> &'static str { \"untracked\" }\n",
        );

        let request_path = write_completion_request(dir.path(), "AF-001", &started.run_id);
        let prepared = prepare_build_agent_review_from_request(dir.path(), &request_path).unwrap();

        assert_eq!(prepared.run_status, "completed");
        let changed_files = load_task_changed_files(dir.path(), "AF-001", &started.run_id).unwrap();
        let changed_paths = changed_files
            .files
            .iter()
            .map(|file| file.path.as_str())
            .collect::<Vec<_>>();
        assert!(changed_paths.contains(&"src/lib.rs"));
        assert!(changed_paths.contains(&"src/helper.rs"));
        assert!(changed_files.base_commit.is_some());
        assert!(changed_files.head_commit.is_some());
        assert_ne!(changed_files.base_commit, changed_files.head_commit);
        assert!(changed_files.tree_sha.is_some());
        assert!(!changed_files.patch_sha256.is_empty());
        assert!(!changed_files.file_content_sha256.is_empty());
        let lib_file = changed_files
            .files
            .iter()
            .find(|file| file.path == "src/lib.rs")
            .expect("committed file present");
        assert_eq!(
            lib_file.sources,
            vec![
                TaskChangedFileSource::Committed,
                TaskChangedFileSource::WorkingTree,
            ]
        );
        let helper_file = changed_files
            .files
            .iter()
            .find(|file| file.path == "src/helper.rs")
            .expect("untracked file present");
        assert_eq!(helper_file.sources, vec![TaskChangedFileSource::Untracked]);

        let evidence = load_task_evidence(dir.path(), "AF-001").unwrap();
        assert!(evidence.validation_command_hash.is_some());
        assert!(evidence.validation_output_hash.is_some());
        assert!(evidence.patch_sha256.is_some());
        assert!(evidence.file_content_sha256.is_some());
        assert!(evidence.tree_sha.is_some());
    }

    #[test]
    fn closeout_proof_rejects_head_sha_mismatch() {
        let dir = tempdir().unwrap();
        write_spec_project_fixture(dir.path(), "proj-001", "AF-001");
        write_file(
            dir.path().join("src/lib.rs"),
            "pub fn status() -> &'static str { \"before\" }\n",
        );
        init_git_repo(dir.path());
        let started = start_build_agent_issue(dir.path(), "AF-001").unwrap();
        write_running_session_snapshot(
            dir.path(),
            "AF-001",
            &started.run_id,
            started.branch_name.as_deref().unwrap(),
        );
        write_file(
            dir.path().join("src/lib.rs"),
            "pub fn status() -> &'static str { \"after\" }\n",
        );
        let request_path = write_completion_request(dir.path(), "AF-001", &started.run_id);
        prepare_build_agent_review_from_request(dir.path(), &request_path).unwrap();
        let issue = agentflow_spec::read_spec_issue(dir.path(), "AF-001").unwrap();
        let mut attestation = trusted_closeout(
            dir.path(),
            "github",
            "https://github.com/atxinbao/agentflow/pull/4",
            started.branch_name.as_deref().unwrap(),
            true,
            "4",
            true,
            Some("2026-06-19T11:20:04Z"),
        );
        attestation.head_sha = Some("mismatch-head".to_string());

        let err = write_build_agent_closeout_proof_from_attestation(
            dir.path(),
            &issue,
            &started.run_id,
            "auto-merge-if-eligible",
            attestation,
        )
        .unwrap_err();

        assert!(err.to_string().contains("head sha mismatch"));
    }

    #[test]
    fn closeout_proof_rejects_pr_url_repository_mismatch() {
        let dir = tempdir().unwrap();
        write_spec_project_fixture(dir.path(), "proj-001", "AF-001");
        write_file(
            dir.path().join("src/lib.rs"),
            "pub fn status() -> &'static str { \"before\" }\n",
        );
        init_git_repo(dir.path());
        let started = start_build_agent_issue(dir.path(), "AF-001").unwrap();
        write_running_session_snapshot(
            dir.path(),
            "AF-001",
            &started.run_id,
            started.branch_name.as_deref().unwrap(),
        );
        write_file(
            dir.path().join("src/lib.rs"),
            "pub fn status() -> &'static str { \"after\" }\n",
        );
        let request_path = write_completion_request(dir.path(), "AF-001", &started.run_id);
        prepare_build_agent_review_from_request(dir.path(), &request_path).unwrap();
        let issue = agentflow_spec::read_spec_issue(dir.path(), "AF-001").unwrap();
        let mut attestation = trusted_closeout(
            dir.path(),
            "github",
            "https://github.com/atxinbao/agentflow/pull/5",
            started.branch_name.as_deref().unwrap(),
            true,
            "5",
            true,
            Some("2026-06-19T11:20:05Z"),
        );
        attestation.review_url = Some("https://github.com/acme/other/pull/5".to_string());

        let err = write_build_agent_closeout_proof_from_attestation(
            dir.path(),
            &issue,
            &started.run_id,
            "auto-merge-if-eligible",
            attestation,
        )
        .unwrap_err();

        assert!(err.to_string().contains("pr_url repository mismatch"));
    }

    #[test]
    fn closeout_proof_rejects_missing_closed_timestamp_for_closed_issue() {
        let dir = tempdir().unwrap();
        write_spec_project_fixture(dir.path(), "proj-001", "AF-001");
        write_file(
            dir.path().join("src/lib.rs"),
            "pub fn status() -> &'static str { \"before\" }\n",
        );
        init_git_repo(dir.path());
        let started = start_build_agent_issue(dir.path(), "AF-001").unwrap();
        write_running_session_snapshot(
            dir.path(),
            "AF-001",
            &started.run_id,
            started.branch_name.as_deref().unwrap(),
        );
        write_file(
            dir.path().join("src/lib.rs"),
            "pub fn status() -> &'static str { \"after\" }\n",
        );
        let request_path = write_completion_request(dir.path(), "AF-001", &started.run_id);
        prepare_build_agent_review_from_request(dir.path(), &request_path).unwrap();
        let issue = agentflow_spec::read_spec_issue(dir.path(), "AF-001").unwrap();
        let attestation = trusted_closeout(
            dir.path(),
            "github",
            "https://github.com/atxinbao/agentflow/pull/6",
            started.branch_name.as_deref().unwrap(),
            true,
            "6",
            true,
            None,
        );

        let err = write_build_agent_closeout_proof_from_attestation(
            dir.path(),
            &issue,
            &started.run_id,
            "auto-merge-if-eligible",
            attestation,
        )
        .unwrap_err();

        assert!(err.to_string().contains("issue_closed_at"));
    }

    #[test]
    fn build_agent_complete_stays_in_review_until_issue_is_closed() {
        let dir = tempdir().unwrap();
        write_two_issue_project_fixture(dir.path(), "proj-chain", "AF-CHAIN-001", "AF-CHAIN-002");
        write_file(
            dir.path().join("src/lib.rs"),
            "pub fn chain() -> &'static str { \"before\" }\n",
        );
        init_git_repo(dir.path());
        let started = start_build_agent_issue(dir.path(), "AF-CHAIN-001").unwrap();
        write_running_session_snapshot(
            dir.path(),
            "AF-CHAIN-001",
            &started.run_id,
            started.branch_name.as_deref().unwrap(),
        );
        write_file(
            dir.path().join("src/lib.rs"),
            "pub fn chain() -> &'static str { \"after\" }\n",
        );
        let request_path = write_completion_request(dir.path(), "AF-CHAIN-001", &started.run_id);
        prepare_build_agent_review_from_request(dir.path(), &request_path).unwrap();
        let issue = agentflow_spec::read_spec_issue(dir.path(), "AF-CHAIN-001").unwrap();
        write_build_agent_closeout_proof_from_attestation(
            dir.path(),
            &issue,
            &started.run_id,
            "auto-merge-if-eligible",
            trusted_closeout(
                dir.path(),
                "github",
                "https://github.com/atxinbao/agentflow/pull/3",
                started.branch_name.as_deref().unwrap(),
                true,
                "3",
                false,
                None,
            ),
        )
        .unwrap();

        let err = complete_build_agent_issue_from_request(dir.path(), &request_path).unwrap_err();

        assert!(err.to_string().contains("evidence gate"));
        let gate = load_task_evidence_gate_decision(dir.path(), "AF-CHAIN-001").unwrap();
        assert!(!gate.passed);
        assert!(gate
            .blockers
            .iter()
            .any(|item| item.contains("mergeProof") || item.contains("issueClosed")));
        let projection =
            agentflow_projection::load_task_projection(dir.path(), "AF-CHAIN-001").unwrap();
        assert_eq!(projection.current_state, "in_review");
        let next_projection =
            agentflow_projection::load_task_projection(dir.path(), "AF-CHAIN-002").unwrap();
        assert_eq!(next_projection.current_state, "backlog");
    }

    #[test]
    fn build_agent_complete_records_failed_gate_when_closeout_proof_is_missing() {
        let dir = tempdir().unwrap();
        write_spec_project_fixture(dir.path(), "proj-001", "AF-001");
        write_file(
            dir.path().join("src/lib.rs"),
            "pub fn status() -> &'static str { \"before\" }\n",
        );
        init_git_repo(dir.path());
        let started = start_build_agent_issue(dir.path(), "AF-001").unwrap();
        write_running_session_snapshot(
            dir.path(),
            "AF-001",
            &started.run_id,
            started.branch_name.as_deref().unwrap(),
        );
        write_file(
            dir.path().join("src/lib.rs"),
            "pub fn status() -> &'static str { \"after\" }\n",
        );
        let request_path = write_completion_request(dir.path(), "AF-001", &started.run_id);
        prepare_build_agent_review_from_request(dir.path(), &request_path).unwrap();

        let err = complete_build_agent_issue_from_request(dir.path(), &request_path).unwrap_err();

        assert!(err.to_string().contains("evidence gate"));
        let gate = load_task_evidence_gate_decision(dir.path(), "AF-001").unwrap();
        assert!(!gate.passed);
        assert!(gate
            .blockers
            .iter()
            .any(|item| item.contains("closeout proof") || item.contains("mergeProof")));
    }

    #[test]
    fn build_agent_review_fails_when_changed_files_escape_allowed_paths() {
        let dir = tempdir().unwrap();
        write_out_of_scope_fixture(dir.path(), "proj-scope", "AF-SCOPE-001");
        write_file(dir.path().join("README.md"), "before\n");
        init_git_repo(dir.path());
        let started = start_build_agent_issue(dir.path(), "AF-SCOPE-001").unwrap();
        write_running_session_snapshot(
            dir.path(),
            "AF-SCOPE-001",
            &started.run_id,
            started.branch_name.as_deref().unwrap(),
        );
        write_file(dir.path().join("README.md"), "after\n");
        let request_path = write_completion_request(dir.path(), "AF-SCOPE-001", &started.run_id);

        let err = prepare_build_agent_review_from_request(dir.path(), &request_path).unwrap_err();

        assert!(err.to_string().contains("超出允许路径：README.md"));
        let projection =
            agentflow_projection::load_task_projection(dir.path(), "AF-SCOPE-001").unwrap();
        assert_eq!(projection.current_state, "blocked");
    }

    #[test]
    fn build_agent_review_blocks_committed_out_of_scope_changes_even_after_worktree_revert() {
        let dir = tempdir().unwrap();
        write_out_of_scope_fixture(dir.path(), "proj-scope", "AF-SCOPE-001");
        write_file(dir.path().join("README.md"), "before\n");
        init_git_repo(dir.path());
        let started = start_build_agent_issue(dir.path(), "AF-SCOPE-001").unwrap();
        write_running_session_snapshot(
            dir.path(),
            "AF-SCOPE-001",
            &started.run_id,
            started.branch_name.as_deref().unwrap(),
        );
        write_file(dir.path().join("README.md"), "committed forbidden change\n");
        run_git(dir.path(), &["add", "README.md"]);
        run_git(dir.path(), &["commit", "-m", "committed forbidden change"]);
        write_file(dir.path().join("README.md"), "before\n");
        let request_path = write_completion_request(dir.path(), "AF-SCOPE-001", &started.run_id);

        let err = prepare_build_agent_review_from_request(dir.path(), &request_path).unwrap_err();

        assert!(err.to_string().contains("超出允许路径：README.md"));
        let changed_files =
            load_task_changed_files(dir.path(), "AF-SCOPE-001", &started.run_id).unwrap();
        let readme = changed_files
            .files
            .iter()
            .find(|file| file.path == "README.md")
            .expect("README tracked in trusted set");
        assert_eq!(
            readme.sources,
            vec![
                TaskChangedFileSource::Committed,
                TaskChangedFileSource::WorkingTree,
            ]
        );
    }

    fn write_spec_project_fixture(root: &Path, project_id: &str, issue_id: &str) {
        let requirement = root.join("docs/requirements/034-complete-test.md");
        fs::create_dir_all(requirement.parent().unwrap()).unwrap();
        fs::write(
            &requirement,
            "# Complete Test\n\n验证 Build Agent complete 写入 task event。\n",
        )
        .unwrap();
        let mut issue = agentflow_spec::SpecIssueDraft::new(issue_id);
        issue.project_id = Some(project_id.to_string());
        issue.allowed_paths = vec!["src/**".to_string()];
        issue.validation_commands = vec!["test -f src/lib.rs".to_string()];
        let issue = agentflow_spec::issue_from_requirement(root, &requirement, issue).unwrap();
        agentflow_spec::write_spec_issue(root, &issue).unwrap();
        let mut project = agentflow_spec::SpecProjectDraft::new(project_id);
        project.issue_ids = vec![issue_id.to_string()];
        let project =
            agentflow_spec::project_from_requirement(root, &requirement, project).unwrap();
        agentflow_spec::write_spec_project(root, &project).unwrap();
    }

    fn write_two_issue_project_fixture(
        root: &Path,
        project_id: &str,
        first_id: &str,
        second_id: &str,
    ) {
        let requirement = root.join("docs/requirements/034-complete-chain-test.md");
        fs::create_dir_all(requirement.parent().unwrap()).unwrap();
        fs::write(
            &requirement,
            "# Complete Chain Test\n\n验证 Done 后自动推进下一条任务。\n",
        )
        .unwrap();
        let mut first = agentflow_spec::SpecIssueDraft::new(first_id);
        first.project_id = Some(project_id.to_string());
        first.allowed_paths = vec!["src/**".to_string()];
        first.validation_commands = vec!["test -f src/lib.rs".to_string()];
        let first = agentflow_spec::issue_from_requirement(root, &requirement, first).unwrap();
        agentflow_spec::write_spec_issue(root, &first).unwrap();

        let mut second = agentflow_spec::SpecIssueDraft::new(second_id);
        second.project_id = Some(project_id.to_string());
        second.blocked_by = vec![first_id.to_string()];
        second.allowed_paths = vec!["src/**".to_string()];
        second.validation_commands = vec!["test -f src/lib.rs".to_string()];
        let second = agentflow_spec::issue_from_requirement(root, &requirement, second).unwrap();
        agentflow_spec::write_spec_issue(root, &second).unwrap();

        let mut project = agentflow_spec::SpecProjectDraft::new(project_id);
        project.issue_ids = vec![first_id.to_string(), second_id.to_string()];
        let project =
            agentflow_spec::project_from_requirement(root, &requirement, project).unwrap();
        agentflow_spec::write_spec_project(root, &project).unwrap();
    }

    fn write_completion_request(root: &Path, issue_id: &str, run_id: &str) -> PathBuf {
        let path = root
            .join(".agentflow/tmp")
            .join(format!("{run_id}-completion-request.json"));
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(
            &path,
            serde_json::to_string_pretty(&serde_json::json!({
                "issueId": issue_id,
                "runId": run_id
            }))
            .unwrap(),
        )
        .unwrap();
        path
    }

    fn write_out_of_scope_fixture(root: &Path, project_id: &str, issue_id: &str) {
        let requirement = root.join("docs/requirements/034-scope-test.md");
        fs::create_dir_all(requirement.parent().unwrap()).unwrap();
        fs::write(&requirement, "# Scope Test\n\n验证路径边界阻断。\n").unwrap();
        let mut issue = agentflow_spec::SpecIssueDraft::new(issue_id);
        issue.project_id = Some(project_id.to_string());
        issue.allowed_paths = vec!["src/**".to_string()];
        issue.validation_commands = vec!["test -f README.md".to_string()];
        let issue = agentflow_spec::issue_from_requirement(root, &requirement, issue).unwrap();
        agentflow_spec::write_spec_issue(root, &issue).unwrap();
        let mut project = agentflow_spec::SpecProjectDraft::new(project_id);
        project.issue_ids = vec![issue_id.to_string()];
        let project =
            agentflow_spec::project_from_requirement(root, &requirement, project).unwrap();
        agentflow_spec::write_spec_project(root, &project).unwrap();
    }

    fn trusted_closeout(
        root: &Path,
        provider: &str,
        review_url: &str,
        source_branch: &str,
        merged: bool,
        issue_ref: &str,
        issue_closed: bool,
        closed_at: Option<&str>,
    ) -> McpCloseoutAttestation {
        let head_sha = git_output(root, &["rev-parse", "HEAD"]);
        McpCloseoutAttestation {
            version: MCP_CLOSEOUT_ATTESTATION_VERSION.to_string(),
            provider: provider.to_string(),
            review_ref: review_url.to_string(),
            review_url: Some(review_url.to_string()),
            repository_full_name: Some("atxinbao/agentflow".to_string()),
            source_branch: Some(source_branch.to_string()),
            target_branch: Some("main".to_string()),
            base_sha: Some(head_sha.clone()),
            head_sha: Some(head_sha),
            merge_commit_sha: merged.then(|| format!("merge-{issue_ref}")),
            merged,
            merged_at: merged.then(|| "2026-06-19T11:19:59Z".to_string()),
            issue_closed,
            issues: vec![McpCloseoutIssueAttestation {
                issue_ref: issue_ref.to_string(),
                issue_url: Some(format!(
                    "https://github.com/atxinbao/agentflow/issues/{issue_ref}"
                )),
                closed: issue_closed,
                closed_at: closed_at.map(ToString::to_string),
            }],
            queried_at: 1_718_000_000,
        }
    }

    fn write_file(path: PathBuf, content: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, content).unwrap();
    }

    fn init_git_repo(root: &Path) {
        write_file(root.join(".gitignore"), ".agentflow/\n");
        run_git(root, &["init"]);
        run_git(root, &["config", "user.email", "codex@example.com"]);
        run_git(root, &["config", "user.name", "Codex"]);
        run_git(root, &["add", "."]);
        run_git(root, &["commit", "-m", "initial fixture"]);
    }

    fn run_git(root: &Path, args: &[&str]) {
        let status = Command::new("git")
            .args(args)
            .current_dir(root)
            .status()
            .unwrap();
        assert!(status.success(), "git {:?} failed", args);
    }

    fn git_output(root: &Path, args: &[&str]) -> String {
        let output = Command::new("git")
            .args(args)
            .current_dir(root)
            .output()
            .unwrap();
        assert!(output.status.success(), "git {:?} failed", args);
        String::from_utf8_lossy(&output.stdout)
            .trim_end()
            .to_string()
    }
}
