use std::collections::BTreeSet;
use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use agentflow_audit::{load_audit_report, load_audit_result_summary};
use agentflow_event_store::{
    map_task_event_to_runtime_event, replay_runtime_events, replay_task_events, ReplayFilter,
    TaskEvent,
};
use agentflow_spec::{
    read_requirement_preview_runtime, read_spec_issue, read_spec_project, SpecIssue, SpecPriority,
    SpecRequiredAgentRole,
};
use agentflow_task_artifacts::load_task_evidence;

use crate::model::{
    ProjectIssueLanes, ProjectionDeliverySummary, ProjectionPublicDelivery, TaskProjection,
    TaskTimelineItem,
};
use crate::storage::{
    load_project_projection, load_requirement_preview_projection, load_task_projection,
};

pub const PROJECTION_QUERY_SURFACE_VERSION: &str = "projection-query-surface.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectionDefinitionVersions {
    pub ontology_version: String,
    pub action_contract_version: String,
    pub role_policy_version: String,
    pub state_machine_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectionFreshness {
    pub projection_version: String,
    pub query_surface_version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_event_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_event_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_event_timestamp: Option<u64>,
    pub last_rebuilt_at: u64,
    pub staleness: String,
    pub definition_versions: ProjectionDefinitionVersions,
    #[serde(default)]
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ViewActionHint {
    pub key: String,
    pub label: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeEventRow {
    pub event_id: String,
    pub event_type: String,
    pub timestamp: u64,
    pub actor_role: String,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssuePreviewItem {
    pub issue_id: String,
    pub title: String,
    pub summary: String,
    pub priority: String,
    pub required_agent_role: String,
    #[serde(default)]
    pub blocked_by: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDependencyEdge {
    pub issue_id: String,
    pub depends_on_issue_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectRunSummary {
    pub issue_id: String,
    pub run_id: String,
    pub run_status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequirementIntakeView {
    pub requirement_id: String,
    pub state: String,
    pub classification: String,
    #[serde(default)]
    pub ambiguities: Vec<String>,
    #[serde(default)]
    pub boundary_notes: Vec<String>,
    #[serde(default)]
    pub allowed_actions: Vec<ViewActionHint>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_event_id: Option<String>,
    pub freshness: ProjectionFreshness,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpecPreviewView {
    pub spec_id: String,
    pub state: String,
    pub requirement_ref: String,
    pub preview_summary: String,
    #[serde(default)]
    pub acceptance_criteria: Vec<String>,
    #[serde(default)]
    pub issue_preview: Vec<IssuePreviewItem>,
    pub confirmation_state: String,
    #[serde(default)]
    pub allowed_actions: Vec<ViewActionHint>,
    pub freshness: ProjectionFreshness,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectHomeView {
    pub project_id: String,
    pub title: String,
    pub objective: String,
    pub state_summary: String,
    pub issue_groups: ProjectIssueLanes,
    #[serde(default)]
    pub dependency_graph: Vec<ProjectDependencyEdge>,
    #[serde(default)]
    pub active_runs: Vec<ProjectRunSummary>,
    #[serde(default)]
    pub blocked_items: Vec<String>,
    #[serde(default)]
    pub recent_events: Vec<RuntimeEventRow>,
    #[serde(default)]
    pub allowed_actions: Vec<ViewActionHint>,
    pub freshness: ProjectionFreshness,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskWorkbenchView {
    pub issue_id: String,
    pub title: String,
    pub summary: String,
    pub issue_state: String,
    pub run_state: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_run: Option<String>,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
    #[serde(default)]
    pub artifact_refs: Vec<String>,
    #[serde(default)]
    pub acceptance_mapping: Vec<String>,
    #[serde(default)]
    pub allowed_actions: Vec<ViewActionHint>,
    #[serde(default)]
    pub blocked_reasons: Vec<String>,
    #[serde(default)]
    pub timeline: Vec<TaskTimelineItem>,
    pub freshness: ProjectionFreshness,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditSurfaceView {
    pub audit_id: String,
    pub audit_state: String,
    pub scope: String,
    #[serde(default)]
    pub evidence_map: Vec<String>,
    #[serde(default)]
    pub findings: Vec<String>,
    #[serde(default)]
    pub traceability: Vec<String>,
    #[serde(default)]
    pub allowed_actions: Vec<ViewActionHint>,
    pub freshness: ProjectionFreshness,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeliveryPackageView {
    pub issue_id: String,
    pub delivery_state: String,
    #[serde(default)]
    pub artifact_refs: Vec<String>,
    #[serde(default)]
    pub verification_logs: Vec<String>,
    #[serde(default)]
    pub acceptance_mapping: Vec<String>,
    pub build_agent_summary: String,
    #[serde(default)]
    pub allowed_actions: Vec<ViewActionHint>,
    pub freshness: ProjectionFreshness,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeHealthView {
    pub project_id: String,
    pub project_status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_issue_id: Option<String>,
    pub active_issue_count: usize,
    pub blocked_issue_count: usize,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub allowed_actions: Vec<ViewActionHint>,
    pub freshness: ProjectionFreshness,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ProjectionScope {
    RequirementPreview { project_id: String },
    Project { project_id: String },
    Issue { issue_id: String },
    Audit { source_issue_id: Option<String> },
}

pub fn get_requirement_intake_view(
    project_root: impl AsRef<Path>,
    requirement_id: &str,
) -> Result<RequirementIntakeView> {
    let runtime = read_requirement_preview_runtime(&project_root, requirement_id)?;
    let projection = load_requirement_preview_projection(&project_root, requirement_id)?;
    let freshness = explain_projection_staleness(
        &project_root,
        ProjectionScope::RequirementPreview {
            project_id: runtime.project_id.clone(),
        },
        &projection.version,
        projection.updated_at,
        None,
    )?;
    let boundary_notes = runtime
        .goal_draft
        .non_goals
        .iter()
        .chain(runtime.goal_draft.constraints.iter())
        .cloned()
        .collect::<Vec<_>>();
    let ambiguities = runtime
        .intake
        .missing_information
        .iter()
        .chain(runtime.intake.clarification_questions.iter())
        .cloned()
        .collect::<Vec<_>>();

    Ok(RequirementIntakeView {
        requirement_id: runtime.requirement_id,
        state: projection.current_state,
        classification: runtime.intake.detected_intent.as_str().to_string(),
        ambiguities,
        boundary_notes,
        allowed_actions: next_action_hints(
            &projection.next_recommended_action,
            &projection.next_recommended_action_label,
            &projection.next_recommended_action_reason,
        ),
        last_event_id: freshness.last_event_id.clone(),
        freshness,
    })
}

pub fn get_spec_preview_view(
    project_root: impl AsRef<Path>,
    requirement_id: &str,
) -> Result<SpecPreviewView> {
    let runtime = read_requirement_preview_runtime(&project_root, requirement_id)?;
    let projection = load_requirement_preview_projection(&project_root, requirement_id)?;
    let freshness = explain_projection_staleness(
        &project_root,
        ProjectionScope::RequirementPreview {
            project_id: runtime.project_id.clone(),
        },
        &projection.version,
        projection.updated_at,
        None,
    )?;

    let mut acceptance_criteria = runtime.goal_draft.success_criteria.clone();
    let mut issue_preview = Vec::new();
    if let Some(plan_draft) = runtime.plan_draft.as_ref() {
        for draft in &plan_draft.issue_contract_drafts {
            acceptance_criteria.extend(draft.acceptance_criteria.clone());
            issue_preview.push(IssuePreviewItem {
                issue_id: draft.issue_draft_id.clone(),
                title: draft.title.clone(),
                summary: draft.goal.clone(),
                priority: priority_label(&draft.priority),
                required_agent_role: required_role_label(&draft.suggested_agent_role),
                blocked_by: draft.dependencies.clone(),
            });
        }
    } else {
        for issue_id in &runtime.materialized_issue_ids {
            let issue = read_spec_issue(&project_root, issue_id)?;
            acceptance_criteria.extend(issue.validation_commands.clone());
            issue_preview.push(spec_issue_preview_item(&issue));
        }
    }
    acceptance_criteria.sort();
    acceptance_criteria.dedup();

    let confirmation_state = runtime
        .confirmation_records
        .last()
        .map(|record| record.decision.clone())
        .unwrap_or_else(|| runtime.current_state.clone());

    Ok(SpecPreviewView {
        spec_id: runtime.project_id.clone(),
        state: projection.current_state,
        requirement_ref: runtime.requirement_id,
        preview_summary: runtime.goal_draft.outcome,
        acceptance_criteria,
        issue_preview,
        confirmation_state,
        allowed_actions: next_action_hints(
            &projection.next_recommended_action,
            &projection.next_recommended_action_label,
            &projection.next_recommended_action_reason,
        ),
        freshness,
    })
}

pub fn get_project_home_view(
    project_root: impl AsRef<Path>,
    project_id: &str,
) -> Result<ProjectHomeView> {
    let spec_project = read_spec_project(&project_root, project_id)?;
    let projection = load_project_projection(&project_root, project_id)?;
    let freshness = explain_projection_staleness(
        &project_root,
        ProjectionScope::Project {
            project_id: project_id.to_string(),
        },
        &projection.version,
        projection.updated_at,
        None,
    )?;

    let mut dependency_graph = Vec::new();
    let mut active_runs = Vec::new();
    for issue_id in &spec_project.issue_ids {
        let issue = read_spec_issue(&project_root, issue_id)?;
        for dependency in &issue.blocked_by {
            dependency_graph.push(ProjectDependencyEdge {
                issue_id: issue.issue_id.clone(),
                depends_on_issue_id: dependency.clone(),
            });
        }
        let task = load_task_projection(&project_root, issue_id).ok();
        if let Some(task) = task {
            if let Some(run_id) = task.latest_run_id.clone() {
                active_runs.push(ProjectRunSummary {
                    issue_id: issue.issue_id.clone(),
                    run_id,
                    run_status: task.runtime.run_status.clone(),
                    branch_name: task.branch_name.clone(),
                });
            }
        }
    }

    Ok(ProjectHomeView {
        project_id: projection.project_id.clone(),
        title: projection.title.clone(),
        objective: projection.objective.clone(),
        state_summary: format!("{} / {}", projection.status, projection.stage_label),
        issue_groups: projection.lanes.clone(),
        dependency_graph,
        active_runs,
        blocked_items: projection
            .blockers
            .iter()
            .map(|blocker| format!("{}: {}", blocker.issue_id, blocker.reason))
            .collect(),
        recent_events: recent_events(
            &project_root,
            ReplayFilter {
                project_id: Some(project_id.to_string()),
                ..ReplayFilter::default()
            },
            8,
        )?,
        allowed_actions: next_action_hints(
            &projection.next_action,
            &projection.next_action_label,
            &projection.next_action_reason,
        ),
        freshness,
    })
}

pub fn get_task_workbench_view(
    project_root: impl AsRef<Path>,
    issue_id: &str,
) -> Result<TaskWorkbenchView> {
    let issue = read_spec_issue(&project_root, issue_id)?;
    let projection = load_task_projection(&project_root, issue_id)?;
    let projection_cursor = projection
        .timeline
        .iter()
        .flat_map(|item| item.events.iter().map(|event| event.event_id.clone()))
        .last();
    let freshness = explain_projection_staleness(
        &project_root,
        ProjectionScope::Issue {
            issue_id: issue_id.to_string(),
        },
        &projection.version,
        projection.updated_at,
        projection_cursor,
    )?;

    let runtime_events = replay_runtime_events(&project_root, ReplayFilter::issue(issue_id))?;
    let mut evidence_refs = BTreeSet::new();
    let mut artifact_refs = BTreeSet::new();
    for event in runtime_events {
        for evidence in event.envelope.evidence_refs {
            evidence_refs.insert(evidence);
        }
        for artifact in event.envelope.artifact_refs {
            artifact_refs.insert(artifact);
        }
    }
    if let Ok(evidence) = load_task_evidence(&project_root, issue_id) {
        evidence_refs.insert(evidence.run_path);
        evidence_refs.insert(evidence.validation_path);
        if let Some(changed_files_path) = evidence.changed_files_path {
            evidence_refs.insert(changed_files_path);
        }
        for path in evidence.command_paths {
            evidence_refs.insert(path);
        }
    }

    Ok(TaskWorkbenchView {
        issue_id: issue.issue_id.clone(),
        title: issue.title.clone(),
        summary: issue.summary.clone(),
        issue_state: projection.current_state.clone(),
        run_state: projection.runtime.run_status.clone(),
        active_run: projection.latest_run_id.clone(),
        evidence_refs: evidence_refs.into_iter().collect(),
        artifact_refs: artifact_refs.into_iter().collect(),
        acceptance_mapping: issue_acceptance_mapping(&issue, &projection.delivery),
        allowed_actions: task_allowed_actions(&projection),
        blocked_reasons: task_blocked_reasons(&issue, &projection),
        timeline: projection.timeline.clone(),
        freshness,
    })
}

pub fn get_audit_surface_view(
    project_root: impl AsRef<Path>,
    audit_id: &str,
) -> Result<AuditSurfaceView> {
    let report = load_audit_report(&project_root, audit_id.to_string())?;
    let summary = load_audit_result_summary(&project_root, audit_id.to_string())?;
    let allowed_actions = audit_allowed_actions(&summary);
    let freshness = explain_projection_staleness(
        &project_root,
        ProjectionScope::Audit {
            source_issue_id: report.audit.source_issue_id.clone(),
        },
        &summary.version,
        summary.requested_at,
        None,
    )?;

    Ok(AuditSurfaceView {
        audit_id: summary.audit_id,
        audit_state: summary.status,
        scope: format!(
            "issue={} / run={}",
            report
                .audit
                .source_issue_id
                .unwrap_or_else(|| "none".to_string()),
            report
                .audit
                .source_run_id
                .unwrap_or_else(|| "none".to_string())
        ),
        evidence_map: report.evidence_map.inputs.values().cloned().collect(),
        findings: report
            .findings
            .findings
            .iter()
            .map(|finding| format!("{}: {}", finding.severity.as_str(), finding.title))
            .collect(),
        traceability: report
            .traceability
            .chain
            .iter()
            .map(|item| format!("{}:{} -> {}", item.layer, item.id, item.path))
            .collect(),
        allowed_actions,
        freshness,
    })
}

pub fn get_delivery_package_view(
    project_root: impl AsRef<Path>,
    issue_id: &str,
) -> Result<DeliveryPackageView> {
    let issue = read_spec_issue(&project_root, issue_id)?;
    let projection = load_task_projection(&project_root, issue_id)?;
    let projection_cursor = projection
        .timeline
        .iter()
        .flat_map(|item| item.events.iter().map(|event| event.event_id.clone()))
        .last();
    let freshness = explain_projection_staleness(
        &project_root,
        ProjectionScope::Issue {
            issue_id: issue_id.to_string(),
        },
        &projection.version,
        projection.updated_at,
        projection_cursor,
    )?;
    let runtime_events = replay_runtime_events(&project_root, ReplayFilter::issue(issue_id))?;
    let mut artifact_refs = BTreeSet::new();
    for event in runtime_events {
        for artifact in event.envelope.artifact_refs {
            artifact_refs.insert(artifact);
        }
    }
    if let Some(pr_url) = projection.public_delivery.pr_url.clone() {
        artifact_refs.insert(pr_url);
    }
    if let Some(changelog_path) = projection.public_delivery.changelog_path.clone() {
        artifact_refs.insert(changelog_path);
    }
    if let Some(release_notes_url) = projection.public_delivery.release_notes_url.clone() {
        artifact_refs.insert(release_notes_url);
    }

    let mut verification_logs = Vec::new();
    if let Ok(evidence) = load_task_evidence(&project_root, issue_id) {
        verification_logs.extend(evidence.command_paths.clone());
        verification_logs.push(evidence.validation_path.clone());
        verification_logs.push(evidence.run_path.clone());
    }

    Ok(DeliveryPackageView {
        issue_id: issue.issue_id.clone(),
        delivery_state: projection.delivery.status.clone(),
        artifact_refs: artifact_refs.into_iter().collect(),
        verification_logs,
        acceptance_mapping: issue_acceptance_mapping(&issue, &projection.delivery),
        build_agent_summary: delivery_summary_line(
            &projection.delivery,
            &projection.public_delivery,
        ),
        allowed_actions: delivery_allowed_actions(&projection),
        freshness,
    })
}

pub fn get_runtime_health_view(
    project_root: impl AsRef<Path>,
    project_id: &str,
) -> Result<RuntimeHealthView> {
    let projection = load_project_projection(&project_root, project_id)?;
    let freshness = explain_projection_staleness(
        &project_root,
        ProjectionScope::Project {
            project_id: project_id.to_string(),
        },
        &projection.version,
        projection.updated_at,
        None,
    )?;

    let mut warnings = Vec::new();
    if !projection.blockers.is_empty() {
        warnings.push("project-blockers-present".to_string());
    }
    if projection
        .audit
        .as_ref()
        .is_some_and(|audit| audit.status == "failed")
    {
        warnings.push("audit-failed".to_string());
    }

    Ok(RuntimeHealthView {
        project_id: projection.project_id.clone(),
        project_status: projection.status.clone(),
        current_issue_id: projection.current_issue_id.clone(),
        active_issue_count: projection.lanes.current.len(),
        blocked_issue_count: projection.lanes.blocked.len(),
        warnings,
        allowed_actions: next_action_hints(
            &projection.next_action,
            &projection.next_action_label,
            &projection.next_action_reason,
        ),
        freshness,
    })
}

fn explain_projection_staleness(
    project_root: impl AsRef<Path>,
    scope: ProjectionScope,
    projection_version: &str,
    last_rebuilt_at: u64,
    projection_cursor: Option<String>,
) -> Result<ProjectionFreshness> {
    let latest = latest_event_summary(project_root, &scope)?;
    let mut warnings = latest.warnings;
    if latest.last_event_id.is_none() {
        warnings.push("no-runtime-event-yet".to_string());
    }

    let staleness = if latest.last_event_id.is_none() {
        "empty".to_string()
    } else if projection_cursor
        .as_ref()
        .zip(latest.last_event_id.as_ref())
        .is_some_and(|(projection_event_id, latest_event_id)| {
            projection_event_id != latest_event_id
        })
    {
        "stale".to_string()
    } else if latest
        .last_event_timestamp
        .is_some_and(|timestamp| timestamp > last_rebuilt_at)
    {
        "stale".to_string()
    } else {
        "current".to_string()
    };

    Ok(ProjectionFreshness {
        projection_version: projection_version.to_string(),
        query_surface_version: PROJECTION_QUERY_SURFACE_VERSION.to_string(),
        last_event_id: latest.last_event_id,
        last_event_type: latest.last_event_type,
        last_event_timestamp: latest.last_event_timestamp,
        last_rebuilt_at,
        staleness,
        definition_versions: latest.definition_versions,
        warnings,
    })
}

#[derive(Debug, Clone)]
struct LatestEventSummary {
    last_event_id: Option<String>,
    last_event_type: Option<String>,
    last_event_timestamp: Option<u64>,
    definition_versions: ProjectionDefinitionVersions,
    warnings: Vec<String>,
}

fn latest_event_summary(
    project_root: impl AsRef<Path>,
    scope: &ProjectionScope,
) -> Result<LatestEventSummary> {
    let filter = match scope {
        ProjectionScope::RequirementPreview { project_id }
        | ProjectionScope::Project { project_id } => ReplayFilter {
            project_id: Some(project_id.clone()),
            ..ReplayFilter::default()
        },
        ProjectionScope::Issue { issue_id } => ReplayFilter::issue(issue_id.clone()),
        ProjectionScope::Audit { source_issue_id } => source_issue_id
            .as_ref()
            .map(|issue_id| ReplayFilter::issue(issue_id.clone()))
            .unwrap_or_default(),
    };
    let events = replay_task_events(project_root, filter)?;
    let Some(last_event) = events.last() else {
        return Ok(LatestEventSummary {
            last_event_id: None,
            last_event_type: None,
            last_event_timestamp: None,
            definition_versions: ProjectionDefinitionVersions {
                ontology_version: "unavailable".to_string(),
                action_contract_version: "unavailable".to_string(),
                role_policy_version: "unavailable".to_string(),
                state_machine_version: "unavailable".to_string(),
            },
            warnings: Vec::new(),
        });
    };
    let compatibility = map_task_event_to_runtime_event(last_event)?;
    Ok(LatestEventSummary {
        last_event_id: Some(last_event.event_id.clone()),
        last_event_type: Some(last_event.event_type.clone()),
        last_event_timestamp: Some(last_event.timestamp),
        definition_versions: ProjectionDefinitionVersions {
            ontology_version: compatibility.envelope.ontology_version,
            action_contract_version: compatibility.envelope.action_contract_version,
            role_policy_version: compatibility.envelope.role_policy_version,
            state_machine_version: compatibility.envelope.state_machine_version,
        },
        warnings: compatibility.warnings,
    })
}

fn recent_events(
    project_root: impl AsRef<Path>,
    filter: ReplayFilter,
    limit: usize,
) -> Result<Vec<RuntimeEventRow>> {
    let events = replay_task_events(project_root, filter)?;
    Ok(events
        .into_iter()
        .rev()
        .take(limit)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .map(runtime_event_row)
        .collect())
}

fn runtime_event_row(event: TaskEvent) -> RuntimeEventRow {
    RuntimeEventRow {
        event_id: event.event_id,
        event_type: event.event_type.clone(),
        timestamp: event.timestamp,
        actor_role: event.actor.role,
        summary: event.event_type,
    }
}

fn next_action_hints(key: &str, label: &str, reason: &str) -> Vec<ViewActionHint> {
    if key.is_empty() {
        return Vec::new();
    }
    vec![ViewActionHint {
        key: key.to_string(),
        label: label.to_string(),
        reason: reason.to_string(),
    }]
}

fn task_allowed_actions(projection: &TaskProjection) -> Vec<ViewActionHint> {
    match projection.display_status.as_str() {
        "backlog" => vec![hint(
            "schedule",
            "等待调度",
            "当前任务还在 backlog，等待 Task Loop 排入执行。",
        )],
        "todo" => vec![hint(
            "launch",
            "准备启动",
            "当前任务已经满足开工前置条件，下一步是拉起执行会话。",
        )],
        "in_progress" => vec![hint(
            "observe-runtime",
            "查看运行态",
            "当前任务正在执行，优先观察实时事件和验证输出。",
        )],
        "in_review" => vec![hint(
            "review-closeout",
            "检查交付",
            "当前任务已进入 review，下一步核对 PR、验证证据和合并证明。",
        )],
        "done" => vec![hint(
            "view-delivery",
            "查看交付",
            "当前任务已经完成，优先查看公开交付和验证证据。",
        )],
        "blocked" => vec![hint(
            "inspect-blocker",
            "查看阻断",
            "当前任务存在阻断，先处理依赖、证据或工作区问题。",
        )],
        "cancel" => vec![hint(
            "inspect-cancel",
            "查看取消原因",
            "当前任务已取消，只保留历史事实和关闭原因。",
        )],
        _ => Vec::new(),
    }
}

fn delivery_allowed_actions(projection: &TaskProjection) -> Vec<ViewActionHint> {
    match projection.delivery.status.as_str() {
        "ready" | "published" => vec![hint(
            "view-public-delivery",
            "查看公开交付",
            "公开交付记录已经可读，优先核对 PR、CHANGELOG 或 release notes。",
        )],
        "drafted" => vec![hint(
            "review-public-delivery",
            "检查交付草稿",
            "公开交付草稿已生成，但还没有进入最终公开状态。",
        )],
        _ => vec![hint(
            "wait-closeout",
            "等待收口",
            "交付记录还没有就绪，先完成验证、合并证明和 Done 写回。",
        )],
    }
}

fn audit_allowed_actions(summary: &agentflow_audit::AuditResultSummary) -> Vec<ViewActionHint> {
    match summary.status.as_str() {
        "requested" | "in_progress" => vec![hint(
            "follow-audit",
            "跟踪审计",
            "审计正在进行，先查看检查点、证据映射和 findings。",
        )],
        "passed" => vec![hint(
            "accept-audit",
            "查看通过结论",
            "审计已经通过，优先核对最终结论与 traceability。",
        )],
        "failed" => vec![hint(
            "repair-from-finding",
            "处理 findings",
            "审计失败，下一步根据 findings 创建修复任务。",
        )],
        _ => vec![hint(
            "view-audit",
            "查看审计记录",
            "当前审计记录已存在，可以直接查看事实和结论。",
        )],
    }
}

fn task_blocked_reasons(issue: &SpecIssue, projection: &TaskProjection) -> Vec<String> {
    if projection.display_status != "blocked" {
        return Vec::new();
    }
    if !issue.blocked_by.is_empty() {
        return issue
            .blocked_by
            .iter()
            .map(|dependency| format!("依赖未完成: {dependency}"))
            .collect();
    }
    projection
        .timeline
        .iter()
        .filter(|item| item.phase.as_str() == "exception")
        .map(|item| item.summary.clone())
        .collect()
}

fn issue_acceptance_mapping(
    issue: &SpecIssue,
    delivery: &ProjectionDeliverySummary,
) -> Vec<String> {
    let mut mapping = issue
        .validation_commands
        .iter()
        .map(|command| format!("验证命令: {command}"))
        .collect::<Vec<_>>();
    mapping.push(format!(
        "证据输出: {}",
        issue.expected_outputs.evidence_path
    ));
    mapping.push(format!(
        "任务运行目录: {}",
        issue.expected_outputs.task_run_dir
    ));
    mapping.push(format!(
        "公开交付: {}",
        issue
            .expected_outputs
            .public_delivery_record
            .changelog_or_release_notes
    ));
    if !delivery.summary_line.is_empty() {
        mapping.push(format!("当前交付总结: {}", delivery.summary_line));
    }
    mapping
}

fn delivery_summary_line(
    delivery: &ProjectionDeliverySummary,
    public_delivery: &ProjectionPublicDelivery,
) -> String {
    if !delivery.summary_line.is_empty() {
        return delivery.summary_line.clone();
    }
    if let Some(pr_url) = public_delivery.pr_url.as_ref() {
        return format!("PR/MR: {pr_url}");
    }
    "公开交付待生成".to_string()
}

fn spec_issue_preview_item(issue: &SpecIssue) -> IssuePreviewItem {
    IssuePreviewItem {
        issue_id: issue.issue_id.clone(),
        title: issue.title.clone(),
        summary: issue.summary.clone(),
        priority: priority_label(&issue.priority),
        required_agent_role: required_role_label(&issue.required_agent_role),
        blocked_by: issue.blocked_by.clone(),
    }
}

fn priority_label(priority: &SpecPriority) -> String {
    match priority {
        SpecPriority::P0 => "P0".to_string(),
        SpecPriority::P1 => "P1".to_string(),
        SpecPriority::P2 => "P2".to_string(),
        SpecPriority::P3 => "P3".to_string(),
    }
}

fn required_role_label(role: &SpecRequiredAgentRole) -> String {
    role.provider_role_alias().to_string()
}

fn hint(key: &str, label: &str, reason: &str) -> ViewActionHint {
    ViewActionHint {
        key: key.to_string(),
        label: label.to_string(),
        reason: reason.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs;

    use agentflow_event_store::{append_task_event_once, EventActor, TaskEventDraft};
    use agentflow_spec::{
        confirm_goal_draft_preview, confirm_plan_draft_preview, issue_from_requirement,
        project_from_requirement, requirement_preview_from_requirement, write_spec_issue,
        write_spec_project, SpecIssueDraft, SpecIssueStatus, SpecProjectDraft,
    };
    use agentflow_task_artifacts::{create_task_run, update_task_run_status, TaskRunStatus};
    use agentflow_workflow_core::{WorkflowAgentRole, WorkflowFlowType};
    use serde_json::json;
    use tempfile::tempdir;

    use crate::projector::rebuild_projections;

    fn write_fixture(root: &Path) {
        let requirement = root.join("docs/requirements/034-test.md");
        fs::create_dir_all(requirement.parent().unwrap()).unwrap();
        fs::write(&requirement, "# 测试需求\n\n用于 projection query 测试。\n").unwrap();
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
        issue.validation_commands = vec!["cargo test -p agentflow-projection".to_string()];
        let issue = issue_from_requirement(root, &requirement, issue).unwrap();
        write_spec_issue(root, &issue).unwrap();

        let mut project = SpecProjectDraft::new("project-projection");
        project.issue_ids = vec!["AF-PROJ-001".to_string()];
        let project = project_from_requirement(root, &requirement, project).unwrap();
        write_spec_project(root, &project).unwrap();
    }

    fn write_completion_ready_artifacts(root: &Path, issue_id: &str, run_id: &str) {
        let task_root = root.join(".agentflow/tasks").join(issue_id);
        let evidence_dir = task_root.join("evidence");
        fs::create_dir_all(&evidence_dir).unwrap();
        fs::write(
            evidence_dir.join("evidence.json"),
            serde_json::to_string_pretty(&json!({
                "version": "task-evidence.v1",
                "issueId": issue_id,
                "runId": run_id,
                "status": "ready",
                "summary": "本地验证通过。",
                "runPath": format!(".agentflow/tasks/{issue_id}/runs/{run_id}/run.json"),
                "commandPaths": [format!(".agentflow/tasks/{issue_id}/runs/{run_id}/verify/local.log")],
                "validationPath": format!(".agentflow/tasks/{issue_id}/runs/{run_id}/validation.json"),
                "createdAt": 1
            }))
            .unwrap(),
        )
        .unwrap();
    }

    fn event(issue_id: &str, event_type: &str, payload: serde_json::Value) -> TaskEventDraft {
        TaskEventDraft {
            flow_type: WorkflowFlowType::Work,
            aggregate_type: "issue".to_string(),
            aggregate_id: issue_id.to_string(),
            project_id: Some("project-projection".to_string()),
            issue_id: Some(issue_id.to_string()),
            run_id: payload
                .get("runId")
                .and_then(serde_json::Value::as_str)
                .map(str::to_string),
            event_type: event_type.to_string(),
            authority_role: Some(WorkflowAgentRole::WorkAgent),
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
    fn task_workbench_view_separates_issue_and_run_state() {
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
                json!({"runId":"run-001","branchName":"agentflow/project-projection/AF-PROJ-001"}),
            ),
        )
        .unwrap();
        create_task_run(
            dir.path(),
            "AF-PROJ-001",
            "run-001",
            "work-agent.issue-loop@v1",
            Some("agentflow/project-projection/AF-PROJ-001".to_string()),
        )
        .unwrap();
        update_task_run_status(
            dir.path(),
            "AF-PROJ-001",
            "run-001",
            TaskRunStatus::Validating,
        )
        .unwrap();

        rebuild_projections(dir.path()).unwrap();
        let view = get_task_workbench_view(dir.path(), "AF-PROJ-001").unwrap();

        assert_eq!(view.issue_state, "in_progress");
        assert_eq!(view.run_state, "queued");
        assert_eq!(view.active_run.as_deref(), Some("run-001"));
        assert_ne!(view.issue_state, view.run_state);
        assert_eq!(view.freshness.staleness, "current");
    }

    #[test]
    fn delivery_view_shows_done_issue_without_audit_side_effect() {
        let dir = tempdir().unwrap();
        write_fixture(dir.path());
        let mut issue = read_spec_issue(dir.path(), "AF-PROJ-001").unwrap();
        issue.status = SpecIssueStatus::Done;
        write_spec_issue(dir.path(), &issue).unwrap();
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
                json!({"runId":"run-001"}),
            ),
        )
        .unwrap();
        append_task_event_once(
            dir.path(),
            event(
                "AF-PROJ-001",
                "issue.completed",
                json!({"runId":"run-001","mergeCommit":"abc123"}),
            ),
        )
        .unwrap();
        write_completion_ready_artifacts(dir.path(), "AF-PROJ-001", "run-001");

        rebuild_projections(dir.path()).unwrap();
        let view = get_delivery_package_view(dir.path(), "AF-PROJ-001").unwrap();
        let task = get_task_workbench_view(dir.path(), "AF-PROJ-001").unwrap();

        assert_eq!(view.issue_id, "AF-PROJ-001");
        assert!(!view.verification_logs.is_empty());
        assert_eq!(task.issue_state, "done");
        assert_eq!(task.freshness.staleness, "current");
    }

    #[test]
    fn freshness_turns_stale_when_new_issue_event_arrives_after_rebuild() {
        let dir = tempdir().unwrap();
        write_fixture(dir.path());
        append_task_event_once(
            dir.path(),
            event("AF-PROJ-001", "issue.scheduled", json!({})),
        )
        .unwrap();
        rebuild_projections(dir.path()).unwrap();
        let current = get_task_workbench_view(dir.path(), "AF-PROJ-001").unwrap();
        assert_eq!(current.freshness.staleness, "current");

        append_task_event_once(
            dir.path(),
            event(
                "AF-PROJ-001",
                "agent.launch.requested",
                json!({"runId":"run-002"}),
            ),
        )
        .unwrap();
        let stale = get_task_workbench_view(dir.path(), "AF-PROJ-001").unwrap();
        assert_eq!(stale.freshness.staleness, "stale");
        assert_ne!(
            current.freshness.last_event_id,
            stale.freshness.last_event_id
        );
    }

    #[test]
    fn spec_preview_view_uses_requirement_runtime_and_plan_drafts() {
        let dir = tempdir().unwrap();
        let requirement = dir.path().join("docs/requirements/040-preview.md");
        fs::create_dir_all(requirement.parent().unwrap()).unwrap();
        fs::write(&requirement, "# 预览\n\n先做 Goal / Plan Preview。\n").unwrap();

        requirement_preview_from_requirement(dir.path(), &requirement, Some("project-preview"))
            .unwrap();
        confirm_goal_draft_preview(dir.path(), "040-preview", "goal-agent").unwrap();
        confirm_plan_draft_preview(dir.path(), "040-preview", "spec-agent").unwrap();
        rebuild_projections(dir.path()).unwrap();

        let intake = get_requirement_intake_view(dir.path(), "040-preview").unwrap();
        let preview = get_spec_preview_view(dir.path(), "040-preview").unwrap();

        assert_eq!(intake.requirement_id, "040-preview");
        assert_eq!(preview.spec_id, "project-preview");
        assert!(!preview.issue_preview.is_empty());
        assert!(!preview.acceptance_criteria.is_empty());
    }
}
