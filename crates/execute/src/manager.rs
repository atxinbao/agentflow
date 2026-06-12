use crate::{
    model::{
        ExecuteBranchCheck, ExecuteIndex, ExecuteManifest, ExecuteResult, ExecuteRun,
        ExecuteRunInput, ExecuteRunPaths, ExecuteRunStatus, ExecuteSnapshot, ExecuteStatusSnapshot,
        ExecuteSummary, ExecuteWorkspaceStatus, EXECUTE_SNAPSHOT_VERSION, EXECUTE_STATUS_VERSION,
    },
    storage::{
        canonical_project_root, ensure_directory, load_leases, load_runs, next_run_id, read_json,
        rebuild_index, relative_path, run_dir, unix_timestamp_seconds, update_run_status,
        write_json, write_json_if_missing, write_run, EXECUTE_DIRECTORIES, EXECUTE_REQUIRED_FILES,
    },
};
use agentflow_input::issue::{
    validate_agent_claim, validate_agent_issue_permission, validate_agent_write_paths, AgentClaim,
    AgentRole, AgentRolesDocument, InputIssue, InputIssueStatus,
};
use anyhow::{Context, Result};
use std::{path::Path, process::Command};

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct IssueLoopProjectionBlocker {
    pub code: String,
    pub reason: String,
    pub source_path: Option<String>,
}

pub fn prepare_execute_workspace(project_root: impl AsRef<Path>) -> Result<ExecuteSnapshot> {
    let root = canonical_project_root(project_root)?;
    let ownership = agentflow_agent_manual::assert_agentflow_workspace_owned_or_creatable(&root)?;
    if matches!(
        ownership.status,
        agentflow_agent_manual::model::WorkspaceOwnershipState::None
    ) {
        agentflow_agent_manual::prepare_agent_working_manual(&root)?;
    }

    for relative_path in EXECUTE_DIRECTORIES {
        ensure_directory(&root.join(relative_path))?;
    }
    agentflow_output::prepare_output_workspace(&root)?;

    write_json_if_missing(
        &root.join(".agentflow/execute/queue/pending.json"),
        &Vec::<String>::new(),
    )?;
    write_json_if_missing(
        &root.join(".agentflow/execute/queue/active.json"),
        &Vec::<String>::new(),
    )?;
    write_json_if_missing(
        &root.join(".agentflow/execute/queue/blocked.json"),
        &Vec::<String>::new(),
    )?;

    let summary = load_summary(&root)?;
    let manifest = ExecuteManifest::new(root.display().to_string(), summary);
    write_json(&root.join(".agentflow/execute/manifest.json"), &manifest)?;
    rebuild_index(&root)?;
    build_execute_snapshot(&root)
}

pub fn validate_execute_workspace(project_root: impl AsRef<Path>) -> Result<ExecuteSnapshot> {
    let root = canonical_project_root(project_root)?;
    build_execute_snapshot(&root)
}

pub fn load_execute_status(project_root: impl AsRef<Path>) -> Result<ExecuteStatusSnapshot> {
    Ok(validate_execute_workspace(project_root)?.status)
}

pub fn load_execute_manifest(project_root: impl AsRef<Path>) -> Result<ExecuteManifest> {
    let root = canonical_project_root(project_root)?;
    read_json(&root.join(".agentflow/execute/manifest.json"))
}

pub fn load_execute_index(project_root: impl AsRef<Path>) -> Result<ExecuteIndex> {
    let root = canonical_project_root(project_root)?;
    read_json(&root.join(".agentflow/execute/index.json"))
}

pub fn load_execute_snapshot(project_root: impl AsRef<Path>) -> Result<ExecuteSnapshot> {
    validate_execute_workspace(project_root)
}

pub fn load_execute_run(project_root: impl AsRef<Path>, run_id: String) -> Result<ExecuteRun> {
    let root = canonical_project_root(project_root)?;
    read_json(&run_dir(&root, &run_id).join("run.json"))
}

pub fn load_execute_result(
    project_root: impl AsRef<Path>,
    run_id: String,
) -> Result<ExecuteResult> {
    let root = canonical_project_root(project_root)?;
    read_json(&run_dir(&root, &run_id).join("result.json"))
}

pub fn create_execute_run(project_root: impl AsRef<Path>, issue_id: String) -> Result<ExecuteRun> {
    let snapshot = prepare_execute_workspace(project_root.as_ref())?;
    if !snapshot.ready {
        anyhow::bail!(
            "execute workspace is not ready: {:?}",
            snapshot.status.errors
        );
    }
    let root = canonical_project_root(project_root)?;
    let issue_path = root
        .join(".agentflow/input/issues")
        .join(format!("{issue_id}.json"));
    if !issue_path.is_file() {
        anyhow::bail!("input issue {issue_id} does not exist");
    }
    let mut issue: InputIssue = read_json(&issue_path)?;
    issue.normalize_execution_metadata();
    if issue.issue_id != issue_id {
        anyhow::bail!(
            "input issue id mismatch: requested {issue_id}, found {}",
            issue.issue_id
        );
    }
    validate_agent_issue_permission(&issue, &AgentRole::BuildAgent)?;
    if !matches!(issue.status, InputIssueStatus::Todo) {
        anyhow::bail!(
            "input issue {} must be todo before Build Agent runtime preflight",
            issue.issue_id
        );
    }
    if let Some(active_run) = load_runs(&root)?
        .into_iter()
        .find(|run| run.issue_id == issue.issue_id && execute_run_active(&run.status))
    {
        anyhow::bail!(
            "input issue {} already has active execute run {}",
            issue.issue_id,
            active_run.run_id
        );
    }

    let run_id = next_run_id(&root)?;
    let run_path = run_dir(&root, &run_id);
    for relative in [
        "",
        "commands",
        "confirmations",
        "checkpoints",
        "patches",
        "review",
    ] {
        ensure_directory(&run_path.join(relative))?;
    }

    let spec_path = issue.source_spec_path.clone();
    let now = unix_timestamp_seconds();
    let run = ExecuteRun {
        version: crate::model::EXECUTE_RUN_VERSION.to_string(),
        run_id: run_id.clone(),
        issue_id: issue.issue_id.clone(),
        source_spec_id: issue.source_spec_id.clone(),
        project_id: issue.project_id.clone(),
        risk_level: format!("{:?}", issue.execution_risk).to_lowercase(),
        status: ExecuteRunStatus::Preflight,
        agent_role: issue.required_agent_role.as_str().to_string(),
        created_by: AgentRole::BuildAgent.as_str().to_string(),
        created_at: now,
        updated_at: now,
        input: ExecuteRunInput {
            issue_path: relative_path(&root, &issue_path),
            spec_path,
            panel_snapshot_id: issue.panel.snapshot_id.clone(),
            context_pack_id: issue.panel.context_pack_id.clone(),
            context_pack_path: (!issue.context_pack_path.trim().is_empty())
                .then(|| issue.context_pack_path.clone()),
        },
        paths: ExecuteRunPaths {
            preflight: format!(".agentflow/execute/runs/{run_id}/preflight.json"),
            plan: format!(".agentflow/execute/runs/{run_id}/plan.json"),
            result: format!(".agentflow/execute/runs/{run_id}/result.json"),
            evidence: format!(".agentflow/output/evidence/{run_id}.json"),
        },
    };
    write_run(&root, &run)?;
    write_json(
        &run_path.join("agent-claim.json"),
        &AgentClaim::new(
            &issue,
            AgentRole::BuildAgent,
            format!("handoff-{}", issue.issue_id),
        ),
    )?;
    let branch_check = write_branch_check(&root, &run, &issue)?;
    if branch_check.status == "blocked" {
        update_run_status(&root, &run_id, ExecuteRunStatus::Blocked)?;
        update_input_issue_status(&root, &issue.issue_id, InputIssueStatus::Blocked)?;
        sync_issue_loop_projection(
            &root,
            &run,
            InputIssueStatus::Blocked,
            None,
            vec![IssueLoopProjectionBlocker {
                code: "branch-check-blocked".to_string(),
                reason: branch_check
                    .blocked_reason
                    .clone()
                    .unwrap_or_else(|| "Issue branch check blocked.".to_string()),
                source_path: Some(format!(".agentflow/execute/runs/{run_id}/branch.json")),
            }],
        )?;
        rebuild_index(&root)?;
        anyhow::bail!(
            "issue branch check blocked {}: {}",
            issue.issue_id,
            branch_check
                .blocked_reason
                .unwrap_or_else(|| "branch check failed".to_string())
        );
    }
    sync_issue_loop_projection(&root, &run, InputIssueStatus::Todo, None, Vec::new())?;
    rebuild_index(&root)?;
    build_execute_snapshot(&root)?;
    Ok(run)
}

fn execute_run_active(status: &ExecuteRunStatus) -> bool {
    matches!(
        status,
        ExecuteRunStatus::Queued
            | ExecuteRunStatus::Preflight
            | ExecuteRunStatus::Planned
            | ExecuteRunStatus::Checkpointed
            | ExecuteRunStatus::Patching
            | ExecuteRunStatus::Running
            | ExecuteRunStatus::Validating
    )
}

pub fn cancel_execute_run(project_root: impl AsRef<Path>, run_id: String) -> Result<ExecuteRun> {
    let root = canonical_project_root(project_root)?;
    crate::lease::release_execute_lease(&root, run_id.clone()).ok();
    let run = update_run_status(&root, &run_id, ExecuteRunStatus::Cancelled)?;
    rebuild_index(&root)?;
    Ok(run)
}

pub(crate) fn build_execute_snapshot(root: &Path) -> Result<ExecuteSnapshot> {
    let manifest_exists = root.join(".agentflow/execute/manifest.json").is_file();
    let index_exists = root.join(".agentflow/execute/index.json").is_file();
    let missing_paths = missing_execute_paths(root);
    let summary = load_summary(root)?;
    let mut warnings = Vec::new();
    let mut errors = Vec::new();

    if let Err(error) = agentflow_agent_manual::assert_agentflow_workspace_owned_or_creatable(root)
    {
        errors.push(format!("ownership check failed: {error}"));
    }

    if !missing_paths.is_empty() {
        errors.push(format!(
            "missing execute paths: {}",
            missing_paths.join(", ")
        ));
    }

    let ready = errors.is_empty();
    if manifest_exists && index_exists && !ready {
        warnings.push("execute facts exist but validation gates are not ready".to_string());
    }
    let status = status(
        root,
        ready,
        manifest_exists,
        index_exists,
        summary.clone(),
        missing_paths,
        warnings,
        errors,
    );
    let manifest = if manifest_exists {
        read_json(&root.join(".agentflow/execute/manifest.json"))
            .unwrap_or_else(|_| ExecuteManifest::new(root.display().to_string(), summary.clone()))
    } else {
        ExecuteManifest::new(root.display().to_string(), summary.clone())
    };
    let index = if index_exists {
        read_json(&root.join(".agentflow/execute/index.json")).unwrap_or_default()
    } else {
        ExecuteIndex::default()
    };

    Ok(ExecuteSnapshot {
        version: EXECUTE_SNAPSHOT_VERSION.to_string(),
        project_root: root.display().to_string(),
        ready,
        status,
        manifest,
        index,
    })
}

pub(crate) fn load_summary(root: &Path) -> Result<ExecuteSummary> {
    let runs = load_runs(root)?;
    let leases = load_leases(root)?;
    Ok(ExecuteSummary {
        runs: runs.len(),
        active_runs: runs
            .iter()
            .filter(|run| {
                matches!(
                    run.status,
                    ExecuteRunStatus::Queued
                        | ExecuteRunStatus::Preflight
                        | ExecuteRunStatus::Planned
                        | ExecuteRunStatus::Checkpointed
                        | ExecuteRunStatus::Patching
                        | ExecuteRunStatus::Running
                        | ExecuteRunStatus::Validating
                )
            })
            .count(),
        blocked_runs: runs
            .iter()
            .filter(|run| matches!(run.status, ExecuteRunStatus::Blocked))
            .count(),
        completed_runs: runs
            .iter()
            .filter(|run| matches!(run.status, ExecuteRunStatus::Completed))
            .count(),
        active_leases: leases
            .iter()
            .filter(|lease| matches!(lease.status, crate::model::ExecuteLeaseStatus::Active))
            .count(),
    })
}

pub(crate) fn update_input_issue_status(
    root: &Path,
    issue_id: &str,
    status: InputIssueStatus,
) -> Result<InputIssue> {
    agentflow_input::update_input_issue_status(root, issue_id, status)
}

pub(crate) fn sync_issue_loop_projection(
    root: &Path,
    run: &ExecuteRun,
    stage: InputIssueStatus,
    review_substate: Option<String>,
    blockers: Vec<IssueLoopProjectionBlocker>,
) -> Result<()> {
    let issue_projection_dir = root.join(".agentflow/state/loops/issues");
    ensure_directory(&issue_projection_dir)?;
    write_json(
        &issue_projection_dir.join(format!("{}.json", sanitize_projection_id(&run.issue_id))),
        &serde_json::json!({
            "version": "agentflow-loop-issue.v1",
            "projectId": run.project_id.clone(),
            "issueId": run.issue_id.clone(),
            "stage": stage.as_str(),
            "runId": run.run_id.clone(),
            "branchName": issue_loop_branch_name(root, &run.run_id),
            "reviewSubstate": review_substate,
            "blockers": blockers,
            "updatedAt": unix_timestamp_seconds()
        }),
    )
}

pub(crate) fn issue_loop_branch_name(root: &Path, run_id: &str) -> Option<String> {
    read_json::<serde_json::Value>(&run_dir(root, run_id).join("branch.json"))
        .ok()
        .and_then(|value| {
            value
                .get("issueBranch")
                .and_then(serde_json::Value::as_str)
                .map(str::to_string)
        })
}

pub(crate) fn sanitize_projection_id(id: &str) -> String {
    id.chars()
        .map(|ch| match ch {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => ch,
        })
        .collect()
}

pub(crate) fn missing_execute_paths(root: &Path) -> Vec<String> {
    EXECUTE_DIRECTORIES
        .iter()
        .copied()
        .chain(EXECUTE_REQUIRED_FILES.iter().copied())
        .filter(|relative_path| !root.join(relative_path).exists())
        .map(str::to_string)
        .collect()
}

pub(crate) fn status(
    root: &Path,
    ready: bool,
    manifest_exists: bool,
    index_exists: bool,
    summary: ExecuteSummary,
    missing_paths: Vec<String>,
    warnings: Vec<String>,
    errors: Vec<String>,
) -> ExecuteStatusSnapshot {
    let status = if errors.iter().any(|error| error.contains("ownership")) {
        ExecuteWorkspaceStatus::Blocked
    } else if !errors.is_empty() {
        ExecuteWorkspaceStatus::Failed
    } else if !missing_paths.is_empty() {
        ExecuteWorkspaceStatus::Missing
    } else if !warnings.is_empty() {
        ExecuteWorkspaceStatus::Degraded
    } else {
        ExecuteWorkspaceStatus::Ready
    };
    ExecuteStatusSnapshot {
        version: EXECUTE_STATUS_VERSION.to_string(),
        project_root: root.display().to_string(),
        status,
        ready,
        manifest_exists,
        index_exists,
        summary,
        missing_paths,
        warnings,
        errors,
    }
}

pub(crate) fn load_issue_for_run(root: &Path, run: &ExecuteRun) -> Result<InputIssue> {
    read_json(&root.join(&run.input.issue_path))
        .with_context(|| format!("load input issue {} for run {}", run.issue_id, run.run_id))
}

pub(crate) fn assert_build_agent_run(root: &Path, run: &ExecuteRun) -> Result<InputIssue> {
    let issue = load_issue_for_run(root, run)?;
    let claim_path = run_dir(root, &run.run_id).join("agent-claim.json");
    let claim: AgentClaim = read_json(&claim_path)
        .with_context(|| format!("load agent claim for run {}", run.run_id))?;
    validate_agent_claim(&issue, &claim)?;
    validate_agent_write_paths(
        &AgentRole::BuildAgent,
        &[
            format!(".agentflow/execute/runs/{}", run.run_id),
            format!(".agentflow/output/evidence/{}.json", run.run_id),
            format!(".agentflow/output/release/{}", run.run_id),
        ],
        &AgentRolesDocument::default(),
    )?;
    Ok(issue)
}

fn write_branch_check(
    root: &Path,
    run: &ExecuteRun,
    issue: &InputIssue,
) -> Result<ExecuteBranchCheck> {
    let before = current_git_branch(root).unwrap_or_else(|| "not-git".to_string());
    let base_branch = default_base_branch(root).unwrap_or_else(|| "main".to_string());
    let project_id = issue
        .project_id
        .clone()
        .unwrap_or_else(|| "direct".to_string());
    let issue_branch = format!(
        "agentflow/{}/{}",
        sanitize_branch_segment(&project_id),
        sanitize_branch_segment(&issue.issue_id)
    );

    let (after, status, blocked_reason) = if before == "not-git" {
        (before.clone(), "skipped-not-git".to_string(), None)
    } else if before == issue_branch {
        (before.clone(), "ready".to_string(), None)
    } else if git_worktree_dirty(root) {
        (
            before.clone(),
            "blocked".to_string(),
            Some(
                "current branch does not match issue branch and worktree has uncommitted changes"
                    .to_string(),
            ),
        )
    } else {
        match Command::new("git")
            .args(["switch", "-C", &issue_branch])
            .current_dir(root)
            .output()
        {
            Ok(output) if output.status.success() => (
                current_git_branch(root).unwrap_or_else(|| issue_branch.clone()),
                "ready".to_string(),
                None,
            ),
            Ok(output) => (
                before.clone(),
                "blocked".to_string(),
                Some(format!(
                    "git switch failed: {}{}",
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                )),
            ),
            Err(error) => (
                before.clone(),
                "blocked".to_string(),
                Some(format!("git switch failed: {error}")),
            ),
        }
    };

    let check = ExecuteBranchCheck {
        version: "execute-branch-check.v1".to_string(),
        run_id: run.run_id.clone(),
        issue_id: issue.issue_id.clone(),
        project_id: issue.project_id.clone(),
        base_branch,
        issue_branch,
        current_branch_before: before,
        current_branch_after: after,
        status,
        blocked_reason,
    };
    write_json(&run_dir(root, &run.run_id).join("branch.json"), &check)?;
    Ok(check)
}

fn current_git_branch(root: &Path) -> Option<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(root)
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|branch| !branch.is_empty())
}

fn default_base_branch(root: &Path) -> Option<String> {
    let output = Command::new("git")
        .args(["symbolic-ref", "--short", "refs/remotes/origin/HEAD"])
        .current_dir(root)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8_lossy(&output.stdout)
        .trim()
        .strip_prefix("origin/")
        .map(str::to_string)
}

fn git_worktree_dirty(root: &Path) -> bool {
    Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(root)
        .output()
        .map(|output| output.status.success() && !output.stdout.is_empty())
        .unwrap_or(false)
}

fn sanitize_branch_segment(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
                ch
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}
