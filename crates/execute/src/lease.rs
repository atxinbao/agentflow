use crate::{
    manager::load_issue_for_run,
    model::{ExecuteLease, ExecuteLeaseStatus, ExecuteRunStatus, EXECUTE_LEASE_VERSION},
    storage::{
        canonical_project_root, read_json, read_run, rebuild_index, update_run_status, write_json,
    },
};
use anyhow::Result;
use std::path::Path;

pub fn acquire_execute_lease(
    project_root: impl AsRef<Path>,
    run_id: String,
) -> Result<ExecuteLease> {
    let root = canonical_project_root(project_root)?;
    let run = read_run(&root, &run_id)?;
    let issue = load_issue_for_run(&root, &run)?;
    let lease_path = root
        .join(".agentflow/execute/leases")
        .join(format!("{}.json", issue.issue_id));

    if lease_path.is_file() {
        let lease: ExecuteLease = read_json(&lease_path)?;
        if matches!(lease.status, ExecuteLeaseStatus::Active) {
            anyhow::bail!("issue {} already has an active lease", issue.issue_id);
        }
    }

    let lease = ExecuteLease {
        version: EXECUTE_LEASE_VERSION.to_string(),
        issue_id: issue.issue_id,
        run_id: run.run_id.clone(),
        status: ExecuteLeaseStatus::Active,
        created_at: crate::storage::unix_timestamp_seconds(),
        released_at: None,
        expires_at: None,
        locked_files: Vec::new(),
    };
    write_json(&lease_path, &lease)?;
    rebuild_index(&root)?;
    Ok(lease)
}

pub fn release_execute_lease(
    project_root: impl AsRef<Path>,
    run_id: String,
) -> Result<ExecuteLease> {
    let root = canonical_project_root(project_root)?;
    let run = read_run(&root, &run_id)?;
    let lease_path = root
        .join(".agentflow/execute/leases")
        .join(format!("{}.json", run.issue_id));
    if !lease_path.is_file() {
        anyhow::bail!("run {} has no active lease file", run_id);
    }
    let mut lease: ExecuteLease = read_json(&lease_path)?;
    if lease.run_id != run_id {
        anyhow::bail!("lease belongs to run {}, not {}", lease.run_id, run_id);
    }
    lease.status = ExecuteLeaseStatus::Released;
    lease.released_at = Some(crate::storage::unix_timestamp_seconds());
    write_json(&lease_path, &lease)?;
    rebuild_index(&root)?;
    Ok(lease)
}

pub(crate) fn has_active_lease_for_run(root: &Path, run_id: &str) -> Result<bool> {
    let run = read_run(root, run_id)?;
    let lease_path = root
        .join(".agentflow/execute/leases")
        .join(format!("{}.json", run.issue_id));
    if !lease_path.is_file() {
        return Ok(false);
    }
    let lease: ExecuteLease = read_json(&lease_path)?;
    Ok(lease.run_id == run_id && matches!(lease.status, ExecuteLeaseStatus::Active))
}

pub(crate) fn finalize_run_and_release(
    root: &Path,
    run_id: &str,
    status: ExecuteRunStatus,
) -> Result<()> {
    update_run_status(root, run_id, status)?;
    release_execute_lease(root, run_id.to_string()).ok();
    rebuild_index(root)?;
    Ok(())
}
