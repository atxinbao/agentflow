use crate::{
    model::{
        IssueStatusIndex, IssueStatusIndexEntry, ProjectBrainProjection, ProjectProjection,
        ProjectionPhase, ProjectionPublicDelivery, ProjectionSummary, TaskProjection,
        TaskTimelineEvent, TaskTimelineItem, ISSUE_STATUS_INDEX_VERSION,
        PROJECT_PROJECTION_VERSION, TASK_PROJECTION_VERSION,
    },
    storage::{write_issue_status_index, write_project_projection, write_task_projection},
};
use agentflow_event_store::{load_task_events, EventStateTransition, TaskEvent};
use agentflow_spec::{
    prepare_spec_workspace, read_project_brain_snapshot, SpecIssue, SpecProject, SpecProjectStatus,
};
use anyhow::{Context, Result};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

const STATE_ORDER: [&str; 5] = ["backlog", "todo", "in_progress", "in_review", "done"];

pub fn rebuild_projections(project_root: impl AsRef<Path>) -> Result<ProjectionSummary> {
    let root = canonical_project_root(project_root)?;
    prepare_spec_workspace(&root)?;
    let issues = read_json_files::<SpecIssue>(&root.join(".agentflow/spec/issues"))?;
    let projects = read_json_files::<SpecProject>(&root.join(".agentflow/spec/projects"))?;
    let events = load_task_events(&root)?;
    let events_by_issue = group_events_by_issue(events);
    let mut task_projections = BTreeMap::new();

    for issue in &issues {
        let projection = project_issue(issue, events_by_issue.get(&issue.issue_id));
        write_task_projection(&root, &projection)?;
        task_projections.insert(issue.issue_id.clone(), projection);
    }

    for project in &projects {
        let projection = project_project(&root, project, &task_projections)?;
        write_project_projection(&root, &projection)?;
    }

    let mut index_entries = issues
        .iter()
        .filter_map(|issue| {
            task_projections
                .get(&issue.issue_id)
                .map(|projection| IssueStatusIndexEntry {
                    issue_id: issue.issue_id.clone(),
                    project_id: issue.project_id.clone(),
                    title: issue.title.clone(),
                    current_state: projection.current_state.clone(),
                    display_status: projection.display_status.clone(),
                    workflow_ref: issue.workflow_ref.clone(),
                    projection_path: format!(
                        ".agentflow/projections/tasks/{}.json",
                        issue.issue_id
                    ),
                    updated_at: projection.updated_at,
                })
        })
        .collect::<Vec<_>>();
    index_entries.sort_by(|left, right| left.issue_id.cmp(&right.issue_id));
    write_issue_status_index(
        &root,
        &IssueStatusIndex {
            version: ISSUE_STATUS_INDEX_VERSION.to_string(),
            updated_at: latest_update(&task_projections),
            issues: index_entries,
        },
    )?;

    Ok(ProjectionSummary {
        task_count: issues.len(),
        project_count: projects.len(),
        index_path: ".agentflow/indexes/issue-status.json".to_string(),
    })
}

fn project_issue(issue: &SpecIssue, events: Option<&Vec<TaskEvent>>) -> TaskProjection {
    let events = events.cloned().unwrap_or_default();
    let mut current_state = issue.status.as_str().to_string();
    let mut updated_at = issue.system.updated_at;
    let mut latest_run_id = None;
    let mut branch_name = None;
    let mut public_delivery = ProjectionPublicDelivery {
        evidence_path: Some(issue.expected_outputs.evidence_path.clone()),
        ..ProjectionPublicDelivery::default()
    };
    let mut state_events = Vec::new();

    for event in events {
        updated_at = updated_at.max(event.timestamp);
        if let Some(run_id) = event.payload.get("runId").and_then(Value::as_str) {
            latest_run_id = Some(run_id.to_string());
        }
        if let Some(branch) = event.payload.get("branchName").and_then(Value::as_str) {
            branch_name = Some(branch.to_string());
        }
        if let Some(pr_url) = event
            .payload
            .get("prUrl")
            .or_else(|| event.payload.get("remoteUrl"))
            .and_then(Value::as_str)
        {
            public_delivery.pr_url = Some(pr_url.to_string());
        }
        if let Some(merge_commit) = event.payload.get("mergeCommit").and_then(Value::as_str) {
            public_delivery.merge_commit = Some(merge_commit.to_string());
        }
        if let Some(changelog_path) = event.payload.get("changelogPath").and_then(Value::as_str) {
            public_delivery.changelog_path = Some(changelog_path.to_string());
        }
        if let Some(release_notes_url) =
            event.payload.get("releaseNotesUrl").and_then(Value::as_str)
        {
            public_delivery.release_notes_url = Some(release_notes_url.to_string());
        }
        if let Some(next_state) = event_to_state(&event) {
            current_state = next_state;
        }
        state_events.push((current_state.clone(), event));
    }

    TaskProjection {
        version: TASK_PROJECTION_VERSION.to_string(),
        issue_id: issue.issue_id.clone(),
        project_id: issue.project_id.clone(),
        workflow_ref: issue.workflow_ref.clone(),
        current_state: current_state.clone(),
        display_status: current_state.clone(),
        current_transition: state_events
            .last()
            .map(|(_, event)| event.event_type.clone()),
        latest_run_id,
        branch_name,
        timeline: build_timeline(issue, &current_state, &state_events),
        public_delivery,
        updated_at,
    }
}

fn project_project(
    root: &Path,
    project: &SpecProject,
    tasks: &BTreeMap<String, TaskProjection>,
) -> Result<ProjectProjection> {
    let mut current_issue_id = None;
    let mut completed = 0;
    let mut updated_at = project.system.updated_at;
    for issue_id in &project.issue_ids {
        let Some(task) = tasks.get(issue_id) else {
            continue;
        };
        updated_at = updated_at.max(task.updated_at);
        if task.current_state == "done" || task.current_state == "cancel" {
            completed += 1;
        } else if current_issue_id.is_none() {
            current_issue_id = Some(issue_id.clone());
        }
    }
    let status = if completed == project.issue_ids.len() && !project.issue_ids.is_empty() {
        "done"
    } else if current_issue_id.is_some() {
        "active"
    } else {
        project_status_as_str(&project.status)
    };
    let brain = read_project_brain_snapshot(root, &project.project_id, &project.title)?;
    Ok(ProjectProjection {
        version: PROJECT_PROJECTION_VERSION.to_string(),
        project_id: project.project_id.clone(),
        title: project.title.clone(),
        objective: project.objective.clone(),
        status: status.to_string(),
        issue_ids: project.issue_ids.clone(),
        current_issue_id,
        issue_count: project.issue_ids.len(),
        completed_issue_count: completed,
        project_brain: ProjectBrainProjection {
            project_path: brain.project_path,
            goal_path: brain.goal_document,
            plan_path: brain.plan_document,
            decisions_path: brain.decisions_document,
            brain_status: brain.brain_status.as_str().to_string(),
            goal_status: brain.goal_status.as_str().to_string(),
            plan_status: brain.plan_status.as_str().to_string(),
            decision_status: brain.decision_status.as_str().to_string(),
            missing_documents: brain.missing_documents,
            open_questions: brain.open_questions,
            next_recommended_action: brain.next_recommended_action,
            readonly: brain.readonly,
        },
        updated_at,
    })
}

fn project_status_as_str(status: &SpecProjectStatus) -> &'static str {
    match status {
        SpecProjectStatus::Planned => "planned",
        SpecProjectStatus::Active => "active",
        SpecProjectStatus::Done => "done",
        SpecProjectStatus::Blocked => "blocked",
        SpecProjectStatus::Cancel => "cancel",
    }
}

fn build_timeline(
    issue: &SpecIssue,
    current_state: &str,
    state_events: &[(String, TaskEvent)],
) -> Vec<TaskTimelineItem> {
    let mut states = STATE_ORDER
        .iter()
        .map(|state| state.to_string())
        .collect::<Vec<_>>();
    if matches!(current_state, "blocked" | "cancel") && !states.contains(&current_state.to_string())
    {
        states.push(current_state.to_string());
    }
    let current_index = states
        .iter()
        .position(|state| state == current_state)
        .unwrap_or(0);
    states
        .into_iter()
        .enumerate()
        .map(|(index, state)| {
            let matching_events = state_events
                .iter()
                .filter(|(event_state, _)| event_state == &state)
                .map(|(_, event)| event)
                .collect::<Vec<_>>();
            let phase = if matches!(state.as_str(), "blocked" | "cancel") {
                ProjectionPhase::Exception
            } else if index < current_index {
                ProjectionPhase::Past
            } else if index == current_index {
                ProjectionPhase::Current
            } else {
                ProjectionPhase::Future
            };
            TaskTimelineItem {
                state: state.clone(),
                phase,
                entered_at: matching_events.first().map(|event| event.timestamp),
                events: matching_events
                    .iter()
                    .map(|event| TaskTimelineEvent {
                        event_id: event.event_id.clone(),
                        event_type: event.event_type.clone(),
                        timestamp: event.timestamp,
                        actor_role: event.actor.role.clone(),
                        actor_kind: event.actor.kind.clone(),
                        summary: event_summary(event),
                        artifact_refs: event.artifact_refs.clone(),
                    })
                    .collect(),
                summary: state_summary(&state, issue),
                live_refs: matching_events
                    .iter()
                    .flat_map(|event| event.artifact_refs.clone())
                    .collect::<BTreeSet<_>>()
                    .into_iter()
                    .collect(),
            }
        })
        .collect()
}

fn event_to_state(event: &TaskEvent) -> Option<String> {
    if let Some(EventStateTransition { to_state, .. }) = event.state.as_ref() {
        return Some(to_state.clone());
    }
    match event.event_type.as_str() {
        "issue.scheduled" => Some("todo".to_string()),
        "agent.launch.requested"
        | "agent.session.created"
        | "agent.session.resumed"
        | "agent.session.running"
        | "agent.session.interrupted" => Some("in_progress".to_string()),
        "agent.session.in_review"
        | "issue.validation.passed"
        | "issue.review.requested"
        | "issue.pr.created"
        | "issue.merge.proof.recorded" => Some("in_review".to_string()),
        "issue.pr.merged" | "issue.completed" | "agent.session.completed" => {
            Some("done".to_string())
        }
        "issue.blocked" | "agent.session.failed" => Some("blocked".to_string()),
        "issue.cancelled" => Some("cancel".to_string()),
        _ => None,
    }
}

fn event_summary(event: &TaskEvent) -> String {
    match event.event_type.as_str() {
        "issue.scheduled" => "任务进入待执行队列。".to_string(),
        "agent.launch.requested" => "已生成 Build Agent 启动请求。".to_string(),
        "agent.session.created" => "外部执行会话已创建。".to_string(),
        "agent.session.resumed" => "外部执行会话已恢复。".to_string(),
        "agent.session.running" => "外部执行会话正在运行。".to_string(),
        "agent.session.interrupted" => "外部执行会话已中断，等待恢复。".to_string(),
        "agent.session.in_review" => "外部执行会话已进入评审。".to_string(),
        "agent.session.completed" => "外部执行会话已完成。".to_string(),
        "agent.session.failed" => "外部执行会话失败。".to_string(),
        "issue.validation.passed" => "本地沙箱验证已通过。".to_string(),
        "issue.review.requested" => "任务已请求评审。".to_string(),
        "issue.pr.created" => "PR/MR 已创建。".to_string(),
        "issue.merge.proof.recorded" => "合并证明已写入。".to_string(),
        "issue.pr.merged" => "PR/MR 已合并。".to_string(),
        "issue.completed" => "任务 Done 写回完成。".to_string(),
        "issue.blocked" => "任务进入阻断状态。".to_string(),
        "issue.cancelled" => "任务已取消。".to_string(),
        other => format!("记录事件：{other}。"),
    }
}

fn state_summary(state: &str, issue: &SpecIssue) -> String {
    match state {
        "backlog" => "任务已生成，等待调度。".to_string(),
        "todo" => "依赖满足，等待执行会话接管。".to_string(),
        "in_progress" => "任务正在执行，实时信息来自事件流。".to_string(),
        "in_review" => "验证完成，等待 PR/MR 合并。".to_string(),
        "done" => "任务已完成，公开交付记录应写入 PR/MR 或发布说明。".to_string(),
        "blocked" => format!("任务被阻断：{}", issue.title),
        "cancel" => "任务已取消。".to_string(),
        _ => "等待事件更新。".to_string(),
    }
}

fn group_events_by_issue(events: Vec<TaskEvent>) -> BTreeMap<String, Vec<TaskEvent>> {
    let mut grouped: BTreeMap<String, Vec<TaskEvent>> = BTreeMap::new();
    for event in events {
        if let Some(issue_id) = event.issue_id.clone() {
            grouped.entry(issue_id).or_default().push(event);
        }
    }
    grouped
}

fn latest_update(tasks: &BTreeMap<String, TaskProjection>) -> u64 {
    tasks
        .values()
        .map(|projection| projection.updated_at)
        .max()
        .unwrap_or(0)
}

fn read_json_files<T: serde::de::DeserializeOwned>(directory: &Path) -> Result<Vec<T>> {
    if !directory.exists() {
        return Ok(Vec::new());
    }
    let mut entries = fs::read_dir(directory)
        .with_context(|| format!("read {}", directory.display()))?
        .collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.path());
    entries
        .into_iter()
        .filter(|entry| entry.path().extension().and_then(|value| value.to_str()) == Some("json"))
        .map(|entry| read_json::<T>(&entry.path()))
        .collect()
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

fn canonical_project_root(project_root: impl AsRef<Path>) -> Result<PathBuf> {
    project_root
        .as_ref()
        .canonicalize()
        .with_context(|| format!("canonicalize {}", project_root.as_ref().display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use agentflow_event_store::{append_task_event_once, EventActor, TaskEventDraft};
    use agentflow_spec::{SpecIssueDraft, SpecProjectDraft};
    use serde_json::json;
    use tempfile::tempdir;

    fn write_fixture(root: &Path) {
        let requirement = root.join("docs/requirements/034-test.md");
        fs::create_dir_all(requirement.parent().unwrap()).unwrap();
        fs::write(&requirement, "# 测试需求\n\n用于 projection 测试。\n").unwrap();
        let project_docs = root.join("docs/projects/project-projection");
        fs::create_dir_all(&project_docs).unwrap();
        fs::write(project_docs.join("GOAL.md"), "# Goal\n\n确认目标。\n").unwrap();
        fs::write(project_docs.join("PLAN.md"), "# Plan\n\n确认计划。\n").unwrap();
        fs::write(
            project_docs.join("DECISIONS.md"),
            "# Decisions\n\n## Decision Log\n\n### 2026-06-18 - Goal confirmation\n",
        )
        .unwrap();

        let mut issue = SpecIssueDraft::new("AF-PROJ-001");
        issue.project_id = Some("project-projection".to_string());
        let issue = agentflow_spec::issue_from_requirement(root, &requirement, issue).unwrap();
        agentflow_spec::write_spec_issue(root, &issue).unwrap();

        let mut project = SpecProjectDraft::new("project-projection");
        project.issue_ids = vec!["AF-PROJ-001".to_string()];
        let project =
            agentflow_spec::project_from_requirement(root, &requirement, project).unwrap();
        agentflow_spec::write_spec_project(root, &project).unwrap();
    }

    fn event(issue_id: &str, event_type: &str, payload: serde_json::Value) -> TaskEventDraft {
        TaskEventDraft {
            flow_type: agentflow_workflow_core::WorkflowFlowType::Work,
            aggregate_type: "issue".to_string(),
            aggregate_id: issue_id.to_string(),
            project_id: Some("project-projection".to_string()),
            issue_id: Some(issue_id.to_string()),
            run_id: payload
                .get("runId")
                .and_then(serde_json::Value::as_str)
                .map(str::to_string),
            event_type: event_type.to_string(),
            authority_role: Some(agentflow_workflow_core::WorkflowAgentRole::WorkAgent),
            actor: EventActor {
                role: "test".to_string(),
                kind: "system".to_string(),
            },
            state: None,
            correlation_id: Some(format!("corr-{issue_id}")),
            causation_id: None,
            payload,
            artifact_refs: Vec::new(),
            idempotency_key: Some(format!("{event_type}:{issue_id}")),
        }
    }

    #[test]
    fn rebuilds_task_projection_from_events() {
        let dir = tempdir().unwrap();
        write_fixture(dir.path());
        append_task_event_once(
            dir.path(),
            event("AF-PROJ-001", "issue.scheduled", json!({})),
        )
        .unwrap();
        append_task_event_once(
            dir.path(),
            event(
                "AF-PROJ-001",
                "agent.launch.requested",
                json!({
                    "runId": "run-001",
                    "branchName": "agentflow/project-projection/AF-PROJ-001"
                }),
            ),
        )
        .unwrap();

        let summary = rebuild_projections(dir.path()).unwrap();
        let projection = crate::storage::load_task_projection(dir.path(), "AF-PROJ-001").unwrap();

        assert_eq!(summary.task_count, 1);
        assert_eq!(projection.current_state, "in_progress");
        assert_eq!(projection.latest_run_id.as_deref(), Some("run-001"));
        assert_eq!(
            projection
                .timeline
                .iter()
                .find(|item| item.state == "in_progress")
                .unwrap()
                .phase,
            ProjectionPhase::Current
        );
        assert!(dir
            .path()
            .join(".agentflow/indexes/issue-status.json")
            .is_file());
    }

    #[test]
    fn project_projection_counts_completed_issues() {
        let dir = tempdir().unwrap();
        write_fixture(dir.path());
        append_task_event_once(
            dir.path(),
            event(
                "AF-PROJ-001",
                "issue.completed",
                json!({
                    "runId": "run-001",
                    "prUrl": "https://github.com/example/repo/pull/1",
                    "mergeCommit": "abc123",
                    "changelogPath": "CHANGELOG.md",
                    "releaseNotesUrl": "docs/release-notes/agentflow-release-notes.md"
                }),
            ),
        )
        .unwrap();

        rebuild_projections(dir.path()).unwrap();
        let task = crate::storage::load_task_projection(dir.path(), "AF-PROJ-001").unwrap();
        let project =
            crate::storage::load_project_projection(dir.path(), "project-projection").unwrap();

        assert_eq!(task.current_state, "done");
        assert_eq!(
            task.public_delivery.pr_url.as_deref(),
            Some("https://github.com/example/repo/pull/1")
        );
        assert_eq!(
            task.public_delivery.changelog_path.as_deref(),
            Some("CHANGELOG.md")
        );
        assert_eq!(
            task.public_delivery.release_notes_url.as_deref(),
            Some("docs/release-notes/agentflow-release-notes.md")
        );
        assert_eq!(project.completed_issue_count, 1);
        assert_eq!(project.status, "done");
        assert_eq!(project.objective, "用于 projection 测试。");
        assert_eq!(project.project_brain.brain_status, "ready-for-project-loop");
        assert_eq!(
            project.project_brain.project_path,
            "docs/projects/project-projection"
        );
    }
}
