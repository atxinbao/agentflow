use crate::{
    events::{append_state_event, load_state_timeline},
    gates::{build_gate_snapshot, write_gate_files},
    health::{collect_health, health_ready, health_status_map, write_health},
    indexes::write_indexes,
    locks::{build_lock_snapshot, write_lock_snapshot},
    model::{
        IssueStatusIndex, StateEventIndex, StateIndex, StateIndexEntry, StateManifest,
        StateManifestSummary, StateStatusSnapshot, StateTimelineEventDraft, StateWorkspaceStatus,
        WorkflowBlockedAction, WorkflowHealthSnapshot, STATE_INDEX_VERSION, STATE_MANIFEST_VERSION,
        STATE_STATUS_VERSION,
    },
    sessions::load_sessions,
    storage::{
        canonical_project_root, ensure_directory, touch_file, unix_timestamp_seconds, write_json,
        STATE_DIRECTORIES, STATE_REQUIRED_FILES,
    },
};
use anyhow::Result;
use std::{collections::BTreeMap, path::Path};

pub fn prepare_state_workspace(project_root: impl AsRef<Path>) -> Result<StateStatusSnapshot> {
    let root = canonical_project_root(project_root)?;
    for relative_path in STATE_DIRECTORIES {
        ensure_directory(&root.join(relative_path))?;
    }
    touch_file(&root.join(".agentflow/state/events/timeline.jsonl"))?;
    let status = refresh_state(&root)?;
    ensure_required_files(&root)?;
    Ok(status)
}

pub fn refresh_state(project_root: impl AsRef<Path>) -> Result<StateStatusSnapshot> {
    let root = canonical_project_root(project_root)?;
    for relative_path in STATE_DIRECTORIES {
        ensure_directory(&root.join(relative_path))?;
    }
    touch_file(&root.join(".agentflow/state/events/timeline.jsonl"))?;

    let health = collect_health(&root);
    write_health(&root, &health)?;
    let gate = build_gate_snapshot(&root, &health)?;
    write_gate_files(&root, &gate)?;
    write_role_mismatch_events(&root, &gate.blocked_actions)?;
    let locks = build_lock_snapshot(&root)?;
    write_lock_snapshot(&root, &locks)?;
    write_indexes(&root, &health, &gate)?;
    let sessions = load_sessions(&root)?;
    let manifest = build_manifest(&root, &health, &gate, sessions.len(), &locks)?;
    write_json(&root.join(".agentflow/state/manifest.json"), &manifest)?;
    let index = build_state_index(&root, &health, &sessions)?;
    write_json(&root.join(".agentflow/state/index.json"), &index)?;

    let status = StateStatusSnapshot {
        version: STATE_STATUS_VERSION.to_string(),
        project_root: root.display().to_string(),
        status: manifest.status,
        current_stage: gate.current_stage,
        audit_status: gate.audit_status,
        active_issue_id: gate.active_issue_id,
        active_run_id: gate.active_run_id,
        health: health_status_map(&health),
        next_actions: gate.allowed_next_actions,
        blockers: gate.blocked_actions,
        updated_at: manifest.updated_at,
    };
    write_json(&root.join(".agentflow/state/status.json"), &status)?;
    ensure_required_files(&root)?;
    Ok(status)
}

pub fn load_state_status(project_root: impl AsRef<Path>) -> Result<StateStatusSnapshot> {
    refresh_state(project_root)
}

pub fn load_state_manifest(project_root: impl AsRef<Path>) -> Result<StateManifest> {
    let root = canonical_project_root(project_root)?;
    crate::storage::read_json(&root.join(".agentflow/state/manifest.json"))
}

pub fn load_state_index(project_root: impl AsRef<Path>) -> Result<StateIndex> {
    let root = canonical_project_root(project_root)?;
    crate::storage::read_json(&root.join(".agentflow/state/index.json"))
}

pub fn load_issue_status_index(project_root: impl AsRef<Path>) -> Result<IssueStatusIndex> {
    let root = canonical_project_root(project_root)?;
    refresh_state(&root)?;
    crate::storage::read_json(&root.join(".agentflow/state/indexes/issue-status.json"))
}

fn write_role_mismatch_events(root: &Path, blockers: &[WorkflowBlockedAction]) -> Result<()> {
    let existing = load_state_timeline(root).unwrap_or_default();
    for blocker in blockers
        .iter()
        .filter(|blocker| blocker.action == "agent-role-mismatch")
    {
        let source_path = blocker.source_path.clone().unwrap_or_default();
        let already_recorded = existing.iter().any(|event| {
            event.event == "agent.role_mismatch"
                && event.details.get("sourcePath") == Some(&source_path)
        });
        if already_recorded {
            continue;
        }
        let mut details = BTreeMap::new();
        details.insert("reason".to_string(), blocker.reason.clone());
        details.insert("sourcePath".to_string(), source_path);
        append_state_event(
            root,
            StateTimelineEventDraft {
                event: "agent.role_mismatch".to_string(),
                details,
            },
        )?;
    }
    Ok(())
}

fn build_manifest(
    root: &Path,
    health: &[WorkflowHealthSnapshot],
    gate: &crate::model::WorkflowGateSnapshot,
    active_sessions: usize,
    locks: &crate::model::StateLockSnapshot,
) -> Result<StateManifest> {
    let status = state_status(health, gate);
    Ok(StateManifest {
        version: STATE_MANIFEST_VERSION.to_string(),
        project_root: root.display().to_string(),
        status,
        updated_at: unix_timestamp_seconds(),
        paths: crate::state_paths(),
        summary: StateManifestSummary {
            health_ready: health_ready(health),
            current_stage: gate.current_stage.as_str().to_string(),
            allowed_next_actions: gate.allowed_next_actions.len(),
            blocked_actions: gate.blocked_actions.len(),
            active_sessions,
            active_locks: locks.active.len(),
            stale_locks: locks.stale.len(),
            audit_status: gate.audit_status.as_str().to_string(),
        },
    })
}

fn state_status(
    health: &[WorkflowHealthSnapshot],
    gate: &crate::model::WorkflowGateSnapshot,
) -> StateWorkspaceStatus {
    if health.iter().any(|item| item.status == "blocked") {
        return StateWorkspaceStatus::Blocked;
    }
    if !gate.blocked_actions.is_empty() {
        return StateWorkspaceStatus::Blocked;
    }
    if health.iter().any(|item| item.status == "failed") {
        return StateWorkspaceStatus::Failed;
    }
    if health.iter().any(|item| item.status == "missing") {
        return StateWorkspaceStatus::Missing;
    }
    if health.iter().any(|item| item.status == "degraded") {
        return StateWorkspaceStatus::Degraded;
    }
    StateWorkspaceStatus::Ready
}

fn build_state_index(
    root: &Path,
    health: &[WorkflowHealthSnapshot],
    sessions: &[crate::model::StateSession],
) -> Result<StateIndex> {
    let health_entries = health
        .iter()
        .map(|item| StateIndexEntry {
            id: item.module.clone(),
            status: item.status.clone(),
            path: format!(".agentflow/state/health/{}.json", item.module),
        })
        .collect();
    let session_entries = sessions
        .iter()
        .map(|session| StateIndexEntry {
            id: session.session_id.clone(),
            status: session.status.clone(),
            path: format!(".agentflow/state/sessions/{}.json", session.session_id),
        })
        .collect();
    let locks = ["active", "stale", "cleanup-candidates"]
        .into_iter()
        .map(|id| StateIndexEntry {
            id: id.to_string(),
            status: if root
                .join(".agentflow/state/locks")
                .join(format!("{id}.json"))
                .is_file()
            {
                "ready".to_string()
            } else {
                "missing".to_string()
            },
            path: format!(".agentflow/state/locks/{id}.json"),
        })
        .collect();
    Ok(StateIndex {
        version: STATE_INDEX_VERSION.to_string(),
        updated_at: unix_timestamp_seconds(),
        health: health_entries,
        sessions: session_entries,
        locks,
        events: StateEventIndex {
            timeline: ".agentflow/state/events/timeline.jsonl".to_string(),
        },
    })
}

fn ensure_required_files(root: &Path) -> Result<()> {
    for relative_path in STATE_REQUIRED_FILES {
        if !root.join(relative_path).is_file() {
            anyhow::bail!("required state file is missing: {relative_path}");
        }
    }
    Ok(())
}
