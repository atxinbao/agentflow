use crate::{
    model::{StateSession, StateSessionUpdate, STATE_SESSION_VERSION},
    storage::{read_json, sorted_child_paths, unix_timestamp_seconds, write_json},
};
use anyhow::Result;
use std::path::Path;

pub fn update_state_session(
    project_root: impl AsRef<Path>,
    update: StateSessionUpdate,
) -> Result<StateSession> {
    let root = crate::storage::canonical_project_root(project_root)?;
    let path = root
        .join(".agentflow/state/sessions")
        .join(format!("{}.json", update.session_id));
    let existing = if path.is_file() {
        read_json::<StateSession>(&path).ok()
    } else {
        None
    };
    let now = unix_timestamp_seconds();
    let session = StateSession {
        version: STATE_SESSION_VERSION.to_string(),
        session_id: update.session_id.clone(),
        project_root: root.display().to_string(),
        active_role: update
            .active_role
            .or_else(|| existing.as_ref().map(|value| value.active_role.clone()))
            .unwrap_or_else(|| "Work Agent".to_string()),
        active_issue_id: update.active_issue_id.or_else(|| {
            existing
                .as_ref()
                .and_then(|value| value.active_issue_id.clone())
        }),
        active_run_id: update.active_run_id.or_else(|| {
            existing
                .as_ref()
                .and_then(|value| value.active_run_id.clone())
        }),
        status: update
            .status
            .or_else(|| existing.as_ref().map(|value| value.status.clone()))
            .unwrap_or_else(|| "idle".to_string()),
        waiting_for_human: update
            .waiting_for_human
            .or_else(|| existing.as_ref().map(|value| value.waiting_for_human))
            .unwrap_or(false),
        last_action: update
            .last_action
            .or_else(|| existing.as_ref().map(|value| value.last_action.clone()))
            .unwrap_or_else(|| "state.session.updated".to_string()),
        updated_at: now,
    };
    write_json(&path, &session)?;
    Ok(session)
}

pub fn load_state_session(
    project_root: impl AsRef<Path>,
    session_id: String,
) -> Result<Option<StateSession>> {
    let root = crate::storage::canonical_project_root(project_root)?;
    let path = root
        .join(".agentflow/state/sessions")
        .join(format!("{session_id}.json"));
    if !path.is_file() {
        return Ok(None);
    }
    Ok(Some(read_json(&path)?))
}

pub(crate) fn load_sessions(root: &Path) -> Result<Vec<StateSession>> {
    let mut sessions = Vec::new();
    for path in sorted_child_paths(&root.join(".agentflow/state/sessions"))? {
        if path.extension().and_then(|value| value.to_str()) == Some("json") {
            if let Ok(session) = read_json::<StateSession>(&path) {
                sessions.push(session);
            }
        }
    }
    Ok(sessions)
}
