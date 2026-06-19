use std::collections::{BTreeMap, BTreeSet};

use anyhow::Result;

use crate::model::{
    DefinitionKind, DefinitionStatus, LinkTypeDefinition, OntologyBundle, OntologyValidationReport,
};
use crate::registry::OntologyRegistry;

pub fn validate_ontology_bundle(bundle: &OntologyBundle) -> OntologyValidationReport {
    let mut report = OntologyValidationReport::success();

    if bundle.ontology_id.trim().is_empty() {
        report.push_error(
            "ontology-id-missing",
            "ontologyId must not be empty",
            Some("ontologyId".into()),
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
            "version must not be empty",
            Some("version".into()),
        );
    }

    let mut object_ids = BTreeSet::new();
    for (index, object_type) in bundle.object_types.iter().enumerate() {
        if !object_ids.insert(object_type.id.clone()) {
            report.push_error(
                "duplicate-object-type",
                format!("object type `{}` is defined more than once", object_type.id),
                Some(format!("objectTypes[{index}].id")),
            );
        }

        let properties: BTreeSet<_> = object_type
            .properties
            .iter()
            .map(|item| item.name.as_str())
            .collect();
        for required in &object_type.required_properties {
            if !properties.contains(required.as_str()) {
                report.push_error(
                    "missing-required-property",
                    format!(
                        "object type `{}` requires property `{required}` but it is not defined",
                        object_type.id
                    ),
                    Some(format!("objectTypes[{index}].requiredProperties")),
                );
            }
        }

        if let Some(state_machine_ref) = &object_type.state_machine_ref {
            if !is_valid_ref(state_machine_ref) {
                report.push_error(
                    "invalid-state-machine-ref",
                    format!(
                        "object type `{}` has invalid stateMachineRef `{state_machine_ref}`",
                        object_type.id
                    ),
                    Some(format!("objectTypes[{index}].stateMachineRef")),
                );
            }
        }
    }

    let object_id_map: BTreeSet<_> = bundle
        .object_types
        .iter()
        .map(|item| item.id.as_str())
        .collect();

    let mut link_ids = BTreeSet::new();
    for (index, link_type) in bundle.link_types.iter().enumerate() {
        if !link_ids.insert(link_type.id.clone()) {
            report.push_error(
                "duplicate-link-type",
                format!("link type `{}` is defined more than once", link_type.id),
                Some(format!("linkTypes[{index}].id")),
            );
        }
        if !object_id_map.contains(link_type.source_object_type.as_str()) {
            report.push_error(
                "missing-link-source",
                format!(
                    "link `{}` references missing source object type `{}`",
                    link_type.id, link_type.source_object_type
                ),
                Some(format!("linkTypes[{index}].sourceObjectType")),
            );
        }
        if !object_id_map.contains(link_type.target_object_type.as_str()) {
            report.push_error(
                "missing-link-target",
                format!(
                    "link `{}` references missing target object type `{}`",
                    link_type.id, link_type.target_object_type
                ),
                Some(format!("linkTypes[{index}].targetObjectType")),
            );
        }
    }

    let link_id_map: BTreeSet<_> = bundle
        .link_types
        .iter()
        .map(|item| item.id.as_str())
        .collect();
    for (index, object_type) in bundle.object_types.iter().enumerate() {
        for link_type_id in &object_type.allowed_link_types {
            if !link_id_map.contains(link_type_id.as_str()) {
                report.push_error(
                    "missing-allowed-link-type",
                    format!(
                        "object type `{}` references missing allowed link `{link_type_id}`",
                        object_type.id
                    ),
                    Some(format!("objectTypes[{index}].allowedLinkTypes")),
                );
            }
        }
    }

    let definition_index = build_definition_index(bundle);
    let mut record_ids = BTreeSet::new();
    for (index, record) in bundle.definition_records.iter().enumerate() {
        if !record_ids.insert(record.id.clone()) {
            report.push_error(
                "duplicate-definition-record",
                format!(
                    "definition record `{}` is defined more than once",
                    record.id
                ),
                Some(format!("definitionRecords[{index}].id")),
            );
        }
        match record.kind {
            DefinitionKind::ObjectType => {
                if !definition_index
                    .object_types
                    .contains_key(record.id.as_str())
                {
                    report.push_error(
                        "missing-object-definition-record-target",
                        format!(
                            "definition record `{}` points to missing object type",
                            record.id
                        ),
                        Some(format!("definitionRecords[{index}]")),
                    );
                }
            }
            DefinitionKind::LinkType => {
                if !definition_index.link_types.contains_key(record.id.as_str()) {
                    report.push_error(
                        "missing-link-definition-record-target",
                        format!(
                            "definition record `{}` points to missing link type",
                            record.id
                        ),
                        Some(format!("definitionRecords[{index}]")),
                    );
                }
            }
            _ => {
                report.push_error(
                    "unsupported-definition-kind",
                    format!(
                        "definition record `{}` uses unsupported kind for AF-OS-001",
                        record.id
                    ),
                    Some(format!("definitionRecords[{index}].kind")),
                );
            }
        }
    }

    let active_link_records = active_link_record_statuses(bundle);
    let object_record_statuses = object_record_statuses(bundle);
    for link_type in &bundle.link_types {
        if matches!(
            active_link_records.get(link_type.id.as_str()),
            Some(DefinitionStatus::Active)
        ) {
            validate_endpoint_status(
                &mut report,
                link_type,
                &object_record_statuses,
                "source",
                &link_type.source_object_type,
            );
            validate_endpoint_status(
                &mut report,
                link_type,
                &object_record_statuses,
                "target",
                &link_type.target_object_type,
            );
        }
    }

    report
}

pub fn validate_link(
    registry: &OntologyRegistry,
    link_type_id: &str,
    source_type: &str,
    target_type: &str,
) -> Result<()> {
    registry.validate_link_endpoint(link_type_id, source_type, target_type)
}

fn is_valid_ref(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty()
        && !trimmed.contains(' ')
        && trimmed.contains('.')
        && !trimmed.starts_with('.')
        && !trimmed.ends_with('.')
}

fn active_link_record_statuses(bundle: &OntologyBundle) -> BTreeMap<&str, DefinitionStatus> {
    bundle
        .definition_records
        .iter()
        .filter(|record| matches!(record.kind, DefinitionKind::LinkType))
        .map(|record| (record.id.as_str(), record.status.clone()))
        .collect()
}

fn object_record_statuses(bundle: &OntologyBundle) -> BTreeMap<&str, DefinitionStatus> {
    bundle
        .definition_records
        .iter()
        .filter(|record| matches!(record.kind, DefinitionKind::ObjectType))
        .map(|record| (record.id.as_str(), record.status.clone()))
        .collect()
}

fn validate_endpoint_status(
    report: &mut OntologyValidationReport,
    link_type: &LinkTypeDefinition,
    object_statuses: &BTreeMap<&str, DefinitionStatus>,
    side: &str,
    object_id: &str,
) {
    if matches!(
        object_statuses.get(object_id),
        Some(DefinitionStatus::Deprecated | DefinitionStatus::Retired)
    ) {
        report.push_error(
            "deprecated-endpoint-reference",
            format!(
                "active link `{}` references deprecated {side} object type `{object_id}`",
                link_type.id
            ),
            Some(format!("linkTypes[{}].{side}", link_type.id)),
        );
    }
}

struct DefinitionIndex<'a> {
    object_types: BTreeMap<&'a str, &'a str>,
    link_types: BTreeMap<&'a str, &'a str>,
}

fn build_definition_index(bundle: &OntologyBundle) -> DefinitionIndex<'_> {
    let object_types = bundle
        .object_types
        .iter()
        .map(|definition| (definition.id.as_str(), definition.name.as_str()))
        .collect();
    let link_types = bundle
        .link_types
        .iter()
        .map(|definition| (definition.id.as_str(), definition.description.as_str()))
        .collect();
    DefinitionIndex {
        object_types,
        link_types,
    }
}

#[cfg(test)]
mod tests {
    use crate::core::core_ontology_bundle;
    use crate::model::{DefinitionKind, DefinitionStatus, OntologyDefinitionRecord};
    use crate::registry::OntologyRegistry;

    use super::validate_ontology_bundle;

    #[test]
    fn core_ontology_bundle_validates() {
        let report = validate_ontology_bundle(&core_ontology_bundle());
        assert!(report.valid, "{:?}", report.errors);
    }

    #[test]
    fn duplicate_object_type_id_fails() {
        let mut bundle = core_ontology_bundle();
        bundle.object_types.push(bundle.object_types[0].clone());

        let report = validate_ontology_bundle(&bundle);
        assert!(!report.valid);
        assert!(report
            .errors
            .iter()
            .any(|error| error.code == "duplicate-object-type"));
    }

    #[test]
    fn duplicate_link_type_id_fails() {
        let mut bundle = core_ontology_bundle();
        bundle.link_types.push(bundle.link_types[0].clone());

        let report = validate_ontology_bundle(&bundle);
        assert!(!report.valid);
        assert!(report
            .errors
            .iter()
            .any(|error| error.code == "duplicate-link-type"));
    }

    #[test]
    fn link_with_missing_source_object_fails() {
        let mut bundle = core_ontology_bundle();
        bundle.link_types[0].source_object_type = "Missing".into();

        let report = validate_ontology_bundle(&bundle);
        assert!(!report.valid);
        assert!(report
            .errors
            .iter()
            .any(|error| error.code == "missing-link-source"));
    }

    #[test]
    fn link_with_missing_target_object_fails() {
        let mut bundle = core_ontology_bundle();
        bundle.link_types[0].target_object_type = "Missing".into();

        let report = validate_ontology_bundle(&bundle);
        assert!(!report.valid);
        assert!(report
            .errors
            .iter()
            .any(|error| error.code == "missing-link-target"));
    }

    #[test]
    fn object_allowed_link_type_must_exist() {
        let mut bundle = core_ontology_bundle();
        bundle.object_types[0]
            .allowed_link_types
            .push("does-not-exist".into());

        let report = validate_ontology_bundle(&bundle);
        assert!(!report.valid);
        assert!(report
            .errors
            .iter()
            .any(|error| error.code == "missing-allowed-link-type"));
    }

    #[test]
    fn definition_record_must_point_to_existing_definition() {
        let mut bundle = core_ontology_bundle();
        bundle.definition_records.push(OntologyDefinitionRecord {
            version: "agentflow-ontology-record.v1".into(),
            id: "Ghost".into(),
            namespace: bundle.namespace.clone(),
            kind: DefinitionKind::ObjectType,
            definition_version: bundle.definition_version.clone(),
            status: DefinitionStatus::Draft,
            owner: "agentflow".into(),
            created_at: "2026-06-20T00:00:00Z".into(),
            updated_at: "2026-06-20T00:00:00Z".into(),
            compatibility: None,
            deprecation: None,
        });

        let report = validate_ontology_bundle(&bundle);
        assert!(!report.valid);
        assert!(report
            .errors
            .iter()
            .any(|error| error.code == "missing-object-definition-record-target"));
    }

    #[test]
    fn project_contains_issue_validates() {
        let registry = OntologyRegistry::load_bundle(core_ontology_bundle()).unwrap();
        registry
            .validate_link_endpoint("contains", "Project", "Issue")
            .unwrap();
    }

    #[test]
    fn run_executes_issue_validates() {
        let registry = OntologyRegistry::load_bundle(core_ontology_bundle()).unwrap();
        registry
            .validate_link_endpoint("executes", "Run", "Issue")
            .unwrap();
    }

    #[test]
    fn finding_requires_fix_issue_validates() {
        let registry = OntologyRegistry::load_bundle(core_ontology_bundle()).unwrap();
        registry
            .validate_link_endpoint("requiresFix", "Finding", "Issue")
            .unwrap();
    }

    #[test]
    fn missing_work_package_does_not_fail_mvp_bundle() {
        let registry = OntologyRegistry::load_bundle(core_ontology_bundle()).unwrap();
        assert!(registry.get_object_type("WorkPackage").is_none());
        assert!(registry.get_object_type("Project").is_some());
    }
}
