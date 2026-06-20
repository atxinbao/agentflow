use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use agentflow_action_arbitration::{
    arbitrate_action, ArbitrationContext, ArbitrationDecisionStatus, ArbitrationRequest,
    DefinitionVersions,
};
use agentflow_action_contract::{core_action_contract_registry, ActionRef, ActionSourceSurface};
use agentflow_object_state::core_object_state_registry;
use agentflow_ontology::core_ontology_registry;
use agentflow_role_policy::core_role_policy_registry;

use crate::errors::{RuntimeCommandError, RuntimeCommandErrorCode};
use crate::mapping::{
    map_command_to_action_proposal, missing_field_error, runtime_query_hint_for_command,
    unsupported_command_error,
};
use crate::responses::{
    RuntimeCommandDecision, RuntimeCommandResponse, RuntimeCommandStatus,
    RuntimeCommandValidationReport, RuntimeHumanDecisionRequest, RUNTIME_COMMAND_API_VERSION,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeCommandRequest {
    pub command_id: String,
    pub command_type: String,
    pub source_surface: ActionSourceSurface,
    pub actor_role: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_object_ref: Option<ActionRef>,
    pub input: Value,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
    #[serde(default)]
    pub artifact_refs: Vec<String>,
    pub idempotency_key: String,
    pub created_at: String,
}

pub fn validate_runtime_command(request: &RuntimeCommandRequest) -> RuntimeCommandValidationReport {
    let mut errors = Vec::new();

    if request.command_id.trim().is_empty() {
        errors.push(missing_field_error(
            "commandId",
            "runtime command requires commandId",
        ));
    }
    if request.command_type.trim().is_empty() {
        errors.push(missing_field_error(
            "commandType",
            "runtime command requires commandType",
        ));
    }
    if request.actor_role.trim().is_empty() {
        errors.push(missing_field_error(
            "actorRole",
            "runtime command requires actorRole",
        ));
    }
    if request.idempotency_key.trim().is_empty() {
        errors.push(missing_field_error(
            "idempotencyKey",
            "runtime command requires idempotencyKey",
        ));
    }
    if request.created_at.trim().is_empty() {
        errors.push(missing_field_error(
            "createdAt",
            "runtime command requires createdAt",
        ));
    }

    let normalized_action_type = match map_command_to_action_proposal(request) {
        Ok(proposal) => Some(proposal.action_type),
        Err(_) => {
            errors.push(unsupported_command_error(request));
            None
        }
    };

    RuntimeCommandValidationReport {
        command_id: request.command_id.clone(),
        command_type: request.command_type.clone(),
        valid: errors.is_empty(),
        errors,
        warnings: Vec::new(),
        normalized_action_type,
    }
}

pub fn execute_command_via_arbitration(
    request: &RuntimeCommandRequest,
) -> Result<RuntimeCommandResponse> {
    let context = build_core_arbitration_context()?;
    execute_command_via_arbitration_with_context(request, &context)
}

pub fn execute_command_via_arbitration_with_context(
    request: &RuntimeCommandRequest,
    context: &ArbitrationContext,
) -> Result<RuntimeCommandResponse> {
    let validation = validate_runtime_command(request);
    let proposal_id = format!("proposal-{}", request.command_id);
    let next_query_hint = Some(runtime_query_hint_for_command(request));

    if !validation.valid {
        return Ok(RuntimeCommandResponse {
            version: RUNTIME_COMMAND_API_VERSION.to_string(),
            command_id: request.command_id.clone(),
            proposal_id,
            status: RuntimeCommandStatus::InvalidCommand,
            decision: RuntimeCommandDecision::InvalidCommand,
            accepted_action_id: None,
            rejected_reasons: validation.errors,
            human_decision_request: None,
            next_query_hint,
            correlation_id: request.command_id.clone(),
        });
    }

    let proposal = map_command_to_action_proposal(request)?;
    let arbitration_request = ArbitrationRequest {
        request_id: request.command_id.clone(),
        proposal: proposal.clone(),
        definition_versions: DefinitionVersions {
            ontology_version: context
                .ontology_registry
                .bundle()
                .definition_version
                .clone(),
            contract_version: context
                .action_contract_registry
                .bundle()
                .definition_version
                .clone(),
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
        },
        requested_at: request.created_at.clone(),
    };

    let decision = arbitrate_action(&arbitration_request, context);
    let response = response_from_arbitration_decision(
        request,
        &proposal.proposal_id,
        decision,
        next_query_hint,
    );
    Ok(response)
}

pub(crate) fn build_core_arbitration_context() -> Result<ArbitrationContext> {
    let ontology = core_ontology_registry();
    let contracts = core_action_contract_registry(&ontology);
    let role_policy = core_role_policy_registry(&ontology, &contracts);
    let object_state = core_object_state_registry(&ontology, &contracts)
        .map_err(|report| anyhow::anyhow!("load core object state registry: {:?}", report))?;
    Ok(ArbitrationContext::new(
        ontology,
        contracts,
        role_policy,
        object_state,
    ))
}

fn response_from_arbitration_decision(
    request: &RuntimeCommandRequest,
    proposal_id: &str,
    decision: agentflow_action_arbitration::ArbitrationDecision,
    next_query_hint: Option<crate::mapping::RuntimeQueryHint>,
) -> RuntimeCommandResponse {
    match decision.status {
        ArbitrationDecisionStatus::Accepted => RuntimeCommandResponse {
            version: RUNTIME_COMMAND_API_VERSION.to_string(),
            command_id: request.command_id.clone(),
            proposal_id: proposal_id.to_string(),
            status: RuntimeCommandStatus::Accepted,
            decision: RuntimeCommandDecision::Accepted,
            accepted_action_id: decision
                .accepted_action
                .as_ref()
                .map(|action| action.accepted_action_id.clone()),
            rejected_reasons: Vec::new(),
            human_decision_request: None,
            next_query_hint,
            correlation_id: request.command_id.clone(),
        },
        ArbitrationDecisionStatus::HumanDecisionRequired => RuntimeCommandResponse {
            version: RUNTIME_COMMAND_API_VERSION.to_string(),
            command_id: request.command_id.clone(),
            proposal_id: proposal_id.to_string(),
            status: RuntimeCommandStatus::HumanDecisionRequired,
            decision: RuntimeCommandDecision::HumanDecisionRequired,
            accepted_action_id: None,
            rejected_reasons: Vec::new(),
            human_decision_request: decision.required_human_decision.map(|required| {
                RuntimeHumanDecisionRequest {
                    question: required.question,
                    allowed_responses: required.allowed_responses,
                    required_evidence_type: required.required_evidence_type,
                }
            }),
            next_query_hint,
            correlation_id: request.command_id.clone(),
        },
        ArbitrationDecisionStatus::Rejected
        | ArbitrationDecisionStatus::ConflictDetected
        | ArbitrationDecisionStatus::Queued => RuntimeCommandResponse {
            version: RUNTIME_COMMAND_API_VERSION.to_string(),
            command_id: request.command_id.clone(),
            proposal_id: proposal_id.to_string(),
            status: RuntimeCommandStatus::Rejected,
            decision: RuntimeCommandDecision::Rejected,
            accepted_action_id: None,
            rejected_reasons: decision
                .rejected_reasons
                .into_iter()
                .map(|reason| {
                    RuntimeCommandError::new(
                        RuntimeCommandErrorCode::ArbitrationRejected,
                        reason.message,
                        reason.detail,
                    )
                })
                .collect(),
            human_decision_request: None,
            next_query_hint,
            correlation_id: request.command_id.clone(),
        },
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use agentflow_action_arbitration::{
        AcceptedAction, ArbitrationDecision, ArbitrationDecisionStatus, DefinitionVersions,
        DependencyFact, EvidenceFact, HumanDecisionRequest, HumanDecisionResponseKind,
        ObjectLockPlan, StateFact,
    };

    use super::{
        build_core_arbitration_context, execute_command_via_arbitration_with_context,
        map_command_to_action_proposal, response_from_arbitration_decision,
        validate_runtime_command, RuntimeCommandRequest,
    };
    use crate::mapping::target_ref;
    use crate::responses::RuntimeCommandStatus;
    use agentflow_action_contract::ActionSourceSurface;

    fn request(command_type: &str) -> RuntimeCommandRequest {
        RuntimeCommandRequest {
            command_id: format!("cmd-{command_type}"),
            command_type: command_type.to_string(),
            source_surface: ActionSourceSurface::Desktop,
            actor_role: "spec-agent".to_string(),
            target_object_ref: None,
            input: json!({ "summary": "整理需求", "requestType": "feature" }),
            evidence_refs: Vec::new(),
            artifact_refs: Vec::new(),
            idempotency_key: format!("idem-{command_type}"),
            created_at: "2026-06-20T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn valid_command_maps_to_action_proposal() {
        let proposal = map_command_to_action_proposal(&request("submitRequirement")).unwrap();
        assert_eq!(proposal.action_type, "submitRequirement");
        assert_eq!(proposal.idempotency_key, "idem-submitRequirement");
    }

    #[test]
    fn invalid_command_returns_invalid_command() {
        let response = execute_command_via_arbitration_with_context(
            &request("unknownCommand"),
            &build_core_arbitration_context().unwrap(),
        )
        .unwrap();
        assert_eq!(response.status, RuntimeCommandStatus::InvalidCommand);
        assert!(!response.rejected_reasons.is_empty());
    }

    #[test]
    fn accepted_decision_maps_to_command_response() {
        let request = request("submitRequirement");
        let response = response_from_arbitration_decision(
            &request,
            "proposal-cmd-submitRequirement",
            ArbitrationDecision {
                decision_id: "decision-cmd-submitRequirement".to_string(),
                request_id: request.command_id.clone(),
                proposal_id: "proposal-cmd-submitRequirement".to_string(),
                status: ArbitrationDecisionStatus::Accepted,
                accepted_action: Some(AcceptedAction {
                    accepted_action_id: "accepted-proposal-cmd-submitRequirement".to_string(),
                    proposal_id: "proposal-cmd-submitRequirement".to_string(),
                    idempotency_key: request.idempotency_key.clone(),
                    action_type: "submitRequirement".to_string(),
                    actor_role: "spec-agent".to_string(),
                    target_object_ref: None,
                    from_state: None,
                    to_state: Some("captured".to_string()),
                    evidence_refs: Vec::new(),
                    artifact_refs: Vec::new(),
                    expected_events: vec!["RequirementSubmitted".to_string()],
                    lock_plan: ObjectLockPlan::default(),
                    definition_versions: DefinitionVersions {
                        ontology_version: "v1".to_string(),
                        contract_version: "v1".to_string(),
                        role_policy_version: "v1".to_string(),
                        object_state_version: "v1".to_string(),
                    },
                }),
                rejected_reasons: Vec::new(),
                required_human_decision: None,
                lock_plan: ObjectLockPlan::default(),
                would_emit_events: vec!["RequirementSubmitted".to_string()],
                created_at: request.created_at.clone(),
            },
            Some(crate::mapping::RuntimeQueryHint {
                view: "RequirementIntakeView".to_string(),
                target_id: None,
                reason: "refresh".to_string(),
            }),
        );
        assert_eq!(response.status, RuntimeCommandStatus::Accepted);
        assert_eq!(
            response.accepted_action_id.as_deref(),
            Some("accepted-proposal-cmd-submitRequirement")
        );
    }

    #[test]
    fn human_decision_maps_to_command_response() {
        let mut request = request("approveSpec");
        request.actor_role = "human-owner".to_string();
        let response = response_from_arbitration_decision(
            &request,
            "proposal-cmd-approveSpec",
            ArbitrationDecision {
                decision_id: "decision-cmd-approveSpec".to_string(),
                request_id: request.command_id.clone(),
                proposal_id: "proposal-cmd-approveSpec".to_string(),
                status: ArbitrationDecisionStatus::HumanDecisionRequired,
                accepted_action: None,
                rejected_reasons: Vec::new(),
                required_human_decision: Some(HumanDecisionRequest {
                    decision_kind: HumanDecisionResponseKind::ApprovalRequired,
                    target_object_ref: Some(target_ref("Spec", "spec-325")),
                    question:
                        "action `approveSpec` requires explicit human decision before Runtime can accept it"
                            .to_string(),
                    allowed_responses: vec!["approve".to_string(), "reject".to_string()],
                    required_evidence_type: "humanConfirmation".to_string(),
                }),
                lock_plan: ObjectLockPlan::default(),
                would_emit_events: Vec::new(),
                created_at: request.created_at.clone(),
            },
            Some(crate::mapping::RuntimeQueryHint {
                view: "SpecPreviewView".to_string(),
                target_id: Some("spec-325".to_string()),
                reason: "refresh".to_string(),
            }),
        );
        assert_eq!(response.status, RuntimeCommandStatus::HumanDecisionRequired);
        assert!(response.human_decision_request.is_some());
    }

    #[test]
    fn rejected_reasons_propagate_from_arbitration() {
        let mut request = request("submitEvidence");
        request.actor_role = "work-agent".to_string();
        request.target_object_ref = Some(target_ref("Run", "run-001"));
        request.input = json!({ "evidenceSummary": "构建日志" });
        request.evidence_refs = vec!["missing-log".to_string()];

        let mut context = build_core_arbitration_context().unwrap();
        context.insert_state(StateFact {
            object_type: "Run".to_string(),
            object_id: "run-001".to_string(),
            state_id: "in_progress".to_string(),
        });
        context.insert_dependency(DependencyFact {
            dependency_key: "noop".to_string(),
            satisfied: true,
            reason: None,
        });

        let response = execute_command_via_arbitration_with_context(&request, &context).unwrap();
        assert_eq!(response.status, RuntimeCommandStatus::Rejected);
        assert!(!response.rejected_reasons.is_empty());
    }

    #[test]
    fn idempotency_key_passes_through_validation() {
        let report = validate_runtime_command(&request("submitRequirement"));
        assert!(report.valid);
        assert_eq!(
            report.normalized_action_type.as_deref(),
            Some("submitRequirement")
        );
    }

    #[test]
    fn request_audit_with_human_confirmation_can_validate() {
        let mut request = request("recordDecision");
        request.actor_role = "human-owner".to_string();
        request.input = json!({
            "outcome": "approve",
            "targetObjectType": "Spec",
            "targetObjectId": "AF-325"
        });
        request.evidence_refs = vec!["human-confirmation-1".to_string()];

        let mut context = build_core_arbitration_context().unwrap();
        context.insert_evidence(EvidenceFact {
            evidence_ref: "human-confirmation-1".to_string(),
            evidence_type: "humanConfirmation".to_string(),
        });

        let response = execute_command_via_arbitration_with_context(&request, &context).unwrap();
        assert_ne!(response.status, RuntimeCommandStatus::InvalidCommand);
    }
}
