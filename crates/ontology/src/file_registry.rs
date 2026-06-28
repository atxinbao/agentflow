use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

use crate::decision::CORE_EVIDENCE_DECISION_MODEL_VERSION;
use crate::kernel::CORE_ONTOLOGY_KERNEL_VERSION;
use crate::schema::CORE_OBJECT_LINK_SCHEMA_VERSION;
use crate::semantics::CORE_ACTION_STATE_SEMANTICS_VERSION;
use crate::skill::CORE_SKILL_REGISTRY_VERSION;

pub const CORE_FILE_BACKED_ONTOLOGY_REGISTRY_VERSION: &str =
    "agentflow-core-file-backed-ontology-registry.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreOntologyRegistrySourceDefinition {
    pub source_id: String,
    pub relative_path: String,
    pub contract_version: String,
    pub read_model_kind: String,
    pub authority_boundary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreOntologyProjectionEntryDefinition {
    pub projection_id: String,
    pub source_id: String,
    pub projection_kind: String,
    pub query_surfaces: Vec<String>,
    pub minimum_record_count: usize,
    pub refresh_rule: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreFileBackedOntologyRegistryProjectionContract {
    pub version: String,
    pub status: String,
    pub authority: String,
    pub storage_boundary: String,
    pub projection_boundary: String,
    pub registry_sources: Vec<CoreOntologyRegistrySourceDefinition>,
    pub projection_entries: Vec<CoreOntologyProjectionEntryDefinition>,
    pub forbidden_core_terms: Vec<String>,
}

pub fn core_file_backed_ontology_registry_projection_contract(
) -> CoreFileBackedOntologyRegistryProjectionContract {
    CoreFileBackedOntologyRegistryProjectionContract {
        version: CORE_FILE_BACKED_ONTOLOGY_REGISTRY_VERSION.to_string(),
        status: "active".to_string(),
        authority:
            "Core ontology registry reads versioned contract files and exposes read-only projections."
                .to_string(),
        storage_boundary:
            "Registry sources are file-backed Core contracts. Runtime facts, external tickets, and Reference App mappings are not Core authority."
                .to_string(),
        projection_boundary:
            "Projection entries are read models derived from registry sources. They may be queried by UI, Runtime, and Industry Apps, but they do not replace source contracts."
                .to_string(),
        registry_sources: vec![
            source(
                "core-ontology-kernel",
                "docs/architecture/054-core-ontology-kernel-contract-v1.md",
                CORE_ONTOLOGY_KERNEL_VERSION,
                "OntologyKernel",
            ),
            source(
                "core-object-link-schema",
                "docs/architecture/055-core-object-link-schema-v1.md",
                CORE_OBJECT_LINK_SCHEMA_VERSION,
                "ObjectLinkSchema",
            ),
            source(
                "core-action-state-semantics",
                "docs/architecture/056-core-action-state-semantics-v1.md",
                CORE_ACTION_STATE_SEMANTICS_VERSION,
                "ActionStateSemantics",
            ),
            source(
                "core-skill-registry",
                "docs/architecture/057-core-skill-registry-action-authorization-v1.md",
                CORE_SKILL_REGISTRY_VERSION,
                "SkillRegistry",
            ),
            source(
                "core-evidence-decision-reference-model",
                "docs/architecture/058-core-evidence-decision-reference-model-v1.md",
                CORE_EVIDENCE_DECISION_MODEL_VERSION,
                "EvidenceDecisionReferenceModel",
            ),
        ],
        projection_entries: vec![
            projection(
                "core-kernel-map",
                "core-ontology-kernel",
                "KernelElementCatalog",
                vec!["coreElementCatalog", "coreBoundaryMap"],
                11,
                "rebuild when a registry source contract version changes",
            ),
            projection(
                "core-object-link-catalog",
                "core-object-link-schema",
                "ObjectLinkCatalog",
                vec!["objectCatalog", "linkCatalog", "relationshipQuery"],
                23,
                "rebuild when object or link schema changes",
            ),
            projection(
                "core-action-state-catalog",
                "core-action-state-semantics",
                "ActionStateCatalog",
                vec!["actionCatalog", "stateCatalog", "transitionQuery"],
                34,
                "rebuild when action, state, or transition semantics change",
            ),
            projection(
                "core-skill-capability-catalog",
                "core-skill-registry",
                "SkillCapabilityCatalog",
                vec!["skillCatalog", "authorizationQuery", "capabilityMatrix"],
                6,
                "rebuild when skill capability definitions change",
            ),
            projection(
                "core-evidence-decision-catalog",
                "core-evidence-decision-reference-model",
                "EvidenceDecisionCatalog",
                vec!["evidenceCatalog", "decisionCatalog", "outcomeQuery"],
                18,
                "rebuild when evidence, decision, or outcome references change",
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

pub fn validate_core_file_backed_ontology_registry_projection_contract(
    contract: &CoreFileBackedOntologyRegistryProjectionContract,
) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    if contract.version != CORE_FILE_BACKED_ONTOLOGY_REGISTRY_VERSION {
        errors.push(format!(
            "file-backed ontology registry version must be `{}`",
            CORE_FILE_BACKED_ONTOLOGY_REGISTRY_VERSION
        ));
    }
    if contract.status != "active" {
        errors.push("file-backed ontology registry status must be active".to_string());
    }
    if !contract.storage_boundary.contains("not Core authority") {
        errors
            .push("storage boundary must say external mappings are not Core authority".to_string());
    }
    if !contract
        .projection_boundary
        .contains("do not replace source contracts")
    {
        errors.push(
            "projection boundary must say projections do not replace source contracts".to_string(),
        );
    }

    let required_sources: BTreeSet<_> = [
        "core-ontology-kernel",
        "core-object-link-schema",
        "core-action-state-semantics",
        "core-skill-registry",
        "core-evidence-decision-reference-model",
    ]
    .into_iter()
    .collect();
    let required_projections: BTreeSet<_> = [
        "core-kernel-map",
        "core-object-link-catalog",
        "core-action-state-catalog",
        "core-skill-capability-catalog",
        "core-evidence-decision-catalog",
    ]
    .into_iter()
    .collect();

    let source_ids: BTreeSet<_> = contract
        .registry_sources
        .iter()
        .map(|source| source.source_id.as_str())
        .collect();
    let projection_ids: BTreeSet<_> = contract
        .projection_entries
        .iter()
        .map(|projection| projection.projection_id.as_str())
        .collect();

    for required in &required_sources {
        if !source_ids.contains(required) {
            errors.push(format!("missing registry source `{required}`"));
        }
    }
    for required in &required_projections {
        if !projection_ids.contains(required) {
            errors.push(format!("missing projection entry `{required}`"));
        }
    }

    for source in &contract.registry_sources {
        if source.relative_path.starts_with('/')
            || source.relative_path.contains("..")
            || source.relative_path.trim().is_empty()
        {
            errors.push(format!(
                "registry source `{}` must use a stable relative path",
                source.source_id
            ));
        }
        if source.contract_version.trim().is_empty() {
            errors.push(format!(
                "registry source `{}` must declare a contract version",
                source.source_id
            ));
        }
        if !source.authority_boundary.contains("not Core authority") {
            errors.push(format!(
                "registry source `{}` must mark mappings as not Core authority",
                source.source_id
            ));
        }
    }

    for projection in &contract.projection_entries {
        if !source_ids.contains(projection.source_id.as_str()) {
            errors.push(format!(
                "projection `{}` references missing source `{}`",
                projection.projection_id, projection.source_id
            ));
        }
        if projection.query_surfaces.is_empty() {
            errors.push(format!(
                "projection `{}` must declare query surfaces",
                projection.projection_id
            ));
        }
        if projection.minimum_record_count == 0 {
            errors.push(format!(
                "projection `{}` must declare minimum record count",
                projection.projection_id
            ));
        }
    }

    let core_surface = contract
        .registry_sources
        .iter()
        .flat_map(|source| {
            [
                source.source_id.clone(),
                source.relative_path.clone(),
                source.contract_version.clone(),
                source.read_model_kind.clone(),
                source.authority_boundary.clone(),
            ]
        })
        .chain(contract.projection_entries.iter().flat_map(|projection| {
            [
                projection.projection_id.clone(),
                projection.source_id.clone(),
                projection.projection_kind.clone(),
                projection.query_surfaces.join(" "),
                projection.refresh_rule.clone(),
            ]
        }))
        .chain([
            contract.authority.clone(),
            contract.storage_boundary.clone(),
            contract.projection_boundary.clone(),
        ])
        .collect::<Vec<_>>();
    for term in &contract.forbidden_core_terms {
        if core_surface
            .iter()
            .any(|value| contains_forbidden_core_term(value, term))
        {
            errors.push(format!(
                "forbidden industry term `{term}` appears in Core file-backed ontology registry"
            ));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn source(
    source_id: &str,
    relative_path: &str,
    contract_version: &str,
    read_model_kind: &str,
) -> CoreOntologyRegistrySourceDefinition {
    CoreOntologyRegistrySourceDefinition {
        source_id: source_id.to_string(),
        relative_path: relative_path.to_string(),
        contract_version: contract_version.to_string(),
        read_model_kind: read_model_kind.to_string(),
        authority_boundary:
            "Reference App mappings may point at this source, but mappings are not Core authority."
                .to_string(),
    }
}

fn projection(
    projection_id: &str,
    source_id: &str,
    projection_kind: &str,
    query_surfaces: Vec<&str>,
    minimum_record_count: usize,
    refresh_rule: &str,
) -> CoreOntologyProjectionEntryDefinition {
    CoreOntologyProjectionEntryDefinition {
        projection_id: projection_id.to_string(),
        source_id: source_id.to_string(),
        projection_kind: projection_kind.to_string(),
        query_surfaces: query_surfaces.into_iter().map(str::to_string).collect(),
        minimum_record_count,
        refresh_rule: refresh_rule.to_string(),
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
    fn core_file_backed_ontology_registry_contract_validates() {
        let contract = core_file_backed_ontology_registry_projection_contract();
        validate_core_file_backed_ontology_registry_projection_contract(&contract).unwrap();
        assert_eq!(contract.registry_sources.len(), 5);
        assert_eq!(contract.projection_entries.len(), 5);
    }

    #[test]
    fn core_file_backed_ontology_registry_rejects_absolute_path() {
        let mut contract = core_file_backed_ontology_registry_projection_contract();
        contract.registry_sources[0].relative_path = "/tmp/core-ontology-kernel.json".to_string();

        let errors =
            validate_core_file_backed_ontology_registry_projection_contract(&contract).unwrap_err();
        assert!(errors
            .iter()
            .any(|error| error.contains("stable relative path")));
    }

    #[test]
    fn core_file_backed_ontology_registry_rejects_missing_projection_source() {
        let mut contract = core_file_backed_ontology_registry_projection_contract();
        contract.projection_entries[0].source_id = "missing-source".to_string();

        let errors =
            validate_core_file_backed_ontology_registry_projection_contract(&contract).unwrap_err();
        assert!(errors
            .iter()
            .any(|error| error.contains("references missing source")));
    }

    #[test]
    fn core_file_backed_ontology_registry_rejects_industry_pollution() {
        let mut contract = core_file_backed_ontology_registry_projection_contract();
        contract.projection_entries[0]
            .query_surfaces
            .push("githubIssueQuery".to_string());

        let errors =
            validate_core_file_backed_ontology_registry_projection_contract(&contract).unwrap_err();
        assert!(errors.iter().any(|error| error.contains("github-issue")));
    }
}
