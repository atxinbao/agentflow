use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const CORE_ACTION_STATE_SEMANTICS_VERSION: &str = "agentflow-core-action-state-semantics.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreActionSemanticDefinition {
    pub action_type: String,
    pub category: String,
    pub target_object_type: String,
    pub description: String,
    pub required_state: Option<String>,
    pub resulting_state: Option<String>,
    pub emitted_event: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreStateSemanticDefinition {
    pub state_id: String,
    pub description: String,
    pub is_terminal: bool,
    pub is_blocking: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreStateTransitionDefinition {
    pub transition_id: String,
    pub action_type: String,
    pub from_states: Vec<String>,
    pub to_state: String,
    pub required_evidence: Vec<String>,
    pub emitted_event: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreActionStateSemanticsContract {
    pub version: String,
    pub status: String,
    pub authority: String,
    pub reference_mapping_boundary: String,
    pub actions: Vec<CoreActionSemanticDefinition>,
    pub states: Vec<CoreStateSemanticDefinition>,
    pub transitions: Vec<CoreStateTransitionDefinition>,
    pub forbidden_core_terms: Vec<String>,
}

pub fn core_action_state_semantics_contract() -> CoreActionStateSemanticsContract {
    CoreActionStateSemanticsContract {
        version: CORE_ACTION_STATE_SEMANTICS_VERSION.to_string(),
        status: "active".to_string(),
        authority: "Core action and state semantics describe generic object lifecycle behavior."
            .to_string(),
        reference_mapping_boundary:
            "Reference App mappings may translate these semantics into domain vocabulary, but mappings are not Core authority."
                .to_string(),
        actions: vec![
            action("captureObject", "intake", "RequestObject", "Capture a new object from human or system input.", None, Some("captured"), "ObjectCaptured"),
            action("normalizeObject", "intake", "IntentObject", "Normalize object content into a structured form.", Some("captured"), Some("understood"), "ObjectNormalized"),
            action("routeObject", "route", "IntentObject", "Route an understood object toward a goal boundary.", Some("understood"), Some("planned"), "ObjectRouted"),
            action("acceptObject", "decision", "DecisionObject", "Record that an object boundary is accepted.", Some("planned"), Some("ready"), "ObjectAccepted"),
            action("startObject", "execution", "ExecutionObject", "Start controlled work for a ready object.", Some("ready"), Some("active"), "ObjectStarted"),
            action("attachEvidence", "evidence", "EvidenceObject", "Attach proof to support an object transition.", Some("active"), Some("active"), "EvidenceAttached"),
            action("attachArtifact", "artifact", "ArtifactObject", "Attach a durable output reference.", Some("active"), Some("active"), "ArtifactAttached"),
            action("submitForReview", "review", "ReviewObject", "Move active work into independent review.", Some("active"), Some("reviewing"), "ObjectSubmittedForReview"),
            action("completeObject", "completion", "DecisionObject", "Record completion after required proof is present.", Some("reviewing"), Some("completed"), "ObjectCompleted"),
            action("blockObject", "exception", "DecisionObject", "Record that an external condition prevents progress.", None, Some("blocked"), "ObjectBlocked"),
            action("cancelObject", "exception", "DecisionObject", "Record that the object should not continue.", None, Some("cancelled"), "ObjectCancelled"),
            action("supersedeObject", "exception", "DecisionObject", "Record that another object has replaced this one.", None, Some("superseded"), "ObjectSuperseded"),
        ],
        states: vec![
            state("captured", "Object was captured but not yet normalized.", false, false),
            state("understood", "Object meaning is normalized and can be routed.", false, false),
            state("planned", "Object has a proposed path but is not ready to run.", false, false),
            state("ready", "Object is ready for controlled work.", false, false),
            state("active", "Object is currently being worked on.", false, false),
            state("reviewing", "Object is waiting for independent review or acceptance.", false, false),
            state("completed", "Object reached accepted completion.", true, false),
            state("blocked", "Object cannot progress until an external blocker is cleared.", false, true),
            state("cancelled", "Object was stopped and should not continue.", true, false),
            state("superseded", "Object was replaced by another object.", true, false),
        ],
        transitions: vec![
            transition("capture", "captureObject", vec![], "captured", vec![], "ObjectCaptured"),
            transition("normalize", "normalizeObject", vec!["captured"], "understood", vec![], "ObjectNormalized"),
            transition("route", "routeObject", vec!["understood"], "planned", vec![], "ObjectRouted"),
            transition("accept", "acceptObject", vec!["planned"], "ready", vec!["DecisionRef"], "ObjectAccepted"),
            transition("start", "startObject", vec!["ready"], "active", vec![], "ObjectStarted"),
            transition("attach-evidence", "attachEvidence", vec!["active"], "active", vec!["EvidenceRef"], "EvidenceAttached"),
            transition("attach-artifact", "attachArtifact", vec!["active"], "active", vec!["ArtifactRef"], "ArtifactAttached"),
            transition("submit-review", "submitForReview", vec!["active"], "reviewing", vec!["EvidenceRef"], "ObjectSubmittedForReview"),
            transition("complete", "completeObject", vec!["reviewing"], "completed", vec!["EvidenceRef", "DecisionRef"], "ObjectCompleted"),
            transition("block", "blockObject", vec!["captured", "understood", "planned", "ready", "active", "reviewing"], "blocked", vec!["DecisionRef"], "ObjectBlocked"),
            transition("cancel", "cancelObject", vec!["captured", "understood", "planned", "ready", "active", "reviewing", "blocked"], "cancelled", vec!["DecisionRef"], "ObjectCancelled"),
            transition("supersede", "supersedeObject", vec!["captured", "understood", "planned", "ready", "active", "reviewing", "blocked"], "superseded", vec!["DecisionRef"], "ObjectSuperseded"),
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

pub fn validate_core_action_state_semantics_contract(
    contract: &CoreActionStateSemanticsContract,
) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    if contract.version != CORE_ACTION_STATE_SEMANTICS_VERSION {
        errors.push(format!(
            "semantics version must be `{}`",
            CORE_ACTION_STATE_SEMANTICS_VERSION
        ));
    }
    if contract.status != "active" {
        errors.push("semantics status must be active".to_string());
    }
    if !contract
        .reference_mapping_boundary
        .contains("not Core authority")
    {
        errors.push(
            "reference mapping boundary must say mappings are not Core authority".to_string(),
        );
    }

    let action_ids: BTreeSet<_> = contract
        .actions
        .iter()
        .map(|action| action.action_type.as_str())
        .collect();
    let state_ids: BTreeSet<_> = contract
        .states
        .iter()
        .map(|state| state.state_id.as_str())
        .collect();

    for required_action in [
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
    ] {
        if !action_ids.contains(required_action) {
            errors.push(format!("missing Core action semantic `{required_action}`"));
        }
    }
    for required_state in [
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
    ] {
        if !state_ids.contains(required_state) {
            errors.push(format!("missing Core state semantic `{required_state}`"));
        }
    }

    for action in &contract.actions {
        if let Some(required_state) = &action.required_state {
            if !state_ids.contains(required_state.as_str()) {
                errors.push(format!(
                    "action `{}` references missing required state `{required_state}`",
                    action.action_type
                ));
            }
        }
        if let Some(resulting_state) = &action.resulting_state {
            if !state_ids.contains(resulting_state.as_str()) {
                errors.push(format!(
                    "action `{}` references missing resulting state `{resulting_state}`",
                    action.action_type
                ));
            }
        }
    }

    for transition in &contract.transitions {
        if !action_ids.contains(transition.action_type.as_str()) {
            errors.push(format!(
                "transition `{}` references missing action `{}`",
                transition.transition_id, transition.action_type
            ));
        }
        if !state_ids.contains(transition.to_state.as_str()) {
            errors.push(format!(
                "transition `{}` references missing target state `{}`",
                transition.transition_id, transition.to_state
            ));
        }
        for from_state in &transition.from_states {
            if !state_ids.contains(from_state.as_str()) {
                errors.push(format!(
                    "transition `{}` references missing source state `{from_state}`",
                    transition.transition_id
                ));
            }
        }
    }

    let core_surface = contract
        .actions
        .iter()
        .flat_map(|action| {
            [
                action.action_type.clone(),
                action.category.clone(),
                action.target_object_type.clone(),
                action.description.clone(),
                action.emitted_event.clone(),
            ]
        })
        .chain(contract.states.iter().flat_map(|state| {
            [
                state.state_id.clone(),
                state.description.clone(),
                state.is_terminal.to_string(),
                state.is_blocking.to_string(),
                String::new(),
            ]
        }))
        .chain(contract.transitions.iter().flat_map(|transition| {
            [
                transition.transition_id.clone(),
                transition.action_type.clone(),
                transition.from_states.join(" "),
                transition.to_state.clone(),
                transition.emitted_event.clone(),
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
                "forbidden industry term `{term}` appears in Core action/state semantics"
            ));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn action(
    action_type: &str,
    category: &str,
    target_object_type: &str,
    description: &str,
    required_state: Option<&str>,
    resulting_state: Option<&str>,
    emitted_event: &str,
) -> CoreActionSemanticDefinition {
    CoreActionSemanticDefinition {
        action_type: action_type.to_string(),
        category: category.to_string(),
        target_object_type: target_object_type.to_string(),
        description: description.to_string(),
        required_state: required_state.map(str::to_string),
        resulting_state: resulting_state.map(str::to_string),
        emitted_event: emitted_event.to_string(),
    }
}

fn state(
    state_id: &str,
    description: &str,
    is_terminal: bool,
    is_blocking: bool,
) -> CoreStateSemanticDefinition {
    CoreStateSemanticDefinition {
        state_id: state_id.to_string(),
        description: description.to_string(),
        is_terminal,
        is_blocking,
    }
}

fn transition(
    transition_id: &str,
    action_type: &str,
    from_states: Vec<&str>,
    to_state: &str,
    required_evidence: Vec<&str>,
    emitted_event: &str,
) -> CoreStateTransitionDefinition {
    CoreStateTransitionDefinition {
        transition_id: transition_id.to_string(),
        action_type: action_type.to_string(),
        from_states: from_states.into_iter().map(str::to_string).collect(),
        to_state: to_state.to_string(),
        required_evidence: required_evidence.into_iter().map(str::to_string).collect(),
        emitted_event: emitted_event.to_string(),
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
    fn core_action_state_semantics_contract_validates() {
        let contract = core_action_state_semantics_contract();
        validate_core_action_state_semantics_contract(&contract).unwrap();
        assert_eq!(contract.actions.len(), 12);
        assert_eq!(contract.states.len(), 10);
        assert_eq!(contract.transitions.len(), 12);
    }

    #[test]
    fn core_action_state_semantics_rejects_missing_action_reference() {
        let mut contract = core_action_state_semantics_contract();
        contract.transitions[0].action_type = "missingAction".to_string();

        let errors = validate_core_action_state_semantics_contract(&contract).unwrap_err();
        assert!(errors.iter().any(|error| error.contains("missing action")));
    }

    #[test]
    fn core_action_state_semantics_rejects_missing_state_reference() {
        let mut contract = core_action_state_semantics_contract();
        contract.actions[0].resulting_state = Some("missingState".to_string());

        let errors = validate_core_action_state_semantics_contract(&contract).unwrap_err();
        assert!(errors
            .iter()
            .any(|error| error.contains("missing resulting state")));
    }

    #[test]
    fn core_action_state_semantics_rejects_industry_pollution() {
        let mut contract = core_action_state_semantics_contract();
        contract.actions[0]
            .description
            .push_str(" This should not become a pull request.");

        let errors = validate_core_action_state_semantics_contract(&contract).unwrap_err();
        assert!(errors.iter().any(|error| error.contains("pull-request")));
    }
}
