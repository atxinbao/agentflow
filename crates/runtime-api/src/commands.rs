use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    fs,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use agentflow_action_arbitration::{
    arbitrate_action, proposal_conflict_scope_key, AcceptedAction, ArbitrationContext,
    ArbitrationDecision, ArbitrationDecisionStatus, ArbitrationRequest, DefinitionVersions,
    PendingProposal,
};
use agentflow_action_contract::{core_action_contract_registry, ActionRef, ActionSourceSurface};
use agentflow_capability_registry::{default_capability_registry, CapabilityRegistry};
use agentflow_event_store::{append_accepted_action_event, AcceptedActionAppendContext, TaskEvent};
use agentflow_governance_policy::{
    evaluate_runtime_governance, AuditSidecarMode, GovernanceDecision, GovernancePolicyReport,
    GovernancePolicyRequest,
};
use agentflow_object_state::core_object_state_registry;
use agentflow_ontology::software_dev_reference_ontology_registry;
use agentflow_role_policy::core_role_policy_registry;
use agentflow_workflow_core::{WorkflowAgentRole, WorkflowFlowType};
use agentflow_workflow_runtime::{
    load_runtime_decision_facts, load_runtime_lock_snapshot, load_runtime_proposal_facts,
    prepare_runtime_workspace, write_runtime_accepted_action_fact, write_runtime_command_fact,
    write_runtime_decision_fact, write_runtime_proposal_fact, RuntimeAcceptedActionFact,
    RuntimeCommandFact, RuntimeCommandValidationFact, RuntimeDecisionFact, RuntimeProposalFact,
    RuntimeQueryHintFact, RUNTIME_ACCEPTED_ACTION_FACT_VERSION, RUNTIME_COMMAND_FACT_VERSION,
    RUNTIME_DECISION_FACT_VERSION, RUNTIME_PROPOSAL_FACT_VERSION,
};

use crate::errors::{RuntimeCommandError, RuntimeCommandErrorCode};
use crate::mapping::{
    map_command_to_action_proposal, missing_field_error, runtime_query_hint_for_command,
    source_surface_label, unsupported_command_error,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub route: Option<RuntimeCommandRoute>,
    pub source_surface: ActionSourceSurface,
    pub actor_role: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skill_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_object_ref: Option<ActionRef>,
    pub input: Value,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
    #[serde(default)]
    pub artifact_refs: Vec<String>,
    #[serde(default)]
    pub expected_outputs: Vec<RuntimeExpectedOutputRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence_policy: Option<RuntimeEvidencePolicyRef>,
    pub idempotency_key: String,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeCommandRoute {
    pub route_id: String,
    pub action_contract_ref: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_object_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pack_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pack_command: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeExpectedOutputRef {
    pub output_type: String,
    pub reference: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeEvidencePolicyRef {
    pub policy_id: String,
    #[serde(default)]
    pub required_evidence: Vec<String>,
    pub missing_evidence_behavior: String,
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
    project_root: impl AsRef<Path>,
    request: &RuntimeCommandRequest,
) -> Result<RuntimeCommandResponse> {
    let root = project_root.as_ref();
    let context = build_project_arbitration_context(root)?;
    execute_command_via_arbitration_with_context(root, request, &context)
}

pub fn execute_command_via_arbitration_with_context(
    project_root: impl AsRef<Path>,
    request: &RuntimeCommandRequest,
    context: &ArbitrationContext,
) -> Result<RuntimeCommandResponse> {
    let root = project_root.as_ref();
    prepare_runtime_workspace(root)?;
    let validation = validate_runtime_command(request);
    let proposal_id = format!("proposal-{}", request.command_id);
    let next_query_hint = Some(runtime_query_hint_for_command(request));
    let recorded_at = unix_timestamp_seconds();

    write_runtime_command_fact(
        root,
        &RuntimeCommandFact {
            version: RUNTIME_COMMAND_FACT_VERSION.to_string(),
            command_id: request.command_id.clone(),
            command_type: request.command_type.clone(),
            route: request
                .route
                .as_ref()
                .and_then(|route| serde_json::to_value(route).ok()),
            source_surface: request.source_surface.clone(),
            actor_role: request.actor_role.clone(),
            skill_ref: request.skill_ref.clone(),
            target_object_ref: request.target_object_ref.clone(),
            input: request.input.clone(),
            evidence_refs: request.evidence_refs.clone(),
            artifact_refs: request.artifact_refs.clone(),
            expected_outputs: request
                .expected_outputs
                .iter()
                .filter_map(|output| serde_json::to_value(output).ok())
                .collect(),
            evidence_policy: request
                .evidence_policy
                .as_ref()
                .and_then(|policy| serde_json::to_value(policy).ok()),
            idempotency_key: request.idempotency_key.clone(),
            created_at: request.created_at.clone(),
            recorded_at,
            validation: build_validation_fact(&validation),
        },
    )?;

    if !validation.valid {
        let response = RuntimeCommandResponse {
            version: RUNTIME_COMMAND_API_VERSION.to_string(),
            command_id: request.command_id.clone(),
            proposal_id,
            status: RuntimeCommandStatus::InvalidCommand,
            decision: RuntimeCommandDecision::InvalidCommand,
            accepted_action_id: None,
            rejected_reasons: validation.errors,
            human_decision_request: None,
            next_query_hint,
            governance_admission: None,
            correlation_id: request.command_id.clone(),
        };
        write_runtime_decision_fact(
            root,
            &build_runtime_decision_fact(
                request,
                response.proposal_id.as_str(),
                None,
                &response,
                None,
                recorded_at,
            ),
        )?;
        return Ok(response);
    }

    let proposal = map_command_to_action_proposal(request)?;
    let governance_admission =
        evaluate_runtime_command_governance(root, request, &proposal, context);
    if governance_admission.decision != GovernanceDecision::Allowed {
        let response = response_from_governance_admission(
            request,
            &proposal.proposal_id,
            &governance_admission,
            next_query_hint,
        );
        write_runtime_decision_fact(
            root,
            &build_runtime_decision_fact(
                request,
                response.proposal_id.as_str(),
                None,
                &response,
                None,
                recorded_at,
            ),
        )?;
        return Ok(response);
    }

    write_runtime_proposal_fact(
        root,
        &RuntimeProposalFact {
            version: RUNTIME_PROPOSAL_FACT_VERSION.to_string(),
            command_id: request.command_id.clone(),
            proposal_id: proposal.proposal_id.clone(),
            action_type: proposal.action_type.clone(),
            actor_role: proposal.actor_role.clone(),
            source_surface: proposal.source_surface.clone(),
            target_object_ref: proposal.target_object_ref.clone(),
            input: proposal.input.clone(),
            evidence_refs: proposal.evidence_refs.clone(),
            artifact_refs: proposal.artifact_refs.clone(),
            reason: proposal.reason.clone(),
            expected_effects: proposal.expected_effects.clone(),
            ontology_version: proposal.ontology_version.clone(),
            contract_version: proposal.contract_version.clone(),
            created_at: proposal.created_at.clone(),
            recorded_at,
        },
    )?;
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
    let mut response = response_from_arbitration_decision(
        request,
        &proposal.proposal_id,
        &decision,
        next_query_hint,
    );
    response.governance_admission = Some(governance_admission);

    let mut accepted_action_event = None;
    if let Some(accepted_action) = decision.accepted_action.as_ref() {
        accepted_action_event = Some(append_runtime_accepted_action_event(
            root,
            request,
            &proposal,
            accepted_action,
            &decision,
            context,
        )?);
        write_runtime_accepted_action_fact(
            root,
            &build_runtime_accepted_action_fact(
                request,
                accepted_action,
                accepted_action_event.as_ref(),
                recorded_at,
            ),
        )?;
    }
    write_runtime_decision_fact(
        root,
        &build_runtime_decision_fact(
            request,
            &proposal.proposal_id,
            Some(&decision),
            &response,
            accepted_action_event,
            recorded_at,
        ),
    )?;
    Ok(response)
}

pub(crate) fn build_core_arbitration_context() -> Result<ArbitrationContext> {
    let ontology = software_dev_reference_ontology_registry();
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

pub(crate) fn build_project_arbitration_context(
    project_root: impl AsRef<Path>,
) -> Result<ArbitrationContext> {
    let mut context = build_core_arbitration_context()?;
    let snapshot = load_runtime_lock_snapshot(&project_root)?;
    for record in snapshot.active_object_locks {
        context.push_lock(record.lock);
    }
    let proposal_facts = load_runtime_proposal_facts(&project_root)?;
    let decision_facts = load_runtime_decision_facts(&project_root)?;
    for proposal in proposal_facts {
        let Some(status) = decision_facts
            .iter()
            .find(|decision| decision.proposal_id == proposal.proposal_id)
            .and_then(|decision| {
                arbitration_status_from_runtime_decision(decision.status.as_str())
            })
        else {
            continue;
        };
        if !matches!(
            status,
            ArbitrationDecisionStatus::Accepted
                | ArbitrationDecisionStatus::HumanDecisionRequired
                | ArbitrationDecisionStatus::Queued
                | ArbitrationDecisionStatus::ConflictDetected
        ) {
            continue;
        }
        let conflict_scope_key = proposal_conflict_scope_key(
            &context,
            &agentflow_action_contract::ActionProposal {
                proposal_id: proposal.proposal_id.clone(),
                idempotency_key: String::new(),
                action_type: proposal.action_type.clone(),
                actor_role: proposal.actor_role.clone(),
                source_surface: proposal.source_surface.clone(),
                target_object_ref: proposal.target_object_ref.clone(),
                input: proposal.input.clone(),
                evidence_refs: proposal.evidence_refs.clone(),
                artifact_refs: proposal.artifact_refs.clone(),
                reason: proposal.reason.clone(),
                expected_effects: proposal.expected_effects.clone(),
                ontology_version: proposal.ontology_version.clone(),
                contract_version: proposal.contract_version.clone(),
                created_at: proposal.created_at.clone(),
            },
        );
        context.push_pending_proposal(PendingProposal {
            proposal_id: proposal.proposal_id,
            actor_role: proposal.actor_role,
            action_type: proposal.action_type,
            target_object_ref: proposal.target_object_ref,
            conflict_scope_key,
            status,
            created_at: proposal.created_at,
        });
    }
    Ok(context)
}

fn response_from_arbitration_decision(
    request: &RuntimeCommandRequest,
    proposal_id: &str,
    decision: &ArbitrationDecision,
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
            governance_admission: None,
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
            human_decision_request: decision.required_human_decision.as_ref().map(|required| {
                RuntimeHumanDecisionRequest {
                    question: required.question.clone(),
                    allowed_responses: required.allowed_responses.clone(),
                    required_evidence_type: required.required_evidence_type.clone(),
                }
            }),
            next_query_hint,
            governance_admission: None,
            correlation_id: request.command_id.clone(),
        },
        ArbitrationDecisionStatus::Queued => RuntimeCommandResponse {
            version: RUNTIME_COMMAND_API_VERSION.to_string(),
            command_id: request.command_id.clone(),
            proposal_id: proposal_id.to_string(),
            status: RuntimeCommandStatus::Queued,
            decision: RuntimeCommandDecision::Queued,
            accepted_action_id: None,
            rejected_reasons: decision
                .rejected_reasons
                .iter()
                .map(|reason| {
                    RuntimeCommandError::new(
                        RuntimeCommandErrorCode::ArbitrationQueued,
                        reason.message.clone(),
                        reason.detail.clone(),
                    )
                })
                .collect(),
            human_decision_request: None,
            next_query_hint,
            governance_admission: None,
            correlation_id: request.command_id.clone(),
        },
        ArbitrationDecisionStatus::Superseded => RuntimeCommandResponse {
            version: RUNTIME_COMMAND_API_VERSION.to_string(),
            command_id: request.command_id.clone(),
            proposal_id: proposal_id.to_string(),
            status: RuntimeCommandStatus::Superseded,
            decision: RuntimeCommandDecision::Superseded,
            accepted_action_id: None,
            rejected_reasons: decision
                .rejected_reasons
                .iter()
                .map(|reason| {
                    RuntimeCommandError::new(
                        RuntimeCommandErrorCode::ArbitrationSuperseded,
                        reason.message.clone(),
                        reason.detail.clone(),
                    )
                })
                .collect(),
            human_decision_request: None,
            next_query_hint,
            governance_admission: None,
            correlation_id: request.command_id.clone(),
        },
        ArbitrationDecisionStatus::Cancelled => RuntimeCommandResponse {
            version: RUNTIME_COMMAND_API_VERSION.to_string(),
            command_id: request.command_id.clone(),
            proposal_id: proposal_id.to_string(),
            status: RuntimeCommandStatus::Cancelled,
            decision: RuntimeCommandDecision::Cancelled,
            accepted_action_id: None,
            rejected_reasons: decision
                .rejected_reasons
                .iter()
                .map(|reason| {
                    RuntimeCommandError::new(
                        RuntimeCommandErrorCode::ArbitrationCancelled,
                        reason.message.clone(),
                        reason.detail.clone(),
                    )
                })
                .collect(),
            human_decision_request: None,
            next_query_hint,
            governance_admission: None,
            correlation_id: request.command_id.clone(),
        },
        ArbitrationDecisionStatus::Rejected | ArbitrationDecisionStatus::ConflictDetected => {
            RuntimeCommandResponse {
                version: RUNTIME_COMMAND_API_VERSION.to_string(),
                command_id: request.command_id.clone(),
                proposal_id: proposal_id.to_string(),
                status: RuntimeCommandStatus::Rejected,
                decision: RuntimeCommandDecision::Rejected,
                accepted_action_id: None,
                rejected_reasons: decision
                    .rejected_reasons
                    .iter()
                    .map(|reason| {
                        RuntimeCommandError::new(
                            RuntimeCommandErrorCode::ArbitrationRejected,
                            reason.message.clone(),
                            reason.detail.clone(),
                        )
                    })
                    .collect(),
                human_decision_request: None,
                next_query_hint,
                governance_admission: None,
                correlation_id: request.command_id.clone(),
            }
        }
    }
}

fn evaluate_runtime_command_governance(
    project_root: &Path,
    request: &RuntimeCommandRequest,
    proposal: &agentflow_action_contract::ActionProposal,
    context: &ArbitrationContext,
) -> GovernancePolicyReport {
    let capability_registry = capability_registry_for_runtime_command(project_root);
    evaluate_runtime_governance(
        &context.role_policy_registry,
        &capability_registry,
        GovernancePolicyRequest {
            actor_role: request.actor_role.clone(),
            action_type: proposal.action_type.clone(),
            generic_action: Some(
                runtime_action_to_generic_action(&proposal.action_type).to_string(),
            ),
            object_type: proposal
                .target_object_ref
                .as_ref()
                .map(|target| target.object_type.clone()),
            skill_ref: request.skill_ref.clone(),
            source_surface: Some(source_surface_label(&request.source_surface).to_string()),
            tool_scopes: value_string_array(&request.input, "toolScopes"),
            connector_scopes: value_string_array(&request.input, "connectorScopes"),
            evidence_refs: request.evidence_refs.clone(),
            artifact_refs: request.artifact_refs.clone(),
            expected_outputs: request
                .expected_outputs
                .iter()
                .map(|output| output.output_type.clone())
                .collect(),
            worker_id: governance_worker_id(request),
            command: governance_command(request),
            audit_sidecar_mode: governance_audit_sidecar_mode(request),
        },
    )
}

fn response_from_governance_admission(
    request: &RuntimeCommandRequest,
    proposal_id: &str,
    report: &GovernancePolicyReport,
    next_query_hint: Option<crate::mapping::RuntimeQueryHint>,
) -> RuntimeCommandResponse {
    let (status, decision, code) = match report.decision {
        GovernanceDecision::Rejected => (
            RuntimeCommandStatus::Rejected,
            RuntimeCommandDecision::Rejected,
            RuntimeCommandErrorCode::GovernanceRejected,
        ),
        GovernanceDecision::Deferred => (
            RuntimeCommandStatus::Deferred,
            RuntimeCommandDecision::Deferred,
            RuntimeCommandErrorCode::GovernanceDeferred,
        ),
        GovernanceDecision::Allowed => (
            RuntimeCommandStatus::Accepted,
            RuntimeCommandDecision::Accepted,
            RuntimeCommandErrorCode::GovernanceRejected,
        ),
    };
    let stage = report
        .trace
        .iter()
        .find(|entry| entry.decision == report.decision)
        .or_else(|| report.trace.first());
    let stage_name = stage
        .map(|entry| entry.stage.clone())
        .unwrap_or_else(|| "governance-admission".to_string());
    let reason = stage
        .map(|entry| entry.reason.clone())
        .unwrap_or_else(|| "runtime governance admission did not allow this command".to_string());

    RuntimeCommandResponse {
        version: RUNTIME_COMMAND_API_VERSION.to_string(),
        command_id: request.command_id.clone(),
        proposal_id: proposal_id.to_string(),
        status,
        decision,
        accepted_action_id: None,
        rejected_reasons: vec![RuntimeCommandError::new(
            code,
            format!(
                "governance admission {} at {}: {}",
                report.decision.as_str(),
                stage_name,
                reason
            ),
            Some(format!("governance.{stage_name}")),
        )],
        human_decision_request: None,
        next_query_hint,
        governance_admission: Some(report.clone()),
        correlation_id: request.command_id.clone(),
    }
}

fn capability_registry_for_runtime_command(project_root: &Path) -> CapabilityRegistry {
    let trusted_registry_path = project_root.join(".agentflow/runtime/capability-registry.json");
    fs::read_to_string(&trusted_registry_path)
        .ok()
        .and_then(|text| serde_json::from_str::<CapabilityRegistry>(&text).ok())
        .unwrap_or_else(default_capability_registry)
}

fn governance_worker_id(request: &RuntimeCommandRequest) -> String {
    value_string(&request.input, "governanceWorkerId")
        .or_else(|| value_string(&request.input, "workerId"))
        .unwrap_or_else(|| "local-shell-validator".to_string())
}

fn governance_command(request: &RuntimeCommandRequest) -> String {
    value_string(&request.input, "governanceCommand")
        .or_else(|| value_string(&request.input, "workerCommand"))
        .unwrap_or_else(|| "validate.build".to_string())
}

fn governance_audit_sidecar_mode(request: &RuntimeCommandRequest) -> AuditSidecarMode {
    match value_string(&request.input, "auditSidecarMode")
        .unwrap_or_default()
        .as_str()
    {
        "independent" => AuditSidecarMode::Independent,
        "bound-to-main-chain" => AuditSidecarMode::BoundToMainChain,
        _ => AuditSidecarMode::NotRequested,
    }
}

fn build_validation_fact(report: &RuntimeCommandValidationReport) -> RuntimeCommandValidationFact {
    RuntimeCommandValidationFact {
        valid: report.valid,
        normalized_action_type: report.normalized_action_type.clone(),
        errors: report
            .errors
            .iter()
            .map(|error| error.message.clone())
            .collect(),
        warnings: report.warnings.clone(),
    }
}

fn build_runtime_decision_fact(
    request: &RuntimeCommandRequest,
    proposal_id: &str,
    decision: Option<&ArbitrationDecision>,
    response: &RuntimeCommandResponse,
    accepted_action_event: Option<TaskEvent>,
    recorded_at: u64,
) -> RuntimeDecisionFact {
    RuntimeDecisionFact {
        version: RUNTIME_DECISION_FACT_VERSION.to_string(),
        command_id: request.command_id.clone(),
        proposal_id: proposal_id.to_string(),
        decision_id: decision.map(|value| value.decision_id.clone()),
        status: runtime_command_status_str(&response.status).to_string(),
        decision: runtime_command_decision_str(&response.decision).to_string(),
        blocking_proposal_id: decision.and_then(|value| value.blocking_proposal_id.clone()),
        accepted_action_id: response.accepted_action_id.clone(),
        rejected_reasons: response
            .rejected_reasons
            .iter()
            .map(|reason| reason.message.clone())
            .collect(),
        human_decision_request: response
            .human_decision_request
            .as_ref()
            .map(|request| request.question.clone()),
        next_query_hint: response
            .next_query_hint
            .as_ref()
            .map(|hint| RuntimeQueryHintFact {
                view: hint.view.clone(),
                target_id: hint.target_id.clone(),
                reason: hint.reason.clone(),
            }),
        governance_admission: response
            .governance_admission
            .as_ref()
            .and_then(|report| serde_json::to_value(report).ok()),
        correlation_id: response.correlation_id.clone(),
        would_emit_events: accepted_action_event
            .as_ref()
            .map(|event| vec![event.event_type.clone()])
            .unwrap_or_else(|| {
                decision
                    .map(|value| value.would_emit_events.clone())
                    .unwrap_or_default()
            }),
        recorded_at,
    }
}

fn build_runtime_accepted_action_fact(
    request: &RuntimeCommandRequest,
    accepted_action: &AcceptedAction,
    accepted_action_event: Option<&TaskEvent>,
    recorded_at: u64,
) -> RuntimeAcceptedActionFact {
    let mut event_id = None;
    let mut event_path = None;
    let mut event_type = None;
    let mut issue_id = None;
    let mut run_id = None;
    if let Some(event) = accepted_action_event {
        event_type = Some(event.event_type.clone());
        event_id = Some(event.event_id.clone());
        event_path = Some(format!(
            ".agentflow/events/task-events/{}.json",
            event.event_id
        ));
        issue_id = event.issue_id.clone();
        run_id = event.run_id.clone();
    }

    RuntimeAcceptedActionFact {
        version: RUNTIME_ACCEPTED_ACTION_FACT_VERSION.to_string(),
        command_id: request.command_id.clone(),
        proposal_id: accepted_action.proposal_id.clone(),
        accepted_action_id: accepted_action.accepted_action_id.clone(),
        issue_id,
        run_id,
        action_type: accepted_action.action_type.clone(),
        actor_role: accepted_action.actor_role.clone(),
        target_object_ref: accepted_action.target_object_ref.clone(),
        from_state: accepted_action.from_state.clone(),
        to_state: accepted_action.to_state.clone(),
        evidence_refs: accepted_action.evidence_refs.clone(),
        artifact_refs: accepted_action.artifact_refs.clone(),
        expected_events: accepted_action.expected_events.clone(),
        lock_plan: accepted_action.lock_plan.clone(),
        definition_versions: accepted_action.definition_versions.clone(),
        event_id,
        event_path,
        event_type,
        recorded_at,
    }
}

fn append_runtime_accepted_action_event(
    project_root: &Path,
    request: &RuntimeCommandRequest,
    proposal: &agentflow_action_contract::ActionProposal,
    accepted_action: &AcceptedAction,
    decision: &ArbitrationDecision,
    context: &ArbitrationContext,
) -> Result<TaskEvent> {
    let contract = context
        .action_contract_registry
        .get_action_contract(&proposal.action_type, &proposal.contract_version)
        .ok_or_else(|| anyhow::anyhow!("missing action contract {}", proposal.action_type))?;
    let event_type = decision
        .would_emit_events
        .first()
        .cloned()
        .or_else(|| accepted_action.expected_events.first().cloned())
        .unwrap_or_else(|| format!("{}Accepted", accepted_action.action_type));
    let aggregate_type = contract
        .expected_events
        .first()
        .and_then(|event| event.object_type.clone())
        .or_else(|| contract.creates_object_type.clone())
        .or_else(|| {
            accepted_action
                .target_object_ref
                .as_ref()
                .map(|target| target.object_type.clone())
        })
        .unwrap_or_else(|| "RuntimeCommand".to_string());
    let aggregate_id =
        resolve_runtime_aggregate_id(&aggregate_type, request, proposal, accepted_action);
    let task_event = append_accepted_action_event(
        project_root,
        accepted_action,
        AcceptedActionAppendContext {
            flow_type: flow_type_for_object_type(&aggregate_type),
            aggregate_type: aggregate_type.clone(),
            aggregate_id: aggregate_id.clone(),
            project_id: resolve_project_id(
                &aggregate_type,
                &aggregate_id,
                request,
                accepted_action,
            ),
            issue_id: resolve_issue_id(&aggregate_type, &aggregate_id, request, accepted_action),
            run_id: resolve_run_id(&aggregate_type, &aggregate_id, request),
            event_type: event_type.clone(),
            authority_role: WorkflowAgentRole::parse_alias(&accepted_action.actor_role),
            actor_kind: "runtime-api".to_string(),
            correlation_id: request.command_id.clone(),
            causation_id: Some(proposal.proposal_id.clone()),
            occurred_at: Some(unix_timestamp_seconds()),
            decision: Some("accepted".to_string()),
            payload: json!({
                "commandId": request.command_id,
                "commandType": request.command_type,
                "proposalId": proposal.proposal_id,
                "acceptedActionId": accepted_action.accepted_action_id,
                "targetObjectRef": accepted_action.target_object_ref,
                "sourceSurface": request.source_surface,
            }),
        },
    )?;
    Ok(task_event)
}

fn resolve_runtime_aggregate_id(
    aggregate_type: &str,
    request: &RuntimeCommandRequest,
    proposal: &agentflow_action_contract::ActionProposal,
    accepted_action: &AcceptedAction,
) -> String {
    if let Some(target) = accepted_action.target_object_ref.as_ref() {
        if target.object_type == aggregate_type {
            return target.id.clone();
        }
    }
    if let Some(target) = proposal.target_object_ref.as_ref() {
        if target.object_type == aggregate_type {
            return target.id.clone();
        }
    }
    candidate_value_strings(&request.input, aggregate_type)
        .or_else(|| value_string(&request.input, "id"))
        .unwrap_or_else(|| request.command_id.clone())
}

fn resolve_project_id(
    aggregate_type: &str,
    aggregate_id: &str,
    request: &RuntimeCommandRequest,
    accepted_action: &AcceptedAction,
) -> Option<String> {
    if aggregate_type == "Project" {
        return Some(aggregate_id.to_string());
    }
    if let Some(target) = accepted_action.target_object_ref.as_ref() {
        if target.object_type == "Project" {
            return Some(target.id.clone());
        }
    }
    value_string(&request.input, "projectId")
}

fn resolve_issue_id(
    aggregate_type: &str,
    aggregate_id: &str,
    request: &RuntimeCommandRequest,
    accepted_action: &AcceptedAction,
) -> Option<String> {
    if aggregate_type == "Issue" {
        return Some(aggregate_id.to_string());
    }
    if let Some(target) = accepted_action.target_object_ref.as_ref() {
        if target.object_type == "Issue" {
            return Some(target.id.clone());
        }
    }
    value_string(&request.input, "issueId")
}

fn resolve_run_id(
    aggregate_type: &str,
    aggregate_id: &str,
    request: &RuntimeCommandRequest,
) -> Option<String> {
    if aggregate_type == "Run" {
        return Some(aggregate_id.to_string());
    }
    value_string(&request.input, "runId")
}

fn flow_type_for_object_type(object_type: &str) -> WorkflowFlowType {
    match object_type {
        "Project" | "Requirement" | "Spec" => WorkflowFlowType::Project,
        "Issue" | "Run" => WorkflowFlowType::Work,
        "Audit" | "Finding" => WorkflowFlowType::Audit,
        "Release" | "Delivery" => WorkflowFlowType::Delivery,
        _ => WorkflowFlowType::Work,
    }
}

fn candidate_value_strings(input: &Value, object_type: &str) -> Option<String> {
    let keys: &[&str] = match object_type {
        "Project" => &["projectId", "id"],
        "Issue" => &["issueId", "id"],
        "Requirement" => &["requirementId", "id"],
        "Spec" => &["specId", "id"],
        "Run" => &["runId", "id"],
        "Release" => &["releaseId", "id"],
        "Delivery" => &["deliveryId", "id"],
        "Audit" => &["auditId", "id"],
        "Finding" => &["findingId", "id"],
        _ => &["id"],
    };
    keys.iter().find_map(|key| value_string(input, key))
}

fn value_string(input: &Value, key: &str) -> Option<String> {
    input.get(key).and_then(Value::as_str).map(str::to_string)
}

fn value_string_array(input: &Value, key: &str) -> Vec<String> {
    input
        .get(key)
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_action_to_generic_action(action_type: &str) -> &'static str {
    match action_type {
        "submitRequirement"
        | "normalizeRequirement"
        | "classifyRequirement"
        | "draftSpec"
        | "approveSpec"
        | "createProject"
        | "createIssue" => "acceptObject",
        "activateIssue" | "claimIssue" | "startRun" => "startObject",
        "writePatch" | "runValidation" | "submitEvidence" => "attachEvidence",
        "prepareDelivery" | "submitArtifact" => "attachArtifact",
        "markIssueDone" => "submitForReview",
        "recordDecision" => "completeObject",
        "requestAudit" => "submitForReview",
        "createFinding" => "blockObject",
        "linkFixIssue" => "supersedeObject",
        _ => "unlisted-action",
    }
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

fn runtime_command_decision_str(decision: &RuntimeCommandDecision) -> &'static str {
    match decision {
        RuntimeCommandDecision::Accepted => "accepted",
        RuntimeCommandDecision::Rejected => "rejected",
        RuntimeCommandDecision::Deferred => "deferred",
        RuntimeCommandDecision::HumanDecisionRequired => "human-decision-required",
        RuntimeCommandDecision::Queued => "queued",
        RuntimeCommandDecision::Superseded => "superseded",
        RuntimeCommandDecision::Cancelled => "cancelled",
        RuntimeCommandDecision::InvalidCommand => "invalid-command",
    }
}

fn arbitration_status_from_runtime_decision(status: &str) -> Option<ArbitrationDecisionStatus> {
    match status {
        "accepted" => Some(ArbitrationDecisionStatus::Accepted),
        "rejected" => Some(ArbitrationDecisionStatus::Rejected),
        "human-decision-required" => Some(ArbitrationDecisionStatus::HumanDecisionRequired),
        "queued" => Some(ArbitrationDecisionStatus::Queued),
        "superseded" => Some(ArbitrationDecisionStatus::Superseded),
        "cancelled" => Some(ArbitrationDecisionStatus::Cancelled),
        "conflict-detected" => Some(ArbitrationDecisionStatus::ConflictDetected),
        _ => None,
    }
}

fn unix_timestamp_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use std::fs;
    use tempfile::tempdir;

    use agentflow_action_arbitration::{
        AcceptedAction, ArbitrationDecision, ArbitrationDecisionStatus, DefinitionVersions,
        DependencyFact, EvidenceFact, HumanDecisionRequest, HumanDecisionResponseKind, ObjectLock,
        ObjectLockKind, ObjectLockPlan, StateFact,
    };
    use agentflow_capability_registry::build_capability_registry_with_smoke;
    use agentflow_event_store::{claim_task_event, EventActor, TaskEventDraft};
    use agentflow_mcp::{McpProviderSmokeArtifact, McpProviderStatus};
    use agentflow_workflow_core::{WorkflowAgentRole, WorkflowFlowType};
    use agentflow_workflow_runtime::{
        load_runtime_accepted_action_fact, load_runtime_accepted_action_facts,
        load_runtime_command_fact, load_runtime_decision_fact, load_runtime_proposal_fact,
        load_runtime_proposal_facts, write_runtime_accepted_action_fact, RuntimeAcceptedActionFact,
        RUNTIME_ACCEPTED_ACTION_FACT_VERSION,
    };

    use super::{
        build_core_arbitration_context, build_project_arbitration_context,
        execute_command_via_arbitration_with_context, map_command_to_action_proposal,
        response_from_arbitration_decision, validate_runtime_command, RuntimeCommandRequest,
    };
    use crate::mapping::{
        action_contract_ref_for_action_type, core_runtime_route, target_ref,
        CORE_RUNTIME_COMMAND_TYPE,
    };
    use crate::responses::RuntimeCommandStatus;
    use agentflow_action_contract::ActionSourceSurface;

    fn request(command_type: &str) -> RuntimeCommandRequest {
        RuntimeCommandRequest {
            command_id: format!("cmd-{command_type}"),
            command_type: CORE_RUNTIME_COMMAND_TYPE.to_string(),
            route: action_contract_ref_for_action_type(command_type).map(|contract_ref| {
                core_runtime_route(format!("core:{command_type}"), contract_ref, None::<String>)
            }),
            source_surface: ActionSourceSurface::Desktop,
            actor_role: "spec-agent".to_string(),
            skill_ref: Some(format!("core:spec-agent:{command_type}")),
            target_object_ref: None,
            input: json!({ "summary": "整理需求", "requestType": "feature" }),
            evidence_refs: Vec::new(),
            artifact_refs: Vec::new(),
            expected_outputs: Vec::new(),
            evidence_policy: None,
            idempotency_key: format!("idem-{command_type}"),
            created_at: "2026-06-20T00:00:00Z".to_string(),
        }
    }

    fn legacy_request(command_type: &str) -> RuntimeCommandRequest {
        let mut value = request(command_type);
        value.command_type = command_type.to_string();
        value.route = None;
        value.skill_ref = None;
        value
    }

    fn project_request(command_id: &str) -> RuntimeCommandRequest {
        RuntimeCommandRequest {
            command_id: command_id.to_string(),
            command_type: CORE_RUNTIME_COMMAND_TYPE.to_string(),
            route: Some(core_runtime_route(
                "core:project.create",
                "action-contract:project.create",
                Some("Spec"),
            )),
            source_surface: ActionSourceSurface::Agent,
            actor_role: "spec-agent".to_string(),
            skill_ref: Some("core:spec-agent:project.create".to_string()),
            target_object_ref: Some(target_ref("Spec", "spec-001")),
            input: json!({
                "projectId": "project-001",
                "projectTitle": "Governance Admission Project"
            }),
            evidence_refs: vec![
                "DecisionRef:approved-spec-1".to_string(),
                "EvidenceRef:human-confirmation-1".to_string(),
            ],
            artifact_refs: vec![
                "ArtifactRef:.agentflow/spec/requirements/req-001/preview.json".to_string(),
            ],
            expected_outputs: Vec::new(),
            evidence_policy: None,
            idempotency_key: format!("spec:req-001:project:project-001:{command_id}"),
            created_at: "2026-06-20T00:00:00Z".to_string(),
        }
    }

    fn work_run_request(command_id: &str) -> RuntimeCommandRequest {
        RuntimeCommandRequest {
            command_id: command_id.to_string(),
            command_type: CORE_RUNTIME_COMMAND_TYPE.to_string(),
            route: Some(core_runtime_route(
                "core:issue.start",
                "action-contract:issue.start",
                Some("Issue"),
            )),
            source_surface: ActionSourceSurface::Agent,
            actor_role: "work-agent".to_string(),
            skill_ref: Some("core:work-agent:work-execution-skill".to_string()),
            target_object_ref: Some(target_ref("Issue", "AF-653")),
            input: json!({
                "runId": "run-653",
                "governanceWorkerId": "local-shell-validator",
                "governanceCommand": "local.test"
            }),
            evidence_refs: vec!["EvidenceRef:issue-ready".to_string()],
            artifact_refs: vec!["ArtifactRef:context-pack".to_string()],
            expected_outputs: Vec::new(),
            evidence_policy: None,
            idempotency_key: format!("work:AF-653:start:{command_id}"),
            created_at: "2026-06-20T00:00:00Z".to_string(),
        }
    }

    fn write_trusted_capability_registry(
        root: &std::path::Path,
        provider_statuses: &[serde_json::Value],
        provider_smoke_artifacts: &[serde_json::Value],
    ) {
        let statuses = provider_statuses
            .iter()
            .cloned()
            .map(serde_json::from_value::<McpProviderStatus>)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        let smoke = provider_smoke_artifacts
            .iter()
            .cloned()
            .map(serde_json::from_value::<McpProviderSmokeArtifact>)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        let registry = build_capability_registry_with_smoke(&statuses, &smoke);
        let registry_path = root.join(".agentflow/runtime/capability-registry.json");
        fs::create_dir_all(registry_path.parent().unwrap()).unwrap();
        fs::write(
            registry_path,
            serde_json::to_string_pretty(&registry).unwrap() + "\n",
        )
        .unwrap();
    }

    #[test]
    fn valid_command_maps_to_action_proposal() {
        let proposal = map_command_to_action_proposal(&request("submitRequirement")).unwrap();
        assert_eq!(proposal.action_type, "submitRequirement");
        assert_eq!(proposal.idempotency_key, "idem-submitRequirement");
    }

    #[test]
    fn command_surface_aliases_map_to_supported_action_contracts() {
        for command_type in [
            "submitRequirement",
            "createIssue",
            "markIssueDone",
            "requestAudit",
            "acceptDelivery",
            "createFollowUp",
        ] {
            let report = validate_runtime_command(&legacy_request(command_type));
            assert!(!report.valid, "{command_type} must not be Core authority");
            assert!(report
                .errors
                .iter()
                .any(|error| error.message.contains("Core route action contract")));
        }
    }

    #[test]
    fn invalid_command_returns_invalid_command() {
        let dir = tempdir().unwrap();
        let response = execute_command_via_arbitration_with_context(
            dir.path(),
            &request("unknownCommand"),
            &build_core_arbitration_context().unwrap(),
        )
        .unwrap();
        assert_eq!(response.status, RuntimeCommandStatus::InvalidCommand);
        assert!(!response.rejected_reasons.is_empty());
        let decision = load_runtime_decision_fact(dir.path(), &response.proposal_id).unwrap();
        assert_eq!(decision.status, "invalid-command");
    }

    #[test]
    fn governance_rejects_before_writing_proposal_or_accepted_action() {
        let dir = tempdir().unwrap();
        let mut request = project_request("cmd-governance-reject");
        request.input["auditSidecarMode"] = json!("bound-to-main-chain");

        let response = execute_command_via_arbitration_with_context(
            dir.path(),
            &request,
            &build_core_arbitration_context().unwrap(),
        )
        .unwrap();
        assert_eq!(response.status, RuntimeCommandStatus::Rejected);
        assert_eq!(
            response
                .governance_admission
                .as_ref()
                .unwrap()
                .decision
                .as_str(),
            "rejected"
        );
        assert!(load_runtime_command_fact(dir.path(), &request.command_id).is_ok());
        assert!(load_runtime_proposal_facts(dir.path()).unwrap().is_empty());
        assert!(load_runtime_accepted_action_facts(dir.path())
            .unwrap()
            .is_empty());
        let decision = load_runtime_decision_fact(dir.path(), &response.proposal_id).unwrap();
        assert_eq!(decision.status, "rejected");
        assert!(decision.governance_admission.is_some());
        assert!(decision
            .rejected_reasons
            .iter()
            .any(|reason| reason.contains("audit-sidecar-policy")));
    }

    #[test]
    fn governance_defers_before_writing_proposal_or_accepted_action() {
        let dir = tempdir().unwrap();
        let mut request = project_request("cmd-governance-defer");
        request.input["governanceWorkerId"] = json!("github");
        request.input["governanceCommand"] = json!("repo.read");

        let response = execute_command_via_arbitration_with_context(
            dir.path(),
            &request,
            &build_core_arbitration_context().unwrap(),
        )
        .unwrap();
        assert_eq!(response.status, RuntimeCommandStatus::Deferred);
        assert_eq!(
            response
                .governance_admission
                .as_ref()
                .unwrap()
                .decision
                .as_str(),
            "deferred"
        );
        assert!(load_runtime_proposal_facts(dir.path()).unwrap().is_empty());
        assert!(load_runtime_accepted_action_facts(dir.path())
            .unwrap()
            .is_empty());
        let decision = load_runtime_decision_fact(dir.path(), &response.proposal_id).unwrap();
        assert_eq!(decision.status, "deferred");
        assert!(decision.governance_admission.is_some());
    }

    #[test]
    fn governance_rejects_missing_skill_before_writing_proposal() {
        let dir = tempdir().unwrap();
        let mut request = work_run_request("cmd-missing-skill");
        request.skill_ref = None;

        let response = execute_command_via_arbitration_with_context(
            dir.path(),
            &request,
            &build_core_arbitration_context().unwrap(),
        )
        .unwrap();

        assert_eq!(response.status, RuntimeCommandStatus::Rejected);
        let report = response.governance_admission.as_ref().unwrap();
        assert_eq!(report.skill_registry_policy.reason, "missingSkillRef");
        assert!(load_runtime_proposal_facts(dir.path()).unwrap().is_empty());
        assert!(load_runtime_accepted_action_facts(dir.path())
            .unwrap()
            .is_empty());
    }

    #[test]
    fn governance_rejects_unauthorized_skill_owner_before_writing_proposal() {
        let dir = tempdir().unwrap();
        let mut request = work_run_request("cmd-wrong-skill-owner");
        request.skill_ref = Some("core:audit-agent:work-execution-skill".to_string());

        let response = execute_command_via_arbitration_with_context(
            dir.path(),
            &request,
            &build_core_arbitration_context().unwrap(),
        )
        .unwrap();

        assert_eq!(response.status, RuntimeCommandStatus::Rejected);
        let report = response.governance_admission.as_ref().unwrap();
        assert_eq!(report.skill_registry_policy.reason, "skillOwnerMismatch");
        assert!(load_runtime_proposal_facts(dir.path()).unwrap().is_empty());
        assert!(load_runtime_accepted_action_facts(dir.path())
            .unwrap()
            .is_empty());
    }

    #[test]
    fn governance_rejects_invalid_target_object_before_writing_proposal() {
        let dir = tempdir().unwrap();
        let mut request = work_run_request("cmd-invalid-object");
        request.target_object_ref = Some(target_ref("Ghost", "AF-653"));

        let response = execute_command_via_arbitration_with_context(
            dir.path(),
            &request,
            &build_core_arbitration_context().unwrap(),
        )
        .unwrap();

        assert_eq!(response.status, RuntimeCommandStatus::Rejected);
        let report = response.governance_admission.as_ref().unwrap();
        assert_eq!(report.skill_registry_policy.reason, "invalidObjectType");
        assert!(load_runtime_proposal_facts(dir.path()).unwrap().is_empty());
        assert!(load_runtime_accepted_action_facts(dir.path())
            .unwrap()
            .is_empty());
    }

    #[test]
    fn governance_rejects_forbidden_surface_before_writing_proposal() {
        let dir = tempdir().unwrap();
        let mut request = work_run_request("cmd-forbidden-surface");
        request.source_surface = ActionSourceSurface::Desktop;

        let response = execute_command_via_arbitration_with_context(
            dir.path(),
            &request,
            &build_core_arbitration_context().unwrap(),
        )
        .unwrap();

        assert_eq!(response.status, RuntimeCommandStatus::Rejected);
        let report = response.governance_admission.as_ref().unwrap();
        assert_eq!(
            report.skill_registry_policy.reason,
            "sourceSurfaceNotAllowed"
        );
        assert!(load_runtime_proposal_facts(dir.path()).unwrap().is_empty());
        assert!(load_runtime_accepted_action_facts(dir.path())
            .unwrap()
            .is_empty());
    }

    #[test]
    fn governance_defers_missing_required_evidence_before_writing_proposal() {
        let dir = tempdir().unwrap();
        let mut request = work_run_request("cmd-missing-evidence");
        request.evidence_refs = Vec::new();
        request.artifact_refs = Vec::new();

        let response = execute_command_via_arbitration_with_context(
            dir.path(),
            &request,
            &build_core_arbitration_context().unwrap(),
        )
        .unwrap();

        assert_eq!(response.status, RuntimeCommandStatus::Deferred);
        let report = response.governance_admission.as_ref().unwrap();
        assert_eq!(
            report.skill_registry_policy.reason,
            "missingRequiredEvidence"
        );
        assert!(load_runtime_proposal_facts(dir.path()).unwrap().is_empty());
        assert!(load_runtime_accepted_action_facts(dir.path())
            .unwrap()
            .is_empty());
    }

    #[test]
    fn governance_rejects_failed_provider_smoke_before_arbitration() {
        let dir = tempdir().unwrap();
        let mut request = project_request("cmd-governance-smoke-reject");
        let provider_status = json!({
            "version": "agentflow-mcp-provider.v1",
            "provider": "codex",
            "kind": "codex",
            "status": "ready",
            "capabilities": [
                { "name": "launch", "available": true, "detail": null },
                { "name": "codex.exec", "available": true, "detail": null }
            ],
            "cli": "codex",
            "installed": true,
            "authenticated": null,
            "repoPermissionChecked": false,
            "repoPermission": null,
            "checkedAt": 1,
            "errors": [],
            "warnings": []
        });
        let provider_smoke = json!({
            "version": "agentflow-mcp-provider-smoke.v1",
            "provider": "codex",
            "outcome": "failed",
            "reason": "provider codex smoke gate failed",
            "health": provider_status,
            "launchRequestPath": null,
            "sessionId": null,
            "sessionSnapshotPath": null,
            "sessionSnapshotReadable": false,
            "terminalStatus": null,
            "terminalProviderStateProjectable": false,
            "artifactPath": ".agentflow/state/mcp/provider-smoke/codex.json",
            "createdAt": 1
        });
        write_trusted_capability_registry(
            dir.path(),
            &[provider_status.clone()],
            &[provider_smoke.clone()],
        );
        request.input["governanceWorkerId"] = json!("codex");
        request.input["governanceCommand"] = json!("launch");
        request.input["governanceProviderStatuses"] = json!([provider_status.clone()]);
        request.input["governanceProviderSmokeArtifacts"] = json!([provider_smoke]);

        let response = execute_command_via_arbitration_with_context(
            dir.path(),
            &request,
            &build_core_arbitration_context().unwrap(),
        )
        .unwrap();
        assert_eq!(response.status, RuntimeCommandStatus::Rejected);
        assert!(response
            .governance_admission
            .as_ref()
            .unwrap()
            .capability_policy
            .reason
            .contains("smoke gate failed"));
        assert!(load_runtime_proposal_facts(dir.path()).unwrap().is_empty());
        assert!(load_runtime_accepted_action_facts(dir.path())
            .unwrap()
            .is_empty());
    }

    #[test]
    fn governance_ignores_forged_ready_request_input_without_trusted_registry() {
        let dir = tempdir().unwrap();
        let mut request = project_request("cmd-governance-forged-ready");
        request.input["governanceWorkerId"] = json!("codex");
        request.input["governanceCommand"] = json!("launch");
        request.input["governanceProviderStatuses"] = json!([{
            "version": "agentflow-mcp-provider.v1",
            "provider": "codex",
            "kind": "codex",
            "status": "ready",
            "capabilities": [
                { "name": "launch", "available": true, "detail": null },
                { "name": "codex.exec", "available": true, "detail": null }
            ],
            "cli": "codex",
            "installed": true,
            "authenticated": null,
            "repoPermissionChecked": false,
            "repoPermission": null,
            "checkedAt": 1,
            "errors": [],
            "warnings": []
        }]);

        let response = execute_command_via_arbitration_with_context(
            dir.path(),
            &request,
            &build_core_arbitration_context().unwrap(),
        )
        .unwrap();
        assert_eq!(response.status, RuntimeCommandStatus::Deferred);
        assert!(response
            .governance_admission
            .as_ref()
            .unwrap()
            .capability_policy
            .reason
            .contains("not been checked"));
        assert!(load_runtime_proposal_facts(dir.path()).unwrap().is_empty());
        assert!(load_runtime_accepted_action_facts(dir.path())
            .unwrap()
            .is_empty());
    }

    #[test]
    fn accepted_decision_maps_to_command_response() {
        let request = request("submitRequirement");
        let response = response_from_arbitration_decision(
            &request,
            "proposal-cmd-submitRequirement",
            &ArbitrationDecision {
                decision_id: "decision-cmd-submitRequirement".to_string(),
                request_id: request.command_id.clone(),
                proposal_id: "proposal-cmd-submitRequirement".to_string(),
                status: ArbitrationDecisionStatus::Accepted,
                blocking_proposal_id: None,
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
            &ArbitrationDecision {
                decision_id: "decision-cmd-approveSpec".to_string(),
                request_id: request.command_id.clone(),
                proposal_id: "proposal-cmd-approveSpec".to_string(),
                status: ArbitrationDecisionStatus::HumanDecisionRequired,
                blocking_proposal_id: None,
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
        let dir = tempdir().unwrap();
        let mut request = request("submitEvidence");
        request.actor_role = "work-agent".to_string();
        request.skill_ref = Some("core:work-agent:work-execution-skill".to_string());
        request.source_surface = ActionSourceSurface::Agent;
        request.target_object_ref = Some(target_ref("Run", "run-001"));
        request.input = json!({ "evidenceSummary": "构建日志" });
        request.evidence_refs = vec!["EvidenceRef:missing-log".to_string()];
        request.artifact_refs = vec!["ArtifactRef:patch-summary".to_string()];

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

        let response =
            execute_command_via_arbitration_with_context(dir.path(), &request, &context).unwrap();
        assert_eq!(response.status, RuntimeCommandStatus::Rejected);
        assert!(!response.rejected_reasons.is_empty());
        assert!(load_runtime_command_fact(dir.path(), &request.command_id).is_ok());
        assert!(load_runtime_proposal_fact(dir.path(), &response.proposal_id).is_ok());
        assert!(load_runtime_decision_fact(dir.path(), &response.proposal_id).is_ok());
        if let Some(accepted_action_id) = response.accepted_action_id.as_deref() {
            assert!(load_runtime_accepted_action_fact(dir.path(), accepted_action_id).is_err());
        }
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
        let dir = tempdir().unwrap();
        let mut request = request("recordDecision");
        request.actor_role = "human-owner".to_string();
        request.skill_ref = Some("core:human-owner:human-decision-skill".to_string());
        request.input = json!({
            "outcome": "approve",
            "targetObjectType": "Spec",
            "targetObjectId": "AF-325"
        });
        request.evidence_refs = vec!["DecisionRef:human-confirmation-1".to_string()];

        let mut context = build_core_arbitration_context().unwrap();
        context.insert_evidence(EvidenceFact {
            evidence_ref: "DecisionRef:human-confirmation-1".to_string(),
            evidence_type: "humanConfirmation".to_string(),
        });

        let response =
            execute_command_via_arbitration_with_context(dir.path(), &request, &context).unwrap();
        assert_ne!(response.status, RuntimeCommandStatus::InvalidCommand);
    }

    #[test]
    fn accepted_command_writes_durable_runtime_records() {
        let dir = tempdir().unwrap();
        let request = RuntimeCommandRequest {
            command_id: "cmd-create-project-runtime-records".to_string(),
            command_type: CORE_RUNTIME_COMMAND_TYPE.to_string(),
            route: Some(core_runtime_route(
                "core:project.create",
                "action-contract:project.create",
                Some("Spec"),
            )),
            source_surface: ActionSourceSurface::Agent,
            actor_role: "spec-agent".to_string(),
            skill_ref: Some("core:spec-agent:project.create".to_string()),
            target_object_ref: Some(target_ref("Spec", "spec-001")),
            input: json!({
                "projectId": "project-001",
                "projectTitle": "Project Runtime Records"
            }),
            evidence_refs: vec![
                "DecisionRef:approved-spec-1".to_string(),
                "EvidenceRef:human-confirmation-1".to_string(),
            ],
            artifact_refs: vec![
                "ArtifactRef:.agentflow/spec/requirements/req-001/preview.json".to_string(),
            ],
            expected_outputs: Vec::new(),
            evidence_policy: None,
            idempotency_key: "spec:req-001:project:project-001:createProject:2026-06-20T00:00:00Z"
                .to_string(),
            created_at: "2026-06-20T00:00:00Z".to_string(),
        };

        let mut context = build_core_arbitration_context().unwrap();
        context.insert_state(StateFact {
            object_type: "Spec".to_string(),
            object_id: "spec-001".to_string(),
            state_id: "approved".to_string(),
        });
        context.insert_evidence(EvidenceFact {
            evidence_ref: "DecisionRef:approved-spec-1".to_string(),
            evidence_type: "approvedSpecAvailable".to_string(),
        });
        context.insert_evidence(EvidenceFact {
            evidence_ref: "EvidenceRef:human-confirmation-1".to_string(),
            evidence_type: "humanConfirmation".to_string(),
        });

        let response =
            execute_command_via_arbitration_with_context(dir.path(), &request, &context).unwrap();
        assert_eq!(response.status, RuntimeCommandStatus::Accepted);
        let command = load_runtime_command_fact(dir.path(), &request.command_id).unwrap();
        let proposal = load_runtime_proposal_fact(dir.path(), &response.proposal_id).unwrap();
        let decision = load_runtime_decision_fact(dir.path(), &response.proposal_id).unwrap();
        let action = load_runtime_accepted_action_fact(
            dir.path(),
            response.accepted_action_id.as_deref().unwrap(),
        )
        .unwrap();
        assert_eq!(command.command_id, request.command_id);
        assert_eq!(proposal.action_type, "createProject");
        assert_eq!(decision.status, "accepted");
        assert_eq!(action.event_type.as_deref(), Some("ProjectCreated"));
        assert!(action.event_path.is_some());
    }

    #[test]
    fn project_context_queues_command_when_runtime_object_lock_is_active() {
        let dir = tempdir().unwrap();
        let requested = agentflow_event_store::append_task_event(
            dir.path(),
            TaskEventDraft {
                flow_type: WorkflowFlowType::Work,
                aggregate_type: "issue".to_string(),
                aggregate_id: "AF-LOCK-001".to_string(),
                project_id: Some("project-locks".to_string()),
                issue_id: Some("AF-LOCK-001".to_string()),
                run_id: Some("run-001".to_string()),
                event_type: "agent.launch.requested".to_string(),
                authority_role: Some(WorkflowAgentRole::WorkAgent),
                actor: EventActor {
                    role: "task-loop".to_string(),
                    kind: "system".to_string(),
                },
                state: None,
                correlation_id: Some("corr-AF-LOCK-001".to_string()),
                causation_id: None,
                payload: json!({ "provider": "codex" }),
                artifact_refs: Vec::new(),
                idempotency_key: Some("agent.launch.requested:AF-LOCK-001:run-001".to_string()),
            },
        )
        .unwrap();
        claim_task_event(
            dir.path(),
            "agent-dispatcher",
            |event, _events| event.event_id == requested.event_id,
            |event, _events| {
                Ok(TaskEventDraft {
                    flow_type: WorkflowFlowType::Work,
                    aggregate_type: "issue".to_string(),
                    aggregate_id: event.aggregate_id.clone(),
                    project_id: event.project_id.clone(),
                    issue_id: event.issue_id.clone(),
                    run_id: event.run_id.clone(),
                    event_type: "agent.launch.claimed".to_string(),
                    authority_role: event.authority_role,
                    actor: EventActor {
                        role: "agent-dispatcher".to_string(),
                        kind: "system".to_string(),
                    },
                    state: None,
                    correlation_id: Some(event.correlation_id.clone()),
                    causation_id: Some(event.event_id.clone()),
                    payload: json!({}),
                    artifact_refs: Vec::new(),
                    idempotency_key: Some("agent.launch.claimed:AF-LOCK-001:run-001".to_string()),
                })
            },
        )
        .unwrap()
        .unwrap();

        write_runtime_accepted_action_fact(
            dir.path(),
            &RuntimeAcceptedActionFact {
                version: RUNTIME_ACCEPTED_ACTION_FACT_VERSION.to_string(),
                command_id: "cmd-lock-seed".to_string(),
                proposal_id: "proposal-cmd-lock-seed".to_string(),
                accepted_action_id: "accepted-proposal-cmd-lock-seed".to_string(),
                issue_id: Some("AF-LOCK-001".to_string()),
                run_id: Some("run-001".to_string()),
                action_type: "createProject".to_string(),
                actor_role: "spec-agent".to_string(),
                target_object_ref: Some(target_ref("Spec", "spec-001")),
                from_state: Some("approved".to_string()),
                to_state: Some("projectCreated".to_string()),
                evidence_refs: vec![
                    "DecisionRef:approved-spec-1".to_string(),
                    "EvidenceRef:human-confirmation-1".to_string(),
                ],
                artifact_refs: Vec::new(),
                expected_events: vec!["ProjectCreated".to_string()],
                lock_plan: ObjectLockPlan {
                    acquire: vec![ObjectLock {
                        lock_id: "lock-proposal-cmd-lock-seed".to_string(),
                        object_type: "Spec".to_string(),
                        object_id: "spec-001".to_string(),
                        lock_kind: ObjectLockKind::DecisionPending,
                        owner_proposal_id: "proposal-cmd-lock-seed".to_string(),
                        owner_role: "spec-agent".to_string(),
                        expires_at: None,
                        reason: "accepted action `createProject`".to_string(),
                    }],
                    release: Vec::new(),
                },
                definition_versions: DefinitionVersions {
                    ontology_version: "v1".to_string(),
                    contract_version: "v1".to_string(),
                    role_policy_version: "v1".to_string(),
                    object_state_version: "v1".to_string(),
                },
                event_id: None,
                event_path: None,
                event_type: Some("ProjectCreated".to_string()),
                recorded_at: 1,
            },
        )
        .unwrap();

        let request = RuntimeCommandRequest {
            command_id: "cmd-create-project-lock-2".to_string(),
            command_type: CORE_RUNTIME_COMMAND_TYPE.to_string(),
            route: Some(core_runtime_route(
                "core:project.create",
                "action-contract:project.create",
                Some("Spec"),
            )),
            source_surface: ActionSourceSurface::Agent,
            actor_role: "spec-agent".to_string(),
            skill_ref: Some("core:spec-agent:project.create".to_string()),
            target_object_ref: Some(target_ref("Spec", "spec-001")),
            input: json!({
                "projectId": "project-002",
                "projectTitle": "Locked Project"
            }),
            evidence_refs: vec![
                "DecisionRef:approved-spec-1".to_string(),
                "EvidenceRef:human-confirmation-1".to_string(),
            ],
            artifact_refs: vec![
                "ArtifactRef:.agentflow/spec/requirements/req-001/preview.json".to_string(),
            ],
            expected_outputs: Vec::new(),
            evidence_policy: None,
            idempotency_key: "spec:spec-001:project:project-002:createProject:2026-06-21T00:05:00Z"
                .to_string(),
            created_at: "2026-06-21T00:05:00Z".to_string(),
        };
        let project_context = build_project_arbitration_context(dir.path()).unwrap();
        let mut second_context = build_core_arbitration_context().unwrap();
        second_context.insert_state(StateFact {
            object_type: "Spec".to_string(),
            object_id: "spec-001".to_string(),
            state_id: "approved".to_string(),
        });
        second_context.insert_evidence(EvidenceFact {
            evidence_ref: "DecisionRef:approved-spec-1".to_string(),
            evidence_type: "approvedSpecAvailable".to_string(),
        });
        second_context.insert_evidence(EvidenceFact {
            evidence_ref: "EvidenceRef:human-confirmation-1".to_string(),
            evidence_type: "humanConfirmation".to_string(),
        });
        for lock in project_context.object_locks {
            second_context.push_lock(lock);
        }
        let second =
            execute_command_via_arbitration_with_context(dir.path(), &request, &second_context)
                .unwrap();
        assert_eq!(second.status, RuntimeCommandStatus::Queued);
        assert_eq!(
            second.decision,
            crate::responses::RuntimeCommandDecision::Queued
        );
        assert!(second
            .rejected_reasons
            .iter()
            .any(|reason| reason.message.contains("active DecisionPending lock")));
    }
}
