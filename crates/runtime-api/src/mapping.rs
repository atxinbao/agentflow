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

    let core_action_type =
        core_action_type_for_runtime_action(&proposal.action_type).ok_or_else(|| {
            anyhow!(
                "missing Core action mapping for reference action `{}`",
                proposal.action_type
            )
        })?;
    let core_action = registry
        .core_action_state_semantics
        .actions
        .iter()
        .find(|action| action.action_type == core_action_type)
        .ok_or_else(|| anyhow!("unknown Core action `{core_action_type}`"))?;
    if !registry
        .core_object_link_schema
        .object_schemas
        .iter()
        .any(|object| object.object_type == core_action.target_object_type)
    {
        anyhow::bail!(
            "unknown Core target object `{}` for action `{core_action_type}`",
            core_action.target_object_type
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
                route.pack_id.as_deref().unwrap_or("software-dev-reference"),
                route.action_contract_ref
            ),
            pack_id: route
                .pack_id
                .clone()
                .unwrap_or_else(|| "software-dev-reference".to_string()),
            pack_command: route.pack_command.clone(),
            action_contract_ref: route.action_contract_ref.clone(),
            app_action_type: proposal.action_type.clone(),
            app_target_object_type,
            source_refs: reference_mapping_source_refs(route),
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
    match action_contract_ref.trim() {
        "action-contract:requirement.submit" | "action-contract:spec.intake" => {
            Some("submitRequirement")
        }
        "action-contract:requirement.normalize" => Some("normalizeRequirement"),
        "action-contract:requirement.classify" => Some("classifyRequirement"),
        "action-contract:spec.draft" => Some("draftSpec"),
        "action-contract:spec.approve" => Some("approveSpec"),
        "action-contract:project.create" => Some("createProject"),
        "action-contract:issue.create" => Some("createIssue"),
        "action-contract:issue.activate" => Some("activateIssue"),
        "action-contract:issue.claim" => Some("claimIssue"),
        "action-contract:issue.start" => Some("startRun"),
        "action-contract:run.patch.write" => Some("writePatch"),
        "action-contract:acceptance.evaluate" => Some("runValidation"),
        "action-contract:delivery.prepare" | "action-contract:delivery.open" => {
            Some("prepareDelivery")
        }
        "action-contract:evidence.submit" => Some("submitEvidence"),
        "action-contract:artifact.submit" => Some("submitArtifact"),
        "action-contract:issue.done" => Some("markIssueDone"),
        "action-contract:decision.record" => Some("recordDecision"),
        "action-contract:audit.request" => Some("requestAudit"),
        "action-contract:finding.create" => Some("createFinding"),
        "action-contract:finding.link-fix-issue" => Some("linkFixIssue"),
        _ => None,
    }
}

pub fn action_contract_ref_for_action_type(action_type: &str) -> Option<&'static str> {
    match action_type.trim() {
        "submitRequirement" => Some("action-contract:requirement.submit"),
        "normalizeRequirement" => Some("action-contract:requirement.normalize"),
        "classifyRequirement" => Some("action-contract:requirement.classify"),
        "draftSpec" => Some("action-contract:spec.draft"),
        "approveSpec" => Some("action-contract:spec.approve"),
        "createProject" => Some("action-contract:project.create"),
        "createIssue" => Some("action-contract:issue.create"),
        "activateIssue" => Some("action-contract:issue.activate"),
        "claimIssue" => Some("action-contract:issue.claim"),
        "startRun" => Some("action-contract:issue.start"),
        "writePatch" => Some("action-contract:run.patch.write"),
        "runValidation" => Some("action-contract:acceptance.evaluate"),
        "prepareDelivery" => Some("action-contract:delivery.prepare"),
        "submitEvidence" => Some("action-contract:evidence.submit"),
        "submitArtifact" => Some("action-contract:artifact.submit"),
        "markIssueDone" => Some("action-contract:issue.done"),
        "recordDecision" => Some("action-contract:decision.record"),
        "requestAudit" => Some("action-contract:audit.request"),
        "createFinding" => Some("action-contract:finding.create"),
        "linkFixIssue" => Some("action-contract:finding.link-fix-issue"),
        _ => None,
    }
}

pub fn core_action_type_for_runtime_action(action_type: &str) -> Option<&'static str> {
    match action_type.trim() {
        "submitRequirement" => Some("captureObject"),
        "normalizeRequirement" => Some("normalizeObject"),
        "classifyRequirement" => Some("routeObject"),
        "draftSpec" => Some("routeObject"),
        "approveSpec" => Some("acceptObject"),
        "createProject" | "createIssue" => Some("routeObject"),
        "activateIssue" => Some("acceptObject"),
        "claimIssue" | "startRun" => Some("startObject"),
        "writePatch" => Some("attachArtifact"),
        "runValidation" | "submitEvidence" => Some("attachEvidence"),
        "prepareDelivery" | "submitArtifact" => Some("attachArtifact"),
        "markIssueDone" => Some("completeObject"),
        "recordDecision" => Some("completeObject"),
        "requestAudit" => Some("submitForReview"),
        "createFinding" => Some("blockObject"),
        "linkFixIssue" => Some("supersedeObject"),
        _ => None,
    }
}

fn resolve_core_action_type(request: &RuntimeCommandRequest) -> Option<&'static str> {
    request
        .route
        .as_ref()
        .and_then(|route| action_type_for_action_contract_ref(&route.action_contract_ref))
}

fn reference_mapping_source_refs(route: &RuntimeCommandRoute) -> Vec<String> {
    let mut refs = vec![route.action_contract_ref.clone()];
    if let Some(pack_id) = &route.pack_id {
        refs.push(format!("pack:{pack_id}"));
    }
    if let Some(pack_command) = &route.pack_command {
        refs.push(format!("pack-command:{pack_command}"));
    }
    refs
}

fn reference_target_object_type_for_action(action_type: &str) -> Option<&'static str> {
    match action_type {
        "submitRequirement" | "normalizeRequirement" | "classifyRequirement" | "draftSpec" => {
            Some("Requirement")
        }
        "approveSpec" | "createProject" => Some("Spec"),
        "createIssue" => Some("Project"),
        "activateIssue" | "claimIssue" | "startRun" | "markIssueDone" | "requestAudit" => {
            Some("Issue")
        }
        "writePatch" | "runValidation" | "prepareDelivery" | "submitEvidence"
        | "submitArtifact" => Some("Run"),
        "recordDecision" => Some("Decision"),
        "createFinding" => Some("Audit"),
        "linkFixIssue" => Some("Finding"),
        _ => None,
    }
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
