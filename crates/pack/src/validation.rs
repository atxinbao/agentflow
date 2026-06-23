use crate::{
    validate_connector_definition, validate_domain_definition, validate_pack_manifest,
    validate_surface_definition, ConnectorCommandBoundary, PackConnectorDefinition,
    PackConnectorValidationReport, PackDomainDefinition, PackDomainValidationReport, PackManifest,
    PackManifestValidationReport, PackSurfaceDefinition, PackSurfaceValidationReport,
};
use anyhow::{Context, Result};
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
pub struct PackMigrationAppliedReceipt {
    pub version: String,
    pub preview_id: String,
    pub pack_id: String,
    pub applied: bool,
    pub writes_authority: bool,
    pub actor: String,
    pub reason: String,
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
) -> PackMigrationAppliedReceipt {
    let applied = confirmation.preview_id == preview.preview_id && confirmation.confirmed;
    PackMigrationAppliedReceipt {
        version: PACK_MIGRATION_APPLIED_RECEIPT_VERSION.to_string(),
        preview_id: preview.preview_id.clone(),
        pack_id: preview.pack_id.clone(),
        applied,
        writes_authority: applied,
        actor: confirmation.actor.clone(),
        reason: confirmation.reason.clone(),
    }
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
        generate_pack_migration_preview, pack_migration_applied_receipt, validate_pack_bundle,
        PackMigrationApplyConfirmation,
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
        let applied = pack_migration_applied_receipt(
            &preview,
            &PackMigrationApplyConfirmation {
                preview_id: "preview-001".to_string(),
                confirmed: true,
                actor: "human-owner".to_string(),
                reason: "approved".to_string(),
            },
        );

        assert!(!rejected.applied);
        assert!(!rejected.writes_authority);
        assert!(applied.applied);
        assert!(applied.writes_authority);
    }
}
