use crate::{
    model::{StateLockEntry, StateLockSnapshot, STATE_LOCKS_VERSION},
    storage::{read_json, sorted_child_paths, write_json},
};
use agentflow_execute::{ExecuteLease, ExecuteLeaseStatus, ExecuteRun, ExecuteRunStatus};
use anyhow::Result;
use std::path::Path;

pub(crate) fn build_lock_snapshot(root: &Path) -> Result<StateLockSnapshot> {
    let mut snapshot = StateLockSnapshot {
        version: STATE_LOCKS_VERSION.to_string(),
        ..StateLockSnapshot::default()
    };
    for path in sorted_child_paths(&root.join(".agentflow/execute/leases"))? {
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let source_path = format!(
            ".agentflow/execute/leases/{}",
            path.file_name()
                .and_then(|value| value.to_str())
                .unwrap_or_default()
        );
        let lease = match read_json::<ExecuteLease>(&path) {
            Ok(lease) => lease,
            Err(error) => {
                snapshot.cleanup_candidates.push(StateLockEntry {
                    kind: "execute-lease".to_string(),
                    issue_id: None,
                    run_id: None,
                    source_path,
                    reason: Some(format!("lease state unreadable: {error}")),
                });
                continue;
            }
        };
        if !matches!(lease.status, ExecuteLeaseStatus::Active) {
            continue;
        }
        let run = read_json::<ExecuteRun>(
            &root
                .join(".agentflow/execute/runs")
                .join(&lease.run_id)
                .join("run.json"),
        );
        let entry = StateLockEntry {
            kind: "execute-lease".to_string(),
            issue_id: Some(lease.issue_id.clone()),
            run_id: Some(lease.run_id.clone()),
            source_path,
            reason: None,
        };
        match run {
            Ok(run) if run_is_terminal(&run.status) => snapshot.stale.push(StateLockEntry {
                reason: Some(format!("run {} is terminal", run.run_id)),
                ..entry
            }),
            Ok(_) => snapshot.active.push(entry),
            Err(error) => snapshot.stale.push(StateLockEntry {
                reason: Some(format!("run missing or unreadable: {error}")),
                ..entry
            }),
        }
    }
    Ok(snapshot)
}

pub(crate) fn write_lock_snapshot(root: &Path, snapshot: &StateLockSnapshot) -> Result<()> {
    let locks_dir = root.join(".agentflow/state/locks");
    write_json(
        &locks_dir.join("active.json"),
        &StateLockSnapshot {
            version: snapshot.version.clone(),
            active: snapshot.active.clone(),
            stale: Vec::new(),
            cleanup_candidates: Vec::new(),
        },
    )?;
    write_json(
        &locks_dir.join("stale.json"),
        &StateLockSnapshot {
            version: snapshot.version.clone(),
            active: Vec::new(),
            stale: snapshot.stale.clone(),
            cleanup_candidates: Vec::new(),
        },
    )?;
    write_json(
        &locks_dir.join("cleanup-candidates.json"),
        &StateLockSnapshot {
            version: snapshot.version.clone(),
            active: Vec::new(),
            stale: Vec::new(),
            cleanup_candidates: snapshot.cleanup_candidates.clone(),
        },
    )
}

pub fn load_state_locks(project_root: impl AsRef<Path>) -> Result<StateLockSnapshot> {
    let root = crate::storage::canonical_project_root(project_root)?;
    let active = read_json::<StateLockSnapshot>(&root.join(".agentflow/state/locks/active.json"))
        .map(|snapshot| snapshot.active)
        .unwrap_or_default();
    let stale = read_json::<StateLockSnapshot>(&root.join(".agentflow/state/locks/stale.json"))
        .map(|snapshot| snapshot.stale)
        .unwrap_or_default();
    let cleanup_candidates = read_json::<StateLockSnapshot>(
        &root.join(".agentflow/state/locks/cleanup-candidates.json"),
    )
    .map(|snapshot| snapshot.cleanup_candidates)
    .unwrap_or_default();
    Ok(StateLockSnapshot {
        version: STATE_LOCKS_VERSION.to_string(),
        active,
        stale,
        cleanup_candidates,
    })
}

fn run_is_terminal(status: &ExecuteRunStatus) -> bool {
    matches!(
        status,
        ExecuteRunStatus::Completed | ExecuteRunStatus::Failed | ExecuteRunStatus::Cancelled
    )
}
