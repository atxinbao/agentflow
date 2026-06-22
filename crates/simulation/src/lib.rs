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
use agentflow_ontology::core_ontology_registry;
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
    pub affected_projections: Vec<SimulationAffectedProjection>,
    pub risk: SimulationRiskLevel,
    #[serde(default)]
    pub conflicts: Vec<SimulationConflict>,
    #[serde(default)]
    pub gate_impact: Vec<SimulationGateImpact>,
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
            affected_projections: Vec::new(),
            risk: SimulationRiskLevel::Low,
            conflicts: Vec::new(),
            gate_impact: Vec::new(),
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
        expected_event("issue.done.requested", Some("Issue"), true),
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

fn core_simulation_context() -> Result<ArbitrationContext> {
    let ontology = core_ontology_registry();
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

#[cfg(test)]
mod tests {
    use super::*;
    use agentflow_action_contract::{ActionRef, ActionSourceSurface};
    use serde_json::json;
    use tempfile::tempdir;

    fn valid_command(command_type: &str) -> RuntimeCommandRequest {
        RuntimeCommandRequest {
            command_id: "cmd-001".to_string(),
            command_type: command_type.to_string(),
            source_surface: ActionSourceSurface::Cli,
            actor_role: "build-agent".to_string(),
            target_object_ref: Some(ActionRef {
                object_type: "Issue".to_string(),
                id: "AF-SIM-001".to_string(),
            }),
            input: json!({"note": "simulate only"}),
            evidence_refs: Vec::new(),
            artifact_refs: Vec::new(),
            idempotency_key: "cmd-001:simulate".to_string(),
            created_at: "2026-06-23T00:00:00Z".to_string(),
        }
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
