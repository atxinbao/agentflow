use crate::{
    model::{
        WorkflowAuditStatus, WorkflowBlockedAction, WorkflowBlockersSnapshot, WorkflowGateSnapshot,
        WorkflowHealthSnapshot, WorkflowNextAction, WorkflowNextActionsSnapshot, WorkflowStage,
        STATE_BLOCKERS_VERSION, STATE_NEXT_ACTIONS_VERSION, STATE_WORKFLOW_GATES_VERSION,
    },
    readiness::{issue_has_readiness_blocker, issue_readiness_blockers},
    storage::{read_json, unix_timestamp_seconds, write_json},
};
use agentflow_projection::{IssueStatusIndex, RequirementPreviewIndex};
use agentflow_spec::{list_spec_projects, read_project_brain_snapshot, SpecIssue, SpecProject};
use agentflow_workflow_core::{
    work_state_is_active, work_state_is_done, work_state_is_in_progress, work_state_is_in_review,
    work_state_is_ready_for_execution,
};
use anyhow::Result;
use std::{fs, path::Path};

pub(crate) fn build_gate_snapshot(
    root: &Path,
    health: &[WorkflowHealthSnapshot],
) -> Result<WorkflowGateSnapshot> {
    let _ = agentflow_projection::rebuild_projections(root);
    let spec_issues = agentflow_spec::list_spec_issues(root).unwrap_or_default();
    let spec_projects = list_spec_projects(root).unwrap_or_default();
    let projection_index = agentflow_projection::load_issue_status_index(root).ok();
    let requirement_preview_index =
        agentflow_projection::load_requirement_preview_index(root).ok();
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
        &spec_projects,
        projection_index.as_ref(),
        requirement_preview_index.as_ref(),
        &audit_status,
        has_execution_blockers,
        root,
    );
    let project_brain_action = project_brain_next_action(root, &spec_projects);
    let requirement_preview_action =
        requirement_preview_next_action(requirement_preview_index.as_ref());
    let allowed_next_actions = allowed_actions(
        &current_stage,
        requirement_preview_action.as_deref(),
        project_brain_action.as_deref(),
    );
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
                        projected_status(issue, projection_index),
                        "todo" | "in_progress" | "in_review" | "backlog"
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
    spec_projects: &[SpecProject],
    projection_index: Option<&IssueStatusIndex>,
    requirement_preview_index: Option<&RequirementPreviewIndex>,
    audit_status: &WorkflowAuditStatus,
    has_blockers: bool,
    root: &Path,
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
        .any(|issue| work_state_is_ready_for_execution(projected_status(issue, projection_index)))
    {
        return WorkflowStage::ExecuteReady;
    }
    if projection_index.as_ref().is_some_and(|index| {
        index
            .issues
            .iter()
            .any(|issue| work_state_is_in_review(&issue.current_state))
    }) {
        return WorkflowStage::ExecuteCompleted;
    }
    if has_completion_candidate(root) {
        return WorkflowStage::CompletionReady;
    }
    if projection_index.as_ref().is_some_and(|index| {
        index
            .issues
            .iter()
            .any(|issue| work_state_is_done(&issue.current_state))
    }) {
        return WorkflowStage::DeliveryReady;
    }
    if has_requirement_preview_runtime_entry(requirement_preview_index)
        || has_project_brain_runtime_entry(root, spec_projects)
    {
        return WorkflowStage::InputReady;
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

fn has_completion_candidate(root: &Path) -> bool {
    let projection_dir = root.join(".agentflow/projections/projects");
    let Ok(entries) = fs::read_dir(&projection_dir) else {
        return false;
    };

    entries
        .filter_map(std::result::Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("json"))
        .filter_map(|path| fs::read_to_string(path).ok())
        .filter_map(|raw| {
            serde_json::from_str::<agentflow_projection::ProjectProjection>(&raw).ok()
        })
        .any(|projection| {
            projection
                .completion
                .as_ref()
                .is_some_and(|completion| completion.current_state == "goal-recheck")
                || (projection.issue_count > 0
                    && projection.completed_issue_count == projection.issue_count
                    && projection.status != "done")
        })
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

fn allowed_actions(
    stage: &WorkflowStage,
    requirement_preview_action: Option<&str>,
    project_brain_action: Option<&str>,
) -> Vec<String> {
    let mut actions = Vec::new();
    match stage {
        WorkflowStage::DeliveryReady => actions.push("prepare-public-delivery".to_string()),
        WorkflowStage::CompletionReady => actions.push("enter-completion-decision".to_string()),
        WorkflowStage::EvidenceReady | WorkflowStage::ExecuteCompleted => {
            actions.push("prepare-public-delivery".to_string());
        }
        WorkflowStage::IssueReady | WorkflowStage::ExecuteReady => {
            actions.push("execute-issue".to_string());
        }
        WorkflowStage::InputReady => actions.push(
            requirement_preview_action
                .or(project_brain_action)
                .unwrap_or("start-project-loop")
                .to_string(),
        ),
        WorkflowStage::PanelReady | WorkflowStage::WorkspaceReady => {
            actions.push("start-new-requirement".to_string());
        }
        _ => {}
    }
    actions
}

fn has_project_brain_runtime_entry(root: &Path, spec_projects: &[SpecProject]) -> bool {
    project_brain_next_action(root, spec_projects).is_some()
}

fn has_requirement_preview_runtime_entry(
    requirement_preview_index: Option<&RequirementPreviewIndex>,
) -> bool {
    requirement_preview_index.is_some_and(|index| {
        index
            .previews
            .iter()
            .any(|preview| preview.lifecycle == "active")
    })
}

fn requirement_preview_next_action(
    requirement_preview_index: Option<&RequirementPreviewIndex>,
) -> Option<String> {
    requirement_preview_index.and_then(|index| {
        index
            .previews
            .iter()
            .filter(|preview| preview.lifecycle == "active")
            .max_by_key(|preview| (preview.updated_at, preview.requirement_id.as_str()))
            .map(|preview| preview.next_recommended_action.clone())
    })
}

fn project_brain_next_action(root: &Path, spec_projects: &[SpecProject]) -> Option<String> {
    spec_projects.iter().find_map(|project| {
        read_project_brain_snapshot(root, &project.project_id, &project.title)
            .ok()
            .map(|snapshot| snapshot.next_recommended_action)
    })
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
        "enter-completion-decision" => "进入完成判断",
        "start-project-loop" => "进入项目循环",
        "create-goal-draft-preview" => "生成 Goal 草稿预览",
        "create-plan-draft-preview" => "生成 Plan 草稿预览",
        "confirm-goal-draft-preview" => "确认 Goal 草稿预览",
        "confirm-plan-draft-preview" => "确认 Plan 草稿预览",
        "confirm-project-brain" => "确认 Project Brain",
        "materialize-spec-project-and-issues" => "物化 SpecProject / SpecIssue",
        "run-goal-recheck" => "重新检查项目目标",
        "resolve-project-brain-blocker" => "处理 Project Brain 阻断",
        "start-new-requirement" => "开始新需求",
        "prepare-public-delivery" => "准备公开交付记录",
        "execute-issue" => "执行任务",
        _ => action,
    }
    .to_string()
}

fn allowed_reason(action: &str, gate: &WorkflowGateSnapshot) -> String {
    match action {
        "enter-completion-decision" => {
            "当前任务已经全部完成，下一步由 Goal Agent 做正式完成判断。".to_string()
        }
        "start-project-loop" => {
            "Project Brain 已确认，可以进入项目循环。".to_string()
        }
        "create-goal-draft-preview" => {
            "当前还没有确认 Goal 文档，先生成 Goal 草稿预览。".to_string()
        }
        "create-plan-draft-preview" => {
            "当前已有 Goal，但还缺 Plan 文档。".to_string()
        }
        "confirm-goal-draft-preview" => {
            "Requirement 已被整理成 Goal 草稿，等待用户确认。".to_string()
        }
        "confirm-plan-draft-preview" => {
            "Goal 已确认，当前等待确认 Plan 草稿。".to_string()
        }
        "confirm-project-brain" => {
            "Project Brain 文档已存在，但还没有全部确认。".to_string()
        }
        "materialize-spec-project-and-issues" => {
            "Goal / Plan 预览都已确认，可以物化成 SpecProject / SpecIssue。".to_string()
        }
        "run-goal-recheck" => {
            "Project Health 提示目标或计划需要重新检查。".to_string()
        }
        "resolve-project-brain-blocker" => {
            "Project Brain 存在阻断，先修复再继续。".to_string()
        }
        "start-new-requirement" => "当前工作流可以接受下一条需求。".to_string(),
        "prepare-public-delivery" => {
            "任务证据已就绪，可以整理公开交付记录。".to_string()
        }
        "execute-issue" => gate
            .active_issue_id
            .as_ref()
            .map(|issue_id| format!("任务 {issue_id} 已准备好进入执行循环。"))
            .unwrap_or_else(|| "当前有任务可以进入执行循环。".to_string()),
        _ => "当前动作已被工作流门禁允许。".to_string(),
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
    use agentflow_projection::{
        storage::write_project_projection, ProjectBrainProjection, ProjectCompletionProjection,
        ProjectIssueLanes, ProjectProjection, PROJECT_PROJECTION_VERSION,
    };
    use agentflow_spec::{
        requirement_preview_from_requirement, write_spec_project, SpecIssueDraft, SpecIssueStatus,
        SpecProjectDraft,
    };
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

    #[test]
    fn completion_candidate_promotes_completion_ready_stage() {
        let dir = tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".agentflow/projections/projects")).unwrap();
        write_project_projection(
            dir.path(),
            &ProjectProjection {
                version: PROJECT_PROJECTION_VERSION.to_string(),
                project_id: "project-001".to_string(),
                title: "Project".to_string(),
                objective: "Objective".to_string(),
                status: "active".to_string(),
                stage_key: "completion-ready".to_string(),
                stage_label: "等待完成判断".to_string(),
                stage_summary: "任务已全部完成，正在等待完成判断。".to_string(),
                issue_ids: vec!["AF-001".to_string()],
                current_issue_id: None,
                lanes: ProjectIssueLanes::default(),
                next_action: "进入完成判断".to_string(),
                next_action_label: "进入完成判断".to_string(),
                next_action_reason: "所有任务已经完成。".to_string(),
                blockers: Vec::new(),
                completion_hint: "全部任务已完成。".to_string(),
                completion: Some(ProjectCompletionProjection {
                    current_state: "goal-recheck".to_string(),
                    latest_outcome: None,
                    next_recommended_action: "enter-completion-decision".to_string(),
                    next_recommended_action_label: "进入完成判断".to_string(),
                    next_recommended_action_reason: "所有任务已经完成。".to_string(),
                    total_issue_count: 1,
                    completed_issue_count: 1,
                    canceled_issue_count: 0,
                    remaining_issue_count: 0,
                    blocked_issue_count: 0,
                    open_questions: vec!["是否接受当前交付？".to_string()],
                    rationale: vec!["当前任务已经全部完成。".to_string()],
                    updated_at: 1,
                }),
                issue_count: 1,
                completed_issue_count: 1,
                project_brain: ProjectBrainProjection {
                    project_path: "docs/projects/project-001.md".to_string(),
                    goal_path: "GOAL.md".to_string(),
                    plan_path: "PLAN.md".to_string(),
                    decisions_path: "DECISIONS.md".to_string(),
                    health_path: "PROJECT_HEALTH.md".to_string(),
                    brain_status: "ready".to_string(),
                    goal_status: "ready".to_string(),
                    plan_status: "ready".to_string(),
                    decision_status: "ready".to_string(),
                    health_status: "missing".to_string(),
                    missing_documents: Vec::new(),
                    open_questions: Vec::new(),
                    next_recommended_action: "进入完成判断".to_string(),
                    next_recommended_action_label: "进入完成判断".to_string(),
                    next_recommended_action_reason: "所有任务已经完成。".to_string(),
                    readonly: true,
                },
                updated_at: 1,
            },
        )
        .unwrap();

        let stage = derive_stage(
            &[],
            &[],
            &[],
            None,
            None,
            &WorkflowAuditStatus::NotRequested,
            false,
            dir.path(),
        );

        assert_eq!(stage, WorkflowStage::CompletionReady);
    }

    #[test]
    fn ready_issue_takes_precedence_over_finished_issue() {
        let dir = tempdir().unwrap();
        let index = IssueStatusIndex {
            version: "issue-status-index.v3".to_string(),
            updated_at: 1,
            issues: vec![
                agentflow_projection::IssueStatusIndexEntry {
                    issue_id: "AF-001".to_string(),
                    project_id: Some("project-001".to_string()),
                    title: "Done".to_string(),
                    current_state: "done".to_string(),
                    display_status: "done".to_string(),
                    workflow_ref: "build-agent.issue-loop@v1".to_string(),
                    projection_path: ".agentflow/projections/tasks/AF-001.json".to_string(),
                    updated_at: 1,
                },
                agentflow_projection::IssueStatusIndexEntry {
                    issue_id: "AF-002".to_string(),
                    project_id: Some("project-001".to_string()),
                    title: "Todo".to_string(),
                    current_state: "todo".to_string(),
                    display_status: "todo".to_string(),
                    workflow_ref: "build-agent.issue-loop@v1".to_string(),
                    projection_path: ".agentflow/projections/tasks/AF-002.json".to_string(),
                    updated_at: 2,
                },
            ],
        };

        let stage = derive_stage(
            &[],
            &[],
            &[],
            Some(&index),
            None,
            &WorkflowAuditStatus::NotRequested,
            false,
            dir.path(),
        );

        assert_eq!(stage, WorkflowStage::ExecuteReady);
    }

    #[test]
    fn confirmed_project_brain_without_issues_enters_input_ready_stage() {
        let dir = tempdir().unwrap();
        let requirement = dir.path().join("docs/requirements/039-test.md");
        std::fs::create_dir_all(requirement.parent().unwrap()).unwrap();
        std::fs::write(&requirement, "# Test\n\nProject brain runtime entry.\n").unwrap();

        let project = agentflow_spec::project_from_requirement(
            dir.path(),
            &requirement,
            SpecProjectDraft::new("project-brain-stage"),
        )
        .unwrap();
        write_spec_project(dir.path(), &project).unwrap();

        let project_docs = dir.path().join("docs/projects/project-brain-stage");
        std::fs::create_dir_all(&project_docs).unwrap();
        std::fs::write(project_docs.join("GOAL.md"), "# Goal\n\n已确认目标。\n").unwrap();
        std::fs::write(project_docs.join("PLAN.md"), "# Plan\n\n已确认计划。\n").unwrap();
        std::fs::write(
            project_docs.join("DECISIONS.md"),
            "# Decisions\n\n已确认。\n",
        )
        .unwrap();

        let stage = derive_stage(
            &[],
            &[],
            &[project],
            None,
            None,
            &WorkflowAuditStatus::NotRequested,
            false,
            dir.path(),
        );

        assert_eq!(stage, WorkflowStage::InputReady);
        assert_eq!(
            allowed_actions(
                &stage,
                None,
                project_brain_next_action(
                    dir.path(),
                    &agentflow_spec::list_spec_projects(dir.path()).unwrap()
                )
                .as_deref()
            ),
            vec!["start-project-loop".to_string()]
        );
    }

    #[test]
    fn active_requirement_preview_enters_input_ready_before_spec_materialization() {
        let dir = tempdir().unwrap();
        let requirement = dir.path().join("docs/requirements/040-preview.md");
        std::fs::create_dir_all(requirement.parent().unwrap()).unwrap();
        std::fs::write(&requirement, "# 预览\n\n先把需求整理成 Goal 和 Plan。\n").unwrap();

        requirement_preview_from_requirement(dir.path(), &requirement, Some("project-preview"))
            .unwrap();
        agentflow_projection::rebuild_projections(dir.path()).unwrap();
        let preview_index = agentflow_projection::load_requirement_preview_index(dir.path()).ok();

        let stage = derive_stage(
            &[],
            &[],
            &[],
            None,
            preview_index.as_ref(),
            &WorkflowAuditStatus::NotRequested,
            false,
            dir.path(),
        );

        assert_eq!(stage, WorkflowStage::InputReady);
        assert_eq!(
            allowed_actions(
                &stage,
                requirement_preview_next_action(preview_index.as_ref()).as_deref(),
                None
            ),
            vec!["confirm-goal-draft-preview".to_string()]
        );
    }
}
