use crate::{
    model::{
        IssueStatusIndex, IssueStatusIndexEntry, OutputStatusIndex, RunStatusIndex,
        RunStatusIndexEntry, WorkflowGateSnapshot, WorkflowHealthSnapshot, WorkspaceStatusIndex,
        STATE_ISSUE_STATUS_INDEX_VERSION, STATE_OUTPUT_STATUS_INDEX_VERSION,
        STATE_RUN_STATUS_INDEX_VERSION, STATE_WORKSPACE_STATUS_INDEX_VERSION,
    },
    storage::{unix_timestamp_seconds, write_json},
};
use agentflow_projection::TaskProjection;
use anyhow::Result;
use std::path::Path;

pub(crate) fn write_indexes(
    root: &Path,
    health: &[WorkflowHealthSnapshot],
    gate: &WorkflowGateSnapshot,
) -> Result<()> {
    let now = unix_timestamp_seconds();
    let audit_index = agentflow_audit::load_audit_index(root).ok();
    let audit_status = gate.audit_status.clone();
    let projection_index = agentflow_projection::rebuild_projections(root)
        .and_then(|_| agentflow_projection::load_issue_status_index(root))
        .ok();

    write_json(
        &root.join(".agentflow/state/indexes/workspace-status.json"),
        &WorkspaceStatusIndex {
            version: STATE_WORKSPACE_STATUS_INDEX_VERSION.to_string(),
            updated_at: now,
            health: health.to_vec(),
            current_stage: gate.current_stage.clone(),
            audit_status: audit_status.clone(),
        },
    )?;

    let projections = projection_index
        .as_ref()
        .map(|index| {
            index
                .issues
                .iter()
                .filter_map(|entry| {
                    agentflow_projection::load_task_projection(root, &entry.issue_id).ok()
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let issues = projections
        .iter()
        .map(|projection| {
            let issue = agentflow_spec::read_spec_issue(root, &projection.issue_id).ok();
            IssueStatusIndexEntry {
                issue_id: projection.issue_id.clone(),
                display_status: display_status(
                    projection,
                    audit_index.as_ref(),
                    &projection.issue_id,
                ),
                priority: issue
                    .as_ref()
                    .map(|issue| format!("{:?}", issue.priority).to_lowercase())
                    .unwrap_or_else(|| "p2".to_string()),
                execution_risk: "normal".to_string(),
                latest_run_id: projection.latest_run_id.clone(),
                execute_status: projection
                    .latest_run_id
                    .as_ref()
                    .map(|_| run_status_from_task_state(&projection.current_state).to_string()),
                evidence_status: evidence_status(root, projection),
                delivery_status: delivery_status(projection),
                audit_status: audit_status.clone(),
            }
        })
        .collect::<Vec<_>>();
    write_json(
        &root.join(".agentflow/state/indexes/issue-status.json"),
        &IssueStatusIndex {
            version: STATE_ISSUE_STATUS_INDEX_VERSION.to_string(),
            updated_at: now,
            issues,
        },
    )?;

    let runs = projections
        .iter()
        .filter_map(|projection| {
            projection
                .latest_run_id
                .as_ref()
                .map(|run_id| RunStatusIndexEntry {
                    run_id: run_id.clone(),
                    issue_id: projection.issue_id.clone(),
                    execute_status: run_status_from_task_state(&projection.current_state)
                        .to_string(),
                    evidence_status: evidence_status(root, projection),
                    delivery_status: delivery_status(projection),
                    audit_status: audit_status.clone(),
                })
        })
        .collect::<Vec<_>>();
    write_json(
        &root.join(".agentflow/state/indexes/run-status.json"),
        &RunStatusIndex {
            version: STATE_RUN_STATUS_INDEX_VERSION.to_string(),
            updated_at: now,
            runs,
        },
    )?;

    let evidence = projections
        .iter()
        .filter(|projection| evidence_status(root, projection) == "ready")
        .count();
    let release_deliveries = projections
        .iter()
        .filter(|projection| delivery_status(projection) != "missing")
        .count();
    let audits = audit_index
        .as_ref()
        .map(|index| index.audits.len())
        .unwrap_or_default();
    write_json(
        &root.join(".agentflow/state/indexes/output-status.json"),
        &OutputStatusIndex {
            version: STATE_OUTPUT_STATUS_INDEX_VERSION.to_string(),
            updated_at: now,
            evidence,
            release_deliveries,
            audits,
            audit_status,
        },
    )
}

fn display_status(
    projection: &TaskProjection,
    audit_index: Option<&agentflow_audit::AuditIndex>,
    issue_id: &str,
) -> String {
    if matches!(projection.current_state.as_str(), "cancel" | "done") {
        return projection.current_state.clone();
    }
    if let Some(status) = audit_display_status(audit_index, issue_id) {
        return status;
    }
    projection.display_status.clone()
}

fn audit_display_status(
    audit_index: Option<&agentflow_audit::AuditIndex>,
    issue_id: &str,
) -> Option<String> {
    audit_index.and_then(|index| {
        index
            .audits
            .iter()
            .rev()
            .find(|entry| entry.source_issue_id.as_deref() == Some(issue_id))
            .and_then(|entry| match entry.status {
                agentflow_audit::AuditStatus::Passed
                | agentflow_audit::AuditStatus::PassedWithWarnings => Some("done".to_string()),
                agentflow_audit::AuditStatus::Failed => Some("in_review".to_string()),
                agentflow_audit::AuditStatus::Cancelled => Some("cancel".to_string()),
                agentflow_audit::AuditStatus::Requested | agentflow_audit::AuditStatus::Running => {
                    None
                }
            })
    })
}

fn evidence_status(root: &Path, projection: &TaskProjection) -> String {
    projection
        .public_delivery
        .evidence_path
        .as_deref()
        .filter(|path| root.join(path).is_file())
        .map(|_| "ready".to_string())
        .unwrap_or_else(|| "missing".to_string())
}

fn delivery_status(projection: &TaskProjection) -> String {
    let delivery = &projection.public_delivery;
    if delivery.changelog_path.is_some() || delivery.release_notes_url.is_some() {
        "published".to_string()
    } else if delivery.pr_url.is_some() || delivery.merge_commit.is_some() {
        "ready".to_string()
    } else {
        "missing".to_string()
    }
}

fn run_status_from_task_state(state: &str) -> &'static str {
    match state {
        "todo" | "in_progress" => "running",
        "in_review" | "done" => "completed",
        "blocked" => "blocked",
        "cancel" => "cancelled",
        _ => "queued",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agentflow_projection::ProjectionPublicDelivery;

    fn projection(state: &str) -> TaskProjection {
        TaskProjection {
            version: "task-projection.v1".to_string(),
            issue_id: "iss-001".to_string(),
            project_id: None,
            workflow_ref: "build-agent.issue-loop@v1".to_string(),
            current_state: state.to_string(),
            display_status: state.to_string(),
            current_transition: None,
            latest_run_id: Some("run-001".to_string()),
            branch_name: None,
            timeline: Vec::new(),
            public_delivery: ProjectionPublicDelivery::default(),
            updated_at: 1,
        }
    }

    #[test]
    fn run_status_maps_from_task_state() {
        assert_eq!(run_status_from_task_state("backlog"), "queued");
        assert_eq!(run_status_from_task_state("todo"), "running");
        assert_eq!(run_status_from_task_state("in_progress"), "running");
        assert_eq!(run_status_from_task_state("in_review"), "completed");
        assert_eq!(run_status_from_task_state("done"), "completed");
        assert_eq!(run_status_from_task_state("blocked"), "blocked");
        assert_eq!(run_status_from_task_state("cancel"), "cancelled");
    }

    #[test]
    fn display_status_uses_projection_without_legacy_input() {
        assert_eq!(display_status(&projection("todo"), None, "iss-001"), "todo");
        assert_eq!(
            display_status(&projection("in_progress"), None, "iss-001"),
            "in_progress"
        );
        assert_eq!(display_status(&projection("done"), None, "iss-001"), "done");
    }
}
