use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const CORE_DECISION_MODEL_CONTRACT_VERSION: &str = "agentflow-core-decision-model.v1";
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
            decision_outcome("cancelled", "the subject was intentionally stopped", true),
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
    for outcome in ["accepted", "rejected", "deferred", "blocked", "cancelled"] {
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

fn decision_outcome(outcome: &str, meaning: &str, terminal: bool) -> CoreDecisionKernelOutcome {
    CoreDecisionKernelOutcome {
        outcome: outcome.to_string(),
        meaning: meaning.to_string(),
        terminal,
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
