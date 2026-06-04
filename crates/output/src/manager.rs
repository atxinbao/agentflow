use crate::{
    audit::{ensure_audit_workspace, rebuild_audit_manifest_and_index},
    model::{
        HumanAudit, OutputEvidence, OutputIndex, OutputIndexEntry, OutputManifest,
        OutputReleaseDelivery, OutputSnapshot, OutputStatusSnapshot, OutputSummary,
        OutputWorkspaceStatus, OUTPUT_DIRECTORIES, OUTPUT_REQUIRED_FILES, OUTPUT_SNAPSHOT_VERSION,
        OUTPUT_STATUS_VERSION,
    },
    storage::{
        canonical_project_root, count_directory_entries, ensure_directory, read_json,
        sorted_child_paths, unix_timestamp_seconds, write_json,
    },
    validate::{validate_output_evidence, validate_release_delivery},
};
use anyhow::Result;
use std::path::Path;

pub fn prepare_output_workspace(project_root: impl AsRef<Path>) -> Result<OutputSnapshot> {
    let root = canonical_project_root(project_root)?;

    for relative_path in OUTPUT_DIRECTORIES {
        ensure_directory(&root.join(relative_path))?;
    }
    ensure_audit_workspace(&root)?;
    rebuild_audit_manifest_and_index(&root)?;

    let index = rebuild_output_index(&root)?;
    let summary = output_summary(&root, &index)?;
    let manifest = OutputManifest::new(
        root.display().to_string(),
        summary,
        unix_timestamp_seconds(),
    );
    write_json(&root.join(".agentflow/output/manifest.json"), &manifest)?;
    build_output_snapshot(&root)
}

pub fn validate_output(project_root: impl AsRef<Path>) -> Result<OutputSnapshot> {
    let root = canonical_project_root(project_root)?;
    build_output_snapshot(&root)
}

pub fn load_output_status(project_root: impl AsRef<Path>) -> Result<OutputStatusSnapshot> {
    Ok(validate_output(project_root)?.status)
}

pub fn load_output_manifest(project_root: impl AsRef<Path>) -> Result<OutputManifest> {
    let root = canonical_project_root(project_root)?;
    read_json(&root.join(".agentflow/output/manifest.json"))
}

pub fn load_output_index(project_root: impl AsRef<Path>) -> Result<OutputIndex> {
    let root = canonical_project_root(project_root)?;
    read_json(&root.join(".agentflow/output/index.json"))
}

pub fn load_output_snapshot(project_root: impl AsRef<Path>) -> Result<OutputSnapshot> {
    validate_output(project_root)
}

pub(crate) fn rebuild_output_index(root: &Path) -> Result<OutputIndex> {
    let updated_at = unix_timestamp_seconds();
    let mut evidence = Vec::new();
    for path in sorted_child_paths(&root.join(".agentflow/output/evidence"))? {
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let Ok(record) = read_json::<OutputEvidence>(&path) else {
            continue;
        };
        evidence.push(OutputIndexEntry {
            run_id: record.run_id.clone(),
            issue_id: record.issue_id.clone(),
            source_spec_id: record.source_spec_id.clone(),
            path: format!(".agentflow/output/evidence/{}.json", record.run_id),
            status: evidence_status(&record),
            updated_at: record.completed_at,
        });
    }

    let mut release_deliveries = Vec::new();
    for path in sorted_child_paths(&root.join(".agentflow/output/release"))? {
        let delivery_path = path.join("delivery.json");
        if !delivery_path.is_file() {
            continue;
        }
        let Ok(record) = read_json::<OutputReleaseDelivery>(&delivery_path) else {
            continue;
        };
        release_deliveries.push(OutputIndexEntry {
            run_id: record.run_id.clone(),
            issue_id: record.issue_id.clone(),
            source_spec_id: record.source_spec_id.clone(),
            path: format!(".agentflow/output/release/{}/delivery.json", record.run_id),
            status: record.status.clone(),
            updated_at: record.created_at,
        });
    }

    let mut audits = Vec::new();
    for path in sorted_child_paths(&root.join(".agentflow/output/audit"))? {
        let audit_path = path.join("audit.json");
        if !audit_path.is_file() {
            continue;
        }
        let Ok(record) = read_json::<HumanAudit>(&audit_path) else {
            continue;
        };
        let issue_id = audit_scope_id(root, &record.audit_id, "issue").unwrap_or_default();
        let source_spec_id = audit_scope_id(root, &record.audit_id, "spec").unwrap_or_default();
        audits.push(OutputIndexEntry {
            run_id: record.audit_id.clone(),
            issue_id,
            source_spec_id,
            path: format!(".agentflow/output/audit/{}/audit.json", record.audit_id),
            status: record.status.as_str().to_string(),
            updated_at: record.requested_at,
        });
    }

    let index = OutputIndex {
        version: crate::model::OUTPUT_INDEX_VERSION.to_string(),
        updated_at,
        evidence,
        release_deliveries,
        audits,
    };
    write_json(&root.join(".agentflow/output/index.json"), &index)?;
    Ok(index)
}

fn build_output_snapshot(root: &Path) -> Result<OutputSnapshot> {
    let manifest_exists = root.join(".agentflow/output/manifest.json").is_file();
    let index_exists = root.join(".agentflow/output/index.json").is_file();
    let missing_paths = missing_output_paths(root);
    let index = if index_exists {
        read_json(&root.join(".agentflow/output/index.json")).unwrap_or_default()
    } else {
        OutputIndex::default()
    };
    let summary = output_summary(root, &index)?;
    let mut warnings = Vec::new();
    let mut errors = Vec::new();

    if !missing_paths.is_empty() {
        errors.push(format!(
            "missing output paths: {}",
            missing_paths.join(", ")
        ));
    }

    for entry in &index.evidence {
        let validation = validate_output_evidence(root, &entry.run_id)?;
        if !validation.valid {
            errors.extend(validation.errors);
            warnings.extend(validation.warnings);
        }
    }

    for entry in &index.release_deliveries {
        let validation = validate_release_delivery(root, &entry.run_id)?;
        if !validation.valid {
            errors.extend(validation.errors);
            warnings.extend(validation.warnings);
        }
    }

    for path in sorted_child_paths(&root.join(".agentflow/output/audit"))? {
        if !path.is_dir() {
            continue;
        }
        let name = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("");
        if name == "manifest.json" || name == "index.json" {
            continue;
        }
        let has_audit_json = path.join("audit.json").is_file();
        let has_audit_report = path.join("audit-report.md").is_file();
        if !has_audit_json && !has_audit_report {
            errors.push(format!(
                "audit output {} must contain audit.json or audit-report.md",
                path.display()
            ));
        }
    }

    let ready = errors.is_empty();
    let status_value = if ready {
        OutputWorkspaceStatus::Ready
    } else if manifest_exists || index_exists {
        OutputWorkspaceStatus::Degraded
    } else {
        OutputWorkspaceStatus::Missing
    };
    let status = OutputStatusSnapshot {
        version: OUTPUT_STATUS_VERSION.to_string(),
        project_root: root.display().to_string(),
        status: status_value,
        ready,
        manifest_exists,
        index_exists,
        summary: summary.clone(),
        missing_paths,
        warnings,
        errors,
    };
    let manifest = if manifest_exists {
        read_json(&root.join(".agentflow/output/manifest.json"))
            .unwrap_or_else(|_| OutputManifest::new(root.display().to_string(), summary.clone(), 0))
    } else {
        OutputManifest::new(root.display().to_string(), summary.clone(), 0)
    };

    Ok(OutputSnapshot {
        version: OUTPUT_SNAPSHOT_VERSION.to_string(),
        project_root: root.display().to_string(),
        ready,
        status,
        manifest,
        index,
    })
}

fn audit_scope_id(root: &Path, audit_id: &str, kind: &str) -> Option<String> {
    let request_path = root
        .join(".agentflow/output/audit")
        .join(audit_id)
        .join("audit-request.json");
    let request: crate::model::AuditRequest = read_json(&request_path).ok()?;
    request
        .scope
        .refs
        .into_iter()
        .find(|reference| reference.kind == kind)
        .map(|reference| reference.id)
}

fn output_summary(root: &Path, index: &OutputIndex) -> Result<OutputSummary> {
    let incomplete_evidence = index
        .evidence
        .iter()
        .map(|entry| validate_output_evidence(root, &entry.run_id))
        .filter(|result| result.as_ref().map(|value| !value.valid).unwrap_or(true))
        .count();
    let incomplete_deliveries = index
        .release_deliveries
        .iter()
        .map(|entry| validate_release_delivery(root, &entry.run_id))
        .filter(|result| result.as_ref().map(|value| !value.valid).unwrap_or(true))
        .count();

    Ok(OutputSummary {
        evidence: index.evidence.len(),
        release_deliveries: index.release_deliveries.len(),
        audits: index.audits.len(),
        logs: count_directory_entries(&root.join(".agentflow/output/logs")),
        backups: count_directory_entries(&root.join(".agentflow/output/backup")),
        incomplete_evidence,
        incomplete_deliveries,
    })
}

fn evidence_status(evidence: &OutputEvidence) -> String {
    if evidence.validation.passed {
        "ready".to_string()
    } else {
        "incomplete".to_string()
    }
}

fn missing_output_paths(root: &Path) -> Vec<String> {
    OUTPUT_DIRECTORIES
        .iter()
        .copied()
        .chain(OUTPUT_REQUIRED_FILES.iter().copied())
        .filter(|relative_path| !root.join(relative_path).exists())
        .map(str::to_string)
        .collect()
}
