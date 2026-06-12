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
            let runs_by_issue = runs_by_issue(execute.as_ref());
            snapshot
                .issues
                .iter()
                .map(|issue| {
                    let latest_run = authoritative_run_for_issue(
                        issue,
                        runs_by_issue.get(&issue.issue_id).map(Vec::as_slice),
                        output.as_ref(),
                    );
                    let latest_run_id = latest_run.map(|run| run.run_id.clone());
                    IssueStatusIndexEntry {
                        issue_id: issue.issue_id.clone(),
                        display_status: display_status(
                            issue,
                            output.as_ref(),
                            latest_run_id.as_deref(),
                            issue_has_readiness_blocker(issue, &gate.blocked_actions),
                        ),
                        priority: issue.priority.as_str().to_string(),
                        execution_risk: format!("{:?}", issue.execution_risk).to_lowercase(),
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
    output: Option<&agentflow_output::OutputSnapshot>,
    _latest_run_id: Option<&str>,
    blocked_by_gate: bool,
) -> DisplayStatus {
    if matches!(issue.status, InputIssueStatus::Cancel) {
        return DisplayStatus::Cancel;
    }
    if matches!(issue.status, InputIssueStatus::Done) {
        return DisplayStatus::Done;
    }

    if let Some(status) = audit_display_status(output, &issue.issue_id) {
        return status;
    }
    if blocked_by_gate
        && matches!(
            issue.status,
            InputIssueStatus::Backlog | InputIssueStatus::Todo
        )
    {
        return DisplayStatus::Blocked;
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
    output: Option<&agentflow_output::OutputSnapshot>,
) -> Option<&'a ExecuteRunIndexEntry> {
    let runs = runs?;
    let delivery_run_id = latest_output_run_id(output, &issue.issue_id);
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

fn latest_output_run_id<'a>(
    output: Option<&'a agentflow_output::OutputSnapshot>,
    issue_id: &str,
) -> Option<&'a str> {
    let snapshot = output?;
    snapshot
        .index
        .release_deliveries
        .iter()
        .filter(|entry| entry.issue_id == issue_id)
        .max_by_key(|entry| (entry.updated_at, entry.run_id.as_str()))
        .map(|entry| entry.run_id.as_str())
        .or_else(|| {
            snapshot
                .index
                .evidence
                .iter()
                .filter(|entry| entry.issue_id == issue_id)
                .max_by_key(|entry| (entry.updated_at, entry.run_id.as_str()))
                .map(|entry| entry.run_id.as_str())
        })
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
            .and_then(|entry| match entry.status.as_str() {
                "passed" | "passed-with-warnings" => Some(DisplayStatus::Done),
                "failed" => Some(DisplayStatus::InReview),
                "cancelled" => Some(DisplayStatus::Cancel),
                "requested" | "running" => None,
                _ => None,
            })
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

#[cfg(test)]
mod tests {
    use super::*;
    use agentflow_execute::ExecuteRunStatus;
    use agentflow_output::{
        OutputIndex, OutputIndexEntry, OutputManifest, OutputSnapshot, OutputStatusSnapshot,
        OutputSummary, OutputWorkspaceStatus,
    };
    use std::collections::BTreeMap;

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

    fn output_with_delivery(status: &str) -> agentflow_output::OutputSnapshot {
        OutputSnapshot {
            version: agentflow_output::OUTPUT_SNAPSHOT_VERSION.to_string(),
            project_root: "/tmp/agentflow-test".to_string(),
            ready: true,
            status: OutputStatusSnapshot {
                version: agentflow_output::OUTPUT_STATUS_VERSION.to_string(),
                project_root: "/tmp/agentflow-test".to_string(),
                status: OutputWorkspaceStatus::Ready,
                ready: true,
                manifest_exists: true,
                index_exists: true,
                summary: OutputSummary::default(),
                missing_paths: Vec::new(),
                warnings: Vec::new(),
                errors: Vec::new(),
            },
            manifest: OutputManifest {
                version: agentflow_output::OUTPUT_MANIFEST_VERSION.to_string(),
                project_root: "/tmp/agentflow-test".to_string(),
                status: OutputWorkspaceStatus::Ready,
                paths: BTreeMap::new(),
                summary: OutputSummary::default(),
                updated_at: 1,
            },
            index: OutputIndex {
                release_deliveries: vec![OutputIndexEntry {
                    run_id: "run-001".to_string(),
                    issue_id: "iss-001".to_string(),
                    status: status.to_string(),
                    updated_at: 1,
                    ..OutputIndexEntry::default()
                }],
                ..OutputIndex::default()
            },
        }
    }

    #[test]
    fn display_status_mapping_covers_input_execute_output_and_audit_states() {
        assert_eq!(
            display_status(&issue(InputIssueStatus::Backlog), None, None, false),
            DisplayStatus::Backlog
        );
        assert_eq!(
            display_status(&issue(InputIssueStatus::Todo), None, None, false),
            DisplayStatus::Todo
        );
        assert_eq!(
            display_status(&issue(InputIssueStatus::InProgress), None, None, false,),
            DisplayStatus::InProgress
        );
        let output = output_with_delivery("drafted");
        assert_eq!(
            display_status(
                &issue(InputIssueStatus::Todo),
                Some(&output),
                Some("run-001"),
                false,
            ),
            DisplayStatus::Todo
        );
        assert_eq!(
            display_status(
                &issue(InputIssueStatus::InReview),
                Some(&output),
                Some("run-001"),
                false,
            ),
            DisplayStatus::InReview
        );
        assert_eq!(
            display_status(&issue(InputIssueStatus::Done), None, None, false),
            DisplayStatus::Done
        );
        assert_eq!(
            display_status(&issue(InputIssueStatus::Todo), None, None, true),
            DisplayStatus::Blocked
        );
        assert_eq!(
            display_status(&issue(InputIssueStatus::Blocked), None, None, false,),
            DisplayStatus::Blocked
        );
        assert_eq!(
            audit_display_status(Some(&output_with_audit("requested")), "iss-001"),
            None
        );
        assert_eq!(
            audit_display_status(Some(&output_with_audit("passed")), "iss-001"),
            Some(DisplayStatus::Done)
        );
        assert_eq!(
            display_status(&issue(InputIssueStatus::Cancel), None, None, false),
            DisplayStatus::Cancel
        );
    }

    #[test]
    fn authoritative_run_prefers_completed_delivery_run_over_newer_blocked_run_after_review() {
        let issue = issue(InputIssueStatus::InReview);
        let output = output_with_delivery("drafted");
        let runs = vec![
            run_with_id("run-002", ExecuteRunStatus::Blocked, 2),
            run_with_id("run-001", ExecuteRunStatus::Completed, 1),
        ];
        let selected = authoritative_run_for_issue(&issue, Some(&runs), Some(&output)).unwrap();
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

    fn output_with_audit(status: &str) -> agentflow_output::OutputSnapshot {
        let mut output = output_with_delivery("drafted");
        output.index.audits = vec![OutputIndexEntry {
            run_id: "run-001".to_string(),
            issue_id: "iss-001".to_string(),
            status: status.to_string(),
            updated_at: 1,
            ..OutputIndexEntry::default()
        }];
        output
    }
}
