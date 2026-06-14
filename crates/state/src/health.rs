use crate::{
    model::{WorkflowHealthSnapshot, STATE_HEALTH_VERSION},
    storage::unix_timestamp_seconds,
};
use agentflow_agent_manual::model::{AgentEnvironmentState, WorkspaceOwnershipState};
use agentflow_panel::PanelStatus;
use anyhow::Result;
use std::{fs, path::Path};

pub(crate) fn collect_health(root: &Path) -> Vec<WorkflowHealthSnapshot> {
    vec![
        workspace_health(root),
        define_health(root),
        panel_health(root),
        spec_health(root),
        projection_health(root),
        tasks_health(root),
        events_health(root),
        audit_health(root),
    ]
}

fn workspace_health(root: &Path) -> WorkflowHealthSnapshot {
    let checked_at = unix_timestamp_seconds();
    match agentflow_agent_manual::check_agentflow_workspace_ownership(root) {
        Ok(ownership) => WorkflowHealthSnapshot {
            version: STATE_HEALTH_VERSION.to_string(),
            module: "workspace".to_string(),
            status: ownership_status_label(&ownership.status).to_string(),
            ready: ownership.ready_for_prepare,
            source_path: ".agentflow/workspace-manifest.json".to_string(),
            checked_at,
            warnings: ownership.warnings,
            errors: ownership.errors,
        },
        Err(error) => failed_health("workspace", ".agentflow/workspace-manifest.json", error),
    }
}

fn define_health(root: &Path) -> WorkflowHealthSnapshot {
    match agentflow_agent_manual::validate_agent_working_manual(root) {
        Ok(status) => WorkflowHealthSnapshot {
            version: STATE_HEALTH_VERSION.to_string(),
            module: "define".to_string(),
            status: agent_status_label(status.status).to_string(),
            ready: status.ready,
            source_path: ".agentflow/define/agent/Agentflow.md".to_string(),
            checked_at: status.checked_at,
            warnings: status.warnings,
            errors: status.errors,
        },
        Err(error) => failed_health("define", ".agentflow/define/agent/Agentflow.md", error),
    }
}

fn panel_health(root: &Path) -> WorkflowHealthSnapshot {
    match agentflow_panel::load_project_panel_status(root) {
        Ok(status) => WorkflowHealthSnapshot {
            version: STATE_HEALTH_VERSION.to_string(),
            module: "panel".to_string(),
            status: panel_status_label(&status.status).to_string(),
            ready: matches!(status.status, PanelStatus::Ready | PanelStatus::Degraded),
            source_path: ".agentflow/panel/manifest.json".to_string(),
            checked_at: unix_timestamp_seconds(),
            warnings: status.degraded_reasons,
            errors: status.last_error.into_iter().collect(),
        },
        Err(error) => {
            missing_or_failed_health(root, "panel", ".agentflow/panel/manifest.json", error)
        }
    }
}

fn spec_health(root: &Path) -> WorkflowHealthSnapshot {
    directory_health(
        root,
        "spec",
        ".agentflow/spec/manifest.json",
        &[
            ".agentflow/spec",
            ".agentflow/spec/projects",
            ".agentflow/spec/issues",
        ],
        &[
            ".agentflow/spec/manifest.json",
            ".agentflow/spec/index.json",
        ],
    )
}

fn projection_health(root: &Path) -> WorkflowHealthSnapshot {
    directory_health(
        root,
        "projection",
        ".agentflow/projections/tasks",
        &[
            ".agentflow/projections",
            ".agentflow/projections/tasks",
            ".agentflow/projections/projects",
            ".agentflow/indexes",
        ],
        &[],
    )
}

fn tasks_health(root: &Path) -> WorkflowHealthSnapshot {
    directory_health(
        root,
        "tasks",
        ".agentflow/tasks",
        &[".agentflow/tasks"],
        &[],
    )
}

fn events_health(root: &Path) -> WorkflowHealthSnapshot {
    directory_health(
        root,
        "events",
        ".agentflow/events",
        &[".agentflow/events"],
        &[],
    )
}

fn audit_health(root: &Path) -> WorkflowHealthSnapshot {
    match agentflow_output::load_audit_status(root) {
        Ok(status) => WorkflowHealthSnapshot {
            version: STATE_HEALTH_VERSION.to_string(),
            module: "audit".to_string(),
            status: if status.summary.audits == 0 {
                "idle".to_string()
            } else {
                status.status
            },
            ready: true,
            source_path: ".agentflow/audit/manifest.json".to_string(),
            checked_at: unix_timestamp_seconds(),
            warnings: Vec::new(),
            errors: Vec::new(),
        },
        Err(_) => WorkflowHealthSnapshot {
            version: STATE_HEALTH_VERSION.to_string(),
            module: "audit".to_string(),
            status: "idle".to_string(),
            ready: true,
            source_path: ".agentflow/audit/manifest.json".to_string(),
            checked_at: unix_timestamp_seconds(),
            warnings: Vec::new(),
            errors: Vec::new(),
        },
    }
}

fn failed_health(
    module: &str,
    source_path: &str,
    error: impl std::fmt::Display,
) -> WorkflowHealthSnapshot {
    WorkflowHealthSnapshot {
        version: STATE_HEALTH_VERSION.to_string(),
        module: module.to_string(),
        status: "failed".to_string(),
        ready: false,
        source_path: source_path.to_string(),
        checked_at: unix_timestamp_seconds(),
        warnings: Vec::new(),
        errors: vec![error.to_string()],
    }
}

fn missing_or_failed_health(
    root: &Path,
    module: &str,
    source_path: &str,
    error: impl std::fmt::Display,
) -> WorkflowHealthSnapshot {
    let status = if !root.join(source_path).is_file()
        || error.to_string().contains("No such file")
        || error.to_string().contains("os error 2")
        || error.to_string().contains("missing")
    {
        "missing"
    } else {
        "failed"
    };
    WorkflowHealthSnapshot {
        version: STATE_HEALTH_VERSION.to_string(),
        module: module.to_string(),
        status: status.to_string(),
        ready: false,
        source_path: source_path.to_string(),
        checked_at: unix_timestamp_seconds(),
        warnings: Vec::new(),
        errors: vec![error.to_string()],
    }
}

fn directory_health(
    root: &Path,
    module: &str,
    source_path: &str,
    directories: &[&str],
    files: &[&str],
) -> WorkflowHealthSnapshot {
    let mut errors = Vec::new();
    for directory in directories {
        let path = root.join(directory);
        if !path.is_dir() {
            errors.push(format!("missing directory: {directory}"));
        }
    }
    for file in files {
        let path = root.join(file);
        if !path.is_file() {
            errors.push(format!("missing file: {file}"));
        } else if let Err(error) = fs::read_to_string(&path) {
            errors.push(format!("read {file}: {error}"));
        }
    }
    WorkflowHealthSnapshot {
        version: STATE_HEALTH_VERSION.to_string(),
        module: module.to_string(),
        status: if errors.is_empty() {
            "ready"
        } else {
            "missing"
        }
        .to_string(),
        ready: errors.is_empty(),
        source_path: source_path.to_string(),
        checked_at: unix_timestamp_seconds(),
        warnings: Vec::new(),
        errors,
    }
}

fn ownership_status_label(status: &WorkspaceOwnershipState) -> &'static str {
    match status {
        WorkspaceOwnershipState::None => "missing",
        WorkspaceOwnershipState::ManagedCurrent => "ready",
        WorkspaceOwnershipState::ManagedLegacy => "degraded",
        WorkspaceOwnershipState::Foreign | WorkspaceOwnershipState::Corrupted => "blocked",
        WorkspaceOwnershipState::Blocked => "blocked",
    }
}

fn agent_status_label(status: AgentEnvironmentState) -> &'static str {
    match status {
        AgentEnvironmentState::Ready | AgentEnvironmentState::Repaired => "ready",
        AgentEnvironmentState::Degraded => "degraded",
        AgentEnvironmentState::Missing => "missing",
        AgentEnvironmentState::Blocked => "blocked",
        AgentEnvironmentState::Failed => "failed",
        AgentEnvironmentState::Checking | AgentEnvironmentState::Repairing => "working",
    }
}

fn panel_status_label(status: &PanelStatus) -> &'static str {
    match status {
        PanelStatus::Ready => "ready",
        PanelStatus::Degraded | PanelStatus::Stale => "degraded",
        PanelStatus::Missing => "missing",
        PanelStatus::Failed => "failed",
        PanelStatus::Indexing => "working",
    }
}

pub(crate) fn health_ready(health: &[WorkflowHealthSnapshot]) -> bool {
    health
        .iter()
        .filter(|item| item.module != "audit")
        .all(|item| item.ready || item.status == "degraded")
}

pub(crate) fn health_status_map(
    health: &[WorkflowHealthSnapshot],
) -> std::collections::BTreeMap<String, String> {
    health
        .iter()
        .map(|item| (item.module.clone(), item.status.clone()))
        .collect()
}

pub(crate) fn write_health(root: &Path, health: &[WorkflowHealthSnapshot]) -> Result<()> {
    for item in health {
        crate::storage::write_json(
            &root
                .join(".agentflow/state/health")
                .join(format!("{}.json", item.module)),
            item,
        )?;
    }
    Ok(())
}
