use crate::model::{
    DeploymentArtifactRef, DeploymentEvidenceReport, DeploymentShapeEvidence, RollbackModel,
    DEPLOYMENT_EVIDENCE_REPORT_VERSION,
};
use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeploymentEvidenceInput {
    pub release_version: String,
    pub release_tag: String,
    pub source_commit_sha: String,
    pub runtime_version: String,
    pub release_facts_path: PathBuf,
    pub remote_release_proof_path: PathBuf,
    pub config_fingerprint_path: Option<PathBuf>,
    pub pack_version_fingerprint_path: PathBuf,
    pub event_store_fingerprint_path: PathBuf,
    pub projection_rebuild_proof_path: PathBuf,
    pub migration_receipt_path: PathBuf,
    pub rollback_receipt_path: PathBuf,
    pub failed_deployment_report_path: Option<PathBuf>,
    pub rollback_target_tag: Option<String>,
    pub rollback_target_commit_sha: Option<String>,
}

pub fn build_deployment_evidence_report(
    input: DeploymentEvidenceInput,
) -> Result<DeploymentEvidenceReport> {
    let release_facts = artifact_ref("release-facts", input.release_facts_path)?;
    let remote_release = artifact_ref("remote-release-proof", input.remote_release_proof_path)?;
    let pack_version_fingerprint = artifact_ref(
        "pack-version-fingerprint",
        input.pack_version_fingerprint_path,
    )?;
    let event_store_fingerprint = artifact_ref(
        "event-store-fingerprint",
        input.event_store_fingerprint_path,
    )?;
    let projection_rebuild_proof = artifact_ref(
        "projection-rebuild-proof",
        input.projection_rebuild_proof_path,
    )?;
    let migration_receipt = artifact_ref("migration-receipt", input.migration_receipt_path)?;
    let rollback_receipt = artifact_ref("rollback-receipt", input.rollback_receipt_path)?;
    let config_fingerprint = input
        .config_fingerprint_path
        .map(|path| artifact_ref("config-fingerprint", path))
        .transpose()?;
    let failed_deployment_report = input
        .failed_deployment_report_path
        .map(|path| artifact_ref("failed-deployment-report", path))
        .transpose()?;

    let local_evidence = vec![
        release_facts.clone(),
        pack_version_fingerprint.clone(),
        event_store_fingerprint.clone(),
        projection_rebuild_proof.clone(),
        migration_receipt.clone(),
    ];
    let cloud_evidence = vec![remote_release.clone(), release_facts.clone()];

    let mut missing_evidence = Vec::new();
    for artifact in local_evidence
        .iter()
        .chain(cloud_evidence.iter())
        .chain(std::iter::once(&rollback_receipt))
    {
        if !artifact.exists {
            missing_evidence.push(artifact.label.clone());
        }
    }
    if config_fingerprint
        .as_ref()
        .is_some_and(|artifact| !artifact.exists)
    {
        missing_evidence.push("config-fingerprint".to_string());
    }
    if failed_deployment_report
        .as_ref()
        .is_some_and(|artifact| !artifact.exists)
    {
        missing_evidence.push("failed-deployment-report".to_string());
    }

    let status = if missing_evidence.is_empty() {
        "passed"
    } else {
        "failed"
    }
    .to_string();

    let target_tag = input
        .rollback_target_tag
        .unwrap_or_else(|| input.release_tag.clone());
    let target_commit_sha = input
        .rollback_target_commit_sha
        .unwrap_or_else(|| input.source_commit_sha.clone());

    Ok(DeploymentEvidenceReport {
        version: DEPLOYMENT_EVIDENCE_REPORT_VERSION.to_string(),
        status,
        release_version: input.release_version,
        release_tag: input.release_tag,
        source_commit_sha: input.source_commit_sha,
        runtime_version: input.runtime_version,
        local_deployment: DeploymentShapeEvidence {
            shape: "local".to_string(),
            status: shape_status(&local_evidence),
            evidence: local_evidence,
        },
        cloud_deployment: DeploymentShapeEvidence {
            shape: "cloud".to_string(),
            status: shape_status(&cloud_evidence),
            evidence: cloud_evidence,
        },
        release_facts,
        config_fingerprint,
        pack_version_fingerprint,
        event_store_fingerprint,
        projection_rebuild_proof,
        migration_receipt,
        rollback_model: RollbackModel {
            provider_agnostic: true,
            target_tag,
            target_commit_sha,
            rollback_receipt,
            failed_deployment_report,
            requires_human_confirmation: true,
            summary:
                "Rollback 只依赖 release tag / commit / migration rollback receipt，不绑定云厂商。"
                    .to_string(),
        },
        writes_authority: false,
        missing_evidence,
        generated_at: unix_timestamp_seconds(),
    })
}

fn artifact_ref(label: impl Into<String>, path: PathBuf) -> Result<DeploymentArtifactRef> {
    let exists = path.is_file();
    let sha256 = if exists {
        Some(file_sha256(&path)?)
    } else {
        None
    };
    Ok(DeploymentArtifactRef {
        label: label.into(),
        path: path.to_string_lossy().to_string(),
        exists,
        sha256,
    })
}

fn file_sha256(path: &Path) -> Result<String> {
    let bytes = fs::read(path).with_context(|| format!("read {}", path.display()))?;
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    Ok(format!("{:x}", hasher.finalize()))
}

fn shape_status(evidence: &[DeploymentArtifactRef]) -> String {
    if evidence.iter().all(|artifact| artifact.exists) {
        "ready".to_string()
    } else {
        "missing-evidence".to_string()
    }
}

fn unix_timestamp_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn deployment_evidence_passes_when_required_artifacts_exist() {
        let dir = tempdir().unwrap();
        let release_facts = write_fixture(dir.path(), "release-facts.json");
        let remote_release = write_fixture(dir.path(), "remote-release.json");
        let pack = write_fixture(dir.path(), "pack.json");
        let replay = write_fixture(dir.path(), "replay.json");
        let projection = write_fixture(dir.path(), "projection.json");
        let migration = write_fixture(dir.path(), "migration.json");
        let rollback = write_fixture(dir.path(), "rollback.json");

        let report = build_deployment_evidence_report(DeploymentEvidenceInput {
            release_version: "v0.9.0".to_string(),
            release_tag: "v0.9.0".to_string(),
            source_commit_sha: "abc123".to_string(),
            runtime_version: "0.9.0".to_string(),
            release_facts_path: release_facts,
            remote_release_proof_path: remote_release,
            config_fingerprint_path: None,
            pack_version_fingerprint_path: pack,
            event_store_fingerprint_path: replay,
            projection_rebuild_proof_path: projection,
            migration_receipt_path: migration,
            rollback_receipt_path: rollback,
            failed_deployment_report_path: None,
            rollback_target_tag: None,
            rollback_target_commit_sha: None,
        })
        .unwrap();

        assert_eq!(report.version, DEPLOYMENT_EVIDENCE_REPORT_VERSION);
        assert_eq!(report.status, "passed");
        assert_eq!(report.local_deployment.status, "ready");
        assert_eq!(report.cloud_deployment.status, "ready");
        assert!(!report.writes_authority);
        assert!(report.rollback_model.provider_agnostic);
        assert!(report.missing_evidence.is_empty());
    }

    #[test]
    fn deployment_evidence_fails_when_rollback_receipt_is_missing() {
        let dir = tempdir().unwrap();
        let release_facts = write_fixture(dir.path(), "release-facts.json");
        let remote_release = write_fixture(dir.path(), "remote-release.json");
        let pack = write_fixture(dir.path(), "pack.json");
        let replay = write_fixture(dir.path(), "replay.json");
        let projection = write_fixture(dir.path(), "projection.json");
        let migration = write_fixture(dir.path(), "migration.json");

        let report = build_deployment_evidence_report(DeploymentEvidenceInput {
            release_version: "v0.9.0".to_string(),
            release_tag: "v0.9.0".to_string(),
            source_commit_sha: "abc123".to_string(),
            runtime_version: "0.9.0".to_string(),
            release_facts_path: release_facts,
            remote_release_proof_path: remote_release,
            config_fingerprint_path: None,
            pack_version_fingerprint_path: pack,
            event_store_fingerprint_path: replay,
            projection_rebuild_proof_path: projection,
            migration_receipt_path: migration,
            rollback_receipt_path: dir.path().join("missing-rollback.json"),
            failed_deployment_report_path: None,
            rollback_target_tag: None,
            rollback_target_commit_sha: None,
        })
        .unwrap();

        assert_eq!(report.status, "failed");
        assert_eq!(report.missing_evidence, vec!["rollback-receipt"]);
    }

    fn write_fixture(root: &Path, name: &str) -> PathBuf {
        let path = root.join(name);
        fs::write(&path, format!("{{\"name\":\"{}\"}}\n", name)).unwrap();
        path
    }
}
