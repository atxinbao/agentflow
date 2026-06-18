//! Active CLI boundary.
//!
//! Active commands are narrow wrappers around the current workspace crates.
//! They must not call archived 2026-05 writers.

use agentflow_event_store::{
    append_task_event_once, EventActor, EventStateTransition, TaskEventDraft,
};
use agentflow_release::{PublicReleaseDocumentPaths, PublicReleaseDocumentTarget};
use agentflow_spec::read_spec_issue;
use agentflow_task_artifacts::{
    load_task_changed_files, load_task_evidence, load_task_run, task_changed_files_path,
    task_evidence_dir, task_run_dir, update_task_run_status, write_task_changed_files,
    write_task_command_record, write_task_evidence, write_task_validation_with_assessment,
    TaskChangedFile, TaskCommandInput, TaskEvidence, TaskRun, TaskRunStatus,
};
use agentflow_task_loop::{AgentLaunchPayload, TaskLoop, TaskLoopLaunch, AGENT_LAUNCH_REQUESTED};
use agentflow_workflow_core::{WorkflowAgentRole, WorkflowFlowType};
use anyhow::{Context, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};
use serde::Deserialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
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
    pub changelog_path: PathBuf,
    pub release_notes_path: PathBuf,
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
    working_tree_hash: String,
}

pub(crate) fn complete_build_agent_issue_from_request(
    root: &Path,
    request_path: &Path,
) -> Result<BuildAgentCompletionOutcome> {
    assert_current_cli_is_fresh(root)?;
    let request = read_completion_request(request_path, "completion")?;
    let review = ensure_review_prepared(root, request.clone())?;
    let mut proof = load_closeout_proof(root, &review.issue_id, &review.run_id)?;
    ensure_closeout_ready(&review.issue_id, &review.run_id, &proof)?;
    let issue = read_spec_issue(root, &review.issue_id)?;
    let completed_project_id = issue.project_id.clone();
    let evidence = load_task_evidence(root, &review.issue_id)?;
    let public_delivery_target = PublicReleaseDocumentTarget::default();
    let public_delivery_paths = write_public_delivery_documents(root, &public_delivery_target)?;
    proof["publicDeliveryWritten"] = serde_json::Value::Bool(true);
    proof["changelogPath"] =
        serde_json::Value::String(public_delivery_paths.changelog_path.clone());
    proof["releaseNotesPath"] =
        serde_json::Value::String(public_delivery_paths.release_notes_path.clone());
    proof["releaseNotesUrl"] =
        serde_json::Value::String(public_delivery_paths.release_notes_path.clone());
    write_json(
        &closeout_proof_path(root, &review.issue_id, &review.run_id),
        &proof,
    )?;
    let proof_path = closeout_proof_path(root, &review.issue_id, &review.run_id);
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
                "remoteUrl": proof.get("remoteUrl").cloned().unwrap_or(serde_json::Value::Null),
                "prUrl": proof.get("prUrl").cloned().unwrap_or(serde_json::Value::Null),
                "issueClosed": proof.get("issueClosed").cloned().unwrap_or(serde_json::Value::Null),
                "closedAt": proof.get("closedAt").cloned().unwrap_or(serde_json::Value::Null),
                "changelogPath": public_delivery_paths.changelog_path,
                "releaseNotesUrl": public_delivery_paths.release_notes_path,
            }),
            artifact_refs: vec![
                task_evidence_path(&review.issue_id),
                relative_path(root, &proof_path),
                public_delivery_paths.changelog_path.clone(),
                public_delivery_paths.release_notes_path.clone(),
            ],
            idempotency_key: Some(format!(
                "issue.completed:{}:{}",
                issue.issue_id, review.run_id
            )),
        },
    )?;
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
        changelog_path: root.join(public_delivery_paths.changelog_path),
        release_notes_path: root.join(public_delivery_paths.release_notes_path),
        next_launch,
    })
}

pub(crate) fn prepare_build_agent_review_from_request(
    root: &Path,
    request_path: &Path,
) -> Result<BuildAgentReview> {
    assert_current_cli_is_fresh(root)?;
    let request = read_completion_request(request_path, "review preparation")?;
    ensure_review_prepared(root, request)
}

fn write_public_delivery_documents(
    root: &Path,
    target: &PublicReleaseDocumentTarget,
) -> Result<PublicReleaseDocumentPaths> {
    let summary = agentflow_release::collect_public_release_summary(root)?;
    agentflow_release::write_public_release_documents(root, &summary, target)
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
    merged: bool,
    issue_closed: bool,
    closed_at: Option<u64>,
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
    let proof_path = closeout_proof_path(root, &issue.issue_id, run_id);
    write_json(
        &proof_path,
        &json!({
            "version": "task-closeout-proof.v1",
            "issueId": issue.issue_id,
            "projectId": issue.project_id,
            "runId": run_id,
            "provider": provider,
            "mergeMode": merge_mode,
            "remoteUrl": remote_url,
            "prUrl": remote_url,
            "merged": merged,
            "issueClosed": issue_closed,
            "closedAt": closed_at,
            "publicDeliveryWritten": false,
        }),
    )?;
    append_task_event_once(
        root,
        TaskEventDraft {
            flow_type: WorkflowFlowType::Work,
            aggregate_type: "issue".to_string(),
            aggregate_id: issue_id.to_string(),
            project_id: issue.project_id.clone(),
            issue_id: Some(issue_id.to_string()),
            run_id: Some(run_id.to_string()),
            event_type: "issue.closeout.proof.recorded".to_string(),
            authority_role: Some(WorkflowAgentRole::WorkAgent),
            actor: EventActor {
                role: "build-agent".to_string(),
                kind: "system".to_string(),
            },
            state: None,
            correlation_id: Some(format!("corr-{issue_id}")),
            causation_id: None,
            payload: json!({
                "issueId": issue_id,
                "projectId": issue.project_id,
                "runId": run_id,
                "provider": provider,
                "mergeMode": merge_mode,
                "remoteUrl": remote_url,
                "prUrl": remote_url,
                "merged": merged,
                "issueClosed": issue_closed,
                "closedAt": closed_at,
                "publicDeliveryWritten": false,
            }),
            artifact_refs: vec![relative_path(root, &proof_path)],
            idempotency_key: Some(format!("issue.closeout-proof.recorded:{issue_id}:{run_id}")),
        },
    )?;
    let _ = agentflow_projection::rebuild_projections(root)?;
    agentflow_state::refresh_state(root)?;
    Ok(BuildAgentCloseoutProof {
        issue_id: issue_id.to_string(),
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
    let changed_files = collect_trusted_changed_files(root)?;
    write_task_changed_files(
        root,
        issue_id,
        run_id,
        changed_files.files.clone(),
        changed_files.base_commit.clone(),
        changed_files.head_commit.clone(),
        changed_files.working_tree_hash.clone(),
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
        update_task_run_status(root, issue_id, run_id, TaskRunStatus::Completed)?;
    } else {
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
                    })
                }).collect::<Vec<_>>(),
                "validationCommandCount": validation.command_ids.len(),
                "boundaryFailures": validation.boundary_failures,
                "changedFilesPath": relative_path(root, &task_changed_files_path(root, issue_id, run_id)?),
                "commandHash": validation.command_hash,
                "changedFileHash": validation.changed_file_hash,
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

fn collect_trusted_changed_files(root: &Path) -> Result<TrustedChangedFilesSnapshot> {
    let status_output = git_stdout(root, &["status", "--porcelain=v1", "--untracked-files=all"])?;
    let diff_stats = collect_diff_stats(root)?;
    let files = status_output
        .lines()
        .filter_map(parse_status_line)
        .filter(|file| !file.path.starts_with(".agentflow/"))
        .map(|mut file| {
            if let Some((insertions, deletions)) = diff_stats.get(&file.path) {
                file.insertions = *insertions;
                file.deletions = *deletions;
            }
            file
        })
        .collect::<Vec<_>>();
    let head_commit = Some(git_stdout(root, &["rev-parse", "HEAD"])?);
    let base_commit = resolve_base_commit(root)
        .ok()
        .or_else(|| head_commit.clone());
    let working_tree_hash = git_stdout(
        root,
        &["status", "--porcelain=v1", "--untracked-files=all", "-z"],
    )
    .map(|raw| sha256_hex(raw.as_bytes()))?;
    Ok(TrustedChangedFilesSnapshot {
        files,
        base_commit,
        head_commit,
        working_tree_hash,
    })
}

fn collect_diff_stats(root: &Path) -> Result<std::collections::HashMap<String, (usize, usize)>> {
    let raw = git_stdout(root, &["diff", "--numstat", "HEAD", "--"])?;
    let mut stats = std::collections::HashMap::new();
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

fn parse_status_line(line: &str) -> Option<TaskChangedFile> {
    if line.len() < 4 {
        return None;
    }
    let status = &line[..2];
    let raw_path = line[3..].trim();
    if raw_path.is_empty() {
        return None;
    }
    Some(TaskChangedFile {
        path: normalize_status_path(raw_path),
        change_type: status_to_change_type(status),
        insertions: 0,
        deletions: 0,
    })
}

fn normalize_status_path(path: &str) -> String {
    path.rsplit(" -> ")
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

fn ensure_closeout_ready(issue_id: &str, run_id: &str, proof: &serde_json::Value) -> Result<()> {
    if !proof
        .get("merged")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
    {
        anyhow::bail!(
            "build agent completion requires merged PR/MR proof for {} {}",
            issue_id,
            run_id
        );
    }
    if !proof
        .get("issueClosed")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
    {
        anyhow::bail!(
            "build agent completion requires closed issue proof for {} {}",
            issue_id,
            run_id
        );
    }
    Ok(())
}

fn closeout_proof_path(root: &Path, issue_id: &str, run_id: &str) -> PathBuf {
    task_run_dir(root, issue_id, run_id)
        .expect("validated task run dir")
        .join("review/closeout-proof.json")
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
        write_build_agent_closeout_proof,
    };
    use agentflow_mcp::{
        McpAgentProvider, McpLaunchMode, McpLaunchPlan, McpLaunchRequest, McpProviderBridge,
        McpProviderKind, McpProviderStatus, McpProviderStatusCode,
    };
    use agentflow_task_artifacts::{load_task_changed_files, load_task_evidence};
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
        let issue =
            agentflow_spec::issue_from_requirement(dir.path(), &requirement, issue).unwrap();
        agentflow_spec::write_spec_issue(dir.path(), &issue).unwrap();
        let mut project = agentflow_spec::SpecProjectDraft::new("proj-001");
        project.issue_ids = vec!["AF-001".to_string()];
        let project =
            agentflow_spec::project_from_requirement(dir.path(), &requirement, project).unwrap();
        agentflow_spec::write_spec_project(dir.path(), &project).unwrap();
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
        let issue =
            agentflow_spec::issue_from_requirement(dir.path(), &requirement, issue).unwrap();
        agentflow_spec::write_spec_issue(dir.path(), &issue).unwrap();
        let mut project = agentflow_spec::SpecProjectDraft::new("proj-start");
        project.issue_ids = vec!["AF-START-001".to_string()];
        let project =
            agentflow_spec::project_from_requirement(dir.path(), &requirement, project).unwrap();
        agentflow_spec::write_spec_project(dir.path(), &project).unwrap();

        let started = start_build_agent_issue(dir.path(), "AF-START-001").unwrap();

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
        assert!(evidence.command_hash.is_some());
        assert!(evidence.changed_file_hash.is_some());
        assert!(evidence.validation_result_hash.is_some());
        let in_review_projection =
            agentflow_projection::load_task_projection(dir.path(), "AF-001").unwrap();
        assert_eq!(in_review_projection.current_state, "in_review");
        assert!(!dir.path().join(".agentflow/output").exists());
        assert!(!dir.path().join(".agentflow/execute").exists());
        assert!(!dir.path().join(".agentflow/input").exists());

        write_build_agent_closeout_proof(
            dir.path(),
            "AF-001",
            &started.run_id,
            "github",
            "auto-merge-if-eligible",
            Some("https://github.com/atxinbao/agentflow/pull/1".to_string()),
            true,
            true,
            Some(1_718_000_001),
        )
        .unwrap();
        let still_review_projection =
            agentflow_projection::load_task_projection(dir.path(), "AF-001").unwrap();
        assert_eq!(still_review_projection.current_state, "in_review");

        let outcome = complete_build_agent_issue_from_request(dir.path(), &request_path).unwrap();
        assert_eq!(outcome.issue_id, "AF-001");
        assert_eq!(outcome.run_status, "completed");
        assert!(outcome.next_launch.is_none());
        assert_eq!(outcome.changelog_path, dir.path().join("CHANGELOG.md"));
        assert_eq!(
            outcome.release_notes_path,
            dir.path()
                .join("docs/release-notes/agentflow-release-notes.md")
        );
        assert!(outcome.changelog_path.is_file());
        assert!(outcome.release_notes_path.is_file());
        let done_projection =
            agentflow_projection::load_task_projection(dir.path(), "AF-001").unwrap();
        assert_eq!(done_projection.current_state, "done");
        assert_eq!(
            done_projection.public_delivery.pr_url.as_deref(),
            Some("https://github.com/atxinbao/agentflow/pull/1")
        );
        assert_eq!(
            done_projection.public_delivery.changelog_path.as_deref(),
            Some("CHANGELOG.md")
        );
        assert_eq!(
            done_projection.public_delivery.release_notes_url.as_deref(),
            Some("docs/release-notes/agentflow-release-notes.md")
        );
        let proof = load_closeout_proof(dir.path(), "AF-001", &started.run_id).unwrap();
        assert!(proof["publicDeliveryWritten"].as_bool().unwrap_or(false));
        assert_eq!(proof["changelogPath"].as_str(), Some("CHANGELOG.md"));
        assert_eq!(
            proof["releaseNotesPath"].as_str(),
            Some("docs/release-notes/agentflow-release-notes.md")
        );
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
    fn build_agent_complete_launches_next_project_issue() {
        let dir = tempdir().unwrap();
        write_two_issue_project_fixture(dir.path(), "proj-chain", "AF-CHAIN-001", "AF-CHAIN-002");
        write_file(
            dir.path().join("src/lib.rs"),
            "pub fn chain() -> &'static str { \"before\" }\n",
        );
        init_git_repo(dir.path());
        let started = start_build_agent_issue(dir.path(), "AF-CHAIN-001").unwrap();
        write_file(
            dir.path().join("src/lib.rs"),
            "pub fn chain() -> &'static str { \"after\" }\n",
        );
        let request_path = write_completion_request(dir.path(), "AF-CHAIN-001", &started.run_id);
        prepare_build_agent_review_from_request(dir.path(), &request_path).unwrap();
        write_build_agent_closeout_proof(
            dir.path(),
            "AF-CHAIN-001",
            &started.run_id,
            "github",
            "auto-merge-if-eligible",
            Some("https://github.com/atxinbao/agentflow/pull/2".to_string()),
            true,
            true,
            Some(1_718_000_002),
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
        assert!(dir.path().join(next_launch.launch_request_path).is_file());
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
        write_file(
            dir.path().join("src/lib.rs"),
            "pub fn chain() -> &'static str { \"after\" }\n",
        );
        let request_path = write_completion_request(dir.path(), "AF-CHAIN-001", &started.run_id);
        prepare_build_agent_review_from_request(dir.path(), &request_path).unwrap();
        write_build_agent_closeout_proof(
            dir.path(),
            "AF-CHAIN-001",
            &started.run_id,
            "github",
            "auto-merge-if-eligible",
            Some("https://github.com/atxinbao/agentflow/pull/3".to_string()),
            true,
            false,
            None,
        )
        .unwrap();

        let err = complete_build_agent_issue_from_request(dir.path(), &request_path).unwrap_err();

        assert!(err.to_string().contains("closed issue proof"));
        let projection =
            agentflow_projection::load_task_projection(dir.path(), "AF-CHAIN-001").unwrap();
        assert_eq!(projection.current_state, "in_review");
        let next_projection =
            agentflow_projection::load_task_projection(dir.path(), "AF-CHAIN-002").unwrap();
        assert_eq!(next_projection.current_state, "backlog");
    }

    #[test]
    fn build_agent_review_fails_when_changed_files_escape_allowed_paths() {
        let dir = tempdir().unwrap();
        write_out_of_scope_fixture(dir.path(), "proj-scope", "AF-SCOPE-001");
        write_file(dir.path().join("README.md"), "before\n");
        init_git_repo(dir.path());
        let started = start_build_agent_issue(dir.path(), "AF-SCOPE-001").unwrap();
        write_file(dir.path().join("README.md"), "after\n");
        let request_path = write_completion_request(dir.path(), "AF-SCOPE-001", &started.run_id);

        let err = prepare_build_agent_review_from_request(dir.path(), &request_path).unwrap_err();

        assert!(err.to_string().contains("超出允许路径：README.md"));
        let projection =
            agentflow_projection::load_task_projection(dir.path(), "AF-SCOPE-001").unwrap();
        assert_eq!(projection.current_state, "blocked");
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
}
