use crate::model::{
    DeploymentArtifactRef, DeploymentEvidenceReport, DeploymentSemanticCheck,
    DeploymentShapeEvidence, ProjectReleaseFacts, RemoteReleaseProof, RollbackModel,
    DEPLOYMENT_EVIDENCE_REPORT_VERSION, PROJECT_RELEASE_FACTS_VERSION,
    REMOTE_RELEASE_PROOF_VERSION,
};
use anyhow::{Context, Result};
use serde_json::Value;
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
    let release_facts = artifact_ref("release-facts", input.release_facts_path.clone())?;
    let remote_release = artifact_ref(
        "remote-release-proof",
        input.remote_release_proof_path.clone(),
    )?;
    let pack_version_fingerprint = artifact_ref(
        "pack-version-fingerprint",
        input.pack_version_fingerprint_path.clone(),
    )?;
    let event_store_fingerprint = artifact_ref(
        "event-store-fingerprint",
        input.event_store_fingerprint_path.clone(),
    )?;
    let projection_rebuild_proof = artifact_ref(
        "projection-rebuild-proof",
        input.projection_rebuild_proof_path.clone(),
    )?;
    let migration_receipt =
        artifact_ref("migration-receipt", input.migration_receipt_path.clone())?;
    let rollback_receipt = artifact_ref("rollback-receipt", input.rollback_receipt_path.clone())?;
    let config_fingerprint = input
        .config_fingerprint_path
        .as_ref()
        .cloned()
        .map(|path| artifact_ref("config-fingerprint", path))
        .transpose()?;
    let failed_deployment_report = input
        .failed_deployment_report_path
        .as_ref()
        .cloned()
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

    let semantic_checks = deployment_semantic_checks(
        &input,
        release_facts.exists,
        &release_facts.path,
        remote_release.exists,
        &remote_release.path,
        &pack_version_fingerprint,
        &event_store_fingerprint,
        &projection_rebuild_proof,
        &migration_receipt,
        &rollback_receipt,
        failed_deployment_report.as_ref(),
    )?;
    let semantic_failures = semantic_checks
        .iter()
        .filter(|check| check.status != "passed")
        .map(|check| check.check_id.clone())
        .collect::<Vec<_>>();

    let status = if missing_evidence.is_empty() && semantic_failures.is_empty() {
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
            status: shape_status(
                &local_evidence,
                &semantic_failures,
                &[
                    "pack-registry",
                    "event-replay-report",
                    "projection-rebuild-proof",
                    "migration-receipt",
                ],
            ),
            evidence: local_evidence,
        },
        cloud_deployment: DeploymentShapeEvidence {
            shape: "cloud".to_string(),
            status: shape_status(
                &cloud_evidence,
                &semantic_failures,
                &["release-facts", "remote-release-proof", "artifact-manifest"],
            ),
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
        semantic_checks,
        semantic_failures,
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

fn shape_status(
    evidence: &[DeploymentArtifactRef],
    semantic_failures: &[String],
    relevant_check_ids: &[&str],
) -> String {
    let has_relevant_semantic_failure = semantic_failures.iter().any(|failure| {
        relevant_check_ids
            .iter()
            .any(|prefix| failure.starts_with(prefix))
    });
    if evidence.iter().all(|artifact| artifact.exists) && !has_relevant_semantic_failure {
        "ready".to_string()
    } else {
        "missing-evidence".to_string()
    }
}

fn deployment_semantic_checks(
    input: &DeploymentEvidenceInput,
    release_facts_exists: bool,
    release_facts_path: &str,
    remote_release_exists: bool,
    remote_release_path: &str,
    pack_version_fingerprint: &DeploymentArtifactRef,
    event_store_fingerprint: &DeploymentArtifactRef,
    projection_rebuild_proof: &DeploymentArtifactRef,
    migration_receipt: &DeploymentArtifactRef,
    rollback_receipt: &DeploymentArtifactRef,
    failed_deployment_report: Option<&DeploymentArtifactRef>,
) -> Result<Vec<DeploymentSemanticCheck>> {
    let mut checks = Vec::new();
    if !release_facts_exists {
        check(
            &mut checks,
            "release-facts.exists",
            false,
            "release facts artifact is missing",
        );
        return Ok(checks);
    }
    if !remote_release_exists {
        check(
            &mut checks,
            "remote-release-proof.exists",
            false,
            "remote release proof artifact is missing",
        );
        return Ok(checks);
    }

    let release_facts: ProjectReleaseFacts = read_json_path(Path::new(release_facts_path))?;
    let remote_release: RemoteReleaseProof = read_json_path(Path::new(remote_release_path))?;

    check_eq(
        &mut checks,
        "release-facts.version",
        release_facts.version.as_str(),
        PROJECT_RELEASE_FACTS_VERSION,
    );
    check_eq(
        &mut checks,
        "remote-release-proof.version",
        remote_release.version.as_str(),
        REMOTE_RELEASE_PROOF_VERSION,
    );
    check_eq(
        &mut checks,
        "release-facts.tag",
        release_facts.tag_name.as_deref().unwrap_or_default(),
        input.release_tag.as_str(),
    );
    check_eq(
        &mut checks,
        "release-facts.commit",
        release_facts.tag_commit_sha.as_deref().unwrap_or_default(),
        input.source_commit_sha.as_str(),
    );
    check_eq(
        &mut checks,
        "remote-release-proof.tag",
        remote_release.tag_name.as_str(),
        input.release_tag.as_str(),
    );
    check_eq(
        &mut checks,
        "remote-release-proof.commit",
        remote_release.release_commit_sha.as_str(),
        input.source_commit_sha.as_str(),
    );
    check_eq(
        &mut checks,
        "remote-release-proof.provider",
        remote_release.provider.as_str(),
        release_facts.remote_provider.as_deref().unwrap_or_default(),
    );
    check_eq(
        &mut checks,
        "remote-release-proof.release-id",
        remote_release.release_id.as_str(),
        release_facts
            .remote_release_id
            .as_deref()
            .unwrap_or_default(),
    );
    check_eq(
        &mut checks,
        "remote-release-proof.url",
        remote_release.release_url.as_str(),
        release_facts
            .remote_release_url
            .as_deref()
            .unwrap_or_default(),
    );
    check_eq(
        &mut checks,
        "remote-release-proof.commit-vs-facts",
        remote_release.release_commit_sha.as_str(),
        release_facts
            .remote_release_commit_sha
            .as_deref()
            .unwrap_or_default(),
    );

    let facts_manifest_path = release_facts
        .artifact_manifest_path
        .as_deref()
        .unwrap_or_default();
    let proof_manifest_path = remote_release
        .artifact_manifest_path
        .as_deref()
        .unwrap_or_default();
    let facts_manifest_sha = release_facts
        .artifact_manifest_sha256
        .as_deref()
        .unwrap_or_default();
    let proof_manifest_sha = remote_release
        .artifact_manifest_sha256
        .as_deref()
        .unwrap_or_default();
    check(
        &mut checks,
        "artifact-manifest.path-present",
        !facts_manifest_path.is_empty() && !proof_manifest_path.is_empty(),
        "artifact manifest path must be present in release facts and remote proof",
    );
    check_eq(
        &mut checks,
        "artifact-manifest.path",
        proof_manifest_path,
        facts_manifest_path,
    );
    check(
        &mut checks,
        "artifact-manifest.sha-present",
        !facts_manifest_sha.is_empty() && !proof_manifest_sha.is_empty(),
        "artifact manifest sha256 must be present in release facts and remote proof",
    );
    check_eq(
        &mut checks,
        "artifact-manifest.sha256",
        proof_manifest_sha,
        facts_manifest_sha,
    );

    let target_tag = input
        .rollback_target_tag
        .as_deref()
        .unwrap_or(input.release_tag.as_str());
    let target_commit = input
        .rollback_target_commit_sha
        .as_deref()
        .unwrap_or(input.source_commit_sha.as_str());
    check_eq(
        &mut checks,
        "rollback.target-tag",
        target_tag,
        input.release_tag.as_str(),
    );
    check_eq(
        &mut checks,
        "rollback.target-commit",
        target_commit,
        input.source_commit_sha.as_str(),
    );

    check_json_version(
        &mut checks,
        "pack-registry",
        pack_version_fingerprint,
        "agentflow-pack-registry.v1",
    )?;
    check_json_status(
        &mut checks,
        "event-replay-report",
        event_store_fingerprint,
        "projection-replay-report.v1",
        "passed",
    )?;
    check_json_status(
        &mut checks,
        "projection-rebuild-proof",
        projection_rebuild_proof,
        "projection-replay-report.v1",
        "passed",
    )?;
    check_json_bool(
        &mut checks,
        "migration-receipt",
        migration_receipt,
        "agentflow-pack-migration-applied-receipt.v1",
        "applied",
        true,
    )?;
    check_json_bool(
        &mut checks,
        "rollback-receipt",
        rollback_receipt,
        "agentflow-pack-migration-rollback-receipt.v1",
        "rolledBack",
        true,
    )?;
    if let Some(report) = failed_deployment_report {
        check_json_status(
            &mut checks,
            "failed-deployment-report",
            report,
            "projection-replay-report.v1",
            "failed",
        )?;
    }

    Ok(checks)
}

fn check(checks: &mut Vec<DeploymentSemanticCheck>, check_id: &str, passed: bool, reason: &str) {
    checks.push(DeploymentSemanticCheck {
        check_id: check_id.to_string(),
        status: if passed { "passed" } else { "failed" }.to_string(),
        reason: reason.to_string(),
    });
}

fn check_eq(
    checks: &mut Vec<DeploymentSemanticCheck>,
    check_id: &str,
    actual: &str,
    expected: &str,
) {
    check(
        checks,
        check_id,
        actual == expected && !actual.is_empty() && !expected.is_empty(),
        &format!("expected `{expected}`, got `{actual}`"),
    );
}

fn check_json_version(
    checks: &mut Vec<DeploymentSemanticCheck>,
    check_id: &str,
    artifact: &DeploymentArtifactRef,
    expected_version: &str,
) -> Result<()> {
    if !artifact.exists {
        check(
            checks,
            &format!("{check_id}.exists"),
            false,
            "semantic artifact is missing",
        );
        return Ok(());
    }
    let value = read_artifact_value(artifact)?;
    check_eq(
        checks,
        &format!("{check_id}.version"),
        value
            .get("version")
            .and_then(Value::as_str)
            .unwrap_or_default(),
        expected_version,
    );
    Ok(())
}

fn check_json_status(
    checks: &mut Vec<DeploymentSemanticCheck>,
    check_id: &str,
    artifact: &DeploymentArtifactRef,
    expected_version: &str,
    expected_status: &str,
) -> Result<()> {
    if !artifact.exists {
        check(
            checks,
            &format!("{check_id}.exists"),
            false,
            "semantic artifact is missing",
        );
        return Ok(());
    }
    let value = read_artifact_value(artifact)?;
    check_eq(
        checks,
        &format!("{check_id}.version"),
        value
            .get("version")
            .and_then(Value::as_str)
            .unwrap_or_default(),
        expected_version,
    );
    check_eq(
        checks,
        &format!("{check_id}.status"),
        value
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or_default(),
        expected_status,
    );
    Ok(())
}

fn check_json_bool(
    checks: &mut Vec<DeploymentSemanticCheck>,
    check_id: &str,
    artifact: &DeploymentArtifactRef,
    expected_version: &str,
    field: &str,
    expected: bool,
) -> Result<()> {
    if !artifact.exists {
        check(
            checks,
            &format!("{check_id}.exists"),
            false,
            "semantic artifact is missing",
        );
        return Ok(());
    }
    let value = read_artifact_value(artifact)?;
    check_eq(
        checks,
        &format!("{check_id}.version"),
        value
            .get("version")
            .and_then(Value::as_str)
            .unwrap_or_default(),
        expected_version,
    );
    check(
        checks,
        &format!("{check_id}.{field}"),
        value.get(field).and_then(Value::as_bool) == Some(expected),
        &format!("expected `{field}` to be `{expected}`"),
    );
    Ok(())
}

fn read_artifact_value(artifact: &DeploymentArtifactRef) -> Result<Value> {
    read_json_path(Path::new(&artifact.path))
}

fn read_json_path<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))
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
        let fixtures = write_semantic_fixtures(dir.path());

        let report = build_deployment_evidence_report(DeploymentEvidenceInput {
            release_version: "v0.9.0".to_string(),
            release_tag: "v0.9.0".to_string(),
            source_commit_sha: "abc123".to_string(),
            runtime_version: "0.9.0".to_string(),
            release_facts_path: fixtures.release_facts,
            remote_release_proof_path: fixtures.remote_release,
            config_fingerprint_path: None,
            pack_version_fingerprint_path: fixtures.pack,
            event_store_fingerprint_path: fixtures.replay.clone(),
            projection_rebuild_proof_path: fixtures.replay,
            migration_receipt_path: fixtures.migration,
            rollback_receipt_path: fixtures.rollback,
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
        assert!(report.semantic_failures.is_empty());
        assert!(report
            .semantic_checks
            .iter()
            .any(|check| check.check_id == "artifact-manifest.sha256"));
    }

    #[test]
    fn deployment_evidence_fails_when_rollback_receipt_is_missing() {
        let dir = tempdir().unwrap();
        let fixtures = write_semantic_fixtures(dir.path());

        let report = build_deployment_evidence_report(DeploymentEvidenceInput {
            release_version: "v0.9.0".to_string(),
            release_tag: "v0.9.0".to_string(),
            source_commit_sha: "abc123".to_string(),
            runtime_version: "0.9.0".to_string(),
            release_facts_path: fixtures.release_facts,
            remote_release_proof_path: fixtures.remote_release,
            config_fingerprint_path: None,
            pack_version_fingerprint_path: fixtures.pack,
            event_store_fingerprint_path: fixtures.replay.clone(),
            projection_rebuild_proof_path: fixtures.replay,
            migration_receipt_path: fixtures.migration,
            rollback_receipt_path: dir.path().join("missing-rollback.json"),
            failed_deployment_report_path: None,
            rollback_target_tag: None,
            rollback_target_commit_sha: None,
        })
        .unwrap();

        assert_eq!(report.status, "failed");
        assert_eq!(report.missing_evidence, vec!["rollback-receipt"]);
    }

    #[test]
    fn deployment_evidence_fails_when_remote_tag_does_not_match_release_tag() {
        let dir = tempdir().unwrap();
        let fixtures = write_semantic_fixtures_with(dir.path(), |facts, remote| {
            facts["tagName"] = serde_json::json!("v0.9.0");
            remote["tagName"] = serde_json::json!("v0.9.1");
        });

        let report = report_from_fixtures(fixtures, "v0.9.0", "abc123", None, None);
        assert_eq!(report.status, "failed");
        assert!(report
            .semantic_failures
            .contains(&"remote-release-proof.tag".to_string()));
        assert_eq!(report.cloud_deployment.status, "missing-evidence");
    }

    #[test]
    fn deployment_evidence_fails_when_artifact_manifest_sha_is_missing() {
        let dir = tempdir().unwrap();
        let fixtures = write_semantic_fixtures_with(dir.path(), |facts, remote| {
            facts["artifactManifestSha256"] = serde_json::Value::Null;
            remote["artifactManifestSha256"] = serde_json::Value::Null;
        });

        let report = report_from_fixtures(fixtures, "v0.9.0", "abc123", None, None);
        assert_eq!(report.status, "failed");
        assert!(report
            .semantic_failures
            .contains(&"artifact-manifest.sha-present".to_string()));
    }

    #[test]
    fn deployment_evidence_fails_when_rollback_target_does_not_match_release() {
        let dir = tempdir().unwrap();
        let fixtures = write_semantic_fixtures(dir.path());

        let report = report_from_fixtures(
            fixtures,
            "v0.9.0",
            "abc123",
            Some("v0.8.0".to_string()),
            Some("abc123".to_string()),
        );
        assert_eq!(report.status, "failed");
        assert!(report
            .semantic_failures
            .contains(&"rollback.target-tag".to_string()));
    }

    struct SemanticFixtures {
        release_facts: PathBuf,
        remote_release: PathBuf,
        pack: PathBuf,
        replay: PathBuf,
        migration: PathBuf,
        rollback: PathBuf,
    }

    fn report_from_fixtures(
        fixtures: SemanticFixtures,
        release_tag: &str,
        source_commit_sha: &str,
        rollback_target_tag: Option<String>,
        rollback_target_commit_sha: Option<String>,
    ) -> DeploymentEvidenceReport {
        build_deployment_evidence_report(DeploymentEvidenceInput {
            release_version: "v0.9.0".to_string(),
            release_tag: release_tag.to_string(),
            source_commit_sha: source_commit_sha.to_string(),
            runtime_version: "0.9.0".to_string(),
            release_facts_path: fixtures.release_facts,
            remote_release_proof_path: fixtures.remote_release,
            config_fingerprint_path: None,
            pack_version_fingerprint_path: fixtures.pack,
            event_store_fingerprint_path: fixtures.replay.clone(),
            projection_rebuild_proof_path: fixtures.replay,
            migration_receipt_path: fixtures.migration,
            rollback_receipt_path: fixtures.rollback,
            failed_deployment_report_path: None,
            rollback_target_tag,
            rollback_target_commit_sha,
        })
        .unwrap()
    }

    fn write_semantic_fixtures(root: &Path) -> SemanticFixtures {
        write_semantic_fixtures_with(root, |_facts, _remote| {})
    }

    fn write_semantic_fixtures_with(
        root: &Path,
        mutate_release: impl FnOnce(&mut serde_json::Value, &mut serde_json::Value),
    ) -> SemanticFixtures {
        let mut facts = serde_json::json!({
            "version": PROJECT_RELEASE_FACTS_VERSION,
            "projectId": "project-release",
            "projectTitle": "Project Release",
            "currentState": "published",
            "publicationStage": "remote-release-created",
            "gateStatus": "ready",
            "gateReason": "ready",
            "completionState": "accepted",
            "completionOutcome": "accept",
            "deliveryStatus": "published",
            "changelogPath": "CHANGELOG.md",
            "releaseNotesPath": "docs/release-notes/project-release.md",
            "entryCount": 1,
            "summaryLine": "ready",
            "tagName": "v0.9.0",
            "tagCommitSha": "abc123",
            "remoteProvider": "github",
            "remoteReleaseId": "rel-001",
            "remoteReleaseUrl": "https://github.com/acme/repo/releases/tag/v0.9.0",
            "remoteReleaseCommitSha": "abc123",
            "artifactManifestPath": "artifacts/project-release-manifest.json",
            "artifactManifestSha256": "sha-123",
            "latestEventId": null,
            "publishedAt": 1,
            "updatedAt": 1
        });
        let mut remote = serde_json::json!({
            "version": REMOTE_RELEASE_PROOF_VERSION,
            "projectId": "project-release",
            "provider": "github",
            "releaseId": "rel-001",
            "releaseUrl": "https://github.com/acme/repo/releases/tag/v0.9.0",
            "tagName": "v0.9.0",
            "releaseCommitSha": "abc123",
            "artifactManifestPath": "artifacts/project-release-manifest.json",
            "artifactManifestSha256": "sha-123",
            "actor": "release-gate",
            "recordedAt": 1
        });
        mutate_release(&mut facts, &mut remote);

        SemanticFixtures {
            release_facts: write_json_fixture(root, "release-facts.json", &facts),
            remote_release: write_json_fixture(root, "remote-release.json", &remote),
            pack: write_json_fixture(
                root,
                "pack.json",
                &serde_json::json!({
                    "version": "agentflow-pack-registry.v1",
                    "writesAuthority": false,
                    "fallback": false
                }),
            ),
            replay: write_json_fixture(
                root,
                "replay.json",
                &serde_json::json!({
                    "version": "projection-replay-report.v1",
                    "status": "passed",
                    "writesAuthority": false,
                    "projectionAuthority": false
                }),
            ),
            migration: write_json_fixture(
                root,
                "migration.json",
                &serde_json::json!({
                    "version": "agentflow-pack-migration-applied-receipt.v1",
                    "applied": true,
                    "writesAuthority": true
                }),
            ),
            rollback: write_json_fixture(
                root,
                "rollback.json",
                &serde_json::json!({
                    "version": "agentflow-pack-migration-rollback-receipt.v1",
                    "rolledBack": true,
                    "writesAuthority": true
                }),
            ),
        }
    }

    fn write_json_fixture(root: &Path, name: &str, value: &serde_json::Value) -> PathBuf {
        let path = root.join(name);
        fs::write(&path, serde_json::to_string_pretty(value).unwrap() + "\n").unwrap();
        path
    }
}
