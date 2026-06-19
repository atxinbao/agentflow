use std::collections::BTreeMap;

use agentflow_action_contract::ActionContractRegistry;
use agentflow_ontology::OntologyRegistry;

use crate::model::{
    ObjectStateDefinition, ObjectStateMachine, ObjectStateMachineBundle, StateTransitionDefinition,
};
use crate::report::{ObjectStateValidationReport, TransitionDecision};
use crate::validation::validate_object_state_bundle;

#[derive(Debug, Clone)]
pub struct ObjectStateMachineRegistry {
    bundle: ObjectStateMachineBundle,
    state_machines: BTreeMap<String, ObjectStateMachine>,
}

impl ObjectStateMachineRegistry {
    pub fn load_bundle(
        bundle: ObjectStateMachineBundle,
        ontology_registry: &OntologyRegistry,
        action_registry: &ActionContractRegistry,
    ) -> Result<Self, ObjectStateValidationReport> {
        let report = validate_object_state_bundle(&bundle, ontology_registry, action_registry);
        if !report.valid {
            return Err(report);
        }
        let state_machines = bundle
            .state_machines
            .iter()
            .cloned()
            .map(|item| (item.object_type.clone(), item))
            .collect();

        Ok(Self {
            bundle,
            state_machines,
        })
    }

    pub fn bundle(&self) -> &ObjectStateMachineBundle {
        &self.bundle
    }

    pub fn list_state_machines(&self) -> Vec<&ObjectStateMachine> {
        self.state_machines.values().collect()
    }

    pub fn get_state_machine(&self, object_type: &str) -> Option<&ObjectStateMachine> {
        self.state_machines.get(object_type)
    }

    pub fn get_state(
        &self,
        object_type: &str,
        state_or_alias: &str,
    ) -> Option<&ObjectStateDefinition> {
        let machine = self.get_state_machine(object_type)?;
        machine.states.iter().find(|state| {
            state.state_id == state_or_alias
                || state
                    .legacy_status_aliases
                    .iter()
                    .any(|alias| alias == state_or_alias)
        })
    }

    pub fn resolve_state_id(&self, object_type: &str, state_or_alias: &str) -> Option<String> {
        self.get_state(object_type, state_or_alias)
            .map(|state| state.state_id.clone())
    }

    pub fn is_transition_defined(
        &self,
        object_type: &str,
        current_state: Option<&str>,
        action_type: &str,
    ) -> TransitionDecision {
        let Some(machine) = self.get_state_machine(object_type) else {
            return TransitionDecision::denied(
                object_type,
                current_state.map(str::to_string),
                action_type,
                format!("object type `{object_type}` has no registered state machine"),
            );
        };

        let resolved_state = match current_state {
            Some(state_or_alias) => match self.resolve_state_id(object_type, state_or_alias) {
                Some(state_id) => Some(state_id),
                None => {
                    return TransitionDecision::denied(
                        object_type,
                        Some(state_or_alias.to_string()),
                        action_type,
                        format!(
                            "state `{state_or_alias}` is not defined for object type `{object_type}`"
                        ),
                    );
                }
            },
            None => None,
        };

        let matches = machine
            .transitions
            .iter()
            .filter(|transition| match resolved_state.as_deref() {
                Some(state_id) => transition.from_states.iter().any(|from| from == state_id),
                None => transition.from_states.is_empty(),
            })
            .filter(|transition| transition_matches_action(transition, action_type))
            .collect::<Vec<_>>();

        match matches.as_slice() {
            [] => TransitionDecision::denied(
                object_type,
                current_state.map(str::to_string),
                action_type,
                match resolved_state.as_deref() {
                    Some(state_id) => {
                        format!("no transition from `{state_id}` on action `{action_type}`")
                    }
                    None => format!("no initial transition on action `{action_type}`"),
                },
            ),
            [transition] => TransitionDecision {
                allowed: true,
                object_type: object_type.to_string(),
                requested_state: current_state.map(str::to_string),
                resolved_state,
                requested_action_type: action_type.to_string(),
                matched_action_type: Some(transition.action_type.clone()),
                next_state: Some(transition.to_state.clone()),
                reason: "transition-defined".into(),
                matched_via_compatibility_alias: transition.action_type != action_type,
                required_evidence: transition.required_evidence.clone(),
                emitted_events: transition.emitted_events.clone(),
                warnings: Vec::new(),
            },
            _ => TransitionDecision::denied(
                object_type,
                current_state.map(str::to_string),
                action_type,
                format!(
                    "multiple transitions match object `{object_type}` state `{:?}` action `{action_type}`",
                    resolved_state
                ),
            ),
        }
    }
}

fn transition_matches_action(transition: &StateTransitionDefinition, action_type: &str) -> bool {
    transition.action_type == action_type
        || transition
            .compatibility_action_aliases
            .iter()
            .any(|alias| alias == action_type)
}

#[cfg(test)]
mod tests {
    use agentflow_action_contract::core_action_contract_registry;
    use agentflow_ontology::core_ontology_registry;

    use crate::core::core_object_state_registry;

    #[test]
    fn legacy_issue_status_maps_to_review_ready() {
        let ontology = core_ontology_registry();
        let actions = core_action_contract_registry(&ontology);
        let registry = core_object_state_registry(&ontology, &actions).unwrap();

        let state = registry.get_state("Issue", "in_review").unwrap();
        assert_eq!(state.state_id, "reviewReady");
    }

    #[test]
    fn issue_done_does_not_allow_request_audit() {
        let ontology = core_ontology_registry();
        let actions = core_action_contract_registry(&ontology);
        let registry = core_object_state_registry(&ontology, &actions).unwrap();

        let decision = registry.is_transition_defined("Issue", Some("done"), "requestAudit");
        assert!(!decision.allowed);
    }

    #[test]
    fn run_completed_does_not_imply_issue_done() {
        let ontology = core_ontology_registry();
        let actions = core_action_contract_registry(&ontology);
        let registry = core_object_state_registry(&ontology, &actions).unwrap();

        let decision = registry.is_transition_defined("Run", Some("completed"), "markIssueDone");
        assert!(!decision.allowed);
    }

    #[test]
    fn finding_fix_required_can_link_fix_issue() {
        let ontology = core_ontology_registry();
        let actions = core_action_contract_registry(&ontology);
        let registry = core_object_state_registry(&ontology, &actions).unwrap();

        let decision =
            registry.is_transition_defined("Finding", Some("fixRequired"), "linkFixIssue");
        assert!(decision.allowed);
        assert_eq!(decision.next_state.as_deref(), Some("fixLinked"));
    }
}
