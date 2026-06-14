use crate::{
    model::{
        WorkflowAuditStatus, WorkflowBlockedAction, WorkflowBlockersSnapshot, WorkflowGateSnapshot,
        WorkflowHealthSnapshot, WorkflowNextAction, WorkflowNextActionsSnapshot, WorkflowStage,
        STATE_BLOCKERS_VERSION, STATE_NEXT_ACTIONS_VERSION, STATE_WORKFLOW_GATES_VERSION,
    },
    readiness::issue_readiness_blockers,
    storage::{read_json, sorted_child_paths, unix_timestamp_seconds, write_json},
};
use agentflow_execute::{ExecutePreflight, ExecuteRun, ExecuteRunStatus};
use agentflow_input::issue::{validate_agent_claim, AgentClaim, InputIssue, InputIssueStatus};
use agentflow_projection::IssueStatusIndex;
use agentflow_spec::{SpecIssue, SpecIssueStatus};
use anyhow::Result;
use std::collections::BTreeMap;
use std::path::Path;

pub(crate) fn build_gate_snapshot(
    root: &Path,
    health: &[WorkflowHealthSnapshot],
) -> Result<WorkflowGateSnapshot> {
    let input = agentflow_input::load_input_snapshot(root).ok();
    let execute = agentflow_execute::load_execute_snapshot(root).ok();
    let output = agentflow_output::load_output_snapshot(root).ok();
    let spec_issues = agentflow_spec::list_spec_issues(root).unwrap_or_default();
    let projection_index = agentflow_projection::load_issue_status_index(root).ok();
    let audit_index = agentflow_output::load_audit_index(root).ok();
    let audit_status = derive_audit_status(audit_index.as_ref());
    let active_run = execute.as_ref().and_then(|snapshot| {
        snapshot
            .index
            .runs
            .iter()
            .find(|run| run_is_active(&run.status))
    });
    let active_run_id = active_run.map(|run| run.run_id.clone());
    let latest_evidence = output
        .as_ref()
        .and_then(|snapshot| snapshot.index.evidence.last())
        .map(|entry| entry.path.clone());
    let latest_delivery = output
        .as_ref()
        .and_then(|snapshot| snapshot.index.release_deliveries.last())
        .map(|entry| entry.path.clone());
    let mut blockers =
        issue_readiness_blockers(root, input.as_ref(), execute.as_ref(), output.as_ref())
            .into_iter()
            .map(|blocker| {
                let _issue_id = blocker.issue_id;
                WorkflowBlockedAction {
                    action: blocker.action,
                    reason: blocker.reason,
                    source_path: blocker.source_path,
                }
            })
            .collect::<Vec<_>>();
    blockers.extend(collect_blockers(root)?);
    if let Some(input) = input.as_ref() {
        for issue in &input.issues {
            if !issue.target_metadata_complete() {
                blockers.push(WorkflowBlockedAction {
                    action: "copy-handoff".to_string(),
                    reason: "任务缺少执行目标，不能生成任务包".to_string(),
                    source_path: Some(issue.issue_path.clone()),
                });
            }
        }
    }
    let active_issue_id = active_run
        .map(|run| run.issue_id.clone())
        .or_else(|| active_ready_issue_id(input.as_ref(), &blockers))
        .or_else(|| first_ready_issue_id(input.as_ref()))
        .or_else(|| active_spec_issue_id(&spec_issues, projection_index.as_ref()));
    let has_execution_blockers = blockers_force_execute_blocked(input.as_ref(), &blockers);
    let current_stage = derive_stage(
        health,
        input.as_ref(),
        execute.as_ref(),
        output.as_ref(),
        &spec_issues,
        projection_index.as_ref(),
        &audit_status,
        has_execution_blockers,
    );
    let allowed_next_actions = allowed_actions(&current_stage);
    Ok(WorkflowGateSnapshot {
        version: STATE_WORKFLOW_GATES_VERSION.to_string(),
        current_stage,
        audit_status,
        active_issue_id,
        active_run_id,
        latest_evidence_path: latest_evidence,
        latest_delivery_path: latest_delivery,
        allowed_next_actions,
        blocked_actions: blockers,
        updated_at: unix_timestamp_seconds(),
    })
}

fn active_ready_issue_id(
    input: Option<&agentflow_input::InputSnapshot>,
    blockers: &[WorkflowBlockedAction],
) -> Option<String> {
    input.and_then(|snapshot| active_ready_issue_id_for_issues(&snapshot.issues, blockers))
}

fn active_ready_issue_id_for_issues(
    issues: &[InputIssue],
    blockers: &[WorkflowBlockedAction],
) -> Option<String> {
    issues
        .iter()
        .find(|issue| issue_is_unblocked_ready_for_execute(issue, blockers))
        .map(|issue| issue.issue_id.clone())
}

fn first_ready_issue_id(input: Option<&agentflow_input::InputSnapshot>) -> Option<String> {
    input.and_then(|snapshot| {
        snapshot
            .issues
            .iter()
            .find(|issue| matches!(issue.status, InputIssueStatus::Todo))
            .map(|issue| issue.issue_id.clone())
    })
}

fn active_spec_issue_id(
    issues: &[SpecIssue],
    projection_index: Option<&IssueStatusIndex>,
) -> Option<String> {
    projection_index
        .and_then(|index| {
            index
                .issues
                .iter()
                .find(|issue| {
                    matches!(
                        issue.current_state.as_str(),
                        "todo" | "in_progress" | "in_review"
                    )
                })
                .map(|issue| issue.issue_id.clone())
        })
        .or_else(|| {
            issues
                .iter()
                .find(|issue| {
                    matches!(
                        issue.status,
                        SpecIssueStatus::Todo
                            | SpecIssueStatus::InProgress
                            | SpecIssueStatus::InReview
                            | SpecIssueStatus::Backlog
                    )
                })
                .map(|issue| issue.issue_id.clone())
        })
}

fn blockers_force_execute_blocked(
    input: Option<&agentflow_input::InputSnapshot>,
    blockers: &[WorkflowBlockedAction],
) -> bool {
    if blockers.is_empty() {
        return false;
    }
    let Some(input) = input else {
        return true;
    };
    blockers_force_execute_blocked_for_issues(&input.issues, blockers)
}

fn blockers_force_execute_blocked_for_issues(
    issues: &[InputIssue],
    blockers: &[WorkflowBlockedAction],
) -> bool {
    let ready_issues = issues
        .iter()
        .filter(|issue| matches!(issue.status, InputIssueStatus::Todo))
        .collect::<Vec<_>>();
    if ready_issues.is_empty() {
        return blockers
            .iter()
            .any(|blocker| !blocker_is_issue_level(blocker));
    }
    !ready_issues
        .iter()
        .any(|issue| issue_is_unblocked_ready_for_execute(issue, blockers))
}

fn issue_is_unblocked_ready_for_execute(
    issue: &InputIssue,
    blockers: &[WorkflowBlockedAction],
) -> bool {
    matches!(issue.status, InputIssueStatus::Todo)
        && !issue.execution_risk.requires_human_confirmation()
        && !issue_has_gate_blocker(issue, blockers)
}

fn issue_has_gate_blocker(issue: &InputIssue, blockers: &[WorkflowBlockedAction]) -> bool {
    let issue_path = issue.issue_path.trim();
    if issue_path.is_empty() {
        return false;
    }
    blockers.iter().any(|blocker| {
        blocker_is_issue_level(blocker) && blocker.source_path.as_deref() == Some(issue_path)
    })
}

fn blocker_is_issue_level(blocker: &WorkflowBlockedAction) -> bool {
    matches!(blocker.action.as_str(), "dependency-ready" | "copy-handoff")
}

pub(crate) fn write_gate_files(root: &Path, gate: &WorkflowGateSnapshot) -> Result<()> {
    let next_actions = WorkflowNextActionsSnapshot {
        version: STATE_NEXT_ACTIONS_VERSION.to_string(),
        actions: build_next_actions(gate),
    };
    let blockers = WorkflowBlockersSnapshot {
        version: STATE_BLOCKERS_VERSION.to_string(),
        blockers: gate.blocked_actions.clone(),
    };
    write_json(&root.join(".agentflow/state/gates/workflow.json"), gate)?;
    write_json(
        &root.join(".agentflow/state/gates/next-actions.json"),
        &next_actions,
    )?;
    write_json(
        &root.join(".agentflow/state/gates/blockers.json"),
        &blockers,
    )
}

pub fn load_workflow_gates(project_root: impl AsRef<Path>) -> Result<WorkflowGateSnapshot> {
    let root = crate::storage::canonical_project_root(project_root)?;
    read_json(&root.join(".agentflow/state/gates/workflow.json"))
}

pub fn load_next_actions(project_root: impl AsRef<Path>) -> Result<WorkflowNextActionsSnapshot> {
    let root = crate::storage::canonical_project_root(project_root)?;
    read_json(&root.join(".agentflow/state/gates/next-actions.json"))
}

pub fn load_blockers(project_root: impl AsRef<Path>) -> Result<WorkflowBlockersSnapshot> {
    let root = crate::storage::canonical_project_root(project_root)?;
    read_json(&root.join(".agentflow/state/gates/blockers.json"))
}

fn derive_stage(
    health: &[WorkflowHealthSnapshot],
    input: Option<&agentflow_input::InputSnapshot>,
    execute: Option<&agentflow_execute::ExecuteSnapshot>,
    output: Option<&agentflow_output::OutputSnapshot>,
    spec_issues: &[SpecIssue],
    projection_index: Option<&IssueStatusIndex>,
    audit_status: &WorkflowAuditStatus,
    has_blockers: bool,
) -> WorkflowStage {
    if health
        .iter()
        .any(|item| item.module == "workspace" && item.status == "missing")
    {
        return WorkflowStage::WorkspaceMissing;
    }
    if health
        .iter()
        .any(|item| item.module == "workspace" && item.status == "blocked")
    {
        return WorkflowStage::WorkspaceBlocked;
    }
    if health.iter().any(|item| item.status == "failed") {
        return WorkflowStage::Failed;
    }
    if matches!(
        audit_status,
        WorkflowAuditStatus::Passed
            | WorkflowAuditStatus::PassedWithWarnings
            | WorkflowAuditStatus::Failed
            | WorkflowAuditStatus::Cancelled
    ) {
        return WorkflowStage::AuditCompleted;
    }
    if matches!(audit_status, WorkflowAuditStatus::Requested) {
        return WorkflowStage::AuditRequested;
    }
    if matches!(audit_status, WorkflowAuditStatus::Running) {
        return WorkflowStage::AuditRunning;
    }
    if output
        .as_ref()
        .is_some_and(|snapshot| !snapshot.index.release_deliveries.is_empty())
    {
        return WorkflowStage::DeliveryReady;
    }
    if output
        .as_ref()
        .is_some_and(|snapshot| !snapshot.index.evidence.is_empty())
    {
        return WorkflowStage::EvidenceReady;
    }
    if execute.as_ref().is_some_and(|snapshot| {
        snapshot
            .index
            .runs
            .iter()
            .any(|run| matches!(run.status, ExecuteRunStatus::Completed))
    }) {
        return WorkflowStage::ExecuteCompleted;
    }
    if projection_index.as_ref().is_some_and(|index| {
        index
            .issues
            .iter()
            .any(|issue| issue.current_state == "done")
    }) {
        return WorkflowStage::DeliveryReady;
    }
    if projection_index.as_ref().is_some_and(|index| {
        index
            .issues
            .iter()
            .any(|issue| issue.current_state == "in_review")
    }) {
        return WorkflowStage::ExecuteCompleted;
    }
    if projection_index.as_ref().is_some_and(|index| {
        index
            .issues
            .iter()
            .any(|issue| issue.current_state == "in_progress")
    }) {
        return WorkflowStage::ExecuteRunning;
    }
    if execute.as_ref().is_some_and(|snapshot| {
        snapshot
            .index
            .runs
            .iter()
            .any(|run| run_is_active(&run.status))
    }) {
        return WorkflowStage::ExecuteRunning;
    }
    if has_blockers
        || execute.as_ref().is_some_and(|snapshot| {
            snapshot
                .index
                .runs
                .iter()
                .any(|run| matches!(run.status, ExecuteRunStatus::Blocked))
        })
    {
        return WorkflowStage::ExecuteBlocked;
    }
    if input.as_ref().is_some_and(|snapshot| {
        snapshot.issues.iter().any(|issue| {
            matches!(issue.status, agentflow_input::issue::InputIssueStatus::Todo)
                && !issue.execution_risk.requires_human_confirmation()
        })
    }) {
        return WorkflowStage::ExecuteReady;
    }
    if projection_index.as_ref().is_some_and(|index| {
        index
            .issues
            .iter()
            .any(|issue| issue.current_state == "todo")
    }) || spec_issues
        .iter()
        .any(|issue| matches!(issue.status, SpecIssueStatus::Todo))
    {
        return WorkflowStage::ExecuteReady;
    }
    if input.as_ref().is_some_and(|snapshot| {
        snapshot.issues.iter().any(|issue| {
            matches!(issue.status, agentflow_input::issue::InputIssueStatus::Todo)
                && issue.execution_risk.requires_human_confirmation()
        })
    }) {
        return WorkflowStage::ExecuteBlocked;
    }
    if input
        .as_ref()
        .is_some_and(|snapshot| !snapshot.issues.is_empty())
    {
        return WorkflowStage::IssueReady;
    }
    if !spec_issues.is_empty()
        || projection_index
            .as_ref()
            .is_some_and(|index| !index.issues.is_empty())
    {
        return WorkflowStage::IssueReady;
    }
    if input
        .as_ref()
        .is_some_and(|snapshot| snapshot.status.summary.approved_specs > 0)
    {
        return WorkflowStage::InputReady;
    }
    if health
        .iter()
        .any(|item| item.module == "panel" && (item.status == "ready" || item.status == "degraded"))
    {
        return WorkflowStage::PanelReady;
    }
    if health
        .iter()
        .any(|item| item.module == "workspace" && item.ready)
        && health
            .iter()
            .any(|item| item.module == "define" && item.ready)
    {
        WorkflowStage::WorkspaceReady
    } else {
        WorkflowStage::WorkspaceMissing
    }
}

fn derive_audit_status(index: Option<&agentflow_output::AuditIndex>) -> WorkflowAuditStatus {
    let Some(index) = index else {
        return WorkflowAuditStatus::NotRequested;
    };
    let Some(latest) = index.audits.iter().max_by_key(|entry| entry.requested_at) else {
        return WorkflowAuditStatus::NotRequested;
    };
    match latest.status {
        agentflow_output::AuditStatus::Requested => WorkflowAuditStatus::Requested,
        agentflow_output::AuditStatus::Running => WorkflowAuditStatus::Running,
        agentflow_output::AuditStatus::Passed => WorkflowAuditStatus::Passed,
        agentflow_output::AuditStatus::PassedWithWarnings => {
            WorkflowAuditStatus::PassedWithWarnings
        }
        agentflow_output::AuditStatus::Failed => WorkflowAuditStatus::Failed,
        agentflow_output::AuditStatus::Cancelled => WorkflowAuditStatus::Cancelled,
    }
}

fn allowed_actions(stage: &WorkflowStage) -> Vec<String> {
    let mut actions = Vec::new();
    match stage {
        WorkflowStage::DeliveryReady => actions.push("start-new-input".to_string()),
        WorkflowStage::EvidenceReady => actions.push("prepare-release-delivery".to_string()),
        WorkflowStage::ExecuteCompleted => actions.push("write-output-evidence".to_string()),
        WorkflowStage::IssueReady | WorkflowStage::ExecuteReady => {
            actions.push("execute-issue".to_string());
        }
        WorkflowStage::InputReady | WorkflowStage::PanelReady | WorkflowStage::WorkspaceReady => {
            actions.push("start-new-input".to_string());
        }
        _ => {}
    }
    actions
}

fn build_next_actions(gate: &WorkflowGateSnapshot) -> Vec<WorkflowNextAction> {
    let mut actions = gate
        .allowed_next_actions
        .iter()
        .map(|action| WorkflowNextAction {
            action: action.clone(),
            label: action_label(action),
            allowed: true,
            reason: allowed_reason(action, gate),
        })
        .collect::<Vec<_>>();
    actions.extend(
        gate.blocked_actions
            .iter()
            .map(|blocker| WorkflowNextAction {
                action: blocker.action.clone(),
                label: action_label(&blocker.action),
                allowed: false,
                reason: blocker.reason.clone(),
            }),
    );
    actions
}

fn collect_blockers(root: &Path) -> Result<Vec<WorkflowBlockedAction>> {
    let mut blockers = Vec::new();
    let latest_run_by_issue = latest_run_ids_by_issue(root)?;
    for run_dir in sorted_child_paths(&root.join(".agentflow/execute/runs"))? {
        let run_path = run_dir.join("run.json");
        let claim_path = run_dir.join("agent-claim.json");
        let run = run_path
            .is_file()
            .then(|| read_json::<ExecuteRun>(&run_path))
            .transpose()
            .ok()
            .flatten();
        let is_latest_issue_run = run.as_ref().is_some_and(|run| {
            latest_run_by_issue
                .get(&run.issue_id)
                .is_some_and(|latest_run_id| latest_run_id == &run.run_id)
        });
        if is_latest_issue_run {
            if let Some(run) = run.as_ref() {
                let issue = read_json(&root.join(&run.input.issue_path));
                let claim = read_json::<AgentClaim>(&claim_path);
                match (issue, claim) {
                    (Ok(issue), Ok(claim)) => {
                        if let Err(error) = validate_agent_claim(&issue, &claim) {
                            blockers.push(WorkflowBlockedAction {
                                action: "agent-role-mismatch".to_string(),
                                reason: format!("Agent 角色不匹配：{error}"),
                                source_path: Some(format!(
                                    ".agentflow/execute/runs/{}/agent-claim.json",
                                    run.run_id
                                )),
                            });
                        }
                    }
                    (Ok(_), Err(error)) => blockers.push(WorkflowBlockedAction {
                        action: "agent-role-mismatch".to_string(),
                        reason: format!("Agent 写回缺少 agent-claim.json：{error}"),
                        source_path: Some(format!(
                            ".agentflow/execute/runs/{}/agent-claim.json",
                            run.run_id
                        )),
                    }),
                    (Err(error), _) => blockers.push(WorkflowBlockedAction {
                        action: "agent-role-mismatch".to_string(),
                        reason: format!("Agent 写回无法关联 Issue：{error}"),
                        source_path: Some(format!(
                            ".agentflow/execute/runs/{}/run.json",
                            run.run_id
                        )),
                    }),
                }
            }
        }

        let preflight_path = run_dir.join("preflight.json");
        if !preflight_path.is_file() {
            continue;
        }
        let Ok(preflight) = read_json::<ExecutePreflight>(&preflight_path) else {
            continue;
        };
        if !run
            .as_ref()
            .is_some_and(|run| run.run_id == preflight.run_id && is_latest_issue_run)
        {
            continue;
        }
        for check in preflight
            .checks
            .iter()
            .filter(|check| matches!(check.status, agentflow_execute::ExecuteCheckStatus::Blocked))
        {
            blockers.push(WorkflowBlockedAction {
                action: "execute-issue".to_string(),
                reason: check
                    .message
                    .clone()
                    .unwrap_or_else(|| format!("{} blocked", check.name)),
                source_path: Some(format!(
                    ".agentflow/execute/runs/{}/preflight.json",
                    preflight.run_id
                )),
            });
        }
    }
    Ok(blockers)
}

fn latest_run_ids_by_issue(root: &Path) -> Result<BTreeMap<String, String>> {
    let mut latest_runs = BTreeMap::<String, ExecuteRun>::new();
    for run_dir in sorted_child_paths(&root.join(".agentflow/execute/runs"))? {
        let run_path = run_dir.join("run.json");
        if !run_path.is_file() {
            continue;
        }
        let Ok(run) = read_json::<ExecuteRun>(&run_path) else {
            continue;
        };
        let replace = match latest_runs.get(&run.issue_id) {
            Some(existing) => {
                (run.updated_at, run.run_id.as_str())
                    > (existing.updated_at, existing.run_id.as_str())
            }
            None => true,
        };
        if replace {
            latest_runs.insert(run.issue_id.clone(), run);
        }
    }
    Ok(latest_runs
        .into_iter()
        .map(|(issue_id, run)| (issue_id, run.run_id))
        .collect())
}

fn run_is_active(status: &ExecuteRunStatus) -> bool {
    matches!(
        status,
        ExecuteRunStatus::Queued
            | ExecuteRunStatus::Preflight
            | ExecuteRunStatus::Planned
            | ExecuteRunStatus::Checkpointed
            | ExecuteRunStatus::Patching
            | ExecuteRunStatus::Running
            | ExecuteRunStatus::Validating
    )
}

fn action_label(action: &str) -> String {
    match action {
        "copy-handoff" => "Copy Agent handoff package",
        "start-new-input" => "Start new requirement intake",
        "prepare-release-delivery" => "Prepare release delivery",
        "write-output-evidence" => "Write output evidence",
        "execute-issue" => "Execute issue",
        _ => action,
    }
    .to_string()
}

fn allowed_reason(action: &str, gate: &WorkflowGateSnapshot) -> String {
    match action {
        "start-new-input" => "Workflow can accept the next requirement intake.".to_string(),
        "prepare-release-delivery" => {
            "Evidence is ready for Build Agent delivery material.".to_string()
        }
        "write-output-evidence" => {
            "Execute run is completed and can produce output evidence.".to_string()
        }
        "execute-issue" => gate
            .active_issue_id
            .as_ref()
            .map(|issue_id| format!("Issue {issue_id} is ready for controlled execute."))
            .unwrap_or_else(|| "An issue is ready for controlled execute.".to_string()),
        _ => "Action is allowed by workflow gate.".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agentflow_input::issue::{InputIssue, InputIssueStatus, InputRiskLevel};

    fn issue(issue_id: &str, status: InputIssueStatus) -> InputIssue {
        InputIssue {
            issue_id: issue_id.to_string(),
            issue_path: format!(".agentflow/input/issues/{issue_id}.json"),
            source_spec_id: "spec-001".to_string(),
            title: issue_id.to_string(),
            summary: issue_id.to_string(),
            status,
            execution_risk: InputRiskLevel::Low,
            ..InputIssue::default()
        }
    }

    fn issue_blocker(issue_id: &str, action: &str) -> WorkflowBlockedAction {
        WorkflowBlockedAction {
            action: action.to_string(),
            reason: "blocked".to_string(),
            source_path: Some(format!(".agentflow/input/issues/{issue_id}.json")),
        }
    }

    #[test]
    fn downstream_issue_blocker_keeps_unblocked_ready_path_executable() {
        let issues = vec![
            issue("AF-001", InputIssueStatus::Todo),
            issue("AF-002", InputIssueStatus::Todo),
        ];
        let blockers = vec![issue_blocker("AF-002", "dependency-ready")];

        assert_eq!(
            active_ready_issue_id_for_issues(&issues, &blockers).as_deref(),
            Some("AF-001")
        );
        assert!(!blockers_force_execute_blocked_for_issues(
            &issues, &blockers
        ));
    }

    #[test]
    fn blockers_force_execute_blocked_when_all_ready_paths_are_blocked() {
        let issues = vec![issue("AF-001", InputIssueStatus::Todo)];
        let blockers = vec![issue_blocker("AF-001", "dependency-ready")];

        assert!(active_ready_issue_id_for_issues(&issues, &blockers).is_none());
        assert!(blockers_force_execute_blocked_for_issues(
            &issues, &blockers
        ));
    }
}
