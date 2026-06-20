use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use agentflow_action_contract::{core_action_contract_registry, ActionSourceSurface};
use agentflow_action_contract::{ActionProposal, ActionRef};
use agentflow_ontology::core_ontology_registry;

use crate::commands::RuntimeCommandRequest;
use crate::errors::{RuntimeCommandError, RuntimeCommandErrorCode};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeQueryHint {
    pub view: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_id: Option<String>,
    pub reason: String,
}

pub fn map_command_to_action_proposal(request: &RuntimeCommandRequest) -> Result<ActionProposal> {
    let ontology = core_ontology_registry();
    let contracts = core_action_contract_registry(&ontology);
    let action_type = canonical_action_type(&request.command_type).ok_or_else(|| {
        anyhow!(
            "unsupported runtime command type `{}`",
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
    let view = match canonical_action_type(&request.command_type)
        .unwrap_or(request.command_type.as_str())
    {
        "submitRequirement" => "RequirementIntakeView",
        "approveSpec" => "SpecPreviewView",
        "createProject" | "createIssue" => "ProjectHomeView",
        "activateIssue" | "startRun" | "submitEvidence" => "TaskWorkbenchView",
        "submitArtifact" | "markIssueDone" => "DeliveryPackageView",
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
            "runtime command `{}` does not map to a supported action contract",
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

fn canonical_action_type(command_type: &str) -> Option<&'static str> {
    match command_type.trim() {
        "submitRequirement" => Some("submitRequirement"),
        "approveSpec" => Some("approveSpec"),
        "createProject" => Some("createProject"),
        "createIssue" => Some("createIssue"),
        "activateIssue" => Some("activateIssue"),
        "startRun" => Some("startRun"),
        "submitEvidence" => Some("submitEvidence"),
        "submitArtifact" | "submitDelivery" => Some("submitArtifact"),
        "markIssueDone" => Some("markIssueDone"),
        "recordDecision" => Some("recordDecision"),
        "requestAudit" => Some("requestAudit"),
        "createFinding" => Some("createFinding"),
        "linkFixIssue" => Some("linkFixIssue"),
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
