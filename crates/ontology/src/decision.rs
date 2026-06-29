use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

use crate::semantics::CoreActionStateSemanticsContract;

pub const CORE_DECISION_MODEL_CONTRACT_VERSION: &str = "agentflow-core-decision-model.v1";
pub const CORE_DECISION_INPUT_BINDING_CONTRACT_VERSION: &str =
    "agentflow-core-decision-input-binding.v1";
pub const CORE_DECISION_OUTCOME_TRANSITION_CONTRACT_VERSION: &str =
    "agentflow-core-decision-outcome-transition.v1";
pub const CORE_DECISION_FAILURE_REASON_CONTRACT_VERSION: &str =
    "agentflow-core-decision-failure-reason.v1";
pub const CORE_EVIDENCE_DECISION_MODEL_VERSION: &str =
    "agentflow-core-evidence-decision-reference-model.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreDecisionModelContract {
    pub version: String,
    pub status: String,
    pub authority: String,
    pub readable_authority_facts: Vec<CoreDecisionReadableFact>,
    pub write_authority: CoreDecisionWriteAuthority,
    pub required_record_fields: Vec<String>,
    pub outcomes: Vec<CoreDecisionKernelOutcome>,
    pub forbidden_core_terms: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreDecisionReadableFact {
    pub fact_kind: String,
    pub accepted_ref_kind: String,
    pub purpose: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreDecisionWriteAuthority {
    pub may_write: Vec<String>,
    pub must_not_write: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreDecisionKernelOutcome {
    pub outcome: String,
    pub meaning: String,
    pub terminal: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreDecisionRecord {
    pub version: String,
    pub decision_id: String,
    pub decided_at: String,
    pub decided_by: String,
    pub subject: CoreDecisionSubjectRef,
    pub inputs: CoreDecisionInputs,
    pub outcome: String,
    pub reasons: Vec<CoreDecisionReason>,
    pub writes: Vec<CoreDecisionWriteRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreDecisionSubjectRef {
    pub subject_ref_kind: String,
    pub subject_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreDecisionInputs {
    pub spec_refs: Vec<String>,
    pub runtime_state_refs: Vec<String>,
    pub evidence_refs: Vec<String>,
    pub prior_decision_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreDecisionReason {
    pub reason_code: String,
    pub message: String,
    pub evidence_refs: Vec<String>,
    pub blocking: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreDecisionWriteRef {
    pub write_kind: String,
    pub target_ref: String,
    pub authority_boundary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreDecisionInputBindingContract {
    pub version: String,
    pub status: String,
    pub authority: String,
    pub required_authority_refs: Vec<CoreDecisionInputAuthorityRequirement>,
    pub optional_context_refs: Vec<CoreDecisionInputAuthorityRequirement>,
    pub rejected_ref_kinds: Vec<String>,
    pub freshness_rule: String,
    pub forbidden_core_terms: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreDecisionInputAuthorityRequirement {
    pub input_kind: String,
    pub accepted_ref_kind: String,
    pub source_kernel: String,
    pub required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreDecisionInputBinding {
    pub version: String,
    pub binding_id: String,
    pub decision_id: String,
    pub spec_bundle_ref: Option<CoreDecisionBoundAuthorityRef>,
    pub ontology_object_refs: Vec<CoreDecisionBoundAuthorityRef>,
    pub runtime_action_state_ref: Option<CoreDecisionBoundAuthorityRef>,
    pub evidence_pack_refs: Vec<CoreDecisionBoundAuthorityRef>,
    pub delivery_context_refs: Vec<CoreDecisionBoundAuthorityRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreDecisionBoundAuthorityRef {
    pub ref_kind: String,
    pub ref_id: String,
    pub authority_path: String,
    pub version: String,
    pub observed_at: String,
    pub stale: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreDecisionOutcomeTransitionContract {
    pub version: String,
    pub status: String,
    pub authority: String,
    pub outcomes: Vec<CoreDecisionOutcomeTransition>,
    pub reason_shape: CoreDecisionReasonShape,
    pub illegal_transition_policy: String,
    pub forbidden_core_terms: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreDecisionOutcomeTransition {
    pub outcome: String,
    pub meaning: String,
    pub terminal: bool,
    pub allowed_from_states: Vec<String>,
    pub allowed_next_states: Vec<String>,
    pub required_reason_fields: Vec<String>,
    pub required_evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreDecisionReasonShape {
    pub required_fields: Vec<String>,
    pub machine_readable_reason_code: bool,
    pub evidence_refs_required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreDecisionTransitionAttempt {
    pub outcome: String,
    pub from_state: String,
    pub requested_next_state: String,
    pub reasons: Vec<CoreDecisionReason>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreDecisionFailureReasonContract {
    pub version: String,
    pub status: String,
    pub authority: String,
    pub applies_to_outcomes: Vec<String>,
    pub required_fields: Vec<String>,
    pub remediation_routes: Vec<CoreDecisionRemediationRoute>,
    pub retry_policy: String,
    pub forbidden_core_terms: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreDecisionRemediationRoute {
    pub route: String,
    pub meaning: String,
    pub retry_eligible: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreDecisionFailureReason {
    pub outcome: String,
    pub reason_code: String,
    pub message: String,
    pub authority_refs: Vec<String>,
    pub missing_evidence_refs: Vec<String>,
    pub remediation_route: String,
    pub retry_eligible: bool,
    pub blocking: bool,
}

pub fn core_decision_model_contract() -> CoreDecisionModelContract {
    CoreDecisionModelContract {
        version: CORE_DECISION_MODEL_CONTRACT_VERSION.to_string(),
        status: "active".to_string(),
        authority: "Core Decision records explain a generic judgment from stable authority inputs."
            .to_string(),
        readable_authority_facts: vec![
            readable_fact(
                "spec",
                "SpecRef",
                "read confirmed intent, scope, boundary, and expected result",
            ),
            readable_fact(
                "runtimeState",
                "RuntimeStateRef",
                "read current object state and route state",
            ),
            readable_fact(
                "evidence",
                "EvidenceRef",
                "read proof packs, receipts, completeness, and trace links",
            ),
            readable_fact(
                "priorDecision",
                "DecisionRef",
                "read earlier judgment when resolving follow-up routes",
            ),
        ],
        write_authority: CoreDecisionWriteAuthority {
            may_write: vec!["decision-record".to_string(), "decision-event".to_string()],
            must_not_write: vec![
                "spec-authority".to_string(),
                "runtime-state-authority".to_string(),
                "evidence-authority".to_string(),
                "projection-read-model".to_string(),
                "provider-session-record".to_string(),
                "audit-sidecar-record".to_string(),
            ],
        },
        required_record_fields: vec![
            "version".to_string(),
            "decisionId".to_string(),
            "decidedAt".to_string(),
            "decidedBy".to_string(),
            "subject".to_string(),
            "inputs".to_string(),
            "outcome".to_string(),
            "reasons".to_string(),
            "writes".to_string(),
        ],
        outcomes: vec![
            decision_outcome(
                "accepted",
                "the subject is allowed to continue to the next route",
                false,
            ),
            decision_outcome(
                "rejected",
                "the subject is stopped because authority inputs contradict the request",
                true,
            ),
            decision_outcome(
                "deferred",
                "the subject waits for additional authority input or proof",
                false,
            ),
            decision_outcome(
                "blocked",
                "the subject cannot continue until blocking reasons are resolved",
                false,
            ),
            decision_outcome(
                "needs-fix",
                "the subject requires additional work before another decision",
                false,
            ),
        ],
        forbidden_core_terms: vec![
            "bug".to_string(),
            "feature".to_string(),
            "issue".to_string(),
            "pr".to_string(),
            "pull-request".to_string(),
            "release".to_string(),
            "repository".to_string(),
            "repository-patch".to_string(),
            "test-log".to_string(),
            "github-issue".to_string(),
        ],
    }
}

pub fn core_decision_failure_reason_contract() -> CoreDecisionFailureReasonContract {
    CoreDecisionFailureReasonContract {
        version: CORE_DECISION_FAILURE_REASON_CONTRACT_VERSION.to_string(),
        status: "active".to_string(),
        authority: "Core Decision failure reasons explain non-accepted outcomes with stable remediation routes."
            .to_string(),
        applies_to_outcomes: vec![
            "rejected".to_string(),
            "deferred".to_string(),
            "blocked".to_string(),
            "needs-fix".to_string(),
        ],
        required_fields: vec![
            "reasonCode".to_string(),
            "message".to_string(),
            "authorityRefs".to_string(),
            "missingEvidenceRefs".to_string(),
            "remediationRoute".to_string(),
            "retryEligible".to_string(),
            "blocking".to_string(),
        ],
        remediation_routes: vec![
            remediation_route(
                "wait-for-authority",
                "wait for a required authority fact to become current",
                true,
            ),
            remediation_route(
                "collect-evidence",
                "collect missing proof before re-evaluating the decision",
                true,
            ),
            remediation_route(
                "revise-subject",
                "revise the subject boundary before re-evaluating the decision",
                true,
            ),
            remediation_route(
                "cancel-subject",
                "stop the subject without another automatic decision attempt",
                false,
            ),
            remediation_route(
                "retry-decision",
                "retry the decision after authority inputs change",
                true,
            ),
        ],
        retry_policy:
            "Retry is allowed only when retryEligible is true and referenced authority facts have changed."
                .to_string(),
        forbidden_core_terms: core_decision_model_contract().forbidden_core_terms,
    }
}

pub fn canonical_core_decision_failure_reason_fixture() -> CoreDecisionFailureReason {
    CoreDecisionFailureReason {
        outcome: "blocked".to_string(),
        reason_code: "authority-ref-stale".to_string(),
        message: "required authority input is stale".to_string(),
        authority_refs: vec!["runtime-state:object-ready".to_string()],
        missing_evidence_refs: vec!["EvidenceRef:current-runtime-state".to_string()],
        remediation_route: "wait-for-authority".to_string(),
        retry_eligible: true,
        blocking: true,
    }
}

pub fn core_decision_input_binding_contract() -> CoreDecisionInputBindingContract {
    CoreDecisionInputBindingContract {
        version: CORE_DECISION_INPUT_BINDING_CONTRACT_VERSION.to_string(),
        status: "active".to_string(),
        authority: "Core Decision input bindings connect a decision to current authority facts before any outcome is evaluated."
            .to_string(),
        required_authority_refs: vec![
            input_requirement("specBundle", "SpecBundleRef", "spec-kernel", true),
            input_requirement("ontologyObject", "OntologyObjectRef", "ontology-kernel", true),
            input_requirement(
                "runtimeActionState",
                "RuntimeActionStateRef",
                "runtime-kernel",
                true,
            ),
            input_requirement("evidencePack", "EvidencePackRef", "evidence-kernel", true),
        ],
        optional_context_refs: vec![input_requirement(
            "deliveryContext",
            "DeliveryContextRef",
            "delivery-context",
            false,
        )],
        rejected_ref_kinds: vec![
            "ProjectionRef".to_string(),
            "ProviderSessionRef".to_string(),
            "CliSessionRef".to_string(),
            "ChatThreadRef".to_string(),
        ],
        freshness_rule:
            "Every required authority ref must be current; stale refs block the decision input binding."
                .to_string(),
        forbidden_core_terms: core_decision_model_contract().forbidden_core_terms,
    }
}

pub fn core_decision_outcome_transition_contract() -> CoreDecisionOutcomeTransitionContract {
    CoreDecisionOutcomeTransitionContract {
        version: CORE_DECISION_OUTCOME_TRANSITION_CONTRACT_VERSION.to_string(),
        status: "active".to_string(),
        authority:
            "Core Decision outcomes describe allowed state routes without writing completion state."
                .to_string(),
        outcomes: vec![
            outcome_transition(
                "accepted",
                "authority inputs allow the subject to continue",
                false,
                vec!["planned", "reviewing"],
                vec!["ready"],
                vec!["EvidenceRef", "DecisionRef"],
            ),
            outcome_transition(
                "rejected",
                "authority inputs contradict the subject boundary",
                true,
                vec!["captured", "understood", "planned", "ready", "reviewing"],
                vec!["cancelled"],
                vec!["DecisionRef"],
            ),
            outcome_transition(
                "deferred",
                "authority inputs are not complete enough to choose a final route",
                false,
                vec!["captured", "understood", "planned", "ready"],
                vec!["planned", "blocked"],
                vec!["EvidenceRef", "DecisionRef"],
            ),
            outcome_transition(
                "blocked",
                "an external blocker prevents progress",
                false,
                vec![
                    "captured",
                    "understood",
                    "planned",
                    "ready",
                    "active",
                    "reviewing",
                ],
                vec!["blocked"],
                vec!["DecisionRef"],
            ),
            outcome_transition(
                "needs-fix",
                "the subject needs additional work before another review or decision",
                false,
                vec!["active", "reviewing"],
                vec!["active"],
                vec!["EvidenceRef", "DecisionRef"],
            ),
        ],
        reason_shape: CoreDecisionReasonShape {
            required_fields: vec![
                "reasonCode".to_string(),
                "message".to_string(),
                "evidenceRefs".to_string(),
                "blocking".to_string(),
            ],
            machine_readable_reason_code: true,
            evidence_refs_required: true,
        },
        illegal_transition_policy:
            "Illegal outcome transitions must fail before any state authority write.".to_string(),
        forbidden_core_terms: core_decision_model_contract().forbidden_core_terms,
    }
}

pub fn canonical_core_decision_record_fixture() -> CoreDecisionRecord {
    CoreDecisionRecord {
        version: CORE_DECISION_MODEL_CONTRACT_VERSION.to_string(),
        decision_id: "decision-core-001".to_string(),
        decided_at: "2026-06-29T00:00:00Z".to_string(),
        decided_by: "role:decision-kernel".to_string(),
        subject: CoreDecisionSubjectRef {
            subject_ref_kind: "TaskRef".to_string(),
            subject_ref: "task:core-decision-model".to_string(),
        },
        inputs: CoreDecisionInputs {
            spec_refs: vec!["spec:core-decision-model".to_string()],
            runtime_state_refs: vec!["runtime-state:task-ready".to_string()],
            evidence_refs: vec!["evidence:core-evidence-pack".to_string()],
            prior_decision_refs: Vec::new(),
        },
        outcome: "accepted".to_string(),
        reasons: vec![CoreDecisionReason {
            reason_code: "authority-inputs-consistent".to_string(),
            message: "authority inputs and evidence are consistent".to_string(),
            evidence_refs: vec!["evidence:core-evidence-pack".to_string()],
            blocking: false,
        }],
        writes: vec![
            CoreDecisionWriteRef {
                write_kind: "decision-record".to_string(),
                target_ref: "decision:decision-core-001".to_string(),
                authority_boundary: "core-decision-authority".to_string(),
            },
            CoreDecisionWriteRef {
                write_kind: "decision-event".to_string(),
                target_ref: "event:decision-recorded-001".to_string(),
                authority_boundary: "event-store".to_string(),
            },
        ],
    }
}

pub fn canonical_core_decision_transition_attempt_fixture() -> CoreDecisionTransitionAttempt {
    CoreDecisionTransitionAttempt {
        outcome: "accepted".to_string(),
        from_state: "planned".to_string(),
        requested_next_state: "ready".to_string(),
        reasons: vec![CoreDecisionReason {
            reason_code: "authority-inputs-consistent".to_string(),
            message: "authority inputs allow the next route".to_string(),
            evidence_refs: vec!["evidence:core-proof-pack".to_string()],
            blocking: false,
        }],
    }
}

pub fn canonical_core_decision_input_binding_fixture() -> CoreDecisionInputBinding {
    CoreDecisionInputBinding {
        version: CORE_DECISION_INPUT_BINDING_CONTRACT_VERSION.to_string(),
        binding_id: "binding-core-001".to_string(),
        decision_id: "decision-core-001".to_string(),
        spec_bundle_ref: Some(bound_authority_ref(
            "SpecBundleRef",
            "spec:core-decision-input-binding",
            "docs/requirements/core-decision-input-binding.md",
            "spec-bundle.v1",
        )),
        ontology_object_refs: vec![bound_authority_ref(
            "OntologyObjectRef",
            "ontology:core/action-state",
            "runtime/ontology/core-action-state-semantics.json",
            "ontology-object.v1",
        )],
        runtime_action_state_ref: Some(bound_authority_ref(
            "RuntimeActionStateRef",
            "runtime-state:object-ready",
            ".agentflow/state/runtime/object-ready.json",
            "runtime-state.v1",
        )),
        evidence_pack_refs: vec![bound_authority_ref(
            "EvidencePackRef",
            "evidence:core-proof-pack",
            ".agentflow/tasks/core-decision/evidence/proof-pack.json",
            "evidence-pack.v1",
        )],
        delivery_context_refs: vec![bound_authority_ref(
            "DeliveryContextRef",
            "delivery-context:optional-public-record",
            "docs/delivery/core-decision-input-binding.md",
            "delivery-context.v1",
        )],
    }
}

pub fn validate_core_decision_model_contract(
    contract: &CoreDecisionModelContract,
) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    if contract.version != CORE_DECISION_MODEL_CONTRACT_VERSION {
        errors.push(format!(
            "decision model version must be `{}`",
            CORE_DECISION_MODEL_CONTRACT_VERSION
        ));
    }
    if contract.status != "active" {
        errors.push("decision model status must be active".to_string());
    }

    let readable_fact_kinds: BTreeSet<_> = contract
        .readable_authority_facts
        .iter()
        .map(|fact| fact.fact_kind.as_str())
        .collect();
    for required in ["spec", "runtimeState", "evidence"] {
        if !readable_fact_kinds.contains(required) {
            errors.push(format!(
                "decision model cannot read required fact `{required}`"
            ));
        }
    }

    let required_fields: BTreeSet<_> = contract
        .required_record_fields
        .iter()
        .map(String::as_str)
        .collect();
    for field in [
        "version",
        "decisionId",
        "decidedAt",
        "subject",
        "inputs",
        "outcome",
        "reasons",
    ] {
        if !required_fields.contains(field) {
            errors.push(format!(
                "decision model missing required record field `{field}`"
            ));
        }
    }

    let may_write: BTreeSet<_> = contract
        .write_authority
        .may_write
        .iter()
        .map(String::as_str)
        .collect();
    for allowed_write in ["decision-record", "decision-event"] {
        if !may_write.contains(allowed_write) {
            errors.push(format!("decision model cannot write `{allowed_write}`"));
        }
    }
    for forbidden_write in [
        "spec-authority",
        "runtime-state-authority",
        "evidence-authority",
        "projection-read-model",
        "provider-session-record",
    ] {
        if !contract
            .write_authority
            .must_not_write
            .iter()
            .any(|item| item == forbidden_write)
        {
            errors.push(format!(
                "decision model must explicitly forbid `{forbidden_write}` writes"
            ));
        }
    }

    let outcomes: BTreeSet<_> = contract
        .outcomes
        .iter()
        .map(|outcome| outcome.outcome.as_str())
        .collect();
    for outcome in ["accepted", "rejected", "deferred", "blocked", "needs-fix"] {
        if !outcomes.contains(outcome) {
            errors.push(format!("decision model missing outcome `{outcome}`"));
        }
    }

    validate_no_forbidden_terms(
        "Core decision model",
        &contract.forbidden_core_terms,
        core_decision_model_surface(contract),
        &mut errors,
    );

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

pub fn validate_core_decision_input_binding_contract(
    contract: &CoreDecisionInputBindingContract,
) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    if contract.version != CORE_DECISION_INPUT_BINDING_CONTRACT_VERSION {
        errors.push(format!(
            "decision input binding version must be `{}`",
            CORE_DECISION_INPUT_BINDING_CONTRACT_VERSION
        ));
    }
    if contract.status != "active" {
        errors.push("decision input binding status must be active".to_string());
    }

    let required_inputs: BTreeSet<_> = contract
        .required_authority_refs
        .iter()
        .filter(|requirement| requirement.required)
        .map(|requirement| requirement.input_kind.as_str())
        .collect();
    for required in [
        "specBundle",
        "ontologyObject",
        "runtimeActionState",
        "evidencePack",
    ] {
        if !required_inputs.contains(required) {
            errors.push(format!(
                "decision input binding missing required authority ref `{required}`"
            ));
        }
    }

    let accepted_required_kinds: BTreeSet<_> = contract
        .required_authority_refs
        .iter()
        .map(|requirement| requirement.accepted_ref_kind.as_str())
        .collect();
    for required_kind in [
        "SpecBundleRef",
        "OntologyObjectRef",
        "RuntimeActionStateRef",
        "EvidencePackRef",
    ] {
        if !accepted_required_kinds.contains(required_kind) {
            errors.push(format!(
                "decision input binding cannot accept required ref kind `{required_kind}`"
            ));
        }
    }

    for rejected_kind in ["ProjectionRef", "ProviderSessionRef"] {
        if !contract
            .rejected_ref_kinds
            .iter()
            .any(|item| item == rejected_kind)
        {
            errors.push(format!(
                "decision input binding must reject `{rejected_kind}`"
            ));
        }
    }

    validate_no_forbidden_terms(
        "Core decision input binding contract",
        &contract.forbidden_core_terms,
        core_decision_input_binding_contract_surface(contract),
        &mut errors,
    );

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

pub fn validate_core_decision_outcome_transition_contract(
    contract: &CoreDecisionOutcomeTransitionContract,
    semantics: &CoreActionStateSemanticsContract,
) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    if contract.version != CORE_DECISION_OUTCOME_TRANSITION_CONTRACT_VERSION {
        errors.push(format!(
            "decision outcome transition version must be `{}`",
            CORE_DECISION_OUTCOME_TRANSITION_CONTRACT_VERSION
        ));
    }
    if contract.status != "active" {
        errors.push("decision outcome transition status must be active".to_string());
    }

    let outcomes: BTreeSet<_> = contract
        .outcomes
        .iter()
        .map(|outcome| outcome.outcome.as_str())
        .collect();
    for required_outcome in ["accepted", "rejected", "deferred", "blocked", "needs-fix"] {
        if !outcomes.contains(required_outcome) {
            errors.push(format!(
                "decision outcome transition missing outcome `{required_outcome}`"
            ));
        }
    }

    let state_ids: BTreeSet<_> = semantics
        .states
        .iter()
        .map(|state| state.state_id.as_str())
        .collect();
    for outcome in &contract.outcomes {
        if outcome.allowed_from_states.is_empty() {
            errors.push(format!(
                "decision outcome `{}` must declare allowed source states",
                outcome.outcome
            ));
        }
        if outcome.allowed_next_states.is_empty() {
            errors.push(format!(
                "decision outcome `{}` must declare allowed next states",
                outcome.outcome
            ));
        }
        for from_state in &outcome.allowed_from_states {
            if !state_ids.contains(from_state.as_str()) {
                errors.push(format!(
                    "decision outcome `{}` references unknown source state `{from_state}`",
                    outcome.outcome
                ));
            }
        }
        for next_state in &outcome.allowed_next_states {
            if !state_ids.contains(next_state.as_str()) {
                errors.push(format!(
                    "decision outcome `{}` references unknown next state `{next_state}`",
                    outcome.outcome
                ));
            }
            if next_state == "completed" {
                errors.push(format!(
                    "decision outcome `{}` must not write completion state",
                    outcome.outcome
                ));
            }
        }
        for required_field in ["reasonCode", "message", "evidenceRefs", "blocking"] {
            if !outcome
                .required_reason_fields
                .iter()
                .any(|field| field == required_field)
            {
                errors.push(format!(
                    "decision outcome `{}` missing reason field `{required_field}`",
                    outcome.outcome
                ));
            }
        }
    }

    for required_field in ["reasonCode", "message", "evidenceRefs", "blocking"] {
        if !contract
            .reason_shape
            .required_fields
            .iter()
            .any(|field| field == required_field)
        {
            errors.push(format!(
                "decision outcome reason shape missing `{required_field}`"
            ));
        }
    }
    if !contract.reason_shape.machine_readable_reason_code {
        errors.push("decision outcome reason code must be machine readable".to_string());
    }
    if !contract.reason_shape.evidence_refs_required {
        errors.push("decision outcome reasons must require evidence refs".to_string());
    }

    validate_no_forbidden_terms(
        "Core decision outcome transition contract",
        &contract.forbidden_core_terms,
        core_decision_outcome_transition_contract_surface(contract),
        &mut errors,
    );

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

pub fn validate_core_decision_failure_reason_contract(
    contract: &CoreDecisionFailureReasonContract,
    outcome_contract: &CoreDecisionOutcomeTransitionContract,
) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    if contract.version != CORE_DECISION_FAILURE_REASON_CONTRACT_VERSION {
        errors.push(format!(
            "decision failure reason version must be `{}`",
            CORE_DECISION_FAILURE_REASON_CONTRACT_VERSION
        ));
    }
    if contract.status != "active" {
        errors.push("decision failure reason status must be active".to_string());
    }

    let transition_outcomes: BTreeSet<_> = outcome_contract
        .outcomes
        .iter()
        .map(|outcome| outcome.outcome.as_str())
        .collect();
    let applies_to: BTreeSet<_> = contract
        .applies_to_outcomes
        .iter()
        .map(String::as_str)
        .collect();
    for outcome in ["rejected", "deferred", "blocked", "needs-fix"] {
        if !applies_to.contains(outcome) {
            errors.push(format!(
                "decision failure reason contract must apply to `{outcome}`"
            ));
        }
        if !transition_outcomes.contains(outcome) {
            errors.push(format!(
                "decision failure reason contract references missing outcome `{outcome}`"
            ));
        }
    }
    if applies_to.contains("accepted") {
        errors.push("decision failure reason contract must not apply to accepted".to_string());
    }
    for outcome in &contract.applies_to_outcomes {
        if !transition_outcomes.contains(outcome.as_str()) {
            errors.push(format!(
                "decision failure reason contract references unknown outcome `{outcome}`"
            ));
        }
    }

    for field in [
        "reasonCode",
        "message",
        "authorityRefs",
        "missingEvidenceRefs",
        "remediationRoute",
        "retryEligible",
        "blocking",
    ] {
        if !contract
            .required_fields
            .iter()
            .any(|required| required == field)
        {
            errors.push(format!(
                "decision failure reason contract missing required field `{field}`"
            ));
        }
    }

    let route_names: BTreeSet<_> = contract
        .remediation_routes
        .iter()
        .map(|route| route.route.as_str())
        .collect();
    for route in [
        "wait-for-authority",
        "collect-evidence",
        "revise-subject",
        "cancel-subject",
        "retry-decision",
    ] {
        if !route_names.contains(route) {
            errors.push(format!(
                "decision failure reason contract missing remediation route `{route}`"
            ));
        }
    }
    for route in &contract.remediation_routes {
        if route.route.trim().is_empty() {
            errors.push("decision failure reason remediation route id is required".to_string());
        }
        if route.meaning.trim().is_empty() {
            errors.push(format!(
                "decision failure reason remediation route `{}` meaning is required",
                route.route
            ));
        }
    }
    if contract.retry_policy.trim().is_empty() {
        errors.push("decision failure reason retry policy is required".to_string());
    }

    validate_no_forbidden_terms(
        "Core decision failure reason contract",
        &contract.forbidden_core_terms,
        core_decision_failure_reason_contract_surface(contract),
        &mut errors,
    );

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

pub fn validate_core_decision_failure_reason(
    contract: &CoreDecisionFailureReasonContract,
    reason: &CoreDecisionFailureReason,
) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    if reason.outcome == "accepted" {
        errors.push("decision failure reason must not be attached to accepted".to_string());
    }
    if !contract
        .applies_to_outcomes
        .iter()
        .any(|outcome| outcome == &reason.outcome)
    {
        errors.push(format!(
            "decision failure reason outcome `{}` is not in contract",
            reason.outcome
        ));
    }
    if reason.reason_code.trim().is_empty() {
        errors.push("decision failure reason code is required".to_string());
    }
    if reason.message.trim().is_empty() {
        errors.push("decision failure reason message is required".to_string());
    }
    if reason.authority_refs.is_empty() {
        errors.push("decision failure reason authority refs are required".to_string());
    }
    if reason.missing_evidence_refs.is_empty() {
        errors.push("decision failure reason missing evidence refs are required".to_string());
    }

    let Some(route) = contract
        .remediation_routes
        .iter()
        .find(|route| route.route == reason.remediation_route)
    else {
        errors.push(format!(
            "decision failure reason remediation route `{}` is not in contract",
            reason.remediation_route
        ));
        validate_no_forbidden_terms(
            "Core decision failure reason",
            &contract.forbidden_core_terms,
            core_decision_failure_reason_surface(reason),
            &mut errors,
        );
        return Err(errors);
    };

    if !route.retry_eligible && reason.retry_eligible {
        errors.push(format!(
            "decision failure reason route `{}` is not retry eligible",
            route.route
        ));
    }
    if reason.outcome == "blocked" && !reason.blocking {
        errors.push("blocked decision failure reason must be blocking".to_string());
    }

    validate_no_forbidden_terms(
        "Core decision failure reason",
        &contract.forbidden_core_terms,
        core_decision_failure_reason_surface(reason),
        &mut errors,
    );

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

pub fn validate_core_decision_record(
    contract: &CoreDecisionModelContract,
    record: &CoreDecisionRecord,
) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    if record.version != CORE_DECISION_MODEL_CONTRACT_VERSION {
        errors.push(format!(
            "decision record version must be `{}`",
            CORE_DECISION_MODEL_CONTRACT_VERSION
        ));
    }
    if record.decision_id.trim().is_empty() {
        errors.push("decision record id is required".to_string());
    }
    if record.decided_at.trim().is_empty() {
        errors.push("decision record timestamp is required".to_string());
    }
    if record.decided_by.trim().is_empty() {
        errors.push("decision record actor is required".to_string());
    }
    if record.subject.subject_ref_kind.trim().is_empty()
        || record.subject.subject_ref.trim().is_empty()
    {
        errors.push("decision record subject is required".to_string());
    }
    if record.inputs.spec_refs.is_empty() {
        errors.push("decision record must reference spec input".to_string());
    }
    if record.inputs.runtime_state_refs.is_empty() {
        errors.push("decision record must reference runtime state input".to_string());
    }
    if record.inputs.evidence_refs.is_empty() {
        errors.push("decision record must reference evidence input".to_string());
    }
    let allowed_outcomes: BTreeSet<_> = contract
        .outcomes
        .iter()
        .map(|outcome| outcome.outcome.as_str())
        .collect();
    if !allowed_outcomes.contains(record.outcome.as_str()) {
        errors.push(format!(
            "decision record outcome `{}` is not in contract",
            record.outcome
        ));
    }
    if record.reasons.is_empty() {
        errors.push("decision record reasons are required".to_string());
    }
    for reason in &record.reasons {
        if reason.reason_code.trim().is_empty() {
            errors.push("decision reason code is required".to_string());
        }
        if reason.message.trim().is_empty() {
            errors.push("decision reason message is required".to_string());
        }
    }
    let allowed_writes: BTreeSet<_> = contract
        .write_authority
        .may_write
        .iter()
        .map(String::as_str)
        .collect();
    for write in &record.writes {
        if !allowed_writes.contains(write.write_kind.as_str()) {
            errors.push(format!(
                "decision record attempted forbidden write `{}`",
                write.write_kind
            ));
        }
    }

    validate_no_forbidden_terms(
        "Core decision record",
        &contract.forbidden_core_terms,
        core_decision_record_surface(record),
        &mut errors,
    );

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

pub fn validate_core_decision_transition_attempt(
    contract: &CoreDecisionOutcomeTransitionContract,
    attempt: &CoreDecisionTransitionAttempt,
) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    let Some(outcome) = contract
        .outcomes
        .iter()
        .find(|outcome| outcome.outcome == attempt.outcome)
    else {
        errors.push(format!(
            "decision transition outcome `{}` is not in contract",
            attempt.outcome
        ));
        return Err(errors);
    };

    if !outcome
        .allowed_from_states
        .iter()
        .any(|state| state == &attempt.from_state)
    {
        errors.push(format!(
            "decision transition outcome `{}` cannot start from `{}`",
            attempt.outcome, attempt.from_state
        ));
    }
    if !outcome
        .allowed_next_states
        .iter()
        .any(|state| state == &attempt.requested_next_state)
    {
        errors.push(format!(
            "decision transition outcome `{}` cannot route to `{}`",
            attempt.outcome, attempt.requested_next_state
        ));
    }
    if attempt.requested_next_state == "completed" {
        errors.push("decision transition must not write completion state".to_string());
    }
    if attempt.reasons.is_empty() {
        errors.push("decision transition reasons are required".to_string());
    }
    for reason in &attempt.reasons {
        if reason.reason_code.trim().is_empty() {
            errors.push("decision transition reason code is required".to_string());
        }
        if reason.message.trim().is_empty() {
            errors.push("decision transition reason message is required".to_string());
        }
        if reason.evidence_refs.is_empty() {
            errors.push("decision transition reason evidence refs are required".to_string());
        }
    }

    validate_no_forbidden_terms(
        "Core decision transition attempt",
        &contract.forbidden_core_terms,
        core_decision_transition_attempt_surface(attempt),
        &mut errors,
    );

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

pub fn validate_core_decision_input_binding(
    contract: &CoreDecisionInputBindingContract,
    binding: &CoreDecisionInputBinding,
) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    if binding.version != CORE_DECISION_INPUT_BINDING_CONTRACT_VERSION {
        errors.push(format!(
            "decision input binding version must be `{}`",
            CORE_DECISION_INPUT_BINDING_CONTRACT_VERSION
        ));
    }
    if binding.binding_id.trim().is_empty() {
        errors.push("decision input binding id is required".to_string());
    }
    if binding.decision_id.trim().is_empty() {
        errors.push("decision input binding decision id is required".to_string());
    }

    validate_required_ref(
        contract,
        "specBundle",
        "SpecBundleRef",
        binding.spec_bundle_ref.as_ref(),
        &mut errors,
    );
    validate_required_ref_slice(
        contract,
        "ontologyObject",
        "OntologyObjectRef",
        &binding.ontology_object_refs,
        &mut errors,
    );
    validate_required_ref(
        contract,
        "runtimeActionState",
        "RuntimeActionStateRef",
        binding.runtime_action_state_ref.as_ref(),
        &mut errors,
    );
    validate_required_ref_slice(
        contract,
        "evidencePack",
        "EvidencePackRef",
        &binding.evidence_pack_refs,
        &mut errors,
    );
    for delivery_context_ref in &binding.delivery_context_refs {
        validate_bound_ref(
            contract,
            "deliveryContext",
            "DeliveryContextRef",
            delivery_context_ref,
            &mut errors,
        );
    }

    validate_no_forbidden_terms(
        "Core decision input binding",
        &contract.forbidden_core_terms,
        core_decision_input_binding_surface(binding),
        &mut errors,
    );

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreEvidenceReferenceDefinition {
    pub evidence_type: String,
    pub accepted_ref_kind: String,
    pub required_for_actions: Vec<String>,
    pub minimum_count: usize,
    pub validation_rule: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreDecisionOutcomeDefinition {
    pub outcome: String,
    pub resulting_state: String,
    pub route_label: String,
    pub required_evidence_types: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreDecisionReferenceDefinition {
    pub decision_type: String,
    pub accepted_ref_kind: String,
    pub applies_to_actions: Vec<String>,
    pub outcomes: Vec<CoreDecisionOutcomeDefinition>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreEvidenceDecisionReferenceModelContract {
    pub version: String,
    pub status: String,
    pub authority: String,
    pub reference_mapping_boundary: String,
    pub evidence_references: Vec<CoreEvidenceReferenceDefinition>,
    pub decision_references: Vec<CoreDecisionReferenceDefinition>,
    pub forbidden_core_terms: Vec<String>,
}

pub fn core_evidence_decision_reference_model_contract(
) -> CoreEvidenceDecisionReferenceModelContract {
    CoreEvidenceDecisionReferenceModelContract {
        version: CORE_EVIDENCE_DECISION_MODEL_VERSION.to_string(),
        status: "active".to_string(),
        authority: "Core evidence and decision references describe generic proof and judgment flow."
            .to_string(),
        reference_mapping_boundary:
            "Reference App mappings may translate evidence and decisions into domain records, but mappings are not Core authority."
                .to_string(),
        evidence_references: vec![
            evidence(
                "intentEvidence",
                "EvidenceRef",
                vec!["captureObject", "normalizeObject", "routeObject"],
                1,
                "intent proof must identify source object and normalized summary",
            ),
            evidence(
                "decisionEvidence",
                "DecisionRef",
                vec![
                    "acceptObject",
                    "blockObject",
                    "cancelObject",
                    "supersedeObject",
                    "completeObject",
                ],
                1,
                "decision proof must identify actor, reason, and outcome",
            ),
            evidence(
                "progressEvidence",
                "EvidenceRef",
                vec!["attachEvidence", "submitForReview", "completeObject"],
                1,
                "progress proof must identify action, subject object, and result",
            ),
            evidence(
                "artifactEvidence",
                "ArtifactRef",
                vec!["attachArtifact"],
                1,
                "artifact proof must identify durable output reference and producer",
            ),
            evidence(
                "reviewEvidence",
                "EvidenceRef",
                vec!["submitForReview", "completeObject", "blockObject"],
                1,
                "review proof must identify reviewed object and conclusion",
            ),
        ],
        decision_references: vec![
            decision(
                "boundaryDecision",
                vec!["acceptObject"],
                vec![
                    outcome("accepted", "ready", "continue", vec!["decisionEvidence"]),
                    outcome("rejected", "cancelled", "stop", vec!["decisionEvidence"]),
                    outcome(
                        "needsMoreInput",
                        "understood",
                        "clarify",
                        vec!["intentEvidence", "decisionEvidence"],
                    ),
                ],
            ),
            decision(
                "routeDecision",
                vec!["routeObject", "supersedeObject"],
                vec![
                    outcome("routeSelected", "planned", "continue", vec!["intentEvidence"]),
                    outcome(
                        "routeDeferred",
                        "blocked",
                        "wait",
                        vec!["intentEvidence", "decisionEvidence"],
                    ),
                    outcome(
                        "replacementSelected",
                        "superseded",
                        "replace",
                        vec!["decisionEvidence"],
                    ),
                ],
            ),
            decision(
                "completionDecision",
                vec!["completeObject", "blockObject", "cancelObject"],
                vec![
                    outcome(
                        "completed",
                        "completed",
                        "finish",
                        vec!["progressEvidence", "decisionEvidence"],
                    ),
                    outcome(
                        "followUpRequired",
                        "active",
                        "continue",
                        vec!["reviewEvidence", "decisionEvidence"],
                    ),
                    outcome("blocked", "blocked", "wait", vec!["decisionEvidence"]),
                    outcome("cancelled", "cancelled", "stop", vec!["decisionEvidence"]),
                ],
            ),
        ],
        forbidden_core_terms: vec![
            "bug".to_string(),
            "feature".to_string(),
            "issue".to_string(),
            "pr".to_string(),
            "pull-request".to_string(),
            "release".to_string(),
            "repository".to_string(),
            "repository-patch".to_string(),
            "test-log".to_string(),
            "github-issue".to_string(),
        ],
    }
}

pub fn validate_core_evidence_decision_reference_model_contract(
    contract: &CoreEvidenceDecisionReferenceModelContract,
) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    if contract.version != CORE_EVIDENCE_DECISION_MODEL_VERSION {
        errors.push(format!(
            "evidence decision model version must be `{}`",
            CORE_EVIDENCE_DECISION_MODEL_VERSION
        ));
    }
    if contract.status != "active" {
        errors.push("evidence decision model status must be active".to_string());
    }
    if !contract
        .reference_mapping_boundary
        .contains("not Core authority")
    {
        errors.push(
            "reference mapping boundary must say mappings are not Core authority".to_string(),
        );
    }

    let allowed_actions: BTreeSet<_> = [
        "captureObject",
        "normalizeObject",
        "routeObject",
        "acceptObject",
        "startObject",
        "attachEvidence",
        "attachArtifact",
        "submitForReview",
        "completeObject",
        "blockObject",
        "cancelObject",
        "supersedeObject",
    ]
    .into_iter()
    .collect();
    let allowed_states: BTreeSet<_> = [
        "captured",
        "understood",
        "planned",
        "ready",
        "active",
        "reviewing",
        "completed",
        "blocked",
        "cancelled",
        "superseded",
    ]
    .into_iter()
    .collect();
    let evidence_types: BTreeSet<_> = contract
        .evidence_references
        .iter()
        .map(|evidence| evidence.evidence_type.as_str())
        .collect();

    for required_evidence in [
        "intentEvidence",
        "decisionEvidence",
        "progressEvidence",
        "artifactEvidence",
        "reviewEvidence",
    ] {
        if !evidence_types.contains(required_evidence) {
            errors.push(format!(
                "missing Core evidence reference `{required_evidence}`"
            ));
        }
    }

    for evidence in &contract.evidence_references {
        if evidence.minimum_count == 0 {
            errors.push(format!(
                "evidence `{}` minimum count must be greater than zero",
                evidence.evidence_type
            ));
        }
        for action in &evidence.required_for_actions {
            if !allowed_actions.contains(action.as_str()) {
                errors.push(format!(
                    "evidence `{}` references unknown action `{action}`",
                    evidence.evidence_type
                ));
            }
        }
    }

    for decision in &contract.decision_references {
        if decision.outcomes.is_empty() {
            errors.push(format!(
                "decision `{}` must declare outcomes",
                decision.decision_type
            ));
        }
        for action in &decision.applies_to_actions {
            if !allowed_actions.contains(action.as_str()) {
                errors.push(format!(
                    "decision `{}` references unknown action `{action}`",
                    decision.decision_type
                ));
            }
        }
        for outcome in &decision.outcomes {
            if !allowed_states.contains(outcome.resulting_state.as_str()) {
                errors.push(format!(
                    "decision `{}` outcome `{}` references unknown state `{}`",
                    decision.decision_type, outcome.outcome, outcome.resulting_state
                ));
            }
            for evidence_type in &outcome.required_evidence_types {
                if !evidence_types.contains(evidence_type.as_str()) {
                    errors.push(format!(
                        "decision `{}` outcome `{}` references missing evidence `{evidence_type}`",
                        decision.decision_type, outcome.outcome
                    ));
                }
            }
        }
    }

    let core_surface = contract
        .evidence_references
        .iter()
        .flat_map(|evidence| {
            [
                evidence.evidence_type.clone(),
                evidence.accepted_ref_kind.clone(),
                evidence.required_for_actions.join(" "),
                evidence.minimum_count.to_string(),
                evidence.validation_rule.clone(),
            ]
        })
        .chain(contract.decision_references.iter().flat_map(|decision| {
            [
                decision.decision_type.clone(),
                decision.accepted_ref_kind.clone(),
                decision.applies_to_actions.join(" "),
                decision
                    .outcomes
                    .iter()
                    .map(|outcome| outcome.outcome.clone())
                    .collect::<Vec<_>>()
                    .join(" "),
                decision
                    .outcomes
                    .iter()
                    .map(|outcome| outcome.route_label.clone())
                    .collect::<Vec<_>>()
                    .join(" "),
            ]
        }))
        .chain([
            contract.authority.clone(),
            contract.reference_mapping_boundary.clone(),
        ])
        .collect::<Vec<_>>();
    validate_no_forbidden_terms(
        "Core evidence/decision model",
        &contract.forbidden_core_terms,
        core_surface,
        &mut errors,
    );

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn readable_fact(
    fact_kind: &str,
    accepted_ref_kind: &str,
    purpose: &str,
) -> CoreDecisionReadableFact {
    CoreDecisionReadableFact {
        fact_kind: fact_kind.to_string(),
        accepted_ref_kind: accepted_ref_kind.to_string(),
        purpose: purpose.to_string(),
    }
}

fn input_requirement(
    input_kind: &str,
    accepted_ref_kind: &str,
    source_kernel: &str,
    required: bool,
) -> CoreDecisionInputAuthorityRequirement {
    CoreDecisionInputAuthorityRequirement {
        input_kind: input_kind.to_string(),
        accepted_ref_kind: accepted_ref_kind.to_string(),
        source_kernel: source_kernel.to_string(),
        required,
    }
}

fn decision_outcome(outcome: &str, meaning: &str, terminal: bool) -> CoreDecisionKernelOutcome {
    CoreDecisionKernelOutcome {
        outcome: outcome.to_string(),
        meaning: meaning.to_string(),
        terminal,
    }
}

fn outcome_transition(
    outcome: &str,
    meaning: &str,
    terminal: bool,
    allowed_from_states: Vec<&str>,
    allowed_next_states: Vec<&str>,
    required_evidence_refs: Vec<&str>,
) -> CoreDecisionOutcomeTransition {
    CoreDecisionOutcomeTransition {
        outcome: outcome.to_string(),
        meaning: meaning.to_string(),
        terminal,
        allowed_from_states: allowed_from_states
            .into_iter()
            .map(str::to_string)
            .collect(),
        allowed_next_states: allowed_next_states
            .into_iter()
            .map(str::to_string)
            .collect(),
        required_reason_fields: vec![
            "reasonCode".to_string(),
            "message".to_string(),
            "evidenceRefs".to_string(),
            "blocking".to_string(),
        ],
        required_evidence_refs: required_evidence_refs
            .into_iter()
            .map(str::to_string)
            .collect(),
    }
}

fn remediation_route(
    route: &str,
    meaning: &str,
    retry_eligible: bool,
) -> CoreDecisionRemediationRoute {
    CoreDecisionRemediationRoute {
        route: route.to_string(),
        meaning: meaning.to_string(),
        retry_eligible,
    }
}

fn bound_authority_ref(
    ref_kind: &str,
    ref_id: &str,
    authority_path: &str,
    version: &str,
) -> CoreDecisionBoundAuthorityRef {
    CoreDecisionBoundAuthorityRef {
        ref_kind: ref_kind.to_string(),
        ref_id: ref_id.to_string(),
        authority_path: authority_path.to_string(),
        version: version.to_string(),
        observed_at: "2026-06-29T00:00:00Z".to_string(),
        stale: false,
    }
}

fn core_decision_model_surface(contract: &CoreDecisionModelContract) -> Vec<String> {
    contract
        .readable_authority_facts
        .iter()
        .flat_map(|fact| {
            [
                fact.fact_kind.clone(),
                fact.accepted_ref_kind.clone(),
                fact.purpose.clone(),
            ]
        })
        .chain([
            contract.authority.clone(),
            contract.write_authority.may_write.join(" "),
            contract.write_authority.must_not_write.join(" "),
            contract.required_record_fields.join(" "),
        ])
        .chain(contract.outcomes.iter().flat_map(|outcome| {
            [
                outcome.outcome.clone(),
                outcome.meaning.clone(),
                outcome.terminal.to_string(),
            ]
        }))
        .collect()
}

fn core_decision_input_binding_contract_surface(
    contract: &CoreDecisionInputBindingContract,
) -> Vec<String> {
    [
        contract.version.clone(),
        contract.status.clone(),
        contract.authority.clone(),
        contract.freshness_rule.clone(),
        contract.rejected_ref_kinds.join(" "),
    ]
    .into_iter()
    .chain(
        contract
            .required_authority_refs
            .iter()
            .flat_map(|requirement| {
                [
                    requirement.input_kind.clone(),
                    requirement.accepted_ref_kind.clone(),
                    requirement.source_kernel.clone(),
                    requirement.required.to_string(),
                ]
            }),
    )
    .chain(
        contract
            .optional_context_refs
            .iter()
            .flat_map(|requirement| {
                [
                    requirement.input_kind.clone(),
                    requirement.accepted_ref_kind.clone(),
                    requirement.source_kernel.clone(),
                    requirement.required.to_string(),
                ]
            }),
    )
    .collect()
}

fn core_decision_outcome_transition_contract_surface(
    contract: &CoreDecisionOutcomeTransitionContract,
) -> Vec<String> {
    [
        contract.version.clone(),
        contract.status.clone(),
        contract.authority.clone(),
        contract.reason_shape.required_fields.join(" "),
        contract
            .reason_shape
            .machine_readable_reason_code
            .to_string(),
        contract.reason_shape.evidence_refs_required.to_string(),
        contract.illegal_transition_policy.clone(),
    ]
    .into_iter()
    .chain(contract.outcomes.iter().flat_map(|outcome| {
        [
            outcome.outcome.clone(),
            outcome.meaning.clone(),
            outcome.terminal.to_string(),
            outcome.allowed_from_states.join(" "),
            outcome.allowed_next_states.join(" "),
            outcome.required_reason_fields.join(" "),
            outcome.required_evidence_refs.join(" "),
        ]
    }))
    .collect()
}

fn core_decision_failure_reason_contract_surface(
    contract: &CoreDecisionFailureReasonContract,
) -> Vec<String> {
    [
        contract.version.clone(),
        contract.status.clone(),
        contract.authority.clone(),
        contract.applies_to_outcomes.join(" "),
        contract.required_fields.join(" "),
        contract.retry_policy.clone(),
    ]
    .into_iter()
    .chain(contract.remediation_routes.iter().flat_map(|route| {
        [
            route.route.clone(),
            route.meaning.clone(),
            route.retry_eligible.to_string(),
        ]
    }))
    .collect()
}

fn core_decision_record_surface(record: &CoreDecisionRecord) -> Vec<String> {
    [
        record.version.clone(),
        record.decision_id.clone(),
        record.decided_at.clone(),
        record.decided_by.clone(),
        record.subject.subject_ref_kind.clone(),
        record.subject.subject_ref.clone(),
        record.inputs.spec_refs.join(" "),
        record.inputs.runtime_state_refs.join(" "),
        record.inputs.evidence_refs.join(" "),
        record.inputs.prior_decision_refs.join(" "),
        record.outcome.clone(),
    ]
    .into_iter()
    .chain(record.reasons.iter().flat_map(|reason| {
        [
            reason.reason_code.clone(),
            reason.message.clone(),
            reason.evidence_refs.join(" "),
            reason.blocking.to_string(),
        ]
    }))
    .chain(record.writes.iter().flat_map(|write| {
        [
            write.write_kind.clone(),
            write.target_ref.clone(),
            write.authority_boundary.clone(),
        ]
    }))
    .collect()
}

fn core_decision_failure_reason_surface(reason: &CoreDecisionFailureReason) -> Vec<String> {
    vec![
        reason.outcome.clone(),
        reason.reason_code.clone(),
        reason.message.clone(),
        reason.authority_refs.join(" "),
        reason.missing_evidence_refs.join(" "),
        reason.remediation_route.clone(),
        reason.retry_eligible.to_string(),
        reason.blocking.to_string(),
    ]
}

fn core_decision_transition_attempt_surface(
    attempt: &CoreDecisionTransitionAttempt,
) -> Vec<String> {
    [
        attempt.outcome.clone(),
        attempt.from_state.clone(),
        attempt.requested_next_state.clone(),
    ]
    .into_iter()
    .chain(attempt.reasons.iter().flat_map(|reason| {
        [
            reason.reason_code.clone(),
            reason.message.clone(),
            reason.evidence_refs.join(" "),
            reason.blocking.to_string(),
        ]
    }))
    .collect()
}

fn core_decision_input_binding_surface(binding: &CoreDecisionInputBinding) -> Vec<String> {
    [
        binding.version.clone(),
        binding.binding_id.clone(),
        binding.decision_id.clone(),
    ]
    .into_iter()
    .chain(
        binding
            .spec_bundle_ref
            .iter()
            .flat_map(core_decision_bound_ref_surface),
    )
    .chain(
        binding
            .ontology_object_refs
            .iter()
            .flat_map(core_decision_bound_ref_surface),
    )
    .chain(
        binding
            .runtime_action_state_ref
            .iter()
            .flat_map(core_decision_bound_ref_surface),
    )
    .chain(
        binding
            .evidence_pack_refs
            .iter()
            .flat_map(core_decision_bound_ref_surface),
    )
    .chain(
        binding
            .delivery_context_refs
            .iter()
            .flat_map(core_decision_bound_ref_surface),
    )
    .collect()
}

fn core_decision_bound_ref_surface(ref_value: &CoreDecisionBoundAuthorityRef) -> Vec<String> {
    vec![
        ref_value.ref_kind.clone(),
        ref_value.ref_id.clone(),
        ref_value.authority_path.clone(),
        ref_value.version.clone(),
        ref_value.observed_at.clone(),
        ref_value.stale.to_string(),
    ]
}

fn validate_required_ref(
    contract: &CoreDecisionInputBindingContract,
    input_kind: &str,
    accepted_ref_kind: &str,
    ref_value: Option<&CoreDecisionBoundAuthorityRef>,
    errors: &mut Vec<String>,
) {
    match ref_value {
        Some(ref_value) => {
            validate_bound_ref(contract, input_kind, accepted_ref_kind, ref_value, errors)
        }
        None => errors.push(format!(
            "decision input binding missing required authority ref `{input_kind}`"
        )),
    }
}

fn validate_required_ref_slice(
    contract: &CoreDecisionInputBindingContract,
    input_kind: &str,
    accepted_ref_kind: &str,
    ref_values: &[CoreDecisionBoundAuthorityRef],
    errors: &mut Vec<String>,
) {
    if ref_values.is_empty() {
        errors.push(format!(
            "decision input binding missing required authority ref `{input_kind}`"
        ));
        return;
    }

    for ref_value in ref_values {
        validate_bound_ref(contract, input_kind, accepted_ref_kind, ref_value, errors);
    }
}

fn validate_bound_ref(
    contract: &CoreDecisionInputBindingContract,
    input_kind: &str,
    accepted_ref_kind: &str,
    ref_value: &CoreDecisionBoundAuthorityRef,
    errors: &mut Vec<String>,
) {
    if contract
        .rejected_ref_kinds
        .iter()
        .any(|rejected| rejected == &ref_value.ref_kind)
    {
        errors.push(format!(
            "decision input binding rejected ref kind `{}` for `{input_kind}`",
            ref_value.ref_kind
        ));
    }

    if ref_value.ref_kind != accepted_ref_kind {
        errors.push(format!(
            "decision input binding expected `{accepted_ref_kind}` for `{input_kind}` but got `{}`",
            ref_value.ref_kind
        ));
    }

    if ref_value.ref_id.trim().is_empty() {
        errors.push(format!(
            "decision input binding `{input_kind}` authority ref id is required"
        ));
    }
    if ref_value.authority_path.trim().is_empty() {
        errors.push(format!(
            "decision input binding `{input_kind}` authority path is required"
        ));
    }
    if ref_value.version.trim().is_empty() {
        errors.push(format!(
            "decision input binding `{input_kind}` authority version is required"
        ));
    }
    if ref_value.observed_at.trim().is_empty() {
        errors.push(format!(
            "decision input binding `{input_kind}` observed timestamp is required"
        ));
    }
    if ref_value.stale {
        errors.push(format!(
            "decision input binding stale authority ref `{input_kind}`"
        ));
    }
    if ref_value
        .authority_path
        .starts_with(".agentflow/projections/")
        || ref_value
            .authority_path
            .starts_with(".agentflow/provider-sessions/")
    {
        errors.push(format!(
            "decision input binding `{input_kind}` must not read projection or provider session authority"
        ));
    }
}

fn validate_no_forbidden_terms(
    context: &str,
    forbidden_terms: &[String],
    surface: Vec<String>,
    errors: &mut Vec<String>,
) {
    for term in forbidden_terms {
        if surface
            .iter()
            .any(|value| contains_forbidden_core_term(value, term))
        {
            errors.push(format!(
                "forbidden industry term `{term}` appears in {context}"
            ));
        }
    }
}

fn evidence(
    evidence_type: &str,
    accepted_ref_kind: &str,
    required_for_actions: Vec<&str>,
    minimum_count: usize,
    validation_rule: &str,
) -> CoreEvidenceReferenceDefinition {
    CoreEvidenceReferenceDefinition {
        evidence_type: evidence_type.to_string(),
        accepted_ref_kind: accepted_ref_kind.to_string(),
        required_for_actions: required_for_actions
            .into_iter()
            .map(str::to_string)
            .collect(),
        minimum_count,
        validation_rule: validation_rule.to_string(),
    }
}

fn decision(
    decision_type: &str,
    applies_to_actions: Vec<&str>,
    outcomes: Vec<CoreDecisionOutcomeDefinition>,
) -> CoreDecisionReferenceDefinition {
    CoreDecisionReferenceDefinition {
        decision_type: decision_type.to_string(),
        accepted_ref_kind: "DecisionRef".to_string(),
        applies_to_actions: applies_to_actions.into_iter().map(str::to_string).collect(),
        outcomes,
    }
}

fn outcome(
    outcome: &str,
    resulting_state: &str,
    route_label: &str,
    required_evidence_types: Vec<&str>,
) -> CoreDecisionOutcomeDefinition {
    CoreDecisionOutcomeDefinition {
        outcome: outcome.to_string(),
        resulting_state: resulting_state.to_string(),
        route_label: route_label.to_string(),
        required_evidence_types: required_evidence_types
            .into_iter()
            .map(str::to_string)
            .collect(),
    }
}

fn contains_forbidden_core_term(value: &str, term: &str) -> bool {
    let normalized_term = normalized_compact(term);
    if normalized_term.is_empty() {
        return false;
    }

    if normalized_term.len() <= 2 {
        return tokenized(value)
            .iter()
            .any(|token| token == &normalized_term);
    }

    normalized_compact(value).contains(&normalized_term)
}

fn normalized_compact(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(|character| character.to_lowercase())
        .collect()
}

fn tokenized(value: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();

    for character in value.chars() {
        if character.is_ascii_alphanumeric() {
            current.extend(character.to_lowercase());
        } else if !current.is_empty() {
            tokens.push(std::mem::take(&mut current));
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantics::core_action_state_semantics_contract;

    #[test]
    fn core_decision_model_contract_validates() {
        let contract = core_decision_model_contract();
        validate_core_decision_model_contract(&contract).unwrap();
        assert_eq!(contract.version, CORE_DECISION_MODEL_CONTRACT_VERSION);
        assert!(contract
            .required_record_fields
            .iter()
            .any(|field| field == "decisionId"));
        assert!(contract
            .readable_authority_facts
            .iter()
            .any(|fact| fact.fact_kind == "evidence"));
        assert!(contract
            .outcomes
            .iter()
            .any(|outcome| outcome.outcome == "needs-fix"));
    }

    #[test]
    fn core_decision_record_fixture_validates() {
        let contract = core_decision_model_contract();
        let record = canonical_core_decision_record_fixture();
        validate_core_decision_record(&contract, &record).unwrap();
        assert_eq!(record.outcome, "accepted");
        assert_eq!(record.writes.len(), 2);
    }

    #[test]
    fn core_decision_record_rejects_unknown_outcome() {
        let contract = core_decision_model_contract();
        let mut record = canonical_core_decision_record_fixture();
        record.outcome = "unknown".to_string();

        let errors = validate_core_decision_record(&contract, &record).unwrap_err();
        assert!(errors
            .iter()
            .any(|error| error.contains("is not in contract")));
    }

    #[test]
    fn core_decision_record_rejects_forbidden_industry_term() {
        let contract = core_decision_model_contract();
        let mut record = canonical_core_decision_record_fixture();
        record.reasons[0]
            .message
            .push_str(" This must not mention github issue.");

        let errors = validate_core_decision_record(&contract, &record).unwrap_err();
        assert!(errors.iter().any(|error| error.contains("github-issue")));
    }

    #[test]
    fn core_decision_outcome_transition_contract_validates() {
        let contract = core_decision_outcome_transition_contract();
        let semantics = core_action_state_semantics_contract();
        validate_core_decision_outcome_transition_contract(&contract, &semantics).unwrap();
        assert!(contract
            .outcomes
            .iter()
            .any(|outcome| outcome.outcome == "needs-fix"));
        assert!(contract.outcomes.iter().all(|outcome| !outcome
            .allowed_next_states
            .contains(&"completed".to_string())));
    }

    #[test]
    fn core_decision_transition_attempt_fixture_validates() {
        let contract = core_decision_outcome_transition_contract();
        let attempt = canonical_core_decision_transition_attempt_fixture();
        validate_core_decision_transition_attempt(&contract, &attempt).unwrap();
        assert_eq!(attempt.outcome, "accepted");
        assert_eq!(attempt.requested_next_state, "ready");
    }

    #[test]
    fn core_decision_transition_rejects_completion_write() {
        let contract = core_decision_outcome_transition_contract();
        let mut attempt = canonical_core_decision_transition_attempt_fixture();
        attempt.requested_next_state = "completed".to_string();

        let errors = validate_core_decision_transition_attempt(&contract, &attempt).unwrap_err();
        assert!(errors
            .iter()
            .any(|error| error.contains("must not write completion state")));
    }

    #[test]
    fn core_decision_transition_rejects_unknown_outcome() {
        let contract = core_decision_outcome_transition_contract();
        let mut attempt = canonical_core_decision_transition_attempt_fixture();
        attempt.outcome = "unknown".to_string();

        let errors = validate_core_decision_transition_attempt(&contract, &attempt).unwrap_err();
        assert!(errors
            .iter()
            .any(|error| error.contains("is not in contract")));
    }

    #[test]
    fn core_decision_transition_rejects_missing_reason() {
        let contract = core_decision_outcome_transition_contract();
        let mut attempt = canonical_core_decision_transition_attempt_fixture();
        attempt.reasons.clear();

        let errors = validate_core_decision_transition_attempt(&contract, &attempt).unwrap_err();
        assert!(errors
            .iter()
            .any(|error| error.contains("reasons are required")));
    }

    #[test]
    fn core_decision_failure_reason_contract_validates() {
        let contract = core_decision_failure_reason_contract();
        let outcome_contract = core_decision_outcome_transition_contract();
        validate_core_decision_failure_reason_contract(&contract, &outcome_contract).unwrap();
        assert_eq!(
            contract.version,
            CORE_DECISION_FAILURE_REASON_CONTRACT_VERSION
        );
        assert!(contract
            .applies_to_outcomes
            .iter()
            .any(|outcome| outcome == "needs-fix"));
        assert!(contract
            .required_fields
            .iter()
            .any(|field| field == "missingEvidenceRefs"));
    }

    #[test]
    fn core_decision_failure_reason_fixture_validates() {
        let contract = core_decision_failure_reason_contract();
        let reason = canonical_core_decision_failure_reason_fixture();
        validate_core_decision_failure_reason(&contract, &reason).unwrap();
        assert_eq!(reason.outcome, "blocked");
        assert!(reason.blocking);
    }

    #[test]
    fn core_decision_failure_reason_rejects_accepted_outcome() {
        let contract = core_decision_failure_reason_contract();
        let mut reason = canonical_core_decision_failure_reason_fixture();
        reason.outcome = "accepted".to_string();

        let errors = validate_core_decision_failure_reason(&contract, &reason).unwrap_err();
        assert!(errors
            .iter()
            .any(|error| error.contains("must not be attached to accepted")));
    }

    #[test]
    fn core_decision_failure_reason_rejects_missing_authority_refs() {
        let contract = core_decision_failure_reason_contract();
        let mut reason = canonical_core_decision_failure_reason_fixture();
        reason.authority_refs.clear();

        let errors = validate_core_decision_failure_reason(&contract, &reason).unwrap_err();
        assert!(errors
            .iter()
            .any(|error| error.contains("authority refs are required")));
    }

    #[test]
    fn core_decision_failure_reason_rejects_missing_evidence_refs() {
        let contract = core_decision_failure_reason_contract();
        let mut reason = canonical_core_decision_failure_reason_fixture();
        reason.missing_evidence_refs.clear();

        let errors = validate_core_decision_failure_reason(&contract, &reason).unwrap_err();
        assert!(errors
            .iter()
            .any(|error| error.contains("missing evidence refs are required")));
    }

    #[test]
    fn core_decision_failure_reason_rejects_unknown_remediation_route() {
        let contract = core_decision_failure_reason_contract();
        let mut reason = canonical_core_decision_failure_reason_fixture();
        reason.remediation_route = "unknown-route".to_string();

        let errors = validate_core_decision_failure_reason(&contract, &reason).unwrap_err();
        assert!(errors
            .iter()
            .any(|error| error.contains("is not in contract")));
    }

    #[test]
    fn core_decision_failure_reason_rejects_invalid_retry_eligibility() {
        let contract = core_decision_failure_reason_contract();
        let mut reason = canonical_core_decision_failure_reason_fixture();
        reason.outcome = "rejected".to_string();
        reason.remediation_route = "cancel-subject".to_string();
        reason.retry_eligible = true;
        reason.blocking = false;

        let errors = validate_core_decision_failure_reason(&contract, &reason).unwrap_err();
        assert!(errors
            .iter()
            .any(|error| error.contains("is not retry eligible")));
    }

    #[test]
    fn core_decision_input_binding_contract_validates() {
        let contract = core_decision_input_binding_contract();
        validate_core_decision_input_binding_contract(&contract).unwrap();
        assert!(contract
            .required_authority_refs
            .iter()
            .any(|requirement| requirement.accepted_ref_kind == "SpecBundleRef"));
        assert!(contract
            .rejected_ref_kinds
            .iter()
            .any(|ref_kind| ref_kind == "ProjectionRef"));
    }

    #[test]
    fn core_decision_input_binding_fixture_validates() {
        let contract = core_decision_input_binding_contract();
        let binding = canonical_core_decision_input_binding_fixture();
        validate_core_decision_input_binding(&contract, &binding).unwrap();
        assert_eq!(
            binding.version,
            CORE_DECISION_INPUT_BINDING_CONTRACT_VERSION
        );
        assert_eq!(binding.evidence_pack_refs.len(), 1);
    }

    #[test]
    fn core_decision_input_binding_rejects_missing_spec() {
        let contract = core_decision_input_binding_contract();
        let mut binding = canonical_core_decision_input_binding_fixture();
        binding.spec_bundle_ref = None;

        let errors = validate_core_decision_input_binding(&contract, &binding).unwrap_err();
        assert!(errors
            .iter()
            .any(|error| error.contains("missing required authority ref `specBundle`")));
    }

    #[test]
    fn core_decision_input_binding_rejects_stale_runtime_state() {
        let contract = core_decision_input_binding_contract();
        let mut binding = canonical_core_decision_input_binding_fixture();
        binding.runtime_action_state_ref.as_mut().unwrap().stale = true;

        let errors = validate_core_decision_input_binding(&contract, &binding).unwrap_err();
        assert!(errors
            .iter()
            .any(|error| error.contains("stale authority ref `runtimeActionState`")));
    }

    #[test]
    fn core_decision_input_binding_rejects_projection_only_ref() {
        let contract = core_decision_input_binding_contract();
        let mut binding = canonical_core_decision_input_binding_fixture();
        binding.evidence_pack_refs[0].ref_kind = "ProjectionRef".to_string();
        binding.evidence_pack_refs[0].authority_path =
            ".agentflow/projections/tasks/task-core.json".to_string();

        let errors = validate_core_decision_input_binding(&contract, &binding).unwrap_err();
        assert!(errors
            .iter()
            .any(|error| error.contains("rejected ref kind `ProjectionRef`")));
        assert!(errors
            .iter()
            .any(|error| error.contains("must not read projection or provider session")));
    }

    #[test]
    fn core_decision_input_binding_rejects_provider_session_ref() {
        let contract = core_decision_input_binding_contract();
        let mut binding = canonical_core_decision_input_binding_fixture();
        binding.spec_bundle_ref.as_mut().unwrap().ref_kind = "ProviderSessionRef".to_string();

        let errors = validate_core_decision_input_binding(&contract, &binding).unwrap_err();
        assert!(errors
            .iter()
            .any(|error| error.contains("rejected ref kind `ProviderSessionRef`")));
    }

    #[test]
    fn core_evidence_decision_reference_model_contract_validates() {
        let contract = core_evidence_decision_reference_model_contract();
        validate_core_evidence_decision_reference_model_contract(&contract).unwrap();
        assert_eq!(contract.evidence_references.len(), 5);
        assert_eq!(contract.decision_references.len(), 3);
    }

    #[test]
    fn core_evidence_decision_model_rejects_unknown_action_reference() {
        let mut contract = core_evidence_decision_reference_model_contract();
        contract.evidence_references[0]
            .required_for_actions
            .push("unknownAction".to_string());

        let errors =
            validate_core_evidence_decision_reference_model_contract(&contract).unwrap_err();
        assert!(errors.iter().any(|error| error.contains("unknown action")));
    }

    #[test]
    fn core_evidence_decision_model_rejects_unknown_evidence_reference() {
        let mut contract = core_evidence_decision_reference_model_contract();
        contract.decision_references[0].outcomes[0]
            .required_evidence_types
            .push("missingEvidence".to_string());

        let errors =
            validate_core_evidence_decision_reference_model_contract(&contract).unwrap_err();
        assert!(errors
            .iter()
            .any(|error| error.contains("missing evidence")));
    }

    #[test]
    fn core_evidence_decision_model_rejects_industry_pollution() {
        let mut contract = core_evidence_decision_reference_model_contract();
        contract.evidence_references[0]
            .validation_rule
            .push_str(" This must not become a test log.");

        let errors =
            validate_core_evidence_decision_reference_model_contract(&contract).unwrap_err();
        assert!(errors.iter().any(|error| error.contains("test-log")));
    }
}
