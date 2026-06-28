use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use agentflow_action_contract::{core_action_contract_registry, ActionSourceSurface};
use agentflow_action_contract::{ActionProposal, ActionRef};
use agentflow_ontology::software_dev_reference_ontology_registry;

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

fn resolve_core_action_type(request: &RuntimeCommandRequest) -> Option<&'static str> {
    request
        .route
        .as_ref()
        .and_then(|route| action_type_for_action_contract_ref(&route.action_contract_ref))
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
