use crate::{
    software_dev_connector_definition, software_dev_domain_definition,
    software_dev_surface_definition, ui_design_connector_definition, ui_design_domain_definition,
    ui_design_surface_definition, validate_connector_definition, validate_domain_definition,
    validate_pack_manifest, validate_surface_definition, ConnectorCommandBoundary,
    PackConnectorDefinition, PackConnectorValidationReport, PackDomainDefinition,
    PackDomainValidationReport, PackManifest, PackManifestValidationReport, PackMigrationPolicy,
    PackSurfaceDefinition, PackSurfaceValidationReport, PackType, PackValidationStatus,
    PACK_MANIFEST_VERSION,
};
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

pub const PACK_VALIDATION_ARTIFACT_VERSION: &str = "agentflow-pack-validation.v1";
pub const PACK_MIGRATION_PREVIEW_VERSION: &str = "agentflow-pack-migration-preview.v1";
pub const PACK_MIGRATION_PREVIEW_RECEIPT_VERSION: &str =
    "agentflow-pack-migration-preview-receipt.v1";
pub const PACK_MIGRATION_APPLIED_RECEIPT_VERSION: &str =
    "agentflow-pack-migration-applied-receipt.v1";
pub const PACK_MIGRATION_CANCEL_RECEIPT_VERSION: &str =
    "agentflow-pack-migration-cancel-receipt.v1";
pub const PACK_MIGRATION_ROLLBACK_RECEIPT_VERSION: &str =
    "agentflow-pack-migration-rollback-receipt.v1";
pub const PACK_READINESS_ARTIFACT_VERSION: &str = "agentflow-pack-readiness.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackValidationArtifact {
    pub version: String,
    pub pack_id: String,
    pub active: bool,
    pub writes_authority: bool,
    pub manifest: PackManifestValidationReport,
    pub domain: PackDomainValidationReport,
    pub surface: PackSurfaceValidationReport,
    pub connector: PackConnectorValidationReport,
    pub version_compatibility: PackVersionCompatibility,
    pub api_plane_mapping: PackApiPlaneMapping,
    pub missing_read_models: Vec<String>,
    pub missing_command_mappings: Vec<String>,
    pub issues: Vec<PackValidationIssue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackValidationIssue {
    pub field: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackVersionCompatibility {
    pub required_runtime: String,
    pub current_runtime: String,
    pub compatible: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackApiPlaneMapping {
    pub checked_entries: Vec<String>,
    pub missing_entries: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackReadinessArtifact {
    pub version: String,
    pub pack_id: String,
    pub status: String,
    pub writes_authority: bool,
    pub can_load: bool,
    pub can_validate: bool,
    pub can_project: bool,
    pub main_chain: Vec<String>,
    pub audit_sidecar_chain: Vec<String>,
    pub finding_policy: String,
    pub validation: PackValidationArtifact,
    pub projection_entries: Vec<String>,
    pub source_refs: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackMigrationPreview {
    pub version: String,
    pub preview_id: String,
    pub pack_id: String,
    pub from_version: String,
    pub to_version: String,
    pub writes_authority: bool,
    pub affected_objects: Vec<String>,
    pub affected_projections: Vec<String>,
    pub required_human_confirmation: bool,
    pub preview_receipt: PackMigrationPreviewReceipt,
    pub applied_receipt_boundary: PackAppliedMigrationBoundary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackMigrationPreviewReceipt {
    pub version: String,
    pub preview_id: String,
    pub pack_id: String,
    pub writes_authority: bool,
    pub affected_object_count: usize,
    pub affected_projection_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackAppliedMigrationBoundary {
    pub version: String,
    pub requires_explicit_confirmation: bool,
    pub receipt_path_pattern: String,
    pub authority_writes_deferred: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackMigrationApplyConfirmation {
    pub preview_id: String,
    pub confirmed: bool,
    pub actor: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackMigrationSemanticTarget {
    pub version: String,
    pub preview_id: String,
    pub pack_id: String,
    pub from_version: String,
    pub to_version: String,
    pub affected_objects: Vec<String>,
    pub affected_projections: Vec<String>,
    pub mutation_target: String,
    pub authority_mutation: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackMigrationAppliedReceipt {
    pub version: String,
    pub preview_id: String,
    pub pack_id: String,
    pub applied: bool,
    pub writes_authority: bool,
    pub semantic_target: PackMigrationSemanticTarget,
    pub actor: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackMigrationCancelReceipt {
    pub version: String,
    pub preview_id: String,
    pub pack_id: String,
    pub cancelled: bool,
    pub writes_authority: bool,
    pub actor: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackMigrationRollbackReceipt {
    pub version: String,
    pub preview_id: String,
    pub pack_id: String,
    pub rolled_back: bool,
    pub writes_authority: bool,
    pub semantic_target: PackMigrationSemanticTarget,
    pub actor: String,
    pub reason: String,
    pub applied_receipt_version: String,
}

pub fn validate_pack_bundle(
    manifest: &PackManifest,
    domain: &PackDomainDefinition,
    surface: &PackSurfaceDefinition,
    connector: &PackConnectorDefinition,
    api_plane_entries: &[String],
    current_runtime_version: &str,
) -> PackValidationArtifact {
    let manifest_report = validate_pack_manifest(manifest);
    let domain_report = validate_domain_definition(domain);
    let surface_report = validate_surface_definition(surface);
    let connector_report = validate_connector_definition(connector);
    let mut issues = Vec::new();

    require_same_pack_id(
        &mut issues,
        "domain.packId",
        &manifest.pack_id,
        &domain.pack_id,
    );
    require_same_pack_id(
        &mut issues,
        "surface.packId",
        &manifest.pack_id,
        &surface.pack_id,
    );
    require_same_pack_id(
        &mut issues,
        "connector.packId",
        &manifest.pack_id,
        &connector.pack_id,
    );

    let version_compatibility =
        version_compatibility(&manifest.runtime_compatibility, current_runtime_version);
    if !version_compatibility.compatible {
        issues.push(issue(
            "runtimeCompatibility",
            "pack runtime compatibility does not match current runtime",
        ));
    }

    let api_plane_mapping = api_plane_mapping(
        surface,
        connector,
        api_plane_entries.iter().map(String::as_str),
    );
    let missing_read_models = missing_read_models(
        surface,
        manifest.projection_entries.iter().map(String::as_str),
    );
    let missing_command_mappings = missing_command_mappings(
        surface,
        connector,
        api_plane_entries.iter().map(String::as_str),
    );
    for missing in &missing_read_models {
        issues.push(issue(
            "surface.readModelDependencies",
            &format!("missing read model mapping for {missing}"),
        ));
    }
    for missing in &missing_command_mappings {
        issues.push(issue(
            "surface.commandEntryMappings",
            &format!("missing command mapping for {missing}"),
        ));
    }
    for missing in &api_plane_mapping.missing_entries {
        issues.push(issue(
            "apiPlaneMapping",
            &format!("missing API Plane entry for {missing}"),
        ));
    }

    let active = manifest_report.valid
        && domain_report.valid
        && surface_report.valid
        && connector_report.valid
        && issues.is_empty();

    PackValidationArtifact {
        version: PACK_VALIDATION_ARTIFACT_VERSION.to_string(),
        pack_id: manifest.pack_id.clone(),
        active,
        writes_authority: false,
        manifest: manifest_report,
        domain: domain_report,
        surface: surface_report,
        connector: connector_report,
        version_compatibility,
        api_plane_mapping,
        missing_read_models,
        missing_command_mappings,
        issues,
    }
}

pub fn load_pack_validation_artifact(path: impl AsRef<Path>) -> Result<PackValidationArtifact> {
    let path = path.as_ref();
    let payload = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&payload)
        .with_context(|| format!("parse pack validation artifact {}", path.display()))
}

pub fn pack_validation_artifact_path(project_root: impl AsRef<Path>, pack_id: &str) -> PathBuf {
    project_root
        .as_ref()
        .join(".agentflow/packs")
        .join(pack_id)
        .join("validation/validation.json")
}

pub fn software_dev_pack_readiness_artifact(
    api_plane_entries: &[String],
    current_runtime_version: &str,
) -> PackReadinessArtifact {
    let domain = software_dev_domain_definition();
    let surface = software_dev_surface_definition();
    let connector = software_dev_connector_definition();
    let manifest = built_in_pack_manifest(
        "software-dev",
        "Software Dev",
        PackType::SoftwareDev,
        &domain,
        &surface,
        &connector,
    );
    let validation = validate_pack_bundle(
        &manifest,
        &domain,
        &surface,
        &connector,
        api_plane_entries,
        current_runtime_version,
    );
    let can_load = !domain.object_types.is_empty()
        && !surface.pages.is_empty()
        && !connector.connectors.is_empty()
        && !manifest.pack_id.is_empty();
    let can_validate = validation.active;
    let can_project = validation.active && validation.missing_read_models.is_empty();
    let mut warnings = Vec::new();
    if !can_validate {
        warnings.push("software-dev-pack-validation-failed".to_string());
    }
    if !can_project {
        warnings.push("software-dev-pack-projection-not-ready".to_string());
    }

    PackReadinessArtifact {
        version: PACK_READINESS_ARTIFACT_VERSION.to_string(),
        pack_id: manifest.pack_id,
        status: if can_load && can_validate && can_project {
            "completed".to_string()
        } else {
            "deferred".to_string()
        },
        writes_authority: false,
        can_load,
        can_validate,
        can_project,
        main_chain: vec![
            "Requirement".to_string(),
            "Spec".to_string(),
            "Issue".to_string(),
            "Run".to_string(),
            "Acceptance".to_string(),
            "Delivery".to_string(),
            "Release".to_string(),
        ],
        audit_sidecar_chain: vec![
            "Delivery".to_string(),
            "OptionalAuditRequest".to_string(),
            "AuditReport".to_string(),
            "Finding".to_string(),
            "FollowUpProposal".to_string(),
        ],
        finding_policy: "finding-generates-follow-up-proposal-only".to_string(),
        validation,
        projection_entries: manifest_projection_entries(),
        source_refs: vec![
            "pack-builtin:software-dev/domain".to_string(),
            "pack-builtin:software-dev/surface".to_string(),
            "pack-builtin:software-dev/connectors".to_string(),
            "projection.pack-industry-workbench".to_string(),
        ],
        warnings,
    }
}

pub fn ui_design_pack_readiness_artifact(
    api_plane_entries: &[String],
    current_runtime_version: &str,
) -> PackReadinessArtifact {
    let domain = ui_design_domain_definition();
    let surface = ui_design_surface_definition();
    let connector = ui_design_connector_definition();
    let manifest = built_in_pack_manifest(
        "ui-design",
        "UI Design",
        PackType::UiDesign,
        &domain,
        &surface,
        &connector,
    );
    let validation = validate_pack_bundle(
        &manifest,
        &domain,
        &surface,
        &connector,
        api_plane_entries,
        current_runtime_version,
    );
    let can_load = !domain.object_types.is_empty()
        && !surface.pages.is_empty()
        && !connector.connectors.is_empty()
        && !manifest.pack_id.is_empty();
    let can_validate = validation.active;
    let can_project = validation.active && validation.missing_read_models.is_empty();
    let mut warnings = Vec::new();
    if !can_validate {
        warnings.push("ui-design-pack-validation-failed".to_string());
    }
    if !can_project {
        warnings.push("ui-design-pack-projection-not-ready".to_string());
    }

    PackReadinessArtifact {
        version: PACK_READINESS_ARTIFACT_VERSION.to_string(),
        pack_id: manifest.pack_id,
        status: if can_load && can_validate && can_project {
            "baseline".to_string()
        } else {
            "deferred".to_string()
        },
        writes_authority: false,
        can_load,
        can_validate,
        can_project,
        main_chain: vec![
            "ProductBrief".to_string(),
            "Direction".to_string(),
            "Wireframe".to_string(),
            "HiFi".to_string(),
            "DesignSystem".to_string(),
            "Handoff".to_string(),
        ],
        audit_sidecar_chain: Vec::new(),
        finding_policy: "not-applicable-ui-design-handoff-only".to_string(),
        validation,
        projection_entries: manifest_projection_entries(),
        source_refs: vec![
            "pack-builtin:ui-design/domain".to_string(),
            "pack-builtin:ui-design/surface".to_string(),
            "pack-builtin:ui-design/connectors".to_string(),
            "projection.pack-industry-workbench".to_string(),
        ],
        warnings,
    }
}

pub fn pack_readiness_api_entries() -> Vec<String> {
    [
        "spec.intake.start",
        "work.issue.start",
        "acceptance.evaluate",
        "delivery.open",
        "audit.request.sidecar",
        "github.repo.read",
        "github.pull-request.create",
        "git.status",
        "git.diff",
        "work-agent.launch",
        "work-agent.complete",
        "browser-preview.smoke",
        "figma.read",
        "assets.read",
        "frontend.status",
        "frontend.diff",
        "design-export.read",
        "design.brief.capture",
        "design.direction.select",
        "design.wireframe.generate",
        "design.hifi.review",
        "design.system.bind",
        "design.handoff.accept",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

pub fn generate_pack_migration_preview(
    preview_id: impl Into<String>,
    pack_id: impl Into<String>,
    from_version: impl Into<String>,
    to_version: impl Into<String>,
    affected_objects: Vec<String>,
    affected_projections: Vec<String>,
) -> PackMigrationPreview {
    let preview_id = preview_id.into();
    let pack_id = pack_id.into();
    let preview_receipt = PackMigrationPreviewReceipt {
        version: PACK_MIGRATION_PREVIEW_RECEIPT_VERSION.to_string(),
        preview_id: preview_id.clone(),
        pack_id: pack_id.clone(),
        writes_authority: false,
        affected_object_count: affected_objects.len(),
        affected_projection_count: affected_projections.len(),
    };
    PackMigrationPreview {
        version: PACK_MIGRATION_PREVIEW_VERSION.to_string(),
        preview_id: preview_id.clone(),
        pack_id: pack_id.clone(),
        from_version: from_version.into(),
        to_version: to_version.into(),
        writes_authority: false,
        affected_objects,
        affected_projections,
        required_human_confirmation: true,
        preview_receipt,
        applied_receipt_boundary: PackAppliedMigrationBoundary {
            version: "agentflow-pack-migration-applied-boundary.v1".to_string(),
            requires_explicit_confirmation: true,
            receipt_path_pattern:
                ".agentflow/packs/<pack-id>/migration/<preview-id>/applied-receipt.json".to_string(),
            authority_writes_deferred: true,
        },
    }
}

pub fn pack_migration_applied_receipt(
    preview: &PackMigrationPreview,
    confirmation: &PackMigrationApplyConfirmation,
) -> Result<PackMigrationAppliedReceipt> {
    validate_migration_confirmation(preview, confirmation)?;
    Ok(PackMigrationAppliedReceipt {
        version: PACK_MIGRATION_APPLIED_RECEIPT_VERSION.to_string(),
        preview_id: preview.preview_id.clone(),
        pack_id: preview.pack_id.clone(),
        applied: true,
        writes_authority: false,
        semantic_target: migration_semantic_target(preview, "receipt-only-apply"),
        actor: confirmation.actor.clone(),
        reason: confirmation.reason.clone(),
    })
}

pub fn cancel_pack_migration(
    preview: &PackMigrationPreview,
    actor: impl Into<String>,
    reason: impl Into<String>,
) -> Result<PackMigrationCancelReceipt> {
    let actor = actor.into();
    let reason = reason.into();
    require_actor_and_reason(&actor, &reason)?;
    Ok(PackMigrationCancelReceipt {
        version: PACK_MIGRATION_CANCEL_RECEIPT_VERSION.to_string(),
        preview_id: preview.preview_id.clone(),
        pack_id: preview.pack_id.clone(),
        cancelled: true,
        writes_authority: false,
        actor,
        reason,
    })
}

pub fn rollback_pack_migration(
    applied: &PackMigrationAppliedReceipt,
    actor: impl Into<String>,
    reason: impl Into<String>,
) -> Result<PackMigrationRollbackReceipt> {
    let actor = actor.into();
    let reason = reason.into();
    require_actor_and_reason(&actor, &reason)?;
    if !applied.applied {
        bail!("cannot rollback a migration that was not applied");
    }
    Ok(PackMigrationRollbackReceipt {
        version: PACK_MIGRATION_ROLLBACK_RECEIPT_VERSION.to_string(),
        preview_id: applied.preview_id.clone(),
        pack_id: applied.pack_id.clone(),
        rolled_back: true,
        writes_authority: false,
        semantic_target: PackMigrationSemanticTarget {
            mutation_target: "receipt-only-rollback".to_string(),
            ..applied.semantic_target.clone()
        },
        actor,
        reason,
        applied_receipt_version: applied.version.clone(),
    })
}

fn migration_semantic_target(
    preview: &PackMigrationPreview,
    mutation_target: impl Into<String>,
) -> PackMigrationSemanticTarget {
    PackMigrationSemanticTarget {
        version: "agentflow-pack-migration-semantic-target.v1".to_string(),
        preview_id: preview.preview_id.clone(),
        pack_id: preview.pack_id.clone(),
        from_version: preview.from_version.clone(),
        to_version: preview.to_version.clone(),
        affected_objects: preview.affected_objects.clone(),
        affected_projections: preview.affected_projections.clone(),
        mutation_target: mutation_target.into(),
        authority_mutation: false,
    }
}

fn validate_migration_confirmation(
    preview: &PackMigrationPreview,
    confirmation: &PackMigrationApplyConfirmation,
) -> Result<()> {
    if confirmation.preview_id != preview.preview_id {
        bail!(
            "migration confirmation previewId `{}` does not match preview `{}`",
            confirmation.preview_id,
            preview.preview_id
        );
    }
    if !confirmation.confirmed {
        bail!("migration apply requires explicit confirmed=true");
    }
    require_actor_and_reason(&confirmation.actor, &confirmation.reason)?;
    Ok(())
}

fn require_actor_and_reason(actor: &str, reason: &str) -> Result<()> {
    if actor.trim().is_empty() {
        bail!("migration receipt actor must not be empty");
    }
    if reason.trim().is_empty() {
        bail!("migration receipt reason must not be empty");
    }
    Ok(())
}

fn version_compatibility(required: &str, current: &str) -> PackVersionCompatibility {
    let compatible = if let Some(required_min) = required.strip_prefix(">=") {
        !current.trim().is_empty() && version_at_least(current, required_min)
    } else {
        required == current
    };
    PackVersionCompatibility {
        required_runtime: required.to_string(),
        current_runtime: current.to_string(),
        compatible,
    }
}

fn built_in_pack_manifest(
    pack_id: &str,
    name: &str,
    pack_type: PackType,
    domain: &PackDomainDefinition,
    surface: &PackSurfaceDefinition,
    connector: &PackConnectorDefinition,
) -> PackManifest {
    PackManifest {
        version: PACK_MANIFEST_VERSION.to_string(),
        pack_id: pack_id.to_string(),
        name: name.to_string(),
        pack_type,
        pack_version: "0.8.0".to_string(),
        runtime_compatibility: ">=0.8.0".to_string(),
        domain_path: "domain/".to_string(),
        surface_path: "surface/".to_string(),
        connector_path: "connectors/".to_string(),
        required_capabilities: connector
            .connectors
            .iter()
            .flat_map(|connector| connector.required_capabilities.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect(),
        owned_object_types: domain
            .object_types
            .iter()
            .map(|object| object.object_type_id.clone())
            .collect(),
        exposed_commands: surface
            .command_entry_mappings
            .iter()
            .map(|command| command.command_type.clone())
            .collect(),
        projection_entries: manifest_projection_entries(),
        migration_policy: PackMigrationPolicy::PreviewOnly,
        validation_status: PackValidationStatus::Valid,
    }
}

fn manifest_projection_entries() -> Vec<String> {
    vec![
        "projection.project-home".to_string(),
        "projection.spec-workbench".to_string(),
        "projection.task-workbench".to_string(),
        "projection.acceptance".to_string(),
        "projection.delivery".to_string(),
        "projection.event-timeline".to_string(),
        "projection.evidence-graph".to_string(),
        "projection.audit-surface".to_string(),
        "projection.design-home".to_string(),
        "projection.brief-intake".to_string(),
        "projection.direction-board".to_string(),
        "projection.wireframe-board".to_string(),
        "projection.hifi-review".to_string(),
        "projection.design-system".to_string(),
        "projection.handoff-surface".to_string(),
    ]
}

fn version_at_least(current: &str, required_min: &str) -> bool {
    version_tuple(current) >= version_tuple(required_min)
}

fn version_tuple(value: &str) -> (u64, u64, u64) {
    let mut parts = value
        .trim()
        .trim_start_matches('v')
        .split('.')
        .map(|part| part.parse::<u64>().unwrap_or(0));
    (
        parts.next().unwrap_or(0),
        parts.next().unwrap_or(0),
        parts.next().unwrap_or(0),
    )
}

fn api_plane_mapping<'a>(
    surface: &PackSurfaceDefinition,
    connector: &PackConnectorDefinition,
    api_plane_entries: impl Iterator<Item = &'a str>,
) -> PackApiPlaneMapping {
    let available = api_plane_entries.collect::<BTreeSet<_>>();
    let checked_entries = surface
        .command_entry_mappings
        .iter()
        .map(|command| command.command_type.clone())
        .chain(connector.connectors.iter().flat_map(|connector| {
            connector
                .supported_actions
                .iter()
                .map(|action| action.command_type.clone())
        }))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let missing_entries = checked_entries
        .iter()
        .filter(|entry| !available.contains(entry.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    PackApiPlaneMapping {
        checked_entries,
        missing_entries,
    }
}

fn missing_read_models<'a>(
    surface: &PackSurfaceDefinition,
    projection_entries: impl Iterator<Item = &'a str>,
) -> Vec<String> {
    let available = projection_entries.collect::<BTreeSet<_>>();
    surface
        .read_model_dependencies
        .iter()
        .filter(|dependency| !available.contains(dependency.projection_ref.as_str()))
        .map(|dependency| dependency.projection_ref.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn missing_command_mappings<'a>(
    surface: &PackSurfaceDefinition,
    connector: &PackConnectorDefinition,
    api_plane_entries: impl Iterator<Item = &'a str>,
) -> Vec<String> {
    api_plane_mapping(surface, connector, api_plane_entries).missing_entries
}

fn require_same_pack_id(
    issues: &mut Vec<PackValidationIssue>,
    field: &str,
    expected: &str,
    actual: &str,
) {
    if expected != actual {
        issues.push(issue(field, "must match manifest packId"));
    }
}

fn issue(field: &str, reason: &str) -> PackValidationIssue {
    PackValidationIssue {
        field: field.to_string(),
        reason: reason.to_string(),
    }
}

pub fn command_boundary_is_runtime_only(boundary: &ConnectorCommandBoundary) -> bool {
    boundary.runtime_command_required && !boundary.authority_write && !boundary.output_authority
}

#[cfg(test)]
mod tests {
    use super::{
        cancel_pack_migration, generate_pack_migration_preview, pack_migration_applied_receipt,
        pack_readiness_api_entries, rollback_pack_migration, software_dev_pack_readiness_artifact,
        ui_design_pack_readiness_artifact, validate_pack_bundle, PackMigrationApplyConfirmation,
    };
    use crate::{
        software_dev_connector_definition, software_dev_domain_definition,
        software_dev_surface_definition, PackManifest, PackMigrationPolicy, PackType,
        PackValidationStatus, PACK_MANIFEST_VERSION,
    };

    fn manifest() -> PackManifest {
        PackManifest {
            version: PACK_MANIFEST_VERSION.to_string(),
            pack_id: "software-dev".to_string(),
            name: "Software Dev".to_string(),
            pack_type: PackType::SoftwareDev,
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
            migration_policy: PackMigrationPolicy::PreviewOnly,
            validation_status: PackValidationStatus::Draft,
        }
    }

    fn api_entries() -> Vec<String> {
        vec![
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
        ]
    }

    #[test]
    fn valid_pack_bundle_can_be_active() {
        let artifact = validate_pack_bundle(
            &manifest(),
            &software_dev_domain_definition(),
            &software_dev_surface_definition(),
            &software_dev_connector_definition(),
            &api_entries(),
            "0.8.0",
        );

        assert!(artifact.active);
        assert!(!artifact.writes_authority);
        assert!(artifact.issues.is_empty());
        assert!(artifact.version_compatibility.compatible);
    }

    #[test]
    fn software_dev_readiness_artifact_proves_load_validate_and_project() {
        let artifact = software_dev_pack_readiness_artifact(&api_entries(), "0.8.0");

        assert_eq!(artifact.pack_id, "software-dev");
        assert_eq!(artifact.status, "completed");
        assert!(artifact.can_load);
        assert!(artifact.can_validate);
        assert!(artifact.can_project);
        assert!(!artifact.writes_authority);
        assert_eq!(
            artifact.main_chain,
            vec![
                "Requirement".to_string(),
                "Spec".to_string(),
                "Issue".to_string(),
                "Run".to_string(),
                "Acceptance".to_string(),
                "Delivery".to_string(),
                "Release".to_string(),
            ]
        );
        assert_eq!(
            artifact.audit_sidecar_chain,
            vec![
                "Delivery".to_string(),
                "OptionalAuditRequest".to_string(),
                "AuditReport".to_string(),
                "Finding".to_string(),
                "FollowUpProposal".to_string(),
            ]
        );
        assert_eq!(
            artifact.finding_policy,
            "finding-generates-follow-up-proposal-only"
        );
        assert!(artifact.validation.active);
        assert!(artifact
            .source_refs
            .contains(&"projection.pack-industry-workbench".to_string()));
    }

    #[test]
    fn ui_design_readiness_artifact_proves_design_baseline_without_task_chain() {
        let artifact = ui_design_pack_readiness_artifact(&pack_readiness_api_entries(), "0.8.0");

        assert_eq!(artifact.pack_id, "ui-design");
        assert_eq!(artifact.status, "baseline");
        assert!(artifact.can_load);
        assert!(artifact.can_validate);
        assert!(artifact.can_project);
        assert!(!artifact.writes_authority);
        assert_eq!(
            artifact.main_chain,
            vec![
                "ProductBrief".to_string(),
                "Direction".to_string(),
                "Wireframe".to_string(),
                "HiFi".to_string(),
                "DesignSystem".to_string(),
                "Handoff".to_string(),
            ]
        );
        assert!(artifact.audit_sidecar_chain.is_empty());
        assert_eq!(
            artifact.finding_policy,
            "not-applicable-ui-design-handoff-only"
        );
        assert!(artifact.validation.active);
        assert!(artifact
            .source_refs
            .contains(&"projection.pack-industry-workbench".to_string()));
    }

    #[test]
    fn invalid_pack_cannot_be_active() {
        let mut surface = software_dev_surface_definition();
        surface.writes_authority = true;
        let artifact = validate_pack_bundle(
            &manifest(),
            &software_dev_domain_definition(),
            &surface,
            &software_dev_connector_definition(),
            &api_entries(),
            "0.8.0",
        );

        assert!(!artifact.active);
        assert!(!artifact.surface.valid);
    }

    #[test]
    fn validation_reports_missing_read_models_and_commands() {
        let artifact = validate_pack_bundle(
            &manifest(),
            &software_dev_domain_definition(),
            &software_dev_surface_definition(),
            &software_dev_connector_definition(),
            &["work.issue.start".to_string()],
            "0.8.0",
        );

        assert!(!artifact.active);
        assert!(!artifact.missing_command_mappings.is_empty());
        assert!(artifact
            .api_plane_mapping
            .missing_entries
            .contains(&"github.pull-request.create".to_string()));
    }

    #[test]
    fn migration_preview_is_preview_only_and_requires_human_confirmation() {
        let preview = generate_pack_migration_preview(
            "preview-001",
            "software-dev",
            "0.8.0",
            "0.8.1",
            vec!["Issue".to_string()],
            vec!["projection.task-workbench".to_string()],
        );

        assert!(!preview.writes_authority);
        assert!(preview.required_human_confirmation);
        assert!(!preview.preview_receipt.writes_authority);
        assert!(
            preview
                .applied_receipt_boundary
                .requires_explicit_confirmation
        );
    }

    #[test]
    fn applied_migration_requires_matching_explicit_confirmation() {
        let preview = generate_pack_migration_preview(
            "preview-001",
            "software-dev",
            "0.8.0",
            "0.8.1",
            vec!["Issue".to_string()],
            vec!["projection.task-workbench".to_string()],
        );
        let rejected = pack_migration_applied_receipt(
            &preview,
            &PackMigrationApplyConfirmation {
                preview_id: "preview-001".to_string(),
                confirmed: false,
                actor: "human-owner".to_string(),
                reason: "not approved".to_string(),
            },
        );
        let mismatched = pack_migration_applied_receipt(
            &preview,
            &PackMigrationApplyConfirmation {
                preview_id: "preview-other".to_string(),
                confirmed: true,
                actor: "human-owner".to_string(),
                reason: "approved".to_string(),
            },
        );
        let applied = pack_migration_applied_receipt(
            &preview,
            &PackMigrationApplyConfirmation {
                preview_id: "preview-001".to_string(),
                confirmed: true,
                actor: "human-owner".to_string(),
                reason: "approved".to_string(),
            },
        )
        .unwrap();

        assert!(rejected.is_err());
        assert!(mismatched.is_err());
        assert!(applied.applied);
        assert!(!applied.writes_authority);
        assert!(!applied.semantic_target.authority_mutation);
        assert_eq!(
            applied.semantic_target.mutation_target,
            "receipt-only-apply"
        );
        assert_eq!(applied.semantic_target.from_version, "0.8.0");
        assert_eq!(applied.semantic_target.to_version, "0.8.1");
    }

    #[test]
    fn migration_cancel_and_rollback_have_distinct_receipts() {
        let preview = generate_pack_migration_preview(
            "preview-001",
            "software-dev",
            "0.8.0",
            "0.8.1",
            vec!["Issue".to_string()],
            vec!["projection.task-workbench".to_string()],
        );
        let cancel = cancel_pack_migration(&preview, "human-owner", "defer migration").unwrap();
        let applied = pack_migration_applied_receipt(
            &preview,
            &PackMigrationApplyConfirmation {
                preview_id: "preview-001".to_string(),
                confirmed: true,
                actor: "human-owner".to_string(),
                reason: "approved".to_string(),
            },
        )
        .unwrap();
        let rollback =
            rollback_pack_migration(&applied, "human-owner", "rollback after failure").unwrap();

        assert!(cancel.cancelled);
        assert!(!cancel.writes_authority);
        assert!(rollback.rolled_back);
        assert!(!rollback.writes_authority);
        assert!(!rollback.semantic_target.authority_mutation);
        assert_eq!(
            rollback.semantic_target.mutation_target,
            "receipt-only-rollback"
        );
        assert_ne!(cancel.version, applied.version);
        assert_ne!(rollback.version, applied.version);
    }
}
