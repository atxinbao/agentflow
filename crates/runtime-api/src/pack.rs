use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

pub type PackRegistryView = agentflow_pack::PackRegistry;
pub type PackValidationArtifactView = agentflow_pack::PackValidationArtifact;

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

pub fn get_pack_registry(project_root: impl AsRef<Path>) -> Result<PackRegistryView> {
    agentflow_pack::load_pack_registry(project_root)
}

pub fn get_pack_validation_artifact(
    artifact_path: impl AsRef<Path>,
) -> Result<PackValidationArtifactView> {
    agentflow_pack::load_pack_validation_artifact(artifact_path)
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

#[cfg(test)]
mod tests {
    use super::{
        get_pack_registry, get_pack_validation_artifact, pack_registry_read_receipt,
        pack_validation_artifact_read_receipt,
    };

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
}
