use crate::commands::{build_core_arbitration_context, RuntimeCommandRequest};
use crate::handoff::WorkCommandHandoff;
use crate::mapping::map_command_to_action_proposal;
use agentflow_action_contract::{
    validate_action_proposal, ActionProposal, ActionProposalValidationReport, ActionRef,
    ActionSourceSurface,
};
use agentflow_object_state::TransitionDecision;
use agentflow_role_policy::RoleCapabilityDecision;
use agentflow_spec::SpecIssue;
use agentflow_task_artifacts::task_work_action_proposals_path;
use agentflow_workflow_core::canonicalize_project_root;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    fs,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

pub const WORK_ACTION_PROPOSAL_CONTRACT_VERSION: &str = "work-action-proposal-contract.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkProposalStageAction {
    StartWork,
    CreateRun,
    WritePatch,
    RunValidation,
    PrepareDelivery,
    MarkDone,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkProposalBinding {
    pub target_object_type: String,
    pub target_object_id: String,
    pub action_contract_id: String,
    pub action_contract_version: String,
    pub role_policy_version: String,
    pub object_state_version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required_state: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_state: Option<String>,
    #[serde(default)]
    pub required_evidence_types: Vec<String>,
    #[serde(default)]
    pub expected_events: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkProposalReadiness {
    pub proposal_valid: bool,
    pub role_allowed: bool,
    pub transition_allowed: bool,
    pub consumable_by_arbitration: bool,
    #[serde(default)]
    pub rejection_reasons: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkActionProposalEntry {
    pub sequence: usize,
    pub stage_action: WorkProposalStageAction,
    pub command: RuntimeCommandRequest,
    pub proposal: ActionProposal,
    pub binding: WorkProposalBinding,
    pub proposal_validation: ActionProposalValidationReport,
    pub role_decision: RoleCapabilityDecision,
    pub transition_decision: TransitionDecision,
    #[serde(default)]
    pub expected_evidence_refs: Vec<String>,
    #[serde(default)]
    pub expected_artifact_refs: Vec<String>,
    pub readiness: WorkProposalReadiness,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkActionProposalContract {
    pub version: String,
    pub issue_id: String,
    pub run_id: String,
    pub workflow_ref: String,
    pub issue_path: String,
    pub work_command_path: String,
    pub contract_path: String,
    pub proposals: Vec<WorkActionProposalEntry>,
}

pub fn write_work_action_proposals_from_spec_issue(
    project_root: impl AsRef<Path>,
    issue: &SpecIssue,
    run_id: &str,
    work_command: &WorkCommandHandoff,
) -> Result<WorkActionProposalContract> {
    let root = canonicalize_project_root(project_root)?;
    let context = build_core_arbitration_context()?;
    let contract_path = task_work_action_proposals_path(&root, &issue.issue_id, run_id)?;
    let contract = build_work_action_proposal_contract(&context, issue, run_id, work_command)?;
    if let Some(parent) = contract_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let mut stored = contract;
    stored.contract_path = normalize_relative_to_root(&root, &contract_path)?;
    fs::write(
        &contract_path,
        serde_json::to_string_pretty(&stored)? + "\n",
    )
    .with_context(|| format!("write {}", contract_path.display()))?;
    Ok(stored)
}

fn build_work_action_proposal_contract(
    context: &agentflow_action_arbitration::ArbitrationContext,
    issue: &SpecIssue,
    run_id: &str,
    work_command: &WorkCommandHandoff,
) -> Result<WorkActionProposalContract> {
    let changed_files_path = format!(
        ".agentflow/tasks/{}/runs/{run_id}/changed-files.json",
        issue.issue_id
    );
    let public_delivery_refs = public_delivery_refs(issue);
    let evidence_path = issue.expected_outputs.evidence_path.clone();
    let workflow_ref = issue.workflow_ref.clone();
    let proposals = vec![
        build_entry(
            context,
            issue,
            WorkProposalStageAction::StartWork,
            command_request(
                format!("claim-issue-{}-{run_id}", issue.issue_id),
                "claimIssue",
                "Issue",
                &issue.issue_id,
                json!({ "claimReason": format!("Start work for {}", issue.issue_id) }),
                vec![],
                vec![issue.system.path.clone()],
                run_id,
            ),
            Some("todo"),
            vec![],
            vec![],
        )?,
        build_entry(
            context,
            issue,
            WorkProposalStageAction::CreateRun,
            command_request(
                work_command.command_id.clone(),
                "startRun",
                "Issue",
                &issue.issue_id,
                json!({ "runId": run_id }),
                vec![],
                vec![work_command.command_path.clone()],
                run_id,
            ),
            Some("todo"),
            vec![],
            vec![issue
                .expected_outputs
                .task_run_dir
                .replace("<run-id>", run_id)],
        )?,
        build_entry(
            context,
            issue,
            WorkProposalStageAction::WritePatch,
            command_request(
                format!("write-patch-{}-{run_id}", issue.issue_id),
                "writePatch",
                "Run",
                run_id,
                json!({ "artifactSummary": format!("Patch summary for {}", issue.issue_id) }),
                vec![changed_files_path.clone()],
                vec![changed_files_path.clone()],
                run_id,
            ),
            Some("running"),
            vec![],
            vec![changed_files_path.clone()],
        )?,
        build_entry(
            context,
            issue,
            WorkProposalStageAction::RunValidation,
            command_request(
                format!("run-validation-{}-{run_id}", issue.issue_id),
                "runValidation",
                "Run",
                run_id,
                json!({ "evidenceSummary": format!("Validation summary for {}", issue.issue_id) }),
                vec![evidence_path.clone()],
                vec![evidence_path.clone()],
                run_id,
            ),
            Some("running"),
            vec![evidence_path.clone()],
            vec![],
        )?,
        build_entry(
            context,
            issue,
            WorkProposalStageAction::PrepareDelivery,
            command_request(
                format!("prepare-delivery-{}-{run_id}", issue.issue_id),
                "prepareDelivery",
                "Run",
                run_id,
                json!({ "artifactSummary": format!("Delivery package for {}", issue.issue_id) }),
                public_delivery_refs.clone(),
                public_delivery_refs.clone(),
                run_id,
            ),
            Some("running"),
            vec![],
            public_delivery_refs.clone(),
        )?,
        build_entry(
            context,
            issue,
            WorkProposalStageAction::MarkDone,
            command_request(
                format!("mark-done-{}-{run_id}", issue.issue_id),
                "markIssueDone",
                "Issue",
                &issue.issue_id,
                json!({ "completionSummary": format!("Close out {}", issue.issue_id) }),
                vec![
                    changed_files_path.clone(),
                    evidence_path.clone(),
                    public_delivery_refs
                        .first()
                        .cloned()
                        .unwrap_or_else(|| "public-delivery-record".to_string()),
                ],
                public_delivery_refs.clone(),
                run_id,
            ),
            Some("in_review"),
            vec![evidence_path],
            public_delivery_refs,
        )?,
    ];

    Ok(WorkActionProposalContract {
        version: WORK_ACTION_PROPOSAL_CONTRACT_VERSION.to_string(),
        issue_id: issue.issue_id.clone(),
        run_id: run_id.to_string(),
        workflow_ref,
        issue_path: issue.system.path.clone(),
        work_command_path: work_command.command_path.clone(),
        contract_path: String::new(),
        proposals,
    })
}

fn build_entry(
    context: &agentflow_action_arbitration::ArbitrationContext,
    issue: &SpecIssue,
    stage_action: WorkProposalStageAction,
    command: RuntimeCommandRequest,
    current_state: Option<&str>,
    expected_evidence_refs: Vec<String>,
    expected_artifact_refs: Vec<String>,
) -> Result<WorkActionProposalEntry> {
    let proposal = map_command_to_action_proposal(&command)?;
    let proposal_validation = validate_action_proposal(
        &proposal,
        &context.action_contract_registry,
        &context.ontology_registry,
    );
    let target_ref = command.target_object_ref.clone().unwrap_or(ActionRef {
        object_type: "Issue".to_string(),
        id: issue.issue_id.clone(),
    });
    let target_object_type = target_ref.object_type.as_str();
    let role_decision = context.role_policy_registry.can_role_propose_action(
        &proposal.actor_role,
        &proposal.action_type,
        Some(target_object_type),
    );
    let transition_decision = context.state_machine_registry.is_transition_defined(
        target_object_type,
        current_state,
        &proposal.action_type,
    );
    let action_contract = context
        .action_contract_registry
        .get_action_contract(&proposal.action_type, &proposal.contract_version)
        .with_context(|| format!("missing action contract {}", proposal.action_type))?;
    let readiness = build_readiness(&proposal_validation, &role_decision, &transition_decision);
    Ok(WorkActionProposalEntry {
        sequence: sequence_for_stage(&stage_action),
        stage_action,
        command,
        proposal: proposal.clone(),
        binding: WorkProposalBinding {
            target_object_type: target_object_type.to_string(),
            target_object_id: target_ref.id,
            action_contract_id: action_contract.id.clone(),
            action_contract_version: action_contract.version.clone(),
            role_policy_version: context
                .role_policy_registry
                .bundle()
                .definition_version
                .clone(),
            object_state_version: context
                .state_machine_registry
                .bundle()
                .definition_version
                .clone(),
            required_state: transition_decision.resolved_state.clone(),
            next_state: transition_decision.next_state.clone(),
            required_evidence_types: action_contract
                .required_evidence
                .iter()
                .map(|item| item.evidence_type.clone())
                .collect(),
            expected_events: action_contract
                .expected_events
                .iter()
                .map(|item| item.event_type.clone())
                .collect(),
        },
        proposal_validation,
        role_decision,
        transition_decision,
        expected_evidence_refs,
        expected_artifact_refs,
        readiness,
    })
}

fn build_readiness(
    proposal_validation: &ActionProposalValidationReport,
    role_decision: &RoleCapabilityDecision,
    transition_decision: &TransitionDecision,
) -> WorkProposalReadiness {
    let proposal_valid = proposal_validation.errors.is_empty();
    let role_allowed = role_decision.allowed;
    let transition_allowed = transition_decision.allowed;
    let mut rejection_reasons = proposal_validation
        .errors
        .iter()
        .map(|error| error.message.clone())
        .collect::<Vec<_>>();
    if !role_allowed {
        rejection_reasons.push(format!("role policy denied: {}", role_decision.reason));
    }
    if !transition_allowed {
        rejection_reasons.push(format!(
            "object state denied: {}",
            transition_decision.reason
        ));
    }
    WorkProposalReadiness {
        proposal_valid,
        role_allowed,
        transition_allowed,
        consumable_by_arbitration: proposal_valid && role_allowed && transition_allowed,
        rejection_reasons,
    }
}

fn command_request(
    command_id: String,
    command_type: &str,
    object_type: &str,
    object_id: &str,
    input: serde_json::Value,
    evidence_refs: Vec<String>,
    artifact_refs: Vec<String>,
    run_id: &str,
) -> RuntimeCommandRequest {
    RuntimeCommandRequest {
        command_id,
        command_type: command_type.to_string(),
        source_surface: ActionSourceSurface::Agent,
        actor_role: "work-agent".to_string(),
        target_object_ref: Some(ActionRef {
            object_type: object_type.to_string(),
            id: object_id.to_string(),
        }),
        input,
        evidence_refs,
        artifact_refs,
        idempotency_key: format!("agent:work-agent:{command_type}:{object_id}:{run_id}"),
        created_at: unix_timestamp_seconds().to_string(),
    }
}

fn public_delivery_refs(issue: &SpecIssue) -> Vec<String> {
    let mut refs = Vec::new();
    if issue.expected_outputs.public_delivery_record.pr_or_mr_body {
        refs.push("pr-or-mr-body".to_string());
    }
    let record = issue
        .expected_outputs
        .public_delivery_record
        .changelog_or_release_notes
        .trim();
    if !record.is_empty() {
        refs.push(record.to_string());
    }
    refs
}

fn sequence_for_stage(stage: &WorkProposalStageAction) -> usize {
    match stage {
        WorkProposalStageAction::StartWork => 1,
        WorkProposalStageAction::CreateRun => 2,
        WorkProposalStageAction::WritePatch => 3,
        WorkProposalStageAction::RunValidation => 4,
        WorkProposalStageAction::PrepareDelivery => 5,
        WorkProposalStageAction::MarkDone => 6,
    }
}

fn normalize_relative_to_root(root: &Path, path: &Path) -> Result<String> {
    let relative = path
        .strip_prefix(root)
        .with_context(|| format!("{} is outside {}", path.display(), root.display()))?;
    Ok(relative
        .components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/"))
}

fn unix_timestamp_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handoff::write_work_command_handoff_from_spec_issue;
    use agentflow_spec::{issue_from_requirement, write_spec_issue, SpecIssueDraft, SpecPriority};
    use tempfile::tempdir;

    fn write_requirement(root: &Path) {
        let path = root.join("docs/requirements/060-work-proposals.md");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(
            &path,
            "# 测试需求\n\n用于 Work Action Proposal contract。\n",
        )
        .unwrap();
    }

    fn ready_issue(root: &Path, issue_id: &str) -> SpecIssue {
        write_requirement(root);
        let requirement = root.join("docs/requirements/060-work-proposals.md");
        let mut draft = SpecIssueDraft::new(issue_id);
        draft.priority = SpecPriority::P1;
        draft.allowed_paths = vec!["apps/desktop/src/**".to_string()];
        draft.forbidden_paths = vec![".agentflow/**".to_string()];
        draft.validation_commands = vec!["npm --prefix apps/desktop run build".to_string()];
        issue_from_requirement(root, &requirement, draft).unwrap()
    }

    #[test]
    fn writes_work_action_proposal_contract_for_spec_issue() {
        let dir = tempdir().unwrap();
        let issue = ready_issue(dir.path(), "AF-WORK-002");
        write_spec_issue(dir.path(), &issue).unwrap();

        let handoff =
            write_work_command_handoff_from_spec_issue(dir.path(), &issue, "run-001").unwrap();
        let contract =
            write_work_action_proposals_from_spec_issue(dir.path(), &issue, "run-001", &handoff)
                .unwrap();

        assert_eq!(
            contract.contract_path,
            ".agentflow/tasks/AF-WORK-002/runs/run-001/launch/work-action-proposals.json"
        );
        assert_eq!(contract.proposals.len(), 6);
        assert!(contract
            .proposals
            .iter()
            .all(|entry| entry.readiness.consumable_by_arbitration));
        assert_eq!(contract.proposals[0].proposal.action_type, "claimIssue");
        assert_eq!(contract.proposals[1].proposal.action_type, "startRun");
        assert_eq!(contract.proposals[5].proposal.action_type, "markIssueDone");
    }
}
