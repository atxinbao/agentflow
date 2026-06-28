//! AgentFlow dry-run simulation boundary.
//!
//! Simulation turns command / issue / completion intent into an in-memory
//! report. It intentionally does not prepare runtime workspaces, append events,
//! write authority files, rebuild projections, or launch providers.

use agentflow_action_arbitration::{
    arbitrate_action, proposal_conflict_scope_key, ArbitrationContext, ArbitrationDecision,
    ArbitrationDecisionStatus, ArbitrationRequest, DefinitionVersions, RejectionReason,
};
use agentflow_action_contract::{
    core_action_contract_registry, ActionContract, ActionExpectedEvent, ActionProposal,
};
use agentflow_object_state::core_object_state_registry;
use agentflow_ontology::software_dev_reference_ontology_registry;
use agentflow_pack::{
    ConnectorSupportedAction, DomainActionSemantic, PackConnectorDefinition, PackDomainDefinition,
    PackSurfaceDefinition, PackValidationArtifact, SurfaceCommandEntryMapping,
};
use agentflow_role_policy::core_role_policy_registry;
use agentflow_runtime_api::{
    map_command_to_action_proposal, validate_runtime_command, RuntimeCommandError,
    RuntimeCommandRequest, RuntimeCommandValidationReport, RuntimeQueryHint,
};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub const SIMULATION_REPORT_VERSION: &str = "agentflow-simulation-report.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SimulationKind {
    Command,
    Issue,
    Completion,
    PackCommand,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SimulationDecision {
    Accepted,
    Rejected,
    HumanDecisionRequired,
    Queued,
    Superseded,
    Cancelled,
    ConflictDetected,
    InvalidCommand,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SimulationRiskLevel {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimulationIssueRequest {
    pub issue_id: String,
    pub actor_role: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    #[serde(default)]
    pub dependency_ids: Vec<String>,
    #[serde(default)]
    pub context_pack_ready: bool,
    #[serde(default)]
    pub workspace_clean: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimulationCompletionRequest {
    pub issue_id: String,
    pub run_id: String,
    pub actor_role: String,
    #[serde(default)]
    pub validation_evidence_refs: Vec<String>,
    #[serde(default)]
    pub delivery_artifact_refs: Vec<String>,
    #[serde(default)]
    pub pr_or_mr_ref: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackCommandSimulationRequest {
    pub simulation_id: String,
    pub command: String,
    pub target_object_type: String,
    pub target_object_id: String,
    pub actor_role: String,
    pub validation: PackValidationArtifact,
    pub domain: PackDomainDefinition,
    pub surface: PackSurfaceDefinition,
    pub connector: PackConnectorDefinition,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimulationExpectedEvent {
    pub event_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object_type: Option<String>,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub payload_fields: Vec<String>,
}

impl From<&ActionExpectedEvent> for SimulationExpectedEvent {
    fn from(event: &ActionExpectedEvent) -> Self {
        Self {
            event_type: event.event_type.clone(),
            object_type: event.object_type.clone(),
            required: event.required,
            payload_fields: event.payload_fields.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimulationRejectedReason {
    pub code: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

impl From<&RuntimeCommandError> for SimulationRejectedReason {
    fn from(error: &RuntimeCommandError) -> Self {
        Self {
            code: format!("{:?}", error.code),
            message: error.message.clone(),
            detail: error.path.clone(),
        }
    }
}

impl From<&RejectionReason> for SimulationRejectedReason {
    fn from(reason: &RejectionReason) -> Self {
        Self {
            code: format!("{:?}", reason.code),
            message: reason.message.clone(),
            detail: reason.detail.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimulationAffectedProjection {
    pub projection_id: String,
    pub reason: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimulationAffectedObject {
    pub object_type: String,
    pub object_id: String,
    pub impact: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_state: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_state: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimulationRequiredEvidence {
    pub evidence_type: String,
    pub required: bool,
    pub status: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimulationStateTransition {
    pub object_type: String,
    pub object_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_state: Option<String>,
    pub to_state: String,
    pub trigger: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimulationDownstreamTrigger {
    pub trigger_id: String,
    pub target: String,
    pub status: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimulationConflict {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope_key: Option<String>,
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blocking_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimulationGateImpact {
    pub gate_id: String,
    pub status: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimulationCompletionCommitPreview {
    pub issue_id: String,
    pub run_id: String,
    pub requires_validation_evidence: bool,
    pub requires_delivery_artifacts: bool,
    pub requires_merge_proof: bool,
    #[serde(default)]
    pub expected_event_chain: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimulationReport {
    pub version: String,
    pub simulation_id: String,
    pub kind: SimulationKind,
    pub decision: SimulationDecision,
    pub writes_authority: bool,
    pub writes_event_store: bool,
    pub executes_provider: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command_validation: Option<RuntimeCommandValidationReport>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proposal: Option<ActionProposal>,
    #[serde(default)]
    pub expected_events: Vec<SimulationExpectedEvent>,
    #[serde(default)]
    pub rejected_reasons: Vec<SimulationRejectedReason>,
    #[serde(default)]
    pub affected_objects: Vec<SimulationAffectedObject>,
    #[serde(default)]
    pub affected_projections: Vec<SimulationAffectedProjection>,
    #[serde(default)]
    pub required_evidence: Vec<SimulationRequiredEvidence>,
    #[serde(default)]
    pub state_transitions: Vec<SimulationStateTransition>,
    #[serde(default)]
    pub downstream_triggers: Vec<SimulationDownstreamTrigger>,
    pub risk: SimulationRiskLevel,
    #[serde(default)]
    pub conflicts: Vec<SimulationConflict>,
    #[serde(default)]
    pub gate_impact: Vec<SimulationGateImpact>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completion_commit: Option<SimulationCompletionCommitPreview>,
    #[serde(default)]
    pub metadata: Value,
}

impl SimulationReport {
    fn readonly(
        simulation_id: impl Into<String>,
        kind: SimulationKind,
        decision: SimulationDecision,
    ) -> Self {
        Self {
            version: SIMULATION_REPORT_VERSION.to_string(),
            simulation_id: simulation_id.into(),
            kind,
            decision,
            writes_authority: false,
            writes_event_store: false,
            executes_provider: false,
            command_validation: None,
            proposal: None,
            expected_events: Vec::new(),
            rejected_reasons: Vec::new(),
            affected_objects: Vec::new(),
            affected_projections: Vec::new(),
            required_evidence: Vec::new(),
            state_transitions: Vec::new(),
            downstream_triggers: Vec::new(),
            risk: SimulationRiskLevel::Low,
            conflicts: Vec::new(),
            gate_impact: Vec::new(),
            completion_commit: None,
            metadata: json!({}),
        }
    }
}

pub fn simulate_command(request: &RuntimeCommandRequest) -> Result<SimulationReport> {
    let validation = validate_runtime_command(request);
    let mut report = SimulationReport::readonly(
        format!("simulation-{}", request.command_id),
        SimulationKind::Command,
        SimulationDecision::InvalidCommand,
    );
    report.command_validation = Some(validation.clone());
    report
        .affected_projections
        .push(affected_projection(runtime_hint(request)));
    if let Some(target) = &request.target_object_ref {
        report.affected_objects.push(affected_object(
            &target.object_type,
            &target.id,
            format!(
                "command {} would propose an action for this object",
                request.command_type
            ),
            None,
            Some("action-proposed"),
        ));
        report.state_transitions.push(state_transition(
            &target.object_type,
            &target.id,
            None,
            "action-proposed",
            &request.command_type,
        ));
    }

    if !validation.valid {
        report.rejected_reasons = validation.errors.iter().map(Into::into).collect();
        report.risk = SimulationRiskLevel::Medium;
        report.gate_impact.push(SimulationGateImpact {
            gate_id: "runtime.command.validation".to_string(),
            status: "rejected".to_string(),
            reason: "command failed structural validation".to_string(),
        });
        return Ok(report);
    }

    let context = core_simulation_context()?;
    let proposal = map_command_to_action_proposal(request)?;
    let contract = contract_for_proposal(&context, &proposal)?;
    let arbitration_request = ArbitrationRequest {
        request_id: format!("simulate-{}", request.command_id),
        proposal: proposal.clone(),
        core_admission: None,
        definition_versions: definition_versions(&context),
        requested_at: request.created_at.clone(),
    };
    let decision = arbitrate_action(&arbitration_request, &context);
    apply_arbitration_result(&mut report, &context, proposal, contract, &decision);
    Ok(report)
}

pub fn simulate_issue(request: &SimulationIssueRequest) -> SimulationReport {
    let mut report = SimulationReport::readonly(
        format!("simulation-issue-{}", request.issue_id),
        SimulationKind::Issue,
        SimulationDecision::Accepted,
    );
    report.expected_events = vec![
        expected_event("issue.todo.checked", Some("Issue"), true),
        expected_event("task.context-pack.checked", Some("Issue"), true),
        expected_event("task.runtime-preflight.checked", Some("Issue"), true),
    ];
    report.affected_objects = vec![affected_object(
        "Issue",
        &request.issue_id,
        "issue launch simulation can move the task toward execution readiness",
        Some("todo"),
        Some("in_progress"),
    )];
    if let Some(project_id) = &request.project_id {
        report.affected_objects.push(affected_object(
            "Project",
            project_id,
            "issue schedulability can change project loop progress",
            None,
            Some("progress-updated"),
        ));
    }
    report.affected_projections = vec![
        SimulationAffectedProjection {
            projection_id: "projection.task".to_string(),
            target_id: Some(request.issue_id.clone()),
            reason: "issue simulation can refresh task timeline readiness".to_string(),
        },
        SimulationAffectedProjection {
            projection_id: "projection.project".to_string(),
            target_id: request.project_id.clone(),
            reason: "issue readiness can change project schedulability".to_string(),
        },
    ];
    report.required_evidence = vec![
        required_evidence(
            "panel-context-pack",
            true,
            if request.context_pack_ready {
                "available"
            } else {
                "missing"
            },
            "Build Agent needs a prepared project context before execution",
        ),
        required_evidence(
            "workspace-clean-proof",
            true,
            if request.workspace_clean {
                "available"
            } else {
                "missing"
            },
            "Runtime preflight must prove there are no uncommitted user changes",
        ),
    ];
    report.state_transitions = vec![state_transition(
        "Issue",
        &request.issue_id,
        Some("todo"),
        "in_progress",
        "runtime-preflight.passed",
    )];
    report.downstream_triggers = vec![downstream_trigger(
        "build-agent.launch.requested",
        &request.issue_id,
        if request.context_pack_ready && request.workspace_clean {
            "would-trigger"
        } else {
            "blocked"
        },
        "launch is only allowed after context pack and workspace preflight pass",
    )];

    if !request.dependency_ids.is_empty() {
        report.risk = SimulationRiskLevel::Medium;
        report.gate_impact.push(SimulationGateImpact {
            gate_id: "issue.dependencies".to_string(),
            status: "requires-check".to_string(),
            reason: format!(
                "simulation must confirm {} dependency issue(s) before launch",
                request.dependency_ids.len()
            ),
        });
    }
    if !request.context_pack_ready {
        report.decision = SimulationDecision::Rejected;
        report.risk = SimulationRiskLevel::High;
        report.rejected_reasons.push(simple_rejection(
            "ContextPackMissing",
            "context pack is not ready",
        ));
        report.gate_impact.push(SimulationGateImpact {
            gate_id: "panel.context-pack.ready".to_string(),
            status: "rejected".to_string(),
            reason: "issue cannot enter execution without context pack".to_string(),
        });
    }
    if !request.workspace_clean {
        report.decision = SimulationDecision::Rejected;
        report.risk = SimulationRiskLevel::High;
        report
            .rejected_reasons
            .push(simple_rejection("WorkspaceDirty", "workspace is not clean"));
        report.gate_impact.push(SimulationGateImpact {
            gate_id: "runtime.workspace.clean".to_string(),
            status: "rejected".to_string(),
            reason: "issue cannot enter execution with uncommitted user changes".to_string(),
        });
    }
    report.metadata = json!({
        "actorRole": request.actor_role,
        "createdAt": request.created_at,
        "dependencyIds": request.dependency_ids,
    });
    report
}

pub fn simulate_completion(request: &SimulationCompletionRequest) -> SimulationReport {
    let mut report = SimulationReport::readonly(
        format!(
            "simulation-completion-{}-{}",
            request.issue_id, request.run_id
        ),
        SimulationKind::Completion,
        SimulationDecision::Accepted,
    );
    report.expected_events = vec![
        expected_event("validation.completed", Some("Run"), true),
        expected_event("delivery.prepared", Some("Issue"), true),
        expected_event("completion.commit.requested", Some("Issue"), true),
        expected_event("completion.commit.accepted", Some("Issue"), true),
        expected_event("issue.done.requested", Some("Issue"), true),
    ];
    report.completion_commit = Some(SimulationCompletionCommitPreview {
        issue_id: request.issue_id.clone(),
        run_id: request.run_id.clone(),
        requires_validation_evidence: true,
        requires_delivery_artifacts: true,
        requires_merge_proof: true,
        expected_event_chain: vec![
            "validation.completed".to_string(),
            "delivery.prepared".to_string(),
            "completion.commit.requested".to_string(),
            "completion.commit.accepted".to_string(),
            "issue.done.requested".to_string(),
        ],
    });
    report.affected_objects = vec![
        affected_object(
            "Issue",
            &request.issue_id,
            "completion simulation can move issue from review to done",
            Some("in_review"),
            Some("done"),
        ),
        affected_object(
            "Run",
            &request.run_id,
            "completion simulation can close the active run",
            Some("completed"),
            Some("archived"),
        ),
    ];
    report.affected_projections = vec![
        SimulationAffectedProjection {
            projection_id: "projection.task".to_string(),
            target_id: Some(request.issue_id.clone()),
            reason: "completion can move task timeline toward done".to_string(),
        },
        SimulationAffectedProjection {
            projection_id: "release.delivery-summary".to_string(),
            target_id: Some(request.issue_id.clone()),
            reason: "completion can expose public delivery summary".to_string(),
        },
    ];
    report.required_evidence = vec![
        required_evidence(
            "validation-evidence",
            true,
            if request.validation_evidence_refs.is_empty() {
                "missing"
            } else {
                "available"
            },
            "done writeback requires validation evidence",
        ),
        required_evidence(
            "delivery-artifact",
            true,
            if request.delivery_artifact_refs.is_empty() {
                "missing"
            } else {
                "available"
            },
            "done writeback requires delivery artifact references",
        ),
        required_evidence(
            "merge-proof",
            true,
            if request.pr_or_mr_ref.is_some() {
                "available"
            } else {
                "missing"
            },
            "done writeback should bind to PR/MR merge proof",
        ),
    ];
    report.state_transitions = vec![state_transition(
        "Issue",
        &request.issue_id,
        Some("in_review"),
        "done",
        "completion.commit.accepted",
    )];
    report.downstream_triggers = vec![
        downstream_trigger(
            "projection.rebuild.requested",
            &request.issue_id,
            "would-trigger",
            "completion changes task and project read models",
        ),
        downstream_trigger(
            "audit.trigger.evaluated",
            &request.issue_id,
            "would-evaluate",
            "audit remains independent and must be evaluated after delivery",
        ),
    ];
    if request.validation_evidence_refs.is_empty() {
        report.decision = SimulationDecision::Rejected;
        report.risk = SimulationRiskLevel::High;
        report.rejected_reasons.push(simple_rejection(
            "ValidationEvidenceMissing",
            "completion requires validation evidence",
        ));
    }
    if request.delivery_artifact_refs.is_empty() {
        report.decision = SimulationDecision::Rejected;
        report.risk = SimulationRiskLevel::High;
        report.rejected_reasons.push(simple_rejection(
            "DeliveryArtifactMissing",
            "completion requires delivery artifact refs",
        ));
    }
    if request.pr_or_mr_ref.is_none() {
        report.risk = SimulationRiskLevel::Medium;
        report.gate_impact.push(SimulationGateImpact {
            gate_id: "provider.merge.proof".to_string(),
            status: "requires-check".to_string(),
            reason: "completion should bind to PR/MR merge proof before done writeback".to_string(),
        });
    }
    report.metadata = json!({
        "actorRole": request.actor_role,
        "createdAt": request.created_at,
        "runId": request.run_id,
        "prOrMrRef": request.pr_or_mr_ref,
    });
    report
}

pub fn simulate_pack_command(request: &PackCommandSimulationRequest) -> SimulationReport {
    let surface_mapping = request
        .surface
        .command_entry_mappings
        .iter()
        .find(|mapping| mapping.command_type == request.command);
    let connector_action = connector_action_for_command(&request.connector, &request.command);
    let domain_action = surface_mapping
        .and_then(|mapping| action_type_from_contract_ref(&mapping.action_contract_ref))
        .and_then(|action_type| domain_action_for_type(&request.domain, action_type));

    let mut report = SimulationReport::readonly(
        request.simulation_id.clone(),
        SimulationKind::PackCommand,
        SimulationDecision::Accepted,
    );
    report.expected_events = pack_expected_events(domain_action, &request.command);
    report.affected_objects = vec![
        affected_object(
            &request.target_object_type,
            &request.target_object_id,
            format!(
                "pack command {} would propose a domain action for this object",
                request.command
            ),
            None,
            Some("action-proposed"),
        ),
        affected_object(
            "Pack",
            &request.validation.pack_id,
            "pack validation and surface mappings are read to evaluate command impact",
            Some(if request.validation.active {
                "active"
            } else {
                "inactive"
            }),
            Some("unchanged"),
        ),
    ];
    report.affected_projections = pack_affected_projections(&request.surface, surface_mapping);
    report.conflicts.push(SimulationConflict {
        scope_key: Some(format!(
            "pack:{}:{}:{}",
            request.validation.pack_id, request.command, request.target_object_id
        )),
        status: "preview-only".to_string(),
        blocking_ref: None,
    });
    report.gate_impact.push(SimulationGateImpact {
        gate_id: "pack.validation.active".to_string(),
        status: if request.validation.active {
            "accepted".to_string()
        } else {
            "rejected".to_string()
        },
        reason: if request.validation.active {
            "pack validation artifact allows simulation".to_string()
        } else {
            "pack validation artifact is not active".to_string()
        },
    });

    if !request.validation.active {
        report.decision = SimulationDecision::Rejected;
        report.risk = SimulationRiskLevel::High;
        for issue in &request.validation.issues {
            report.rejected_reasons.push(simple_rejection(
                "PackValidationFailed",
                &format!("{}: {}", issue.field, issue.reason),
            ));
        }
    }
    if surface_mapping.is_none() {
        report.decision = SimulationDecision::Rejected;
        report.risk = SimulationRiskLevel::High;
        report.rejected_reasons.push(simple_rejection(
            "SurfaceMappingMissing",
            "pack surface does not expose this command",
        ));
        report.gate_impact.push(SimulationGateImpact {
            gate_id: "pack.surface.command-mapping".to_string(),
            status: "rejected".to_string(),
            reason: "missing surface command mapping".to_string(),
        });
    }
    if surface_mapping.is_none() && connector_action.is_none() {
        report.decision = SimulationDecision::Rejected;
        report.risk = SimulationRiskLevel::High;
        report.rejected_reasons.push(simple_rejection(
            "ConnectorActionMissing",
            "pack connector does not expose this command",
        ));
        report.gate_impact.push(SimulationGateImpact {
            gate_id: "pack.connector.action".to_string(),
            status: "rejected".to_string(),
            reason: "missing connector action".to_string(),
        });
    }
    if !request.validation.missing_read_models.is_empty() {
        report.decision = SimulationDecision::Rejected;
        report.risk = SimulationRiskLevel::High;
        report.rejected_reasons.push(simple_rejection(
            "ReadModelMissing",
            "pack surface depends on missing read models",
        ));
        report.gate_impact.push(SimulationGateImpact {
            gate_id: "pack.surface.read-models".to_string(),
            status: "rejected".to_string(),
            reason: format!(
                "{} read model mapping(s) missing",
                request.validation.missing_read_models.len()
            ),
        });
    }

    let required_capabilities = connector_action
        .map(|action| vec![action.required_capability.clone()])
        .unwrap_or_default();
    let required_evidence_names = domain_action
        .map(|action| action.required_evidence.clone())
        .unwrap_or_default();
    report.required_evidence = required_evidence_names
        .iter()
        .map(|evidence| {
            required_evidence(
                evidence,
                true,
                "required-preview",
                format!(
                    "pack command {} declares this evidence requirement",
                    request.command
                ),
            )
        })
        .collect();
    if report.required_evidence.is_empty() {
        report.required_evidence.push(required_evidence(
            "pack-validation-artifact",
            true,
            if request.validation.active {
                "available"
            } else {
                "missing"
            },
            "pack command simulation always depends on validation artifact state",
        ));
    }
    report.state_transitions = vec![state_transition(
        &request.target_object_type,
        &request.target_object_id,
        None,
        "action-proposed",
        &request.command,
    )];
    report.downstream_triggers = report
        .expected_events
        .iter()
        .map(|event| {
            downstream_trigger(
                &event.event_type,
                event
                    .object_type
                    .as_deref()
                    .unwrap_or(&request.target_object_type),
                if event.required {
                    "would-trigger"
                } else {
                    "optional"
                },
                "simulation previews the event chain without appending events",
            )
        })
        .collect();
    report.metadata = json!({
        "command": request.command,
        "targetObject": {
            "objectType": request.target_object_type,
            "id": request.target_object_id,
        },
        "actorRole": request.actor_role,
        "packId": request.validation.pack_id,
        "requiredCapabilities": required_capabilities,
        "requiredEvidence": required_evidence_names,
        "acceptanceImpact": {
            "status": if report.decision == SimulationDecision::Accepted { "would-pass" } else { "would-reject" },
            "reason": if report.decision == SimulationDecision::Accepted {
                "pack command dry-run found surface, connector, read model, and validation coverage"
            } else {
                "pack command dry-run found missing validation, mapping, connector, or read model coverage"
            },
        },
        "createdAt": request.created_at,
    });
    report
}

fn core_simulation_context() -> Result<ArbitrationContext> {
    let ontology = software_dev_reference_ontology_registry();
    let action_contract = core_action_contract_registry(&ontology);
    let role_policy = core_role_policy_registry(&ontology, &action_contract);
    let object_state =
        core_object_state_registry(&ontology, &action_contract).map_err(|report| {
            anyhow!(
                "built-in object state registry failed validation during simulation: {:?}",
                report
            )
        })?;
    Ok(ArbitrationContext::new(
        ontology,
        action_contract,
        role_policy,
        object_state,
    ))
}

fn connector_action_for_command<'a>(
    connector: &'a PackConnectorDefinition,
    command: &str,
) -> Option<&'a ConnectorSupportedAction> {
    connector
        .connectors
        .iter()
        .flat_map(|connector| connector.supported_actions.iter())
        .find(|action| action.command_type == command)
}

fn domain_action_for_type<'a>(
    domain: &'a PackDomainDefinition,
    action_type: &str,
) -> Option<&'a DomainActionSemantic> {
    domain
        .action_semantics
        .iter()
        .find(|action| action.action_type == action_type)
}

fn action_type_from_contract_ref(contract_ref: &str) -> Option<&str> {
    match contract_ref {
        "action-contract:spec.intake" => Some("submitRequirement"),
        "action-contract:issue.start" => Some("startRun"),
        "action-contract:acceptance.evaluate" => Some("runValidation"),
        "action-contract:delivery.open" => Some("prepareDelivery"),
        "action-contract:audit.request" => Some("requestAudit"),
        _ => contract_ref.strip_prefix("action-contract:"),
    }
}

fn pack_expected_events(
    domain_action: Option<&DomainActionSemantic>,
    command: &str,
) -> Vec<SimulationExpectedEvent> {
    let action_type = domain_action
        .map(|action| action.action_type.as_str())
        .unwrap_or(command);
    let object_type = domain_action.map(|action| action.target_object_type.as_str());
    vec![
        expected_event(format!("pack.command.{command}.validated"), None, true),
        expected_event(format!("{action_type}.proposed"), object_type, true),
        expected_event(format!("{action_type}.accepted"), object_type, false),
    ]
}

fn pack_affected_projections(
    surface: &PackSurfaceDefinition,
    surface_mapping: Option<&SurfaceCommandEntryMapping>,
) -> Vec<SimulationAffectedProjection> {
    let Some(surface_mapping) = surface_mapping else {
        return Vec::new();
    };
    surface
        .read_model_dependencies
        .iter()
        .filter(|dependency| dependency.page_id == surface_mapping.page_id)
        .map(|dependency| SimulationAffectedProjection {
            projection_id: dependency.projection_ref.clone(),
            target_id: Some(surface_mapping.page_id.clone()),
            reason: format!(
                "pack command {} can affect page {}",
                surface_mapping.command_type, surface_mapping.page_id
            ),
        })
        .collect()
}

fn definition_versions(context: &ArbitrationContext) -> DefinitionVersions {
    DefinitionVersions {
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
    }
}

fn contract_for_proposal<'a>(
    context: &'a ArbitrationContext,
    proposal: &ActionProposal,
) -> Result<&'a ActionContract> {
    context
        .action_contract_registry
        .get_action_contract(&proposal.action_type, &proposal.contract_version)
        .ok_or_else(|| {
            anyhow!(
                "missing action contract {}@{}",
                proposal.action_type,
                proposal.contract_version
            )
        })
}

fn apply_arbitration_result(
    report: &mut SimulationReport,
    context: &ArbitrationContext,
    proposal: ActionProposal,
    contract: &ActionContract,
    decision: &ArbitrationDecision,
) {
    report.proposal = Some(proposal.clone());
    report.expected_events = contract.expected_events.iter().map(Into::into).collect();
    report.rejected_reasons = decision.rejected_reasons.iter().map(Into::into).collect();
    report.decision = match decision.status {
        ArbitrationDecisionStatus::Accepted => SimulationDecision::Accepted,
        ArbitrationDecisionStatus::Rejected => SimulationDecision::Rejected,
        ArbitrationDecisionStatus::HumanDecisionRequired => {
            SimulationDecision::HumanDecisionRequired
        }
        ArbitrationDecisionStatus::Queued => SimulationDecision::Queued,
        ArbitrationDecisionStatus::Superseded => SimulationDecision::Superseded,
        ArbitrationDecisionStatus::Cancelled => SimulationDecision::Cancelled,
        ArbitrationDecisionStatus::ConflictDetected => SimulationDecision::ConflictDetected,
    };
    report.risk = risk_for_decision(decision);

    let scope_key = proposal_conflict_scope_key(context, &proposal);
    if scope_key.is_some() || decision.blocking_proposal_id.is_some() {
        report.conflicts.push(SimulationConflict {
            scope_key,
            status: format!("{:?}", decision.status),
            blocking_ref: decision.blocking_proposal_id.clone(),
        });
    }
    if !decision.would_emit_events.is_empty() {
        for event in &decision.would_emit_events {
            if !report
                .expected_events
                .iter()
                .any(|expected| expected.event_type == *event)
            {
                report.expected_events.push(expected_event(
                    event,
                    contract.target_object_type.as_deref(),
                    false,
                ));
            }
        }
    }
    report.gate_impact.push(SimulationGateImpact {
        gate_id: "action.arbitration".to_string(),
        status: format!("{:?}", decision.status),
        reason: gate_reason(decision),
    });
}

fn risk_for_decision(decision: &ArbitrationDecision) -> SimulationRiskLevel {
    match decision.status {
        ArbitrationDecisionStatus::Accepted => SimulationRiskLevel::Low,
        ArbitrationDecisionStatus::HumanDecisionRequired
        | ArbitrationDecisionStatus::Queued
        | ArbitrationDecisionStatus::Superseded
        | ArbitrationDecisionStatus::ConflictDetected => SimulationRiskLevel::Medium,
        ArbitrationDecisionStatus::Rejected | ArbitrationDecisionStatus::Cancelled => {
            SimulationRiskLevel::High
        }
    }
}

fn gate_reason(decision: &ArbitrationDecision) -> String {
    match decision.status {
        ArbitrationDecisionStatus::Accepted => "proposal would pass arbitration".to_string(),
        ArbitrationDecisionStatus::HumanDecisionRequired => {
            "proposal would require human decision before write".to_string()
        }
        ArbitrationDecisionStatus::Queued => {
            "proposal would queue behind another claim".to_string()
        }
        ArbitrationDecisionStatus::Superseded => "proposal would be superseded".to_string(),
        ArbitrationDecisionStatus::Cancelled => "proposal would be cancelled".to_string(),
        ArbitrationDecisionStatus::ConflictDetected => "proposal would hit a conflict".to_string(),
        ArbitrationDecisionStatus::Rejected => "proposal would be rejected".to_string(),
    }
}

fn affected_projection(hint: RuntimeQueryHint) -> SimulationAffectedProjection {
    SimulationAffectedProjection {
        projection_id: hint.view,
        target_id: hint.target_id,
        reason: hint.reason,
    }
}

fn runtime_hint(request: &RuntimeCommandRequest) -> RuntimeQueryHint {
    agentflow_runtime_api::mapping::runtime_query_hint_for_command(request)
}

fn expected_event(
    event_type: impl Into<String>,
    object_type: Option<&str>,
    required: bool,
) -> SimulationExpectedEvent {
    SimulationExpectedEvent {
        event_type: event_type.into(),
        object_type: object_type.map(str::to_string),
        required,
        payload_fields: Vec::new(),
    }
}

fn simple_rejection(code: &str, message: &str) -> SimulationRejectedReason {
    SimulationRejectedReason {
        code: code.to_string(),
        message: message.to_string(),
        detail: None,
    }
}

fn affected_object(
    object_type: &str,
    object_id: &str,
    impact: impl Into<String>,
    current_state: Option<&str>,
    next_state: Option<&str>,
) -> SimulationAffectedObject {
    SimulationAffectedObject {
        object_type: object_type.to_string(),
        object_id: object_id.to_string(),
        impact: impact.into(),
        current_state: current_state.map(str::to_string),
        next_state: next_state.map(str::to_string),
    }
}

fn required_evidence(
    evidence_type: &str,
    required: bool,
    status: &str,
    reason: impl Into<String>,
) -> SimulationRequiredEvidence {
    SimulationRequiredEvidence {
        evidence_type: evidence_type.to_string(),
        required,
        status: status.to_string(),
        reason: reason.into(),
    }
}

fn state_transition(
    object_type: &str,
    object_id: &str,
    from_state: Option<&str>,
    to_state: &str,
    trigger: &str,
) -> SimulationStateTransition {
    SimulationStateTransition {
        object_type: object_type.to_string(),
        object_id: object_id.to_string(),
        from_state: from_state.map(str::to_string),
        to_state: to_state.to_string(),
        trigger: trigger.to_string(),
    }
}

fn downstream_trigger(
    trigger_id: &str,
    target: &str,
    status: &str,
    reason: impl Into<String>,
) -> SimulationDownstreamTrigger {
    SimulationDownstreamTrigger {
        trigger_id: trigger_id.to_string(),
        target: target.to_string(),
        status: status.to_string(),
        reason: reason.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agentflow_action_contract::{ActionRef, ActionSourceSurface};
    use agentflow_pack::{
        software_dev_connector_definition, software_dev_domain_definition,
        software_dev_surface_definition, ui_design_connector_definition,
        ui_design_domain_definition, ui_design_surface_definition, validate_pack_bundle,
        PackConnectorDefinition, PackDomainDefinition, PackManifest, PackMigrationPolicy,
        PackSurfaceDefinition, PackType, PackValidationArtifact, PackValidationStatus,
        PACK_MANIFEST_VERSION,
    };
    use agentflow_runtime_api::{
        action_contract_ref_for_action_type, core_runtime_route, CORE_RUNTIME_COMMAND_TYPE,
    };
    use serde_json::json;
    use tempfile::tempdir;

    fn valid_command(command_type: &str) -> RuntimeCommandRequest {
        RuntimeCommandRequest {
            command_id: "cmd-001".to_string(),
            command_type: CORE_RUNTIME_COMMAND_TYPE.to_string(),
            route: action_contract_ref_for_action_type(command_type).map(|contract_ref| {
                core_runtime_route(format!("core:{command_type}"), contract_ref, Some("Issue"))
            }),
            source_surface: ActionSourceSurface::Cli,
            actor_role: "build-agent".to_string(),
            skill_ref: Some(format!("core:build-agent:{command_type}")),
            target_object_ref: Some(ActionRef {
                object_type: "Issue".to_string(),
                id: "AF-SIM-001".to_string(),
            }),
            input: json!({"note": "simulate only"}),
            evidence_refs: Vec::new(),
            artifact_refs: Vec::new(),
            expected_outputs: Vec::new(),
            evidence_policy: None,
            idempotency_key: "cmd-001:simulate".to_string(),
            created_at: "2026-06-23T00:00:00Z".to_string(),
        }
    }

    fn pack_manifest(
        pack_id: &str,
        pack_type: PackType,
        surface: &PackSurfaceDefinition,
        connector: &PackConnectorDefinition,
    ) -> PackManifest {
        PackManifest {
            version: PACK_MANIFEST_VERSION.to_string(),
            pack_id: pack_id.to_string(),
            name: pack_id.to_string(),
            pack_type,
            pack_version: "0.8.0".to_string(),
            runtime_compatibility: ">=0.8.0".to_string(),
            domain_path: "domain/".to_string(),
            surface_path: "surface/".to_string(),
            connector_path: "connectors/".to_string(),
            required_capabilities: connector
                .connectors
                .iter()
                .flat_map(|connector| connector.required_capabilities.clone())
                .collect(),
            owned_object_types: vec!["Issue".to_string()],
            exposed_commands: surface
                .command_entry_mappings
                .iter()
                .map(|mapping| mapping.command_type.clone())
                .chain(connector.connectors.iter().flat_map(|connector| {
                    connector
                        .supported_actions
                        .iter()
                        .map(|action| action.command_type.clone())
                }))
                .collect(),
            projection_entries: surface
                .read_model_dependencies
                .iter()
                .map(|dependency| dependency.projection_ref.clone())
                .collect(),
            migration_policy: PackMigrationPolicy::PreviewOnly,
            validation_status: PackValidationStatus::Draft,
        }
    }

    fn pack_api_entries(
        surface: &PackSurfaceDefinition,
        connector: &PackConnectorDefinition,
    ) -> Vec<String> {
        surface
            .command_entry_mappings
            .iter()
            .map(|mapping| mapping.command_type.clone())
            .chain(connector.connectors.iter().flat_map(|connector| {
                connector
                    .supported_actions
                    .iter()
                    .map(|action| action.command_type.clone())
            }))
            .collect()
    }

    fn pack_validation(
        pack_id: &str,
        pack_type: PackType,
        domain: &PackDomainDefinition,
        surface: &PackSurfaceDefinition,
        connector: &PackConnectorDefinition,
    ) -> PackValidationArtifact {
        let manifest = pack_manifest(pack_id, pack_type, surface, connector);
        validate_pack_bundle(
            &manifest,
            domain,
            surface,
            connector,
            &pack_api_entries(surface, connector),
            "0.8.0",
        )
    }

    #[test]
    fn command_simulation_is_read_only_and_reports_rejection() {
        let report = simulate_command(&valid_command("startRun")).unwrap();

        assert_eq!(report.kind, SimulationKind::Command);
        assert!(!report.writes_authority);
        assert!(!report.writes_event_store);
        assert!(!report.executes_provider);
        assert!(!report.expected_events.is_empty());
        assert!(!report.affected_projections.is_empty());
        assert!(!report.rejected_reasons.is_empty());
        assert!(matches!(
            report.decision,
            SimulationDecision::Rejected | SimulationDecision::HumanDecisionRequired
        ));
    }

    #[test]
    fn unsupported_command_simulation_outputs_rejected_reason() {
        let mut request = valid_command("unknownCommand");
        request.target_object_ref = None;
        let report = simulate_command(&request).unwrap();

        assert_eq!(report.decision, SimulationDecision::InvalidCommand);
        assert!(!report.writes_authority);
        assert!(report
            .rejected_reasons
            .iter()
            .any(|reason| reason.code == "UnsupportedCommand"));
    }

    #[test]
    fn issue_simulation_reports_required_gates_without_writes() {
        let report = simulate_issue(&SimulationIssueRequest {
            issue_id: "AF-SIM-002".to_string(),
            actor_role: "build-agent".to_string(),
            project_id: Some("project-sim".to_string()),
            dependency_ids: vec!["AF-BLOCKER-001".to_string()],
            context_pack_ready: false,
            workspace_clean: false,
            created_at: "2026-06-23T00:00:00Z".to_string(),
        });

        assert_eq!(report.decision, SimulationDecision::Rejected);
        assert!(!report.writes_authority);
        assert!(!report.writes_event_store);
        assert!(report.gate_impact.iter().any(|gate| {
            gate.gate_id == "panel.context-pack.ready" && gate.status == "rejected"
        }));
        assert!(report.gate_impact.iter().any(|gate| {
            gate.gate_id == "runtime.workspace.clean" && gate.status == "rejected"
        }));
        assert!(report
            .affected_objects
            .iter()
            .any(|object| object.object_type == "Issue" && object.object_id == "AF-SIM-002"));
        assert!(report
            .required_evidence
            .iter()
            .any(|evidence| evidence.evidence_type == "panel-context-pack"
                && evidence.status == "missing"));
        assert!(report
            .state_transitions
            .iter()
            .any(|transition| transition.to_state == "in_progress"));
        assert!(report
            .downstream_triggers
            .iter()
            .any(
                |trigger| trigger.trigger_id == "build-agent.launch.requested"
                    && trigger.status == "blocked"
            ));
    }

    #[test]
    fn completion_simulation_requires_evidence_and_delivery_refs() {
        let report = simulate_completion(&SimulationCompletionRequest {
            issue_id: "AF-SIM-003".to_string(),
            run_id: "run-001".to_string(),
            actor_role: "build-agent".to_string(),
            validation_evidence_refs: Vec::new(),
            delivery_artifact_refs: Vec::new(),
            pr_or_mr_ref: None,
            created_at: "2026-06-23T00:00:00Z".to_string(),
        });

        assert_eq!(report.decision, SimulationDecision::Rejected);
        assert!(!report.rejected_reasons.is_empty());
        assert!(report
            .affected_projections
            .iter()
            .any(|projection| projection.projection_id == "release.delivery-summary"));
        let completion_commit = report.completion_commit.unwrap();
        assert!(completion_commit.requires_validation_evidence);
        assert!(completion_commit.requires_delivery_artifacts);
        assert!(completion_commit.requires_merge_proof);
        assert!(completion_commit
            .expected_event_chain
            .contains(&"completion.commit.accepted".to_string()));
        assert!(report
            .required_evidence
            .iter()
            .any(|evidence| evidence.evidence_type == "validation-evidence"
                && evidence.status == "missing"));
        assert!(report
            .affected_objects
            .iter()
            .any(|object| object.object_type == "Run" && object.object_id == "run-001"));
        assert!(report
            .downstream_triggers
            .iter()
            .any(|trigger| trigger.trigger_id == "projection.rebuild.requested"));
    }

    #[test]
    fn pack_command_simulation_covers_software_dev_command_without_writes() {
        let domain = software_dev_domain_definition();
        let surface = software_dev_surface_definition();
        let connector = software_dev_connector_definition();
        let validation = pack_validation(
            "software-dev",
            PackType::SoftwareDev,
            &domain,
            &surface,
            &connector,
        );
        let report = simulate_pack_command(&PackCommandSimulationRequest {
            simulation_id: "sim-pack-software-001".to_string(),
            command: "work.issue.start".to_string(),
            target_object_type: "Issue".to_string(),
            target_object_id: "AF-PACK-001".to_string(),
            actor_role: "work-agent".to_string(),
            validation,
            domain,
            surface,
            connector,
            created_at: "2026-06-23T00:00:00Z".to_string(),
        });

        assert_eq!(report.kind, SimulationKind::PackCommand);
        assert_eq!(report.decision, SimulationDecision::Accepted);
        assert!(!report.writes_authority);
        assert!(!report.writes_event_store);
        assert!(!report.executes_provider);
        assert!(report
            .expected_events
            .iter()
            .any(|event| event.event_type == "startRun.proposed"));
        assert!(report
            .affected_projections
            .iter()
            .any(|projection| projection.projection_id == "projection.task-workbench"));
        assert!(report
            .affected_objects
            .iter()
            .any(|object| object.object_type == "Issue" && object.object_id == "AF-PACK-001"));
        assert!(report
            .required_evidence
            .iter()
            .any(|evidence| evidence.evidence_type == "validation"));
        assert!(report
            .state_transitions
            .iter()
            .any(|transition| transition.trigger == "work.issue.start"
                && transition.to_state == "action-proposed"));
        assert!(!report.downstream_triggers.is_empty());
        assert!(report
            .conflicts
            .iter()
            .any(|conflict| conflict.status == "preview-only"));
        assert_eq!(report.metadata["requiredEvidence"][0], "validation");
    }

    #[test]
    fn pack_command_simulation_covers_ui_design_command_without_provider_launch() {
        let domain = ui_design_domain_definition();
        let surface = ui_design_surface_definition();
        let connector = ui_design_connector_definition();
        let validation = pack_validation(
            "ui-design",
            PackType::UiDesign,
            &domain,
            &surface,
            &connector,
        );
        let report = simulate_pack_command(&PackCommandSimulationRequest {
            simulation_id: "sim-pack-design-001".to_string(),
            command: "design.wireframe.generate".to_string(),
            target_object_type: "Wireframe".to_string(),
            target_object_id: "wireframe-001".to_string(),
            actor_role: "work-agent".to_string(),
            validation,
            domain,
            surface,
            connector,
            created_at: "2026-06-23T00:00:00Z".to_string(),
        });

        assert_eq!(report.decision, SimulationDecision::Accepted);
        assert!(!report.executes_provider);
        assert!(report
            .expected_events
            .iter()
            .any(|event| event.event_type == "design.generate-wireframe.proposed"));
    }

    #[test]
    fn pack_command_simulation_reports_missing_mappings_and_read_models() {
        let domain = software_dev_domain_definition();
        let surface = software_dev_surface_definition();
        let connector = software_dev_connector_definition();
        let mut manifest =
            pack_manifest("software-dev", PackType::SoftwareDev, &surface, &connector);
        manifest.projection_entries.clear();
        let validation = validate_pack_bundle(
            &manifest,
            &domain,
            &surface,
            &connector,
            &pack_api_entries(&surface, &connector),
            "0.8.0",
        );
        let report = simulate_pack_command(&PackCommandSimulationRequest {
            simulation_id: "sim-pack-missing-001".to_string(),
            command: "unknown.command".to_string(),
            target_object_type: "Issue".to_string(),
            target_object_id: "AF-PACK-002".to_string(),
            actor_role: "work-agent".to_string(),
            validation,
            domain,
            surface,
            connector,
            created_at: "2026-06-23T00:00:00Z".to_string(),
        });

        assert_eq!(report.decision, SimulationDecision::Rejected);
        assert!(report
            .rejected_reasons
            .iter()
            .any(|reason| reason.code == "SurfaceMappingMissing"));
        assert!(report
            .rejected_reasons
            .iter()
            .any(|reason| reason.code == "ConnectorActionMissing"));
        assert!(report
            .rejected_reasons
            .iter()
            .any(|reason| reason.code == "ReadModelMissing"));
    }

    #[test]
    fn simulation_does_not_create_agentflow_files() {
        let dir = tempdir().unwrap();
        let _report = simulate_issue(&SimulationIssueRequest {
            issue_id: "AF-SIM-004".to_string(),
            actor_role: "build-agent".to_string(),
            project_id: None,
            dependency_ids: Vec::new(),
            context_pack_ready: true,
            workspace_clean: true,
            created_at: "2026-06-23T00:00:00Z".to_string(),
        });

        assert!(!dir.path().join(".agentflow").exists());
    }
}
