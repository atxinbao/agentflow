use crate::{
    model::{WorkflowHealthSnapshot, STATE_HEALTH_VERSION},
    storage::unix_timestamp_seconds,
};
use agentflow_agent_manual::model::{AgentEnvironmentState, WorkspaceOwnershipState};
use agentflow_execute::model::ExecuteWorkspaceStatus;
use agentflow_input::model::InputWorkspaceStatus;
use agentflow_output::model::OutputWorkspaceStatus;
use agentflow_panel::PanelStatus;
use anyhow::Result;
use std::path::Path;

pub(crate) fn collect_health(root: &Path) -> Vec<WorkflowHealthSnapshot> {
    vec![
        workspace_health(root),
        define_health(root),
        panel_health(root),
        input_health(root),
        execute_health(root),
        output_health(root),
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

fn input_health(root: &Path) -> WorkflowHealthSnapshot {
    match agentflow_input::load_input_status(root) {
        Ok(status) => {
            let status_label = if !root.join(".agentflow/input/manifest.json").is_file()
                || (!status.errors.is_empty()
                    && status.errors.iter().all(|error| is_missing_error(error)))
            {
                "missing".to_string()
            } else {
                input_status_label(&status.status).to_string()
            };
            WorkflowHealthSnapshot {
                version: STATE_HEALTH_VERSION.to_string(),
                module: "input".to_string(),
                status: status_label,
                ready: status.ready,
                source_path: ".agentflow/input/manifest.json".to_string(),
                checked_at: unix_timestamp_seconds(),
                warnings: status.warnings,
                errors: status.errors,
            }
        }
        Err(error) => {
            missing_or_failed_health(root, "input", ".agentflow/input/manifest.json", error)
        }
    }
}

fn execute_health(root: &Path) -> WorkflowHealthSnapshot {
    match agentflow_execute::load_execute_status(root) {
        Ok(status) => {
            let status_label = if !root.join(".agentflow/execute/manifest.json").is_file()
                || (!status.errors.is_empty()
                    && status.errors.iter().all(|error| is_missing_error(error)))
            {
                "missing".to_string()
            } else if status.summary.active_runs > 0 {
                "working".to_string()
            } else {
                execute_status_label(&status.status).to_string()
            };
            WorkflowHealthSnapshot {
                version: STATE_HEALTH_VERSION.to_string(),
                module: "execute".to_string(),
                status: status_label,
                ready: status.ready,
                source_path: ".agentflow/execute/manifest.json".to_string(),
                checked_at: unix_timestamp_seconds(),
                warnings: status.warnings,
                errors: status.errors,
            }
        }
        Err(error) => {
            missing_or_failed_health(root, "execute", ".agentflow/execute/manifest.json", error)
        }
    }
}

fn output_health(root: &Path) -> WorkflowHealthSnapshot {
    match agentflow_output::load_output_status(root) {
        Ok(status) => {
            let status_label = if status.summary.incomplete_evidence > 0
                || status.summary.incomplete_deliveries > 0
            {
                "degraded".to_string()
            } else {
                output_status_label(&status.status).to_string()
            };
            WorkflowHealthSnapshot {
                version: STATE_HEALTH_VERSION.to_string(),
                module: "output".to_string(),
                status: status_label,
                ready: status.ready,
                source_path: ".agentflow/output/manifest.json".to_string(),
                checked_at: unix_timestamp_seconds(),
                warnings: status.warnings,
                errors: status.errors,
            }
        }
        Err(error) => {
            missing_or_failed_health(root, "output", ".agentflow/output/manifest.json", error)
        }
    }
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
            source_path: ".agentflow/output/audit/manifest.json".to_string(),
            checked_at: unix_timestamp_seconds(),
            warnings: Vec::new(),
            errors: Vec::new(),
        },
        Err(_) => WorkflowHealthSnapshot {
            version: STATE_HEALTH_VERSION.to_string(),
            module: "audit".to_string(),
            status: "idle".to_string(),
            ready: true,
            source_path: ".agentflow/output/audit/manifest.json".to_string(),
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

fn is_missing_error(message: &str) -> bool {
    message.contains("missing")
        || message.contains("No such file")
        || message.contains("os error 2")
        || message.contains("not found")
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

fn input_status_label(status: &InputWorkspaceStatus) -> &'static str {
    match status {
        InputWorkspaceStatus::Ready => "ready",
        InputWorkspaceStatus::Degraded => "degraded",
        InputWorkspaceStatus::Missing => "missing",
        InputWorkspaceStatus::Failed => "failed",
        InputWorkspaceStatus::Blocked => "blocked",
    }
}

fn execute_status_label(status: &ExecuteWorkspaceStatus) -> &'static str {
    match status {
        ExecuteWorkspaceStatus::Ready => "ready",
        ExecuteWorkspaceStatus::Degraded => "degraded",
        ExecuteWorkspaceStatus::Missing => "missing",
        ExecuteWorkspaceStatus::Failed => "failed",
        ExecuteWorkspaceStatus::Blocked => "blocked",
    }
}

fn output_status_label(status: &OutputWorkspaceStatus) -> &'static str {
    match status {
        OutputWorkspaceStatus::Ready => "ready",
        OutputWorkspaceStatus::Degraded => "degraded",
        OutputWorkspaceStatus::Missing => "missing",
        OutputWorkspaceStatus::Failed => "failed",
        OutputWorkspaceStatus::Blocked => "blocked",
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
