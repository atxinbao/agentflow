use crate::{
    model::{
        ExecuteIndex, ExecuteManifest, ExecuteResult, ExecuteRun, ExecuteRunInput, ExecuteRunPaths,
        ExecuteRunStatus, ExecuteSnapshot, ExecuteStatusSnapshot, ExecuteSummary,
        ExecuteWorkspaceStatus, EXECUTE_SNAPSHOT_VERSION, EXECUTE_STATUS_VERSION,
    },
    storage::{
        canonical_project_root, ensure_directory, load_leases, load_runs, next_run_id, read_json,
        rebuild_index, relative_path, run_dir, unix_timestamp_seconds, update_run_status,
        write_json, write_json_if_missing, write_run, EXECUTE_DIRECTORIES, EXECUTE_REQUIRED_FILES,
    },
};
use agentflow_input::issue::InputIssue;
use anyhow::{Context, Result};
use std::path::Path;

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
    let issue: InputIssue = read_json(&issue_path)?;
    if issue.issue_id != issue_id {
        anyhow::bail!(
            "input issue id mismatch: requested {issue_id}, found {}",
            issue.issue_id
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

    let spec_path = if issue.source_spec_id.is_empty() {
        String::new()
    } else {
        format!(".agentflow/input/specs/approved/{}", issue.source_spec_id)
    };
    let now = unix_timestamp_seconds();
    let run = ExecuteRun {
        version: crate::model::EXECUTE_RUN_VERSION.to_string(),
        run_id: run_id.clone(),
        issue_id: issue.issue_id.clone(),
        source_spec_id: issue.source_spec_id.clone(),
        project_id: issue.project_id.clone(),
        risk_level: format!("{:?}", issue.risk_level).to_lowercase(),
        status: ExecuteRunStatus::Preflight,
        agent_role: "Build Agent".to_string(),
        created_by: "agent".to_string(),
        created_at: now,
        updated_at: now,
        input: ExecuteRunInput {
            issue_path: relative_path(&root, &issue_path),
            spec_path,
            panel_snapshot_id: issue.panel.snapshot_id.clone(),
            context_pack_id: issue.panel.context_pack_id.clone(),
        },
        paths: ExecuteRunPaths {
            preflight: format!(".agentflow/execute/runs/{run_id}/preflight.json"),
            plan: format!(".agentflow/execute/runs/{run_id}/plan.json"),
            result: format!(".agentflow/execute/runs/{run_id}/result.json"),
            evidence: format!(".agentflow/output/evidence/{run_id}.json"),
        },
    };
    write_run(&root, &run)?;
    rebuild_index(&root)?;
    build_execute_snapshot(&root)?;
    Ok(run)
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
