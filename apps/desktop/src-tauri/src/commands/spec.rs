//! Spec task commands for Desktop.
//!
//! These commands expose `.agentflow/spec/**` as the task contract source for
//! the Desktop task page. They do not execute tasks or mutate runtime state.

use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DesktopSpecTaskSnapshot {
    version: String,
    project_root: String,
    projects: Vec<agentflow_spec::SpecProject>,
    issues: Vec<agentflow_spec::SpecIssue>,
    updated_at: u64,
}

#[tauri::command]
pub(crate) fn load_spec_task_snapshot(
    project_root: String,
) -> Result<DesktopSpecTaskSnapshot, String> {
    agentflow_spec::prepare_spec_workspace(&project_root).map_err(|error| error.to_string())?;
    let projects =
        agentflow_spec::list_spec_projects(&project_root).map_err(|error| error.to_string())?;
    let issues =
        agentflow_spec::list_spec_issues(&project_root).map_err(|error| error.to_string())?;

    Ok(DesktopSpecTaskSnapshot {
        version: "desktop-spec-task-snapshot.v1".to_string(),
        project_root,
        projects,
        issues,
        updated_at: unix_timestamp_seconds(),
    })
}

fn unix_timestamp_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}
