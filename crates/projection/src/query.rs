use std::collections::BTreeSet;
use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use agentflow_audit::{load_audit_report, load_audit_result_summary};
use agentflow_event_store::{
    classify_task_event, map_task_event_to_runtime_event, replay_runtime_events,
    replay_task_events, ReplayFilter, TaskEvent,
};
use agentflow_spec::{
    read_requirement_preview_runtime, read_spec_issue, read_spec_project, SpecIssue, SpecPriority,
    SpecRequiredAgentRole,
};
use agentflow_task_artifacts::{
    load_task_evidence, load_task_run, load_task_session_evidence,
    load_task_session_history_record, load_task_session_recovery_summary,
};

use crate::model::{
    ProjectIssueLanes, ProjectionDeliverySummary, ProjectionPublicDelivery, TaskProjection,
    TaskTimelineItem,
};
use crate::storage::{
    load_issue_status_index, load_project_projection, load_requirement_preview_projection,
    load_spec_loop_projection, load_task_projection,
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
pub struct WorkLoopEventView {
    pub event_id: String,
    pub event_type: String,
    pub category: String,
    pub stage_key: String,
    pub stage_label: String,
    pub timestamp: u64,
    pub actor_role: String,
    pub actor_kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_state: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to_state: Option<String>,
    pub summary: String,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
    #[serde(default)]
    pub artifact_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkLoopEvidenceSummaryView {
    pub status: String,
    pub summary: String,
    #[serde(default)]
    pub verification_refs: Vec<String>,
    #[serde(default)]
    pub session_refs: Vec<String>,
    #[serde(default)]
    pub delivery_refs: Vec<String>,
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
pub struct SpecLoopStageView {
    pub stage: String,
    pub path: String,
    pub status: String,
    pub authority: String,
    pub authority_layer: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_state: Option<String>,
    #[serde(default)]
    pub input_refs: Vec<String>,
    #[serde(default)]
    pub output_refs: Vec<String>,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
    pub summary: String,
    pub updated_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpecLoopAuthorityLayerView {
    pub authority_layer: String,
    pub path: String,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpecLoopTraceabilityView {
    pub from_ref: String,
    pub to_ref: String,
    pub relation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpecLoopActionProposalView {
    pub proposal_ref: String,
    pub action_type: String,
    pub target_object_type: String,
    pub target_object_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_object_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_object_id: Option<String>,
    pub actor_role: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub handoff_rule: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command_status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decision_status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accepted_action_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proposal_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decision_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accepted_action_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpecLoopView {
    pub requirement_id: String,
    pub requirement_path: String,
    pub project_id: String,
    pub project_title: String,
    pub lifecycle: String,
    pub current_state: String,
    pub manifest_path: String,
    pub runtime_path: String,
    pub next_recommended_action: String,
    pub next_recommended_action_label: String,
    pub next_recommended_action_reason: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub materialized_project_id: Option<String>,
    #[serde(default)]
    pub materialized_issue_ids: Vec<String>,
    #[serde(default)]
    pub stages: Vec<SpecLoopStageView>,
    #[serde(default)]
    pub authority_layers: Vec<SpecLoopAuthorityLayerView>,
    #[serde(default)]
    pub traceability: Vec<SpecLoopTraceabilityView>,
    #[serde(default)]
    pub runtime_action_proposals: Vec<SpecLoopActionProposalView>,
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
    pub state_explanation: String,
    pub evidence_summary: WorkLoopEvidenceSummaryView,
    #[serde(default)]
    pub event_stream: Vec<WorkLoopEventView>,
    #[serde(default)]
    pub timeline: Vec<TaskTimelineItem>,
    pub freshness: ProjectionFreshness,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkLoopRunView {
    pub issue_id: String,
    pub run_id: String,
    pub issue_state: String,
    pub run_state: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_status: Option<String>,
    pub state_explanation: String,
    pub evidence_summary: WorkLoopEvidenceSummaryView,
    #[serde(default)]
    pub event_stream: Vec<WorkLoopEventView>,
    pub freshness: ProjectionFreshness,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkLoopSessionView {
    pub issue_id: String,
    pub run_id: String,
    pub session_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_owner: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_status: Option<String>,
    #[serde(default)]
    pub attempt_count: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub started_at: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_heartbeat_at: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recovery_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resumed_from_attempt: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retry_policy: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retryable: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub terminal_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
    pub state_explanation: String,
    pub evidence_summary: WorkLoopEvidenceSummaryView,
    #[serde(default)]
    pub event_stream: Vec<WorkLoopEventView>,
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

pub fn get_spec_loop_view(
    project_root: impl AsRef<Path>,
    requirement_id: &str,
) -> Result<SpecLoopView> {
    let runtime = read_requirement_preview_runtime(&project_root, requirement_id)?;
    let projection = load_spec_loop_projection(&project_root, requirement_id)?;
    let freshness = explain_projection_staleness(
        &project_root,
        ProjectionScope::RequirementPreview {
            project_id: runtime.project_id.clone(),
        },
        &projection.version,
        projection.updated_at,
        None,
    )?;

    Ok(SpecLoopView {
        requirement_id: projection.requirement_id,
        requirement_path: projection.requirement_path,
        project_id: projection.project_id,
        project_title: projection.project_title,
        lifecycle: projection.lifecycle,
        current_state: projection.current_state,
        manifest_path: projection.manifest_path,
        runtime_path: projection.runtime_path,
        next_recommended_action: projection.next_recommended_action.clone(),
        next_recommended_action_label: projection.next_recommended_action_label.clone(),
        next_recommended_action_reason: projection.next_recommended_action_reason.clone(),
        materialized_project_id: projection.materialized_project_id,
        materialized_issue_ids: projection.materialized_issue_ids,
        stages: projection
            .stages
            .into_iter()
            .map(|stage| SpecLoopStageView {
                stage: stage.stage,
                path: stage.path,
                status: stage.status,
                authority: stage.authority,
                authority_layer: stage.authority_layer,
                current_state: stage.current_state,
                input_refs: stage.input_refs,
                output_refs: stage.output_refs,
                evidence_refs: stage.evidence_refs,
                summary: stage.summary,
                updated_at: stage.updated_at,
            })
            .collect(),
        authority_layers: projection
            .authority_layers
            .into_iter()
            .map(|entry| SpecLoopAuthorityLayerView {
                authority_layer: entry.authority_layer,
                path: entry.path,
                summary: entry.summary,
            })
            .collect(),
        traceability: projection
            .traceability
            .into_iter()
            .map(|edge| SpecLoopTraceabilityView {
                from_ref: edge.from_ref,
                to_ref: edge.to_ref,
                relation: edge.relation,
            })
            .collect(),
        runtime_action_proposals: projection
            .runtime_action_proposals
            .into_iter()
            .map(|proposal| SpecLoopActionProposalView {
                proposal_ref: proposal.proposal_ref,
                action_type: proposal.action_type,
                target_object_type: proposal.target_object_type,
                target_object_id: proposal.target_object_id,
                created_object_type: proposal.created_object_type,
                created_object_id: proposal.created_object_id,
                actor_role: proposal.actor_role,
                handoff_rule: proposal.handoff_rule,
                command_status: proposal.command_status,
                decision_status: proposal.decision_status,
                accepted_action_id: proposal.accepted_action_id,
                command_path: proposal.command_path,
                proposal_path: proposal.proposal_path,
                decision_path: proposal.decision_path,
                accepted_action_path: proposal.accepted_action_path,
            })
            .collect(),
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
    let event_stream = collect_work_loop_events(
        &project_root,
        ReplayFilter::issue(issue_id),
        EventStreamScope::Issue,
    )?;
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
    let evidence_summary = build_work_loop_evidence_summary(
        &project_root,
        issue_id,
        projection.latest_run_id.as_deref(),
        projection.session.session_id.as_deref(),
        &projection.public_delivery,
    );
    let state_explanation =
        explain_issue_state(&projection.current_state, &event_stream, &projection);

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
        state_explanation,
        evidence_summary,
        event_stream,
        timeline: projection.timeline.clone(),
        freshness,
    })
}

pub fn get_work_loop_run_view(
    project_root: impl AsRef<Path>,
    issue_id: &str,
    run_id: &str,
) -> Result<WorkLoopRunView> {
    let projection = load_task_projection(&project_root, issue_id)?;
    let task_run = load_task_run(&project_root, issue_id, run_id)?;
    let event_stream = collect_work_loop_events(
        &project_root,
        ReplayFilter::run(issue_id.to_string(), run_id.to_string()),
        EventStreamScope::Run {
            run_id: run_id.to_string(),
        },
    )?;
    let evidence_summary = build_work_loop_evidence_summary(
        &project_root,
        issue_id,
        Some(run_id),
        task_run.session_id.as_deref(),
        &projection.public_delivery,
    );
    let run_state = task_run_status_label(&task_run.status).to_string();
    let state_explanation = explain_run_state(&run_state, &event_stream, &task_run);
    let freshness = explain_projection_staleness(
        &project_root,
        ProjectionScope::Issue {
            issue_id: issue_id.to_string(),
        },
        &projection.version,
        projection.updated_at,
        projection
            .timeline
            .iter()
            .flat_map(|item| item.events.iter().map(|event| event.event_id.clone()))
            .last(),
    )?;

    Ok(WorkLoopRunView {
        issue_id: issue_id.to_string(),
        run_id: run_id.to_string(),
        issue_state: projection.current_state,
        run_state,
        branch_name: task_run.branch_name,
        session_id: task_run.session_id,
        session_status: task_run.session_status,
        state_explanation,
        evidence_summary,
        event_stream,
        freshness,
    })
}

pub fn get_work_loop_session_view(
    project_root: impl AsRef<Path>,
    session_id: &str,
) -> Result<WorkLoopSessionView> {
    let (issue_id, run_id, projection) =
        find_session_projection_context(&project_root, session_id)?;
    let session_record =
        load_task_session_history_record(&project_root, &issue_id, &run_id, session_id)?;
    let recovery_summary =
        load_task_session_recovery_summary(&project_root, &issue_id, &run_id).ok();
    let event_stream = collect_work_loop_events(
        &project_root,
        ReplayFilter::run(issue_id.clone(), run_id.clone()),
        EventStreamScope::Session {
            run_id: run_id.clone(),
            session_id: session_id.to_string(),
        },
    )?;
    let evidence_summary = build_work_loop_evidence_summary(
        &project_root,
        &issue_id,
        Some(&run_id),
        Some(session_id),
        &projection.public_delivery,
    );
    let state_explanation =
        explain_session_state(&session_record, recovery_summary.as_ref(), &event_stream);
    let freshness = explain_projection_staleness(
        &project_root,
        ProjectionScope::Issue {
            issue_id: issue_id.clone(),
        },
        &projection.version,
        projection.updated_at,
        projection
            .timeline
            .iter()
            .flat_map(|item| item.events.iter().map(|event| event.event_id.clone()))
            .last(),
    )?;

    Ok(WorkLoopSessionView {
        issue_id,
        run_id,
        session_id: session_id.to_string(),
        provider: Some(session_record.provider),
        session_owner: Some(session_record.session_owner),
        session_status: Some(session_record.status.as_str().to_string()),
        attempt_count: session_record.attempt_count,
        started_at: Some(session_record.started_at),
        last_heartbeat_at: Some(session_record.last_heartbeat_at),
        recovery_reason: recovery_summary
            .as_ref()
            .and_then(|summary| summary.recovery_reason.clone())
            .or(session_record.recovery_reason),
        resumed_from_attempt: recovery_summary
            .as_ref()
            .and_then(|summary| summary.resumed_from_attempt)
            .or(session_record.resumed_from_attempt),
        retry_policy: session_record.retry_policy,
        retryable: Some(session_record.retryable),
        terminal_reason: session_record.terminal_reason,
        last_error: session_record.last_error,
        state_explanation,
        evidence_summary,
        event_stream,
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

#[derive(Debug, Clone)]
enum EventStreamScope {
    Issue,
    Run { run_id: String },
    Session { run_id: String, session_id: String },
}

fn collect_work_loop_events(
    project_root: impl AsRef<Path>,
    filter: ReplayFilter,
    scope: EventStreamScope,
) -> Result<Vec<WorkLoopEventView>> {
    let events = replay_task_events(project_root, filter)?;
    Ok(events
        .into_iter()
        .filter(|event| match &scope {
            EventStreamScope::Issue => true,
            EventStreamScope::Run { run_id } => event.run_id.as_deref() == Some(run_id.as_str()),
            EventStreamScope::Session { run_id, session_id } => {
                if event.run_id.as_deref() != Some(run_id.as_str()) {
                    return false;
                }
                match payload_string(&event.payload, "sessionId") {
                    Some(value) => value == *session_id,
                    None => true,
                }
            }
        })
        .map(work_loop_event_view)
        .collect())
}

fn work_loop_event_view(event: TaskEvent) -> WorkLoopEventView {
    let compatibility = map_task_event_to_runtime_event(&event).ok();
    let mut evidence_refs = BTreeSet::new();
    let mut artifact_refs = BTreeSet::new();
    if let Some(runtime) = compatibility.as_ref() {
        evidence_refs.extend(runtime.envelope.evidence_refs.iter().cloned());
        artifact_refs.extend(runtime.envelope.artifact_refs.iter().cloned());
    }
    artifact_refs.extend(event.artifact_refs.iter().cloned());

    let (stage_key, stage_label) = work_loop_stage(event.event_type.as_str());
    WorkLoopEventView {
        event_id: event.event_id.clone(),
        event_type: event.event_type.clone(),
        category: classify_task_event(event.event_type.as_str())
            .as_str()
            .to_string(),
        stage_key: stage_key.to_string(),
        stage_label: stage_label.to_string(),
        timestamp: event.timestamp,
        actor_role: event.actor.role.clone(),
        actor_kind: event.actor.kind.clone(),
        run_id: event.run_id.clone(),
        session_id: payload_string(&event.payload, "sessionId"),
        from_state: event.state.as_ref().map(|state| state.from_state.clone()),
        to_state: event.state.as_ref().map(|state| state.to_state.clone()),
        summary: work_loop_event_summary(&event),
        evidence_refs: evidence_refs.into_iter().collect(),
        artifact_refs: artifact_refs.into_iter().collect(),
    }
}

fn work_loop_stage(event_type: &str) -> (&'static str, &'static str) {
    match event_type {
        "issue.scheduled" => ("todo", "准备开工"),
        "issue.preflight.passed"
        | "issue.preflight.failed"
        | "panel.context-pack.ready"
        | "panel.context-pack.failed" => ("preflight", "前置检测"),
        "agent.launch.requested" | "agent.launch.claimed" => ("launch", "启动会话"),
        value if value.starts_with("agent.session.") => ("session", "执行会话"),
        "issue.validation.passed" | "issue.validation.failed" => ("verification", "沙箱验证"),
        "issue.review.requested"
        | "issue.pr.created"
        | "issue.closeout.proof.recorded"
        | "issue.pr.merged" => ("review", "评审收口"),
        "issue.completed" => ("done", "Done 写回"),
        _ => ("event", "事件记录"),
    }
}

fn work_loop_event_summary(event: &TaskEvent) -> String {
    match event.event_type.as_str() {
        "issue.scheduled" => "任务进入待执行队列。".to_string(),
        "agent.launch.requested" => "已生成 Work Agent 启动请求。".to_string(),
        "agent.launch.claimed" => "执行会话已认领启动请求。".to_string(),
        "agent.session.created" => "外部执行会话已创建。".to_string(),
        "agent.session.resumed" => "外部执行会话已恢复。".to_string(),
        "agent.session.running" => "外部执行会话正在运行。".to_string(),
        "agent.session.interrupted" => "外部执行会话已中断，等待恢复。".to_string(),
        "agent.session.in_review" => "外部执行会话已进入评审。".to_string(),
        "agent.session.completed" => "外部执行会话已完成。".to_string(),
        "agent.session.failed" => "外部执行会话失败。".to_string(),
        "agent.session.cancelled" => "外部执行会话已取消。".to_string(),
        "issue.validation.passed" => "本地沙箱验证已通过。".to_string(),
        "issue.validation.failed" => "本地沙箱验证失败。".to_string(),
        "issue.review.requested" => "任务已请求评审。".to_string(),
        "issue.pr.created" => "PR/MR 已创建。".to_string(),
        "issue.closeout.proof.recorded" => "收口证明已写入，等待 Done 写回。".to_string(),
        "issue.pr.merged" => "PR/MR 已合并，等待关单与收口证明。".to_string(),
        "issue.completed" => "任务 Done 写回完成。".to_string(),
        "issue.blocked" => "任务进入阻断状态。".to_string(),
        "issue.cancelled" => "任务已取消。".to_string(),
        other => format!("记录事件：{other}。"),
    }
}

fn payload_string(payload: &Value, key: &str) -> Option<String> {
    payload
        .get(key)
        .and_then(Value::as_str)
        .map(ToString::to_string)
}

fn build_work_loop_evidence_summary(
    project_root: impl AsRef<Path>,
    issue_id: &str,
    run_id: Option<&str>,
    session_id: Option<&str>,
    public_delivery: &ProjectionPublicDelivery,
) -> WorkLoopEvidenceSummaryView {
    let mut status = "missing".to_string();
    let mut summary_parts = Vec::new();
    let mut verification_refs = BTreeSet::new();
    let mut session_refs = BTreeSet::new();
    let mut delivery_refs = BTreeSet::new();

    if let Ok(evidence) = load_task_evidence(&project_root, issue_id) {
        status = evidence.status.clone();
        if !evidence.summary.trim().is_empty() {
            summary_parts.push(evidence.summary);
        }
        verification_refs.insert(evidence.run_path);
        verification_refs.insert(evidence.validation_path);
        verification_refs.extend(evidence.command_paths);
        if let Some(changed_files_path) = evidence.changed_files_path {
            verification_refs.insert(changed_files_path);
        }
    }

    if let (Some(run_id), Some(_session_id)) = (run_id, session_id) {
        if let Ok(session_evidence) = load_task_session_evidence(&project_root, issue_id, run_id) {
            status = session_evidence.status.as_str().to_string();
            if !session_evidence.summary.trim().is_empty() {
                summary_parts.push(session_evidence.summary);
            }
            session_refs.extend(session_evidence.refs);
        }
    }

    if let Some(pr_url) = public_delivery.pr_url.clone() {
        delivery_refs.insert(pr_url);
    }
    if let Some(evidence_path) = public_delivery.evidence_path.clone() {
        delivery_refs.insert(evidence_path);
    }
    if let Some(changelog_path) = public_delivery.changelog_path.clone() {
        delivery_refs.insert(changelog_path);
    }
    if let Some(release_notes_url) = public_delivery.release_notes_url.clone() {
        delivery_refs.insert(release_notes_url);
    }

    if summary_parts.is_empty() {
        summary_parts.push("当前还没有可展示的验证或交付证据。".to_string());
    }

    WorkLoopEvidenceSummaryView {
        status,
        summary: summary_parts.join(" "),
        verification_refs: verification_refs.into_iter().collect(),
        session_refs: session_refs.into_iter().collect(),
        delivery_refs: delivery_refs.into_iter().collect(),
    }
}

fn explain_issue_state(
    current_state: &str,
    event_stream: &[WorkLoopEventView],
    projection: &TaskProjection,
) -> String {
    if let Some(last_event) = event_stream.last() {
        return match current_state {
            "todo" => format!("任务已满足开工前置条件，当前停在 {}。", last_event.summary),
            "in_progress" => format!("任务正在执行，当前事实是：{}。", last_event.summary),
            "in_review" => format!("任务已进入评审，当前事实是：{}。", last_event.summary),
            "done" => format!("任务已完成，最终写回事实是：{}。", last_event.summary),
            "blocked" => format!("任务已阻断，最近事实是：{}。", last_event.summary),
            "cancel" => format!("任务已取消，最近事实是：{}。", last_event.summary),
            _ => format!("当前状态由最新事件驱动：{}。", last_event.summary),
        };
    }

    match current_state {
        "backlog" => "任务还未进入执行，等待进入调度。".to_string(),
        "todo" => "任务已准备开工，等待拉起执行会话。".to_string(),
        "in_progress" => "任务正在执行，但还没有生成可展示事件。".to_string(),
        "in_review" => "任务已进入评审，但还没有生成可展示事件。".to_string(),
        "done" => "任务已完成。".to_string(),
        "blocked" => projection
            .audit
            .findings
            .first()
            .cloned()
            .unwrap_or_else(|| "任务被阻断。".to_string()),
        "cancel" => "任务已取消。".to_string(),
        _ => "当前状态没有额外解释。".to_string(),
    }
}

fn explain_run_state(
    run_state: &str,
    event_stream: &[WorkLoopEventView],
    task_run: &agentflow_task_artifacts::TaskRun,
) -> String {
    if let Some(last_event) = event_stream.last() {
        return format!(
            "当前 run 状态是 {run_state}，最近事件是：{}。",
            last_event.summary
        );
    }
    match run_state {
        "queued" => "当前 run 已创建，等待真正进入执行。".to_string(),
        "in_progress" => "当前 run 正在执行。".to_string(),
        "validating" => "当前 run 正在收集验证结果。".to_string(),
        "completed" => "当前 run 已完成，等待或已经进入后续写回。".to_string(),
        "failed" => task_run
            .last_error
            .clone()
            .map(|error| format!("当前 run 已失败：{error}。"))
            .unwrap_or_else(|| "当前 run 已失败。".to_string()),
        "cancelled" => "当前 run 已取消。".to_string(),
        _ => "当前 run 状态未知。".to_string(),
    }
}

fn explain_session_state(
    session_record: &agentflow_task_artifacts::TaskWorkSessionRecord,
    recovery_summary: Option<&agentflow_task_artifacts::TaskWorkSessionRecoverySummary>,
    event_stream: &[WorkLoopEventView],
) -> String {
    if let Some(last_event) = event_stream.last() {
        return format!(
            "当前会话状态是 {}，最近事件是：{}。",
            session_record.status.as_str(),
            last_event.summary
        );
    }
    if let Some(summary) = recovery_summary {
        return format!(
            "当前会话处于 {}，恢复原因：{}。",
            session_record.status.as_str(),
            summary
                .recovery_reason
                .clone()
                .unwrap_or_else(|| "未记录".to_string())
        );
    }
    format!("当前会话状态是 {}。", session_record.status.as_str())
}

fn find_session_projection_context(
    project_root: impl AsRef<Path>,
    session_id: &str,
) -> Result<(String, String, TaskProjection)> {
    let index = load_issue_status_index(&project_root)?;
    for entry in index.issues {
        let projection = load_task_projection(&project_root, &entry.issue_id)?;
        let Some(run_id) = projection.latest_run_id.clone() else {
            continue;
        };
        if projection.session.session_id.as_deref() == Some(session_id)
            || load_task_session_history_record(&project_root, &entry.issue_id, &run_id, session_id)
                .is_ok()
        {
            return Ok((entry.issue_id, run_id, projection));
        }
    }
    anyhow::bail!("failed to locate work session `{session_id}` in task projections")
}

fn task_run_status_label(status: &agentflow_task_artifacts::TaskRunStatus) -> &'static str {
    match status {
        agentflow_task_artifacts::TaskRunStatus::Queued => "queued",
        agentflow_task_artifacts::TaskRunStatus::InProgress => "in_progress",
        agentflow_task_artifacts::TaskRunStatus::Validating => "validating",
        agentflow_task_artifacts::TaskRunStatus::Completed => "completed",
        agentflow_task_artifacts::TaskRunStatus::Failed => "failed",
        agentflow_task_artifacts::TaskRunStatus::Cancelled => "cancelled",
    }
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
    let summary = work_loop_event_summary(&event);
    RuntimeEventRow {
        event_id: event.event_id.clone(),
        event_type: event.event_type.clone(),
        timestamp: event.timestamp,
        actor_role: event.actor.role.clone(),
        summary,
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
        materialize_spec_from_requirement_preview, project_from_requirement,
        requirement_preview_from_requirement, write_spec_issue, write_spec_project, SpecIssueDraft,
        SpecIssueStatus, SpecProjectDraft,
    };
    use agentflow_task_artifacts::{
        create_task_run, sync_task_session, update_task_run_status, TaskRunStatus,
        TaskSessionMirror,
    };
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

    fn sync_running_session(root: &Path, issue_id: &str, run_id: &str, session_id: &str) {
        sync_task_session(
            root,
            issue_id,
            run_id,
            &TaskSessionMirror {
                provider: "codex".to_string(),
                session_owner: "work-agent".to_string(),
                session_id: session_id.to_string(),
                status: agentflow_task_artifacts::TaskWorkSessionStatus::Running,
                branch_name: Some(format!("agentflow/{issue_id}/{run_id}")),
                working_directory: root.display().to_string(),
                workspace_root: Some(root.display().to_string()),
                worktree_root: Some(root.display().to_string()),
                runtime_root: Some(
                    root.join(format!(".agentflow/tasks/{issue_id}/runs/{run_id}/runtime"))
                        .display()
                        .to_string(),
                ),
                temp_root: None,
                cache_root: None,
                evidence_root: Some(
                    root.join(format!(".agentflow/tasks/{issue_id}/evidence"))
                        .display()
                        .to_string(),
                ),
                launch_request_path: format!(
                    ".agentflow/tasks/{issue_id}/runs/{run_id}/launch/agent-request.json"
                ),
                plan_path: format!(".agentflow/tasks/{issue_id}/runs/{run_id}/plan.json"),
                log_path: Some(format!(
                    ".agentflow/tasks/{issue_id}/runs/{run_id}/runtime.log"
                )),
                last_message_path: None,
                exit_proof_path: None,
                merge_proof_path: None,
                started_at: 10,
                last_heartbeat_at: 40,
                attempt_count: 2,
                retry_policy: Some("reuse-session-or-relaunch".to_string()),
                max_attempts: Some(3),
                resumed_from_attempt: Some(1),
                retryable: true,
                recovery_reason: Some("timeout".to_string()),
                merge_state: None,
                writeback_state: None,
                terminal_reason: None,
                last_error: None,
                exited_at: None,
                exit_code: None,
                updated_at: 40,
            },
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
    fn task_workbench_view_exposes_event_stream_and_evidence_summary() {
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
        append_task_event_once(
            dir.path(),
            event(
                "AF-PROJ-001",
                "agent.session.running",
                json!({"runId":"run-001","sessionId":"codex-run-001","sessionStatus":"running","provider":"codex","ownerId":"work-agent"}),
            ),
        )
        .unwrap();
        append_task_event_once(
            dir.path(),
            event(
                "AF-PROJ-001",
                "issue.validation.passed",
                json!({"runId":"run-001"}),
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
        sync_running_session(dir.path(), "AF-PROJ-001", "run-001", "codex-run-001");
        write_completion_ready_artifacts(dir.path(), "AF-PROJ-001", "run-001");

        rebuild_projections(dir.path()).unwrap();
        let view = get_task_workbench_view(dir.path(), "AF-PROJ-001").unwrap();

        assert!(view.state_explanation.contains("当前事实"));
        assert!(view
            .event_stream
            .iter()
            .any(|event| event.stage_key == "session"));
        assert!(view
            .event_stream
            .iter()
            .any(|event| event.event_type == "issue.validation.passed"));
        assert_eq!(view.evidence_summary.status, "running");
        assert!(!view.evidence_summary.verification_refs.is_empty());
        assert!(!view.evidence_summary.session_refs.is_empty());
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
    fn work_loop_run_view_filters_events_and_keeps_done_writeback_visible() {
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
                json!({"runId":"run-001"}),
            ),
        )
        .unwrap();
        append_task_event_once(
            dir.path(),
            event(
                "AF-PROJ-001",
                "issue.validation.passed",
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
            TaskRunStatus::Completed,
        )
        .unwrap();
        write_completion_ready_artifacts(dir.path(), "AF-PROJ-001", "run-001");

        rebuild_projections(dir.path()).unwrap();
        let view = get_work_loop_run_view(dir.path(), "AF-PROJ-001", "run-001").unwrap();

        assert_eq!(view.run_id, "run-001");
        assert_eq!(view.run_state, "completed");
        assert!(view.state_explanation.contains("Done 写回"));
        assert_eq!(
            view.event_stream
                .last()
                .map(|event| event.event_type.as_str()),
            Some("issue.completed")
        );
        assert!(!view.evidence_summary.verification_refs.is_empty());
    }

    #[test]
    fn work_loop_session_view_reads_recovery_and_session_evidence() {
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
                json!({"runId":"run-001"}),
            ),
        )
        .unwrap();
        append_task_event_once(
            dir.path(),
            event(
                "AF-PROJ-001",
                "agent.session.interrupted",
                json!({
                    "runId":"run-001",
                    "sessionId":"codex-run-001",
                    "sessionStatus":"interrupted",
                    "provider":"codex",
                    "ownerId":"work-agent",
                    "startedAt":10,
                    "lastHeartbeatAt":40,
                    "recoveryReason":"timeout"
                }),
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
        sync_running_session(dir.path(), "AF-PROJ-001", "run-001", "codex-run-001");

        rebuild_projections(dir.path()).unwrap();
        let view = get_work_loop_session_view(dir.path(), "codex-run-001").unwrap();

        assert_eq!(view.issue_id, "AF-PROJ-001");
        assert_eq!(view.run_id, "run-001");
        assert_eq!(view.session_status.as_deref(), Some("running"));
        assert_eq!(view.recovery_reason.as_deref(), Some("timeout"));
        assert_eq!(view.resumed_from_attempt, Some(1));
        assert_eq!(view.attempt_count, 2);
        assert!(view
            .event_stream
            .iter()
            .any(|event| event.event_type == "agent.session.interrupted"));
        assert!(!view.evidence_summary.session_refs.is_empty());
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

    #[test]
    fn spec_loop_view_covers_stage_files_traceability_and_action_proposals() {
        let dir = tempdir().unwrap();
        let requirement = dir.path().join("docs/requirements/041-spec-loop-view.md");
        fs::create_dir_all(requirement.parent().unwrap()).unwrap();
        fs::write(
            &requirement,
            "# Spec Loop View\n\n把需求转成 preview、confirmation、materialization 和 runtime action proposal。\n",
        )
        .unwrap();

        requirement_preview_from_requirement(dir.path(), &requirement, Some("project-preview"))
            .unwrap();
        confirm_goal_draft_preview(dir.path(), "041-spec-loop-view", "goal-agent").unwrap();
        confirm_plan_draft_preview(dir.path(), "041-spec-loop-view", "spec-agent").unwrap();
        materialize_spec_from_requirement_preview(dir.path(), "041-spec-loop-view").unwrap();
        rebuild_projections(dir.path()).unwrap();

        let view = get_spec_loop_view(dir.path(), "041-spec-loop-view").unwrap();

        assert_eq!(view.requirement_id, "041-spec-loop-view");
        assert_eq!(
            view.manifest_path,
            ".agentflow/spec/requirements/041-spec-loop-view/manifest.json"
        );
        assert_eq!(view.stages.len(), 8);
        assert!(view
            .stages
            .iter()
            .all(|stage| stage.authority_layer == "preview-artifact"));
        assert_eq!(
            view.stages
                .iter()
                .map(|stage| stage.stage.as_str())
                .collect::<Vec<_>>(),
            vec![
                "intake",
                "classification",
                "context",
                "boundary",
                "route",
                "preview",
                "confirmation",
                "materialization"
            ]
        );
        assert_eq!(
            view.materialized_project_id.as_deref(),
            Some("project-preview")
        );
        assert!(view.authority_layers.iter().any(|entry| {
            entry.authority_layer == "preview-artifact"
                && entry.path == ".agentflow/spec/requirements/041-spec-loop-view"
        }));
        assert!(view.authority_layers.iter().any(|entry| {
            entry.authority_layer == "project-authority"
                && entry.path == ".agentflow/spec/projects/project-preview.json"
        }));
        assert_eq!(
            view.authority_layers
                .iter()
                .filter(|entry| {
                    entry.authority_layer == "issue-authority"
                        && entry.path.starts_with(".agentflow/spec/issues/")
                })
                .count(),
            view.materialized_issue_ids.len()
        );
        assert!(view.authority_layers.iter().any(|entry| {
            entry.authority_layer == "derived-projection"
                && entry.path == ".agentflow/projections/spec-loops/041-spec-loop-view.json"
        }));
        assert_eq!(
            view.runtime_action_proposals.len(),
            1 + view.materialized_issue_ids.len()
        );
        assert!(view.traceability.iter().any(|edge| {
            edge.from_ref == "docs/requirements/041-spec-loop-view.md"
                && edge.to_ref.ends_with("/intake.json")
                && edge.relation == "stage-input"
        }));
        assert!(view.traceability.iter().any(|edge| {
            edge.from_ref.ends_with("/materialization.json")
                && edge
                    .to_ref
                    .starts_with("runtime-action-proposal:createProject:")
                && edge.relation == "runtime-action-proposal"
        }));
        assert_eq!(view.freshness.staleness, "current");
    }
}
