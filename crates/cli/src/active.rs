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
    load_task_evidence, load_task_run, task_evidence_dir, task_run_dir, update_task_run_status,
    write_task_command_record, write_task_evidence, write_task_validation, TaskCommandInput,
    TaskEvidence, TaskRun, TaskRunStatus,
};
use agentflow_task_loop::{AgentLaunchPayload, TaskLoop, TaskLoopLaunch, AGENT_LAUNCH_REQUESTED};
use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::json;
use std::{
    fs,
    path::{Path, PathBuf},
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
    "crates/agent-bridge/src",
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
pub(crate) struct BuildAgentMergeProof {
    pub issue_id: String,
    pub run_id: String,
    pub merged: bool,
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
    #[serde(default)]
    changed_files: Vec<BuildAgentChangedFile>,
    #[serde(default)]
    validation_commands: Vec<BuildAgentValidationCommand>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BuildAgentChangedFile {
    path: String,
    change_type: String,
    insertions: usize,
    deletions: usize,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BuildAgentValidationCommand {
    label: String,
    program: String,
    args: Vec<String>,
    exit_code: Option<i32>,
    stdout: Option<String>,
    stderr: Option<String>,
    source: Option<String>,
}

pub(crate) fn complete_build_agent_issue_from_request(
    root: &Path,
    request_path: &Path,
) -> Result<BuildAgentCompletionOutcome> {
    assert_current_cli_is_fresh(root)?;
    let request = read_completion_request(request_path, "completion")?;
    let review = ensure_review_prepared(root, request.clone())?;
    let proof = load_merge_proof(root, &review.issue_id, &review.run_id)?;
    if !proof
        .get("merged")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
    {
        anyhow::bail!(
            "build agent completion requires merged PR/MR proof for {} {}",
            review.issue_id,
            review.run_id
        );
    }
    let issue = read_spec_issue(root, &review.issue_id)?;
    let evidence = load_task_evidence(root, &review.issue_id)?;
    let proof_path = merge_proof_path(root, &review.issue_id, &review.run_id);
    let public_delivery_target = PublicReleaseDocumentTarget::default();
    append_task_event_once(
        root,
        TaskEventDraft {
            aggregate_type: "issue".to_string(),
            aggregate_id: issue.issue_id.clone(),
            project_id: issue.project_id.clone(),
            issue_id: Some(issue.issue_id.clone()),
            event_type: "issue.completed".to_string(),
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
                "mergeProofPath": relative_path(root, &proof_path),
                "provider": proof.get("provider").cloned().unwrap_or(serde_json::Value::Null),
                "mergeMode": proof.get("mergeMode").cloned().unwrap_or(serde_json::Value::Null),
                "remoteUrl": proof.get("remoteUrl").cloned().unwrap_or(serde_json::Value::Null),
                "prUrl": proof.get("remoteUrl").cloned().unwrap_or(serde_json::Value::Null),
                "changelogPath": public_delivery_target.changelog_path.display().to_string(),
                "releaseNotesUrl": public_delivery_target.release_notes_path.display().to_string(),
            }),
            artifact_refs: vec![
                task_evidence_path(&review.issue_id),
                relative_path(root, &proof_path),
                public_delivery_target.changelog_path.display().to_string(),
                public_delivery_target
                    .release_notes_path
                    .display()
                    .to_string(),
            ],
            idempotency_key: Some(format!("issue.completed:{}", review.run_id)),
        },
    )?;
    let _ = agentflow_projection::rebuild_projections(root)?;
    let public_delivery_paths = write_public_delivery_documents(root, &public_delivery_target)?;
    agentflow_state::refresh_state(root)?;
    Ok(BuildAgentCompletionOutcome {
        issue_id: review.issue_id,
        run_id: review.run_id,
        run_status: review.run_status,
        validation_passed: review.validation_passed,
        evidence_path: evidence_path(root, &evidence),
        changelog_path: root.join(public_delivery_paths.changelog_path),
        release_notes_path: root.join(public_delivery_paths.release_notes_path),
        next_launch: None,
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
    claim_next_build_agent_launch_with_bridge(
        root,
        &agentflow_agent_bridge::AgentBridge::with_default_providers(),
    )
}

fn claim_next_build_agent_launch_with_bridge(
    root: &Path,
    bridge: &agentflow_agent_bridge::AgentBridge,
) -> Result<Option<BuildAgentLaunchClaim>> {
    let Some(claim) = bridge.claim_next_launch(root)? else {
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
            event.event_type == AGENT_LAUNCH_REQUESTED
                && event
                    .payload
                    .get("runId")
                    .and_then(serde_json::Value::as_str)
                    == Some(run_id)
        })
        .ok_or_else(|| anyhow::anyhow!("missing agent launch request for run {run_id}"))?;
    serde_json::from_value(event.payload)
        .with_context(|| format!("parse agent launch payload {}", event.event_id))
}

pub(crate) fn write_build_agent_merge_proof(
    root: &Path,
    issue_id: &str,
    run_id: &str,
    provider: &str,
    merge_mode: &str,
    remote_url: Option<String>,
    merged: bool,
) -> Result<BuildAgentMergeProof> {
    assert_current_cli_is_fresh(root)?;
    let issue =
        read_spec_issue(root, issue_id).with_context(|| format!("load spec issue {issue_id}"))?;
    let run = load_task_run(root, &issue.issue_id, run_id)?;
    if run.issue_id != issue.issue_id {
        anyhow::bail!(
            "merge proof issueId mismatch: request {}, run {}",
            issue.issue_id,
            run.issue_id
        );
    }
    let proof_path = merge_proof_path(root, &issue.issue_id, run_id);
    write_json(
        &proof_path,
        &json!({
            "version": "task-merge-proof.v1",
            "issueId": issue.issue_id,
            "projectId": issue.project_id,
            "runId": run_id,
            "provider": provider,
            "mergeMode": merge_mode,
            "remoteUrl": remote_url,
            "prUrl": remote_url,
            "merged": merged,
        }),
    )?;
    append_task_event_once(
        root,
        TaskEventDraft {
            aggregate_type: "issue".to_string(),
            aggregate_id: issue_id.to_string(),
            project_id: issue.project_id.clone(),
            issue_id: Some(issue_id.to_string()),
            event_type: "issue.merge.proof.recorded".to_string(),
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
            }),
            artifact_refs: vec![relative_path(root, &proof_path)],
            idempotency_key: Some(format!("issue.merge-proof.recorded:{run_id}")),
        },
    )?;
    let _ = agentflow_projection::rebuild_projections(root)?;
    agentflow_state::refresh_state(root)?;
    Ok(BuildAgentMergeProof {
        issue_id: issue_id.to_string(),
        run_id: run_id.to_string(),
        merged,
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
    if request.validation_commands.is_empty() {
        anyhow::bail!("build agent review preparation requires validation command results");
    }

    let issue =
        read_spec_issue(root, issue_id).with_context(|| format!("load spec issue {issue_id}"))?;
    let run = load_task_run(root, issue_id, run_id)?;
    if run.issue_id != issue.issue_id {
        anyhow::bail!(
            "build agent review preparation issueId mismatch: request {}, run {}",
            issue.issue_id,
            run.issue_id
        );
    }

    let existing_evidence = task_evidence_dir(root, issue_id).join("evidence.json");
    let evidence = if existing_evidence.is_file() {
        load_task_evidence(root, issue_id)?
    } else {
        update_task_run_status(root, issue_id, run_id, TaskRunStatus::Validating)?;
        for command in &request.validation_commands {
            write_task_command_record(
                root,
                issue_id,
                run_id,
                TaskCommandInput {
                    label: command.label.clone(),
                    program: command.program.clone(),
                    args: command.args.clone(),
                    exit_code: command.exit_code,
                    stdout: command.stdout.clone().unwrap_or_default(),
                    stderr: command.stderr.clone().unwrap_or_default(),
                },
            )?;
        }
        let validation = write_task_validation(root, issue_id, run_id)?;
        let evidence = write_task_evidence(
            root,
            issue_id,
            run_id,
            format!(
                "验证命令 {} 条，{}。",
                validation.command_ids.len(),
                if validation.passed {
                    "全部通过"
                } else {
                    "存在失败"
                }
            ),
        )?;
        if validation.passed {
            update_task_run_status(root, issue_id, run_id, TaskRunStatus::Completed)?;
        } else {
            update_task_run_status(root, issue_id, run_id, TaskRunStatus::Failed)?;
            append_validation_failed_event(root, &issue, run_id, &evidence)?;
            let _ = agentflow_projection::rebuild_projections(root)?;
            agentflow_state::refresh_state(root)?;
            anyhow::bail!("build agent review preparation validation failed for {issue_id}");
        }
        evidence
    };

    append_task_event_once(
        root,
        TaskEventDraft {
            aggregate_type: "issue".to_string(),
            aggregate_id: issue.issue_id.clone(),
            project_id: issue.project_id.clone(),
            issue_id: Some(issue.issue_id.clone()),
            event_type: "issue.validation.passed".to_string(),
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
                "changedFiles": request.changed_files.iter().map(|file| {
                    json!({
                        "path": file.path.as_str(),
                        "changeType": file.change_type.as_str(),
                        "insertions": file.insertions,
                        "deletions": file.deletions,
                    })
                }).collect::<Vec<_>>(),
                "validationCommands": request.validation_commands.iter().map(|command| {
                    json!({
                        "label": command.label.as_str(),
                        "program": command.program.as_str(),
                        "args": &command.args,
                        "exitCode": command.exit_code,
                        "source": command.source.as_deref(),
                    })
                }).collect::<Vec<_>>(),
                "validationCommandCount": request.validation_commands.len(),
                "evidencePath": task_evidence_path(issue_id),
            }),
            artifact_refs: vec![task_evidence_path(issue_id)],
            idempotency_key: Some(format!("issue.validation.passed:{run_id}")),
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

fn append_validation_failed_event(
    root: &Path,
    issue: &agentflow_spec::SpecIssue,
    run_id: &str,
    evidence: &TaskEvidence,
) -> Result<()> {
    append_task_event_once(
        root,
        TaskEventDraft {
            aggregate_type: "issue".to_string(),
            aggregate_id: issue.issue_id.clone(),
            project_id: issue.project_id.clone(),
            issue_id: Some(issue.issue_id.clone()),
            event_type: "issue.validation.failed".to_string(),
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
            idempotency_key: Some(format!("issue.validation.failed:{run_id}")),
        },
    )?;
    Ok(())
}

fn load_merge_proof(root: &Path, issue_id: &str, run_id: &str) -> Result<serde_json::Value> {
    let path = merge_proof_path(root, issue_id, run_id);
    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

fn merge_proof_path(root: &Path, issue_id: &str, run_id: &str) -> PathBuf {
    task_run_dir(root, issue_id, run_id).join("review/merge-proof.json")
}

fn task_evidence_path(issue_id: &str) -> String {
    format!(".agentflow/tasks/{issue_id}/evidence/evidence.json")
}

fn evidence_path(root: &Path, evidence: &TaskEvidence) -> PathBuf {
    task_evidence_dir(root, &evidence.issue_id).join("evidence.json")
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
        binary_is_stale, claim_next_build_agent_launch_with_bridge,
        complete_build_agent_issue_from_request, is_local_target_binary,
        prepare_build_agent_review_from_request, rebuild_hint, start_build_agent_issue,
        write_build_agent_merge_proof,
    };
    use agentflow_mcp::{
        McpAgentProvider, McpLaunchMode, McpLaunchPlan, McpLaunchRequest, McpProviderBridge,
        McpProviderKind, McpProviderStatus, McpProviderStatusCode,
    };
    use anyhow::Result;
    use std::{
        fs,
        path::{Path, PathBuf},
        time::{Duration, UNIX_EPOCH},
    };
    use tempfile::tempdir;

    struct FakeProvider;

    impl McpAgentProvider for FakeProvider {
        fn provider_id(&self) -> &'static str {
            "fake"
        }

        fn kind(&self) -> McpProviderKind {
            McpProviderKind::Codex
        }

        fn check_health(&self, _project_root: &Path) -> McpProviderStatus {
            let mut status = McpProviderStatus::new(McpProviderKind::Codex, 1);
            status.provider = "fake".to_string();
            status.status = McpProviderStatusCode::Ready;
            status
        }

        fn build_launch_plan(
            &self,
            _project_root: &Path,
            request: &McpLaunchRequest,
        ) -> Result<McpLaunchPlan> {
            let mut plan = McpLaunchPlan::new(
                "fake",
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
            "# Claim Test\n\n验证 CLI claim 走 AgentBridge。\n",
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
            .request_agent_launch(dir.path(), "AF-001", "fake")
            .unwrap();
        let mut providers = McpProviderBridge::new();
        providers.register(Box::new(FakeProvider));
        let bridge = agentflow_agent_bridge::AgentBridge::new(providers);

        let claim = claim_next_build_agent_launch_with_bridge(dir.path(), &bridge)
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
            .any(|event| event.event_type == agentflow_agent_bridge::AGENT_SESSION_CREATED));
        assert!(
            claim_next_build_agent_launch_with_bridge(dir.path(), &bridge)
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
        let started = start_build_agent_issue(dir.path(), "AF-001").unwrap();

        let request_path = write_completion_request(dir.path(), "AF-001", &started.run_id);
        let prepared = prepare_build_agent_review_from_request(dir.path(), &request_path).unwrap();
        assert_eq!(prepared.issue_id, "AF-001");
        assert_eq!(prepared.run_status, "completed");
        assert!(prepared.validation_passed);
        assert!(prepared.evidence_path.is_file());
        let in_review_projection =
            agentflow_projection::load_task_projection(dir.path(), "AF-001").unwrap();
        assert_eq!(in_review_projection.current_state, "in_review");
        assert!(!dir.path().join(".agentflow/output").exists());
        assert!(!dir.path().join(".agentflow/execute").exists());
        assert!(!dir.path().join(".agentflow/input").exists());

        write_build_agent_merge_proof(
            dir.path(),
            "AF-001",
            &started.run_id,
            "github",
            "auto-merge-if-eligible",
            Some("https://github.com/atxinbao/agentflow/pull/1".to_string()),
            true,
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
        assert!(fs::read_to_string(&outcome.changelog_path)
            .unwrap()
            .contains("AF-001"));
        assert!(fs::read_to_string(&outcome.release_notes_path)
            .unwrap()
            .contains("AF-001"));
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
            .any(|event| event.event_type == "issue.merge.proof.recorded"));
        assert!(events
            .iter()
            .any(|event| event.event_type == "issue.completed"));
        assert!(!dir.path().join(".agentflow/output").exists());
        assert!(!dir.path().join(".agentflow/execute").exists());
        assert!(!dir.path().join(".agentflow/input").exists());
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
        let issue = agentflow_spec::issue_from_requirement(root, &requirement, issue).unwrap();
        agentflow_spec::write_spec_issue(root, &issue).unwrap();
        let mut project = agentflow_spec::SpecProjectDraft::new(project_id);
        project.issue_ids = vec![issue_id.to_string()];
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
                "runId": run_id,
                "changedFiles": [{
                    "path": "src/lib.rs",
                    "changeType": "modified",
                    "insertions": 1,
                    "deletions": 1
                }],
                "validationCommands": [{
                    "label": "printf ok",
                    "program": "printf",
                    "args": ["ok"],
                    "exitCode": 0,
                    "stdout": "ok",
                    "stderr": "",
                    "source": "test"
                }]
            }))
            .unwrap(),
        )
        .unwrap();
        path
    }
}
