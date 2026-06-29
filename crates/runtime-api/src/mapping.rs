use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use agentflow_action_contract::{core_action_contract_registry, ActionSourceSurface};
use agentflow_action_contract::{ActionProposal, ActionRef};
use agentflow_ontology::{
    load_core_file_backed_ontology_registry_projection, software_dev_reference_ontology_registry,
    CoreFileBackedOntologyRuntimeProjection,
};

use crate::commands::{RuntimeCommandRequest, RuntimeCommandRoute};
use crate::errors::{RuntimeCommandError, RuntimeCommandErrorCode};

pub const CORE_RUNTIME_COMMAND_TYPE: &str = "core.action.invoke";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeQueryHint {
    pub view: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_id: Option<String>,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeReferenceAppMapping {
    pub mapping_id: String,
    pub pack_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pack_command: Option<String>,
    pub action_contract_ref: String,
    pub app_action_type: String,
    pub app_target_object_type: String,
    #[serde(default)]
    pub source_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeActionProposalMaterialization {
    pub core_action_type: String,
    pub core_target_object_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub core_required_state: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub core_resulting_state: Option<String>,
    #[serde(default)]
    pub core_required_evidence: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub core_expected_event: Option<String>,
    pub reference_mapping: RuntimeReferenceAppMapping,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeReferenceAppMappingDefinition {
    pub action_contract_ref: &'static str,
    pub app_action_type: &'static str,
    pub core_action_type: &'static str,
    pub reference_target_object_type: &'static str,
    pub pack_id: &'static str,
    pub source_ref: &'static str,
}

const SOFTWARE_DEV_REFERENCE_PACK_ID: &str = "software-dev-reference";
const SOFTWARE_DEV_REFERENCE_MAPPING_SOURCE: &str = "reference-app:software-dev";

const SOFTWARE_DEV_REFERENCE_MAPPING_CATALOG: &[RuntimeReferenceAppMappingDefinition] = &[
    RuntimeReferenceAppMappingDefinition {
        action_contract_ref: "action-contract:requirement.submit",
        app_action_type: "submitRequirement",
        core_action_type: "captureObject",
        reference_target_object_type: "Requirement",
        pack_id: SOFTWARE_DEV_REFERENCE_PACK_ID,
        source_ref: SOFTWARE_DEV_REFERENCE_MAPPING_SOURCE,
    },
    RuntimeReferenceAppMappingDefinition {
        action_contract_ref: "action-contract:spec.intake",
        app_action_type: "submitRequirement",
        core_action_type: "captureObject",
        reference_target_object_type: "Requirement",
        pack_id: SOFTWARE_DEV_REFERENCE_PACK_ID,
        source_ref: SOFTWARE_DEV_REFERENCE_MAPPING_SOURCE,
    },
    RuntimeReferenceAppMappingDefinition {
        action_contract_ref: "action-contract:requirement.normalize",
        app_action_type: "normalizeRequirement",
        core_action_type: "normalizeObject",
        reference_target_object_type: "Requirement",
        pack_id: SOFTWARE_DEV_REFERENCE_PACK_ID,
        source_ref: SOFTWARE_DEV_REFERENCE_MAPPING_SOURCE,
    },
    RuntimeReferenceAppMappingDefinition {
        action_contract_ref: "action-contract:requirement.classify",
        app_action_type: "classifyRequirement",
        core_action_type: "routeObject",
        reference_target_object_type: "Requirement",
        pack_id: SOFTWARE_DEV_REFERENCE_PACK_ID,
        source_ref: SOFTWARE_DEV_REFERENCE_MAPPING_SOURCE,
    },
    RuntimeReferenceAppMappingDefinition {
        action_contract_ref: "action-contract:spec.draft",
        app_action_type: "draftSpec",
        core_action_type: "routeObject",
        reference_target_object_type: "Requirement",
        pack_id: SOFTWARE_DEV_REFERENCE_PACK_ID,
        source_ref: SOFTWARE_DEV_REFERENCE_MAPPING_SOURCE,
    },
    RuntimeReferenceAppMappingDefinition {
        action_contract_ref: "action-contract:spec.approve",
        app_action_type: "approveSpec",
        core_action_type: "acceptObject",
        reference_target_object_type: "Spec",
        pack_id: SOFTWARE_DEV_REFERENCE_PACK_ID,
        source_ref: SOFTWARE_DEV_REFERENCE_MAPPING_SOURCE,
    },
    RuntimeReferenceAppMappingDefinition {
        action_contract_ref: "action-contract:project.create",
        app_action_type: "createProject",
        core_action_type: "routeObject",
        reference_target_object_type: "Spec",
        pack_id: SOFTWARE_DEV_REFERENCE_PACK_ID,
        source_ref: SOFTWARE_DEV_REFERENCE_MAPPING_SOURCE,
    },
    RuntimeReferenceAppMappingDefinition {
        action_contract_ref: "action-contract:issue.create",
        app_action_type: "createIssue",
        core_action_type: "routeObject",
        reference_target_object_type: "Project",
        pack_id: SOFTWARE_DEV_REFERENCE_PACK_ID,
        source_ref: SOFTWARE_DEV_REFERENCE_MAPPING_SOURCE,
    },
    RuntimeReferenceAppMappingDefinition {
        action_contract_ref: "action-contract:issue.activate",
        app_action_type: "activateIssue",
        core_action_type: "acceptObject",
        reference_target_object_type: "Issue",
        pack_id: SOFTWARE_DEV_REFERENCE_PACK_ID,
        source_ref: SOFTWARE_DEV_REFERENCE_MAPPING_SOURCE,
    },
    RuntimeReferenceAppMappingDefinition {
        action_contract_ref: "action-contract:issue.claim",
        app_action_type: "claimIssue",
        core_action_type: "startObject",
        reference_target_object_type: "Issue",
        pack_id: SOFTWARE_DEV_REFERENCE_PACK_ID,
        source_ref: SOFTWARE_DEV_REFERENCE_MAPPING_SOURCE,
    },
    RuntimeReferenceAppMappingDefinition {
        action_contract_ref: "action-contract:issue.start",
        app_action_type: "startRun",
        core_action_type: "startObject",
        reference_target_object_type: "Issue",
        pack_id: SOFTWARE_DEV_REFERENCE_PACK_ID,
        source_ref: SOFTWARE_DEV_REFERENCE_MAPPING_SOURCE,
    },
    RuntimeReferenceAppMappingDefinition {
        action_contract_ref: "action-contract:run.patch.write",
        app_action_type: "writePatch",
        core_action_type: "attachArtifact",
        reference_target_object_type: "Run",
        pack_id: SOFTWARE_DEV_REFERENCE_PACK_ID,
        source_ref: SOFTWARE_DEV_REFERENCE_MAPPING_SOURCE,
    },
    RuntimeReferenceAppMappingDefinition {
        action_contract_ref: "action-contract:acceptance.evaluate",
        app_action_type: "runValidation",
        core_action_type: "attachEvidence",
        reference_target_object_type: "Run",
        pack_id: SOFTWARE_DEV_REFERENCE_PACK_ID,
        source_ref: SOFTWARE_DEV_REFERENCE_MAPPING_SOURCE,
    },
    RuntimeReferenceAppMappingDefinition {
        action_contract_ref: "action-contract:delivery.prepare",
        app_action_type: "prepareDelivery",
        core_action_type: "attachArtifact",
        reference_target_object_type: "Run",
        pack_id: SOFTWARE_DEV_REFERENCE_PACK_ID,
        source_ref: SOFTWARE_DEV_REFERENCE_MAPPING_SOURCE,
    },
    RuntimeReferenceAppMappingDefinition {
        action_contract_ref: "action-contract:delivery.open",
        app_action_type: "prepareDelivery",
        core_action_type: "attachArtifact",
        reference_target_object_type: "Run",
        pack_id: SOFTWARE_DEV_REFERENCE_PACK_ID,
        source_ref: SOFTWARE_DEV_REFERENCE_MAPPING_SOURCE,
    },
    RuntimeReferenceAppMappingDefinition {
        action_contract_ref: "action-contract:evidence.submit",
        app_action_type: "submitEvidence",
        core_action_type: "attachEvidence",
        reference_target_object_type: "Run",
        pack_id: SOFTWARE_DEV_REFERENCE_PACK_ID,
        source_ref: SOFTWARE_DEV_REFERENCE_MAPPING_SOURCE,
    },
    RuntimeReferenceAppMappingDefinition {
        action_contract_ref: "action-contract:artifact.submit",
        app_action_type: "submitArtifact",
        core_action_type: "attachArtifact",
        reference_target_object_type: "Run",
        pack_id: SOFTWARE_DEV_REFERENCE_PACK_ID,
        source_ref: SOFTWARE_DEV_REFERENCE_MAPPING_SOURCE,
    },
    RuntimeReferenceAppMappingDefinition {
        action_contract_ref: "action-contract:issue.done",
        app_action_type: "markIssueDone",
        core_action_type: "completeObject",
        reference_target_object_type: "Issue",
        pack_id: SOFTWARE_DEV_REFERENCE_PACK_ID,
        source_ref: SOFTWARE_DEV_REFERENCE_MAPPING_SOURCE,
    },
    RuntimeReferenceAppMappingDefinition {
        action_contract_ref: "action-contract:decision.record",
        app_action_type: "recordDecision",
        core_action_type: "completeObject",
        reference_target_object_type: "Decision",
        pack_id: SOFTWARE_DEV_REFERENCE_PACK_ID,
        source_ref: SOFTWARE_DEV_REFERENCE_MAPPING_SOURCE,
    },
    RuntimeReferenceAppMappingDefinition {
        action_contract_ref: "action-contract:audit.request",
        app_action_type: "requestAudit",
        core_action_type: "submitForReview",
        reference_target_object_type: "Issue",
        pack_id: SOFTWARE_DEV_REFERENCE_PACK_ID,
        source_ref: SOFTWARE_DEV_REFERENCE_MAPPING_SOURCE,
    },
    RuntimeReferenceAppMappingDefinition {
        action_contract_ref: "action-contract:finding.create",
        app_action_type: "createFinding",
        core_action_type: "blockObject",
        reference_target_object_type: "Audit",
        pack_id: SOFTWARE_DEV_REFERENCE_PACK_ID,
        source_ref: SOFTWARE_DEV_REFERENCE_MAPPING_SOURCE,
    },
    RuntimeReferenceAppMappingDefinition {
        action_contract_ref: "action-contract:finding.link-fix-issue",
        app_action_type: "linkFixIssue",
        core_action_type: "supersedeObject",
        reference_target_object_type: "Finding",
        pack_id: SOFTWARE_DEV_REFERENCE_PACK_ID,
        source_ref: SOFTWARE_DEV_REFERENCE_MAPPING_SOURCE,
    },
];

pub fn software_dev_reference_mapping_catalog() -> &'static [RuntimeReferenceAppMappingDefinition] {
    SOFTWARE_DEV_REFERENCE_MAPPING_CATALOG
}

pub fn map_command_to_action_proposal(request: &RuntimeCommandRequest) -> Result<ActionProposal> {
    let ontology = software_dev_reference_ontology_registry();
    let contracts = core_action_contract_registry(&ontology);
    let action_type = resolve_core_action_type(request).ok_or_else(|| {
        anyhow!(
            "unsupported runtime command `{}`: Core commands must resolve through route.actionContractRef",
            request.command_type
        )
    })?;
    let action_type_definition = contracts
        .get_action_type(action_type)
        .ok_or_else(|| anyhow!("missing action type definition `{action_type}`"))?;

    Ok(ActionProposal {
        proposal_id: format!("proposal-{}", request.command_id),
        idempotency_key: request.idempotency_key.clone(),
        action_type: action_type.to_string(),
        actor_role: request.actor_role.clone(),
        source_surface: request.source_surface.clone(),
        target_object_ref: request.target_object_ref.clone(),
        input: request.input.clone(),
        evidence_refs: request.evidence_refs.clone(),
        artifact_refs: request.artifact_refs.clone(),
        reason: Some(format!(
            "runtime command `{}` from {:?}",
            request.command_type, request.source_surface
        )),
        expected_effects: action_type_definition
            .contract_ref
            .split('/')
            .map(|value| value.to_string())
            .collect(),
        ontology_version: ontology.bundle().definition_version.clone(),
        contract_version: contracts.bundle().definition_version.clone(),
        created_at: request.created_at.clone(),
    })
}

pub fn materialize_core_action_proposal(
    request: &RuntimeCommandRequest,
    proposal: &ActionProposal,
) -> Result<RuntimeActionProposalMaterialization> {
    let registry = load_core_file_backed_ontology_registry_projection(".")?;
    materialize_core_action_proposal_with_registry(request, proposal, &registry)
}

pub fn materialize_core_action_proposal_with_registry(
    request: &RuntimeCommandRequest,
    proposal: &ActionProposal,
    registry: &CoreFileBackedOntologyRuntimeProjection,
) -> Result<RuntimeActionProposalMaterialization> {
    let route = request.route.as_ref().ok_or_else(|| {
        anyhow!(
            "missing reference mapping: route.actionContractRef is required for Core proposal materialization"
        )
    })?;
    if route.action_contract_ref.trim().is_empty() {
        anyhow::bail!(
            "missing reference mapping: route.actionContractRef is required for Core proposal materialization"
        );
    }

    let mapping = reference_mapping_for_action_contract_ref(&route.action_contract_ref)
        .ok_or_else(|| {
            anyhow!(
                "missing Core action mapping for reference action contract `{}`",
                route.action_contract_ref
            )
        })?;
    if mapping.app_action_type != proposal.action_type {
        anyhow::bail!(
            "reference mapping mismatch: route action contract `{}` maps to `{}`, got proposal action `{}`",
            route.action_contract_ref,
            mapping.app_action_type,
            proposal.action_type
        );
    }
    let core_action = registry
        .core_action_state_semantics
        .actions
        .iter()
        .find(|action| action.action_type == mapping.core_action_type)
        .ok_or_else(|| anyhow!("unknown Core action `{}`", mapping.core_action_type))?;
    if !registry
        .core_object_link_schema
        .object_schemas
        .iter()
        .any(|object| object.object_type == core_action.target_object_type)
    {
        anyhow::bail!(
            "unknown Core target object `{}` for action `{}`",
            core_action.target_object_type,
            mapping.core_action_type
        );
    }

    let app_target_object_type = route
        .target_object_type
        .clone()
        .or_else(|| {
            proposal
                .target_object_ref
                .as_ref()
                .map(|target| target.object_type.clone())
        })
        .or_else(|| {
            reference_target_object_type_for_action(&proposal.action_type).map(str::to_string)
        })
        .ok_or_else(|| {
            anyhow!(
                "missing reference mapping: app target object type is required for `{}`",
                proposal.action_type
            )
        })?;
    if app_target_object_type == core_action.target_object_type {
        anyhow::bail!(
            "polluted Core proposal: app target object `{app_target_object_type}` must be mapped through reference mapping, not used as Core target"
        );
    }

    Ok(RuntimeActionProposalMaterialization {
        core_action_type: core_action.action_type.clone(),
        core_target_object_type: core_action.target_object_type.clone(),
        core_required_state: core_action.required_state.clone(),
        core_resulting_state: core_action.resulting_state.clone(),
        core_required_evidence: registry
            .core_action_state_semantics
            .transitions
            .iter()
            .filter(|transition| transition.action_type == core_action.action_type)
            .flat_map(|transition| transition.required_evidence.iter().cloned())
            .collect(),
        core_expected_event: Some(core_action.emitted_event.clone()),
        reference_mapping: RuntimeReferenceAppMapping {
            mapping_id: format!(
                "reference-mapping:{}:{}",
                route.pack_id.as_deref().unwrap_or(mapping.pack_id),
                route.action_contract_ref
            ),
            pack_id: route
                .pack_id
                .clone()
                .unwrap_or_else(|| mapping.pack_id.to_string()),
            pack_command: route.pack_command.clone(),
            action_contract_ref: route.action_contract_ref.clone(),
            app_action_type: proposal.action_type.clone(),
            app_target_object_type,
            source_refs: reference_mapping_source_refs(route, mapping),
        },
    })
}

pub fn validate_core_action_proposal_materialization(
    request: &RuntimeCommandRequest,
    proposal: &ActionProposal,
) -> Vec<RuntimeCommandError> {
    match materialize_core_action_proposal(request, proposal) {
        Ok(_) => Vec::new(),
        Err(error) => vec![RuntimeCommandError::new(
            RuntimeCommandErrorCode::MappingFailed,
            error.to_string(),
            Some("route.actionContractRef"),
        )],
    }
}

pub fn runtime_query_hint_for_command(request: &RuntimeCommandRequest) -> RuntimeQueryHint {
    let action_type = resolve_core_action_type(request).unwrap_or(request.command_type.as_str());
    let view = match action_type {
        "submitRequirement" => "RequirementIntakeView",
        "approveSpec" => "SpecPreviewView",
        "createProject" | "createIssue" => "ProjectHomeView",
        "activateIssue" | "claimIssue" | "startRun" | "writePatch" | "runValidation"
        | "submitEvidence" => "TaskWorkbenchView",
        "prepareDelivery" | "submitArtifact" | "markIssueDone" => "DeliveryPackageView",
        "requestAudit" | "createFinding" | "linkFixIssue" => "AuditSurfaceView",
        "recordDecision" => "RuntimeHealthView",
        _ => "ProjectHomeView",
    };
    RuntimeQueryHint {
        view: view.to_string(),
        target_id: request
            .target_object_ref
            .as_ref()
            .map(|target| target.id.clone()),
        reason: format!("refresh `{view}` after `{}`", request.command_type),
    }
}

pub fn unsupported_command_error(request: &RuntimeCommandRequest) -> RuntimeCommandError {
    RuntimeCommandError::new(
        RuntimeCommandErrorCode::UnsupportedCommand,
        format!(
            "runtime command `{}` does not map to a supported Core route action contract",
            request.command_type
        ),
        Some("commandType"),
    )
}

pub fn missing_field_error(field: &str, message: impl Into<String>) -> RuntimeCommandError {
    RuntimeCommandError::new(
        RuntimeCommandErrorCode::MissingField,
        message.into(),
        Some(field.to_string()),
    )
}

pub fn target_ref(object_type: impl Into<String>, id: impl Into<String>) -> ActionRef {
    ActionRef {
        object_type: object_type.into(),
        id: id.into(),
    }
}

pub fn core_runtime_route(
    route_id: impl Into<String>,
    action_contract_ref: impl Into<String>,
    target_object_type: Option<impl Into<String>>,
) -> RuntimeCommandRoute {
    RuntimeCommandRoute {
        route_id: route_id.into(),
        action_contract_ref: action_contract_ref.into(),
        target_object_type: target_object_type.map(Into::into),
        pack_id: None,
        pack_command: None,
    }
}

pub fn pack_runtime_route(
    pack_id: impl Into<String>,
    pack_command: impl Into<String>,
    action_contract_ref: impl Into<String>,
    target_object_type: impl Into<String>,
) -> RuntimeCommandRoute {
    let pack_id = pack_id.into();
    let pack_command = pack_command.into();
    RuntimeCommandRoute {
        route_id: format!("pack:{pack_id}:{pack_command}"),
        action_contract_ref: action_contract_ref.into(),
        target_object_type: Some(target_object_type.into()),
        pack_id: Some(pack_id),
        pack_command: Some(pack_command),
    }
}

pub fn action_type_for_action_contract_ref(action_contract_ref: &str) -> Option<&'static str> {
    reference_mapping_for_action_contract_ref(action_contract_ref)
        .map(|mapping| mapping.app_action_type)
}

pub fn action_contract_ref_for_action_type(action_type: &str) -> Option<&'static str> {
    reference_mapping_for_action_type(action_type).map(|mapping| mapping.action_contract_ref)
}

pub fn core_action_type_for_runtime_action(action_type: &str) -> Option<&'static str> {
    reference_mapping_for_action_type(action_type).map(|mapping| mapping.core_action_type)
}

fn resolve_core_action_type(request: &RuntimeCommandRequest) -> Option<&'static str> {
    request
        .route
        .as_ref()
        .and_then(|route| action_type_for_action_contract_ref(&route.action_contract_ref))
}

fn reference_mapping_source_refs(
    route: &RuntimeCommandRoute,
    mapping: &RuntimeReferenceAppMappingDefinition,
) -> Vec<String> {
    let mut refs = vec![
        route.action_contract_ref.clone(),
        mapping.source_ref.to_string(),
        format!("mapping-catalog:{}", mapping.pack_id),
    ];
    if let Some(pack_id) = &route.pack_id {
        refs.push(format!("pack:{pack_id}"));
    }
    if let Some(pack_command) = &route.pack_command {
        refs.push(format!("pack-command:{pack_command}"));
    }
    refs
}

fn reference_target_object_type_for_action(action_type: &str) -> Option<&'static str> {
    reference_mapping_for_action_type(action_type)
        .map(|mapping| mapping.reference_target_object_type)
}

fn reference_mapping_for_action_contract_ref(
    action_contract_ref: &str,
) -> Option<&'static RuntimeReferenceAppMappingDefinition> {
    let action_contract_ref = action_contract_ref.trim();
    software_dev_reference_mapping_catalog()
        .iter()
        .find(|mapping| mapping.action_contract_ref == action_contract_ref)
}

fn reference_mapping_for_action_type(
    action_type: &str,
) -> Option<&'static RuntimeReferenceAppMappingDefinition> {
    let action_type = action_type.trim();
    software_dev_reference_mapping_catalog()
        .iter()
        .find(|mapping| mapping.app_action_type == action_type)
}

pub fn source_surface_label(surface: &ActionSourceSurface) -> &'static str {
    match surface {
        ActionSourceSurface::Conversation => "conversation",
        ActionSourceSurface::Desktop => "desktop",
        ActionSourceSurface::Cli => "cli",
        ActionSourceSurface::Sdk => "sdk",
        ActionSourceSurface::Agent => "agent",
        ActionSourceSurface::System => "system",
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn software_dev_reference_mapping_catalog_declares_reference_boundary() {
        let catalog = software_dev_reference_mapping_catalog();
        assert!(catalog.len() >= 20);
        assert!(catalog.iter().all(|mapping| {
            mapping.pack_id == SOFTWARE_DEV_REFERENCE_PACK_ID
                && mapping.source_ref == SOFTWARE_DEV_REFERENCE_MAPPING_SOURCE
        }));
        assert!(catalog
            .iter()
            .all(|mapping| !mapping.core_action_type.contains("Requirement")
                && !mapping.core_action_type.contains("Issue")
                && !mapping.core_action_type.contains("Release")));
    }

    #[test]
    fn action_contract_refs_are_unique_catalog_entries() {
        let mut refs = HashSet::new();
        for mapping in software_dev_reference_mapping_catalog() {
            assert!(
                refs.insert(mapping.action_contract_ref),
                "duplicate action contract ref: {}",
                mapping.action_contract_ref
            );
        }
    }

    #[test]
    fn reference_mapping_helpers_resolve_from_catalog() {
        for mapping in software_dev_reference_mapping_catalog() {
            assert_eq!(
                action_type_for_action_contract_ref(mapping.action_contract_ref),
                Some(mapping.app_action_type)
            );
            assert_eq!(
                core_action_type_for_runtime_action(mapping.app_action_type),
                Some(mapping.core_action_type)
            );
            assert_eq!(
                reference_target_object_type_for_action(mapping.app_action_type),
                Some(mapping.reference_target_object_type)
            );
        }
        assert_eq!(
            action_contract_ref_for_action_type("submitRequirement"),
            Some("action-contract:requirement.submit")
        );
        assert_eq!(
            action_contract_ref_for_action_type("prepareDelivery"),
            Some("action-contract:delivery.prepare")
        );
    }
}
