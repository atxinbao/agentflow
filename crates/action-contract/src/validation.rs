use std::collections::{BTreeMap, BTreeSet};

use agentflow_ontology::OntologyRegistry;
use serde_json::Value;

use crate::model::{
    ActionContract, ActionContractBundle, ActionDefinitionStatus, ActionFieldDefinition,
    ActionFieldValueType, ActionProposal, ActionTargetMode,
};
use crate::registry::ActionContractRegistry;
use crate::report::{
    ActionContractValidationReport, ActionProposalValidationReport, ActionProposalValidationStatus,
};

const FORBIDDEN_MVP_OBJECTS: &[&str] =
    &["WorkPackage", "Delivery", "DeliveryPackage", "AuditFinding"];

pub fn validate_action_contract_bundle(
    bundle: &ActionContractBundle,
    ontology_registry: &OntologyRegistry,
) -> ActionContractValidationReport {
    let mut report = ActionContractValidationReport::success();

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
            "version-missing",
            "definitionVersion must not be empty",
            Some("definitionVersion".into()),
        );
    }

    let mut action_type_ids = BTreeSet::new();
    for (index, action_type) in bundle.action_types.iter().enumerate() {
        if !action_type_ids.insert(action_type.id.clone()) {
            report.push_error(
                "duplicate-action-type",
                format!("action type `{}` is defined more than once", action_type.id),
                Some(format!("actionTypes[{index}].id")),
            );
        }
        validate_object_type_ref(
            &mut report,
            ontology_registry,
            action_type.target_object_type.as_deref(),
            format!("actionTypes[{index}].targetObjectType"),
        );
        validate_created_object_type(
            &mut report,
            ontology_registry,
            action_type.creates_object_type.as_deref(),
            format!("actionTypes[{index}].createsObjectType"),
        );
    }

    let mut contract_ids = BTreeSet::new();
    let action_type_map: BTreeMap<_, _> = bundle
        .action_types
        .iter()
        .map(|action_type| (action_type.id.as_str(), action_type))
        .collect();
    for (index, contract) in bundle.contracts.iter().enumerate() {
        if !contract_ids.insert(contract.id.clone()) {
            report.push_error(
                "duplicate-contract-id",
                format!("contract `{}` is defined more than once", contract.id),
                Some(format!("contracts[{index}].id")),
            );
        }

        let Some(action_type) = action_type_map.get(contract.action_type.as_str()) else {
            report.push_error(
                "unknown-action-type",
                format!(
                    "contract `{}` references unknown action type `{}`",
                    contract.id, contract.action_type
                ),
                Some(format!("contracts[{index}].actionType")),
            );
            continue;
        };

        if action_type.target_mode != contract.target_mode {
            report.push_error(
                "target-mode-mismatch",
                format!(
                    "contract `{}` target mode does not match action type `{}`",
                    contract.id, action_type.id
                ),
                Some(format!("contracts[{index}].targetMode")),
            );
        }
        if action_type.target_object_type != contract.target_object_type {
            report.push_error(
                "target-object-mismatch",
                format!(
                    "contract `{}` target object does not match action type `{}`",
                    contract.id, action_type.id
                ),
                Some(format!("contracts[{index}].targetObjectType")),
            );
        }
        if action_type.creates_object_type != contract.creates_object_type {
            report.push_error(
                "creates-object-mismatch",
                format!(
                    "contract `{}` creates object does not match action type `{}`",
                    contract.id, action_type.id
                ),
                Some(format!("contracts[{index}].createsObjectType")),
            );
        }

        validate_object_type_ref(
            &mut report,
            ontology_registry,
            contract.target_object_type.as_deref(),
            format!("contracts[{index}].targetObjectType"),
        );
        validate_created_object_type(
            &mut report,
            ontology_registry,
            contract.creates_object_type.as_deref(),
            format!("contracts[{index}].createsObjectType"),
        );

        validate_input_schema(&mut report, ontology_registry, contract, index);

        for expected_link in &contract.expected_links {
            if ontology_registry.get_link_type(expected_link).is_none() {
                report.push_error(
                    "unknown-expected-link-type",
                    format!(
                        "contract `{}` references unknown expected link `{expected_link}`",
                        contract.id
                    ),
                    Some(format!("contracts[{index}].expectedLinks")),
                );
            }
        }

        if contract.expected_events.is_empty() {
            report.push_error(
                "missing-expected-event",
                format!(
                    "contract `{}` must declare at least one expected event",
                    contract.id
                ),
                Some(format!("contracts[{index}].expectedEvents")),
            );
        }
    }

    report
}

pub fn validate_action_proposal(
    proposal: &ActionProposal,
    contract_registry: &ActionContractRegistry,
    ontology_registry: &OntologyRegistry,
) -> ActionProposalValidationReport {
    let Some(action_type) = contract_registry.get_action_type(&proposal.action_type) else {
        let mut report = ActionProposalValidationReport::invalid(
            proposal,
            ActionProposalValidationStatus::Unsupported,
        );
        report.push_error(
            "unknownActionType",
            format!("unknown action type `{}`", proposal.action_type),
            Some("actionType".into()),
        );
        return report;
    };

    let Some(contract) =
        contract_registry.get_action_contract(&proposal.action_type, &proposal.contract_version)
    else {
        let mut report = ActionProposalValidationReport::invalid(
            proposal,
            ActionProposalValidationStatus::VersionMismatch,
        );
        report.push_error(
            "unknownContractVersion",
            format!(
                "unknown contract version `{}` for action `{}`",
                proposal.contract_version, proposal.action_type
            ),
            Some("contractVersion".into()),
        );
        return report;
    };

    let mut report = ActionProposalValidationReport::valid(proposal.clone());

    if proposal.ontology_version != ontology_registry.bundle().definition_version {
        report.status = ActionProposalValidationStatus::VersionMismatch;
        report.push_error(
            "ontologyVersionMismatch",
            format!(
                "proposal ontology version `{}` does not match registry `{}`",
                proposal.ontology_version,
                ontology_registry.bundle().definition_version
            ),
            Some("ontologyVersion".into()),
        );
    }

    if matches!(contract.status, ActionDefinitionStatus::Retired) {
        report.status = ActionProposalValidationStatus::Unsupported;
        report.push_error(
            "contractRetired",
            format!("contract `{}` is retired", contract.id),
            Some("contractVersion".into()),
        );
    } else if matches!(contract.status, ActionDefinitionStatus::Deprecated) {
        report
            .warnings
            .push(format!("contract `{}` is deprecated", contract.id));
    }

    validate_proposal_target(
        &mut report,
        proposal,
        contract,
        action_type,
        ontology_registry,
    );
    validate_proposal_input(&mut report, proposal, contract);
    validate_idempotency_key(&mut report, proposal);
    validate_evidence_refs(&mut report, proposal, contract);

    report
}

fn validate_proposal_target(
    report: &mut ActionProposalValidationReport,
    proposal: &ActionProposal,
    contract: &ActionContract,
    action_type: &crate::model::ActionTypeDefinition,
    ontology_registry: &OntologyRegistry,
) {
    match contract.target_mode {
        ActionTargetMode::ExistingObject
        | ActionTargetMode::LinkObjects
        | ActionTargetMode::RecordDecision => {
            let Some(target) = proposal.target_object_ref.as_ref() else {
                report.push_error(
                    "invalidTargetMode",
                    "targetObjectRef is required for this action target mode",
                    Some("targetObjectRef".into()),
                );
                return;
            };
            let expected_type = contract
                .target_object_type
                .as_deref()
                .or(action_type.target_object_type.as_deref());
            if let Some(expected_type) = expected_type {
                if target.object_type != expected_type {
                    report.push_error(
                        "unknownTargetObjectType",
                        format!(
                            "proposal target object type `{}` does not match contract target `{expected_type}`",
                            target.object_type
                        ),
                        Some("targetObjectRef.objectType".into()),
                    );
                }
                if ontology_registry.get_object_type(expected_type).is_none() {
                    report.push_error(
                        "unknownTargetObjectType",
                        format!("target object type `{expected_type}` is not defined in ontology"),
                        Some("targetObjectRef.objectType".into()),
                    );
                }
            }
        }
        ActionTargetMode::CreateObject => {}
    }

    if let Some(created_object_type) = contract.creates_object_type.as_deref() {
        if ontology_registry
            .get_object_type(created_object_type)
            .is_none()
        {
            report.push_error(
                "unknownCreatedObjectType",
                format!("created object type `{created_object_type}` is not defined in ontology"),
                Some("createsObjectType".into()),
            );
        }
    }
}

fn validate_proposal_input(
    report: &mut ActionProposalValidationReport,
    proposal: &ActionProposal,
    contract: &ActionContract,
) {
    let Some(input) = proposal.input.as_object() else {
        report.push_error(
            "invalidInputField",
            "proposal input must be a JSON object",
            Some("input".into()),
        );
        return;
    };

    let field_map: BTreeMap<_, _> = contract
        .input_schema
        .fields
        .iter()
        .map(|field| (field.name.as_str(), field))
        .collect();
    for required in &contract.input_schema.required_fields {
        if !input.contains_key(required) {
            report.push_error(
                "missingRequiredInput",
                format!("missing required input field `{required}`"),
                Some("input".into()),
            );
        }
    }

    if !contract.input_schema.allow_additional_fields {
        for key in input.keys() {
            if !field_map.contains_key(key.as_str()) {
                report.push_error(
                    "invalidInputField",
                    format!("unexpected input field `{key}`"),
                    Some(format!("input.{key}")),
                );
            }
        }
    }

    for field in &contract.input_schema.fields {
        if let Some(value) = input.get(&field.name) {
            validate_field_value(report, field, value);
        }
    }
}

fn validate_field_value(
    report: &mut ActionProposalValidationReport,
    field: &ActionFieldDefinition,
    value: &Value,
) {
    let valid = match field.value_type {
        ActionFieldValueType::String
        | ActionFieldValueType::Enum
        | ActionFieldValueType::Timestamp => value.is_string(),
        ActionFieldValueType::Number => value.is_number(),
        ActionFieldValueType::Boolean => value.is_boolean(),
        ActionFieldValueType::StructuredObject => value.is_object(),
        ActionFieldValueType::ObjectRef
        | ActionFieldValueType::EvidenceRef
        | ActionFieldValueType::ArtifactRef => value.is_string(),
        ActionFieldValueType::ObjectRefList
        | ActionFieldValueType::EvidenceRefList
        | ActionFieldValueType::ArtifactRefList => value.is_array(),
    };
    if !valid {
        report.push_error(
            "invalidInputField",
            format!("input field `{}` has invalid value type", field.name),
            Some(format!("input.{}", field.name)),
        );
        return;
    }
    if matches!(field.value_type, ActionFieldValueType::Enum) {
        if let Some(enum_value) = value.as_str() {
            if !field.enum_values.is_empty()
                && !field.enum_values.iter().any(|item| item == enum_value)
            {
                report.push_error(
                    "invalidInputField",
                    format!(
                        "input field `{}` has unsupported enum value `{enum_value}`",
                        field.name
                    ),
                    Some(format!("input.{}", field.name)),
                );
            }
        }
    }
}

fn validate_idempotency_key(
    report: &mut ActionProposalValidationReport,
    proposal: &ActionProposal,
) {
    let parts: Vec<_> = proposal.idempotency_key.split(':').collect();
    if parts.len() < 5 || parts.iter().any(|part| part.trim().is_empty()) {
        report.push_error(
            "invalidIdempotencyKey",
            "idempotencyKey must contain at least 5 non-empty ':' separated parts",
            Some("idempotencyKey".into()),
        );
    }
}

fn validate_evidence_refs(
    report: &mut ActionProposalValidationReport,
    proposal: &ActionProposal,
    contract: &ActionContract,
) {
    if proposal
        .evidence_refs
        .iter()
        .any(|item| item.trim().is_empty())
    {
        report.push_error(
            "unknownEvidenceRefKind",
            "evidenceRefs must not contain empty values",
            Some("evidenceRefs".into()),
        );
    }

    let required_count: usize = contract
        .required_evidence
        .iter()
        .filter(|item| item.required)
        .map(|item| item.min_count.max(1))
        .sum();
    if proposal.evidence_refs.len() < required_count {
        report.push_error(
            "missingRequiredEvidence",
            format!(
                "proposal requires at least {required_count} evidence refs but only {} provided",
                proposal.evidence_refs.len()
            ),
            Some("evidenceRefs".into()),
        );
    }
}

fn validate_object_type_ref(
    report: &mut ActionContractValidationReport,
    ontology_registry: &OntologyRegistry,
    object_type: Option<&str>,
    path: String,
) {
    if let Some(object_type) = object_type {
        if FORBIDDEN_MVP_OBJECTS.contains(&object_type) {
            report.push_error(
                "forbidden-mvp-object-type",
                format!("object type `{object_type}` is not allowed in AF-OS-002 MVP"),
                Some(path.clone()),
            );
            return;
        }
        if ontology_registry.get_object_type(object_type).is_none() {
            report.push_error(
                "unknown-target-object-type",
                format!("object type `{object_type}` is not defined in ontology"),
                Some(path),
            );
        }
    }
}

fn validate_created_object_type(
    report: &mut ActionContractValidationReport,
    ontology_registry: &OntologyRegistry,
    object_type: Option<&str>,
    path: String,
) {
    validate_object_type_ref(report, ontology_registry, object_type, path);
}

fn validate_input_schema(
    report: &mut ActionContractValidationReport,
    ontology_registry: &OntologyRegistry,
    contract: &ActionContract,
    index: usize,
) {
    let field_names: BTreeSet<_> = contract
        .input_schema
        .fields
        .iter()
        .map(|field| field.name.as_str())
        .collect();
    for required in &contract.input_schema.required_fields {
        if !field_names.contains(required.as_str()) {
            report.push_error(
                "missing-required-input-definition",
                format!(
                    "contract `{}` requires input field `{required}` but field definition is missing",
                    contract.id
                ),
                Some(format!("contracts[{index}].inputSchema.requiredFields")),
            );
        }
    }
    for (field_index, field) in contract.input_schema.fields.iter().enumerate() {
        if let Some(object_type_ref) = field.object_type_ref.as_deref() {
            if ontology_registry.get_object_type(object_type_ref).is_none() {
                report.push_error(
                    "unknown-target-object-type",
                    format!(
                        "field `{}` references unknown object type `{object_type_ref}`",
                        field.name
                    ),
                    Some(format!(
                        "contracts[{index}].inputSchema.fields[{field_index}].objectTypeRef"
                    )),
                );
            }
        }
        if let Some(link_type_ref) = field.link_type_ref.as_deref() {
            if ontology_registry.get_link_type(link_type_ref).is_none() {
                report.push_error(
                    "unknown-expected-link-type",
                    format!(
                        "field `{}` references unknown link type `{link_type_ref}`",
                        field.name
                    ),
                    Some(format!(
                        "contracts[{index}].inputSchema.fields[{field_index}].linkTypeRef"
                    )),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use agentflow_ontology::core_ontology_registry;
    use serde_json::json;

    use crate::core::core_action_contract_bundle;
    use crate::model::{ActionDefinitionStatus, ActionProposal, ActionSourceSurface};
    use crate::registry::ActionContractRegistry;

    use super::{validate_action_contract_bundle, validate_action_proposal};

    fn registry() -> ActionContractRegistry {
        let ontology = core_ontology_registry();
        ActionContractRegistry::load_bundle(core_action_contract_bundle(), &ontology).unwrap()
    }

    #[test]
    fn core_action_contract_bundle_validates() {
        let ontology = core_ontology_registry();
        let report = validate_action_contract_bundle(&core_action_contract_bundle(), &ontology);
        assert!(report.valid, "{:?}", report.errors);
    }

    #[test]
    fn unknown_action_type_fails() {
        let ontology = core_ontology_registry();
        let registry = registry();
        let proposal = ActionProposal {
            proposal_id: "proposal-1".into(),
            idempotency_key: "agent:role:unknown:target:hash".into(),
            action_type: "doesNotExist".into(),
            actor_role: "work-agent".into(),
            source_surface: ActionSourceSurface::Agent,
            target_object_ref: None,
            input: json!({}),
            evidence_refs: vec![],
            artifact_refs: vec![],
            reason: None,
            expected_effects: vec![],
            ontology_version: ontology.bundle().definition_version.clone(),
            contract_version: registry.bundle().definition_version.clone(),
            created_at: "2026-06-20T00:00:00Z".into(),
        };

        let report = validate_action_proposal(&proposal, &registry, &ontology);
        assert_eq!(
            report.status,
            crate::report::ActionProposalValidationStatus::Unsupported
        );
        assert!(report
            .errors
            .iter()
            .any(|error| error.code == "unknownActionType"));
    }

    #[test]
    fn retired_contract_rejects_proposal() {
        let ontology = core_ontology_registry();
        let mut bundle = core_action_contract_bundle();
        bundle
            .contracts
            .iter_mut()
            .find(|contract| contract.action_type == "approveSpec")
            .unwrap()
            .status = ActionDefinitionStatus::Retired;
        let registry = ActionContractRegistry::load_bundle(bundle, &ontology).unwrap();
        let proposal = ActionProposal {
            proposal_id: "proposal-1".into(),
            idempotency_key: "desktop:human:approveSpec:spec:hash".into(),
            action_type: "approveSpec".into(),
            actor_role: "human-owner".into(),
            source_surface: ActionSourceSurface::Desktop,
            target_object_ref: Some(crate::model::ActionRef {
                object_type: "Spec".into(),
                id: "spec-001".into(),
            }),
            input: json!({ "decisionSummary": "同意" }),
            evidence_refs: vec!["decision-001".into()],
            artifact_refs: vec![],
            reason: None,
            expected_effects: vec![],
            ontology_version: ontology.bundle().definition_version.clone(),
            contract_version: registry.bundle().definition_version.clone(),
            created_at: "2026-06-20T00:00:00Z".into(),
        };

        let report = validate_action_proposal(&proposal, &registry, &ontology);
        assert_eq!(
            report.status,
            crate::report::ActionProposalValidationStatus::Unsupported
        );
        assert!(report
            .errors
            .iter()
            .any(|error| error.code == "contractRetired"));
    }

    #[test]
    fn proposal_with_missing_required_input_fails() {
        let ontology = core_ontology_registry();
        let registry = registry();
        let proposal = ActionProposal {
            proposal_id: "proposal-1".into(),
            idempotency_key: "agent:work-agent:startRun:issue:hash".into(),
            action_type: "startRun".into(),
            actor_role: "work-agent".into(),
            source_surface: ActionSourceSurface::Agent,
            target_object_ref: Some(crate::model::ActionRef {
                object_type: "Issue".into(),
                id: "AF-001".into(),
            }),
            input: json!({}),
            evidence_refs: vec![],
            artifact_refs: vec![],
            reason: None,
            expected_effects: vec![],
            ontology_version: ontology.bundle().definition_version.clone(),
            contract_version: registry.bundle().definition_version.clone(),
            created_at: "2026-06-20T00:00:00Z".into(),
        };

        let report = validate_action_proposal(&proposal, &registry, &ontology);
        assert!(report
            .errors
            .iter()
            .any(|error| error.code == "missingRequiredInput"));
    }

    #[test]
    fn proposal_with_unknown_target_object_type_fails() {
        let ontology = core_ontology_registry();
        let registry = registry();
        let proposal = ActionProposal {
            proposal_id: "proposal-1".into(),
            idempotency_key: "agent:work-agent:startRun:ghost:hash".into(),
            action_type: "startRun".into(),
            actor_role: "work-agent".into(),
            source_surface: ActionSourceSurface::Agent,
            target_object_ref: Some(crate::model::ActionRef {
                object_type: "Ghost".into(),
                id: "ghost-001".into(),
            }),
            input: json!({ "runId": "run-001" }),
            evidence_refs: vec![],
            artifact_refs: vec![],
            reason: None,
            expected_effects: vec![],
            ontology_version: ontology.bundle().definition_version.clone(),
            contract_version: registry.bundle().definition_version.clone(),
            created_at: "2026-06-20T00:00:00Z".into(),
        };

        let report = validate_action_proposal(&proposal, &registry, &ontology);
        assert!(report
            .errors
            .iter()
            .any(|error| error.code == "unknownTargetObjectType"));
    }

    #[test]
    fn invalid_expected_link_type_fails_bundle_validation() {
        let ontology = core_ontology_registry();
        let mut bundle = core_action_contract_bundle();
        bundle
            .contracts
            .iter_mut()
            .find(|contract| contract.action_type == "requestAudit")
            .unwrap()
            .expected_links
            .push("missing-link".into());
        let report = validate_action_contract_bundle(&bundle, &ontology);
        assert!(report
            .errors
            .iter()
            .any(|error| error.code == "unknown-expected-link-type"));
    }

    #[test]
    fn approve_spec_requires_human_confirmation() {
        let ontology = core_ontology_registry();
        let registry = registry();
        let proposal = ActionProposal {
            proposal_id: "proposal-1".into(),
            idempotency_key: "desktop:human:approveSpec:spec:hash".into(),
            action_type: "approveSpec".into(),
            actor_role: "human-owner".into(),
            source_surface: ActionSourceSurface::Desktop,
            target_object_ref: Some(crate::model::ActionRef {
                object_type: "Spec".into(),
                id: "spec-001".into(),
            }),
            input: json!({ "decisionSummary": "同意" }),
            evidence_refs: vec![],
            artifact_refs: vec![],
            reason: None,
            expected_effects: vec![],
            ontology_version: ontology.bundle().definition_version.clone(),
            contract_version: registry.bundle().definition_version.clone(),
            created_at: "2026-06-20T00:00:00Z".into(),
        };

        let report = validate_action_proposal(&proposal, &registry, &ontology);
        assert!(report
            .errors
            .iter()
            .any(|error| error.code == "missingRequiredEvidence"));
    }

    #[test]
    fn mark_issue_done_requires_verification_and_artifact_evidence() {
        let ontology = core_ontology_registry();
        let registry = registry();
        let proposal = ActionProposal {
            proposal_id: "proposal-1".into(),
            idempotency_key: "agent:work-agent:markIssueDone:issue:hash".into(),
            action_type: "markIssueDone".into(),
            actor_role: "work-agent".into(),
            source_surface: ActionSourceSurface::Agent,
            target_object_ref: Some(crate::model::ActionRef {
                object_type: "Issue".into(),
                id: "AF-001".into(),
            }),
            input: json!({ "completionSummary": "done" }),
            evidence_refs: vec!["summary-001".into(), "verification-001".into()],
            artifact_refs: vec![],
            reason: None,
            expected_effects: vec![],
            ontology_version: ontology.bundle().definition_version.clone(),
            contract_version: registry.bundle().definition_version.clone(),
            created_at: "2026-06-20T00:00:00Z".into(),
        };

        let report = validate_action_proposal(&proposal, &registry, &ontology);
        assert!(report
            .errors
            .iter()
            .any(|error| error.code == "missingRequiredEvidence"));
    }

    #[test]
    fn mark_issue_done_does_not_include_request_audit_side_effect() {
        let bundle = core_action_contract_bundle();
        let contract = bundle
            .contracts
            .iter()
            .find(|contract| contract.action_type == "markIssueDone")
            .unwrap();
        assert!(contract
            .effects
            .iter()
            .all(|effect| effect.event_type.as_deref() != Some("AuditRequested")));
    }

    #[test]
    fn request_audit_is_explicit_independent_action() {
        let bundle = core_action_contract_bundle();
        assert!(bundle
            .action_types
            .iter()
            .any(|action| action.id == "requestAudit"));
    }

    #[test]
    fn create_finding_creates_finding_not_audit_finding() {
        let bundle = core_action_contract_bundle();
        let contract = bundle
            .contracts
            .iter()
            .find(|contract| contract.action_type == "createFinding")
            .unwrap();
        assert_eq!(contract.creates_object_type.as_deref(), Some("Finding"));
        assert_ne!(
            contract.creates_object_type.as_deref(),
            Some("AuditFinding")
        );
    }

    #[test]
    fn work_package_target_is_rejected_in_mvp() {
        let ontology = core_ontology_registry();
        let mut bundle = core_action_contract_bundle();
        bundle.action_types[0].target_object_type = Some("WorkPackage".into());
        let report = validate_action_contract_bundle(&bundle, &ontology);
        assert!(report
            .errors
            .iter()
            .any(|error| error.code == "forbidden-mvp-object-type"));
    }

    #[test]
    fn idempotency_key_shape_is_required() {
        let ontology = core_ontology_registry();
        let registry = registry();
        let proposal = ActionProposal {
            proposal_id: "proposal-1".into(),
            idempotency_key: "bad".into(),
            action_type: "startRun".into(),
            actor_role: "work-agent".into(),
            source_surface: ActionSourceSurface::Agent,
            target_object_ref: Some(crate::model::ActionRef {
                object_type: "Issue".into(),
                id: "AF-001".into(),
            }),
            input: json!({ "runId": "run-001" }),
            evidence_refs: vec![],
            artifact_refs: vec![],
            reason: None,
            expected_effects: vec![],
            ontology_version: ontology.bundle().definition_version.clone(),
            contract_version: registry.bundle().definition_version.clone(),
            created_at: "2026-06-20T00:00:00Z".into(),
        };

        let report = validate_action_proposal(&proposal, &registry, &ontology);
        assert!(report
            .errors
            .iter()
            .any(|error| error.code == "invalidIdempotencyKey"));
    }

    #[test]
    fn contract_version_mismatch_returns_validation_error() {
        let ontology = core_ontology_registry();
        let registry = registry();
        let proposal = ActionProposal {
            proposal_id: "proposal-1".into(),
            idempotency_key: "agent:work-agent:startRun:issue:hash".into(),
            action_type: "startRun".into(),
            actor_role: "work-agent".into(),
            source_surface: ActionSourceSurface::Agent,
            target_object_ref: Some(crate::model::ActionRef {
                object_type: "Issue".into(),
                id: "AF-001".into(),
            }),
            input: json!({ "runId": "run-001" }),
            evidence_refs: vec![],
            artifact_refs: vec![],
            reason: None,
            expected_effects: vec![],
            ontology_version: ontology.bundle().definition_version.clone(),
            contract_version: "legacy-v0".into(),
            created_at: "2026-06-20T00:00:00Z".into(),
        };

        let report = validate_action_proposal(&proposal, &registry, &ontology);
        assert_eq!(
            report.status,
            crate::report::ActionProposalValidationStatus::VersionMismatch
        );
    }
}
