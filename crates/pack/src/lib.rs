//! AgentFlow Pack manifest schema and local registry.
//!
//! Pack files describe industry definitions under `.agentflow/packs/**`.
//! This crate only reads and validates those definitions. It does not write
//! `.agentflow/**` authority, append events, launch providers, or mutate runtime
//! state.

pub mod domain;

use anyhow::{Context, Result};
pub use domain::{
    software_dev_domain_definition, ui_design_domain_definition, validate_domain_definition,
    DomainAcceptanceSemantic, DomainActionSemantic, DomainAuditTriggerHint, DomainEvidencePolicy,
    DomainLinkType, DomainMigrationCompatibility, DomainObjectType, DomainStateMachine,
    DomainStateTransition, PackDomainDefinition, PackDomainValidationIssue,
    PackDomainValidationReport, PACK_DOMAIN_VERSION,
};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub const PACK_MANIFEST_VERSION: &str = "agentflow-pack-manifest.v1";
pub const PACK_REGISTRY_VERSION: &str = "agentflow-pack-registry.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PackType {
    SoftwareDev,
    UiDesign,
    Domain,
    Surface,
    Connector,
    Custom,
}

impl PackType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SoftwareDev => "software-dev",
            Self::UiDesign => "ui-design",
            Self::Domain => "domain",
            Self::Surface => "surface",
            Self::Connector => "connector",
            Self::Custom => "custom",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PackMigrationPolicy {
    None,
    PreviewOnly,
    ExplicitApplyRequired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PackValidationStatus {
    Draft,
    Valid,
    Invalid,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackManifest {
    pub version: String,
    pub pack_id: String,
    pub name: String,
    pub pack_type: PackType,
    pub pack_version: String,
    pub runtime_compatibility: String,
    pub domain_path: String,
    pub surface_path: String,
    pub connector_path: String,
    #[serde(default)]
    pub required_capabilities: Vec<String>,
    #[serde(default)]
    pub owned_object_types: Vec<String>,
    #[serde(default)]
    pub exposed_commands: Vec<String>,
    #[serde(default)]
    pub projection_entries: Vec<String>,
    pub migration_policy: PackMigrationPolicy,
    pub validation_status: PackValidationStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackManifestValidationIssue {
    pub field: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackManifestValidationReport {
    pub version: String,
    pub pack_id: String,
    pub valid: bool,
    pub issues: Vec<PackManifestValidationIssue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackRegistryEntry {
    pub pack_id: String,
    pub name: String,
    pub pack_type: PackType,
    pub pack_version: String,
    pub manifest_path: String,
    pub pack_root: String,
    pub domain_path: String,
    pub surface_path: String,
    pub connector_path: String,
    pub required_capabilities: Vec<String>,
    pub owned_object_types: Vec<String>,
    pub exposed_commands: Vec<String>,
    pub projection_entries: Vec<String>,
    pub migration_policy: PackMigrationPolicy,
    pub validation_status: PackValidationStatus,
    pub validation: PackManifestValidationReport,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackRegistry {
    pub version: String,
    pub root: String,
    pub writes_authority: bool,
    pub entries: Vec<PackRegistryEntry>,
}

impl PackRegistry {
    pub fn pack(&self, pack_id: &str) -> Option<&PackRegistryEntry> {
        self.entries.iter().find(|entry| entry.pack_id == pack_id)
    }
}

pub fn pack_root(project_root: impl AsRef<Path>) -> PathBuf {
    project_root.as_ref().join(".agentflow/packs")
}

pub fn load_pack_manifest(path: impl AsRef<Path>) -> Result<PackManifest> {
    let path = path.as_ref();
    let payload = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let manifest = serde_json::from_str::<PackManifest>(&payload)
        .with_context(|| format!("parse pack manifest {}", path.display()))?;
    Ok(manifest)
}

pub fn validate_pack_manifest(manifest: &PackManifest) -> PackManifestValidationReport {
    let mut issues = Vec::new();
    require_non_empty(&mut issues, "version", &manifest.version);
    require_value(
        &mut issues,
        "version",
        &manifest.version,
        PACK_MANIFEST_VERSION,
    );
    require_non_empty(&mut issues, "packId", &manifest.pack_id);
    require_non_empty(&mut issues, "name", &manifest.name);
    require_non_empty(&mut issues, "packVersion", &manifest.pack_version);
    require_non_empty(
        &mut issues,
        "runtimeCompatibility",
        &manifest.runtime_compatibility,
    );
    require_relative_path(&mut issues, "domainPath", &manifest.domain_path);
    require_relative_path(&mut issues, "surfacePath", &manifest.surface_path);
    require_relative_path(&mut issues, "connectorPath", &manifest.connector_path);

    PackManifestValidationReport {
        version: "agentflow-pack-manifest-validation.v1".to_string(),
        pack_id: manifest.pack_id.clone(),
        valid: issues.is_empty(),
        issues,
    }
}

pub fn load_pack_registry(project_root: impl AsRef<Path>) -> Result<PackRegistry> {
    let root = pack_root(project_root);
    if !root.exists() {
        return Ok(PackRegistry {
            version: PACK_REGISTRY_VERSION.to_string(),
            root: normalize_path(&root),
            writes_authority: false,
            entries: Vec::new(),
        });
    }

    let mut entries = Vec::new();
    for pack_dir in fs::read_dir(&root).with_context(|| format!("read {}", root.display()))? {
        let pack_dir = pack_dir?;
        let file_type = pack_dir.file_type()?;
        if !file_type.is_dir() {
            continue;
        }
        let pack_root = pack_dir.path();
        let manifest_path = pack_root.join("pack.json");
        if !manifest_path.is_file() {
            continue;
        }
        let manifest = load_pack_manifest(&manifest_path)?;
        let validation = validate_pack_manifest(&manifest);
        entries.push(registry_entry(
            pack_root,
            manifest_path,
            manifest,
            validation,
        ));
    }
    entries.sort_by(|left, right| left.pack_id.cmp(&right.pack_id));

    Ok(PackRegistry {
        version: PACK_REGISTRY_VERSION.to_string(),
        root: normalize_path(&root),
        writes_authority: false,
        entries,
    })
}

fn registry_entry(
    pack_root: PathBuf,
    manifest_path: PathBuf,
    manifest: PackManifest,
    validation: PackManifestValidationReport,
) -> PackRegistryEntry {
    PackRegistryEntry {
        pack_id: manifest.pack_id,
        name: manifest.name,
        pack_type: manifest.pack_type,
        pack_version: manifest.pack_version,
        manifest_path: normalize_path(&manifest_path),
        pack_root: normalize_path(&pack_root),
        domain_path: manifest.domain_path,
        surface_path: manifest.surface_path,
        connector_path: manifest.connector_path,
        required_capabilities: manifest.required_capabilities,
        owned_object_types: manifest.owned_object_types,
        exposed_commands: manifest.exposed_commands,
        projection_entries: manifest.projection_entries,
        migration_policy: manifest.migration_policy,
        validation_status: manifest.validation_status,
        validation,
    }
}

fn require_non_empty(issues: &mut Vec<PackManifestValidationIssue>, field: &str, value: &str) {
    if value.trim().is_empty() {
        issues.push(PackManifestValidationIssue {
            field: field.to_string(),
            reason: "must not be empty".to_string(),
        });
    }
}

fn require_value(
    issues: &mut Vec<PackManifestValidationIssue>,
    field: &str,
    value: &str,
    expected: &str,
) {
    if value != expected {
        issues.push(PackManifestValidationIssue {
            field: field.to_string(),
            reason: format!("must be {expected}"),
        });
    }
}

fn require_relative_path(issues: &mut Vec<PackManifestValidationIssue>, field: &str, value: &str) {
    require_non_empty(issues, field, value);
    if value.starts_with('/') || value.contains("..") {
        issues.push(PackManifestValidationIssue {
            field: field.to_string(),
            reason: "must be a pack-local relative path".to_string(),
        });
    }
}

fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::{
        load_pack_registry, validate_pack_manifest, PackManifest, PackMigrationPolicy, PackType,
        PackValidationStatus, PACK_MANIFEST_VERSION,
    };

    fn manifest(pack_id: &str) -> PackManifest {
        PackManifest {
            version: PACK_MANIFEST_VERSION.to_string(),
            pack_id: pack_id.to_string(),
            name: "Software Dev".to_string(),
            pack_type: PackType::SoftwareDev,
            pack_version: "0.1.0".to_string(),
            runtime_compatibility: ">=0.8.0".to_string(),
            domain_path: "domain/".to_string(),
            surface_path: "surface/".to_string(),
            connector_path: "connectors/".to_string(),
            required_capabilities: vec!["provider.codex.launch".to_string()],
            owned_object_types: vec!["Issue".to_string()],
            exposed_commands: vec!["work.start".to_string()],
            projection_entries: vec!["task-workbench".to_string()],
            migration_policy: PackMigrationPolicy::PreviewOnly,
            validation_status: PackValidationStatus::Draft,
        }
    }

    #[test]
    fn manifest_validation_rejects_missing_required_fields() {
        let mut manifest = manifest("");
        manifest.domain_path = "../domain".to_string();

        let report = validate_pack_manifest(&manifest);

        assert!(!report.valid);
        assert!(report.issues.iter().any(|issue| issue.field == "packId"));
        assert!(report
            .issues
            .iter()
            .any(|issue| issue.field == "domainPath"));
    }

    #[test]
    fn registry_lists_local_pack_manifests_without_authority_writes() {
        let dir = tempfile::tempdir().unwrap();
        let pack_dir = dir.path().join(".agentflow/packs/software-dev");
        std::fs::create_dir_all(&pack_dir).unwrap();
        std::fs::write(
            pack_dir.join("pack.json"),
            serde_json::to_string_pretty(&manifest("software-dev")).unwrap(),
        )
        .unwrap();

        let registry = load_pack_registry(dir.path()).unwrap();

        assert!(!registry.writes_authority);
        assert_eq!(registry.entries.len(), 1);
        let entry = registry.pack("software-dev").unwrap();
        assert_eq!(
            entry.manifest_path,
            normalize_for_test(pack_dir.join("pack.json"))
        );
        assert!(entry.validation.valid);
        assert_eq!(entry.required_capabilities, vec!["provider.codex.launch"]);
    }

    fn normalize_for_test(path: std::path::PathBuf) -> String {
        path.to_string_lossy().replace('\\', "/")
    }
}
