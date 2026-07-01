use crate::commands::{
    execute_command_via_arbitration, validate_runtime_command, RuntimeCommandRequest,
};
use crate::errors::{RuntimeCommandError, RuntimeCommandErrorCode};
use crate::responses::{
    RuntimeCommandDecision, RuntimeCommandResponse, RuntimeCommandStatus,
    RuntimeCommandValidationReport, RUNTIME_COMMAND_API_VERSION,
};
use agentflow_action_contract::{ActionRef, ActionSourceSurface};
use agentflow_capability_registry::{
    evaluate_command, evaluate_pack_connector_commands, CapabilityRegistry,
    PackConnectorCommandDecision, WorkerHealth,
};
use anyhow::{Context, Result};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::BTreeSet,
    fmt, fs,
    path::{Path, PathBuf},
};

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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure_stage: Option<String>,
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
    let registry = agentflow_pack::load_pack_registry(project_root)?;
    let mut commands = Vec::new();
    for entry in registry.entries.iter() {
        if pack_id.is_some_and(|requested| requested != entry.pack_id) {
            continue;
        }
        let surface = load_pack_surface_definition(entry)?;
        commands.extend(surface.command_entry_mappings.iter().map(|mapping| {
            PackCommandEntryView {
                pack_id: entry.pack_id.clone(),
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
    resolve_pack_command(project_root.as_ref(), pack_id, command)
        .map(|resolved| resolved.route)
        .map_err(anyhow::Error::msg)
}

pub fn query_pack_capability_status(
    project_root: impl AsRef<Path>,
    pack_id: &str,
    command: &str,
) -> Result<PackCapabilityStatusView> {
    resolve_pack_command(project_root.as_ref(), pack_id, command)
        .map(|resolved| resolved.capability)
        .map_err(anyhow::Error::msg)
}

pub fn validate_pack_command(
    project_root: impl AsRef<Path>,
    request: &PackCommandRequest,
) -> Result<PackCommandValidationReport> {
    let project_root = project_root.as_ref();
    let mut rejected_reasons = required_request_errors(request);
    let mut failure_stage = if rejected_reasons.is_empty() {
        None
    } else {
        Some("schema".to_string())
    };
    if !rejected_reasons.is_empty() {
        rejected_reasons = rejected_reasons
            .into_iter()
            .map(|error| {
                RuntimeCommandError::new(
                    error.code,
                    pack_command_failure_message(request, failure_stage.as_deref(), error.message),
                    error.path,
                )
            })
            .collect();
    }
    let resolved = if rejected_reasons.is_empty() {
        match resolve_pack_command(project_root, &request.pack_id, &request.command) {
            Ok(resolved) => Some(resolved),
            Err(error) => {
                set_failure_stage(&mut failure_stage, error.stage);
                rejected_reasons.push(pack_command_error(
                    RuntimeCommandErrorCode::UnsupportedCommand,
                    pack_command_failure_message(request, failure_stage.as_deref(), error.message),
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
        if !validation.errors.is_empty() {
            set_failure_stage(&mut failure_stage, "surface-mapping");
        }
        rejected_reasons.extend(validation.errors.iter().cloned().map(|error| {
            RuntimeCommandError::new(
                error.code,
                pack_command_failure_message(request, failure_stage.as_deref(), error.message),
                error.path,
            )
        }));
    }
    if let Some(resolved) = resolved.as_ref() {
        if !resolved.capability.available {
            set_failure_stage(
                &mut failure_stage,
                if resolved
                    .capability
                    .reason
                    .contains("no connector capability mapping")
                {
                    "connector"
                } else {
                    "capability"
                },
            );
            rejected_reasons.push(pack_command_error(
                RuntimeCommandErrorCode::MappingFailed,
                pack_command_failure_message(
                    request,
                    failure_stage.as_deref(),
                    format!("command unavailable: {}", resolved.capability.reason),
                ),
                Some("capabilityStatus"),
            ));
        }
    }

    Ok(PackCommandValidationReport {
        version: PACK_COMMAND_SURFACE_VERSION.to_string(),
        pack_id: request.pack_id.clone(),
        command_id: request.command_id.clone(),
        command: request.command.clone(),
        valid: rejected_reasons.is_empty(),
        failure_stage,
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
    if !validation.valid {
        return Ok(invalid_pack_command_response(
            request,
            validation.rejected_reasons,
        ));
    }
    if let Some(runtime_command) = validation.runtime_command.as_ref() {
        execute_command_via_arbitration(root, runtime_command)
    } else {
        Ok(invalid_pack_command_response(
            request,
            vec![RuntimeCommandError::new(
                RuntimeCommandErrorCode::UnsupportedCommand,
                format!(
                    "pack command `{}` did not produce a Core runtime route",
                    request.command
                ),
                Some("command"),
            )],
        ))
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
struct FileBackedPackBundle {
    entry: agentflow_pack::PackRegistryEntry,
    domain: agentflow_pack::PackDomainDefinition,
    surface: agentflow_pack::PackSurfaceDefinition,
    connector: agentflow_pack::PackConnectorDefinition,
}

#[derive(Debug, Clone)]
struct ResolvedPackCommand {
    route: PackSurfaceRouteView,
    capability: PackCapabilityStatusView,
}

#[derive(Debug, Clone)]
struct PackCommandResolveError {
    stage: &'static str,
    message: String,
}

impl fmt::Display for PackCommandResolveError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.stage, self.message)
    }
}

fn pack_resolve_error(stage: &'static str, message: impl Into<String>) -> PackCommandResolveError {
    PackCommandResolveError {
        stage,
        message: message.into(),
    }
}

fn resolve_pack_command(
    project_root: &Path,
    pack_id: &str,
    command: &str,
) -> Result<ResolvedPackCommand, PackCommandResolveError> {
    if let Ok(resolved) = resolve_product_command(project_root, pack_id, command) {
        return Ok(resolved);
    }

    let registry = agentflow_pack::load_pack_registry(project_root).map_err(|error| {
        pack_resolve_error("read-model", format!("pack registry unreadable: {error}"))
    })?;
    let entry = registry.pack(pack_id).cloned().ok_or_else(|| {
        pack_resolve_error(
            "read-model",
            format!("pack `{pack_id}` is not registered in the Runtime API command surface"),
        )
    })?;
    let bundle = load_file_backed_pack_bundle(entry)
        .map_err(|message| pack_resolve_error("read-model", message))?;
    let mapping = bundle
        .surface
        .command_entry_mappings
        .iter()
        .find(|mapping| mapping.command_type == command)
        .ok_or_else(|| {
            pack_resolve_error(
                "surface-mapping",
                format!("pack command `{command}` is not exposed by pack `{pack_id}`"),
            )
        })?;
    let runtime_command_type = runtime_command_type_for_action_contract(
        &mapping.action_contract_ref,
    )
    .ok_or_else(|| {
        pack_resolve_error(
            "surface-mapping",
            format!(
                "pack command `{command}` uses unsupported action contract `{}`",
                mapping.action_contract_ref
            ),
        )
    })?;
    let action_type =
        crate::mapping::action_type_for_action_contract_ref(&mapping.action_contract_ref)
            .ok_or_else(|| {
                pack_resolve_error(
                    "surface-mapping",
                    format!(
                        "pack command `{command}` uses unsupported action contract `{}`",
                        mapping.action_contract_ref
                    ),
                )
            })?;
    let action_semantic = bundle
        .domain
        .action_semantics
        .iter()
        .find(|semantic| semantic.action_type == action_type)
        .ok_or_else(|| {
            pack_resolve_error(
                "surface-mapping",
                format!(
                    "pack command `{command}` maps to action `{action_type}` without domain action semantic"
                ),
            )
        })?;
    let capability_registry = load_project_capability_registry(project_root).map_err(|error| {
        pack_resolve_error(
            "capability",
            format!("capability registry unreadable: {error}"),
        )
    })?;
    let connector_decisions =
        evaluate_pack_connector_commands(&capability_registry, &bundle.connector)
            .into_iter()
            .filter(|decision| decision.command_type == command)
            .collect::<Vec<_>>();
    let capability = capability_status_from_decisions(pack_id, command, &connector_decisions);

    Ok(ResolvedPackCommand {
        route: PackSurfaceRouteView {
            version: PACK_COMMAND_SURFACE_VERSION.to_string(),
            pack_id: pack_id.to_string(),
            command: command.to_string(),
            page_id: mapping.page_id.clone(),
            route: format!("{:?}", mapping.route),
            action_contract_ref: mapping.action_contract_ref.clone(),
            runtime_command_type: runtime_command_type.to_string(),
            target_object_type: action_semantic.target_object_type.clone(),
            source_refs: vec![
                bundle.entry.manifest_path.clone(),
                definition_path_for_entry(&bundle.entry, &bundle.entry.domain_path)
                    .to_string_lossy()
                    .replace('\\', "/"),
                definition_path_for_entry(&bundle.entry, &bundle.entry.surface_path)
                    .to_string_lossy()
                    .replace('\\', "/"),
                definition_path_for_entry(&bundle.entry, &bundle.entry.connector_path)
                    .to_string_lossy()
                    .replace('\\', "/"),
            ],
        },
        capability,
    })
}

fn resolve_product_command(
    project_root: &Path,
    pack_id: &str,
    command: &str,
) -> Result<ResolvedPackCommand, PackCommandResolveError> {
    let registry = agentflow_pack::load_product_registry(project_root).map_err(|error| {
        pack_resolve_error(
            "read-model",
            format!("product registry unreadable: {error}"),
        )
    })?;
    let entry = registry.product(pack_id).cloned().ok_or_else(|| {
        pack_resolve_error(
            "read-model",
            format!("product `{pack_id}` is not registered in products/**"),
        )
    })?;
    if !entry.valid {
        return Err(pack_resolve_error(
            "read-model",
            format!("product `{pack_id}` is invalid: {:?}", entry.diagnostics),
        ));
    }
    let definition = agentflow_pack::load_product_definition_from_entry(&entry)
        .map_err(|error| pack_resolve_error("read-model", error.to_string()))?;
    if !definition.valid {
        return Err(pack_resolve_error(
            "read-model",
            format!(
                "product `{pack_id}` definition is invalid: {:?}",
                definition.diagnostics
            ),
        ));
    }
    let command_entry = definition
        .surface
        .commands
        .iter()
        .find(|entry| entry.id == command)
        .ok_or_else(|| {
            pack_resolve_error(
                "surface-mapping",
                format!("product command `{command}` is not exposed by product `{pack_id}`"),
            )
        })?;
    let product_route = agentflow_pack::product_command_route(&definition, command_entry)
        .map_err(|error| pack_resolve_error("surface-mapping", error.to_string()))?;
    if crate::mapping::action_type_for_action_contract_ref(&product_route.action_contract_ref)
        .is_none()
    {
        return Err(pack_resolve_error(
            "surface-mapping",
            format!(
                "product command `{command}` uses unsupported action contract `{}`",
                product_route.action_contract_ref
            ),
        ));
    }
    let capability = product_capability_status(project_root, &definition, &product_route);

    Ok(ResolvedPackCommand {
        route: PackSurfaceRouteView {
            version: PACK_COMMAND_SURFACE_VERSION.to_string(),
            pack_id: product_route.pack_id.clone(),
            command: product_route.command.clone(),
            page_id: product_page_for_command(&definition.surface, command)
                .unwrap_or("task-workbench")
                .to_string(),
            route: "product-surface/runtime-command".to_string(),
            action_contract_ref: product_route.action_contract_ref,
            runtime_command_type: crate::mapping::CORE_RUNTIME_COMMAND_TYPE.to_string(),
            target_object_type: product_route.target_object_type,
            source_refs: product_route.source_refs,
        },
        capability,
    })
}

fn product_page_for_command<'a>(
    surface: &'a agentflow_pack::ProductSurfaceDefinition,
    command: &str,
) -> Option<&'a str> {
    let page_id = match command {
        "work.issue.start" | "work.issue.review" => "task-workbench",
        _ => return None,
    };
    surface
        .pages
        .iter()
        .find(|page| page.id == page_id)
        .map(|page| page.id.as_str())
}

fn product_capability_status(
    project_root: &Path,
    definition: &agentflow_pack::ProductDefinition,
    route: &agentflow_pack::ProductCommandRoute,
) -> PackCapabilityStatusView {
    let (worker_id, required_capability) = product_required_capability(&route.command);
    let connector_exists = definition
        .connectors
        .connectors
        .iter()
        .any(|connector| connector.id == worker_id && !connector.authority);
    if !connector_exists {
        return PackCapabilityStatusView {
            version: PACK_COMMAND_SURFACE_VERSION.to_string(),
            pack_id: route.pack_id.clone(),
            command: route.command.clone(),
            required_capabilities: vec![required_capability.to_string()],
            provider_ids: vec![worker_id.to_string()],
            command_boundary: "runtime-api/product-surface/action-contract/arbitration".to_string(),
            available: false,
            reason: format!(
                "product command `{}` has no non-authority connector `{worker_id}`",
                route.command
            ),
        };
    }

    let registry = load_project_capability_registry(project_root)
        .unwrap_or_else(|_| agentflow_capability_registry::default_capability_registry());
    let decision = evaluate_command(&registry, worker_id, required_capability);
    PackCapabilityStatusView {
        version: PACK_COMMAND_SURFACE_VERSION.to_string(),
        pack_id: route.pack_id.clone(),
        command: route.command.clone(),
        required_capabilities: decision.required_capabilities,
        provider_ids: vec![worker_id.to_string()],
        command_boundary: "runtime-api/product-surface/action-contract/arbitration".to_string(),
        available: decision.enabled && decision.health == WorkerHealth::Ready,
        reason: decision.disabled_reason.unwrap_or_else(|| {
            "product command is available through product connector and capability registry"
                .to_string()
        }),
    }
}

fn product_required_capability(command: &str) -> (&'static str, &'static str) {
    match command {
        "work.issue.review" => ("codex", "build_agent.complete"),
        _ => ("codex", "launch"),
    }
}

fn capability_status_from_decisions(
    pack_id: &str,
    command: &str,
    decisions: &[PackConnectorCommandDecision],
) -> PackCapabilityStatusView {
    let required_capabilities = decisions
        .iter()
        .map(|decision| decision.required_capability.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let provider_ids = decisions
        .iter()
        .map(|decision| decision.connector_id.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let unavailable_reasons = decisions
        .iter()
        .filter_map(|decision| {
            if decision.enabled && decision.health == WorkerHealth::Ready {
                None
            } else {
                Some(decision.disabled_reason.clone().unwrap_or_else(|| {
                    format!(
                        "worker {} is not ready: {}",
                        decision.worker_id,
                        decision.health.as_str()
                    )
                }))
            }
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let available = !decisions.is_empty() && unavailable_reasons.is_empty();
    let reason = if decisions.is_empty() {
        format!("pack command `{command}` has no connector capability mapping")
    } else if available {
        "pack command is available through capability registry and provider smoke state".to_string()
    } else {
        unavailable_reasons.join("; ")
    };

    PackCapabilityStatusView {
        version: PACK_COMMAND_SURFACE_VERSION.to_string(),
        pack_id: pack_id.to_string(),
        command: command.to_string(),
        required_capabilities,
        provider_ids,
        command_boundary: "runtime-api/action-contract/arbitration".to_string(),
        available,
        reason,
    }
}

fn load_project_capability_registry(project_root: &Path) -> Result<CapabilityRegistry> {
    let path = project_root.join(".agentflow/runtime/capability-registry.json");
    if !path.is_file() {
        return Ok(agentflow_capability_registry::default_capability_registry());
    }
    load_pack_definition(path)
}

fn load_file_backed_pack_bundle(
    entry: agentflow_pack::PackRegistryEntry,
) -> Result<FileBackedPackBundle, String> {
    let domain = load_pack_domain_definition(&entry).map_err(|error| {
        format!(
            "pack `{}` domain definition unreadable: {error}",
            entry.pack_id
        )
    })?;
    let surface = load_pack_surface_definition(&entry).map_err(|error| {
        format!(
            "pack `{}` surface definition unreadable: {error}",
            entry.pack_id
        )
    })?;
    let connector = load_pack_connector_definition(&entry).map_err(|error| {
        format!(
            "pack `{}` connector definition unreadable: {error}",
            entry.pack_id
        )
    })?;
    Ok(FileBackedPackBundle {
        entry,
        domain,
        surface,
        connector,
    })
}

fn load_pack_domain_definition(
    entry: &agentflow_pack::PackRegistryEntry,
) -> Result<agentflow_pack::PackDomainDefinition> {
    load_pack_definition(definition_path_for_entry(entry, &entry.domain_path))
}

fn load_pack_surface_definition(
    entry: &agentflow_pack::PackRegistryEntry,
) -> Result<agentflow_pack::PackSurfaceDefinition> {
    load_pack_definition(definition_path_for_entry(entry, &entry.surface_path))
}

fn load_pack_connector_definition(
    entry: &agentflow_pack::PackRegistryEntry,
) -> Result<agentflow_pack::PackConnectorDefinition> {
    load_pack_definition(definition_path_for_entry(entry, &entry.connector_path))
}

fn load_pack_definition<T: DeserializeOwned>(path: PathBuf) -> Result<T> {
    let payload = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str::<T>(&payload).with_context(|| format!("parse {}", path.display()))
}

fn definition_path_for_entry(
    entry: &agentflow_pack::PackRegistryEntry,
    relative_path: &str,
) -> PathBuf {
    let path = PathBuf::from(&entry.pack_root).join(relative_path);
    if path.extension().is_some() {
        path
    } else {
        path.join("definition.json")
    }
}

fn runtime_command_type_for_action_contract(action_contract_ref: &str) -> Option<&'static str> {
    crate::mapping::action_type_for_action_contract_ref(action_contract_ref)
        .map(|_| crate::mapping::CORE_RUNTIME_COMMAND_TYPE)
}

fn runtime_command_from_pack_request(
    request: &PackCommandRequest,
    resolved: &ResolvedPackCommand,
) -> RuntimeCommandRequest {
    RuntimeCommandRequest {
        command_id: request.command_id.clone(),
        command_type: resolved.route.runtime_command_type.clone(),
        route: Some(crate::mapping::pack_runtime_route(
            request.pack_id.clone(),
            request.command.clone(),
            resolved.route.action_contract_ref.clone(),
            request.target_object_type.clone(),
        )),
        source_surface: request.source_surface.clone(),
        actor_role: request.actor_role.clone(),
        skill_ref: Some(format!("pack:{}:{}", request.pack_id, request.command)),
        target_object_ref: Some(ActionRef {
            object_type: request.target_object_type.clone(),
            id: request.target_object_id.clone(),
        }),
        input: request.input.clone(),
        evidence_refs: request.evidence_refs.clone(),
        artifact_refs: request.artifact_refs.clone(),
        expected_outputs: Vec::new(),
        evidence_policy: None,
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

fn set_failure_stage(stage: &mut Option<String>, value: &'static str) {
    if stage.is_none() {
        *stage = Some(value.to_string());
    }
}

fn pack_command_failure_message(
    request: &PackCommandRequest,
    stage: Option<&str>,
    reason: impl Into<String>,
) -> String {
    format!(
        "pack `{}` command `{}` failed at {}: {}",
        request.pack_id,
        request.command,
        stage.unwrap_or("validation"),
        reason.into()
    )
}

fn invalid_pack_command_response(
    request: &PackCommandRequest,
    rejected_reasons: Vec<RuntimeCommandError>,
) -> RuntimeCommandResponse {
    RuntimeCommandResponse {
        version: RUNTIME_COMMAND_API_VERSION.to_string(),
        command_id: request.command_id.clone(),
        proposal_id: format!("proposal-{}", request.command_id),
        status: RuntimeCommandStatus::InvalidCommand,
        decision: RuntimeCommandDecision::InvalidCommand,
        accepted_action_id: None,
        rejected_reasons,
        human_decision_request: None,
        next_query_hint: None,
        governance_admission: None,
        correlation_id: request.command_id.clone(),
    }
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
    use agentflow_capability_registry::{
        CapabilityPolicy, CapabilityRegistry, WorkerBoundary, WorkerCapability, WorkerHealth,
        WorkerKind, WorkerRegistryEntry, CAPABILITY_REGISTRY_VERSION,
    };
    use serde_json::json;
    use std::path::Path;

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
        write_pack_bundle(dir.path(), "software-dev", "work.issue.start");
        write_pack_bundle(dir.path(), "ui-design", "design.wireframe.generate");

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
        write_pack_bundle(dir.path(), "software-dev", "work.issue.start");

        let route =
            query_pack_surface_route(dir.path(), "software-dev", "work.issue.start").unwrap();
        let status =
            query_pack_capability_status(dir.path(), "software-dev", "work.issue.start").unwrap();

        assert_eq!(
            route.runtime_command_type,
            crate::mapping::CORE_RUNTIME_COMMAND_TYPE
        );
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
    fn runtime_resolves_product_surface_route_before_pack_registry() {
        let root = workspace_root();

        let route = query_pack_surface_route(&root, "software-dev", "work.issue.start").unwrap();
        let review_route =
            query_pack_surface_route(&root, "software-dev", "work.issue.review").unwrap();

        assert_eq!(route.action_contract_ref, "action-contract:issue.start");
        assert_eq!(route.target_object_type, "Issue");
        assert!(route
            .source_refs
            .iter()
            .any(|source| source == "products/software-dev/surface/definition.json"));
        assert_eq!(
            review_route.action_contract_ref,
            "action-contract:delivery.prepare"
        );
        assert!(review_route
            .source_refs
            .iter()
            .any(|source| source == "products/software-dev/product.toml"));
    }

    #[test]
    fn runtime_validates_pack_command_through_action_contract() {
        let dir = tempfile::tempdir().unwrap();
        write_pack_bundle(dir.path(), "software-dev", "work.issue.start");
        let request = software_issue_start_request();

        let report = validate_pack_command(dir.path(), &request).unwrap();

        assert!(report.valid);
        let runtime_command = report.runtime_command.unwrap();
        assert_eq!(
            runtime_command.command_type,
            crate::mapping::CORE_RUNTIME_COMMAND_TYPE
        );
        let route = runtime_command.route.as_ref().unwrap();
        assert_eq!(route.action_contract_ref, "action-contract:issue.start");
        assert_eq!(route.pack_id.as_deref(), Some("software-dev"));
        assert_eq!(route.pack_command.as_deref(), Some("work.issue.start"));
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
    fn runtime_disables_pack_command_when_capability_is_disabled() {
        let dir = tempfile::tempdir().unwrap();
        write_pack_bundle(dir.path(), "software-dev", "work.issue.start");
        write_github_capability_registry(
            dir.path(),
            WorkerHealth::Ready,
            "repo.read",
            false,
            CapabilityPolicy::Disabled,
            Some("provider capability disabled by fixture"),
        );
        let request = software_issue_start_request();

        let report = validate_pack_command(dir.path(), &request).unwrap();
        let status = report.capability_status.unwrap();

        assert!(!status.available);
        assert!(status
            .reason
            .contains("provider capability disabled by fixture"));
        assert!(!report.valid);
        assert!(report
            .rejected_reasons
            .iter()
            .any(|reason| reason.message.contains("unavailable")));
    }

    #[test]
    fn runtime_reports_degraded_capability_as_unavailable_not_ready() {
        let dir = tempfile::tempdir().unwrap();
        write_pack_bundle(dir.path(), "software-dev", "work.issue.start");
        write_github_capability_registry(
            dir.path(),
            WorkerHealth::Degraded,
            "repo.read",
            true,
            CapabilityPolicy::Allowed,
            None,
        );

        let status =
            query_pack_capability_status(dir.path(), "software-dev", "work.issue.start").unwrap();

        assert!(!status.available);
        assert!(status.reason.contains("degraded"));
    }

    #[test]
    fn runtime_rejects_invalid_pack_command_with_readable_reason() {
        let dir = tempfile::tempdir().unwrap();
        write_pack_bundle(dir.path(), "software-dev", "work.issue.start");
        let mut request = software_issue_start_request();
        request.command = "work.issue.teleport".to_string();

        let report = validate_pack_command(dir.path(), &request).unwrap();

        assert!(!report.valid);
        assert_eq!(report.failure_stage.as_deref(), Some("surface-mapping"));
        assert!(report.rejected_reasons.iter().any(|reason| reason
            .message
            .contains("software-dev")
            && reason.message.contains("work.issue.teleport")
            && reason.message.contains("surface-mapping")));
    }

    #[test]
    fn runtime_rejects_schema_failure_with_pack_and_command_context() {
        let dir = tempfile::tempdir().unwrap();
        write_pack_bundle(dir.path(), "software-dev", "work.issue.start");
        let mut request = software_issue_start_request();
        request.command_id = "".to_string();

        let report = validate_pack_command(dir.path(), &request).unwrap();

        assert!(!report.valid);
        assert_eq!(report.failure_stage.as_deref(), Some("schema"));
        assert!(report.rejected_reasons.iter().any(|reason| {
            reason.message.contains("software-dev")
                && reason.message.contains("work.issue.start")
                && reason.message.contains("schema")
        }));
    }

    #[test]
    fn runtime_submit_rejects_invalid_pack_command_without_runtime_write() {
        let dir = tempfile::tempdir().unwrap();
        write_pack_bundle(dir.path(), "software-dev", "work.issue.start");
        let mut request = software_issue_start_request();
        request.command = "work.issue.teleport".to_string();

        let response = submit_pack_action_proposal(dir.path(), &request).unwrap();

        assert_eq!(response.status, RuntimeCommandStatus::InvalidCommand);
        assert!(!response.rejected_reasons.is_empty());
        assert!(!dir.path().join(".agentflow/runtime/commands").exists());
    }

    #[test]
    fn runtime_rejects_registered_pack_when_definition_file_is_missing() {
        let dir = tempfile::tempdir().unwrap();
        write_pack_bundle(dir.path(), "software-dev", "work.issue.start");
        std::fs::remove_file(
            dir.path()
                .join(".agentflow/packs/software-dev/surface/definition.json"),
        )
        .unwrap();
        let request = software_issue_start_request();

        let report = validate_pack_command(dir.path(), &request).unwrap();

        assert!(!report.valid);
        assert!(report.rejected_reasons.iter().any(|reason| {
            reason.message.contains("surface definition unreadable")
                && reason.message.contains("definition.json")
        }));
    }

    #[test]
    fn runtime_dry_run_pack_command_does_not_write_or_execute() {
        let dir = tempfile::tempdir().unwrap();
        write_pack_bundle(dir.path(), "software-dev", "work.issue.start");
        let request = software_issue_start_request();

        let report = dry_run_pack_command(dir.path(), &request).unwrap();

        assert!(report.valid);
        assert!(!report.writes_authority);
        assert!(!report.writes_event_store);
        assert!(!report.executes_provider);
        assert!(report.would_submit_to_arbitration);
        assert!(!dir.path().join(".agentflow/runtime/commands").exists());
    }

    #[test]
    fn runtime_submit_pack_action_uses_arbitration_entrypoint() {
        let dir = tempfile::tempdir().unwrap();
        write_pack_bundle(dir.path(), "software-dev", "work.issue.start");
        let request = software_issue_start_request();

        let response = submit_pack_action_proposal(dir.path(), &request).unwrap();

        assert_ne!(response.status, RuntimeCommandStatus::InvalidCommand);
        assert!(dir.path().join(".agentflow/runtime/commands").exists());
    }

    #[test]
    fn runtime_resolves_custom_pack_command_from_registry_files() {
        let dir = tempfile::tempdir().unwrap();
        write_pack_bundle(dir.path(), "custom-pack", "custom.issue.start");
        let mut request = software_issue_start_request();
        request.pack_id = "custom-pack".to_string();
        request.command = "custom.issue.start".to_string();

        let report = validate_pack_command(dir.path(), &request).unwrap();

        assert!(report.valid);
        let route = report.surface_route.unwrap();
        assert_eq!(route.pack_id, "custom-pack");
        assert_eq!(route.command, "custom.issue.start");
        assert!(route
            .source_refs
            .iter()
            .any(|source| source.ends_with("domain/definition.json")));
        let capability = report.capability_status.unwrap();
        assert!(capability
            .required_capabilities
            .contains(&"custom-pack.capability".to_string()));
        assert!(capability.provider_ids.contains(&"github".to_string()));
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

    fn write_pack_bundle(root: &Path, pack_id: &str, command: &str) {
        let pack_dir = root.join(".agentflow/packs").join(pack_id);
        std::fs::create_dir_all(pack_dir.join("domain")).unwrap();
        std::fs::create_dir_all(pack_dir.join("surface")).unwrap();
        std::fs::create_dir_all(pack_dir.join("connectors")).unwrap();

        let manifest = agentflow_pack::PackManifest {
            version: agentflow_pack::PACK_MANIFEST_VERSION.to_string(),
            pack_id: pack_id.to_string(),
            name: format!("{pack_id} test pack"),
            pack_type: agentflow_pack::PackType::Custom,
            pack_version: "0.8.1".to_string(),
            runtime_compatibility: ">=0.8.0".to_string(),
            domain_path: "domain/".to_string(),
            surface_path: "surface/".to_string(),
            connector_path: "connectors/".to_string(),
            required_capabilities: vec![format!("{pack_id}.capability")],
            owned_object_types: vec!["Issue".to_string(), "Run".to_string()],
            exposed_commands: vec![command.to_string()],
            projection_entries: vec!["projection.task-workbench".to_string()],
            migration_policy: agentflow_pack::PackMigrationPolicy::PreviewOnly,
            validation_status: agentflow_pack::PackValidationStatus::Valid,
        };
        let mut domain = agentflow_pack::software_dev_domain_definition();
        domain.pack_id = pack_id.to_string();
        domain.domain_id = format!("{pack_id}-domain");
        let mut surface = agentflow_pack::software_dev_surface_definition();
        surface.pack_id = pack_id.to_string();
        surface.surface_id = format!("{pack_id}-surface");
        for mapping in surface.command_entry_mappings.iter_mut() {
            if mapping.action_contract_ref == "action-contract:issue.start" {
                mapping.command_type = command.to_string();
            }
        }
        let mut connector = agentflow_pack::software_dev_connector_definition();
        connector.pack_id = pack_id.to_string();
        connector.connector_id = format!("{pack_id}-connectors");
        if let Some(github_connector) = connector
            .connectors
            .iter_mut()
            .find(|connector| connector.connector_id == "github")
        {
            let capability = capability_for_pack_command(pack_id, command);
            github_connector
                .required_capabilities
                .push(capability.clone());
            github_connector.required_capabilities.sort();
            github_connector.required_capabilities.dedup();
            github_connector
                .supported_actions
                .push(agentflow_pack::ConnectorSupportedAction {
                    action_id: format!("{command}.capability"),
                    label: command.to_string(),
                    required_capability: capability,
                    command_type: command.to_string(),
                    writes_external: false,
                    evidence_output: "connector.evidence".to_string(),
                });
        }

        write_json(pack_dir.join("pack.json"), &manifest);
        write_json(pack_dir.join("domain/definition.json"), &domain);
        write_json(pack_dir.join("surface/definition.json"), &surface);
        write_json(pack_dir.join("connectors/definition.json"), &connector);
        write_github_capability_registry(
            root,
            WorkerHealth::Ready,
            &capability_for_pack_command(pack_id, command),
            true,
            CapabilityPolicy::Allowed,
            None,
        );
    }

    fn capability_for_pack_command(pack_id: &str, command: &str) -> String {
        if pack_id == "software-dev" && command == "work.issue.start" {
            "repo.read".to_string()
        } else {
            format!("{pack_id}.capability")
        }
    }

    fn write_github_capability_registry(
        root: &Path,
        health: WorkerHealth,
        capability_id: &str,
        available: bool,
        policy: CapabilityPolicy,
        disabled_reason: Option<&str>,
    ) {
        let registry = CapabilityRegistry {
            version: CAPABILITY_REGISTRY_VERSION.to_string(),
            workers: vec![WorkerRegistryEntry {
                worker_id: "github".to_string(),
                title: "GitHub Connector".to_string(),
                kind: WorkerKind::AgentProvider,
                health,
                requires_auth: false,
                disabled_reason: None,
                provider_smoke: None,
                runtime_roles: Vec::new(),
                skill_packs: Vec::new(),
                tool_kinds: Vec::new(),
                capabilities: vec![WorkerCapability {
                    capability_id: capability_id.to_string(),
                    label: capability_id.to_string(),
                    command: capability_id.to_string(),
                    required: true,
                    available,
                    requires_auth: false,
                    policy,
                    disabled_reason: disabled_reason.map(str::to_string),
                }],
                boundary: WorkerBoundary::connector(
                    vec!["repo".to_string()],
                    vec!["pull-request".to_string()],
                    vec!["evidence".to_string()],
                ),
            }],
        };
        let path = root.join(".agentflow/runtime/capability-registry.json");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        write_json(path, &registry);
    }

    fn write_json(path: impl AsRef<Path>, value: &impl serde::Serialize) {
        std::fs::write(path, serde_json::to_string_pretty(value).unwrap()).unwrap();
    }

    fn workspace_root() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .unwrap()
            .to_path_buf()
    }
}
