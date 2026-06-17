use crate::{
    model::{
        WorkflowAuditStatus, WorkflowBlockedAction, WorkflowBlockersSnapshot, WorkflowGateSnapshot,
        WorkflowHealthSnapshot, WorkflowNextAction, WorkflowNextActionsSnapshot, WorkflowStage,
        STATE_BLOCKERS_VERSION, STATE_NEXT_ACTIONS_VERSION, STATE_WORKFLOW_GATES_VERSION,
    },
    readiness::{issue_has_readiness_blocker, issue_readiness_blockers},
    storage::{read_json, unix_timestamp_seconds, write_json},
};
use agentflow_projection::IssueStatusIndex;
use agentflow_spec::{SpecIssue, SpecIssueStatus};
use agentflow_workflow_core::{
    work_state_is_active, work_state_is_done, work_state_is_in_progress, work_state_is_in_review,
    work_state_is_ready_for_execution,
};
use anyhow::Result;
use std::path::Path;

pub(crate) fn build_gate_snapshot(
    root: &Path,
    health: &[WorkflowHealthSnapshot],
) -> Result<WorkflowGateSnapshot> {
    let _ = agentflow_projection::rebuild_projections(root);
    let spec_issues = agentflow_spec::list_spec_issues(root).unwrap_or_default();
    let projection_index = agentflow_projection::load_issue_status_index(root).ok();
    let audit_index = agentflow_audit::load_audit_index(root).ok();
    let audit_status = derive_audit_status(audit_index.as_ref());
    let latest_projection = projection_index.as_ref().and_then(|index| {
        index
            .issues
            .iter()
            .max_by_key(|issue| (issue.updated_at, issue.issue_id.as_str()))
            .and_then(|issue| {
                agentflow_projection::load_task_projection(root, &issue.issue_id).ok()
            })
    });
    let active_run_id = latest_projection
        .as_ref()
        .and_then(|projection| projection.latest_run_id.clone());
    let latest_evidence = latest_projection
        .as_ref()
        .and_then(|projection| projection.public_delivery.evidence_path.clone());
    let latest_delivery = latest_projection.as_ref().and_then(|projection| {
        projection
            .public_delivery
            .changelog_path
            .clone()
            .or(projection.public_delivery.release_notes_url.clone())
            .or(projection.public_delivery.pr_url.clone())
    });
    let blockers = issue_readiness_blockers(root, &spec_issues)
        .into_iter()
        .map(|blocker| WorkflowBlockedAction {
            action: blocker.action,
            reason: blocker.reason,
            source_path: blocker.source_path,
        })
        .collect::<Vec<_>>();
    let active_issue_id = active_ready_issue_id(&spec_issues, projection_index.as_ref(), &blockers)
        .or_else(|| active_spec_issue_id(&spec_issues, projection_index.as_ref()));
    let has_execution_blockers =
        blockers_force_task_blocked(&spec_issues, projection_index.as_ref(), &blockers);
    let current_stage = derive_stage(
        health,
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
    issues: &[SpecIssue],
    projection_index: Option<&IssueStatusIndex>,
    blockers: &[WorkflowBlockedAction],
) -> Option<String> {
    issues
        .iter()
        .find(|issue| issue_is_unblocked_ready_for_task(issue, projection_index, blockers))
        .map(|issue| issue.issue_id.clone())
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
                .find(|issue| work_state_is_active(&issue.current_state))
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

fn blockers_force_task_blocked(
    issues: &[SpecIssue],
    projection_index: Option<&IssueStatusIndex>,
    blockers: &[WorkflowBlockedAction],
) -> bool {
    if blockers.is_empty() {
        return false;
    }
    let ready_issues = issues
        .iter()
        .filter(|issue| {
            work_state_is_ready_for_execution(projected_status(issue, projection_index))
        })
        .collect::<Vec<_>>();
    if ready_issues.is_empty() {
        return blockers
            .iter()
            .any(|blocker| !blocker_is_issue_level(blocker));
    }
    !ready_issues
        .iter()
        .any(|issue| !issue_has_gate_blocker(issue, blockers))
}

fn issue_is_unblocked_ready_for_task(
    issue: &SpecIssue,
    projection_index: Option<&IssueStatusIndex>,
    blockers: &[WorkflowBlockedAction],
) -> bool {
    work_state_is_ready_for_execution(projected_status(issue, projection_index))
        && !issue_has_gate_blocker(issue, blockers)
}

fn issue_has_gate_blocker(issue: &SpecIssue, blockers: &[WorkflowBlockedAction]) -> bool {
    issue_has_readiness_blocker(&issue.system.path, blockers)
}

fn blocker_is_issue_level(blocker: &WorkflowBlockedAction) -> bool {
    matches!(blocker.action.as_str(), "dependency-ready")
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
    if projection_index.as_ref().is_some_and(|index| {
        index
            .issues
            .iter()
            .any(|issue| work_state_is_done(&issue.current_state))
    }) {
        return WorkflowStage::DeliveryReady;
    }
    if projection_index.as_ref().is_some_and(|index| {
        index
            .issues
            .iter()
            .any(|issue| work_state_is_in_review(&issue.current_state))
    }) {
        return WorkflowStage::ExecuteCompleted;
    }
    if projection_index.as_ref().is_some_and(|index| {
        index
            .issues
            .iter()
            .any(|issue| work_state_is_in_progress(&issue.current_state))
    }) {
        return WorkflowStage::ExecuteRunning;
    }
    if has_blockers {
        return WorkflowStage::ExecuteBlocked;
    }
    if projection_index.as_ref().is_some_and(|index| {
        index
            .issues
            .iter()
            .any(|issue| work_state_is_ready_for_execution(&issue.current_state))
    }) || spec_issues
        .iter()
        .any(|issue| matches!(issue.status, SpecIssueStatus::Todo))
    {
        return WorkflowStage::ExecuteReady;
    }
    if !spec_issues.is_empty()
        || projection_index
            .as_ref()
            .is_some_and(|index| !index.issues.is_empty())
    {
        return WorkflowStage::IssueReady;
    }
    if health
        .iter()
        .any(|item| item.module == "panel" && item.status == "ready")
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

fn derive_audit_status(index: Option<&agentflow_audit::AuditIndex>) -> WorkflowAuditStatus {
    let Some(index) = index else {
        return WorkflowAuditStatus::NotRequested;
    };
    let Some(latest) = index.audits.iter().max_by_key(|entry| entry.requested_at) else {
        return WorkflowAuditStatus::NotRequested;
    };
    match latest.status {
        agentflow_audit::AuditStatus::Requested => WorkflowAuditStatus::Requested,
        agentflow_audit::AuditStatus::Running => WorkflowAuditStatus::Running,
        agentflow_audit::AuditStatus::Passed => WorkflowAuditStatus::Passed,
        agentflow_audit::AuditStatus::PassedWithWarnings => WorkflowAuditStatus::PassedWithWarnings,
        agentflow_audit::AuditStatus::Failed => WorkflowAuditStatus::Failed,
        agentflow_audit::AuditStatus::Cancelled => WorkflowAuditStatus::Cancelled,
    }
}

fn allowed_actions(stage: &WorkflowStage) -> Vec<String> {
    let mut actions = Vec::new();
    match stage {
        WorkflowStage::DeliveryReady => actions.push("start-new-requirement".to_string()),
        WorkflowStage::EvidenceReady | WorkflowStage::ExecuteCompleted => {
            actions.push("prepare-public-delivery".to_string());
        }
        WorkflowStage::IssueReady | WorkflowStage::ExecuteReady => {
            actions.push("execute-issue".to_string());
        }
        WorkflowStage::PanelReady | WorkflowStage::WorkspaceReady => {
            actions.push("start-new-requirement".to_string());
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

fn action_label(action: &str) -> String {
    match action {
        "start-new-requirement" => "Start new requirement",
        "prepare-public-delivery" => "Prepare public delivery",
        "execute-issue" => "Execute issue",
        _ => action,
    }
    .to_string()
}

fn allowed_reason(action: &str, gate: &WorkflowGateSnapshot) -> String {
    match action {
        "start-new-requirement" => "Workflow can accept the next requirement.".to_string(),
        "prepare-public-delivery" => {
            "Task evidence is ready for public delivery record.".to_string()
        }
        "execute-issue" => gate
            .active_issue_id
            .as_ref()
            .map(|issue_id| format!("Issue {issue_id} is ready for task loop."))
            .unwrap_or_else(|| "An issue is ready for task loop.".to_string()),
        _ => "Action is allowed by workflow gate.".to_string(),
    }
}

fn projected_status<'a>(
    issue: &'a SpecIssue,
    projection_index: Option<&'a IssueStatusIndex>,
) -> &'a str {
    projection_index
        .and_then(|index| {
            index
                .issues
                .iter()
                .find(|entry| entry.issue_id == issue.issue_id)
        })
        .map(|entry| entry.current_state.as_str())
        .unwrap_or_else(|| issue.status.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;
    use agentflow_spec::{SpecIssueDraft, SpecIssueStatus};
    use tempfile::tempdir;

    fn issue(root: &Path, issue_id: &str, status: SpecIssueStatus) -> SpecIssue {
        let requirement = root.join(format!("docs/requirements/{issue_id}.md"));
        std::fs::create_dir_all(requirement.parent().unwrap()).unwrap();
        std::fs::write(&requirement, format!("# {issue_id}\n")).unwrap();
        let mut draft = SpecIssueDraft::new(issue_id);
        draft.title = Some(issue_id.to_string());
        let mut issue = agentflow_spec::issue_from_requirement(root, &requirement, draft).unwrap();
        issue.status = status;
        issue
    }

    fn issue_blocker(issue: &SpecIssue, action: &str) -> WorkflowBlockedAction {
        WorkflowBlockedAction {
            action: action.to_string(),
            reason: "blocked".to_string(),
            source_path: Some(issue.system.path.clone()),
        }
    }

    #[test]
    fn downstream_issue_blocker_keeps_unblocked_ready_path_executable() {
        let dir = tempdir().unwrap();
        let first = issue(dir.path(), "AF-001", SpecIssueStatus::Todo);
        let second = issue(dir.path(), "AF-002", SpecIssueStatus::Todo);
        let issues = vec![first.clone(), second.clone()];
        let blockers = vec![issue_blocker(&second, "dependency-ready")];

        assert_eq!(
            active_ready_issue_id(&issues, None, &blockers).as_deref(),
            Some("AF-001")
        );
        assert!(!blockers_force_task_blocked(&issues, None, &blockers));
    }

    #[test]
    fn blockers_force_task_blocked_when_all_ready_paths_are_blocked() {
        let dir = tempdir().unwrap();
        let first = issue(dir.path(), "AF-001", SpecIssueStatus::Todo);
        let issues = vec![first.clone()];
        let blockers = vec![issue_blocker(&first, "dependency-ready")];

        assert!(active_ready_issue_id(&issues, None, &blockers).is_none());
        assert!(blockers_force_task_blocked(&issues, None, &blockers));
    }
}
