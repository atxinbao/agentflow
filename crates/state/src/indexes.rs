use crate::{
    model::{
        IssueStatusIndex, IssueStatusIndexEntry, OutputStatusIndex, RunStatusIndex,
        RunStatusIndexEntry, WorkflowGateSnapshot, WorkflowHealthSnapshot, WorkspaceStatusIndex,
        STATE_ISSUE_STATUS_INDEX_VERSION, STATE_OUTPUT_STATUS_INDEX_VERSION,
        STATE_RUN_STATUS_INDEX_VERSION, STATE_WORKSPACE_STATUS_INDEX_VERSION,
    },
    storage::{unix_timestamp_seconds, write_json},
};
use agentflow_projection::{ProjectionAuditSummary, TaskProjection};
use agentflow_workflow_core::{
    work_state_is_blocked, work_state_is_cancel, work_state_is_done, work_state_is_in_progress,
    work_state_is_in_review, work_state_is_ready_for_execution,
};
use anyhow::Result;
use std::path::Path;

pub(crate) fn write_indexes(
    root: &Path,
    health: &[WorkflowHealthSnapshot],
    gate: &WorkflowGateSnapshot,
) -> Result<()> {
    let now = unix_timestamp_seconds();
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
                display_status: display_status(projection, &projection.audit),
                priority: issue
                    .as_ref()
                    .map(|issue| format!("{:?}", issue.priority).to_lowercase())
                    .unwrap_or_else(|| "p2".to_string()),
                execution_risk: "normal".to_string(),
                latest_run_id: projection.latest_run_id.clone(),
                execute_status: projection
                    .runtime
                    .run_id
                    .as_ref()
                    .map(|_| projection.runtime.run_status.clone())
                    .or_else(|| {
                        projection.latest_run_id.as_ref().map(|_| {
                            run_status_from_task_state(&projection.current_state).to_string()
                        })
                    }),
                evidence_status: evidence_status(projection),
                delivery_status: delivery_status(projection),
                audit_status: projection_audit_status(&projection.audit)
                    .unwrap_or_else(|| audit_status.clone()),
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
                    execute_status: projection.runtime.run_status.clone(),
                    evidence_status: evidence_status(projection),
                    delivery_status: delivery_status(projection),
                    audit_status: projection_audit_status(&projection.audit)
                        .unwrap_or_else(|| audit_status.clone()),
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
        .filter(|projection| evidence_status(projection) == "ready")
        .count();
    let release_deliveries = projections
        .iter()
        .filter(|projection| delivery_status(projection) != "missing")
        .count();
    let audits = projections
        .iter()
        .filter(|projection| projection.audit.status != "not-requested")
        .count();
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

fn display_status(projection: &TaskProjection, audit: &ProjectionAuditSummary) -> String {
    if work_state_is_cancel(&projection.current_state)
        || work_state_is_done(&projection.current_state)
    {
        return projection.current_state.clone();
    }
    if let Some(status) = audit_display_status(audit) {
        return status;
    }
    projection.display_status.clone()
}

fn audit_display_status(audit: &ProjectionAuditSummary) -> Option<String> {
    match audit.status.as_str() {
        "passed" | "passed-with-warnings" => Some("done".to_string()),
        "failed" => Some("in_review".to_string()),
        "cancelled" => Some("cancel".to_string()),
        _ => None,
    }
}

fn projection_audit_status(
    audit: &ProjectionAuditSummary,
) -> Option<crate::model::WorkflowAuditStatus> {
    Some(match audit.status.as_str() {
        "requested" => crate::model::WorkflowAuditStatus::Requested,
        "running" => crate::model::WorkflowAuditStatus::Running,
        "passed" => crate::model::WorkflowAuditStatus::Passed,
        "passed-with-warnings" => crate::model::WorkflowAuditStatus::PassedWithWarnings,
        "failed" => crate::model::WorkflowAuditStatus::Failed,
        "cancelled" => crate::model::WorkflowAuditStatus::Cancelled,
        "not-requested" => crate::model::WorkflowAuditStatus::NotRequested,
        _ => return None,
    })
}

fn evidence_status(projection: &TaskProjection) -> String {
    projection.delivery.evidence_status.clone()
}

fn delivery_status(projection: &TaskProjection) -> String {
    projection.delivery.status.clone()
}

fn run_status_from_task_state(state: &str) -> &'static str {
    if work_state_is_ready_for_execution(state) {
        "queued"
    } else if work_state_is_in_progress(state) {
        "in_progress"
    } else if work_state_is_in_review(state) || work_state_is_done(state) {
        "completed"
    } else if work_state_is_blocked(state) {
        "blocked"
    } else if work_state_is_cancel(state) {
        "cancelled"
    } else {
        "queued"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agentflow_projection::{
        ProjectionAuditSummary, ProjectionDeliverySummary, ProjectionPublicDelivery,
        ProjectionRuntimeSummary, ProjectionSessionSummary,
    };

    fn projection(state: &str) -> TaskProjection {
        TaskProjection {
            version: "task-projection.v2".to_string(),
            issue_id: "iss-001".to_string(),
            project_id: None,
            workflow_ref: "work-agent.issue-loop@v1".to_string(),
            current_state: state.to_string(),
            display_status: state.to_string(),
            current_transition: None,
            latest_run_id: Some("run-001".to_string()),
            branch_name: None,
            timeline: Vec::new(),
            public_delivery: ProjectionPublicDelivery::default(),
            runtime: ProjectionRuntimeSummary {
                run_id: Some("run-001".to_string()),
                run_status: run_status_from_task_state(state).to_string(),
                branch_name: None,
                checkpoint_count: 0,
                latest_checkpoint_id: None,
                latest_checkpoint_state: None,
                latest_checkpoint_summary: None,
            },
            session: ProjectionSessionSummary::default(),
            delivery: ProjectionDeliverySummary {
                status: "missing".to_string(),
                evidence_status: "missing".to_string(),
                evidence_path: None,
                pr_url: None,
                merge_commit: None,
                public_record_path: None,
                ..ProjectionDeliverySummary::default()
            },
            audit: ProjectionAuditSummary {
                status: "not-requested".to_string(),
                latest_audit_id: None,
                ..ProjectionAuditSummary::default()
            },
            updated_at: 1,
        }
    }

    #[test]
    fn run_status_maps_from_task_state() {
        assert_eq!(run_status_from_task_state("backlog"), "queued");
        assert_eq!(run_status_from_task_state("todo"), "queued");
        assert_eq!(run_status_from_task_state("in_progress"), "in_progress");
        assert_eq!(run_status_from_task_state("in_review"), "completed");
        assert_eq!(run_status_from_task_state("done"), "completed");
        assert_eq!(run_status_from_task_state("blocked"), "blocked");
        assert_eq!(run_status_from_task_state("cancel"), "cancelled");
    }

    #[test]
    fn display_status_uses_projection_without_legacy_input() {
        assert_eq!(
            display_status(&projection("todo"), &ProjectionAuditSummary::default()),
            "todo"
        );
        assert_eq!(
            display_status(
                &projection("in_progress"),
                &ProjectionAuditSummary::default()
            ),
            "in_progress"
        );
        assert_eq!(
            display_status(&projection("done"), &ProjectionAuditSummary::default()),
            "done"
        );
    }
}
