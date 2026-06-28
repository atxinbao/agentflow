use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    fmt, fs,
    path::{Path, PathBuf},
};

use crate::decision::CORE_EVIDENCE_DECISION_MODEL_VERSION;
use crate::decision::{
    core_evidence_decision_reference_model_contract,
    validate_core_evidence_decision_reference_model_contract,
    CoreEvidenceDecisionReferenceModelContract,
};
use crate::kernel::{
    core_ontology_kernel_contract, validate_core_ontology_kernel_contract,
    CoreOntologyKernelContract, CORE_ONTOLOGY_KERNEL_VERSION,
};
use crate::schema::CORE_OBJECT_LINK_SCHEMA_VERSION;
use crate::schema::{
    core_object_link_schema_contract, validate_core_object_link_schema_contract,
    CoreObjectLinkSchemaContract,
};
use crate::semantics::{
    core_action_state_semantics_contract, validate_core_action_state_semantics_contract,
    CoreActionStateSemanticsContract, CORE_ACTION_STATE_SEMANTICS_VERSION,
};
use crate::skill::{
    core_skill_registry_contract, validate_core_skill_registry_contract, CoreSkillRegistryContract,
    CORE_SKILL_REGISTRY_VERSION,
};

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreOntologyRegistryPollutionDiagnostic {
    pub field: String,
    pub original_text: String,
    pub normalized_term: String,
    pub mapping_boundary: String,
    pub allowed_as_reference_mapping: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadedCoreOntologyRegistrySource {
    pub source_id: String,
    pub relative_path: String,
    pub contract_version: String,
    pub read_model_kind: String,
    pub source_fingerprint: String,
    pub byte_len: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreFileBackedOntologyRuntimeProjection {
    pub version: String,
    pub status: String,
    pub registry_sources: Vec<LoadedCoreOntologyRegistrySource>,
    pub projection_entries: Vec<CoreOntologyProjectionEntryDefinition>,
    pub pollution_diagnostics: Vec<CoreOntologyRegistryPollutionDiagnostic>,
    pub core_ontology_kernel: CoreOntologyKernelContract,
    pub core_object_link_schema: CoreObjectLinkSchemaContract,
    pub core_action_state_semantics: CoreActionStateSemanticsContract,
    pub core_skill_registry: CoreSkillRegistryContract,
    pub core_evidence_decision_reference_model: CoreEvidenceDecisionReferenceModelContract,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreFileBackedOntologyRegistryLoadError {
    pub code: String,
    pub message: String,
    #[serde(default)]
    pub diagnostics: Vec<CoreOntologyRegistryPollutionDiagnostic>,
}

impl fmt::Display for CoreFileBackedOntologyRegistryLoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for CoreFileBackedOntologyRegistryLoadError {}

#[derive(Debug, Default)]
pub struct CoreFileBackedOntologyRuntimeLoader {
    cached: Option<CoreFileBackedOntologyRuntimeProjection>,
    cached_root: Option<PathBuf>,
    cached_fingerprints: Vec<String>,
}

impl CoreFileBackedOntologyRuntimeLoader {
    pub fn load(
        &mut self,
        project_root: impl AsRef<Path>,
    ) -> Result<CoreFileBackedOntologyRuntimeProjection, CoreFileBackedOntologyRegistryLoadError>
    {
        let root = resolve_registry_root(
            project_root.as_ref(),
            &core_file_backed_ontology_registry_projection_contract(),
        );
        let contract = core_file_backed_ontology_registry_projection_contract();
        let source_fingerprints = read_registry_source_fingerprints(&root, &contract)?;

        if self.cached_root.as_deref() == Some(root.as_path())
            && self.cached_fingerprints == source_fingerprints
        {
            if let Some(cached) = &self.cached {
                return Ok(cached.clone());
            }
        }

        let loaded =
            load_core_file_backed_ontology_registry_projection_with_contract(&root, &contract)?;
        self.cached_root = Some(root);
        self.cached_fingerprints = loaded
            .registry_sources
            .iter()
            .map(|source| source.source_fingerprint.clone())
            .collect();
        self.cached = Some(loaded.clone());
        Ok(loaded)
    }
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

pub fn load_core_file_backed_ontology_registry_projection(
    project_root: impl AsRef<Path>,
) -> Result<CoreFileBackedOntologyRuntimeProjection, CoreFileBackedOntologyRegistryLoadError> {
    load_core_file_backed_ontology_registry_projection_with_contract(
        project_root,
        &core_file_backed_ontology_registry_projection_contract(),
    )
}

pub fn load_core_file_backed_ontology_registry_projection_with_contract(
    project_root: impl AsRef<Path>,
    contract: &CoreFileBackedOntologyRegistryProjectionContract,
) -> Result<CoreFileBackedOntologyRuntimeProjection, CoreFileBackedOntologyRegistryLoadError> {
    if let Err(errors) = validate_core_file_backed_ontology_registry_projection_contract(contract) {
        return Err(load_error(
            "invalid-contract",
            errors.join("; "),
            Vec::new(),
        ));
    }

    let root = resolve_registry_root(project_root.as_ref(), contract);
    let loaded_sources = load_registry_sources(&root, contract)?;
    let core_ontology_kernel = core_ontology_kernel_contract();
    let core_object_link_schema = core_object_link_schema_contract();
    let core_action_state_semantics = core_action_state_semantics_contract();
    let core_skill_registry = core_skill_registry_contract();
    let core_evidence_decision_reference_model = core_evidence_decision_reference_model_contract();

    validate_typed_core_contracts(
        &core_ontology_kernel,
        &core_object_link_schema,
        &core_action_state_semantics,
        &core_skill_registry,
        &core_evidence_decision_reference_model,
    )?;

    let pollution_diagnostics =
        diagnose_core_file_backed_ontology_registry_projection_contract(contract);
    if !pollution_diagnostics.is_empty() {
        return Err(load_error(
            "forbidden-core-term",
            "file-backed ontology registry contains forbidden Core authority terms",
            pollution_diagnostics,
        ));
    }

    Ok(CoreFileBackedOntologyRuntimeProjection {
        version: contract.version.clone(),
        status: contract.status.clone(),
        registry_sources: loaded_sources,
        projection_entries: contract.projection_entries.clone(),
        pollution_diagnostics: Vec::new(),
        core_ontology_kernel,
        core_object_link_schema,
        core_action_state_semantics,
        core_skill_registry,
        core_evidence_decision_reference_model,
    })
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

    for diagnostic in diagnose_core_file_backed_ontology_registry_projection_contract(contract) {
        errors.push(format!(
            "forbidden industry term `{}` appears in `{}` as `{}`",
            diagnostic.normalized_term, diagnostic.field, diagnostic.original_text
        ));
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

pub fn diagnose_core_file_backed_ontology_registry_projection_contract(
    contract: &CoreFileBackedOntologyRegistryProjectionContract,
) -> Vec<CoreOntologyRegistryPollutionDiagnostic> {
    let mut diagnostics = Vec::new();

    for (index, source) in contract.registry_sources.iter().enumerate() {
        push_pollution_diagnostics(
            &format!("registrySources[{index}].sourceId"),
            &source.source_id,
            contract,
            &mut diagnostics,
        );
        push_pollution_diagnostics(
            &format!("registrySources[{index}].relativePath"),
            &source.relative_path,
            contract,
            &mut diagnostics,
        );
        push_pollution_diagnostics(
            &format!("registrySources[{index}].readModelKind"),
            &source.read_model_kind,
            contract,
            &mut diagnostics,
        );
    }
    for (index, projection) in contract.projection_entries.iter().enumerate() {
        push_pollution_diagnostics(
            &format!("projectionEntries[{index}].projectionId"),
            &projection.projection_id,
            contract,
            &mut diagnostics,
        );
        push_pollution_diagnostics(
            &format!("projectionEntries[{index}].sourceId"),
            &projection.source_id,
            contract,
            &mut diagnostics,
        );
        push_pollution_diagnostics(
            &format!("projectionEntries[{index}].projectionKind"),
            &projection.projection_kind,
            contract,
            &mut diagnostics,
        );
        for (surface_index, query_surface) in projection.query_surfaces.iter().enumerate() {
            push_pollution_diagnostics(
                &format!("projectionEntries[{index}].querySurfaces[{surface_index}]"),
                query_surface,
                contract,
                &mut diagnostics,
            );
        }
        push_pollution_diagnostics(
            &format!("projectionEntries[{index}].refreshRule"),
            &projection.refresh_rule,
            contract,
            &mut diagnostics,
        );
    }
    push_pollution_diagnostics("authority", &contract.authority, contract, &mut diagnostics);
    push_pollution_diagnostics(
        "storageBoundary",
        &contract.storage_boundary,
        contract,
        &mut diagnostics,
    );
    push_pollution_diagnostics(
        "projectionBoundary",
        &contract.projection_boundary,
        contract,
        &mut diagnostics,
    );

    diagnostics
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

fn push_pollution_diagnostics(
    field: &str,
    value: &str,
    contract: &CoreFileBackedOntologyRegistryProjectionContract,
    diagnostics: &mut Vec<CoreOntologyRegistryPollutionDiagnostic>,
) {
    for term in &contract.forbidden_core_terms {
        if contains_forbidden_core_term(value, term) {
            diagnostics.push(CoreOntologyRegistryPollutionDiagnostic {
                field: field.to_string(),
                original_text: value.to_string(),
                normalized_term: normalized_compact(term),
                mapping_boundary:
                    "allowed only as Reference App mapping; mappings are not Core authority"
                        .to_string(),
                allowed_as_reference_mapping: true,
            });
        }
    }
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

fn load_registry_sources(
    project_root: &Path,
    contract: &CoreFileBackedOntologyRegistryProjectionContract,
) -> Result<Vec<LoadedCoreOntologyRegistrySource>, CoreFileBackedOntologyRegistryLoadError> {
    contract
        .registry_sources
        .iter()
        .map(|source| load_registry_source(project_root, source))
        .collect()
}

fn resolve_registry_root(
    requested_root: &Path,
    contract: &CoreFileBackedOntologyRegistryProjectionContract,
) -> PathBuf {
    if all_registry_sources_exist(requested_root, contract) {
        return requested_root.to_path_buf();
    }

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    if all_registry_sources_exist(&source_root, contract) {
        return source_root;
    }

    requested_root.to_path_buf()
}

fn all_registry_sources_exist(
    root: &Path,
    contract: &CoreFileBackedOntologyRegistryProjectionContract,
) -> bool {
    contract.registry_sources.iter().all(|source| {
        let relative_path = Path::new(&source.relative_path);
        !relative_path.is_absolute()
            && !source.relative_path.contains("..")
            && root.join(relative_path).is_file()
    })
}

fn load_registry_source(
    project_root: &Path,
    source: &CoreOntologyRegistrySourceDefinition,
) -> Result<LoadedCoreOntologyRegistrySource, CoreFileBackedOntologyRegistryLoadError> {
    let relative_path = Path::new(&source.relative_path);
    if relative_path.is_absolute()
        || source.relative_path.contains("..")
        || source.relative_path.trim().is_empty()
    {
        return Err(load_error(
            "invalid-source-path",
            format!(
                "registry source `{}` must use a stable relative path",
                source.source_id
            ),
            Vec::new(),
        ));
    }

    let source_path = project_root.join(relative_path);
    let content = fs::read_to_string(&source_path).map_err(|error| {
        load_error(
            "missing-source",
            format!(
                "registry source `{}` could not be read at `{}`: {error}",
                source.source_id, source.relative_path
            ),
            Vec::new(),
        )
    })?;

    if !content.contains(&source.contract_version) {
        return Err(load_error(
            "malformed-contract",
            format!(
                "registry source `{}` does not declare contract version `{}`",
                source.source_id, source.contract_version
            ),
            Vec::new(),
        ));
    }
    if !content.contains("not Core authority") && !content.contains("不是 Core authority") {
        return Err(load_error(
            "malformed-contract",
            format!(
                "registry source `{}` must preserve Reference App mapping boundary",
                source.source_id
            ),
            Vec::new(),
        ));
    }

    Ok(LoadedCoreOntologyRegistrySource {
        source_id: source.source_id.clone(),
        relative_path: source.relative_path.clone(),
        contract_version: source.contract_version.clone(),
        read_model_kind: source.read_model_kind.clone(),
        source_fingerprint: source_fingerprint(&content),
        byte_len: content.len(),
    })
}

fn read_registry_source_fingerprints(
    project_root: &Path,
    contract: &CoreFileBackedOntologyRegistryProjectionContract,
) -> Result<Vec<String>, CoreFileBackedOntologyRegistryLoadError> {
    contract
        .registry_sources
        .iter()
        .map(|source| {
            let content =
                fs::read_to_string(project_root.join(&source.relative_path)).map_err(|error| {
                    load_error(
                        "missing-source",
                        format!(
                            "registry source `{}` could not be read at `{}`: {error}",
                            source.source_id, source.relative_path
                        ),
                        Vec::new(),
                    )
                })?;
            Ok(source_fingerprint(&content))
        })
        .collect()
}

fn source_fingerprint(content: &str) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in content.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("fnv1a64:{hash:016x}:{}", content.len())
}

fn validate_typed_core_contracts(
    kernel: &CoreOntologyKernelContract,
    schema: &CoreObjectLinkSchemaContract,
    semantics: &CoreActionStateSemanticsContract,
    skills: &CoreSkillRegistryContract,
    decisions: &CoreEvidenceDecisionReferenceModelContract,
) -> Result<(), CoreFileBackedOntologyRegistryLoadError> {
    let mut errors = Vec::new();
    if let Err(report) = validate_core_ontology_kernel_contract(kernel) {
        errors.extend(report);
    }
    if let Err(report) = validate_core_object_link_schema_contract(schema) {
        errors.extend(report);
    }
    if let Err(report) = validate_core_action_state_semantics_contract(semantics) {
        errors.extend(report);
    }
    if let Err(report) = validate_core_skill_registry_contract(skills) {
        errors.extend(report);
    }
    if let Err(report) = validate_core_evidence_decision_reference_model_contract(decisions) {
        errors.extend(report);
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(load_error(
            "malformed-core-contract",
            errors.join("; "),
            Vec::new(),
        ))
    }
}

fn load_error(
    code: impl Into<String>,
    message: impl Into<String>,
    diagnostics: Vec<CoreOntologyRegistryPollutionDiagnostic>,
) -> CoreFileBackedOntologyRegistryLoadError {
    CoreFileBackedOntologyRegistryLoadError {
        code: code.into(),
        message: message.into(),
        diagnostics,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

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
        assert!(errors.iter().any(|error| error.contains("githubissue")));
        let diagnostics =
            diagnose_core_file_backed_ontology_registry_projection_contract(&contract);
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.original_text == "githubIssueQuery"
                && diagnostic.normalized_term == "githubissue"
                && diagnostic.allowed_as_reference_mapping
        }));
    }

    #[test]
    fn core_file_backed_ontology_runtime_loader_reads_sources() {
        let projection = load_core_file_backed_ontology_registry_projection(".").unwrap();

        assert_eq!(projection.registry_sources.len(), 5);
        assert_eq!(projection.projection_entries.len(), 5);
        assert!(projection
            .registry_sources
            .iter()
            .all(|source| source.source_fingerprint.starts_with("fnv1a64:")));
        assert!(projection
            .core_action_state_semantics
            .actions
            .iter()
            .any(|action| action.action_type == "startObject"));
    }

    #[test]
    fn core_file_backed_ontology_runtime_loader_caches_by_source_fingerprint() {
        let mut loader = CoreFileBackedOntologyRuntimeLoader::default();

        let first = loader.load(".").unwrap();
        let second = loader.load(".").unwrap();

        assert_eq!(
            first.registry_sources[0].source_fingerprint,
            second.registry_sources[0].source_fingerprint
        );
    }

    #[test]
    fn core_file_backed_ontology_runtime_loader_rejects_missing_source() {
        let mut contract = core_file_backed_ontology_registry_projection_contract();
        contract.registry_sources[0].relative_path = "docs/architecture/missing.md".to_string();

        let error =
            load_core_file_backed_ontology_registry_projection_with_contract(".", &contract)
                .unwrap_err();
        assert_eq!(error.code, "missing-source");
    }

    #[test]
    fn core_file_backed_ontology_runtime_loader_rejects_malformed_contract() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("contract.md");
        let mut file = fs::File::create(&path).unwrap();
        writeln!(
            file,
            "# Broken\n\nThis file preserves mappings are not Core authority but omits version."
        )
        .unwrap();

        let mut contract = core_file_backed_ontology_registry_projection_contract();
        contract.registry_sources[0].relative_path = "contract.md".to_string();

        let error =
            load_core_file_backed_ontology_registry_projection_with_contract(dir.path(), &contract)
                .unwrap_err();
        assert_eq!(error.code, "malformed-contract");
    }
}
