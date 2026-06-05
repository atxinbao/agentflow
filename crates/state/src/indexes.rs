use crate::{
    model::{
        IssueStatusIndex, IssueStatusIndexEntry, OutputStatusIndex, RunStatusIndex,
        RunStatusIndexEntry, WorkflowGateSnapshot, WorkflowHealthSnapshot, WorkspaceStatusIndex,
        STATE_ISSUE_STATUS_INDEX_VERSION, STATE_OUTPUT_STATUS_INDEX_VERSION,
        STATE_RUN_STATUS_INDEX_VERSION, STATE_WORKSPACE_STATUS_INDEX_VERSION,
    },
    storage::{unix_timestamp_seconds, write_json},
};
use agentflow_execute::{ExecuteRunIndexEntry, ExecuteRunStatus};
use agentflow_input::issue::{DisplayStatus, InputIssue, InputIssueStatus};
use anyhow::Result;
use std::path::Path;

pub(crate) fn write_indexes(
    root: &Path,
    health: &[WorkflowHealthSnapshot],
    gate: &WorkflowGateSnapshot,
) -> Result<()> {
    let now = unix_timestamp_seconds();
    let input = agentflow_input::load_input_snapshot(root).ok();
    let execute = agentflow_execute::load_execute_snapshot(root).ok();
    let output = agentflow_output::load_output_snapshot(root).ok();
    let audit_status = gate.audit_status.clone();

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

    let issues = input
        .as_ref()
        .map(|snapshot| {
            snapshot
                .issues
                .iter()
                .map(|issue| {
                    let latest_run = execute.as_ref().and_then(|execute| {
                        execute
                            .index
                            .runs
                            .iter()
                            .filter(|run| run.issue_id == issue.issue_id)
                            .max_by_key(|run| run.updated_at)
                    });
                    let latest_run_id = latest_run.map(|run| run.run_id.clone());
                    IssueStatusIndexEntry {
                        issue_id: issue.issue_id.clone(),
                        display_status: display_status(
                            issue,
                            latest_run,
                            output.as_ref(),
                            latest_run_id.as_deref(),
                        ),
                        risk_level: format!("{:?}", issue.risk_level).to_lowercase(),
                        latest_run_id: latest_run_id.clone(),
                        execute_status: latest_run
                            .map(|run| format!("{:?}", run.status).to_lowercase()),
                        evidence_status: evidence_status(output.as_ref(), latest_run_id.as_deref()),
                        delivery_status: delivery_status(output.as_ref(), latest_run_id.as_deref()),
                        audit_status: audit_status.clone(),
                    }
                })
                .collect()
        })
        .unwrap_or_default();
    write_json(
        &root.join(".agentflow/state/indexes/issue-status.json"),
        &IssueStatusIndex {
            version: STATE_ISSUE_STATUS_INDEX_VERSION.to_string(),
            updated_at: now,
            issues,
        },
    )?;

    let runs = execute
        .as_ref()
        .map(|snapshot| {
            snapshot
                .index
                .runs
                .iter()
                .map(|run| RunStatusIndexEntry {
                    run_id: run.run_id.clone(),
                    issue_id: run.issue_id.clone(),
                    execute_status: format!("{:?}", run.status).to_lowercase(),
                    evidence_status: evidence_status(output.as_ref(), Some(&run.run_id)),
                    delivery_status: delivery_status(output.as_ref(), Some(&run.run_id)),
                    audit_status: audit_status.clone(),
                })
                .collect()
        })
        .unwrap_or_default();
    write_json(
        &root.join(".agentflow/state/indexes/run-status.json"),
        &RunStatusIndex {
            version: STATE_RUN_STATUS_INDEX_VERSION.to_string(),
            updated_at: now,
            runs,
        },
    )?;

    let (evidence, release_deliveries, audits) = output
        .as_ref()
        .map(|snapshot| {
            (
                snapshot.index.evidence.len(),
                snapshot.index.release_deliveries.len(),
                snapshot.index.audits.len(),
            )
        })
        .unwrap_or((0, 0, 0));
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
    issue: &InputIssue,
    latest_run: Option<&ExecuteRunIndexEntry>,
    output: Option<&agentflow_output::OutputSnapshot>,
    latest_run_id: Option<&str>,
) -> DisplayStatus {
    if matches!(issue.status, InputIssueStatus::Canceled) {
        return DisplayStatus::Cancel;
    }

    if let Some(status) = audit_display_status(output, &issue.issue_id) {
        return status;
    }

    if let Some(run) = latest_run {
        return match run.status {
            ExecuteRunStatus::Cancelled => DisplayStatus::Cancel,
            ExecuteRunStatus::Completed => DisplayStatus::Review,
            ExecuteRunStatus::Failed => DisplayStatus::Review,
            ExecuteRunStatus::Queued
            | ExecuteRunStatus::Preflight
            | ExecuteRunStatus::Blocked
            | ExecuteRunStatus::Planned
            | ExecuteRunStatus::Checkpointed
            | ExecuteRunStatus::Patching
            | ExecuteRunStatus::Running
            | ExecuteRunStatus::Validating => DisplayStatus::InProgress,
        };
    }

    if output_has_issue_delivery(output, &issue.issue_id, latest_run_id) {
        return DisplayStatus::Review;
    }

    DisplayStatus::from_input_status(&issue.status)
}

fn audit_display_status(
    output: Option<&agentflow_output::OutputSnapshot>,
    issue_id: &str,
) -> Option<DisplayStatus> {
    output.and_then(|snapshot| {
        snapshot
            .index
            .audits
            .iter()
            .rev()
            .find(|entry| entry.issue_id == issue_id)
            .map(|entry| match entry.status.as_str() {
                "passed" | "passed-with-warnings" => DisplayStatus::Done,
                "cancelled" => DisplayStatus::Cancel,
                _ => DisplayStatus::Review,
            })
    })
}

fn output_has_issue_delivery(
    output: Option<&agentflow_output::OutputSnapshot>,
    issue_id: &str,
    latest_run_id: Option<&str>,
) -> bool {
    let Some(snapshot) = output else {
        return false;
    };

    snapshot.index.release_deliveries.iter().any(|entry| {
        entry.issue_id == issue_id || latest_run_id.is_some_and(|run_id| entry.run_id == run_id)
    }) || snapshot.index.evidence.iter().any(|entry| {
        entry.issue_id == issue_id || latest_run_id.is_some_and(|run_id| entry.run_id == run_id)
    })
}

fn evidence_status(
    output: Option<&agentflow_output::OutputSnapshot>,
    run_id: Option<&str>,
) -> String {
    let Some(run_id) = run_id else {
        return "missing".to_string();
    };
    output
        .and_then(|snapshot| {
            snapshot
                .index
                .evidence
                .iter()
                .find(|entry| entry.run_id == run_id)
        })
        .map(|entry| entry.status.clone())
        .unwrap_or_else(|| "missing".to_string())
}

fn delivery_status(
    output: Option<&agentflow_output::OutputSnapshot>,
    run_id: Option<&str>,
) -> String {
    let Some(run_id) = run_id else {
        return "missing".to_string();
    };
    output
        .and_then(|snapshot| {
            snapshot
                .index
                .release_deliveries
                .iter()
                .find(|entry| entry.run_id == run_id)
        })
        .map(|entry| entry.status.clone())
        .unwrap_or_else(|| "missing".to_string())
}
