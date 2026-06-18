use crate::model::{
    AgentLaunchPayload, TaskLoopLaunch, TaskLoopSchedule, TaskLoopTick, AGENT_LAUNCH_REQUESTED,
    ISSUE_SCHEDULED, TASK_LOOP_LAUNCH_REQUEST_VERSION,
};
use agentflow_event_store::{
    allocate_task_sequence, append_task_event_once, load_task_events, EventActor,
    EventStateTransition, TaskEvent, TaskEventDraft,
};
use agentflow_spec::{
    read_spec_issue, read_spec_project, SpecIssue, SpecIssueStatus, SpecPriority, SpecProject,
};
use agentflow_task_artifacts::create_task_run;
use agentflow_workflow_core::{WorkflowAgentRole, WorkflowFlowType};
use anyhow::{Context, Result};
use serde_json::json;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

const ISSUE_CONTRACT_COMPLETE_GUARD: &str = "issue.contract.complete";
const DEPENDENCIES_DONE_GUARD: &str = "dependencies.done";
const PROJECT_PREDECESSORS_DONE_GUARD: &str = "project.sequence.predecessors.done";
const PROJECT_SERIAL_SLOT_FREE_GUARD: &str = "project.serial_slot.free";

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
        let Some(issue) = next_schedulable_issue(&issues, &project, &states) else {
            return Ok(None);
        };
        let guards = validate_schedule_guards(issue, Some(&project), &states)?;
        Ok(Some(append_issue_scheduled_event(&root, issue, &guards)?))
    }

    pub fn request_agent_launch(
        &self,
        project_root: impl AsRef<Path>,
        issue_id: &str,
        provider: &str,
    ) -> Result<TaskLoopLaunch> {
        let root = canonical_project_root(project_root)?;
        let issue = read_spec_issue(&root, issue_id)?;
        if issue.project_id.as_deref() != Some(self.project_id.as_str()) {
            anyhow::bail!(
                "issue {} does not belong to project {}",
                issue.issue_id,
                self.project_id
            );
        }
        request_issue_launch_inner(&root, issue, provider)
    }

    pub fn start_issue(
        project_root: impl AsRef<Path>,
        issue_id: &str,
        provider: &str,
    ) -> Result<TaskLoopTick> {
        let root = canonical_project_root(project_root)?;
        let issue = read_spec_issue(&root, issue_id)?;
        validate_issue_scope(&root, &issue)?;
        let context = load_issue_runtime_context(&root, &issue)?;
        let state = context
            .states
            .get(&issue.issue_id)
            .cloned()
            .unwrap_or_else(|| issue.status.clone());
        let schedule = match state {
            SpecIssueStatus::Backlog => Some(schedule_specific_issue(&root, &issue)?),
            SpecIssueStatus::Todo => None,
            SpecIssueStatus::InProgress => {
                if let Some(launch) = recoverable_launch_for_issue(&root, &issue)? {
                    return Ok(TaskLoopTick {
                        schedule: None,
                        launch,
                    });
                }
                anyhow::bail!(
                    "issue {} is already in progress without a recoverable launch request",
                    issue.issue_id
                );
            }
            _ => {
                anyhow::bail!(
                    "issue {} must be backlog, todo, or recoverable in_progress before agent launch, found {}",
                    issue.issue_id,
                    state.as_str()
                );
            }
        };
        let launch = request_issue_launch_inner(&root, issue, provider)?;
        Ok(TaskLoopTick { schedule, launch })
    }

    pub fn request_direct_agent_launch(
        project_root: impl AsRef<Path>,
        issue_id: &str,
        provider: &str,
    ) -> Result<TaskLoopLaunch> {
        let root = canonical_project_root(project_root)?;
        let issue = read_spec_issue(&root, issue_id)?;
        if issue.project_id.is_some() {
            anyhow::bail!(
                "direct launch only supports direct issues; {} belongs to {}",
                issue.issue_id,
                issue.project_id.as_deref().unwrap_or("unknown")
            );
        }
        request_issue_launch_inner(&root, issue, provider)
    }

    pub fn schedule_direct_issue(
        project_root: impl AsRef<Path>,
        issue_id: &str,
    ) -> Result<Option<TaskLoopSchedule>> {
        let root = canonical_project_root(project_root)?;
        let issue = read_spec_issue(&root, issue_id)?;
        if issue.project_id.is_some() {
            anyhow::bail!(
                "direct schedule only supports direct issues; {} belongs to {}",
                issue.issue_id,
                issue.project_id.as_deref().unwrap_or("unknown")
            );
        }
        if current_issue_state(&root, &issue)? != SpecIssueStatus::Backlog {
            return Ok(None);
        }
        Ok(Some(schedule_specific_issue(&root, &issue)?))
    }

    pub fn tick(
        &self,
        project_root: impl AsRef<Path>,
        provider: &str,
    ) -> Result<Option<TaskLoopTick>> {
        let root = canonical_project_root(project_root)?;
        let project = read_spec_project(&root, &self.project_id)?;
        let issues = load_project_issues(&root, &project)?;
        let states = current_issue_states(&root, &issues)?;
        if let Some(launch) = recoverable_launch_for_project(&root, &issues, &states)? {
            return Ok(Some(TaskLoopTick {
                schedule: None,
                launch,
            }));
        }
        let launched = launch_requested_issue_ids(&root)?;

        if let Some(issue) = next_launchable_issue(&issues, &project, &states, &launched) {
            let launch = self.request_agent_launch(&root, &issue.issue_id, provider)?;
            return Ok(Some(TaskLoopTick {
                schedule: None,
                launch,
            }));
        }

        let Some(schedule) = self.schedule_next_issue(&root)? else {
            return Ok(None);
        };
        let launch = self.request_agent_launch(&root, &schedule.issue_id, provider)?;
        Ok(Some(TaskLoopTick {
            schedule: Some(schedule),
            launch,
        }))
    }
}

fn request_issue_launch_inner(
    root: &Path,
    issue: SpecIssue,
    provider: &str,
) -> Result<TaskLoopLaunch> {
    if launch_requested_issue_ids(root)?.contains(&issue.issue_id) {
        anyhow::bail!("issue {} already has a launch request", issue.issue_id);
    }
    let state = current_issue_state(root, &issue)?;
    if !matches!(state, SpecIssueStatus::Todo) {
        anyhow::bail!(
            "issue {} must be todo before agent launch, found {}",
            issue.issue_id,
            state.as_str()
        );
    }
    let context = load_issue_runtime_context(root, &issue)?;
    validate_launch_guards(&issue, context.project.as_ref(), &context.states)?;
    let run_id = next_run_id(root, &issue.issue_id)?;
    let branch_name = branch_name(&issue);
    let run = create_task_run(
        root,
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
    write_launch_request(root, &payload)?;

    let event = append_task_event_once(
        root,
        TaskEventDraft {
            flow_type: WorkflowFlowType::Work,
            aggregate_type: "issue".to_string(),
            aggregate_id: issue.issue_id.clone(),
            project_id: issue.project_id.clone(),
            issue_id: Some(issue.issue_id.clone()),
            run_id: Some(run.run_id.clone()),
            event_type: AGENT_LAUNCH_REQUESTED.to_string(),
            authority_role: Some(WorkflowAgentRole::WorkAgent),
            actor: EventActor {
                role: "task-loop".to_string(),
                kind: "system".to_string(),
            },
            state: Some(EventStateTransition {
                from_state: SpecIssueStatus::Todo.as_str().to_string(),
                to_state: SpecIssueStatus::InProgress.as_str().to_string(),
            }),
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
            idempotency_key: Some(format!(
                "agent.launch.requested:{}:{}",
                issue.issue_id, run.run_id
            )),
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

fn validate_issue_scope(root: &Path, issue: &SpecIssue) -> Result<()> {
    if let Some(project_id) = issue.project_id.as_deref() {
        let project = read_spec_project(root, project_id)?;
        if !project
            .issue_ids
            .iter()
            .any(|value| value == &issue.issue_id)
        {
            anyhow::bail!(
                "project {} does not reference issue {}",
                project_id,
                issue.issue_id
            );
        }
    }
    Ok(())
}

fn schedule_specific_issue(root: &Path, issue: &SpecIssue) -> Result<TaskLoopSchedule> {
    let context = load_issue_runtime_context(root, issue)?;
    let guards = validate_schedule_guards(issue, context.project.as_ref(), &context.states)?;
    append_issue_scheduled_event(root, issue, &guards)
}

fn append_issue_scheduled_event(
    root: &Path,
    issue: &SpecIssue,
    guards_passed: &[&str],
) -> Result<TaskLoopSchedule> {
    let event = append_task_event_once(
        root,
        TaskEventDraft {
            flow_type: WorkflowFlowType::Work,
            aggregate_type: "issue".to_string(),
            aggregate_id: issue.issue_id.clone(),
            project_id: issue.project_id.clone(),
            issue_id: Some(issue.issue_id.clone()),
            run_id: None,
            event_type: ISSUE_SCHEDULED.to_string(),
            authority_role: Some(WorkflowAgentRole::WorkAgent),
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
                "guardsPassed": guards_passed
            }),
            artifact_refs: vec![issue.system.path.clone()],
            idempotency_key: Some(format!("issue.scheduled:{}", issue.issue_id)),
        },
    )?;

    Ok(TaskLoopSchedule {
        project_id: issue
            .project_id
            .clone()
            .unwrap_or_else(|| "direct".to_string()),
        issue_id: issue.issue_id.clone(),
        workflow_ref: issue.workflow_ref.clone(),
        event_id: event.event_id,
    })
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
    project: &SpecProject,
    states: &BTreeMap<String, SpecIssueStatus>,
) -> Option<&'a SpecIssue> {
    issues.iter().find(|issue| {
        states.get(&issue.issue_id) == Some(&SpecIssueStatus::Backlog)
            && validate_schedule_guards(issue, Some(project), states).is_ok()
    })
}

fn next_launchable_issue<'a>(
    issues: &'a [SpecIssue],
    project: &SpecProject,
    states: &BTreeMap<String, SpecIssueStatus>,
    launched: &BTreeSet<String>,
) -> Option<&'a SpecIssue> {
    issues.iter().find(|issue| {
        states.get(&issue.issue_id) == Some(&SpecIssueStatus::Todo)
            && !launched.contains(&issue.issue_id)
            && validate_launch_guards(issue, Some(project), states).is_ok()
    })
}

fn launch_requested_issue_ids(root: &Path) -> Result<BTreeSet<String>> {
    Ok(load_task_events(root)?
        .into_iter()
        .filter(|event| event.event_type == AGENT_LAUNCH_REQUESTED)
        .filter_map(|event| event.issue_id)
        .collect())
}

fn recoverable_launch_for_project(
    root: &Path,
    issues: &[SpecIssue],
    states: &BTreeMap<String, SpecIssueStatus>,
) -> Result<Option<TaskLoopLaunch>> {
    let events = load_task_events(root)?;
    let recoverable_run_ids = recoverable_launch_run_ids(&events);
    let mut candidates = issues
        .iter()
        .filter(|issue| states.get(&issue.issue_id) == Some(&SpecIssueStatus::InProgress))
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| {
        priority_rank(&left.priority)
            .cmp(&priority_rank(&right.priority))
            .then_with(|| issue_number(&left.issue_id).cmp(&issue_number(&right.issue_id)))
            .then_with(|| left.issue_id.cmp(&right.issue_id))
    });
    for issue in candidates {
        if let Some(launch) = recoverable_launch_from_events(issue, &events, &recoverable_run_ids)?
        {
            return Ok(Some(launch));
        }
    }
    Ok(None)
}

fn recoverable_launch_for_issue(root: &Path, issue: &SpecIssue) -> Result<Option<TaskLoopLaunch>> {
    let events = load_task_events(root)?;
    let recoverable_run_ids = recoverable_launch_run_ids(&events);
    recoverable_launch_from_events(issue, &events, &recoverable_run_ids)
}

fn recoverable_launch_run_ids(events: &[TaskEvent]) -> BTreeSet<String> {
    let mut claimable = BTreeMap::new();
    for event in events {
        let Some(run_id) = event.run_id.clone() else {
            continue;
        };
        match event.event_type.as_str() {
            AGENT_LAUNCH_REQUESTED => {
                claimable.entry(run_id).or_insert(true);
            }
            "agent.launch.claimed"
            | "agent.session.created"
            | "agent.session.running"
            | "agent.session.in_review"
            | "agent.session.completed" => {
                claimable.insert(run_id, false);
            }
            "agent.session.interrupted" | "agent.session.failed" => {
                claimable.insert(run_id, true);
            }
            _ => {}
        }
    }
    claimable
        .into_iter()
        .filter_map(|(run_id, allowed)| allowed.then_some(run_id))
        .collect()
}

fn recoverable_launch_from_events(
    issue: &SpecIssue,
    events: &[TaskEvent],
    recoverable_run_ids: &BTreeSet<String>,
) -> Result<Option<TaskLoopLaunch>> {
    let Some(event) = events.iter().rev().find(|event| {
        event.event_type == AGENT_LAUNCH_REQUESTED
            && event.issue_id.as_deref() == Some(issue.issue_id.as_str())
            && event
                .run_id
                .as_deref()
                .is_some_and(|run_id| recoverable_run_ids.contains(run_id))
    }) else {
        return Ok(None);
    };
    let payload: AgentLaunchPayload = serde_json::from_value(event.payload.clone())?;
    Ok(Some(TaskLoopLaunch {
        project_id: payload.project_id,
        issue_id: payload.issue_id,
        run_id: payload.run_id,
        branch_name: payload.branch_name,
        launch_request_path: payload.launch_request_path,
        event_id: event.event_id.clone(),
    }))
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

struct IssueRuntimeContext {
    project: Option<SpecProject>,
    states: BTreeMap<String, SpecIssueStatus>,
}

fn load_issue_runtime_context(root: &Path, issue: &SpecIssue) -> Result<IssueRuntimeContext> {
    if let Some(project_id) = issue.project_id.as_deref() {
        let project = read_spec_project(root, project_id)?;
        let issues = load_project_issues(root, &project)?;
        let states = current_issue_states(root, &issues)?;
        Ok(IssueRuntimeContext {
            project: Some(project),
            states,
        })
    } else {
        Ok(IssueRuntimeContext {
            project: None,
            states: current_issue_states(root, std::slice::from_ref(issue))?,
        })
    }
}

fn validate_schedule_guards(
    issue: &SpecIssue,
    project: Option<&SpecProject>,
    states: &BTreeMap<String, SpecIssueStatus>,
) -> Result<Vec<&'static str>> {
    let mut passed = vec![ISSUE_CONTRACT_COMPLETE_GUARD, DEPENDENCIES_DONE_GUARD];
    ensure_issue_contract_complete(issue)?;
    ensure_dependencies_done(issue, states)?;
    if let Some(project) = project {
        ensure_project_predecessors_done(project, issue, states)?;
        ensure_project_serial_slot_free(project, &issue.issue_id, states, false)?;
        passed.push(PROJECT_PREDECESSORS_DONE_GUARD);
        passed.push(PROJECT_SERIAL_SLOT_FREE_GUARD);
    }
    Ok(passed)
}

fn validate_launch_guards(
    issue: &SpecIssue,
    project: Option<&SpecProject>,
    states: &BTreeMap<String, SpecIssueStatus>,
) -> Result<Vec<&'static str>> {
    let mut passed = vec![ISSUE_CONTRACT_COMPLETE_GUARD, DEPENDENCIES_DONE_GUARD];
    ensure_issue_contract_complete(issue)?;
    ensure_dependencies_done(issue, states)?;
    if let Some(project) = project {
        ensure_project_predecessors_done(project, issue, states)?;
        ensure_project_serial_slot_free(project, &issue.issue_id, states, true)?;
        passed.push(PROJECT_PREDECESSORS_DONE_GUARD);
        passed.push(PROJECT_SERIAL_SLOT_FREE_GUARD);
    }
    Ok(passed)
}

fn ensure_issue_contract_complete(issue: &SpecIssue) -> Result<()> {
    if issue.issue_id.trim().is_empty()
        || issue.workflow_ref.trim().is_empty()
        || issue.source_spec_id.trim().is_empty()
        || issue.system.path.trim().is_empty()
    {
        anyhow::bail!("issue {} contract incomplete", issue.issue_id);
    }
    Ok(())
}

fn ensure_dependencies_done(
    issue: &SpecIssue,
    states: &BTreeMap<String, SpecIssueStatus>,
) -> Result<()> {
    let unfinished = issue
        .blocked_by
        .iter()
        .filter(|dependency| !matches!(states.get(*dependency), Some(SpecIssueStatus::Done)))
        .cloned()
        .collect::<Vec<_>>();
    if unfinished.is_empty() {
        return Ok(());
    }
    anyhow::bail!(
        "issue {} has unfinished dependencies: {}",
        issue.issue_id,
        unfinished.join(", ")
    );
}

fn ensure_project_predecessors_done(
    project: &SpecProject,
    issue: &SpecIssue,
    states: &BTreeMap<String, SpecIssueStatus>,
) -> Result<()> {
    let Some(issue_index) = project
        .issue_ids
        .iter()
        .position(|value| value == &issue.issue_id)
    else {
        anyhow::bail!(
            "project {} does not reference issue {}",
            project.project_id,
            issue.issue_id
        );
    };
    let unfinished = project.issue_ids[..issue_index]
        .iter()
        .filter(|predecessor| {
            !matches!(
                states.get(*predecessor),
                Some(SpecIssueStatus::Done | SpecIssueStatus::Cancel)
            )
        })
        .cloned()
        .collect::<Vec<_>>();
    if unfinished.is_empty() {
        return Ok(());
    }
    anyhow::bail!(
        "issue {} has unfinished project predecessors: {}",
        issue.issue_id,
        unfinished.join(", ")
    );
}

fn ensure_project_serial_slot_free(
    project: &SpecProject,
    issue_id: &str,
    states: &BTreeMap<String, SpecIssueStatus>,
    allow_self_active: bool,
) -> Result<()> {
    let active = project
        .issue_ids
        .iter()
        .filter(|candidate| {
            matches!(
                states.get(*candidate),
                Some(
                    SpecIssueStatus::Todo | SpecIssueStatus::InProgress | SpecIssueStatus::InReview
                )
            ) && (!allow_self_active || candidate.as_str() != issue_id)
        })
        .cloned()
        .collect::<Vec<_>>();
    if active.is_empty() {
        return Ok(());
    }
    anyhow::bail!(
        "project {} already has active issue(s): {}",
        project.project_id,
        active.join(", ")
    );
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
    Ok(format!(
        "run-{:03}",
        allocate_task_sequence(root, &format!("run-id:{issue_id}"))?
    ))
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
        project.issue_ids = vec!["AF-TASK-001".to_string(), "AF-TASK-002".to_string()];
        let project =
            agentflow_spec::project_from_requirement(root, &requirement, project).unwrap();
        write_spec_project(root, &project).unwrap();
    }

    fn write_ordered_project_without_dependencies(root: &Path) {
        write_requirement(root);
        let requirement = root.join("docs/requirements/034-test.md");
        for issue_id in ["AF-TASK-001", "AF-TASK-002", "AF-TASK-003"] {
            let mut draft = SpecIssueDraft::new(issue_id);
            draft.project_id = Some("project-task-loop".to_string());
            draft.priority = if issue_id == "AF-TASK-003" {
                SpecPriority::P0
            } else {
                SpecPriority::P1
            };
            let issue = agentflow_spec::issue_from_requirement(root, &requirement, draft).unwrap();
            write_spec_issue(root, &issue).unwrap();
        }

        let mut project = SpecProjectDraft::new("project-task-loop");
        project.issue_ids = vec![
            "AF-TASK-001".to_string(),
            "AF-TASK-002".to_string(),
            "AF-TASK-003".to_string(),
        ];
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
    fn tick_schedules_and_launches_next_issue() {
        let dir = tempdir().unwrap();
        write_project_with_issues(dir.path());
        let loop_driver = TaskLoop::new("project-task-loop");

        let tick = loop_driver.tick(dir.path(), "codex").unwrap().unwrap();

        let schedule = tick.schedule.unwrap();
        assert_eq!(schedule.issue_id, "AF-TASK-001");
        assert_eq!(tick.launch.issue_id, "AF-TASK-001");
        assert_eq!(tick.launch.run_id, "run-001");
        assert!(dir.path().join(&tick.launch.launch_request_path).is_file());
        let events = replay_task_events(dir.path(), ReplayFilter::issue("AF-TASK-001")).unwrap();
        assert_eq!(events[0].event_type, ISSUE_SCHEDULED);
        assert!(events
            .iter()
            .any(|event| event.event_type == AGENT_LAUNCH_REQUESTED));
    }

    #[test]
    fn start_issue_schedules_and_launches_project_issue() {
        let dir = tempdir().unwrap();
        write_project_with_issues(dir.path());

        let tick = TaskLoop::start_issue(dir.path(), "AF-TASK-001", "codex").unwrap();

        assert_eq!(
            tick.schedule
                .as_ref()
                .map(|schedule| schedule.issue_id.as_str()),
            Some("AF-TASK-001")
        );
        assert_eq!(tick.launch.issue_id, "AF-TASK-001");
        assert_eq!(tick.launch.project_id.as_deref(), Some("project-task-loop"));
        assert_eq!(
            tick.launch.branch_name,
            "agentflow/project-task-loop/AF-TASK-001"
        );
        assert!(dir.path().join(&tick.launch.launch_request_path).is_file());
        let events = replay_task_events(dir.path(), ReplayFilter::issue("AF-TASK-001")).unwrap();
        assert_eq!(
            events
                .iter()
                .filter_map(|event| event.state.as_ref())
                .last()
                .map(|state| state.to_state.as_str()),
            Some(SpecIssueStatus::InProgress.as_str())
        );
    }

    #[test]
    fn start_issue_supports_direct_issue() {
        let dir = tempdir().unwrap();
        write_requirement(dir.path());
        let requirement = dir.path().join("docs/requirements/034-test.md");
        let direct = agentflow_spec::issue_from_requirement(
            dir.path(),
            &requirement,
            SpecIssueDraft::new("AF-DIRECT-001"),
        )
        .unwrap();
        write_spec_issue(dir.path(), &direct).unwrap();

        let tick = TaskLoop::start_issue(dir.path(), "AF-DIRECT-001", "codex").unwrap();

        assert_eq!(
            tick.schedule
                .as_ref()
                .map(|schedule| schedule.project_id.as_str()),
            Some("direct")
        );
        assert_eq!(tick.launch.project_id, None);
        assert_eq!(tick.launch.branch_name, "agentflow/direct/AF-DIRECT-001");
        assert!(dir.path().join(&tick.launch.launch_request_path).is_file());
    }

    #[test]
    fn start_issue_rejects_unfinished_dependency() {
        let dir = tempdir().unwrap();
        write_project_with_issues(dir.path());

        let err = TaskLoop::start_issue(dir.path(), "AF-TASK-002", "codex").unwrap_err();

        assert!(err.to_string().contains("unfinished dependencies"));
    }

    #[test]
    fn schedule_next_issue_respects_project_issue_order_over_priority() {
        let dir = tempdir().unwrap();
        write_ordered_project_without_dependencies(dir.path());
        let loop_driver = TaskLoop::new("project-task-loop");

        let scheduled = loop_driver
            .schedule_next_issue(dir.path())
            .unwrap()
            .unwrap();

        assert_eq!(scheduled.issue_id, "AF-TASK-001");
    }

    #[test]
    fn start_issue_rejects_unfinished_project_predecessor() {
        let dir = tempdir().unwrap();
        write_ordered_project_without_dependencies(dir.path());

        let err = TaskLoop::start_issue(dir.path(), "AF-TASK-002", "codex").unwrap_err();

        assert!(err.to_string().contains("unfinished project predecessors"));
    }

    #[test]
    fn start_issue_rejects_second_active_issue_in_same_project() {
        let dir = tempdir().unwrap();
        write_ordered_project_without_dependencies(dir.path());
        let loop_driver = TaskLoop::new("project-task-loop");
        loop_driver
            .schedule_next_issue(dir.path())
            .unwrap()
            .unwrap();
        let issue = read_spec_issue(dir.path(), "AF-TASK-002").unwrap();
        append_issue_scheduled_event(
            dir.path(),
            &issue,
            &[
                ISSUE_CONTRACT_COMPLETE_GUARD,
                DEPENDENCIES_DONE_GUARD,
                PROJECT_PREDECESSORS_DONE_GUARD,
                PROJECT_SERIAL_SLOT_FREE_GUARD,
            ],
        )
        .unwrap();

        let err = TaskLoop::start_issue(dir.path(), "AF-TASK-001", "codex").unwrap_err();

        assert!(err.to_string().contains("active issue"));
    }

    #[test]
    fn tick_launches_existing_todo_without_rescheduling() {
        let dir = tempdir().unwrap();
        write_project_with_issues(dir.path());
        let loop_driver = TaskLoop::new("project-task-loop");
        loop_driver
            .schedule_next_issue(dir.path())
            .unwrap()
            .unwrap();

        let tick = loop_driver.tick(dir.path(), "codex").unwrap().unwrap();

        assert!(tick.schedule.is_none());
        assert_eq!(tick.launch.issue_id, "AF-TASK-001");
        let events = replay_task_events(dir.path(), ReplayFilter::issue("AF-TASK-001")).unwrap();
        assert_eq!(
            events
                .iter()
                .filter(|event| event.event_type == ISSUE_SCHEDULED)
                .count(),
            1
        );
    }

    #[test]
    fn tick_does_not_duplicate_pending_launch() {
        let dir = tempdir().unwrap();
        write_project_with_issues(dir.path());
        let loop_driver = TaskLoop::new("project-task-loop");

        let first_tick = loop_driver.tick(dir.path(), "codex").unwrap().unwrap();
        let second_tick = loop_driver.tick(dir.path(), "codex").unwrap().unwrap();

        assert_eq!(first_tick.launch.issue_id, "AF-TASK-001");
        assert_eq!(second_tick.launch.issue_id, "AF-TASK-001");
        assert!(second_tick.schedule.is_none());
        let events = replay_task_events(dir.path(), ReplayFilter::issue("AF-TASK-001")).unwrap();
        assert_eq!(
            events
                .iter()
                .filter(|event| event.event_type == AGENT_LAUNCH_REQUESTED)
                .count(),
            1
        );
    }

    #[test]
    fn tick_reuses_interrupted_in_progress_launch_before_new_schedule() {
        let dir = tempdir().unwrap();
        write_project_with_issues(dir.path());
        let loop_driver = TaskLoop::new("project-task-loop");
        let first_tick = loop_driver.tick(dir.path(), "codex").unwrap().unwrap();

        append_task_event_once(
            dir.path(),
            TaskEventDraft {
                flow_type: WorkflowFlowType::Work,
                aggregate_type: "issue".to_string(),
                aggregate_id: "AF-TASK-001".to_string(),
                project_id: Some("project-task-loop".to_string()),
                issue_id: Some("AF-TASK-001".to_string()),
                run_id: Some(first_tick.launch.run_id.clone()),
                event_type: "agent.session.interrupted".to_string(),
                authority_role: Some(WorkflowAgentRole::WorkAgent),
                actor: EventActor {
                    role: "agent-dispatcher".to_string(),
                    kind: "system".to_string(),
                },
                state: None,
                correlation_id: Some("corr-AF-TASK-001".to_string()),
                causation_id: None,
                payload: json!({
                    "issueId": "AF-TASK-001",
                    "projectId": "project-task-loop",
                    "runId": first_tick.launch.run_id,
                    "sessionId": "codex-run-001",
                    "sessionStatus": "interrupted",
                }),
                artifact_refs: Vec::new(),
                idempotency_key: Some("agent.session.interrupted:AF-TASK-001:run-001".to_string()),
            },
        )
        .unwrap();

        let resumed_tick = loop_driver.tick(dir.path(), "codex").unwrap().unwrap();

        assert!(resumed_tick.schedule.is_none());
        assert_eq!(resumed_tick.launch.issue_id, "AF-TASK-001");
        assert_eq!(resumed_tick.launch.run_id, "run-001");
    }

    #[test]
    fn launch_rejects_existing_launch_request() {
        let dir = tempdir().unwrap();
        write_project_with_issues(dir.path());
        let loop_driver = TaskLoop::new("project-task-loop");
        loop_driver
            .schedule_next_issue(dir.path())
            .unwrap()
            .unwrap();
        loop_driver
            .request_agent_launch(dir.path(), "AF-TASK-001", "codex")
            .unwrap();

        let err = loop_driver
            .request_agent_launch(dir.path(), "AF-TASK-001", "codex")
            .unwrap_err();

        assert!(err.to_string().contains("already has a launch request"));
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

    #[test]
    fn tick_does_not_schedule_second_issue_while_project_has_in_review_issue() {
        let dir = tempdir().unwrap();
        write_ordered_project_without_dependencies(dir.path());
        let loop_driver = TaskLoop::new("project-task-loop");
        let first_tick = loop_driver.tick(dir.path(), "codex").unwrap().unwrap();

        append_task_event_once(
            dir.path(),
            TaskEventDraft {
                flow_type: WorkflowFlowType::Work,
                aggregate_type: "issue".to_string(),
                aggregate_id: "AF-TASK-001".to_string(),
                project_id: Some("project-task-loop".to_string()),
                issue_id: Some("AF-TASK-001".to_string()),
                run_id: Some(first_tick.launch.run_id),
                event_type: "agent.session.in_review".to_string(),
                authority_role: Some(WorkflowAgentRole::WorkAgent),
                actor: EventActor {
                    role: "build-agent".to_string(),
                    kind: "agent".to_string(),
                },
                state: Some(EventStateTransition {
                    from_state: SpecIssueStatus::InProgress.as_str().to_string(),
                    to_state: SpecIssueStatus::InReview.as_str().to_string(),
                }),
                correlation_id: Some("corr-AF-TASK-001".to_string()),
                causation_id: None,
                payload: json!({
                    "issueId": "AF-TASK-001",
                    "projectId": "project-task-loop",
                    "runId": "run-001",
                }),
                artifact_refs: Vec::new(),
                idempotency_key: Some("agent.session.in_review:AF-TASK-001:run-001".to_string()),
            },
        )
        .unwrap();

        let tick = loop_driver.tick(dir.path(), "codex").unwrap();

        assert!(tick.is_none());
    }
}
