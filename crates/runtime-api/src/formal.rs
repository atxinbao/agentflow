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
    confirm_goal_draft_preview, confirm_plan_draft_preview, list_spec_issues,
    materialize_spec_from_requirement_preview, read_completion_decision_runtime,
    read_requirement_preview_runtime, record_completion_decision,
    requirement_preview_from_requirement, sync_completion_decision_runtimes,
    CompletionDecisionOutcome, CompletionDecisionRuntime, RequirementPreviewRuntime, SpecIssue,
    SpecProject,
};
use agentflow_task_artifacts::load_task_run;
use anyhow::{bail, Result};
use serde::Serialize;
use std::{collections::BTreeSet, path::Path};

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
    let (project, issues) = materialize_spec_from_requirement_preview(root, requirement_id)?;
    let action_proposal_bridge =
        build_materialization_bridge_records(root, requirement_id, &project, &issues)?;
    let _ = agentflow_projection::rebuild_projections(root)?;
    let accepted_count = action_proposal_bridge
        .iter()
        .filter(|record| record.arbitration.status == RuntimeCommandStatus::Accepted)
        .count();
    let rejected_count = action_proposal_bridge
        .iter()
        .filter(|record| {
            matches!(
                record.arbitration.status,
                RuntimeCommandStatus::Rejected | RuntimeCommandStatus::InvalidCommand
            )
        })
        .count();
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

fn build_materialization_bridge_records(
    root: &Path,
    requirement_id: &str,
    project: &SpecProject,
    issues: &[SpecIssue],
) -> Result<Vec<ActionProposalBridgeRecord>> {
    let mut context = build_materialization_bridge_context(root, requirement_id)?;
    build_materialization_bridge_records_with_context(
        root,
        requirement_id,
        project,
        issues,
        &mut context,
    )
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
    sequence: usize,
    entity_kind: &str,
    entity_id: &str,
    command: RuntimeCommandRequest,
    context: &ArbitrationContext,
) -> Result<ActionProposalBridgeRecord> {
    let proposal = map_command_to_action_proposal(&command)?;
    let binding = build_action_proposal_binding_summary(&proposal, context)?;
    let arbitration = execute_command_via_arbitration_with_context(&command, context)?;
    Ok(ActionProposalBridgeRecord {
        sequence,
        entity_kind: entity_kind.to_string(),
        entity_id: entity_id.to_string(),
        command,
        proposal,
        binding,
        arbitration,
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
        build_core_arbitration_context, build_materialization_bridge_records_with_context,
        project_confirm_goal, project_confirm_plan, project_intake, project_materialize,
        RuntimeCommandStatus, SPEC_ACTION_PROPOSAL_BRIDGE_VERSION,
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
    }

    #[test]
    fn rejected_bridge_records_do_not_write_event_store() {
        let dir = tempdir().unwrap();
        let requirement = write_requirement(dir.path());

        project_intake(dir.path(), &requirement, Some("project-preview")).unwrap();
        project_confirm_goal(dir.path(), "999-task-workflow-test", "goal-agent").unwrap();
        project_confirm_plan(dir.path(), "999-task-workflow-test", "spec-agent").unwrap();

        let (project, issues) = agentflow_spec::materialize_spec_from_requirement_preview(
            dir.path(),
            "999-task-workflow-test",
        )
        .unwrap();
        let event_store_before = snapshot_event_store(dir.path());

        let mut context = build_core_arbitration_context().unwrap();
        let bridge = build_materialization_bridge_records_with_context(
            dir.path(),
            "999-task-workflow-test",
            &project,
            &issues,
            &mut context,
        )
        .unwrap();

        assert_eq!(bridge.len(), 3);
        assert_eq!(bridge[0].arbitration.status, RuntimeCommandStatus::Rejected);
        assert_eq!(bridge[1].arbitration.status, RuntimeCommandStatus::Rejected);
        assert_eq!(bridge[2].arbitration.status, RuntimeCommandStatus::Rejected);
        assert_eq!(snapshot_event_store(dir.path()), event_store_before);
    }
}
