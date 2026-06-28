use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const CORE_EVIDENCE_DECISION_MODEL_VERSION: &str =
    "agentflow-core-evidence-decision-reference-model.v1";

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
    for term in &contract.forbidden_core_terms {
        if core_surface
            .iter()
            .any(|value| contains_forbidden_core_term(value, term))
        {
            errors.push(format!(
                "forbidden industry term `{term}` appears in Core evidence/decision model"
            ));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
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
