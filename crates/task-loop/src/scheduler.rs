use crate::model::{
    AgentLaunchPayload, TaskLoopLaunch, TaskLoopSchedule, AGENT_LAUNCH_REQUESTED, ISSUE_SCHEDULED,
    TASK_LOOP_LAUNCH_REQUEST_VERSION,
};
use agentflow_event_store::{
    append_task_event_once, load_task_events, EventActor, EventStateTransition, TaskEventDraft,
};
use agentflow_spec::{
    read_spec_issue, read_spec_project, SpecIssue, SpecIssueStatus, SpecPriority, SpecProject,
};
use agentflow_task_artifacts::create_task_run;
use anyhow::{Context, Result};
use serde_json::json;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskLoop {
    project_id: String,
}

impl TaskLoop {
    pub fn new(project_id: impl Into<String>) -> Self {
        Self {
            project_id: project_id.into(),
        }
    }

    pub fn project_id(&self) -> &str {
        &self.project_id
    }

    pub fn schedule_next_issue(
        &self,
        project_root: impl AsRef<Path>,
    ) -> Result<Option<TaskLoopSchedule>> {
        let root = canonical_project_root(project_root)?;
        let project = read_spec_project(&root, &self.project_id)?;
        let issues = load_project_issues(&root, &project)?;
        let states = current_issue_states(&root, &issues)?;
        let Some(issue) = next_schedulable_issue(&issues, &states) else {
            return Ok(None);
        };

        let event = append_task_event_once(
            &root,
            TaskEventDraft {
                aggregate_type: "issue".to_string(),
                aggregate_id: issue.issue_id.clone(),
                project_id: issue.project_id.clone(),
                issue_id: Some(issue.issue_id.clone()),
                event_type: ISSUE_SCHEDULED.to_string(),
                actor: EventActor {
                    role: "task-loop".to_string(),
                    kind: "system".to_string(),
                },
                state: Some(EventStateTransition {
                    from_state: SpecIssueStatus::Backlog.as_str().to_string(),
                    to_state: SpecIssueStatus::Todo.as_str().to_string(),
                }),
                correlation_id: Some(format!("corr-{}", issue.issue_id)),
                causation_id: None,
                payload: json!({
                    "workflowRef": issue.workflow_ref,
                    "transitionId": "schedule",
                    "guardsPassed": [
                        "issue.contract.complete",
                        "dependencies.done"
                    ]
                }),
                artifact_refs: vec![issue.system.path.clone()],
                idempotency_key: Some(format!("issue.scheduled:{}", issue.issue_id)),
            },
        )?;

        Ok(Some(TaskLoopSchedule {
            project_id: self.project_id.clone(),
            issue_id: issue.issue_id.clone(),
            workflow_ref: issue.workflow_ref.clone(),
            event_id: event.event_id,
        }))
    }

    pub fn request_agent_launch(
        &self,
        project_root: impl AsRef<Path>,
        issue_id: &str,
        provider: &str,
    ) -> Result<TaskLoopLaunch> {
        let root = canonical_project_root(project_root)?;
        let issue = read_spec_issue(&root, issue_id)?;
        let state = current_issue_state(&root, &issue)?;
        if !matches!(state, SpecIssueStatus::Todo) {
            anyhow::bail!(
                "issue {} must be todo before agent launch, found {}",
                issue.issue_id,
                state.as_str()
            );
        }
        let run_id = next_run_id(&root, &issue.issue_id)?;
        let branch_name = branch_name(&issue);
        let run = create_task_run(
            &root,
            &issue.issue_id,
            &run_id,
            &issue.workflow_ref,
            Some(branch_name.clone()),
        )?;
        let payload = AgentLaunchPayload {
            version: TASK_LOOP_LAUNCH_REQUEST_VERSION.to_string(),
            provider: provider.to_string(),
            issue_id: issue.issue_id.clone(),
            project_id: issue.project_id.clone(),
            run_id: run.run_id.clone(),
            agent_role: "build-agent".to_string(),
            workflow_ref: issue.workflow_ref.clone(),
            working_directory: root.display().to_string(),
            issue_path: issue.system.path.clone(),
            launch_request_path: launch_request_path(&issue.issue_id, &run.run_id),
            context_pack_path: None,
            branch_name: branch_name.clone(),
            merge_mode: "auto-merge-if-eligible".to_string(),
        };
        write_launch_request(&root, &payload)?;

        let event = append_task_event_once(
            &root,
            TaskEventDraft {
                aggregate_type: "issue".to_string(),
                aggregate_id: issue.issue_id.clone(),
                project_id: issue.project_id.clone(),
                issue_id: Some(issue.issue_id.clone()),
                event_type: AGENT_LAUNCH_REQUESTED.to_string(),
                actor: EventActor {
                    role: "task-loop".to_string(),
                    kind: "system".to_string(),
                },
                state: None,
                correlation_id: Some(format!("corr-{}", issue.issue_id)),
                causation_id: None,
                payload: serde_json::to_value(&payload)?,
                artifact_refs: vec![
                    payload.launch_request_path.clone(),
                    format!(
                        ".agentflow/tasks/{}/runs/{}/run.json",
                        issue.issue_id, run.run_id
                    ),
                ],
                idempotency_key: Some(format!("agent.launch.requested:{}", run.run_id)),
            },
        )?;

        Ok(TaskLoopLaunch {
            project_id: issue.project_id.clone(),
            issue_id: issue.issue_id,
            run_id,
            branch_name,
            launch_request_path: payload.launch_request_path,
            event_id: event.event_id,
        })
    }
}

fn load_project_issues(root: &Path, project: &SpecProject) -> Result<Vec<SpecIssue>> {
    project
        .issue_ids
        .iter()
        .map(|issue_id| read_spec_issue(root, issue_id))
        .collect()
}

fn next_schedulable_issue<'a>(
    issues: &'a [SpecIssue],
    states: &BTreeMap<String, SpecIssueStatus>,
) -> Option<&'a SpecIssue> {
    let done = states
        .iter()
        .filter_map(|(issue_id, state)| {
            matches!(state, SpecIssueStatus::Done).then_some(issue_id.clone())
        })
        .collect::<BTreeSet<_>>();
    let mut candidates = issues
        .iter()
        .filter(|issue| {
            states.get(&issue.issue_id) == Some(&SpecIssueStatus::Backlog)
                && issue
                    .blocked_by
                    .iter()
                    .all(|dependency| done.contains(dependency))
        })
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| {
        left.blocked_by
            .len()
            .cmp(&right.blocked_by.len())
            .then_with(|| priority_rank(&left.priority).cmp(&priority_rank(&right.priority)))
            .then_with(|| issue_number(&left.issue_id).cmp(&issue_number(&right.issue_id)))
            .then_with(|| left.issue_id.cmp(&right.issue_id))
    });
    candidates.into_iter().next()
}

fn current_issue_states(
    root: &Path,
    issues: &[SpecIssue],
) -> Result<BTreeMap<String, SpecIssueStatus>> {
    let mut states = issues
        .iter()
        .map(|issue| (issue.issue_id.clone(), issue.status.clone()))
        .collect::<BTreeMap<_, _>>();
    for event in load_task_events(root)? {
        let Some(issue_id) = event.issue_id else {
            continue;
        };
        if let Some(state) = event.state {
            if let Some(parsed) = parse_issue_status(&state.to_state) {
                states.insert(issue_id, parsed);
            }
        }
    }
    Ok(states)
}

fn current_issue_state(root: &Path, issue: &SpecIssue) -> Result<SpecIssueStatus> {
    Ok(current_issue_states(root, std::slice::from_ref(issue))?
        .remove(&issue.issue_id)
        .unwrap_or_else(|| issue.status.clone()))
}

fn parse_issue_status(value: &str) -> Option<SpecIssueStatus> {
    match value {
        "backlog" => Some(SpecIssueStatus::Backlog),
        "todo" => Some(SpecIssueStatus::Todo),
        "in_progress" => Some(SpecIssueStatus::InProgress),
        "in_review" => Some(SpecIssueStatus::InReview),
        "done" => Some(SpecIssueStatus::Done),
        "blocked" => Some(SpecIssueStatus::Blocked),
        "cancel" => Some(SpecIssueStatus::Cancel),
        _ => None,
    }
}

fn priority_rank(priority: &SpecPriority) -> u8 {
    match priority {
        SpecPriority::P0 => 0,
        SpecPriority::P1 => 1,
        SpecPriority::P2 => 2,
        SpecPriority::P3 => 3,
    }
}

fn issue_number(issue_id: &str) -> u64 {
    issue_id
        .rsplit_once('-')
        .and_then(|(_, number)| number.parse::<u64>().ok())
        .unwrap_or(u64::MAX)
}

fn next_run_id(root: &Path, issue_id: &str) -> Result<String> {
    let runs_dir = root.join(format!(".agentflow/tasks/{issue_id}/runs"));
    if !runs_dir.exists() {
        return Ok("run-001".to_string());
    }
    let mut max_seen = 0_u64;
    for entry in fs::read_dir(&runs_dir).with_context(|| format!("read {}", runs_dir.display()))? {
        let entry = entry?;
        let Some(name) = entry.file_name().to_str().map(str::to_string) else {
            continue;
        };
        if let Some(number) = name
            .strip_prefix("run-")
            .and_then(|number| number.parse::<u64>().ok())
        {
            max_seen = max_seen.max(number);
        }
    }
    Ok(format!("run-{:03}", max_seen + 1))
}

fn branch_name(issue: &SpecIssue) -> String {
    let project = issue.project_id.as_deref().unwrap_or("direct");
    format!("agentflow/{project}/{}", issue.issue_id)
}

fn launch_request_path(issue_id: &str, run_id: &str) -> String {
    format!(".agentflow/tasks/{issue_id}/runs/{run_id}/launch/agent-request.json")
}

fn write_launch_request(root: &Path, payload: &AgentLaunchPayload) -> Result<()> {
    let path = root.join(&payload.launch_request_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&path, serde_json::to_string_pretty(payload)? + "\n")
        .with_context(|| format!("write {}", path.display()))
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
    use agentflow_event_store::{replay_task_events, ReplayFilter};
    use agentflow_spec::{
        write_spec_issue, write_spec_project, SpecIssueDraft, SpecPriority, SpecProjectDraft,
        DEFAULT_WORKFLOW_REF,
    };
    use std::path::Path;
    use tempfile::tempdir;

    fn write_requirement(root: &Path) {
        let path = root.join("docs/requirements/034-test.md");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "# 测试需求\n\n用于 task-loop 测试。\n").unwrap();
    }

    fn write_project_with_issues(root: &Path) {
        write_requirement(root);
        let requirement = root.join("docs/requirements/034-test.md");
        let mut first = SpecIssueDraft::new("AF-TASK-001");
        first.project_id = Some("project-task-loop".to_string());
        first.priority = SpecPriority::P1;
        let first = agentflow_spec::issue_from_requirement(root, &requirement, first).unwrap();
        write_spec_issue(root, &first).unwrap();

        let mut second = SpecIssueDraft::new("AF-TASK-002");
        second.project_id = Some("project-task-loop".to_string());
        second.priority = SpecPriority::P0;
        second.blocked_by = vec!["AF-TASK-001".to_string()];
        let second = agentflow_spec::issue_from_requirement(root, &requirement, second).unwrap();
        write_spec_issue(root, &second).unwrap();

        let mut project = SpecProjectDraft::new("project-task-loop");
        project.issue_ids = vec!["AF-TASK-002".to_string(), "AF-TASK-001".to_string()];
        let project =
            agentflow_spec::project_from_requirement(root, &requirement, project).unwrap();
        write_spec_project(root, &project).unwrap();
    }

    #[test]
    fn schedules_dependency_ready_backlog_issue() {
        let dir = tempdir().unwrap();
        write_project_with_issues(dir.path());
        let loop_driver = TaskLoop::new("project-task-loop");

        let scheduled = loop_driver
            .schedule_next_issue(dir.path())
            .unwrap()
            .unwrap();

        assert_eq!(scheduled.issue_id, "AF-TASK-001");
        assert_eq!(scheduled.workflow_ref, DEFAULT_WORKFLOW_REF);
        let events = replay_task_events(dir.path(), ReplayFilter::issue("AF-TASK-001")).unwrap();
        assert_eq!(events[0].event_type, ISSUE_SCHEDULED);
        assert_eq!(
            events[0].state.as_ref().unwrap().to_state,
            SpecIssueStatus::Todo.as_str()
        );
    }

    #[test]
    fn launch_requires_todo_and_writes_request_event() {
        let dir = tempdir().unwrap();
        write_project_with_issues(dir.path());
        let loop_driver = TaskLoop::new("project-task-loop");
        loop_driver
            .schedule_next_issue(dir.path())
            .unwrap()
            .unwrap();

        let launch = loop_driver
            .request_agent_launch(dir.path(), "AF-TASK-001", "codex")
            .unwrap();

        assert_eq!(launch.run_id, "run-001");
        assert!(dir
            .path()
            .join(".agentflow/tasks/AF-TASK-001/runs/run-001/run.json")
            .is_file());
        assert!(dir.path().join(&launch.launch_request_path).is_file());
        let events = replay_task_events(dir.path(), ReplayFilter::issue("AF-TASK-001")).unwrap();
        assert!(events
            .iter()
            .any(|event| event.event_type == AGENT_LAUNCH_REQUESTED));
    }

    #[test]
    fn does_not_schedule_dependency_waiting_issue() {
        let dir = tempdir().unwrap();
        write_project_with_issues(dir.path());
        let loop_driver = TaskLoop::new("project-task-loop");
        loop_driver
            .schedule_next_issue(dir.path())
            .unwrap()
            .unwrap();
        let scheduled_again = loop_driver.schedule_next_issue(dir.path()).unwrap();

        assert!(scheduled_again.is_none());
    }
}
