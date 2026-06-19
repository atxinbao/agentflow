use std::collections::{BTreeMap, BTreeSet};

use agentflow_action_contract::ActionContractRegistry;
use agentflow_ontology::OntologyRegistry;

use crate::model::{
    ObjectStateMachine, ObjectStateMachineBundle, ObjectStateMachineStatus, StateProjectionHints,
    StateTransitionDefinition,
};
use crate::report::ObjectStateValidationReport;

pub fn validate_object_state_bundle(
    bundle: &ObjectStateMachineBundle,
    ontology_registry: &OntologyRegistry,
    action_registry: &ActionContractRegistry,
) -> ObjectStateValidationReport {
    let mut report = ObjectStateValidationReport::success();

    if bundle.registry_id.trim().is_empty() {
        report.push_error(
            "registry-id-missing",
            "registryId must not be empty",
            Some("registryId".into()),
        );
    }
    if bundle.namespace.trim().is_empty() {
        report.push_error(
            "namespace-missing",
            "namespace must not be empty",
            Some("namespace".into()),
        );
    }
    if bundle.definition_version.trim().is_empty() {
        report.push_error(
            "definition-version-missing",
            "definitionVersion must not be empty",
            Some("definitionVersion".into()),
        );
    }

    let known_object_types = ontology_registry
        .list_object_types()
        .into_iter()
        .map(|item| item.id.as_str())
        .collect::<BTreeSet<_>>();
    let known_link_types = ontology_registry
        .list_link_types()
        .into_iter()
        .map(|item| item.id.as_str())
        .collect::<BTreeSet<_>>();

    let mut machine_ids = BTreeSet::new();
    let mut object_types = BTreeSet::new();
    for (index, machine) in bundle.state_machines.iter().enumerate() {
        let path = format!("stateMachines[{index}]");
        validate_machine_header(
            machine,
            &path,
            &known_object_types,
            &mut machine_ids,
            &mut object_types,
            &mut report,
        );
        validate_states(machine, &path, &mut report);
        validate_projection_hints(machine, &path, &mut report);
        validate_transitions(
            machine,
            &path,
            &known_link_types,
            action_registry,
            &mut report,
        );
        validate_forbidden_implicit_transitions(machine, &path, &mut report);
        validate_draft_action_coverage(machine, &path, action_registry, &mut report);
    }

    report
}

fn validate_machine_header(
    machine: &ObjectStateMachine,
    path: &str,
    known_object_types: &BTreeSet<&str>,
    machine_ids: &mut BTreeSet<String>,
    object_types: &mut BTreeSet<String>,
    report: &mut ObjectStateValidationReport,
) {
    if machine.state_machine_id.trim().is_empty() {
        report.push_error(
            "state-machine-id-missing",
            "stateMachineId must not be empty",
            Some(format!("{path}.stateMachineId")),
        );
    } else if !machine_ids.insert(machine.state_machine_id.clone()) {
        report.push_error(
            "duplicate-state-machine-id",
            format!(
                "state machine `{}` is defined more than once",
                machine.state_machine_id
            ),
            Some(format!("{path}.stateMachineId")),
        );
    }

    if machine.object_type.trim().is_empty() {
        report.push_error(
            "object-type-missing",
            "objectType must not be empty",
            Some(format!("{path}.objectType")),
        );
    } else {
        if !known_object_types.contains(machine.object_type.as_str()) {
            report.push_error(
                "unknown-object-type",
                format!(
                    "state machine `{}` references unknown object type `{}`",
                    machine.state_machine_id, machine.object_type
                ),
                Some(format!("{path}.objectType")),
            );
        }
        if !object_types.insert(machine.object_type.clone()) {
            report.push_error(
                "duplicate-object-type-machine",
                format!(
                    "object type `{}` is bound by more than one state machine",
                    machine.object_type
                ),
                Some(format!("{path}.objectType")),
            );
        }
    }
}

fn validate_states(
    machine: &ObjectStateMachine,
    path: &str,
    report: &mut ObjectStateValidationReport,
) {
    let mut state_ids = BTreeSet::new();
    let mut alias_to_state = BTreeMap::<String, String>::new();
    for (index, state) in machine.states.iter().enumerate() {
        let state_path = format!("{path}.states[{index}]");
        if state.state_id.trim().is_empty() {
            report.push_error(
                "state-id-missing",
                "stateId must not be empty",
                Some(format!("{state_path}.stateId")),
            );
        } else if !state_ids.insert(state.state_id.clone()) {
            report.push_error(
                "duplicate-state-id",
                format!(
                    "state machine `{}` duplicates state `{}`",
                    machine.state_machine_id, state.state_id
                ),
                Some(format!("{state_path}.stateId")),
            );
        }
        if state.name.trim().is_empty() {
            report.push_error(
                "state-name-missing",
                format!(
                    "state machine `{}` state `{}` must have non-empty name",
                    machine.state_machine_id, state.state_id
                ),
                Some(format!("{state_path}.name")),
            );
        }
        for alias in &state.legacy_status_aliases {
            if alias.trim().is_empty() {
                report.push_error(
                    "empty-legacy-status-alias",
                    format!(
                        "state machine `{}` state `{}` contains empty legacy alias",
                        machine.state_machine_id, state.state_id
                    ),
                    Some(format!("{state_path}.legacyStatusAliases")),
                );
                continue;
            }
            if let Some(existing) = alias_to_state.insert(alias.clone(), state.state_id.clone()) {
                if existing != state.state_id {
                    report.push_error(
                        "duplicate-legacy-status-alias",
                        format!(
                            "legacy status alias `{alias}` maps to both `{existing}` and `{}`",
                            state.state_id
                        ),
                        Some(format!("{state_path}.legacyStatusAliases")),
                    );
                }
            }
        }
    }

    if !state_ids.contains(machine.initial_state.as_str()) {
        report.push_error(
            "initial-state-missing",
            format!(
                "state machine `{}` initial state `{}` is not defined",
                machine.state_machine_id, machine.initial_state
            ),
            Some(format!("{path}.initialState")),
        );
    }

    for terminal_state in &machine.terminal_states {
        if !state_ids.contains(terminal_state.as_str()) {
            report.push_error(
                "unknown-terminal-state",
                format!(
                    "state machine `{}` terminal state `{terminal_state}` is not defined",
                    machine.state_machine_id
                ),
                Some(format!("{path}.terminalStates")),
            );
            continue;
        }
        if let Some(state) = machine
            .states
            .iter()
            .find(|state| state.state_id == *terminal_state)
        {
            if !state.is_terminal {
                report.push_error(
                    "terminal-flag-mismatch",
                    format!(
                        "state machine `{}` state `{terminal_state}` is listed in terminalStates but isTerminal=false",
                        machine.state_machine_id
                    ),
                    Some(format!("{path}.terminalStates")),
                );
            }
        }
    }
}

fn validate_projection_hints(
    machine: &ObjectStateMachine,
    path: &str,
    report: &mut ObjectStateValidationReport,
) {
    let known_states = machine
        .states
        .iter()
        .map(|state| state.state_id.as_str())
        .collect::<BTreeSet<_>>();
    validate_hint_states(
        machine,
        &machine.projection_hints,
        path,
        "timelineOrder",
        &machine.projection_hints.timeline_order,
        &known_states,
        report,
    );
    validate_hint_states(
        machine,
        &machine.projection_hints,
        path,
        "preferredCurrentStates",
        &machine.projection_hints.preferred_current_states,
        &known_states,
        report,
    );
}

fn validate_hint_states(
    machine: &ObjectStateMachine,
    _hints: &StateProjectionHints,
    path: &str,
    field: &str,
    states: &[String],
    known_states: &BTreeSet<&str>,
    report: &mut ObjectStateValidationReport,
) {
    for (index, state_id) in states.iter().enumerate() {
        if !known_states.contains(state_id.as_str()) {
            report.push_error(
                "unknown-projection-hint-state",
                format!(
                    "state machine `{}` projection hint references unknown state `{state_id}`",
                    machine.state_machine_id
                ),
                Some(format!("{path}.projectionHints.{field}[{index}]")),
            );
        }
    }
}

fn validate_transitions(
    machine: &ObjectStateMachine,
    path: &str,
    known_link_types: &BTreeSet<&str>,
    action_registry: &ActionContractRegistry,
    report: &mut ObjectStateValidationReport,
) {
    let known_states = machine
        .states
        .iter()
        .map(|state| state.state_id.as_str())
        .collect::<BTreeSet<_>>();
    let mut transition_ids = BTreeSet::new();

    for (index, transition) in machine.transitions.iter().enumerate() {
        let transition_path = format!("{path}.transitions[{index}]");
        if transition.transition_id.trim().is_empty() {
            report.push_error(
                "transition-id-missing",
                "transitionId must not be empty",
                Some(format!("{transition_path}.transitionId")),
            );
        } else if !transition_ids.insert(transition.transition_id.clone()) {
            report.push_error(
                "duplicate-transition-id",
                format!(
                    "state machine `{}` duplicates transition `{}`",
                    machine.state_machine_id, transition.transition_id
                ),
                Some(format!("{transition_path}.transitionId")),
            );
        }

        if !known_states.contains(transition.to_state.as_str()) {
            report.push_error(
                "unknown-transition-target-state",
                format!(
                    "state machine `{}` transition `{}` points to unknown state `{}`",
                    machine.state_machine_id, transition.transition_id, transition.to_state
                ),
                Some(format!("{transition_path}.toState")),
            );
        }
        for (from_index, from_state) in transition.from_states.iter().enumerate() {
            if !known_states.contains(from_state.as_str()) {
                report.push_error(
                    "unknown-transition-source-state",
                    format!(
                        "state machine `{}` transition `{}` references unknown source state `{from_state}`",
                        machine.state_machine_id, transition.transition_id
                    ),
                    Some(format!("{transition_path}.from[{from_index}]")),
                );
            }
            if machine
                .terminal_states
                .iter()
                .any(|terminal| terminal == from_state)
            {
                let explicit_reopen = transition.to_state == "reopened"
                    || transition.action_type.starts_with("reopen");
                if !explicit_reopen {
                    report.push_error(
                        "terminal-state-outgoing-transition",
                        format!(
                            "state machine `{}` transition `{}` must not originate from terminal state `{from_state}`",
                            machine.state_machine_id, transition.transition_id
                        ),
                        Some(format!("{transition_path}.from[{from_index}]")),
                    );
                }
            }
        }
        for (link_index, link_type) in transition.link_effects.iter().enumerate() {
            if !known_link_types.contains(link_type.as_str()) {
                report.push_error(
                    "unknown-link-type",
                    format!(
                        "state machine `{}` transition `{}` references unknown link type `{link_type}`",
                        machine.state_machine_id, transition.transition_id
                    ),
                    Some(format!("{transition_path}.linkEffects[{link_index}]")),
                );
            }
        }
        for (event_index, event_type) in transition.emitted_events.iter().enumerate() {
            if event_type.trim().is_empty() {
                report.push_error(
                    "empty-emitted-event",
                    format!(
                        "state machine `{}` transition `{}` contains empty emitted event",
                        machine.state_machine_id, transition.transition_id
                    ),
                    Some(format!("{transition_path}.emittedEvents[{event_index}]")),
                );
            }
        }

        validate_transition_action_coverage(
            machine,
            transition,
            &transition_path,
            action_registry,
            report,
        );
    }
}

fn validate_transition_action_coverage(
    machine: &ObjectStateMachine,
    transition: &StateTransitionDefinition,
    path: &str,
    action_registry: &ActionContractRegistry,
    report: &mut ObjectStateValidationReport,
) {
    let Some(contract_action_type) =
        resolve_known_contract_action_type(transition, action_registry)
    else {
        report.push_warning(
            "unknown-action-type",
            format!(
                "state machine `{}` transition `{}` action `{}` is not yet registered in action-contract",
                machine.state_machine_id, transition.transition_id, transition.action_type
            ),
            Some(format!("{path}.actionType")),
        );
        return;
    };

    let Some(contract) = action_registry.get_action_contract(
        contract_action_type,
        action_registry.bundle().definition_version.as_str(),
    ) else {
        report.push_warning(
            "missing-action-contract",
            format!(
                "state machine `{}` transition `{}` action `{contract_action_type}` has no active contract",
                machine.state_machine_id, transition.transition_id
            ),
            Some(format!("{path}.actionType")),
        );
        return;
    };

    let known_evidence = contract
        .required_evidence
        .iter()
        .map(|evidence| evidence.evidence_type.as_str())
        .collect::<BTreeSet<_>>();

    for (index, evidence_type) in transition.required_evidence.iter().enumerate() {
        if !known_evidence.contains(evidence_type.as_str()) {
            report.push_error(
                "unknown-required-evidence",
                format!(
                    "state machine `{}` transition `{}` requires evidence `{evidence_type}` not declared by action `{contract_action_type}`",
                    machine.state_machine_id, transition.transition_id
                ),
                Some(format!("{path}.requiredEvidence[{index}]")),
            );
        }
    }
}

fn validate_forbidden_implicit_transitions(
    machine: &ObjectStateMachine,
    path: &str,
    report: &mut ObjectStateValidationReport,
) {
    for (index, transition) in machine.transitions.iter().enumerate() {
        let transition_path = format!("{path}.transitions[{index}]");
        if machine.object_type == "Issue" && transition.action_type == "requestAudit" {
            report.push_error(
                "forbidden-implicit-audit-transition",
                "Issue state machine must not create Audit through implicit requestAudit transition",
                Some(format!("{transition_path}.actionType")),
            );
        }
        if machine.object_type == "Issue"
            && transition
                .from_states
                .iter()
                .any(|state| state == "running")
            && transition.to_state == "done"
            && transition.action_type == "completeRun"
        {
            report.push_error(
                "forbidden-run-completion-direct-done",
                "Issue state machine must not use completeRun to jump directly from running to done",
                Some(format!("{transition_path}.actionType")),
            );
        }
        if machine.object_type == "Finding"
            && transition
                .from_states
                .iter()
                .any(|state| state == "fixRequired")
            && transition.to_state == "done"
        {
            report.push_error(
                "forbidden-finding-mutate-issue-done",
                "Finding transitions must not mutate Issue done state directly",
                Some(format!("{transition_path}.toState")),
            );
        }
    }
}

fn validate_draft_action_coverage(
    machine: &ObjectStateMachine,
    path: &str,
    action_registry: &ActionContractRegistry,
    report: &mut ObjectStateValidationReport,
) {
    if machine.status != ObjectStateMachineStatus::Draft {
        return;
    }
    if machine
        .transitions
        .iter()
        .all(|transition| resolve_known_contract_action_type(transition, action_registry).is_some())
    {
        return;
    }
    report.push_warning(
        "draft-action-gap",
        format!(
            "state machine `{}` is still draft and depends on future action-contract expansion for some transitions",
            machine.state_machine_id
        ),
        Some(path.to_string()),
    );
}

fn resolve_known_contract_action_type<'a>(
    transition: &'a StateTransitionDefinition,
    action_registry: &ActionContractRegistry,
) -> Option<&'a str> {
    if action_registry
        .get_action_type(transition.action_type.as_str())
        .is_some()
    {
        return Some(transition.action_type.as_str());
    }
    transition
        .compatibility_action_aliases
        .iter()
        .find_map(|alias| {
            action_registry
                .get_action_type(alias.as_str())
                .map(|_| alias.as_str())
        })
}

#[cfg(test)]
mod tests {
    use agentflow_action_contract::{core_action_contract_bundle, core_action_contract_registry};
    use agentflow_ontology::core_ontology_registry;

    use crate::core::core_object_state_bundle;
    use crate::model::ObjectStateMachineStatus;
    use crate::validate_object_state_bundle;

    #[test]
    fn core_object_state_bundle_validates() {
        let ontology = core_ontology_registry();
        let actions = core_action_contract_registry(&ontology);

        let report = validate_object_state_bundle(&core_object_state_bundle(), &ontology, &actions);
        assert!(report.valid, "{report:?}");
    }

    #[test]
    fn unknown_object_type_fails() {
        let ontology = core_ontology_registry();
        let actions = core_action_contract_registry(&ontology);
        let mut bundle = core_object_state_bundle();
        bundle.state_machines[0].object_type = "Unknown".into();

        let report = validate_object_state_bundle(&bundle, &ontology, &actions);
        assert!(!report.valid);
        assert!(report
            .errors
            .iter()
            .any(|error| error.code == "unknown-object-type"));
    }

    #[test]
    fn transition_with_missing_state_fails() {
        let ontology = core_ontology_registry();
        let actions = core_action_contract_registry(&ontology);
        let mut bundle = core_object_state_bundle();
        bundle.state_machines[2].transitions[0].to_state = "missing".into();

        let report = validate_object_state_bundle(&bundle, &ontology, &actions);
        assert!(!report.valid);
        assert!(report
            .errors
            .iter()
            .any(|error| error.code == "unknown-transition-target-state"));
    }

    #[test]
    fn link_effect_unknown_link_type_fails() {
        let ontology = core_ontology_registry();
        let actions = core_action_contract_registry(&ontology);
        let mut bundle = core_object_state_bundle();
        bundle.state_machines[5].transitions[0].link_effects = vec!["unknownLink".into()];

        let report = validate_object_state_bundle(&bundle, &ontology, &actions);
        assert!(!report.valid);
        assert!(report
            .errors
            .iter()
            .any(|error| error.code == "unknown-link-type"));
    }

    #[test]
    fn mark_issue_done_must_keep_known_required_evidence() {
        let ontology = core_ontology_registry();
        let actions = core_action_contract_registry(&ontology);
        let mut bundle = core_object_state_bundle();
        let transition = bundle.state_machines[2]
            .transitions
            .iter_mut()
            .find(|transition| transition.action_type == "markIssueDone")
            .unwrap();
        transition.required_evidence.push("unknownEvidence".into());

        let report = validate_object_state_bundle(&bundle, &ontology, &actions);
        assert!(!report.valid);
        assert!(report
            .errors
            .iter()
            .any(|error| error.code == "unknown-required-evidence"));
    }

    #[test]
    fn unknown_transition_action_warns_for_draft_bundle() {
        let ontology = core_ontology_registry();
        let actions = core_action_contract_registry(&ontology);
        let mut bundle = core_object_state_bundle();
        bundle.state_machines[0].transitions[0].action_type = "futureAction".into();
        bundle.state_machines[0].transitions[0]
            .compatibility_action_aliases
            .clear();

        let report = validate_object_state_bundle(&bundle, &ontology, &actions);
        assert!(report.valid);
        assert!(report
            .warnings
            .iter()
            .any(|warning| warning.code == "unknown-action-type"));
    }

    #[test]
    fn bundle_is_draft_by_default() {
        let bundle = core_object_state_bundle();
        assert_eq!(bundle.status, ObjectStateMachineStatus::Draft);
        assert_eq!(
            bundle.version,
            super::super::model::OBJECT_STATE_BUNDLE_VERSION
        );
    }

    #[test]
    fn action_bundle_sanity_for_state_machine_dependency() {
        let bundle = core_action_contract_bundle();
        assert!(bundle
            .action_types
            .iter()
            .any(|action| action.id == "markIssueDone"));
    }
}
