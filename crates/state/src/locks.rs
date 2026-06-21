use crate::{
    model::{StateLockSnapshot, STATE_LOCKS_VERSION},
    storage::{read_json, write_json},
};
use agentflow_workflow_runtime::load_runtime_lock_snapshot;
use anyhow::Result;
use std::path::Path;

pub(crate) fn build_lock_snapshot(root: &Path) -> Result<StateLockSnapshot> {
    let runtime = load_runtime_lock_snapshot(root)?;

    let mut active = runtime
        .active_leases
        .into_iter()
        .map(|lease| crate::model::StateLockEntry {
            kind: "issue-lease".to_string(),
            issue_id: Some(lease.issue_id.clone()),
            run_id: Some(lease.run_id.clone()),
            source_path: format!(".agentflow/events/claims/{}.json", lease.run_id),
            holder: Some(lease.owner_id.clone()),
            lease_id: Some(lease.lease_id.clone()),
            object_type: Some("Issue".to_string()),
            object_id: Some(lease.issue_id.clone()),
            expires_at: Some(lease.expires_at),
            status: Some(lease.status.as_str().to_string()),
            reason: Some(format!(
                "run {} 持有执行租约，持有者 {}",
                lease.run_id, lease.owner_id
            )),
        })
        .collect::<Vec<_>>();

    active.extend(runtime.active_object_locks.into_iter().map(|record| {
        let kind = format!("{:?}", record.lock.lock_kind).to_ascii_lowercase();
        crate::model::StateLockEntry {
            kind: format!("object-lock-{kind}"),
            issue_id: record.issue_id,
            run_id: record.run_id,
            source_path: record.source_path,
            holder: Some(record.lock.owner_role.clone()),
            lease_id: None,
            object_type: Some(record.lock.object_type.clone()),
            object_id: Some(record.lock.object_id.clone()),
            expires_at: record
                .lock
                .expires_at
                .as_deref()
                .and_then(|value| value.parse::<u64>().ok()),
            status: Some("active".to_string()),
            reason: Some(record.lock.reason.clone()),
        }
    }));

    let stale = runtime
        .stale_leases
        .into_iter()
        .map(|lease| crate::model::StateLockEntry {
            kind: "issue-lease".to_string(),
            issue_id: Some(lease.issue_id.clone()),
            run_id: Some(lease.run_id.clone()),
            source_path: format!(".agentflow/events/claims/{}.json", lease.run_id),
            holder: Some(lease.owner_id.clone()),
            lease_id: Some(lease.lease_id.clone()),
            object_type: Some("Issue".to_string()),
            object_id: Some(lease.issue_id.clone()),
            expires_at: Some(lease.expires_at),
            status: Some(lease.status.as_str().to_string()),
            reason: lease.release_reason.clone(),
        })
        .collect();

    let cleanup_candidates = runtime
        .cleanup_candidates
        .into_iter()
        .map(|lease| crate::model::StateLockEntry {
            kind: "issue-lease".to_string(),
            issue_id: Some(lease.issue_id.clone()),
            run_id: Some(lease.run_id.clone()),
            source_path: format!(".agentflow/events/claims/{}.json", lease.run_id),
            holder: Some(lease.owner_id.clone()),
            lease_id: Some(lease.lease_id.clone()),
            object_type: Some("Issue".to_string()),
            object_id: Some(lease.issue_id.clone()),
            expires_at: Some(lease.expires_at),
            status: Some(lease.status.as_str().to_string()),
            reason: lease.release_reason.clone(),
        })
        .collect();

    Ok(StateLockSnapshot {
        version: STATE_LOCKS_VERSION.to_string(),
        active,
        stale,
        cleanup_candidates,
    })
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
