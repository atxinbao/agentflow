use crate::{
    records::RuntimeAcceptedActionFact,
    storage::{load_runtime_accepted_action_facts, runtime_accepted_action_fact_path},
};
use agentflow_action_arbitration::ObjectLock;
use agentflow_event_store::{load_task_claim_leases, TaskEventClaimLease, TaskEventClaimStatus};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeObjectLockRecord {
    pub issue_id: Option<String>,
    pub run_id: Option<String>,
    pub source_path: String,
    pub lock: ObjectLock,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeLockSnapshot {
    pub active_leases: Vec<TaskEventClaimLease>,
    pub stale_leases: Vec<TaskEventClaimLease>,
    pub cleanup_candidates: Vec<TaskEventClaimLease>,
    pub active_object_locks: Vec<RuntimeObjectLockRecord>,
}

pub fn load_runtime_lock_snapshot(project_root: impl AsRef<Path>) -> Result<RuntimeLockSnapshot> {
    let root = project_root.as_ref();
    let mut active_leases = Vec::new();
    let mut stale_leases = Vec::new();
    let mut cleanup_candidates = Vec::new();
    let mut active_run_ids = BTreeSet::new();

    for lease in load_task_claim_leases(root)? {
        match lease.status {
            TaskEventClaimStatus::Active => {
                active_run_ids.insert(lease.run_id.clone());
                active_leases.push(lease);
            }
            TaskEventClaimStatus::Expired => {
                cleanup_candidates.push(lease.clone());
                stale_leases.push(lease);
            }
            TaskEventClaimStatus::Released => {}
        }
    }

    let mut active_object_locks = collect_active_object_locks(root, &active_run_ids)?;
    active_object_locks.sort_by(|left, right| {
        left.lock
            .object_type
            .cmp(&right.lock.object_type)
            .then_with(|| left.lock.object_id.cmp(&right.lock.object_id))
            .then_with(|| left.lock.lock_id.cmp(&right.lock.lock_id))
    });
    active_leases.sort_by(|left, right| left.run_id.cmp(&right.run_id));
    stale_leases.sort_by(|left, right| left.run_id.cmp(&right.run_id));
    cleanup_candidates.sort_by(|left, right| left.run_id.cmp(&right.run_id));

    Ok(RuntimeLockSnapshot {
        active_leases,
        stale_leases,
        cleanup_candidates,
        active_object_locks,
    })
}

fn collect_active_object_locks(
    root: &Path,
    active_run_ids: &BTreeSet<String>,
) -> Result<Vec<RuntimeObjectLockRecord>> {
    let mut facts = load_runtime_accepted_action_facts(root)?;
    facts.sort_by(|left, right| {
        left.recorded_at
            .cmp(&right.recorded_at)
            .then_with(|| left.accepted_action_id.cmp(&right.accepted_action_id))
    });

    let mut active_locks = BTreeMap::new();
    for fact in facts {
        let Some(run_id) = fact.run_id.clone() else {
            continue;
        };
        if !active_run_ids.contains(&run_id) {
            continue;
        }
        for release_id in &fact.lock_plan.release {
            active_locks.remove(release_id);
        }
        for lock in &fact.lock_plan.acquire {
            active_locks.insert(
                lock.lock_id.clone(),
                runtime_object_lock_record(root, &fact, lock.clone()),
            );
        }
    }

    Ok(active_locks.into_values().collect())
}

fn runtime_object_lock_record(
    root: &Path,
    fact: &RuntimeAcceptedActionFact,
    lock: ObjectLock,
) -> RuntimeObjectLockRecord {
    RuntimeObjectLockRecord {
        issue_id: fact.issue_id.clone(),
        run_id: fact.run_id.clone(),
        source_path: runtime_accepted_action_fact_path(root, &fact.accepted_action_id)
            .display()
            .to_string(),
        lock,
    }
}
