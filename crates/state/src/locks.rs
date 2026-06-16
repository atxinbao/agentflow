use crate::{
    model::{StateLockSnapshot, STATE_LOCKS_VERSION},
    storage::{read_json, write_json},
};
use anyhow::Result;
use std::path::Path;

pub(crate) fn build_lock_snapshot(_root: &Path) -> Result<StateLockSnapshot> {
    Ok(StateLockSnapshot {
        version: STATE_LOCKS_VERSION.to_string(),
        active: Vec::new(),
        stale: Vec::new(),
        cleanup_candidates: Vec::new(),
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
