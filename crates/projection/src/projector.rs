use crate::{
    model::{
        CompletionDecisionIndex, CompletionDecisionIndexEntry, CompletionDecisionProjection,
        IssueStatusIndex, IssueStatusIndexEntry, ProjectBlockerSummary, ProjectBrainProjection,
        ProjectCompletionProjection, ProjectExternalReviewProjection, ProjectIssueLanes,
        ProjectProjection, ProjectReleaseProjection, ProjectionAcceptanceSubGateSummary,
        ProjectionAcceptanceSummary, ProjectionAcceptanceTraceabilitySummary,
        ProjectionAuditSummary, ProjectionDeliverySummary, ProjectionPhase,
        ProjectionPublicDelivery, ProjectionRuntimeSummary, ProjectionSessionSummary,
        ProjectionSummary, RequirementPreviewIndex, RequirementPreviewIndexEntry,
        RequirementPreviewProjection, SpecLoopActionProposalProjection, SpecLoopProjection,
        SpecLoopStageProjection, SpecLoopTraceabilityEdge, TaskProjection, TaskTimelineEvent,
        TaskTimelineItem, COMPLETION_DECISION_INDEX_VERSION,
        COMPLETION_DECISION_PROJECTION_VERSION, ISSUE_STATUS_INDEX_VERSION,
        PROJECT_PROJECTION_VERSION, REQUIREMENT_PREVIEW_INDEX_VERSION,
        REQUIREMENT_PREVIEW_PROJECTION_VERSION, SPEC_LOOP_PROJECTION_VERSION,
        TASK_PROJECTION_VERSION,
    },
    storage::{
        write_completion_decision_index, write_completion_decision_projection,
        write_issue_status_index, write_project_projection, write_requirement_preview_index,
        write_requirement_preview_projection, write_spec_loop_projection, write_task_projection,
    },
};
use agentflow_audit::load_audit_result_summary;
use agentflow_event_store::{load_task_events, EventStateTransition, TaskEvent};
use agentflow_release::{
    load_delivery_summary, load_project_delivery_summary, load_project_external_review_surface,
    load_project_release_facts,
};
use agentflow_spec::{
    list_completion_decision_runtimes, list_requirement_preview_runtimes,
    list_spec_loop_stage_artifacts, prepare_spec_workspace, read_project_brain_snapshot,
    read_spec_loop_requirement_manifest, sync_completion_decision_runtimes,
    CompletionDecisionRuntime, RequirementPreviewRuntime, SpecIssue, SpecProject,
    SpecProjectStatus,
};
use agentflow_workflow_runtime::{
    load_runtime_command_bundle, runtime_accepted_action_fact_path, runtime_command_fact_path,
    runtime_decision_fact_path, runtime_proposal_fact_path,
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
    let projects_by_id = projects
        .iter()
        .map(|project| (project.project_id.clone(), project))
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
        let projection_path = write_requirement_preview_projection(&root, &projection)?
            .display()
            .to_string();
        let spec_loop_projection =
            project_spec_loop(&root, preview, &issues_by_id, &projects_by_id)?;
        write_spec_loop_projection(&root, &spec_loop_projection)?;
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
        let projection_path = write_completion_decision_projection(&root, &projection)?
            .display()
            .to_string();
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

fn project_spec_loop(
    root: &Path,
    preview: &RequirementPreviewRuntime,
    issues_by_id: &HashMap<String, &SpecIssue>,
    projects_by_id: &HashMap<String, &SpecProject>,
) -> Result<SpecLoopProjection> {
    let manifest = read_spec_loop_requirement_manifest(root, &preview.requirement_id)?;
    let stage_artifacts = list_spec_loop_stage_artifacts(root, &preview.requirement_id)?;
    let stage_entries = manifest
        .stage_files
        .iter()
        .map(|entry| (entry.stage.as_str().to_string(), entry))
        .collect::<HashMap<_, _>>();

    let stages = stage_artifacts
        .iter()
        .map(|artifact| SpecLoopStageProjection {
            stage: artifact.stage.as_str().to_string(),
            path: stage_entries
                .get(artifact.stage.as_str())
                .map(|entry| entry.path.clone())
                .unwrap_or_else(|| artifact.stage.as_str().to_string()),
            status: artifact.status.as_str().to_string(),
            authority: artifact.authority.as_str().to_string(),
            authority_layer: stage_entries
                .get(artifact.stage.as_str())
                .map(|entry| entry.authority_layer.as_str().to_string())
                .unwrap_or_else(|| "preview-artifact".to_string()),
            current_state: artifact.current_state.clone(),
            input_refs: artifact.input_refs.clone(),
            output_refs: artifact.output_refs.clone(),
            evidence_refs: artifact.evidence_refs.clone(),
            summary: artifact.summary.clone(),
            updated_at: artifact.updated_at,
        })
        .collect::<Vec<_>>();
    let authority_layers = manifest
        .authority_layers
        .iter()
        .map(|entry| crate::model::SpecLoopAuthorityLayerProjection {
            authority_layer: entry.authority_layer.as_str().to_string(),
            path: entry.path.clone(),
            summary: entry.summary.clone(),
        })
        .collect::<Vec<_>>();

    let runtime_action_proposals =
        build_spec_loop_action_proposals(root, preview, issues_by_id, projects_by_id);
    let traceability = build_spec_loop_traceability(&stages, &runtime_action_proposals);

    Ok(SpecLoopProjection {
        version: SPEC_LOOP_PROJECTION_VERSION.to_string(),
        requirement_id: preview.requirement_id.clone(),
        requirement_path: preview.requirement_path.clone(),
        project_id: preview.project_id.clone(),
        project_title: preview.project_title.clone(),
        lifecycle: preview.lifecycle.as_str().to_string(),
        current_state: preview.current_state.clone(),
        manifest_path: format!(
            ".agentflow/spec/requirements/{}/manifest.json",
            preview.requirement_id
        ),
        runtime_path: manifest.runtime_path,
        next_recommended_action: preview.next_recommended_action.clone(),
        next_recommended_action_label: preview.next_recommended_action_label.clone(),
        next_recommended_action_reason: preview.next_recommended_action_reason.clone(),
        materialized_project_id: preview.materialized_project_id.clone(),
        materialized_issue_ids: preview.materialized_issue_ids.clone(),
        stages,
        authority_layers,
        traceability,
        runtime_action_proposals,
        updated_at: preview.updated_at,
    })
}

fn build_spec_loop_traceability(
    stages: &[SpecLoopStageProjection],
    runtime_action_proposals: &[SpecLoopActionProposalProjection],
) -> Vec<SpecLoopTraceabilityEdge> {
    let mut edges = Vec::new();
    let materialization_path = stages
        .iter()
        .find(|stage| stage.stage == "materialization")
        .map(|stage| stage.path.clone());
    for stage in stages {
        for input_ref in &stage.input_refs {
            edges.push(SpecLoopTraceabilityEdge {
                from_ref: input_ref.clone(),
                to_ref: stage.path.clone(),
                relation: "stage-input".to_string(),
            });
        }
        for output_ref in &stage.output_refs {
            if output_ref == &stage.path {
                continue;
            }
            edges.push(SpecLoopTraceabilityEdge {
                from_ref: stage.path.clone(),
                to_ref: output_ref.clone(),
                relation: "stage-output".to_string(),
            });
        }
        for evidence_ref in &stage.evidence_refs {
            edges.push(SpecLoopTraceabilityEdge {
                from_ref: evidence_ref.clone(),
                to_ref: stage.path.clone(),
                relation: "stage-evidence".to_string(),
            });
        }
    }
    if let Some(materialization_path) = materialization_path {
        for proposal in runtime_action_proposals {
            edges.push(SpecLoopTraceabilityEdge {
                from_ref: materialization_path.clone(),
                to_ref: proposal.proposal_ref.clone(),
                relation: "runtime-action-proposal".to_string(),
            });
        }
    }
    edges
}

fn build_spec_loop_action_proposals(
    root: &Path,
    preview: &RequirementPreviewRuntime,
    issues_by_id: &HashMap<String, &SpecIssue>,
    projects_by_id: &HashMap<String, &SpecProject>,
) -> Vec<SpecLoopActionProposalProjection> {
    let mut proposals = Vec::new();
    if let (Some(project_id), Some(plan_draft)) = (
        preview.materialized_project_id.as_ref(),
        preview.plan_draft.as_ref(),
    ) {
        proposals.push(enrich_runtime_action_proposal(
            root,
            format!("cmd-create-project-{project_id}"),
            SpecLoopActionProposalProjection {
                proposal_ref: format!("runtime-action-proposal:createProject:{project_id}"),
                action_type: "createProject".to_string(),
                target_object_type: "Spec".to_string(),
                target_object_id: plan_draft.plan_draft_id.clone(),
                created_object_type: Some("Project".to_string()),
                created_object_id: Some(project_id.clone()),
                actor_role: "spec-agent".to_string(),
                handoff_rule: None,
                command_status: None,
                decision_status: None,
                accepted_action_id: None,
                command_path: None,
                proposal_path: None,
                decision_path: None,
                accepted_action_path: None,
            },
        ));
        if let Some(project) = projects_by_id.get(project_id) {
            for issue_id in &project.issue_ids {
                if let Some(issue) = issues_by_id.get(issue_id) {
                    proposals.push(enrich_runtime_action_proposal(
                        root,
                        format!("cmd-create-issue-{issue_id}"),
                        SpecLoopActionProposalProjection {
                            proposal_ref: format!("runtime-action-proposal:createIssue:{issue_id}"),
                            action_type: "createIssue".to_string(),
                            target_object_type: "Project".to_string(),
                            target_object_id: project_id.clone(),
                            created_object_type: Some("Issue".to_string()),
                            created_object_id: Some(issue.issue_id.clone()),
                            actor_role: "spec-agent".to_string(),
                            handoff_rule: Some("spec-to-work-approved-spec".to_string()),
                            command_status: None,
                            decision_status: None,
                            accepted_action_id: None,
                            command_path: None,
                            proposal_path: None,
                            decision_path: None,
                            accepted_action_path: None,
                        },
                    ));
                }
            }
        } else {
            for issue_id in &preview.materialized_issue_ids {
                proposals.push(enrich_runtime_action_proposal(
                    root,
                    format!("cmd-create-issue-{issue_id}"),
                    SpecLoopActionProposalProjection {
                        proposal_ref: format!("runtime-action-proposal:createIssue:{issue_id}"),
                        action_type: "createIssue".to_string(),
                        target_object_type: "Project".to_string(),
                        target_object_id: project_id.clone(),
                        created_object_type: Some("Issue".to_string()),
                        created_object_id: Some(issue_id.clone()),
                        actor_role: "spec-agent".to_string(),
                        handoff_rule: Some("spec-to-work-approved-spec".to_string()),
                        command_status: None,
                        decision_status: None,
                        accepted_action_id: None,
                        command_path: None,
                        proposal_path: None,
                        decision_path: None,
                        accepted_action_path: None,
                    },
                ));
            }
        }
    }
    proposals
}

fn enrich_runtime_action_proposal(
    root: &Path,
    command_id: String,
    mut proposal: SpecLoopActionProposalProjection,
) -> SpecLoopActionProposalProjection {
    let proposal_id = format!("proposal-{command_id}");
    let command_path = runtime_command_fact_path(root, &command_id);
    let proposal_path = runtime_proposal_fact_path(root, &proposal_id);
    let decision_path = runtime_decision_fact_path(root, &proposal_id);
    let command_path_str = command_path.display().to_string();
    let proposal_path_str = proposal_path.display().to_string();
    let decision_path_str = decision_path.display().to_string();
    proposal.command_path = Some(relative_projection_path(root, &command_path_str));
    proposal.proposal_path = Some(relative_projection_path(root, &proposal_path_str));
    proposal.decision_path = Some(relative_projection_path(root, &decision_path_str));
    if let Ok(bundle) = load_runtime_command_bundle(root, &command_id) {
        proposal.command_status = Some(if bundle.command.validation.valid {
            "recorded".to_string()
        } else {
            "invalid".to_string()
        });
        if let Some(decision) = bundle.decision {
            proposal.decision_status = Some(decision.status.clone());
            proposal.accepted_action_id = decision.accepted_action_id.clone();
            if let Some(accepted_action_id) = decision.accepted_action_id {
                let accepted_action_path =
                    runtime_accepted_action_fact_path(root, &accepted_action_id);
                let accepted_action_path_str = accepted_action_path.display().to_string();
                proposal.accepted_action_path =
                    Some(relative_projection_path(root, &accepted_action_path_str));
            }
        }
    }
    proposal
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
        task_evidence_ready_count: runtime.facts.task_evidence_ready_count,
        task_evidence_missing_count: runtime.facts.task_evidence_missing_count,
        delivery_status: runtime.facts.delivery_status.clone(),
        delivery_missing_count: runtime.facts.delivery_missing_count,
        audit_required: runtime.facts.audit_required,
        audit_status: runtime.facts.audit_status.clone(),
        audit_blocking_findings: runtime.facts.audit_blocking_findings,
        goal_recheck_status: runtime.facts.goal_recheck_status.clone(),
        project_health_status: runtime.facts.project_health_status.clone(),
        release_readiness: runtime.facts.release_readiness.clone(),
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
    let terminal_issue_authority = matches!(current_state.as_str(), "done" | "cancel");
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
        let mut event_state = current_state.clone();
        if let Some(next_state) = authoritative_state_from_event(&event) {
            let next_state_is_terminal = matches!(next_state.as_str(), "done" | "cancel");
            event_state = next_state.clone();
            if !terminal_issue_authority || next_state_is_terminal {
                current_state = next_state;
            }
            if let Some(run_id) = event_run_id.as_deref() {
                authoritative_run_id = Some(run_id.to_string());
                active_run_id = Some(run_id.to_string());
            }
        }
        state_events.push((event_state, event));
    }
    let latest_run_id = authoritative_run_id.or(active_run_id).or(latest_run_id);

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
    let acceptance = build_acceptance_summary(root, issue);
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
        acceptance,
        updated_at,
    }
}

fn build_acceptance_summary(root: &Path, issue: &SpecIssue) -> Option<ProjectionAcceptanceSummary> {
    let decision =
        agentflow_task_artifacts::load_task_acceptance_gate_decision(root, &issue.issue_id).ok()?;
    Some(ProjectionAcceptanceSummary {
        outcome: decision.outcome.as_str().to_string(),
        passed: decision.passed,
        summary: decision.summary,
        failure_reasons: decision.failure_reasons,
        next_steps: decision.next_steps,
        sub_gates: decision
            .sub_gates
            .into_iter()
            .map(|gate| ProjectionAcceptanceSubGateSummary {
                gate: gate.gate.as_str().to_string(),
                passed: gate.passed,
                failure_reasons: gate.failure_reasons,
                repair_suggestion: gate.repair_suggestion,
            })
            .collect(),
        traceability: ProjectionAcceptanceTraceabilitySummary {
            issue_id: decision.traceability.issue_id,
            run_id: decision.traceability.run_id,
            acceptance_decision_path: decision.traceability.acceptance_decision_path,
            evidence_path: decision.traceability.evidence_path,
            validation_path: decision.traceability.validation_path,
            closeout_proof_path: decision.traceability.closeout_proof_path,
            session_id: decision.traceability.session_id,
            provider: decision.traceability.provider,
            pr_url: decision.traceability.pr_url,
            merge_commit_sha: decision.traceability.merge_commit_sha,
        },
        checked_at: decision.checked_at,
    })
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
    let mut current_issue_priority = None;
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
                let issue_priority = active_project_issue_priority(task.current_state.as_str());
                if current_issue_priority
                    .map(|priority| issue_priority < priority)
                    .unwrap_or(true)
                {
                    current_issue_id = Some(issue_id.clone());
                    current_issue_state = Some(task.current_state.clone());
                    current_issue_priority = Some(issue_priority);
                }
                current_lane.push(issue_id.clone());
            }
        }
    }
    let all_finished = completed == project.issue_ids.len() && !project.issue_ids.is_empty();
    let completion_projection = completion.map(project_completion_decision);
    let project_delivery = build_project_delivery_summary(root, project, tasks);
    let project_release = load_project_release_facts(root, &project.project_id).ok();
    let project_external_review =
        load_project_external_review_surface(root, &project.project_id).ok();
    let project_audit = build_project_audit_summary(project, tasks);
    let status = if all_finished {
        match completion_projection
            .as_ref()
            .map(|projection| projection.current_state.as_str())
        {
            Some("accepted") => "done",
            Some("pause") => "blocked",
            _ => "active",
        }
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
    let (stage_key, stage_label, stage_summary) = if all_finished {
        if let Some(release) = project_release.as_ref() {
            match release.current_state.as_str() {
                "published" => (
                    "release-published".to_string(),
                    "已发布".to_string(),
                    release.summary_line.clone(),
                ),
                "in_progress" => match release.publication_stage.as_str() {
                    "public-record-written" => (
                        "release-public-record-written".to_string(),
                        "公开记录已写入".to_string(),
                        release.summary_line.clone(),
                    ),
                    "tag-created" => (
                        "release-tag-created".to_string(),
                        "Tag 已记录".to_string(),
                        release.summary_line.clone(),
                    ),
                    "remote-release-created" => (
                        "release-remote-created".to_string(),
                        "远端发布已确认".to_string(),
                        release.summary_line.clone(),
                    ),
                    _ => (
                        "release-in-progress".to_string(),
                        "发布中".to_string(),
                        release.summary_line.clone(),
                    ),
                },
                "ready" => (
                    "release-ready".to_string(),
                    "待发布".to_string(),
                    release.summary_line.clone(),
                ),
                "blocked" => (
                    "release-blocked".to_string(),
                    "发布阻断".to_string(),
                    release.summary_line.clone(),
                ),
                _ => match completion_projection.as_ref() {
                    Some(projection) if projection.current_state == "accepted" => (
                        "done".to_string(),
                        "项目已完成".to_string(),
                        "全部任务已完成，完成判断已经接受。".to_string(),
                    ),
                    Some(projection) if projection.current_state == "continue" => (
                        "continue".to_string(),
                        "继续推进".to_string(),
                        projection.next_recommended_action_reason.clone(),
                    ),
                    Some(projection) if projection.current_state == "adjust" => (
                        "adjust".to_string(),
                        "需要调整".to_string(),
                        projection.next_recommended_action_reason.clone(),
                    ),
                    Some(projection) if projection.current_state == "pause" => (
                        "pause".to_string(),
                        "已暂停".to_string(),
                        projection.next_recommended_action_reason.clone(),
                    ),
                    Some(projection) if projection.current_state == "next-stage" => (
                        "next-stage".to_string(),
                        "进入下一阶段".to_string(),
                        projection.next_recommended_action_reason.clone(),
                    ),
                    _ => (
                        "completion-ready".to_string(),
                        "等待完成判断".to_string(),
                        "任务已全部完成，正在等待 Goal Recheck / Completion Runtime 做最后判断。"
                            .to_string(),
                    ),
                },
            }
        } else {
            match completion_projection.as_ref() {
                Some(projection) if projection.current_state == "accepted" => (
                    "done".to_string(),
                    "项目已完成".to_string(),
                    "全部任务已完成，完成判断已经接受。".to_string(),
                ),
                Some(projection) if projection.current_state == "continue" => (
                    "continue".to_string(),
                    "继续推进".to_string(),
                    projection.next_recommended_action_reason.clone(),
                ),
                Some(projection) if projection.current_state == "adjust" => (
                    "adjust".to_string(),
                    "需要调整".to_string(),
                    projection.next_recommended_action_reason.clone(),
                ),
                Some(projection) if projection.current_state == "pause" => (
                    "pause".to_string(),
                    "已暂停".to_string(),
                    projection.next_recommended_action_reason.clone(),
                ),
                Some(projection) if projection.current_state == "next-stage" => (
                    "next-stage".to_string(),
                    "进入下一阶段".to_string(),
                    projection.next_recommended_action_reason.clone(),
                ),
                _ => (
                    "completion-ready".to_string(),
                    "等待完成判断".to_string(),
                    "任务已全部完成，正在等待 Goal Recheck / Completion Runtime 做最后判断。"
                        .to_string(),
                ),
            }
        }
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
            Some("in_review") => format!(
                "{issue_id} 已完成本地验证，当前正在等待 PR/MR 合并、Issue 关闭和 Done 写回。"
            ),
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
    let (next_action, next_action_label, next_action_reason) =
        if let Some(issue_id) = current_issue_id.clone() {
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
    let completion_hint = if let Some(release) = project_release.as_ref() {
        append_audit_hint(
            append_delivery_hint(release.summary_line.clone(), project_delivery.as_ref()),
            project_audit.as_ref(),
        )
    } else if let Some(completion) = completion_projection.as_ref() {
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
            task_evidence_ready_count: projection.task_evidence_ready_count,
            task_evidence_missing_count: projection.task_evidence_missing_count,
            delivery_status: projection.delivery_status,
            delivery_missing_count: projection.delivery_missing_count,
            audit_required: projection.audit_required,
            audit_status: projection.audit_status,
            audit_blocking_findings: projection.audit_blocking_findings,
            goal_recheck_status: projection.goal_recheck_status,
            project_health_status: projection.project_health_status,
            release_readiness: projection.release_readiness,
            open_questions: projection.open_questions,
            rationale: projection.rationale,
            updated_at: projection.updated_at,
        }),
        release: project_release.map(|release| ProjectReleaseProjection {
            current_state: release.current_state,
            publication_stage: release.publication_stage,
            gate_status: release.gate_status,
            gate_reason: release.gate_reason,
            completion_state: release.completion_state,
            completion_outcome: release.completion_outcome,
            delivery_status: release.delivery_status,
            public_record_written_at: release.public_record_written_at,
            changelog_path: release.changelog_path,
            release_notes_path: release.release_notes_path,
            entry_count: release.entry_count,
            summary_line: release.summary_line,
            tag_name: release.tag_name,
            tag_commit_sha: release.tag_commit_sha,
            tag_proof_path: release.tag_proof_path,
            remote_provider: release.remote_provider,
            remote_release_id: release.remote_release_id,
            remote_release_url: release.remote_release_url,
            remote_release_commit_sha: release.remote_release_commit_sha,
            remote_release_proof_path: release.remote_release_proof_path,
            artifact_manifest_path: release.artifact_manifest_path,
            artifact_manifest_sha256: release.artifact_manifest_sha256,
            published_at: release.published_at,
            updated_at: release.updated_at,
        }),
        external_review: project_external_review.map(|review| ProjectExternalReviewProjection {
            review_status: review.review_status,
            handoff_path: review.handoff_path,
            total_entries: review.total_entries,
            summary_line: review.summary_line,
            latest_audit_status: review
                .audit_summary
                .as_ref()
                .and_then(|summary| summary.latest_status.clone()),
            findings_count: review
                .audit_summary
                .as_ref()
                .map(|summary| summary.findings_count)
                .unwrap_or(0),
            risk_items: review.risk_items,
            generated_at: review.generated_at,
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

fn normalize_runtime_run_status(run_status: Option<&str>, session_status: Option<&str>) -> String {
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
                if event.payload.get("providerKind").is_some() {
                    summary.provider_kind = event
                        .payload
                        .get("providerKind")
                        .and_then(Value::as_str)
                        .map(str::to_string);
                }
                if event.payload.get("providerStatus").is_some() {
                    summary.provider_status = event
                        .payload
                        .get("providerStatus")
                        .and_then(Value::as_str)
                        .map(str::to_string);
                }
                summary.status = Some("requested".to_string());
                summary.launch_requested_at = Some(event.timestamp);
                summary.updated_at = Some(event.timestamp);
                summary.working_directory = event
                    .payload
                    .get("workingDirectory")
                    .and_then(Value::as_str)
                    .map(str::to_string)
                    .or_else(|| summary.working_directory.clone());
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
                if event.payload.get("providerKind").is_some() {
                    summary.provider_kind = event
                        .payload
                        .get("providerKind")
                        .and_then(Value::as_str)
                        .map(str::to_string);
                }
                if event.payload.get("providerStatus").is_some() {
                    summary.provider_status = event
                        .payload
                        .get("providerStatus")
                        .and_then(Value::as_str)
                        .map(str::to_string);
                }
                if event.payload.get("ownerId").is_some() {
                    summary.owner_id = event
                        .payload
                        .get("ownerId")
                        .and_then(Value::as_str)
                        .map(str::to_string);
                }
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
                summary.attempt_count = event
                    .payload
                    .get("attemptCount")
                    .and_then(Value::as_u64)
                    .map(|value| value as u32)
                    .unwrap_or(summary.attempt_count);
                summary.updated_at = Some(event.timestamp);
                if event.payload.get("workingDirectory").is_some() {
                    summary.working_directory = event
                        .payload
                        .get("workingDirectory")
                        .and_then(Value::as_str)
                        .map(str::to_string);
                }
                if event.payload.get("workspaceRoot").is_some() {
                    summary.workspace_root = event
                        .payload
                        .get("workspaceRoot")
                        .and_then(Value::as_str)
                        .map(str::to_string);
                }
                if event.payload.get("worktreeRoot").is_some() {
                    summary.worktree_root = event
                        .payload
                        .get("worktreeRoot")
                        .and_then(Value::as_str)
                        .map(str::to_string);
                }
                if event.payload.get("runtimeRoot").is_some() {
                    summary.runtime_root = event
                        .payload
                        .get("runtimeRoot")
                        .and_then(Value::as_str)
                        .map(str::to_string);
                }
                if event.payload.get("tempRoot").is_some() {
                    summary.temp_root = event
                        .payload
                        .get("tempRoot")
                        .and_then(Value::as_str)
                        .map(str::to_string);
                }
                if event.payload.get("cacheRoot").is_some() {
                    summary.cache_root = event
                        .payload
                        .get("cacheRoot")
                        .and_then(Value::as_str)
                        .map(str::to_string);
                }
                if event.payload.get("evidenceRoot").is_some() {
                    summary.evidence_root = event
                        .payload
                        .get("evidenceRoot")
                        .and_then(Value::as_str)
                        .map(str::to_string);
                }
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
                if event.payload.get("lastMessagePath").is_some() {
                    summary.last_message_path = event
                        .payload
                        .get("lastMessagePath")
                        .and_then(Value::as_str)
                        .map(str::to_string);
                }
                if event.payload.get("exitProofPath").is_some() {
                    summary.exit_proof_path = event
                        .payload
                        .get("exitProofPath")
                        .and_then(Value::as_str)
                        .map(str::to_string);
                }
                if event.payload.get("mergeProofPath").is_some() {
                    summary.merge_proof_path = event
                        .payload
                        .get("mergeProofPath")
                        .and_then(Value::as_str)
                        .map(str::to_string);
                }
                if event.payload.get("mergeState").is_some() {
                    summary.merge_state = event
                        .payload
                        .get("mergeState")
                        .and_then(Value::as_str)
                        .map(str::to_string);
                }
                if event.payload.get("writebackState").is_some() {
                    summary.writeback_state = event
                        .payload
                        .get("writebackState")
                        .and_then(Value::as_str)
                        .map(str::to_string);
                }
                if event.payload.get("selectionStatus").is_some() {
                    summary.selection_status = event
                        .payload
                        .get("selectionStatus")
                        .and_then(Value::as_str)
                        .map(str::to_string);
                }
                if event.payload.get("selectionReason").is_some() {
                    summary.selection_reason = event
                        .payload
                        .get("selectionReason")
                        .and_then(Value::as_str)
                        .map(str::to_string);
                }
                if event.payload.get("degradationReason").is_some() {
                    summary.degradation_reason = event
                        .payload
                        .get("degradationReason")
                        .and_then(Value::as_str)
                        .map(str::to_string);
                }
                if let Some(Value::Array(items)) = event.payload.get("supportedRoles") {
                    summary.supported_roles = items
                        .iter()
                        .filter_map(Value::as_str)
                        .map(str::to_string)
                        .collect();
                }
                if let Some(Value::Array(items)) = event.payload.get("supportedSkillPacks") {
                    summary.supported_skill_packs = items
                        .iter()
                        .filter_map(Value::as_str)
                        .map(str::to_string)
                        .collect();
                }
                if let Some(Value::Array(items)) = event.payload.get("requiredCapabilities") {
                    summary.required_capabilities = items
                        .iter()
                        .filter_map(Value::as_str)
                        .map(str::to_string)
                        .collect();
                }
                if let Some(Value::Array(items)) = event.payload.get("degradedCapabilities") {
                    summary.degraded_capabilities = items
                        .iter()
                        .filter_map(Value::as_str)
                        .map(str::to_string)
                        .collect();
                }
                if let Some(Value::Array(items)) = event.payload.get("missingRequiredCapabilities")
                {
                    summary.missing_required_capabilities = items
                        .iter()
                        .filter_map(Value::as_str)
                        .map(str::to_string)
                        .collect();
                }
                if let Some(Value::Array(items)) = event.payload.get("missingDegradedCapabilities")
                {
                    summary.missing_degraded_capabilities = items
                        .iter()
                        .filter_map(Value::as_str)
                        .map(str::to_string)
                        .collect();
                }
                if event.payload.get("recoveryReason").is_some() {
                    summary.recovery_reason = event
                        .payload
                        .get("recoveryReason")
                        .and_then(Value::as_str)
                        .map(str::to_string);
                }
                if event.payload.get("lastError").is_some() {
                    summary.last_error = event
                        .payload
                        .get("lastError")
                        .and_then(Value::as_str)
                        .map(str::to_string);
                }
                if event.payload.get("processGroupId").is_some() {
                    summary.process_group_id = event
                        .payload
                        .get("processGroupId")
                        .and_then(Value::as_u64)
                        .map(|value| value as u32);
                }
                if event.payload.get("startedAt").is_some() {
                    summary.started_at = event.payload.get("startedAt").and_then(Value::as_u64);
                }
                if event.payload.get("lastHeartbeatAt").is_some() {
                    summary.last_heartbeat_at =
                        event.payload.get("lastHeartbeatAt").and_then(Value::as_u64);
                }
                if event.payload.get("permissionMode").is_some() {
                    summary.permission_mode = event
                        .payload
                        .get("permissionMode")
                        .and_then(Value::as_str)
                        .map(str::to_string);
                }
                if event.payload.get("approvalPolicy").is_some() {
                    summary.approval_policy = event
                        .payload
                        .get("approvalPolicy")
                        .and_then(Value::as_str)
                        .map(str::to_string);
                }
                if event.payload.get("sandboxMode").is_some() {
                    summary.sandbox_mode = event
                        .payload
                        .get("sandboxMode")
                        .and_then(Value::as_str)
                        .map(str::to_string);
                }
                if event.payload.get("supervisionMode").is_some() {
                    summary.supervision_mode = event
                        .payload
                        .get("supervisionMode")
                        .and_then(Value::as_str)
                        .map(str::to_string);
                }
                if event.payload.get("governancePolicyVersion").is_some() {
                    summary.governance_policy_version = event
                        .payload
                        .get("governancePolicyVersion")
                        .and_then(Value::as_str)
                        .map(str::to_string);
                }
                if event.payload.get("claimPolicy").is_some() {
                    summary.claim_policy = event
                        .payload
                        .get("claimPolicy")
                        .and_then(Value::as_str)
                        .map(str::to_string);
                }
                if event.payload.get("timeoutPolicy").is_some() {
                    summary.timeout_policy = event
                        .payload
                        .get("timeoutPolicy")
                        .and_then(Value::as_str)
                        .map(str::to_string);
                }
                if event.payload.get("timeoutSeconds").is_some() {
                    summary.timeout_seconds =
                        event.payload.get("timeoutSeconds").and_then(Value::as_u64);
                }
                if event.payload.get("timeoutAt").is_some() {
                    summary.timeout_at = event.payload.get("timeoutAt").and_then(Value::as_u64);
                }
                if event.payload.get("timedOutAt").is_some() {
                    summary.timed_out_at = event.payload.get("timedOutAt").and_then(Value::as_u64);
                }
                if event.payload.get("takeoverPolicy").is_some() {
                    summary.takeover_policy = event
                        .payload
                        .get("takeoverPolicy")
                        .and_then(Value::as_str)
                        .map(str::to_string);
                }
                if event.payload.get("retryPolicy").is_some() {
                    summary.retry_policy = event
                        .payload
                        .get("retryPolicy")
                        .and_then(Value::as_str)
                        .map(str::to_string);
                }
                if event.payload.get("maxAttempts").is_some() {
                    summary.max_attempts = event
                        .payload
                        .get("maxAttempts")
                        .and_then(Value::as_u64)
                        .map(|value| value as u32);
                }
                if event.payload.get("cancelPolicy").is_some() {
                    summary.cancel_policy = event
                        .payload
                        .get("cancelPolicy")
                        .and_then(Value::as_str)
                        .map(str::to_string);
                }
                if event.payload.get("cancelRequestedAt").is_some() {
                    summary.cancel_requested_at = event
                        .payload
                        .get("cancelRequestedAt")
                        .and_then(Value::as_u64);
                }
                if event.payload.get("cancelledAt").is_some() {
                    summary.cancelled_at = event.payload.get("cancelledAt").and_then(Value::as_u64);
                }
                if event.payload.get("resumedFromAttempt").is_some() {
                    summary.resumed_from_attempt = event
                        .payload
                        .get("resumedFromAttempt")
                        .and_then(Value::as_u64)
                        .map(|value| value as u32);
                }
                if event.payload.get("takeoverSessionId").is_some() {
                    summary.takeover_session_id = event
                        .payload
                        .get("takeoverSessionId")
                        .and_then(Value::as_str)
                        .map(str::to_string);
                }
                if event.payload.get("terminalReason").is_some() {
                    summary.terminal_reason = event
                        .payload
                        .get("terminalReason")
                        .and_then(Value::as_str)
                        .map(str::to_string);
                }
                if event.payload.get("retryable").is_some() {
                    summary.retryable = event.payload.get("retryable").and_then(Value::as_bool);
                }
                if event.payload.get("exitedAt").is_some() {
                    summary.exited_at = event.payload.get("exitedAt").and_then(Value::as_u64);
                }
                if event.payload.get("exitCode").is_some() {
                    summary.exit_code = event
                        .payload
                        .get("exitCode")
                        .and_then(Value::as_i64)
                        .map(|value| value as i32);
                }
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
        let should_prefer_summary = summary.status == "published"
            || summary.public_record_path.is_some()
            || summary
                .public_record_items
                .iter()
                .any(|item| item != "PR/MR body" && item != "项目级发布记录");
        if should_prefer_summary {
            return ProjectionDeliverySummary {
                status: summary.status,
                evidence_status: summary.evidence_status,
                evidence_path: summary.evidence_path,
                pr_url: summary.pr_url,
                merge_commit: summary.merge_commit,
                public_record_path: summary.public_record_path,
                public_record_targets: summary.public_record_targets,
                public_record_markdown: summary.public_record_markdown,
                summary_line: summary.summary_line,
                public_record_items: summary.public_record_items,
                missing_public_records: summary.missing_public_records,
                current_issue_id: None,
                published_count: 0,
                ready_count: 0,
                missing_count: 0,
            };
        }
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
    let mut public_record_targets = vec!["PR/MR body".to_string()];
    if public_delivery.pr_url.is_some() {
        public_record_items.push("PR/MR body".to_string());
    }
    if let Some(path) = public_delivery.changelog_path.clone() {
        public_record_items.push(path.clone());
        public_record_targets.push(path);
    }
    if let Some(path) = public_delivery.release_notes_url.clone() {
        if !public_record_items.iter().any(|item| item == &path) {
            public_record_items.push(path.clone());
        }
        if !public_record_targets.iter().any(|item| item == &path) {
            public_record_targets.push(path);
        }
    }
    if public_record_targets.len() == 1 && status == "ready" {
        public_record_targets.push("项目级发布记录".to_string());
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
        public_record_targets,
        public_record_markdown: format!(
            "# {} {}\n\n## 公开交付\n\n- 状态：{}\n- 目标位置：{}\n",
            issue.issue_id,
            issue.title,
            match status.as_str() {
                "published" => "已发布",
                "ready" => "待发布",
                _ => "缺失",
            },
            if public_record_items.is_empty() {
                "PR/MR body".to_string()
            } else {
                public_record_items.join("、")
            }
        ),
        summary_line: match status.as_str() {
            "published" => format!("公开交付已统一写入 {}。", public_record_items.join("、")),
            "ready" => "公开交付准备已完成，当前已具备 PR/MR 事实，项目级发布记录由独立流程处理。"
                .to_string(),
            _ => "当前还没有公开交付记录。".to_string(),
        },
        public_record_items,
        missing_public_records: if status == "ready" {
            vec!["项目级发布记录".to_string()]
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
        let should_prefer_summary = summary.status == "published"
            || summary
                .public_record_items
                .iter()
                .any(|item| item != "PR/MR body" && item != "项目级发布记录");
        if should_prefer_summary {
            return Some(ProjectionDeliverySummary {
                status: summary.status,
                evidence_status: "ready".to_string(),
                evidence_path: None,
                pr_url: None,
                merge_commit: None,
                public_record_path: summary.public_record_items.first().cloned(),
                public_record_targets: summary.public_record_items.clone(),
                public_record_markdown: String::new(),
                summary_line: summary.summary_line,
                public_record_items: summary.public_record_items,
                missing_public_records: summary.missing_public_records,
                current_issue_id: summary.current_issue_id,
                published_count: summary.published_count,
                ready_count: summary.ready_count,
                missing_count: summary.missing_count,
            });
        }
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
        public_record_targets: public_record_items.clone(),
        public_record_markdown: String::new(),
        summary_line: if missing_count > 0 {
            "项目仍有任务缺少公开交付记录。".to_string()
        } else if public_record_items.is_empty() {
            "当前项目还没有公开交付记录。".to_string()
        } else {
            format!("项目公开交付已汇总到 {}。", public_record_items.join("、"))
        },
        public_record_items,
        missing_public_records: if missing_count > 0 {
            vec!["项目级发布记录".to_string()]
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

    let audit_result =
        audit.and_then(|entry| load_audit_result_summary(root, entry.audit_id.clone()).ok());

    ProjectionAuditSummary {
        status: audit
            .map(|entry| entry.status.clone())
            .unwrap_or_else(|| "not-requested".to_string()),
        latest_audit_id: audit.map(|entry| entry.audit_id.clone()),
        source_issue_id: audit
            .and_then(|entry| entry.source_issue_id.clone())
            .or_else(|| {
                audit_result
                    .as_ref()
                    .and_then(|summary| summary.source_issue_id.clone())
            }),
        report_path: audit.map(|entry| entry.report_path.clone()).or_else(|| {
            audit_result
                .as_ref()
                .map(|summary| summary.report_path.clone())
        }),
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

fn active_project_issue_priority(state: &str) -> u8 {
    match state {
        "in_progress" => 0,
        "in_review" => 1,
        "todo" => 2,
        "blocked" => 3,
        _ => 4,
    }
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
        | "issue.closeout.proof.recorded"
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
        "agent.launch.requested" => "已生成 Work Agent 启动请求。".to_string(),
        "agent.session.created" => "外部执行会话已创建。".to_string(),
        "agent.session.resumed" => "外部执行会话已恢复。".to_string(),
        "agent.session.running" => "外部执行会话正在运行。".to_string(),
        "agent.session.interrupted" => "外部执行会话已中断，等待恢复。".to_string(),
        "agent.session.in_review" => "外部执行会话已进入评审。".to_string(),
        "agent.session.completed" => "外部执行会话已完成。".to_string(),
        "agent.session.failed" => "外部执行会话失败。".to_string(),
        "agent.session.cancelled" => "外部执行会话已取消。".to_string(),
        "issue.validation.passed" => "本地沙箱验证已通过。".to_string(),
        "issue.review.requested" => "任务已请求评审。".to_string(),
        "issue.pr.created" => "PR/MR 已创建。".to_string(),
        "issue.closeout.proof.recorded" => "收口证明已写入，等待 Done 写回。".to_string(),
        "issue.pr.merged" => "PR/MR 已合并，等待关单与收口证明。".to_string(),
        "issue.acceptance.accepted" => "验收判定已通过。".to_string(),
        "issue.acceptance.rejected" => "验收判定被拒绝，需先修复失败原因。".to_string(),
        "issue.acceptance.human-review-required" => {
            "验收判定需要人工判断，不能伪装成自动通过。".to_string()
        }
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
        "in_review" => "验证已通过，等待 PR/MR 合并、Issue 关闭和 Done 写回。".to_string(),
        "done" => "任务已完成，公开交付与发布由独立流程处理。".to_string(),
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
        read_spec_issue, read_spec_project, requirement_preview_from_requirement, write_spec_issue,
        write_spec_project, CompletionDecisionOutcome, SpecIssueDraft, SpecIssueStatus,
        SpecProjectDraft,
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

    fn write_completion_ready_artifacts(
        root: &Path,
        issue_id: &str,
        run_id: &str,
        public_delivery_written: bool,
    ) {
        let task_root = root.join(".agentflow/tasks").join(issue_id);
        let evidence_dir = task_root.join("evidence");
        let review_dir = task_root.join("runs").join(run_id).join("review");
        fs::create_dir_all(&evidence_dir).unwrap();
        fs::create_dir_all(&review_dir).unwrap();
        fs::write(
            evidence_dir.join("evidence.json"),
            serde_json::to_string_pretty(&json!({
                "version": "task-evidence.v1",
                "issueId": issue_id,
                "runId": run_id,
                "status": "ready",
                "summary": "本地验证通过。",
                "runPath": format!(".agentflow/tasks/{issue_id}/runs/{run_id}/run.json"),
                "commandPaths": [],
                "validationPath": format!(".agentflow/tasks/{issue_id}/runs/{run_id}/validation.json"),
                "createdAt": 1
            }))
            .unwrap(),
        )
        .unwrap();
        fs::write(
            review_dir.join("closeout-proof.json"),
            serde_json::to_string_pretty(&json!({
                "version": "closeout-proof.v1",
                "issueId": issue_id,
                "runId": run_id,
                "merged": true,
                "issueClosed": true,
                "publicDeliveryWritten": public_delivery_written,
                "prUrl": "https://github.com/example/repo/pull/1",
                "mergeCommitSha": "abc123",
                "changelogPath": if public_delivery_written { Some("CHANGELOG.md") } else { None::<&str> },
                "releaseNotesPath": if public_delivery_written { Some("docs/release-notes/test.md") } else { None::<&str> }
            }))
            .unwrap(),
        )
        .unwrap();
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
        write_completion_ready_artifacts(dir.path(), "AF-PROJ-001", "run-001", false);
        append_task_event_once(
            dir.path(),
            event(
                "AF-PROJ-001",
                "issue.completed",
                json!({
                    "runId": "run-001",
                    "prUrl": "https://github.com/example/repo/pull/1",
                    "mergeCommit": "abc123"
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
        assert_eq!(task.public_delivery.changelog_path.as_deref(), None);
        assert_eq!(task.public_delivery.release_notes_url.as_deref(), None);
        assert_eq!(project.completed_issue_count, 1);
        assert_eq!(project.status, "active");
        assert_eq!(project.next_action, "enter-completion-decision");
        assert_eq!(project.next_action_label, "进入完成判断");
        assert_eq!(
            project
                .completion
                .as_ref()
                .map(|completion| completion.current_state.as_str()),
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
        assert_eq!(
            project
                .completion
                .as_ref()
                .map(|completion| completion.current_state.as_str()),
            Some("goal-recheck")
        );
    }

    #[test]
    fn terminal_issue_authority_is_not_regressed_by_stale_runtime_events() {
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
                "issue.review.requested",
                json!({"runId": "run-001"}),
            ),
        )
        .unwrap();
        append_task_event_once(
            dir.path(),
            event(
                "AF-PROJ-001",
                "issue.pr.created",
                json!({"runId": "run-001"}),
            ),
        )
        .unwrap();

        rebuild_projections(dir.path()).unwrap();
        let task = crate::storage::load_task_projection(dir.path(), "AF-PROJ-001").unwrap();
        let project =
            crate::storage::load_project_projection(dir.path(), "project-projection").unwrap();

        assert_eq!(task.current_state, "done");
        assert_eq!(task.display_status, "done");
        assert_eq!(project.completed_issue_count, 1);
        assert_eq!(project.current_issue_id, None);
        assert!(task
            .timeline
            .iter()
            .find(|item| item.state == "todo")
            .is_some_and(|item| !item.events.is_empty()));
        assert!(task
            .timeline
            .iter()
            .find(|item| item.state == "in_progress")
            .is_some_and(|item| !item.events.is_empty()));
        assert!(task
            .timeline
            .iter()
            .find(|item| item.state == "in_review")
            .is_some_and(|item| !item.events.is_empty()));
    }

    #[test]
    fn project_projection_prefers_running_issue_over_todo_for_current_issue() {
        let dir = tempdir().unwrap();
        write_fixture(dir.path());
        let requirement = dir.path().join("docs/requirements/034-test.md");

        let mut second_issue = SpecIssueDraft::new("AF-PROJ-002");
        second_issue.project_id = Some("project-projection".to_string());
        let second_issue =
            agentflow_spec::issue_from_requirement(dir.path(), &requirement, second_issue).unwrap();
        write_spec_issue(dir.path(), &second_issue).unwrap();

        let mut project = read_spec_project(dir.path(), "project-projection").unwrap();
        project.issue_ids = vec!["AF-PROJ-001".to_string(), "AF-PROJ-002".to_string()];
        write_spec_project(dir.path(), &project).unwrap();

        append_task_event_once(
            dir.path(),
            event("AF-PROJ-001", "issue.scheduled", json!({})),
        )
        .unwrap();
        append_task_event_once(
            dir.path(),
            event("AF-PROJ-002", "issue.scheduled", json!({})),
        )
        .unwrap();
        append_task_event_once(
            dir.path(),
            event(
                "AF-PROJ-002",
                "agent.launch.requested",
                json!({
                    "runId": "run-002",
                    "branchName": "agentflow/project-projection/AF-PROJ-002"
                }),
            ),
        )
        .unwrap();

        rebuild_projections(dir.path()).unwrap();
        let project =
            crate::storage::load_project_projection(dir.path(), "project-projection").unwrap();

        assert_eq!(project.current_issue_id.as_deref(), Some("AF-PROJ-002"));
        assert_eq!(project.stage_label, "正在推进");
        assert_eq!(project.next_action_label, "继续当前任务");
    }

    #[test]
    fn pause_completion_decision_blocks_finished_project() {
        let dir = tempdir().unwrap();
        write_fixture(dir.path());
        let mut issue = read_spec_issue(dir.path(), "AF-PROJ-001").unwrap();
        issue.status = SpecIssueStatus::Done;
        write_spec_issue(dir.path(), &issue).unwrap();
        agentflow_spec::sync_completion_decision_runtimes(dir.path()).unwrap();
        agentflow_spec::record_completion_decision(
            dir.path(),
            "project-projection",
            CompletionDecisionOutcome::Pause,
            "goal-agent",
            "项目先暂停，等待后续人工决定。",
            vec!["当前交付已经完成，但暂不进入接受态。".to_string()],
        )
        .unwrap();

        rebuild_projections(dir.path()).unwrap();
        let project =
            crate::storage::load_project_projection(dir.path(), "project-projection").unwrap();

        assert_eq!(project.status, "blocked");
        assert_eq!(project.stage_label, "已暂停");
        assert_eq!(project.next_action_label, "暂停项目");
        assert_eq!(
            project
                .completion
                .as_ref()
                .and_then(|completion| completion.latest_outcome.as_deref()),
            Some("pause")
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
    fn pr_merged_stays_in_review_until_issue_completed() {
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
                "issue.review.requested",
                json!({"runId": "run-001"}),
            ),
        )
        .unwrap();
        append_task_event_once(
            dir.path(),
            event(
                "AF-PROJ-001",
                "issue.pr.created",
                json!({"runId": "run-001"}),
            ),
        )
        .unwrap();
        append_task_event_once(
            dir.path(),
            event(
                "AF-PROJ-001",
                "issue.closeout.proof.recorded",
                json!({"runId": "run-001"}),
            ),
        )
        .unwrap();
        append_task_event_once(
            dir.path(),
            event(
                "AF-PROJ-001",
                "issue.pr.merged",
                json!({"runId": "run-001"}),
            ),
        )
        .unwrap();

        rebuild_projections(dir.path()).unwrap();
        let task = crate::storage::load_task_projection(dir.path(), "AF-PROJ-001").unwrap();
        let project =
            crate::storage::load_project_projection(dir.path(), "project-projection").unwrap();

        assert_eq!(task.current_state, "in_review");
        assert_eq!(task.display_status, "in_review");
        assert!(task
            .timeline
            .iter()
            .find(|item| item.state == "in_review")
            .unwrap()
            .summary
            .contains("Issue 关闭"));
        assert!(project.stage_summary.contains("PR/MR 合并"));
        assert!(project.stage_summary.contains("Done 写回"));
    }

    #[test]
    fn projection_tracks_retry_and_writeback_fields_from_session_events() {
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
                "agent.session.failed",
                json!({
                    "provider": "codex",
                    "providerKind": "codex",
                    "providerStatus": "ready",
                    "ownerId": "work-agent",
                    "runId": "run-001",
                    "sessionId": "codex-run-001",
                    "sessionStatus": "failed",
                    "attemptCount": 1,
                    "startedAt": 10,
                    "lastHeartbeatAt": 20,
                    "workingDirectory": "/repo/.agentflow/tasks/AF-PROJ-001/runs/run-001/runtime/worktree",
                    "workspaceRoot": "/repo",
                    "worktreeRoot": "/repo/.agentflow/tasks/AF-PROJ-001/runs/run-001/runtime/worktree",
                    "selectionStatus": "ready",
                    "selectionReason": "provider codex supports runtime role work-agent and required capabilities are ready",
                    "degradationReason": null,
                    "supportedRoles": ["spec-agent", "work-agent", "audit-agent"],
                    "supportedSkillPacks": ["contract-skills", "execution-skills", "judgment-skills"],
                    "requiredCapabilities": ["launch", "codex.exec", "session.poll", "session.logs", "session.cancel", "build_agent.complete"],
                    "degradedCapabilities": [],
                    "missingRequiredCapabilities": [],
                    "missingDegradedCapabilities": [],
                    "logPath": ".agentflow/state/mcp/sessions/codex-run-001.jsonl",
                    "lastMessagePath": ".agentflow/state/mcp/sessions/codex-run-001-last-message.txt",
                    "exitProofPath": ".agentflow/state/mcp/sessions/codex-run-001-exit.json",
                    "permissionMode": "never",
                    "approvalPolicy": "never",
                    "sandboxMode": "workspace-write",
                    "supervisionMode": "local-process-watch",
                    "governancePolicyVersion": "agentflow-mcp-session-policy.v1",
                    "claimPolicy": "single-active-session-per-run",
                    "timeoutPolicy": "interrupt-and-recover",
                    "timeoutSeconds": 3600,
                    "timeoutAt": 100,
                    "takeoverPolicy": "resume-interrupted-or-failed-attempt",
                    "retryPolicy": "bounded-retry",
                    "maxAttempts": 3,
                    "cancelPolicy": "terminal-for-current-run",
                    "retryable": true,
                    "lastError": "first attempt failed"
                }),
            ),
        )
        .unwrap();
        append_task_event_once(
            dir.path(),
            event(
                "AF-PROJ-001",
                "agent.session.resumed",
                json!({
                    "provider": "codex",
                    "ownerId": "work-agent",
                    "runId": "run-001",
                    "sessionId": "codex-run-001",
                    "sessionStatus": "running",
                    "attemptCount": 2,
                    "startedAt": 10,
                    "lastHeartbeatAt": 30,
                    "workingDirectory": "/repo/.agentflow/tasks/AF-PROJ-001/runs/run-001/runtime/worktree",
                    "workspaceRoot": "/repo",
                    "worktreeRoot": "/repo/.agentflow/tasks/AF-PROJ-001/runs/run-001/runtime/worktree",
                    "logPath": ".agentflow/state/mcp/sessions/codex-run-001-attempt-2.jsonl",
                    "lastMessagePath": ".agentflow/state/mcp/sessions/codex-run-001-attempt-2-last-message.txt",
                    "exitProofPath": ".agentflow/state/mcp/sessions/codex-run-001-attempt-2-exit.json",
                    "recoveryReason": "retry after failed session",
                    "resumedFromAttempt": 1,
                    "takeoverSessionId": "codex-run-001",
                    "retryable": true
                }),
            ),
        )
        .unwrap();
        append_task_event_once(
            dir.path(),
            event(
                "AF-PROJ-001",
                "agent.session.in_review",
                json!({
                    "provider": "codex",
                    "ownerId": "work-agent",
                    "runId": "run-001",
                    "sessionId": "codex-run-001",
                    "sessionStatus": "in-review",
                    "attemptCount": 2,
                    "startedAt": 10,
                    "lastHeartbeatAt": 40,
                    "workingDirectory": "/repo/.agentflow/tasks/AF-PROJ-001/runs/run-001/runtime/worktree",
                    "workspaceRoot": "/repo",
                    "worktreeRoot": "/repo/.agentflow/tasks/AF-PROJ-001/runs/run-001/runtime/worktree",
                    "runtimeRoot": "/repo/.agentflow/tasks/AF-PROJ-001/runs/run-001/runtime",
                    "tempRoot": "/repo/.agentflow/tasks/AF-PROJ-001/runs/run-001/runtime/tmp",
                    "cacheRoot": "/repo/.agentflow/tasks/AF-PROJ-001/runs/run-001/runtime/cache",
                    "evidenceRoot": "/repo/.agentflow/tasks/AF-PROJ-001/runs/run-001/runtime/evidence",
                    "logPath": ".agentflow/state/mcp/sessions/codex-run-001-attempt-2.jsonl",
                    "exitProofPath": ".agentflow/state/mcp/sessions/codex-run-001-attempt-2-exit.json",
                    "mergeProofPath": ".agentflow/tasks/AF-PROJ-001/runs/run-001/review/closeout-proof.json",
                    "mergeState": "awaiting-closeout",
                    "writebackState": "awaiting-complete",
                    "processGroupId": 4321,
                    "permissionMode": "never",
                    "approvalPolicy": "never",
                    "sandboxMode": "workspace-write",
                    "supervisionMode": "local-process-watch",
                    "retryable": true,
                    "lastError": null
                }),
            ),
        )
        .unwrap();

        rebuild_projections(dir.path()).unwrap();
        let projection = crate::storage::load_task_projection(dir.path(), "AF-PROJ-001").unwrap();

        assert_eq!(projection.session.status.as_deref(), Some("in-review"));
        assert_eq!(projection.session.provider_kind.as_deref(), Some("codex"));
        assert_eq!(projection.session.provider_status.as_deref(), Some("ready"));
        assert_eq!(projection.session.owner_id.as_deref(), Some("work-agent"));
        assert_eq!(projection.session.attempt_count, 2);
        assert_eq!(projection.session.started_at, Some(10));
        assert_eq!(projection.session.last_heartbeat_at, Some(40));
        assert_eq!(
            projection.session.runtime_root.as_deref(),
            Some("/repo/.agentflow/tasks/AF-PROJ-001/runs/run-001/runtime")
        );
        assert_eq!(
            projection.session.temp_root.as_deref(),
            Some("/repo/.agentflow/tasks/AF-PROJ-001/runs/run-001/runtime/tmp")
        );
        assert_eq!(
            projection.session.cache_root.as_deref(),
            Some("/repo/.agentflow/tasks/AF-PROJ-001/runs/run-001/runtime/cache")
        );
        assert_eq!(
            projection.session.evidence_root.as_deref(),
            Some("/repo/.agentflow/tasks/AF-PROJ-001/runs/run-001/runtime/evidence")
        );
        assert_eq!(projection.session.process_group_id, Some(4321));
        assert_eq!(
            projection.session.selection_status.as_deref(),
            Some("ready")
        );
        assert_eq!(
            projection.session.selection_reason.as_deref(),
            Some("provider codex supports runtime role work-agent and required capabilities are ready")
        );
        assert_eq!(projection.session.degradation_reason, None);
        assert_eq!(
            projection.session.required_capabilities,
            vec![
                "launch".to_string(),
                "codex.exec".to_string(),
                "session.poll".to_string(),
                "session.logs".to_string(),
                "session.cancel".to_string(),
                "build_agent.complete".to_string(),
            ]
        );
        assert!(projection.session.missing_degraded_capabilities.is_empty());
        assert_eq!(
            projection.session.working_directory.as_deref(),
            Some("/repo/.agentflow/tasks/AF-PROJ-001/runs/run-001/runtime/worktree")
        );
        assert_eq!(projection.session.workspace_root.as_deref(), Some("/repo"));
        assert_eq!(
            projection.session.worktree_root.as_deref(),
            Some("/repo/.agentflow/tasks/AF-PROJ-001/runs/run-001/runtime/worktree")
        );
        assert_eq!(
            projection.session.recovery_reason.as_deref(),
            Some("retry after failed session")
        );
        assert_eq!(
            projection.session.log_path.as_deref(),
            Some(".agentflow/state/mcp/sessions/codex-run-001-attempt-2.jsonl")
        );
        assert_eq!(
            projection.session.last_message_path.as_deref(),
            Some(".agentflow/state/mcp/sessions/codex-run-001-attempt-2-last-message.txt")
        );
        assert_eq!(
            projection.session.exit_proof_path.as_deref(),
            Some(".agentflow/state/mcp/sessions/codex-run-001-attempt-2-exit.json")
        );
        assert_eq!(
            projection.session.merge_proof_path.as_deref(),
            Some(".agentflow/tasks/AF-PROJ-001/runs/run-001/review/closeout-proof.json")
        );
        assert_eq!(
            projection.session.merge_state.as_deref(),
            Some("awaiting-closeout")
        );
        assert_eq!(
            projection.session.writeback_state.as_deref(),
            Some("awaiting-complete")
        );
        assert_eq!(
            projection.session.governance_policy_version.as_deref(),
            Some("agentflow-mcp-session-policy.v1")
        );
        assert_eq!(
            projection.session.claim_policy.as_deref(),
            Some("single-active-session-per-run")
        );
        assert_eq!(
            projection.session.timeout_policy.as_deref(),
            Some("interrupt-and-recover")
        );
        assert_eq!(projection.session.timeout_seconds, Some(3600));
        assert_eq!(projection.session.timeout_at, Some(100));
        assert_eq!(
            projection.session.takeover_policy.as_deref(),
            Some("resume-interrupted-or-failed-attempt")
        );
        assert_eq!(
            projection.session.retry_policy.as_deref(),
            Some("bounded-retry")
        );
        assert_eq!(projection.session.max_attempts, Some(3));
        assert_eq!(
            projection.session.cancel_policy.as_deref(),
            Some("terminal-for-current-run")
        );
        assert_eq!(projection.session.permission_mode.as_deref(), Some("never"));
        assert_eq!(projection.session.approval_policy.as_deref(), Some("never"));
        assert_eq!(
            projection.session.sandbox_mode.as_deref(),
            Some("workspace-write")
        );
        assert_eq!(
            projection.session.supervision_mode.as_deref(),
            Some("local-process-watch")
        );
        assert_eq!(projection.session.resumed_from_attempt, Some(1));
        assert_eq!(
            projection.session.takeover_session_id.as_deref(),
            Some("codex-run-001")
        );
        assert_eq!(projection.session.retryable, Some(true));
        assert_eq!(projection.session.last_error, None);
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
            crate::storage::load_requirement_preview_projection(dir.path(), "040-preview").unwrap();
        let index = crate::storage::load_requirement_preview_index(dir.path()).unwrap();

        assert_eq!(projection.current_state, "goal_draft");
        assert_eq!(projection.lifecycle, "active");
        assert_eq!(
            projection.next_recommended_action,
            "confirm-goal-draft-preview"
        );
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
        write_completion_ready_artifacts(dir.path(), "AF-PROJ-001", "run-001", false);
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
