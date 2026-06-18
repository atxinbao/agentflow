use crate::{
    model::{
        CompletionDecisionIndex, CompletionDecisionIndexEntry, CompletionDecisionProjection,
        IssueStatusIndex, IssueStatusIndexEntry, ProjectBlockerSummary, ProjectBrainProjection,
        ProjectCompletionProjection, ProjectIssueLanes, ProjectProjection,
        ProjectionAuditSummary, ProjectionDeliverySummary, ProjectionPhase,
        ProjectionPublicDelivery, ProjectionRuntimeSummary, ProjectionSessionSummary,
        ProjectionSummary, RequirementPreviewIndex, RequirementPreviewIndexEntry,
        RequirementPreviewProjection, TaskProjection, TaskTimelineEvent, TaskTimelineItem,
        COMPLETION_DECISION_INDEX_VERSION, COMPLETION_DECISION_PROJECTION_VERSION,
        ISSUE_STATUS_INDEX_VERSION, PROJECT_PROJECTION_VERSION,
        REQUIREMENT_PREVIEW_INDEX_VERSION, REQUIREMENT_PREVIEW_PROJECTION_VERSION,
        TASK_PROJECTION_VERSION,
    },
    storage::{
        write_completion_decision_index, write_completion_decision_projection,
        write_issue_status_index, write_project_projection, write_requirement_preview_index,
        write_requirement_preview_projection, write_task_projection,
    },
};
use agentflow_audit::load_audit_result_summary;
use agentflow_event_store::{load_task_events, EventStateTransition, TaskEvent};
use agentflow_release::{load_delivery_summary, load_project_delivery_summary};
use agentflow_spec::{
    list_completion_decision_runtimes, list_requirement_preview_runtimes, prepare_spec_workspace,
    read_project_brain_snapshot, sync_completion_decision_runtimes, CompletionDecisionRuntime,
    RequirementPreviewRuntime, SpecIssue, SpecProject, SpecProjectStatus,
};
use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    fs,
    path::{Path, PathBuf},
};

const STATE_ORDER: [&str; 5] = ["backlog", "todo", "in_progress", "in_review", "done"];
const AGENT_LAUNCH_CLAIMED_EVENT: &str = "agent.launch.claimed";

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProjectionAuditIndexFile {
    #[serde(default)]
    audits: Vec<ProjectionAuditIndexEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProjectionAuditIndexEntry {
    audit_id: String,
    status: String,
    requested_at: u64,
    source_run_id: Option<String>,
    source_issue_id: Option<String>,
    report_path: String,
}

pub fn rebuild_projections(project_root: impl AsRef<Path>) -> Result<ProjectionSummary> {
    let root = canonical_project_root(project_root)?;
    prepare_spec_workspace(&root)?;
    let completion_runtimes = sync_completion_decision_runtimes(&root).unwrap_or_default();
    let issues = read_json_files::<SpecIssue>(&root.join(".agentflow/spec/issues"))?;
    let projects = read_json_files::<SpecProject>(&root.join(".agentflow/spec/projects"))?;
    let requirement_previews = list_requirement_preview_runtimes(&root).unwrap_or_default();
    let completion_runtimes = if completion_runtimes.is_empty() {
        list_completion_decision_runtimes(&root).unwrap_or_default()
    } else {
        completion_runtimes
    };
    let completion_by_project = completion_runtimes
        .iter()
        .map(|runtime| (runtime.project_id.clone(), runtime))
        .collect::<HashMap<_, _>>();
    let events = load_task_events(&root)?;
    let audit_index = load_projection_audit_index(&root).unwrap_or_default();
    let events_by_issue = group_events_by_issue(events);
    let issues_by_id = issues
        .iter()
        .map(|issue| (issue.issue_id.clone(), issue))
        .collect::<HashMap<_, _>>();
    let mut task_projections = BTreeMap::new();

    for issue in &issues {
        let projection = project_issue(
            &root,
            issue,
            events_by_issue.get(&issue.issue_id),
            &audit_index,
        );
        write_task_projection(&root, &projection)?;
        task_projections.insert(issue.issue_id.clone(), projection);
    }

    for project in &projects {
        let projection = project_project(
            &root,
            project,
            &issues_by_id,
            &task_projections,
            completion_by_project.get(&project.project_id).copied(),
        )?;
        write_project_projection(&root, &projection)?;
    }

    let mut requirement_preview_entries = Vec::new();
    for preview in &requirement_previews {
        let projection = project_requirement_preview(preview);
        let projection_path =
            write_requirement_preview_projection(&root, &projection)?.display().to_string();
        requirement_preview_entries.push(RequirementPreviewIndexEntry {
            requirement_id: preview.requirement_id.clone(),
            project_id: preview.project_id.clone(),
            current_state: projection.current_state.clone(),
            lifecycle: projection.lifecycle.clone(),
            next_recommended_action: projection.next_recommended_action.clone(),
            projection_path: relative_projection_path(&root, &projection_path),
            updated_at: projection.updated_at,
        });
    }
    requirement_preview_entries
        .sort_by(|left, right| left.requirement_id.cmp(&right.requirement_id));
    write_requirement_preview_index(
        &root,
        &RequirementPreviewIndex {
            version: REQUIREMENT_PREVIEW_INDEX_VERSION.to_string(),
            updated_at: requirement_preview_entries
                .iter()
                .map(|entry| entry.updated_at)
                .max()
                .unwrap_or_default(),
            previews: requirement_preview_entries,
        },
    )?;

    let mut completion_entries = Vec::new();
    for runtime in &completion_runtimes {
        let projection = project_completion_decision(runtime);
        let projection_path =
            write_completion_decision_projection(&root, &projection)?.display().to_string();
        completion_entries.push(CompletionDecisionIndexEntry {
            project_id: runtime.project_id.clone(),
            current_state: projection.current_state.clone(),
            latest_outcome: projection.latest_outcome.clone(),
            next_recommended_action: projection.next_recommended_action.clone(),
            projection_path: relative_projection_path(&root, &projection_path),
            updated_at: projection.updated_at,
        });
    }
    completion_entries.sort_by(|left, right| left.project_id.cmp(&right.project_id));
    write_completion_decision_index(
        &root,
        &CompletionDecisionIndex {
            version: COMPLETION_DECISION_INDEX_VERSION.to_string(),
            updated_at: completion_entries
                .iter()
                .map(|entry| entry.updated_at)
                .max()
                .unwrap_or_default(),
            decisions: completion_entries,
        },
    )?;

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

fn project_requirement_preview(
    preview: &RequirementPreviewRuntime,
) -> RequirementPreviewProjection {
    RequirementPreviewProjection {
        version: REQUIREMENT_PREVIEW_PROJECTION_VERSION.to_string(),
        requirement_id: preview.requirement_id.clone(),
        requirement_path: preview.requirement_path.clone(),
        project_id: preview.project_id.clone(),
        project_title: preview.project_title.clone(),
        lifecycle: preview.lifecycle.as_str().to_string(),
        current_state: preview.current_state.clone(),
        goal_status: preview.goal_draft.status.as_str().to_string(),
        plan_status: preview
            .plan_draft
            .as_ref()
            .map(|draft| draft.status.as_str().to_string()),
        next_recommended_action: preview.next_recommended_action.clone(),
        next_recommended_action_label: preview.next_recommended_action_label.clone(),
        next_recommended_action_reason: preview.next_recommended_action_reason.clone(),
        issue_contract_draft_count: preview
            .plan_draft
            .as_ref()
            .map(|draft| draft.issue_contract_drafts.len())
            .unwrap_or_default(),
        materialized_project_id: preview.materialized_project_id.clone(),
        materialized_issue_ids: preview.materialized_issue_ids.clone(),
        updated_at: preview.updated_at,
    }
}

fn project_completion_decision(
    runtime: &CompletionDecisionRuntime,
) -> CompletionDecisionProjection {
    CompletionDecisionProjection {
        version: COMPLETION_DECISION_PROJECTION_VERSION.to_string(),
        project_id: runtime.project_id.clone(),
        project_title: runtime.project_title.clone(),
        current_state: runtime.current_state.as_str().to_string(),
        latest_outcome: runtime
            .latest_outcome
            .as_ref()
            .map(|outcome| outcome.as_str().to_string()),
        next_recommended_action: runtime.next_recommended_action.clone(),
        next_recommended_action_label: runtime.next_recommended_action_label.clone(),
        next_recommended_action_reason: runtime.next_recommended_action_reason.clone(),
        total_issue_count: runtime.facts.total_issue_count,
        completed_issue_count: runtime.facts.completed_issue_count,
        canceled_issue_count: runtime.facts.canceled_issue_count,
        remaining_issue_count: runtime.facts.remaining_issue_count,
        blocked_issue_count: runtime.facts.blocked_issue_count,
        open_questions: runtime.open_questions.clone(),
        rationale: runtime.rationale.clone(),
        projection_path: format!(
            ".agentflow/projections/completions/{}.json",
            runtime.project_id
        ),
        updated_at: runtime.updated_at,
    }
}

fn project_issue(
    root: &Path,
    issue: &SpecIssue,
    events: Option<&Vec<TaskEvent>>,
    audit_index: &ProjectionAuditIndexFile,
) -> TaskProjection {
    let events = events.cloned().unwrap_or_default();
    let mut current_state = issue.status.as_str().to_string();
    let mut updated_at = issue.system.updated_at;
    let mut latest_run_id = None;
    let mut authoritative_run_id = None;
    let mut active_run_id = None;
    let mut branch_name = None;
    let mut public_delivery = ProjectionPublicDelivery {
        evidence_path: Some(issue.expected_outputs.evidence_path.clone()),
        ..ProjectionPublicDelivery::default()
    };
    let mut state_events = Vec::new();

    for event in events {
        updated_at = updated_at.max(event.timestamp);
        let event_run_id = event
            .run_id
            .as_deref()
            .or_else(|| event.payload.get("runId").and_then(Value::as_str))
            .map(str::to_string);
        let should_track = should_track_issue_event(
            &current_state,
            active_run_id.as_deref(),
            event_run_id.as_deref(),
            event.event_type.as_str(),
        );
        if !should_track {
            continue;
        }
        if let Some(run_id) = event_run_id.as_deref() {
            latest_run_id = Some(run_id.to_string());
        }
        if event.event_type == "agent.launch.requested" {
            active_run_id = event_run_id.clone().or(active_run_id);
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
        if let Some(next_state) = authoritative_state_from_event(&event) {
            current_state = next_state;
            if let Some(run_id) = event_run_id.as_deref() {
                authoritative_run_id = Some(run_id.to_string());
                active_run_id = Some(run_id.to_string());
            }
        }
        state_events.push((current_state.clone(), event));
    }
    let latest_run_id = authoritative_run_id
        .or(active_run_id)
        .or(latest_run_id);

    let session = build_session_summary(&state_events);
    let runtime = build_runtime_summary(
        root,
        issue,
        latest_run_id.as_deref(),
        branch_name.as_deref(),
        &session,
    );
    let delivery = build_delivery_summary(root, issue, &public_delivery);
    let audit = build_audit_summary(root, issue, latest_run_id.as_deref(), audit_index);
    branch_name = branch_name
        .or_else(|| runtime.branch_name.clone())
        .or_else(|| session.branch_name.clone());

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
        runtime,
        session,
        delivery,
        audit,
        updated_at,
    }
}

fn project_project(
    root: &Path,
    project: &SpecProject,
    issues_by_id: &HashMap<String, &SpecIssue>,
    tasks: &BTreeMap<String, TaskProjection>,
    completion: Option<&CompletionDecisionRuntime>,
) -> Result<ProjectProjection> {
    let mut current_issue_id = None;
    let mut current_issue_state = None;
    let mut completed = 0;
    let mut updated_at = project.system.updated_at;
    let mut current_lane = Vec::new();
    let mut past_lane = Vec::new();
    let mut future_lane = Vec::new();
    let mut blocked_lane = Vec::new();
    let mut blockers = Vec::new();
    for issue_id in &project.issue_ids {
        let Some(task) = tasks.get(issue_id) else {
            continue;
        };
        updated_at = updated_at.max(task.updated_at);
        match task.current_state.as_str() {
            "done" | "cancel" => {
                completed += 1;
                past_lane.push(issue_id.clone());
            }
            "backlog" => future_lane.push(issue_id.clone()),
            "blocked" => {
                current_lane.push(issue_id.clone());
                blocked_lane.push(issue_id.clone());
                let blocked_by = issues_by_id
                    .get(issue_id)
                    .map(|issue| issue.blocked_by.clone())
                    .unwrap_or_default();
                blockers.push(ProjectBlockerSummary {
                    issue_id: issue_id.clone(),
                    reason: if blocked_by.is_empty() {
                        "任务被阻断，等待补充阻断原因。".to_string()
                    } else {
                        format!("等待依赖 {} 完成。", blocked_by.join("、"))
                    },
                });
            }
            _ => {
                if current_issue_id.is_none() {
                    current_issue_id = Some(issue_id.clone());
                    current_issue_state = Some(task.current_state.clone());
                }
                current_lane.push(issue_id.clone());
            }
        }
    }
    let all_finished = completed == project.issue_ids.len() && !project.issue_ids.is_empty();
    let completion_projection = completion.map(project_completion_decision);
    let project_delivery = build_project_delivery_summary(root, project, tasks);
    let project_audit = build_project_audit_summary(project, tasks);
    let status = if completion_projection
        .as_ref()
        .is_some_and(|projection| projection.current_state == "accepted")
        && all_finished
    {
        "done"
    } else if !blocked_lane.is_empty()
        && current_lane.len() == blocked_lane.len()
        && future_lane.is_empty()
    {
        "blocked"
    } else if current_issue_id.is_some() || all_finished {
        "active"
    } else {
        project_status_as_str(&project.status)
    };
    let brain = read_project_brain_snapshot(root, &project.project_id, &project.title)?;
    let (stage_key, stage_label, stage_summary) = if completion_projection
        .as_ref()
        .is_some_and(|projection| projection.current_state == "accepted")
        && all_finished
    {
        (
            "done".to_string(),
            "项目已完成".to_string(),
            "全部任务已完成，完成判断已经接受。".to_string(),
        )
    } else if all_finished {
        (
            "completion-ready".to_string(),
            "等待完成判断".to_string(),
            "任务已全部完成，正在等待 Goal Recheck / Completion Runtime 做最后判断。".to_string(),
        )
    } else if let Some(issue_id) = current_issue_id.as_ref() {
        let label = match current_issue_state.as_deref() {
            Some("todo") => "准备开工",
            Some("in_review") => "正在评审",
            Some("blocked") => "已阻断",
            Some("in_progress") => "正在推进",
            _ => "正在推进",
        };
        let summary = match current_issue_state.as_deref() {
            Some("todo") => format!("{issue_id} 已进入待处理阶段，正在等待执行线程正式开工。"),
            Some("in_review") => format!("{issue_id} 已完成本地验证，当前正在等待评审收口。"),
            Some("blocked") => format!("{issue_id} 当前被阻断，项目节奏停在阻断处理。"),
            _ => format!("{issue_id} 正在推进，项目当前主节奏围绕这条任务展开。"),
        };
        ("active".to_string(), label.to_string(), summary)
    } else if let Some(issue_id) = future_lane.first() {
        (
            "ready-to-start".to_string(),
            "准备开工".to_string(),
            format!("当前还没有活跃任务，下一条待启动任务是 {issue_id}。"),
        )
    } else if !blocked_lane.is_empty() {
        (
            "blocked".to_string(),
            "已阻断".to_string(),
            "当前没有可继续推进的任务，项目停在阻断处理阶段。".to_string(),
        )
    } else {
        (
            "project-brain".to_string(),
            "等待项目判断".to_string(),
            "项目仍停留在 Project Brain / 调度判断阶段，尚未进入稳定任务循环。".to_string(),
        )
    };
    let (next_action, next_action_label, next_action_reason) = if let Some(issue_id) = current_issue_id.clone() {
        (
            format!("继续推进 {issue_id}。"),
            "继续当前任务".to_string(),
            stage_summary.clone(),
        )
    } else if let Some(completion) = completion_projection.as_ref() {
        (
            completion.next_recommended_action.clone(),
            completion.next_recommended_action_label.clone(),
            completion.next_recommended_action_reason.clone(),
        )
    } else if let Some(issue_id) = future_lane.first() {
        (
            format!("启动 {issue_id}。"),
            "启动下一条任务".to_string(),
            format!("{issue_id} 当前是项目下一条最直接的推进入口。"),
        )
    } else if !blocked_lane.is_empty() {
        (
            "先解除阻断项，再继续推进项目。".to_string(),
            "处理阻断项".to_string(),
            blockers
                .first()
                .map(|blocker| blocker.reason.clone())
                .unwrap_or_else(|| "当前存在阻断项，解除后才能继续推进项目。".to_string()),
        )
    } else if all_finished {
        (
            "进入完成判断".to_string(),
            "进入完成判断".to_string(),
            "任务已经全部完成，下一步需要判断项目是否真正结束。".to_string(),
        )
    } else {
        (
            brain.next_recommended_action.clone(),
            brain.next_recommended_action_label.clone(),
            brain.next_recommended_action_reason.clone(),
        )
    };
    let completion_hint = if let Some(completion) = completion_projection.as_ref() {
        append_audit_hint(
            append_delivery_hint(
                completion.next_recommended_action_reason.clone(),
                project_delivery.as_ref(),
            ),
            project_audit.as_ref(),
        )
    } else if all_finished {
        append_audit_hint(
            append_delivery_hint(
                "全部任务已完成，下一步由 Goal / Completion Runtime 重新判断项目是否真正结束。"
                    .to_string(),
                project_delivery.as_ref(),
            ),
            project_audit.as_ref(),
        )
    } else {
        append_audit_hint(
            append_delivery_hint(
                format!(
                    "当前已完成 {completed}/{} 条任务，继续按状态流推进。",
                    project.issue_ids.len()
                ),
                project_delivery.as_ref(),
            ),
            project_audit.as_ref(),
        )
    };
    Ok(ProjectProjection {
        version: PROJECT_PROJECTION_VERSION.to_string(),
        project_id: project.project_id.clone(),
        title: project.title.clone(),
        objective: project.objective.clone(),
        status: status.to_string(),
        stage_key,
        stage_label,
        stage_summary,
        issue_ids: project.issue_ids.clone(),
        current_issue_id,
        lanes: ProjectIssueLanes {
            current: current_lane,
            past: past_lane,
            future: future_lane,
            blocked: blocked_lane,
        },
        next_action,
        next_action_label,
        next_action_reason,
        blockers,
        completion_hint,
        completion: completion_projection.map(|projection| ProjectCompletionProjection {
            current_state: projection.current_state,
            latest_outcome: projection.latest_outcome,
            next_recommended_action: projection.next_recommended_action,
            next_recommended_action_label: projection.next_recommended_action_label,
            next_recommended_action_reason: projection.next_recommended_action_reason,
            total_issue_count: projection.total_issue_count,
            completed_issue_count: projection.completed_issue_count,
            canceled_issue_count: projection.canceled_issue_count,
            remaining_issue_count: projection.remaining_issue_count,
            blocked_issue_count: projection.blocked_issue_count,
            open_questions: projection.open_questions,
            rationale: projection.rationale,
            updated_at: projection.updated_at,
        }),
        delivery: project_delivery,
        audit: project_audit,
        issue_count: project.issue_ids.len(),
        completed_issue_count: completed,
        project_brain: ProjectBrainProjection {
            project_path: brain.project_path,
            goal_path: brain.goal_document,
            plan_path: brain.plan_document,
            decisions_path: brain.decisions_document,
            health_path: brain.health_document,
            brain_status: brain.brain_status.as_str().to_string(),
            goal_status: brain.goal_status.as_str().to_string(),
            plan_status: brain.plan_status.as_str().to_string(),
            decision_status: brain.decision_status.as_str().to_string(),
            health_status: brain.health_status.as_str().to_string(),
            missing_documents: brain.missing_documents,
            open_questions: brain.open_questions,
            next_recommended_action: brain.next_recommended_action,
            next_recommended_action_label: brain.next_recommended_action_label,
            next_recommended_action_reason: brain.next_recommended_action_reason,
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

fn build_runtime_summary(
    root: &Path,
    issue: &SpecIssue,
    run_id: Option<&str>,
    branch_name: Option<&str>,
    session: &ProjectionSessionSummary,
) -> ProjectionRuntimeSummary {
    let Some(run_id) = run_id else {
        return ProjectionRuntimeSummary::default();
    };

    let run = agentflow_task_artifacts::load_task_run(root, &issue.issue_id, run_id).ok();
    let checkpoints =
        agentflow_task_artifacts::load_task_run_checkpoints(root, &issue.issue_id, run_id)
            .unwrap_or_default();
    let latest_checkpoint = checkpoints.last();

    ProjectionRuntimeSummary {
        run_id: Some(run_id.to_string()),
        run_status: normalize_runtime_run_status(
            run.as_ref().map(|run| task_run_status_as_str(&run.status)),
            session.status.as_deref(),
        ),
        branch_name: branch_name
            .map(str::to_string)
            .or_else(|| run.as_ref().and_then(|run| run.branch_name.clone())),
        checkpoint_count: checkpoints.len(),
        latest_checkpoint_id: latest_checkpoint.map(|checkpoint| checkpoint.checkpoint_id.clone()),
        latest_checkpoint_state: latest_checkpoint.map(|checkpoint| checkpoint.state.clone()),
        latest_checkpoint_summary: latest_checkpoint.map(|checkpoint| checkpoint.summary.clone()),
    }
}

fn normalize_runtime_run_status(
    run_status: Option<&str>,
    session_status: Option<&str>,
) -> String {
    match session_status {
        Some("requested" | "queued" | "claimed" | "starting") => "queued".to_string(),
        Some("running" | "interrupted") => "in_progress".to_string(),
        Some("in-review" | "done") => "validating".to_string(),
        Some("failed") => "failed".to_string(),
        Some("cancelled") => "cancelled".to_string(),
        _ => run_status.unwrap_or("missing").to_string(),
    }
}

fn build_session_summary(state_events: &[(String, TaskEvent)]) -> ProjectionSessionSummary {
    let mut summary = ProjectionSessionSummary::default();
    for (_, event) in state_events {
        match event.event_type.as_str() {
            "agent.launch.requested" => {
                summary.provider = event
                    .payload
                    .get("provider")
                    .and_then(Value::as_str)
                    .map(str::to_string)
                    .or_else(|| summary.provider.clone());
                summary.status = Some("requested".to_string());
                summary.launch_requested_at = Some(event.timestamp);
                summary.updated_at = Some(event.timestamp);
                summary.launch_request_path = event
                    .payload
                    .get("launchRequestPath")
                    .and_then(Value::as_str)
                    .map(str::to_string)
                    .or_else(|| summary.launch_request_path.clone());
                summary.branch_name = event
                    .payload
                    .get("branchName")
                    .and_then(Value::as_str)
                    .map(str::to_string)
                    .or_else(|| summary.branch_name.clone());
            }
            AGENT_LAUNCH_CLAIMED_EVENT
            | "agent.session.created"
            | "agent.session.resumed"
            | "agent.session.running"
            | "agent.session.interrupted"
            | "agent.session.in_review"
            | "agent.session.completed"
            | "agent.session.failed"
            | "agent.session.cancelled" => {
                summary.provider = event
                    .payload
                    .get("provider")
                    .and_then(Value::as_str)
                    .map(str::to_string)
                    .or_else(|| summary.provider.clone());
                summary.session_id = event
                    .payload
                    .get("sessionId")
                    .and_then(Value::as_str)
                    .map(str::to_string)
                    .or_else(|| summary.session_id.clone());
                summary.status = event
                    .payload
                    .get("sessionStatus")
                    .and_then(Value::as_str)
                    .map(str::to_string)
                    .or_else(|| summary.status.clone())
                    .or_else(|| fallback_session_status(event.event_type.as_str()));
                summary.updated_at = Some(event.timestamp);
                summary.launch_request_path = event
                    .payload
                    .get("launchRequestPath")
                    .and_then(Value::as_str)
                    .map(str::to_string)
                    .or_else(|| summary.launch_request_path.clone());
                summary.plan_path = event
                    .payload
                    .get("planPath")
                    .and_then(Value::as_str)
                    .map(str::to_string)
                    .or_else(|| summary.plan_path.clone());
                summary.log_path = event
                    .payload
                    .get("logPath")
                    .and_then(Value::as_str)
                    .map(str::to_string)
                    .or_else(|| summary.log_path.clone());
                summary.branch_name = event
                    .payload
                    .get("branchName")
                    .and_then(Value::as_str)
                    .map(str::to_string)
                    .or_else(|| summary.branch_name.clone());
                if event.event_type == AGENT_LAUNCH_CLAIMED_EVENT {
                    summary.claimed_at = Some(event.timestamp);
                }
                if matches!(
                    event.event_type.as_str(),
                    "agent.session.created" | "agent.session.resumed"
                ) && summary.created_at.is_none()
                {
                    summary.created_at = Some(event.timestamp);
                }
            }
            _ => {}
        }
    }
    summary
}

fn build_delivery_summary(
    root: &Path,
    issue: &SpecIssue,
    public_delivery: &ProjectionPublicDelivery,
) -> ProjectionDeliverySummary {
    if let Ok(summary) = load_delivery_summary(root, &issue.issue_id) {
        return ProjectionDeliverySummary {
            status: summary.status,
            evidence_status: summary.evidence_status,
            evidence_path: summary.evidence_path,
            pr_url: summary.pr_url,
            merge_commit: summary.merge_commit,
            public_record_path: summary.public_record_path,
            summary_line: summary.summary_line,
            public_record_items: summary.public_record_items,
            missing_public_records: summary.missing_public_records,
            current_issue_id: None,
            published_count: 0,
            ready_count: 0,
            missing_count: 0,
        };
    }

    let evidence = agentflow_task_artifacts::load_task_evidence(root, &issue.issue_id).ok();
    let public_record_path = public_delivery
        .changelog_path
        .clone()
        .or_else(|| public_delivery.release_notes_url.clone())
        .filter(|path| root.join(path).is_file());
    let evidence_status = if evidence.is_some() {
        "ready".to_string()
    } else {
        "missing".to_string()
    };
    let status = if public_record_path.is_some() {
        "published".to_string()
    } else if public_delivery.pr_url.is_some() || public_delivery.merge_commit.is_some() {
        "ready".to_string()
    } else {
        "missing".to_string()
    };
    let mut public_record_items = Vec::new();
    if public_delivery.pr_url.is_some() {
        public_record_items.push("PR/MR body".to_string());
    }
    if let Some(path) = public_delivery.changelog_path.clone() {
        public_record_items.push(path);
    }
    if let Some(path) = public_delivery.release_notes_url.clone() {
        if !public_record_items.iter().any(|item| item == &path) {
            public_record_items.push(path);
        }
    }

    ProjectionDeliverySummary {
        status: status.clone(),
        evidence_status,
        evidence_path: evidence
            .as_ref()
            .map(|_| issue.expected_outputs.evidence_path.clone()),
        pr_url: public_delivery.pr_url.clone(),
        merge_commit: public_delivery.merge_commit.clone(),
        public_record_path,
        summary_line: match status.as_str() {
            "published" => format!("公开交付已整理到 {}。", public_record_items.join("、")),
            "ready" => "公开交付已开始整理，等待写入 CHANGELOG 或 release notes。".to_string(),
            _ => "当前还没有公开交付记录。".to_string(),
        },
        public_record_items,
        missing_public_records: if status == "ready" {
            vec!["CHANGELOG.md 或 release notes".to_string()]
        } else {
            Vec::new()
        },
        current_issue_id: None,
        published_count: 0,
        ready_count: 0,
        missing_count: 0,
    }
}

fn build_project_delivery_summary(
    root: &Path,
    project: &SpecProject,
    tasks: &BTreeMap<String, TaskProjection>,
) -> Option<ProjectionDeliverySummary> {
    if let Ok(Some(summary)) = load_project_delivery_summary(root, &project.project_id) {
        return Some(ProjectionDeliverySummary {
            status: summary.status,
            evidence_status: "ready".to_string(),
            evidence_path: None,
            pr_url: None,
            merge_commit: None,
            public_record_path: summary.public_record_items.first().cloned(),
            summary_line: summary.summary_line,
            public_record_items: summary.public_record_items,
            missing_public_records: summary.missing_public_records,
            current_issue_id: summary.current_issue_id,
            published_count: summary.published_count,
            ready_count: summary.ready_count,
            missing_count: summary.missing_count,
        });
    }

    let issue_ids = project.issue_ids.iter().cloned().collect::<BTreeSet<_>>();
    let summaries = tasks
        .values()
        .filter(|task| issue_ids.contains(&task.issue_id))
        .map(|task| task.delivery.clone())
        .collect::<Vec<_>>();
    if summaries.is_empty() {
        return None;
    }
    let published_count = summaries
        .iter()
        .filter(|summary| summary.status == "published")
        .count();
    let ready_count = summaries
        .iter()
        .filter(|summary| summary.status == "ready")
        .count();
    let missing_count = summaries
        .iter()
        .filter(|summary| summary.status == "missing")
        .count();
    let public_record_items = summaries
        .iter()
        .flat_map(|summary| summary.public_record_items.clone())
        .collect::<Vec<_>>();
    Some(ProjectionDeliverySummary {
        status: if missing_count == 0 && ready_count == 0 && published_count > 0 {
            "published".to_string()
        } else if published_count > 0 || ready_count > 0 {
            "ready".to_string()
        } else {
            "missing".to_string()
        },
        evidence_status: "ready".to_string(),
        evidence_path: None,
        pr_url: None,
        merge_commit: None,
        public_record_path: public_record_items.first().cloned(),
        summary_line: if missing_count > 0 {
            "项目仍有任务缺少公开交付记录。".to_string()
        } else if public_record_items.is_empty() {
            "当前项目还没有公开交付记录。".to_string()
        } else {
            format!("项目公开交付已汇总到 {}。", public_record_items.join("、"))
        },
        public_record_items,
        missing_public_records: if missing_count > 0 {
            vec!["CHANGELOG.md 或 release notes".to_string()]
        } else {
            Vec::new()
        },
        current_issue_id: project
            .issue_ids
            .iter()
            .find(|issue_id| {
                tasks
                    .get(*issue_id)
                    .is_some_and(|task| !matches!(task.current_state.as_str(), "done" | "cancel"))
            })
            .cloned(),
        published_count,
        ready_count,
        missing_count,
    })
}

fn build_audit_summary(
    root: &Path,
    issue: &SpecIssue,
    run_id: Option<&str>,
    audit_index: &ProjectionAuditIndexFile,
) -> ProjectionAuditSummary {
    let audit = audit_index.audits.iter().rev().find(|entry| {
        entry.source_issue_id.as_deref() == Some(issue.issue_id.as_str())
            || run_id.is_some_and(|run_id| entry.source_run_id.as_deref() == Some(run_id))
    });

    let audit_result = audit.and_then(|entry| load_audit_result_summary(root, entry.audit_id.clone()).ok());

    ProjectionAuditSummary {
        status: audit
            .map(|entry| entry.status.clone())
            .unwrap_or_else(|| "not-requested".to_string()),
        latest_audit_id: audit.map(|entry| entry.audit_id.clone()),
        source_issue_id: audit
            .and_then(|entry| entry.source_issue_id.clone())
            .or_else(|| audit_result.as_ref().and_then(|summary| summary.source_issue_id.clone())),
        report_path: audit
            .map(|entry| entry.report_path.clone())
            .or_else(|| audit_result.as_ref().map(|summary| summary.report_path.clone())),
        requested_at: audit
            .map(|entry| entry.requested_at)
            .or_else(|| audit_result.as_ref().map(|summary| summary.requested_at)),
        summary_line: audit_result
            .as_ref()
            .map(|summary| summary.summary_line.clone())
            .unwrap_or_else(|| {
                audit
                    .map(|entry| format!("审计状态：{}。", entry.status))
                    .unwrap_or_else(|| "当前没有审计请求。".to_string())
            }),
        findings_count: audit_result
            .as_ref()
            .map(|summary| summary.findings_count)
            .unwrap_or(0),
        findings: audit_result
            .as_ref()
            .map(|summary| summary.findings.clone())
            .unwrap_or_default(),
        evidence_gaps: audit_result
            .as_ref()
            .map(|summary| summary.evidence_gaps.clone())
            .unwrap_or_default(),
        repair_recommendations: audit_result
            .as_ref()
            .map(|summary| summary.repair_recommendations.clone())
            .unwrap_or_default(),
    }
}

fn build_project_audit_summary(
    project: &SpecProject,
    tasks: &BTreeMap<String, TaskProjection>,
) -> Option<ProjectionAuditSummary> {
    project
        .issue_ids
        .iter()
        .filter_map(|issue_id| tasks.get(issue_id).map(|task| &task.audit))
        .filter(|audit| audit.status != "not-requested")
        .max_by(|left, right| {
            left.requested_at
                .unwrap_or_default()
                .cmp(&right.requested_at.unwrap_or_default())
                .then_with(|| left.latest_audit_id.cmp(&right.latest_audit_id))
        })
        .cloned()
}

fn append_audit_hint(base: String, audit: Option<&ProjectionAuditSummary>) -> String {
    let Some(audit) = audit else {
        return base;
    };
    if audit.status == "not-requested" || audit.summary_line.trim().is_empty() {
        return base;
    }
    format!("{base} 最近审计：{}", audit.summary_line)
}

fn append_delivery_hint(base: String, delivery: Option<&ProjectionDeliverySummary>) -> String {
    let Some(delivery) = delivery else {
        return base;
    };
    if delivery.summary_line.trim().is_empty() {
        return base;
    }
    format!("{base} 最近交付：{}", delivery.summary_line)
}

fn load_projection_audit_index(root: &Path) -> Result<ProjectionAuditIndexFile> {
    let path = root.join(".agentflow/audit/index.json");
    if !path.is_file() {
        return Ok(ProjectionAuditIndexFile::default());
    }
    let raw = fs::read_to_string(&path)
        .with_context(|| format!("failed to read audit index {}", path.display()))?;
    serde_json::from_str(&raw)
        .with_context(|| format!("failed to parse audit index {}", path.display()))
}

fn task_run_status_as_str(status: &agentflow_task_artifacts::TaskRunStatus) -> &'static str {
    match status {
        agentflow_task_artifacts::TaskRunStatus::Queued => "queued",
        agentflow_task_artifacts::TaskRunStatus::InProgress => "in_progress",
        agentflow_task_artifacts::TaskRunStatus::Validating => "validating",
        agentflow_task_artifacts::TaskRunStatus::Completed => "completed",
        agentflow_task_artifacts::TaskRunStatus::Failed => "failed",
        agentflow_task_artifacts::TaskRunStatus::Cancelled => "cancelled",
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

fn authoritative_state_from_event(event: &TaskEvent) -> Option<String> {
    if let Some(EventStateTransition { to_state, .. }) = event.state.as_ref() {
        return Some(to_state.clone());
    }
    match event.event_type.as_str() {
        "issue.scheduled" => Some("todo".to_string()),
        "agent.launch.requested" => Some("in_progress".to_string()),
        "issue.validation.passed"
        | "issue.review.requested"
        | "issue.pr.created"
        | "issue.merge.proof.recorded"
        | "issue.pr.merged" => Some("in_review".to_string()),
        "issue.completed" => Some("done".to_string()),
        "issue.blocked" | "issue.validation.failed" => Some("blocked".to_string()),
        "issue.cancelled" => Some("cancel".to_string()),
        _ => None,
    }
}

fn should_track_issue_event(
    current_state: &str,
    active_run_id: Option<&str>,
    event_run_id: Option<&str>,
    event_type: &str,
) -> bool {
    let Some(event_run_id) = event_run_id else {
        return true;
    };
    if event_type == "agent.launch.requested" {
        return true;
    }
    match active_run_id {
        None => true,
        Some(active_run_id) if active_run_id == event_run_id => true,
        Some(_) if matches!(current_state, "in_review" | "done") => false,
        Some(_) => false,
    }
}

fn fallback_session_status(event_type: &str) -> Option<String> {
    match event_type {
        AGENT_LAUNCH_CLAIMED_EVENT => Some("claimed".to_string()),
        "agent.session.created" => Some("queued".to_string()),
        "agent.session.resumed" => Some("running".to_string()),
        "agent.session.running" => Some("running".to_string()),
        "agent.session.interrupted" => Some("interrupted".to_string()),
        "agent.session.in_review" => Some("in-review".to_string()),
        "agent.session.completed" => Some("done".to_string()),
        "agent.session.failed" => Some("failed".to_string()),
        "agent.session.cancelled" => Some("cancelled".to_string()),
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

fn relative_projection_path(root: &Path, absolute_path: &str) -> String {
    let path = Path::new(absolute_path);
    path.strip_prefix(root)
        .ok()
        .map(|relative| relative.display().to_string())
        .unwrap_or_else(|| absolute_path.to_string())
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
    use agentflow_spec::{
        read_spec_issue, requirement_preview_from_requirement, write_spec_issue,
        CompletionDecisionOutcome, SpecIssueDraft, SpecIssueStatus, SpecProjectDraft,
    };
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

    fn write_audit_fixture(root: &Path, issue_id: &str, run_id: &str, audit_id: &str) {
        let audit_dir = root.join(".agentflow/audit").join(audit_id);
        fs::create_dir_all(&audit_dir).unwrap();
        fs::create_dir_all(root.join(".agentflow/audit")).unwrap();
        fs::write(
            root.join(".agentflow/audit/index.json"),
            serde_json::to_string_pretty(&json!({
                "version": "audit-index.v1",
                "updatedAt": 300,
                "audits": [
                    {
                        "auditId": audit_id,
                        "status": "failed",
                        "trigger": "human-via-agent",
                        "requestedBy": "human-via-agent",
                        "requestedAt": 300,
                        "sourceDeliveryId": null,
                        "sourceRunId": run_id,
                        "sourceIssueId": issue_id,
                        "sourceSpecId": "spec-projection",
                        "reportPath": format!(".agentflow/audit/{audit_id}/audit-report.md"),
                        "auditPath": format!(".agentflow/audit/{audit_id}/audit.json")
                    }
                ]
            }))
            .unwrap(),
        )
        .unwrap();
        fs::write(
            audit_dir.join("audit-request.json"),
            serde_json::to_string_pretty(&json!({
                "version": "audit-request.v1",
                "auditId": audit_id,
                "trigger": "human-via-agent",
                "requestedBy": "human-via-agent",
                "requestedAt": 300,
                "reason": "检查交付完整性",
                "scope": {
                    "description": "检查交付链路",
                    "refs": [
                        {"kind": "issue", "id": issue_id, "path": format!(".agentflow/spec/issues/{issue_id}.json")},
                        {"kind": "task-run", "id": run_id, "path": format!(".agentflow/tasks/{issue_id}/runs/{run_id}/run.json")}
                    ]
                }
            }))
            .unwrap(),
        )
        .unwrap();
        fs::write(
            audit_dir.join("audit.json"),
            serde_json::to_string_pretty(&json!({
                "version": "output-audit.v1",
                "auditId": audit_id,
                "trigger": "human-via-agent",
                "requestedBy": "human-via-agent",
                "requestedAt": 300,
                "sourceRunId": run_id,
                "sourceIssueId": issue_id,
                "status": "failed",
                "summary": {
                    "checks": 7,
                    "passed": 4,
                    "warnings": 1,
                    "failed": 2,
                    "findings": 1
                },
                "checks": {
                    "runExists": "passed",
                    "changedFilesRecorded": "warning",
                    "allowedWritePathsOnly": "passed",
                    "commandsRecorded": "passed",
                    "highRiskConfirmedIfNeeded": "passed",
                    "evidenceComplete": "failed",
                    "publicDeliveryComplete": "failed"
                },
                "paths": {
                    "report": format!(".agentflow/audit/{audit_id}/audit-report.md")
                }
            }))
            .unwrap(),
        )
        .unwrap();
        fs::write(
            audit_dir.join("findings.json"),
            serde_json::to_string_pretty(&json!({
                "version": "audit-findings.v1",
                "auditId": audit_id,
                "findings": [
                    {
                        "findingId": "finding-001",
                        "severity": "high",
                        "category": "evidence",
                        "title": "验证证据缺失",
                        "detail": "缺少本地验证记录",
                        "evidencePath": format!(".agentflow/tasks/{issue_id}/evidence/evidence.json"),
                        "recommendation": "补齐本地验证证据。"
                    }
                ]
            }))
            .unwrap(),
        )
        .unwrap();
        fs::write(audit_dir.join("audit-report.md"), "# Audit Report\n").unwrap();
        fs::write(
            audit_dir.join("evidence-map.json"),
            serde_json::to_string_pretty(&json!({
                "version": "audit-evidence-map.v1",
                "auditId": audit_id,
                "inputs": {
                    "issue": format!(".agentflow/spec/issues/{issue_id}.json")
                }
            }))
            .unwrap(),
        )
        .unwrap();
        fs::write(
            audit_dir.join("traceability.json"),
            serde_json::to_string_pretty(&json!({
                "version": "audit-traceability.v1",
                "auditId": audit_id,
                "chain": []
            }))
            .unwrap(),
        )
        .unwrap();
        fs::write(audit_dir.join("checklist.md"), "# Checklist\n").unwrap();
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
        assert_eq!(projection.session.status.as_deref(), Some("requested"));
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
        let mut issue = read_spec_issue(dir.path(), "AF-PROJ-001").unwrap();
        issue.status = SpecIssueStatus::Done;
        write_spec_issue(dir.path(), &issue).unwrap();
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
        assert_eq!(project.status, "active");
        assert_eq!(project.next_action, "enter-completion-decision");
        assert_eq!(project.next_action_label, "进入完成判断");
        assert_eq!(
            project.completion.as_ref().map(|completion| completion.current_state.as_str()),
            Some("goal-recheck")
        );
        assert_eq!(project.objective, "用于 projection 测试。");
        assert_eq!(project.project_brain.brain_status, "ready-for-project-loop");
        assert_eq!(
            project.project_brain.project_path,
            "docs/projects/project-projection"
        );
        assert_eq!(
            project.project_brain.health_path,
            "docs/projects/project-projection/PROJECT_HEALTH.md"
        );
        assert_eq!(project.project_brain.health_status, "missing");
        assert_eq!(
            project.project_brain.next_recommended_action_label,
            "进入项目循环"
        );
    }

    #[test]
    fn provider_session_events_do_not_override_authoritative_issue_state() {
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
                    "provider": "codex",
                    "runId": "run-001",
                    "branchName": "agentflow/project-projection/AF-PROJ-001",
                    "launchRequestPath": ".agentflow/tasks/AF-PROJ-001/runs/run-001/launch/agent-request.json"
                }),
            ),
        )
        .unwrap();
        append_task_event_once(
            dir.path(),
            event(
                "AF-PROJ-001",
                "agent.session.completed",
                json!({
                    "provider": "codex",
                    "runId": "run-001",
                    "sessionId": "codex-run-001",
                    "sessionStatus": "done",
                    "logPath": ".agentflow/state/mcp/sessions/codex-run-001.jsonl"
                }),
            ),
        )
        .unwrap();

        rebuild_projections(dir.path()).unwrap();
        let projection = crate::storage::load_task_projection(dir.path(), "AF-PROJ-001").unwrap();

        assert_eq!(projection.current_state, "in_progress");
        assert_eq!(projection.display_status, "in_progress");
        assert_eq!(projection.session.status.as_deref(), Some("done"));
        assert_eq!(projection.runtime.run_status, "validating");
        let in_progress = projection
            .timeline
            .iter()
            .find(|item| item.state == "in_progress")
            .unwrap();
        assert!(in_progress
            .events
            .iter()
            .any(|event| event.event_type == "agent.session.completed"));
    }

    #[test]
    fn rebuilds_audit_summary_into_task_and_project_projection() {
        let dir = tempdir().unwrap();
        write_fixture(dir.path());
        append_task_event_once(
            dir.path(),
            event(
                "AF-PROJ-001",
                "issue.completed",
                json!({
                    "runId": "run-001",
                    "mergeCommit": "abc123"
                }),
            ),
        )
        .unwrap();
        write_audit_fixture(dir.path(), "AF-PROJ-001", "run-001", "audit-001");

        rebuild_projections(dir.path()).unwrap();
        let task = crate::storage::load_task_projection(dir.path(), "AF-PROJ-001").unwrap();
        let project =
            crate::storage::load_project_projection(dir.path(), "project-projection").unwrap();

        assert_eq!(task.audit.status, "failed");
        assert_eq!(task.audit.latest_audit_id.as_deref(), Some("audit-001"));
        assert!(task.audit.summary_line.contains("审计未通过"));
        assert!(task
            .audit
            .evidence_gaps
            .iter()
            .any(|line| line.contains("验证证据不完整")));
        assert!(task
            .audit
            .repair_recommendations
            .iter()
            .any(|line| line.contains("补齐本地验证证据")));
        assert_eq!(
            project
                .audit
                .as_ref()
                .and_then(|audit| audit.latest_audit_id.as_deref()),
            Some("audit-001")
        );
        assert!(project.completion_hint.contains("最近审计"));
    }

    #[test]
    fn stale_failed_run_does_not_override_completed_mainline() {
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
        append_task_event_once(
            dir.path(),
            event(
                "AF-PROJ-001",
                "issue.completed",
                json!({
                    "runId": "run-001",
                    "mergeCommit": "abc123"
                }),
            ),
        )
        .unwrap();
        append_task_event_once(
            dir.path(),
            event(
                "AF-PROJ-001",
                "issue.validation.failed",
                json!({
                    "runId": "run-000",
                    "summary": "old failed run"
                }),
            ),
        )
        .unwrap();

        rebuild_projections(dir.path()).unwrap();
        let projection = crate::storage::load_task_projection(dir.path(), "AF-PROJ-001").unwrap();

        assert_eq!(projection.current_state, "done");
        assert_eq!(projection.latest_run_id.as_deref(), Some("run-001"));
        assert_eq!(projection.runtime.run_id.as_deref(), Some("run-001"));
    }

    #[test]
    fn rebuilds_requirement_preview_projection_before_spec_materialization() {
        let dir = tempdir().unwrap();
        let requirement = dir.path().join("docs/requirements/040-preview.md");
        std::fs::create_dir_all(requirement.parent().unwrap()).unwrap();
        std::fs::write(&requirement, "# 预览\n\n先做 Goal / Plan Preview。\n").unwrap();

        requirement_preview_from_requirement(dir.path(), &requirement, Some("project-preview"))
            .unwrap();
        rebuild_projections(dir.path()).unwrap();

        let projection =
            crate::storage::load_requirement_preview_projection(dir.path(), "040-preview")
                .unwrap();
        let index = crate::storage::load_requirement_preview_index(dir.path()).unwrap();

        assert_eq!(projection.current_state, "goal_draft");
        assert_eq!(projection.lifecycle, "active");
        assert_eq!(projection.next_recommended_action, "confirm-goal-draft-preview");
        assert_eq!(projection.issue_contract_draft_count, 0);
        assert_eq!(index.previews.len(), 1);
        assert_eq!(index.previews[0].project_id, "project-preview");
    }

    #[test]
    fn accepted_completion_decision_marks_project_done() {
        let dir = tempdir().unwrap();
        write_fixture(dir.path());
        let mut issue = read_spec_issue(dir.path(), "AF-PROJ-001").unwrap();
        issue.status = SpecIssueStatus::Done;
        write_spec_issue(dir.path(), &issue).unwrap();
        agentflow_spec::sync_completion_decision_runtimes(dir.path()).unwrap();
        agentflow_spec::record_completion_decision(
            dir.path(),
            "project-projection",
            CompletionDecisionOutcome::Accept,
            "goal-agent",
            "当前项目已经完成。",
            vec!["所有任务与交付都满足当前项目目标。".to_string()],
        )
        .unwrap();

        rebuild_projections(dir.path()).unwrap();
        let project =
            crate::storage::load_project_projection(dir.path(), "project-projection").unwrap();
        assert_eq!(project.status, "done");
        assert_eq!(
            project
                .completion
                .as_ref()
                .and_then(|completion| completion.latest_outcome.as_deref()),
            Some("accept")
        );
    }
}
