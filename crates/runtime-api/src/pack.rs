use crate::commands::{
    execute_command_via_arbitration, validate_runtime_command, RuntimeCommandRequest,
};
use crate::errors::{RuntimeCommandError, RuntimeCommandErrorCode};
use crate::responses::{RuntimeCommandResponse, RuntimeCommandValidationReport};
use agentflow_action_contract::{ActionRef, ActionSourceSurface};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::BTreeSet, path::Path};

pub type PackRegistryView = agentflow_pack::PackRegistry;
pub type PackValidationArtifactView = agentflow_pack::PackValidationArtifact;

pub const PACK_COMMAND_SURFACE_VERSION: &str = "agentflow-pack-command-surface.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackRegistryReadReceipt {
    pub writes_authority: bool,
    pub entry_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackValidationArtifactReadReceipt {
    pub writes_authority: bool,
    pub active: bool,
    pub issue_count: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackCommandRequest {
    pub pack_id: String,
    pub command_id: String,
    pub command: String,
    pub actor_role: String,
    pub source_surface: ActionSourceSurface,
    pub target_object_type: String,
    pub target_object_id: String,
    #[serde(default)]
    pub input: Value,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
    #[serde(default)]
    pub artifact_refs: Vec<String>,
    pub idempotency_key: String,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackCommandEntryView {
    pub pack_id: String,
    pub command: String,
    pub label: String,
    pub page_id: String,
    pub route: String,
    pub action_contract_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackCommandListView {
    pub version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pack_id: Option<String>,
    pub writes_authority: bool,
    pub commands: Vec<PackCommandEntryView>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackSurfaceRouteView {
    pub version: String,
    pub pack_id: String,
    pub command: String,
    pub page_id: String,
    pub route: String,
    pub action_contract_ref: String,
    pub runtime_command_type: String,
    pub target_object_type: String,
    pub source_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackCapabilityStatusView {
    pub version: String,
    pub pack_id: String,
    pub command: String,
    pub required_capabilities: Vec<String>,
    pub provider_ids: Vec<String>,
    pub command_boundary: String,
    pub available: bool,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackCommandValidationReport {
    pub version: String,
    pub pack_id: String,
    pub command_id: String,
    pub command: String,
    pub valid: bool,
    pub runtime_command: Option<RuntimeCommandRequest>,
    pub runtime_validation: Option<RuntimeCommandValidationReport>,
    pub surface_route: Option<PackSurfaceRouteView>,
    pub capability_status: Option<PackCapabilityStatusView>,
    pub rejected_reasons: Vec<RuntimeCommandError>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackCommandDryRunReport {
    pub version: String,
    pub pack_id: String,
    pub command_id: String,
    pub command: String,
    pub valid: bool,
    pub writes_authority: bool,
    pub writes_event_store: bool,
    pub executes_provider: bool,
    pub runtime_command: Option<RuntimeCommandRequest>,
    pub would_submit_to_arbitration: bool,
    pub expected_events: Vec<String>,
    pub affected_projections: Vec<String>,
    pub rejected_reasons: Vec<RuntimeCommandError>,
}

pub fn get_pack_registry(project_root: impl AsRef<Path>) -> Result<PackRegistryView> {
    agentflow_pack::load_pack_registry(project_root)
}

pub fn get_pack_validation_artifact(
    artifact_path: impl AsRef<Path>,
) -> Result<PackValidationArtifactView> {
    agentflow_pack::load_pack_validation_artifact(artifact_path)
}

pub fn list_pack_commands(
    project_root: impl AsRef<Path>,
    pack_id: Option<&str>,
) -> Result<PackCommandListView> {
    let _ = agentflow_pack::load_pack_registry(project_root)?;
    let mut commands = Vec::new();
    for bundle in built_in_pack_bundles() {
        if pack_id.is_some_and(|requested| requested != bundle.pack_id) {
            continue;
        }
        commands.extend(bundle.surface.command_entry_mappings.iter().map(|mapping| {
            PackCommandEntryView {
                pack_id: bundle.pack_id.to_string(),
                command: mapping.command_type.clone(),
                label: mapping.label.clone(),
                page_id: mapping.page_id.clone(),
                route: format!("{:?}", mapping.route),
                action_contract_ref: mapping.action_contract_ref.clone(),
            }
        }));
    }
    commands.sort_by(|left, right| {
        left.pack_id
            .cmp(&right.pack_id)
            .then_with(|| left.command.cmp(&right.command))
    });
    Ok(PackCommandListView {
        version: PACK_COMMAND_SURFACE_VERSION.to_string(),
        pack_id: pack_id.map(str::to_string),
        writes_authority: false,
        commands,
    })
}

pub fn query_pack_surface_route(
    project_root: impl AsRef<Path>,
    pack_id: &str,
    command: &str,
) -> Result<PackSurfaceRouteView> {
    let _ = agentflow_pack::load_pack_registry(project_root)?;
    resolve_pack_command(pack_id, command)
        .map(|resolved| resolved.route)
        .map_err(anyhow::Error::msg)
}

pub fn query_pack_capability_status(
    project_root: impl AsRef<Path>,
    pack_id: &str,
    command: &str,
) -> Result<PackCapabilityStatusView> {
    let _ = agentflow_pack::load_pack_registry(project_root)?;
    resolve_pack_command(pack_id, command)
        .map(|resolved| resolved.capability)
        .map_err(anyhow::Error::msg)
}

pub fn validate_pack_command(
    project_root: impl AsRef<Path>,
    request: &PackCommandRequest,
) -> Result<PackCommandValidationReport> {
    let _ = agentflow_pack::load_pack_registry(project_root)?;
    let mut rejected_reasons = required_request_errors(request);
    let resolved = if rejected_reasons.is_empty() {
        match resolve_pack_command(&request.pack_id, &request.command) {
            Ok(resolved) => Some(resolved),
            Err(message) => {
                rejected_reasons.push(pack_command_error(
                    RuntimeCommandErrorCode::UnsupportedCommand,
                    message,
                    Some("command"),
                ));
                None
            }
        }
    } else {
        None
    };

    let runtime_command = resolved
        .as_ref()
        .map(|resolved| runtime_command_from_pack_request(request, resolved));
    let runtime_validation = runtime_command.as_ref().map(validate_runtime_command);
    if let Some(validation) = runtime_validation.as_ref() {
        rejected_reasons.extend(validation.errors.clone());
    }

    Ok(PackCommandValidationReport {
        version: PACK_COMMAND_SURFACE_VERSION.to_string(),
        pack_id: request.pack_id.clone(),
        command_id: request.command_id.clone(),
        command: request.command.clone(),
        valid: rejected_reasons.is_empty(),
        runtime_command,
        runtime_validation,
        surface_route: resolved.as_ref().map(|resolved| resolved.route.clone()),
        capability_status: resolved
            .as_ref()
            .map(|resolved| resolved.capability.clone()),
        rejected_reasons,
    })
}

pub fn dry_run_pack_command(
    project_root: impl AsRef<Path>,
    request: &PackCommandRequest,
) -> Result<PackCommandDryRunReport> {
    let validation = validate_pack_command(project_root, request)?;
    let expected_events = validation
        .runtime_validation
        .as_ref()
        .and_then(|report| report.normalized_action_type.as_ref())
        .map(|action| vec![format!("accepted-action.{action}")])
        .unwrap_or_default();
    let affected_projections = validation
        .surface_route
        .as_ref()
        .map(|route| vec![format!("projection.refresh:{}", route.target_object_type)])
        .unwrap_or_default();

    Ok(PackCommandDryRunReport {
        version: PACK_COMMAND_SURFACE_VERSION.to_string(),
        pack_id: request.pack_id.clone(),
        command_id: request.command_id.clone(),
        command: request.command.clone(),
        valid: validation.valid,
        writes_authority: false,
        writes_event_store: false,
        executes_provider: false,
        runtime_command: validation.runtime_command,
        would_submit_to_arbitration: validation.valid,
        expected_events,
        affected_projections,
        rejected_reasons: validation.rejected_reasons,
    })
}

pub fn submit_pack_action_proposal(
    project_root: impl AsRef<Path>,
    request: &PackCommandRequest,
) -> Result<RuntimeCommandResponse> {
    let root = project_root.as_ref();
    let validation = validate_pack_command(root, request)?;
    if let Some(runtime_command) = validation.runtime_command.as_ref() {
        execute_command_via_arbitration(root, runtime_command)
    } else {
        let runtime_command = RuntimeCommandRequest {
            command_id: request.command_id.clone(),
            command_type: request.command.clone(),
            source_surface: request.source_surface.clone(),
            actor_role: request.actor_role.clone(),
            target_object_ref: None,
            input: request.input.clone(),
            evidence_refs: request.evidence_refs.clone(),
            artifact_refs: request.artifact_refs.clone(),
            idempotency_key: request.idempotency_key.clone(),
            created_at: request.created_at.clone(),
        };
        execute_command_via_arbitration(root, &runtime_command)
    }
}

pub fn pack_registry_read_receipt(registry: &PackRegistryView) -> PackRegistryReadReceipt {
    PackRegistryReadReceipt {
        writes_authority: registry.writes_authority,
        entry_count: registry.entries.len(),
    }
}

pub fn pack_validation_artifact_read_receipt(
    artifact: &PackValidationArtifactView,
) -> PackValidationArtifactReadReceipt {
    PackValidationArtifactReadReceipt {
        writes_authority: artifact.writes_authority,
        active: artifact.active,
        issue_count: artifact.issues.len(),
    }
}

#[derive(Debug, Clone)]
struct BuiltInPackBundle {
    pack_id: &'static str,
    surface: agentflow_pack::PackSurfaceDefinition,
    connector: agentflow_pack::PackConnectorDefinition,
}

#[derive(Debug, Clone)]
struct ResolvedPackCommand {
    route: PackSurfaceRouteView,
    capability: PackCapabilityStatusView,
}

fn built_in_pack_bundles() -> Vec<BuiltInPackBundle> {
    vec![
        BuiltInPackBundle {
            pack_id: "software-dev",
            surface: agentflow_pack::software_dev_surface_definition(),
            connector: agentflow_pack::software_dev_connector_definition(),
        },
        BuiltInPackBundle {
            pack_id: "ui-design",
            surface: agentflow_pack::ui_design_surface_definition(),
            connector: agentflow_pack::ui_design_connector_definition(),
        },
    ]
}

fn resolve_pack_command(pack_id: &str, command: &str) -> Result<ResolvedPackCommand, String> {
    let bundle = built_in_pack_bundles()
        .into_iter()
        .find(|bundle| bundle.pack_id == pack_id)
        .ok_or_else(|| {
            format!("pack `{pack_id}` is not registered in the Runtime API command surface")
        })?;
    let mapping = bundle
        .surface
        .command_entry_mappings
        .iter()
        .find(|mapping| mapping.command_type == command)
        .ok_or_else(|| format!("pack command `{command}` is not exposed by pack `{pack_id}`"))?;
    let runtime_command_type = runtime_command_type_for_action_contract(
        &mapping.action_contract_ref,
    )
    .ok_or_else(|| {
        format!(
            "pack command `{command}` uses unsupported action contract `{}`",
            mapping.action_contract_ref
        )
    })?;
    let target_object_type = target_object_type_for_runtime_command(runtime_command_type)
        .ok_or_else(|| {
            format!(
                "pack command `{command}` maps to unsupported runtime command `{runtime_command_type}`"
            )
        })?;
    let provider_actions = bundle
        .connector
        .connectors
        .iter()
        .flat_map(|connector| {
            connector
                .supported_actions
                .iter()
                .filter(move |action| action.command_type == command)
                .map(move |action| {
                    (
                        connector.connector_id.clone(),
                        action.required_capability.clone(),
                    )
                })
        })
        .collect::<Vec<_>>();
    let required_capabilities = provider_actions
        .iter()
        .map(|(_, capability)| capability.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let provider_ids = provider_actions
        .iter()
        .map(|(provider, _)| provider.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    Ok(ResolvedPackCommand {
        route: PackSurfaceRouteView {
            version: PACK_COMMAND_SURFACE_VERSION.to_string(),
            pack_id: pack_id.to_string(),
            command: command.to_string(),
            page_id: mapping.page_id.clone(),
            route: format!("{:?}", mapping.route),
            action_contract_ref: mapping.action_contract_ref.clone(),
            runtime_command_type: runtime_command_type.to_string(),
            target_object_type: target_object_type.to_string(),
            source_refs: vec![
                format!("pack:{pack_id}/surface"),
                format!("pack:{pack_id}/connectors"),
            ],
        },
        capability: PackCapabilityStatusView {
            version: PACK_COMMAND_SURFACE_VERSION.to_string(),
            pack_id: pack_id.to_string(),
            command: command.to_string(),
            required_capabilities,
            provider_ids,
            command_boundary: "runtime-api/action-contract/arbitration".to_string(),
            available: true,
            reason: "pack command can enter Runtime API; provider execution still requires runtime preflight".to_string(),
        },
    })
}

fn runtime_command_type_for_action_contract(action_contract_ref: &str) -> Option<&'static str> {
    match action_contract_ref {
        "action-contract:spec.intake" => Some("submitRequirement"),
        "action-contract:issue.start" => Some("startRun"),
        "action-contract:acceptance.evaluate" => Some("runValidation"),
        "action-contract:delivery.open" => Some("prepareDelivery"),
        "action-contract:audit.request" => Some("requestAudit"),
        _ => None,
    }
}

fn target_object_type_for_runtime_command(runtime_command_type: &str) -> Option<&'static str> {
    match runtime_command_type {
        "submitRequirement" => Some("Requirement"),
        "startRun" | "requestAudit" => Some("Issue"),
        "runValidation" | "prepareDelivery" => Some("Run"),
        _ => None,
    }
}

fn runtime_command_from_pack_request(
    request: &PackCommandRequest,
    resolved: &ResolvedPackCommand,
) -> RuntimeCommandRequest {
    RuntimeCommandRequest {
        command_id: request.command_id.clone(),
        command_type: resolved.route.runtime_command_type.clone(),
        source_surface: request.source_surface.clone(),
        actor_role: request.actor_role.clone(),
        target_object_ref: Some(ActionRef {
            object_type: request.target_object_type.clone(),
            id: request.target_object_id.clone(),
        }),
        input: request.input.clone(),
        evidence_refs: request.evidence_refs.clone(),
        artifact_refs: request.artifact_refs.clone(),
        idempotency_key: request.idempotency_key.clone(),
        created_at: request.created_at.clone(),
    }
}

fn required_request_errors(request: &PackCommandRequest) -> Vec<RuntimeCommandError> {
    let mut errors = Vec::new();
    require_non_empty(&mut errors, "packId", &request.pack_id);
    require_non_empty(&mut errors, "commandId", &request.command_id);
    require_non_empty(&mut errors, "command", &request.command);
    require_non_empty(&mut errors, "actorRole", &request.actor_role);
    require_non_empty(&mut errors, "targetObjectType", &request.target_object_type);
    require_non_empty(&mut errors, "targetObjectId", &request.target_object_id);
    require_non_empty(&mut errors, "idempotencyKey", &request.idempotency_key);
    require_non_empty(&mut errors, "createdAt", &request.created_at);
    errors
}

fn require_non_empty(errors: &mut Vec<RuntimeCommandError>, field: &str, value: &str) {
    if value.trim().is_empty() {
        errors.push(pack_command_error(
            RuntimeCommandErrorCode::MissingField,
            format!("pack command requires {field}"),
            Some(field),
        ));
    }
}

fn pack_command_error(
    code: RuntimeCommandErrorCode,
    message: impl Into<String>,
    path: Option<impl Into<String>>,
) -> RuntimeCommandError {
    RuntimeCommandError::new(code, message, path)
}

#[cfg(test)]
mod tests {
    use super::{
        dry_run_pack_command, get_pack_registry, get_pack_validation_artifact, list_pack_commands,
        pack_registry_read_receipt, pack_validation_artifact_read_receipt,
        query_pack_capability_status, query_pack_surface_route, submit_pack_action_proposal,
        validate_pack_command, PackCommandRequest,
    };
    use crate::responses::RuntimeCommandStatus;
    use agentflow_action_contract::ActionSourceSurface;
    use serde_json::json;

    #[test]
    fn runtime_reads_pack_registry_without_authority_write() {
        let dir = tempfile::tempdir().unwrap();

        let registry = get_pack_registry(dir.path()).unwrap();
        let receipt = pack_registry_read_receipt(&registry);

        assert!(!receipt.writes_authority);
        assert_eq!(receipt.entry_count, 0);
    }

    #[test]
    fn runtime_reads_pack_validation_artifact_without_authority_write() {
        let dir = tempfile::tempdir().unwrap();
        let artifact_path = dir.path().join("validation.json");
        let manifest = agentflow_pack::PackManifest {
            version: agentflow_pack::PACK_MANIFEST_VERSION.to_string(),
            pack_id: "software-dev".to_string(),
            name: "Software Dev".to_string(),
            pack_type: agentflow_pack::PackType::SoftwareDev,
            pack_version: "0.8.0".to_string(),
            runtime_compatibility: ">=0.8.0".to_string(),
            domain_path: "domain/".to_string(),
            surface_path: "surface/".to_string(),
            connector_path: "connectors/".to_string(),
            required_capabilities: vec!["provider.codex.launch".to_string()],
            owned_object_types: vec!["Issue".to_string()],
            exposed_commands: vec!["work.issue.start".to_string()],
            projection_entries: vec![
                "projection.project-home".to_string(),
                "projection.spec-workbench".to_string(),
                "projection.task-workbench".to_string(),
                "projection.acceptance".to_string(),
                "projection.delivery".to_string(),
                "projection.event-timeline".to_string(),
                "projection.evidence-graph".to_string(),
                "projection.audit-surface".to_string(),
            ],
            migration_policy: agentflow_pack::PackMigrationPolicy::PreviewOnly,
            validation_status: agentflow_pack::PackValidationStatus::Draft,
        };
        let api_entries = vec![
            "spec.intake.start".to_string(),
            "work.issue.start".to_string(),
            "acceptance.evaluate".to_string(),
            "delivery.open".to_string(),
            "audit.request.sidecar".to_string(),
            "github.repo.read".to_string(),
            "github.pull-request.create".to_string(),
            "git.status".to_string(),
            "git.diff".to_string(),
            "work-agent.launch".to_string(),
            "work-agent.complete".to_string(),
            "browser-preview.smoke".to_string(),
        ];
        let generated = agentflow_pack::validate_pack_bundle(
            &manifest,
            &agentflow_pack::software_dev_domain_definition(),
            &agentflow_pack::software_dev_surface_definition(),
            &agentflow_pack::software_dev_connector_definition(),
            &api_entries,
            "0.8.0",
        );
        std::fs::write(
            &artifact_path,
            serde_json::to_string_pretty(&generated).unwrap(),
        )
        .unwrap();

        let artifact = get_pack_validation_artifact(&artifact_path).unwrap();
        let receipt = pack_validation_artifact_read_receipt(&artifact);

        assert!(receipt.active);
        assert!(!receipt.writes_authority);
        assert_eq!(receipt.issue_count, 0);
    }

    #[test]
    fn runtime_lists_built_in_pack_commands_without_authority_write() {
        let dir = tempfile::tempdir().unwrap();

        let view = list_pack_commands(dir.path(), None).unwrap();

        assert!(!view.writes_authority);
        assert!(view
            .commands
            .iter()
            .any(|command| command.pack_id == "software-dev"
                && command.command == "work.issue.start"));
        assert!(view
            .commands
            .iter()
            .any(|command| command.pack_id == "ui-design"
                && command.command == "design.wireframe.generate"));
    }

    #[test]
    fn runtime_resolves_pack_route_and_capability_status() {
        let dir = tempfile::tempdir().unwrap();

        let route =
            query_pack_surface_route(dir.path(), "software-dev", "work.issue.start").unwrap();
        let status =
            query_pack_capability_status(dir.path(), "software-dev", "work.issue.start").unwrap();

        assert_eq!(route.runtime_command_type, "startRun");
        assert_eq!(route.target_object_type, "Issue");
        assert!(route
            .source_refs
            .iter()
            .any(|item| item.contains("surface")));
        assert!(status.available);
        assert_eq!(
            status.command_boundary,
            "runtime-api/action-contract/arbitration"
        );
    }

    #[test]
    fn runtime_validates_pack_command_through_action_contract() {
        let dir = tempfile::tempdir().unwrap();
        let request = software_issue_start_request();

        let report = validate_pack_command(dir.path(), &request).unwrap();

        assert!(report.valid);
        let runtime_command = report.runtime_command.unwrap();
        assert_eq!(runtime_command.command_type, "startRun");
        assert_eq!(runtime_command.actor_role, "work-agent");
        assert_eq!(
            runtime_command.target_object_ref.unwrap().object_type,
            "Issue"
        );
        assert_eq!(
            report.runtime_validation.unwrap().normalized_action_type,
            Some("startRun".to_string())
        );
    }

    #[test]
    fn runtime_rejects_invalid_pack_command_with_readable_reason() {
        let dir = tempfile::tempdir().unwrap();
        let mut request = software_issue_start_request();
        request.command = "work.issue.teleport".to_string();

        let report = validate_pack_command(dir.path(), &request).unwrap();

        assert!(!report.valid);
        assert!(report
            .rejected_reasons
            .iter()
            .any(|reason| reason.message.contains("work.issue.teleport")));
    }

    #[test]
    fn runtime_dry_run_pack_command_does_not_write_or_execute() {
        let dir = tempfile::tempdir().unwrap();
        let request = software_issue_start_request();

        let report = dry_run_pack_command(dir.path(), &request).unwrap();

        assert!(report.valid);
        assert!(!report.writes_authority);
        assert!(!report.writes_event_store);
        assert!(!report.executes_provider);
        assert!(report.would_submit_to_arbitration);
        assert!(dir.path().join(".agentflow").exists() == false);
    }

    #[test]
    fn runtime_submit_pack_action_uses_arbitration_entrypoint() {
        let dir = tempfile::tempdir().unwrap();
        let request = software_issue_start_request();

        let response = submit_pack_action_proposal(dir.path(), &request).unwrap();

        assert_ne!(response.status, RuntimeCommandStatus::InvalidCommand);
        assert!(dir.path().join(".agentflow/runtime/commands").exists());
    }

    fn software_issue_start_request() -> PackCommandRequest {
        PackCommandRequest {
            pack_id: "software-dev".to_string(),
            command_id: "pack-command-001".to_string(),
            command: "work.issue.start".to_string(),
            actor_role: "work-agent".to_string(),
            source_surface: ActionSourceSurface::Desktop,
            target_object_type: "Issue".to_string(),
            target_object_id: "AF-001".to_string(),
            input: json!({"reason": "start issue from pack command surface"}),
            evidence_refs: Vec::new(),
            artifact_refs: Vec::new(),
            idempotency_key: "pack-command-001".to_string(),
            created_at: "2026-06-23T00:00:00Z".to_string(),
        }
    }
}
