use crate::{
    model::{
        IssueStatusIndex, IssueStatusIndexEntry, OutputStatusIndex, RunStatusIndex,
        RunStatusIndexEntry, WorkflowGateSnapshot, WorkflowHealthSnapshot, WorkspaceStatusIndex,
        STATE_ISSUE_STATUS_INDEX_VERSION, STATE_OUTPUT_STATUS_INDEX_VERSION,
        STATE_RUN_STATUS_INDEX_VERSION, STATE_WORKSPACE_STATUS_INDEX_VERSION,
    },
    readiness::issue_has_readiness_blocker,
    storage::{unix_timestamp_seconds, write_json},
};
use agentflow_execute::{ExecuteRunIndexEntry, ExecuteRunStatus};
use agentflow_input::issue::{DisplayStatus, InputIssue, InputIssueStatus};
use agentflow_projection::TaskProjection;
use anyhow::Result;
use std::{collections::BTreeMap, path::Path};

pub(crate) fn write_indexes(
    root: &Path,
    health: &[WorkflowHealthSnapshot],
    gate: &WorkflowGateSnapshot,
) -> Result<()> {
    let now = unix_timestamp_seconds();
    let input = agentflow_input::load_input_snapshot(root).ok();
    let execute = agentflow_execute::load_execute_snapshot(root).ok();
    let audit_index = agentflow_audit::load_audit_index(root).ok();
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
            let runs_by_issue = runs_by_issue(execute.as_ref());
            snapshot
                .issues
                .iter()
                .map(|issue| {
                    let latest_run = authoritative_run_for_issue(
                        issue,
                        runs_by_issue.get(&issue.issue_id).map(Vec::as_slice),
                        load_task_projection(root, &issue.issue_id).as_ref(),
                    );
                    let latest_run_id = latest_run.map(|run| run.run_id.clone());
                    let projection = load_task_projection(root, &issue.issue_id);
                    IssueStatusIndexEntry {
                        issue_id: issue.issue_id.clone(),
                        display_status: display_status(
                            issue,
                            audit_index.as_ref(),
                            issue_has_readiness_blocker(issue, &gate.blocked_actions),
                        ),
                        priority: issue.priority.as_str().to_string(),
                        execution_risk: format!("{:?}", issue.execution_risk).to_lowercase(),
                        latest_run_id: latest_run_id.clone(),
                        execute_status: latest_run
                            .map(|run| format!("{:?}", run.status).to_lowercase()),
                        evidence_status: evidence_status(root, projection.as_ref()),
                        delivery_status: delivery_status(projection.as_ref()),
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
                    evidence_status: evidence_status(
                        root,
                        load_task_projection(root, &run.issue_id).as_ref(),
                    ),
                    delivery_status: delivery_status(
                        load_task_projection(root, &run.issue_id).as_ref(),
                    ),
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

    let projections = input
        .as_ref()
        .map(|snapshot| {
            snapshot
                .issues
                .iter()
                .filter_map(|issue| load_task_projection(root, &issue.issue_id))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let evidence = projections
        .iter()
        .filter(|projection| projection.public_delivery.evidence_path.is_some())
        .count();
    let release_deliveries = projections
        .iter()
        .filter(|projection| delivery_status(Some(projection)) != "missing")
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
    issue: &InputIssue,
    audit_index: Option<&agentflow_audit::AuditIndex>,
    _blocked_by_gate: bool,
) -> DisplayStatus {
    if matches!(issue.status, InputIssueStatus::Cancel) {
        return DisplayStatus::Cancel;
    }
    if matches!(issue.status, InputIssueStatus::Done) {
        return DisplayStatus::Done;
    }

    if let Some(status) = audit_display_status(audit_index, &issue.issue_id) {
        return status;
    }

    DisplayStatus::from_input_status(&issue.status)
}

fn runs_by_issue(
    execute: Option<&agentflow_execute::ExecuteSnapshot>,
) -> BTreeMap<String, Vec<ExecuteRunIndexEntry>> {
    let mut grouped = BTreeMap::<String, Vec<ExecuteRunIndexEntry>>::new();
    let Some(execute) = execute else {
        return grouped;
    };
    for run in &execute.index.runs {
        grouped
            .entry(run.issue_id.clone())
            .or_default()
            .push(run.clone());
    }
    for runs in grouped.values_mut() {
        runs.sort_by(|left, right| {
            (right.updated_at, right.run_id.as_str()).cmp(&(left.updated_at, left.run_id.as_str()))
        });
    }
    grouped
}

fn authoritative_run_for_issue<'a>(
    issue: &InputIssue,
    runs: Option<&'a [ExecuteRunIndexEntry]>,
    projection: Option<&TaskProjection>,
) -> Option<&'a ExecuteRunIndexEntry> {
    let runs = runs?;
    let delivery_run_id = issue
        .latest_run_id
        .as_deref()
        .or_else(|| projection.and_then(|projection| projection.latest_run_id.as_deref()));
    match issue.status {
        InputIssueStatus::Done | InputIssueStatus::InReview => delivery_run_id
            .and_then(|run_id| runs.iter().find(|run| run.run_id == run_id))
            .or_else(|| {
                runs.iter().find(|run| {
                    matches!(
                        run.status,
                        ExecuteRunStatus::Completed | ExecuteRunStatus::Failed
                    )
                })
            })
            .or_else(|| runs.first()),
        InputIssueStatus::InProgress => runs
            .iter()
            .find(|run| {
                matches!(
                    run.status,
                    ExecuteRunStatus::Queued
                        | ExecuteRunStatus::Preflight
                        | ExecuteRunStatus::Planned
                        | ExecuteRunStatus::Checkpointed
                        | ExecuteRunStatus::Patching
                        | ExecuteRunStatus::Running
                        | ExecuteRunStatus::Validating
                )
            })
            .or_else(|| {
                runs.iter()
                    .find(|run| run.status == ExecuteRunStatus::Completed)
            })
            .or_else(|| runs.first()),
        InputIssueStatus::Blocked => runs
            .iter()
            .find(|run| run.status == ExecuteRunStatus::Blocked)
            .or_else(|| runs.first()),
        InputIssueStatus::Todo | InputIssueStatus::Backlog => runs
            .iter()
            .find(|run| {
                matches!(
                    run.status,
                    ExecuteRunStatus::Queued
                        | ExecuteRunStatus::Preflight
                        | ExecuteRunStatus::Planned
                        | ExecuteRunStatus::Checkpointed
                        | ExecuteRunStatus::Patching
                        | ExecuteRunStatus::Running
                        | ExecuteRunStatus::Validating
                )
            })
            .or_else(|| runs.first()),
        InputIssueStatus::Cancel => runs
            .iter()
            .find(|run| run.status == ExecuteRunStatus::Cancelled)
            .or_else(|| runs.first()),
    }
}

fn audit_display_status(
    audit_index: Option<&agentflow_audit::AuditIndex>,
    issue_id: &str,
) -> Option<DisplayStatus> {
    audit_index.and_then(|index| {
        index
            .audits
            .iter()
            .rev()
            .find(|entry| entry.source_issue_id.as_deref() == Some(issue_id))
            .and_then(|entry| match entry.status.as_str() {
                "passed" | "passed-with-warnings" => Some(DisplayStatus::Done),
                "failed" => Some(DisplayStatus::InReview),
                "cancelled" => Some(DisplayStatus::Cancel),
                "requested" | "running" => None,
                _ => None,
            })
    })
}

fn evidence_status(root: &Path, projection: Option<&TaskProjection>) -> String {
    projection
        .and_then(|projection| projection.public_delivery.evidence_path.as_deref())
        .filter(|path| root.join(path).is_file())
        .map(|_| "ready".to_string())
        .unwrap_or_else(|| "missing".to_string())
}

fn delivery_status(projection: Option<&TaskProjection>) -> String {
    let Some(delivery) = projection.map(|projection| &projection.public_delivery) else {
        return "missing".to_string();
    };
    if delivery.changelog_path.is_some() || delivery.release_notes_url.is_some() {
        "published".to_string()
    } else if delivery.pr_url.is_some() || delivery.merge_commit.is_some() {
        "ready".to_string()
    } else {
        "missing".to_string()
    }
}

fn load_task_projection(root: &Path, issue_id: &str) -> Option<TaskProjection> {
    agentflow_projection::load_task_projection(root, issue_id).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use agentflow_execute::ExecuteRunStatus;

    fn issue(status: InputIssueStatus) -> InputIssue {
        InputIssue {
            issue_id: "iss-001".to_string(),
            status,
            ..InputIssue::default()
        }
    }

    fn run_with_id(
        run_id: &str,
        status: ExecuteRunStatus,
        updated_at: u64,
    ) -> ExecuteRunIndexEntry {
        ExecuteRunIndexEntry {
            run_id: run_id.to_string(),
            issue_id: "iss-001".to_string(),
            status,
            updated_at,
            ..ExecuteRunIndexEntry::default()
        }
    }

    #[test]
    fn display_status_mapping_covers_input_execute_and_audit_states() {
        assert_eq!(
            display_status(&issue(InputIssueStatus::Backlog), None, false),
            DisplayStatus::Backlog
        );
        assert_eq!(
            display_status(&issue(InputIssueStatus::Todo), None, false),
            DisplayStatus::Todo
        );
        assert_eq!(
            display_status(&issue(InputIssueStatus::InProgress), None, false,),
            DisplayStatus::InProgress
        );
        assert_eq!(
            display_status(&issue(InputIssueStatus::Todo), None, false),
            DisplayStatus::Todo
        );
        assert_eq!(
            display_status(&issue(InputIssueStatus::InReview), None, false),
            DisplayStatus::InReview
        );
        assert_eq!(
            display_status(&issue(InputIssueStatus::Done), None, false),
            DisplayStatus::Done
        );
        assert_eq!(
            display_status(&issue(InputIssueStatus::Todo), None, true),
            DisplayStatus::Todo
        );
        assert_eq!(
            display_status(&issue(InputIssueStatus::Blocked), None, false,),
            DisplayStatus::Blocked
        );
        assert_eq!(
            audit_display_status(Some(&audit_index("requested")), "iss-001"),
            None
        );
        assert_eq!(
            audit_display_status(Some(&audit_index("passed")), "iss-001"),
            Some(DisplayStatus::Done)
        );
        assert_eq!(
            display_status(&issue(InputIssueStatus::Cancel), None, false),
            DisplayStatus::Cancel
        );
    }

    #[test]
    fn authoritative_run_prefers_completed_delivery_run_over_newer_blocked_run_after_review() {
        let mut issue = issue(InputIssueStatus::InReview);
        issue.latest_run_id = Some("run-001".to_string());
        let runs = vec![
            run_with_id("run-002", ExecuteRunStatus::Blocked, 2),
            run_with_id("run-001", ExecuteRunStatus::Completed, 1),
        ];
        let selected = authoritative_run_for_issue(&issue, Some(&runs), None).unwrap();
        assert_eq!(selected.run_id, "run-001");
    }

    #[test]
    fn authoritative_run_prefers_active_run_during_in_progress() {
        let issue = issue(InputIssueStatus::InProgress);
        let runs = vec![
            run_with_id("run-002", ExecuteRunStatus::Blocked, 2),
            run_with_id("run-001", ExecuteRunStatus::Running, 1),
        ];
        let selected = authoritative_run_for_issue(&issue, Some(&runs), None).unwrap();
        assert_eq!(selected.run_id, "run-001");
    }

    fn audit_index(status: &str) -> agentflow_audit::AuditIndex {
        let audit_status = match status {
            "passed" => agentflow_audit::AuditStatus::Passed,
            "failed" => agentflow_audit::AuditStatus::Failed,
            "cancelled" => agentflow_audit::AuditStatus::Cancelled,
            _ => agentflow_audit::AuditStatus::Requested,
        };
        agentflow_audit::AuditIndex {
            audits: vec![agentflow_audit::AuditIndexEntry {
                audit_id: "audit-001".to_string(),
                status: audit_status,
                trigger: agentflow_audit::AuditTrigger::HumanViaAgent,
                requested_by: "test".to_string(),
                requested_at: 1,
                source_delivery_id: None,
                source_run_id: Some("run-001".to_string()),
                source_issue_id: Some("iss-001".to_string()),
                source_spec_id: None,
                report_path: ".agentflow/audit/audit-001/audit-report.md".to_string(),
                audit_path: ".agentflow/audit/audit-001/audit.json".to_string(),
            }],
            ..agentflow_audit::AuditIndex::default()
        }
    }
}
