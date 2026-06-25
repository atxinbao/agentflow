//! Transitional formal runtime wrappers.
//!
//! 这些 API 复用现有 spec / release / audit 实现，目的是让 Desktop 与 CLI
//! 的正式入口先统一收口到 runtime-api，而不是继续各自直接依赖底层写实现。
//! 真正的 command -> proposal -> arbitration -> accepted-event 主链路由
//! `commands.rs` 提供，并在后续 issue 继续替换这里的兼容 wrapper。

use crate::commands::{
    build_core_arbitration_context, execute_command_via_arbitration_with_context,
    RuntimeCommandRequest,
};
use crate::mapping::map_command_to_action_proposal;
use crate::responses::{RuntimeCommandResponse, RuntimeCommandStatus};
use agentflow_action_arbitration::{ArbitrationContext, EvidenceFact, StateFact};
use agentflow_action_contract::{ActionProposal, ActionRef, ActionSourceSurface};
use agentflow_audit::{
    request_human_audit, AuditScope, AuditScopeRef, HumanAuditReport, HumanAuditRequestDraft,
};
use agentflow_release::{
    confirm_project_release, prepare_project_release, publish_project_release,
    record_project_release_tag, record_project_remote_release,
};
use agentflow_spec::{
    confirm_goal_draft_preview, confirm_plan_draft_preview,
    draft_materialization_contracts_from_requirement_preview, list_spec_issues,
    materialize_spec_from_requirement_preview, read_completion_decision_runtime,
    read_requirement_preview_runtime, record_completion_decision,
    requirement_preview_from_requirement, sync_completion_decision_runtimes,
    write_requirement_materialization_report, write_requirement_preview_runtime,
    CompletionDecisionOutcome, CompletionDecisionRuntime, MaterializationDecision,
    MaterializationProposalDecision, RequirementPreviewRuntime, SpecIssue,
    SpecMaterializationReport, SpecProject,
};
use agentflow_task_artifacts::load_task_run;
use agentflow_workflow_runtime::load_runtime_command_bundle;
use anyhow::{bail, Result};
use serde::Serialize;
use std::{
    collections::BTreeSet,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

const SPEC_ACTION_PROPOSAL_BRIDGE_VERSION: &str = "agentflow-spec-action-proposal-bridge.v1";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionProposalBindingSummary {
    pub target_object_type: String,
    pub target_object_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_object_type: Option<String>,
    pub actor_role: String,
    pub runtime_role: String,
    pub action_contract_ref: String,
    pub action_contract_version: String,
    pub role_policy_ref: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state_machine_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object_state_precondition: Option<String>,
    #[serde(default)]
    pub contract_preconditions: Vec<String>,
    #[serde(default)]
    pub required_evidence: Vec<String>,
    #[serde(default)]
    pub expected_events: Vec<String>,
    pub handoff_required: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub handoff_rule: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionProposalBridgeRecord {
    pub sequence: usize,
    pub entity_kind: String,
    pub entity_id: String,
    pub command: RuntimeCommandRequest,
    pub proposal: ActionProposal,
    pub binding: ActionProposalBindingSummary,
    pub arbitration: RuntimeCommandResponse,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectMaterializationResult {
    pub project: SpecProject,
    pub issues: Vec<SpecIssue>,
    pub action_proposal_bridge_version: String,
    pub action_proposal_bridge: Vec<ActionProposalBridgeRecord>,
    pub accepted_count: usize,
    pub rejected_count: usize,
}

pub fn project_intake(
    root: &Path,
    requirement_path: &Path,
    project_id: Option<&str>,
) -> Result<RequirementPreviewRuntime> {
    let preview = requirement_preview_from_requirement(root, requirement_path, project_id)?;
    let _ = agentflow_projection::rebuild_projections(root)?;
    Ok(preview)
}

pub fn project_preview_goal(
    root: &Path,
    requirement_id: &str,
) -> Result<RequirementPreviewRuntime> {
    read_requirement_preview_runtime(root, requirement_id)
}

pub fn project_confirm_goal(
    root: &Path,
    requirement_id: &str,
    actor: &str,
) -> Result<RequirementPreviewRuntime> {
    let preview = confirm_goal_draft_preview(root, requirement_id, actor)?;
    let _ = agentflow_projection::rebuild_projections(root)?;
    Ok(preview)
}

pub fn project_confirm_plan(
    root: &Path,
    requirement_id: &str,
    actor: &str,
) -> Result<RequirementPreviewRuntime> {
    let preview = confirm_plan_draft_preview(root, requirement_id, actor)?;
    let _ = agentflow_projection::rebuild_projections(root)?;
    Ok(preview)
}

pub fn project_materialize(
    root: &Path,
    requirement_id: &str,
) -> Result<ProjectMaterializationResult> {
    let mut context = build_materialization_bridge_context(root, requirement_id)?;
    let result = project_materialize_with_context(root, requirement_id, &mut context)?;
    let _ = agentflow_projection::rebuild_projections(root)?;
    Ok(result)
}

fn project_materialize_with_context(
    root: &Path,
    requirement_id: &str,
    context: &mut ArbitrationContext,
) -> Result<ProjectMaterializationResult> {
    let (mut preview, draft_project, draft_issues) =
        draft_materialization_contracts_from_requirement_preview(root, requirement_id)?;
    let action_proposal_bridge = build_materialization_bridge_records_with_context(
        root,
        requirement_id,
        &draft_project,
        &draft_issues,
        context,
    )?;
    let accepted_count = action_proposal_bridge
        .iter()
        .filter(|record| record.arbitration.status == RuntimeCommandStatus::Accepted)
        .count();
    let rejected_count = action_proposal_bridge.len().saturating_sub(accepted_count);
    let materialization_report = build_materialization_report(
        &preview,
        &action_proposal_bridge,
        accepted_count,
        rejected_count,
    );
    write_requirement_materialization_report(root, &materialization_report)?;

    if rejected_count > 0 {
        let first_rejection = action_proposal_bridge
            .iter()
            .find(|record| record.arbitration.status != RuntimeCommandStatus::Accepted)
            .map(format_rejection_reason)
            .unwrap_or_else(|| "存在未通过的 runtime action proposal。".to_string());
        preview.next_recommended_action = "review-materialization-rejection".to_string();
        preview.next_recommended_action_label = "处理物化拒绝".to_string();
        preview.next_recommended_action_reason =
            format!("当前 formal materialization 被仲裁拒绝：{first_rejection}");
        preview.updated_at = unix_timestamp_seconds();
        write_requirement_preview_runtime(root, &preview)?;
        return Ok(ProjectMaterializationResult {
            project: draft_project,
            issues: draft_issues,
            action_proposal_bridge_version: SPEC_ACTION_PROPOSAL_BRIDGE_VERSION.to_string(),
            action_proposal_bridge,
            accepted_count,
            rejected_count,
        });
    }

    let (project, issues) = materialize_spec_from_requirement_preview(root, requirement_id)?;
    Ok(ProjectMaterializationResult {
        project,
        issues,
        action_proposal_bridge_version: SPEC_ACTION_PROPOSAL_BRIDGE_VERSION.to_string(),
        action_proposal_bridge,
        accepted_count,
        rejected_count,
    })
}

pub fn audit_request_human(
    root: &Path,
    run_id: &str,
    issue_id: Option<&str>,
    reason: &str,
    public_delivery_path: &str,
) -> Result<HumanAuditReport> {
    let issue_id = resolve_issue_id_for_run(root, run_id, issue_id)?;
    let draft = HumanAuditRequestDraft {
        reason: reason.trim().to_string(),
        scope: AuditScope {
            description: format!("Audit workflow delivery for {issue_id} / {run_id}."),
            refs: vec![
                AuditScopeRef {
                    kind: "issue".to_string(),
                    id: issue_id.clone(),
                    path: format!(".agentflow/spec/issues/{issue_id}.json"),
                },
                AuditScopeRef {
                    kind: "task-run".to_string(),
                    id: run_id.to_string(),
                    path: format!(".agentflow/tasks/{issue_id}/runs/{run_id}/run.json"),
                },
                AuditScopeRef {
                    kind: "evidence".to_string(),
                    id: run_id.to_string(),
                    path: format!(".agentflow/tasks/{issue_id}/evidence/evidence.json"),
                },
                AuditScopeRef {
                    kind: "public-delivery".to_string(),
                    id: public_delivery_path.trim().to_string(),
                    path: public_delivery_path.trim().to_string(),
                },
            ],
        },
    };
    request_human_audit(root, draft)
}

pub fn completion_inspect(root: &Path, project_id: &str) -> Result<CompletionDecisionRuntime> {
    let _ = sync_completion_decision_runtimes(root)?;
    read_completion_decision_runtime(root, project_id)
}

pub fn completion_decide(
    root: &Path,
    project_id: &str,
    outcome: &str,
    actor: &str,
    summary: &str,
    rationale: Vec<String>,
) -> Result<CompletionDecisionRuntime> {
    let outcome = parse_completion_outcome(outcome)?;
    let runtime = record_completion_decision(root, project_id, outcome, actor, summary, rationale)?;
    let _ = agentflow_projection::rebuild_projections(root)?;
    Ok(runtime)
}

pub fn release_prepare(
    root: &Path,
    project_id: &str,
) -> Result<agentflow_release::ProjectReleaseFacts> {
    let facts = prepare_project_release(root, project_id)?;
    let _ = agentflow_projection::rebuild_projections(root)?;
    Ok(facts)
}

pub fn release_confirm(
    root: &Path,
    project_id: &str,
) -> Result<agentflow_release::ProjectReleaseFacts> {
    let facts = confirm_project_release(root, project_id)?;
    let _ = agentflow_projection::rebuild_projections(root)?;
    Ok(facts)
}

pub fn release_publish(
    root: &Path,
    project_id: &str,
) -> Result<agentflow_release::ProjectReleaseFacts> {
    let facts = publish_project_release(root, project_id)?;
    let _ = agentflow_projection::rebuild_projections(root)?;
    Ok(facts)
}

pub fn release_record_tag(
    root: &Path,
    project_id: &str,
    tag_name: &str,
    tag_commit_sha: &str,
    actor: &str,
) -> Result<agentflow_release::ProjectReleaseFacts> {
    let facts = record_project_release_tag(root, project_id, tag_name, tag_commit_sha, actor)?;
    let _ = agentflow_projection::rebuild_projections(root)?;
    Ok(facts)
}

pub fn release_record_remote(
    root: &Path,
    project_id: &str,
    provider: &str,
    release_id: &str,
    release_url: &str,
    tag_name: &str,
    release_commit_sha: &str,
    artifact_manifest_path: &str,
    actor: &str,
) -> Result<agentflow_release::ProjectReleaseFacts> {
    let facts = record_project_remote_release(
        root,
        project_id,
        provider,
        release_id,
        release_url,
        tag_name,
        release_commit_sha,
        artifact_manifest_path,
        actor,
    )?;
    let _ = agentflow_projection::rebuild_projections(root)?;
    Ok(facts)
}

pub fn parse_completion_outcome(raw: &str) -> Result<CompletionDecisionOutcome> {
    match raw.trim() {
        "continue" => Ok(CompletionDecisionOutcome::Continue),
        "adjust" => Ok(CompletionDecisionOutcome::Adjust),
        "pause" => Ok(CompletionDecisionOutcome::Pause),
        "accept" => Ok(CompletionDecisionOutcome::Accept),
        "next-stage" => Ok(CompletionDecisionOutcome::NextStage),
        other => bail!("unsupported completion outcome: {other}"),
    }
}

fn build_materialization_bridge_context(
    root: &Path,
    requirement_id: &str,
) -> Result<ArbitrationContext> {
    let preview = read_requirement_preview_runtime(root, requirement_id)?;
    let source_spec_id = preview
        .plan_draft
        .as_ref()
        .map(|draft| draft.plan_draft_id.clone())
        .ok_or_else(|| anyhow::anyhow!("plan draft is missing for materialized preview"))?;
    let mut context = build_core_arbitration_context()?;
    insert_materialization_confirmation_facts(&mut context, requirement_id, &preview);
    context.insert_state(StateFact {
        object_type: "Spec".to_string(),
        object_id: source_spec_id,
        state_id: "approved".to_string(),
    });
    Ok(context)
}

fn build_materialization_bridge_records_with_context(
    root: &Path,
    requirement_id: &str,
    project: &SpecProject,
    issues: &[SpecIssue],
    context: &mut ArbitrationContext,
) -> Result<Vec<ActionProposalBridgeRecord>> {
    let preview = read_requirement_preview_runtime(root, requirement_id)?;
    let preview_revision = preview.revision;
    insert_materialization_confirmation_facts(context, requirement_id, &preview);
    let source_spec_id = preview
        .plan_draft
        .as_ref()
        .map(|draft| draft.plan_draft_id.clone())
        .ok_or_else(|| anyhow::anyhow!("plan draft is missing for materialized preview"))?;

    let mut records = Vec::new();

    let project_command = build_create_project_runtime_command(
        requirement_id,
        preview_revision,
        project,
        &source_spec_id,
    );
    let project_record = build_action_proposal_bridge_record(
        root,
        1,
        "project",
        &project.project_id,
        project_command,
        context,
    )?;
    if project_record.arbitration.status == RuntimeCommandStatus::Accepted {
        context.insert_state(StateFact {
            object_type: "Project".to_string(),
            object_id: project.project_id.clone(),
            state_id: "planned".to_string(),
        });
    }
    records.push(project_record);

    for (index, issue) in issues.iter().enumerate() {
        let issue_command = build_create_issue_runtime_command(
            requirement_id,
            preview_revision,
            project,
            issue,
            index + 1,
        );
        let issue_record = build_action_proposal_bridge_record(
            root,
            index + 2,
            "issue",
            &issue.issue_id,
            issue_command,
            context,
        )?;
        if issue_record.arbitration.status == RuntimeCommandStatus::Accepted {
            context.insert_state(StateFact {
                object_type: "Project".to_string(),
                object_id: project.project_id.clone(),
                state_id: "active".to_string(),
            });
            context.insert_state(StateFact {
                object_type: "Issue".to_string(),
                object_id: issue.issue_id.clone(),
                state_id: "proposed".to_string(),
            });
        }
        records.push(issue_record);
    }

    Ok(records)
}

fn insert_materialization_confirmation_facts(
    context: &mut ArbitrationContext,
    requirement_id: &str,
    preview: &RequirementPreviewRuntime,
) {
    if preview
        .confirmation_records
        .iter()
        .any(|record| record.decision == "confirmed")
    {
        let evidence_ref =
            materialization_confirmation_evidence_ref(requirement_id, preview.revision);
        context.insert_evidence(EvidenceFact {
            evidence_ref,
            evidence_type: "humanConfirmation".to_string(),
        });
    }
}

fn build_action_proposal_bridge_record(
    root: &Path,
    sequence: usize,
    entity_kind: &str,
    entity_id: &str,
    command: RuntimeCommandRequest,
    context: &ArbitrationContext,
) -> Result<ActionProposalBridgeRecord> {
    let proposal = map_command_to_action_proposal(&command)?;
    let binding = build_action_proposal_binding_summary(&proposal, context)?;
    let _ = execute_command_via_arbitration_with_context(root, &command, context)?;
    let stored = load_runtime_command_bundle(root, &command.command_id)?;
    let proposal = stored.proposal.ok_or_else(|| {
        anyhow::anyhow!(
            "missing durable runtime proposal for {}",
            command.command_id
        )
    })?;
    let arbitration = stored.decision.ok_or_else(|| {
        anyhow::anyhow!(
            "missing durable runtime decision for {}",
            command.command_id
        )
    })?;
    Ok(ActionProposalBridgeRecord {
        sequence,
        entity_kind: entity_kind.to_string(),
        entity_id: entity_id.to_string(),
        command,
        proposal: agentflow_action_contract::ActionProposal {
            proposal_id: proposal.proposal_id,
            idempotency_key: stored.command.idempotency_key,
            action_type: proposal.action_type,
            actor_role: proposal.actor_role,
            source_surface: proposal.source_surface,
            target_object_ref: proposal.target_object_ref,
            input: proposal.input,
            evidence_refs: proposal.evidence_refs,
            artifact_refs: proposal.artifact_refs,
            reason: proposal.reason,
            expected_effects: proposal.expected_effects,
            ontology_version: proposal.ontology_version,
            contract_version: proposal.contract_version,
            created_at: proposal.created_at,
        },
        binding,
        arbitration: crate::responses::RuntimeCommandResponse {
            version: crate::responses::RUNTIME_COMMAND_API_VERSION.to_string(),
            command_id: arbitration.command_id,
            proposal_id: arbitration.proposal_id,
            status: match arbitration.status.as_str() {
                "accepted" => RuntimeCommandStatus::Accepted,
                "deferred" => RuntimeCommandStatus::Deferred,
                "human-decision-required" => RuntimeCommandStatus::HumanDecisionRequired,
                "queued" => RuntimeCommandStatus::Queued,
                "superseded" => RuntimeCommandStatus::Superseded,
                "cancelled" => RuntimeCommandStatus::Cancelled,
                "invalid-command" => RuntimeCommandStatus::InvalidCommand,
                _ => RuntimeCommandStatus::Rejected,
            },
            decision: match arbitration.decision.as_str() {
                "accepted" => crate::responses::RuntimeCommandDecision::Accepted,
                "deferred" => crate::responses::RuntimeCommandDecision::Deferred,
                "human-decision-required" => {
                    crate::responses::RuntimeCommandDecision::HumanDecisionRequired
                }
                "queued" => crate::responses::RuntimeCommandDecision::Queued,
                "superseded" => crate::responses::RuntimeCommandDecision::Superseded,
                "cancelled" => crate::responses::RuntimeCommandDecision::Cancelled,
                "invalid-command" => crate::responses::RuntimeCommandDecision::InvalidCommand,
                _ => crate::responses::RuntimeCommandDecision::Rejected,
            },
            accepted_action_id: arbitration.accepted_action_id,
            rejected_reasons: arbitration
                .rejected_reasons
                .into_iter()
                .map(|message| {
                    crate::errors::RuntimeCommandError::new(
                        crate::errors::RuntimeCommandErrorCode::ArbitrationRejected,
                        message,
                        None::<String>,
                    )
                })
                .collect(),
            human_decision_request: arbitration.human_decision_request.map(|question| {
                crate::responses::RuntimeHumanDecisionRequest {
                    question,
                    allowed_responses: Vec::new(),
                    required_evidence_type: "humanConfirmation".to_string(),
                }
            }),
            next_query_hint: arbitration.next_query_hint.map(|hint| {
                crate::mapping::RuntimeQueryHint {
                    view: hint.view,
                    target_id: hint.target_id,
                    reason: hint.reason,
                }
            }),
            governance_admission: arbitration
                .governance_admission
                .and_then(|value| serde_json::from_value(value).ok()),
            correlation_id: arbitration.correlation_id,
        },
    })
}

fn build_action_proposal_binding_summary(
    proposal: &ActionProposal,
    context: &ArbitrationContext,
) -> Result<ActionProposalBindingSummary> {
    let action_type = context
        .action_contract_registry
        .get_action_type(&proposal.action_type)
        .ok_or_else(|| anyhow::anyhow!("unknown action type {}", proposal.action_type))?;
    let contract = context
        .action_contract_registry
        .get_action_contract(&proposal.action_type, &proposal.contract_version)
        .ok_or_else(|| anyhow::anyhow!("missing action contract {}", proposal.action_type))?;
    let runtime_role = context
        .role_policy_registry
        .resolve_runtime_role(&proposal.actor_role)
        .ok_or_else(|| anyhow::anyhow!("unknown actor role {}", proposal.actor_role))?;
    let role_policy = context
        .role_policy_registry
        .get_role_policy(runtime_role)
        .ok_or_else(|| anyhow::anyhow!("missing role policy {}", runtime_role.as_str()))?;
    let capability = role_policy
        .action_capabilities
        .iter()
        .find(|capability| capability.action_type == proposal.action_type);
    let target = proposal.target_object_ref.as_ref().ok_or_else(|| {
        anyhow::anyhow!("proposal {} is missing target object", proposal.proposal_id)
    })?;
    let current_state = context.current_state_for(target);
    let transition = context.state_machine_registry.is_transition_defined(
        &target.object_type,
        current_state,
        &proposal.action_type,
    );
    let state_machine_ref = context
        .state_machine_registry
        .get_state_machine(&target.object_type)
        .map(|machine| format!("{}@{}", machine.state_machine_id, machine.version));

    Ok(ActionProposalBindingSummary {
        target_object_type: target.object_type.clone(),
        target_object_id: target.id.clone(),
        created_object_type: contract.creates_object_type.clone(),
        actor_role: proposal.actor_role.clone(),
        runtime_role: runtime_role.as_str().to_string(),
        action_contract_ref: action_type.contract_ref.clone(),
        action_contract_version: proposal.contract_version.clone(),
        role_policy_ref: format!(
            "{}@{}",
            context.role_policy_registry.bundle().namespace,
            context.role_policy_registry.bundle().definition_version
        ),
        state_machine_ref,
        object_state_precondition: Some(format_object_state_precondition(
            &target.object_type,
            current_state,
            &proposal.action_type,
            &transition,
        )),
        contract_preconditions: contract
            .preconditions
            .iter()
            .map(|precondition| precondition.description.clone())
            .collect(),
        required_evidence: contract
            .required_evidence
            .iter()
            .map(|required| required.evidence_type.clone())
            .collect(),
        expected_events: collect_expected_events(contract, &transition.emitted_events),
        handoff_required: capability.is_some_and(|capability| capability.requires_handoff),
        handoff_rule: capability.and_then(|capability| capability.handoff_rule.clone()),
    })
}

fn format_object_state_precondition(
    target_object_type: &str,
    current_state: Option<&str>,
    action_type: &str,
    transition: &agentflow_object_state::TransitionDecision,
) -> String {
    match (
        transition.allowed,
        transition.next_state.as_deref(),
        current_state,
    ) {
        (true, Some(next_state), Some(current_state)) => {
            format!("{target_object_type}:{current_state} --{action_type}--> {next_state}")
        }
        (true, Some(next_state), None) => {
            format!("{target_object_type}:<initial> --{action_type}--> {next_state}")
        }
        _ => transition.reason.clone(),
    }
}

fn collect_expected_events(
    contract: &agentflow_action_contract::ActionContract,
    transition_events: &[String],
) -> Vec<String> {
    let mut expected = BTreeSet::new();
    for event in &contract.expected_events {
        expected.insert(event.event_type.clone());
    }
    for event in transition_events {
        expected.insert(event.clone());
    }
    expected.into_iter().collect()
}

fn build_create_project_runtime_command(
    requirement_id: &str,
    preview_revision: u32,
    project: &SpecProject,
    source_spec_id: &str,
) -> RuntimeCommandRequest {
    RuntimeCommandRequest {
        command_id: format!("cmd-create-project-{}", project.project_id),
        command_type: "createProject".to_string(),
        source_surface: ActionSourceSurface::Agent,
        actor_role: "spec-agent".to_string(),
        target_object_ref: Some(ActionRef {
            object_type: "Spec".to_string(),
            id: source_spec_id.to_string(),
        }),
        input: serde_json::json!({
            "projectId": project.project_id.clone(),
            "projectTitle": project.title.clone(),
        }),
        evidence_refs: vec![materialization_confirmation_evidence_ref(
            requirement_id,
            preview_revision,
        )],
        artifact_refs: vec![
            project.source_requirement_path.clone(),
            format!(".agentflow/spec/requirements/{requirement_id}/preview.json"),
            project.system.path.clone(),
        ],
        idempotency_key: format!(
            "spec:{requirement_id}:project:{}:createProject:{}",
            project.project_id, project.system.updated_at
        ),
        created_at: project.system.updated_at.to_string(),
    }
}

fn build_create_issue_runtime_command(
    requirement_id: &str,
    preview_revision: u32,
    project: &SpecProject,
    issue: &SpecIssue,
    ordinal: usize,
) -> RuntimeCommandRequest {
    RuntimeCommandRequest {
        command_id: format!("cmd-create-issue-{}", issue.issue_id),
        command_type: "createIssue".to_string(),
        source_surface: ActionSourceSurface::Agent,
        actor_role: "spec-agent".to_string(),
        target_object_ref: Some(ActionRef {
            object_type: "Project".to_string(),
            id: project.project_id.clone(),
        }),
        input: serde_json::json!({
            "issueId": issue.issue_id.clone(),
            "title": issue.title.clone(),
        }),
        evidence_refs: vec![materialization_confirmation_evidence_ref(
            requirement_id,
            preview_revision,
        )],
        artifact_refs: vec![
            issue.source_requirement_path.clone(),
            format!(".agentflow/spec/requirements/{requirement_id}/preview.json"),
            issue.system.path.clone(),
        ],
        idempotency_key: format!(
            "spec:{requirement_id}:issue:{}:createIssue:{ordinal}",
            issue.issue_id
        ),
        created_at: issue.system.updated_at.to_string(),
    }
}

fn materialization_confirmation_evidence_ref(
    requirement_id: &str,
    preview_revision: u32,
) -> String {
    format!("confirmation:{requirement_id}:preview-r{preview_revision}")
}

fn build_materialization_report(
    preview: &RequirementPreviewRuntime,
    action_proposal_bridge: &[ActionProposalBridgeRecord],
    accepted_count: usize,
    rejected_count: usize,
) -> SpecMaterializationReport {
    let decision = if rejected_count == 0 {
        MaterializationDecision::Accepted
    } else {
        MaterializationDecision::Rejected
    };
    let proposal_decisions = action_proposal_bridge
        .iter()
        .map(|record| MaterializationProposalDecision {
            sequence: record.sequence,
            entity_kind: record.entity_kind.clone(),
            entity_id: record.entity_id.clone(),
            command_type: record.command.command_type.clone(),
            proposal_id: record.proposal.proposal_id.clone(),
            status: runtime_command_status_str(&record.arbitration.status).to_string(),
            rejected_reasons: record
                .arbitration
                .rejected_reasons
                .iter()
                .map(|reason| reason.message.clone())
                .collect(),
        })
        .collect();
    SpecMaterializationReport {
        version: agentflow_spec::model::SPEC_MATERIALIZATION_REPORT_VERSION.to_string(),
        requirement_id: preview.requirement_id.clone(),
        project_id: preview.project_id.clone(),
        decision,
        accepted_count,
        rejected_count,
        proposal_decisions,
        updated_at: unix_timestamp_seconds(),
    }
}

fn format_rejection_reason(record: &ActionProposalBridgeRecord) -> String {
    if !record.arbitration.rejected_reasons.is_empty() {
        let reasons = record
            .arbitration
            .rejected_reasons
            .iter()
            .map(|reason| reason.message.as_str())
            .collect::<Vec<_>>()
            .join("; ");
        return format!(
            "{} {} 被拒绝：{}",
            record.entity_kind, record.entity_id, reasons
        );
    }
    if let Some(request) = &record.arbitration.human_decision_request {
        return format!(
            "{} {} 需要人工决策：{}",
            record.entity_kind, record.entity_id, request.question
        );
    }
    format!(
        "{} {} 的 proposal 状态是 {}",
        record.entity_kind,
        record.entity_id,
        runtime_command_status_str(&record.arbitration.status)
    )
}

fn runtime_command_status_str(status: &RuntimeCommandStatus) -> &'static str {
    match status {
        RuntimeCommandStatus::Accepted => "accepted",
        RuntimeCommandStatus::Rejected => "rejected",
        RuntimeCommandStatus::Deferred => "deferred",
        RuntimeCommandStatus::HumanDecisionRequired => "human-decision-required",
        RuntimeCommandStatus::Queued => "queued",
        RuntimeCommandStatus::Superseded => "superseded",
        RuntimeCommandStatus::Cancelled => "cancelled",
        RuntimeCommandStatus::InvalidCommand => "invalid-command",
    }
}

fn unix_timestamp_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn resolve_issue_id_for_run(root: &Path, run_id: &str, issue_id: Option<&str>) -> Result<String> {
    if let Some(issue_id) = issue_id.map(str::trim).filter(|value| !value.is_empty()) {
        load_task_run(root, issue_id, run_id)?;
        return Ok(issue_id.to_string());
    }

    let issues = list_spec_issues(root)?;
    let mut matched = issues
        .into_iter()
        .filter_map(|issue| {
            load_task_run(root, &issue.issue_id, run_id)
                .ok()
                .map(|_| issue.issue_id)
        })
        .collect::<Vec<_>>();
    matched.sort();
    matched.dedup();
    match matched.len() {
        1 => Ok(matched.remove(0)),
        0 => bail!("no task run matched runId {run_id}"),
        _ => bail!("multiple task runs matched runId {run_id}; pass --issue-id explicitly"),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        build_core_arbitration_context, project_confirm_goal, project_confirm_plan, project_intake,
        project_materialize, project_materialize_with_context, RuntimeCommandStatus,
        SPEC_ACTION_PROPOSAL_BRIDGE_VERSION,
    };
    use agentflow_spec::{
        read_requirement_materialization_report, read_requirement_preview_runtime,
        read_spec_loop_stage_artifact, MaterializationDecision, SpecLoopStageStatus,
    };
    use std::{
        fs,
        path::{Path, PathBuf},
    };
    use tempfile::tempdir;

    fn write_requirement(root: &Path) -> PathBuf {
        let path = root.join("docs/requirements/999-task-workflow-test.md");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "# 任务工作流测试\n\n把任务运行状态改成事件驱动。\n").unwrap();
        path
    }

    fn snapshot_event_store(root: &Path) -> Vec<String> {
        let events_root = root.join(".agentflow/events");
        if !events_root.exists() {
            return Vec::new();
        }
        let mut paths = Vec::new();
        let mut stack = vec![events_root];
        while let Some(path) = stack.pop() {
            if path.is_dir() {
                for entry in fs::read_dir(&path).unwrap() {
                    stack.push(entry.unwrap().path());
                }
            } else {
                paths.push(path.strip_prefix(root).unwrap().display().to_string());
            }
        }
        paths.sort();
        paths
    }

    #[test]
    fn project_materialize_bridges_to_runtime_commands_and_arbitration() {
        let dir = tempdir().unwrap();
        let requirement = write_requirement(dir.path());

        project_intake(dir.path(), &requirement, Some("project-preview")).unwrap();
        project_confirm_goal(dir.path(), "999-task-workflow-test", "goal-agent").unwrap();
        project_confirm_plan(dir.path(), "999-task-workflow-test", "spec-agent").unwrap();

        let result = project_materialize(dir.path(), "999-task-workflow-test").unwrap();

        assert_eq!(result.project.project_id, "project-preview");
        assert_eq!(result.issues.len(), 2);
        assert_eq!(
            result.action_proposal_bridge_version,
            SPEC_ACTION_PROPOSAL_BRIDGE_VERSION
        );
        assert_eq!(result.action_proposal_bridge.len(), 3);
        assert_eq!(result.accepted_count, 3);
        assert_eq!(result.rejected_count, 0);

        let project_record = &result.action_proposal_bridge[0];
        assert_eq!(project_record.command.command_type, "createProject");
        assert_eq!(project_record.binding.target_object_type, "Spec");
        assert_eq!(project_record.binding.actor_role, "spec-agent");
        assert_eq!(project_record.binding.runtime_role, "spec-agent");
        assert_eq!(
            project_record.binding.object_state_precondition.as_deref(),
            Some("Spec:approved --createProject--> approved")
        );
        assert!(project_record
            .binding
            .contract_preconditions
            .iter()
            .any(|value| value.contains("Spec must exist")));
        assert_eq!(
            project_record.arbitration.status,
            RuntimeCommandStatus::Accepted
        );

        let first_issue_record = &result.action_proposal_bridge[1];
        assert_eq!(first_issue_record.command.command_type, "createIssue");
        assert_eq!(first_issue_record.binding.target_object_type, "Project");
        assert_eq!(
            first_issue_record.binding.handoff_rule.as_deref(),
            Some("spec-to-work-approved-spec")
        );
        assert_eq!(
            first_issue_record
                .binding
                .object_state_precondition
                .as_deref(),
            Some("Project:planned --createIssue--> active")
        );
        assert_eq!(
            first_issue_record.arbitration.status,
            RuntimeCommandStatus::Accepted
        );

        let second_issue_record = &result.action_proposal_bridge[2];
        assert_eq!(
            second_issue_record
                .binding
                .object_state_precondition
                .as_deref(),
            Some("Project:active --createIssue--> active")
        );
        assert_eq!(
            second_issue_record.arbitration.status,
            RuntimeCommandStatus::Accepted
        );

        let report =
            read_requirement_materialization_report(dir.path(), "999-task-workflow-test").unwrap();
        assert_eq!(report.decision, MaterializationDecision::Accepted);
        assert_eq!(report.accepted_count, 3);
        assert_eq!(report.rejected_count, 0);

        let stage = read_spec_loop_stage_artifact(
            dir.path(),
            "999-task-workflow-test",
            agentflow_spec::SpecLoopStageName::Materialization,
        )
        .unwrap();
        assert_eq!(stage.status, SpecLoopStageStatus::Materialized);
        assert!(stage
            .output_refs
            .iter()
            .any(|path| path.ends_with("materialization-report.json")));
    }

    #[test]
    fn rejected_materialization_writes_report_without_authority() {
        let dir = tempdir().unwrap();
        let requirement = write_requirement(dir.path());

        project_intake(dir.path(), &requirement, Some("project-preview")).unwrap();
        project_confirm_goal(dir.path(), "999-task-workflow-test", "goal-agent").unwrap();
        project_confirm_plan(dir.path(), "999-task-workflow-test", "spec-agent").unwrap();

        let event_store_before = snapshot_event_store(dir.path());

        let mut context = build_core_arbitration_context().unwrap();
        let result =
            project_materialize_with_context(dir.path(), "999-task-workflow-test", &mut context)
                .unwrap();

        assert_eq!(result.action_proposal_bridge.len(), 3);
        assert_eq!(result.accepted_count, 0);
        assert_eq!(result.rejected_count, 3);
        assert!(result
            .action_proposal_bridge
            .iter()
            .all(|record| record.arbitration.status == RuntimeCommandStatus::Rejected));
        assert_eq!(snapshot_event_store(dir.path()), event_store_before);
        assert!(!dir
            .path()
            .join(".agentflow/spec/projects/project-preview.json")
            .exists());
        assert_eq!(
            fs::read_dir(dir.path().join(".agentflow/spec/issues"))
                .unwrap()
                .count(),
            0
        );

        let report =
            read_requirement_materialization_report(dir.path(), "999-task-workflow-test").unwrap();
        assert_eq!(report.decision, MaterializationDecision::Rejected);
        assert_eq!(report.accepted_count, 0);
        assert_eq!(report.rejected_count, 3);

        let preview =
            read_requirement_preview_runtime(dir.path(), "999-task-workflow-test").unwrap();
        assert_eq!(
            preview.lifecycle,
            agentflow_spec::RequirementPreviewLifecycle::Active
        );
        assert_eq!(preview.current_state, "confirmed");
        assert_eq!(
            preview.next_recommended_action,
            "review-materialization-rejection"
        );

        let stage = read_spec_loop_stage_artifact(
            dir.path(),
            "999-task-workflow-test",
            agentflow_spec::SpecLoopStageName::Materialization,
        )
        .unwrap();
        assert_eq!(stage.status, SpecLoopStageStatus::Rejected);
        let payload = stage.payload.unwrap();
        assert_eq!(payload["decision"], "rejected");
    }
}
