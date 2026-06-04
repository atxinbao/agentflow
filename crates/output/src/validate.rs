use crate::{
    model::{
        OutputEvidence, OutputReleaseDelivery, OutputValidationReport, OUTPUT_EVIDENCE_VERSION,
        OUTPUT_RELEASE_DELIVERY_VERSION,
    },
    storage::{canonical_project_root, read_json},
};
use anyhow::Result;
use serde_json::Value;
use std::path::Path;

pub fn load_output_evidence(
    project_root: impl AsRef<Path>,
    run_id: String,
) -> Result<OutputEvidence> {
    let root = canonical_project_root(project_root)?;
    read_json(
        &root
            .join(".agentflow/output/evidence")
            .join(format!("{run_id}.json")),
    )
}

pub fn load_release_delivery(
    project_root: impl AsRef<Path>,
    run_id: String,
) -> Result<OutputReleaseDelivery> {
    let root = canonical_project_root(project_root)?;
    read_json(
        &root
            .join(".agentflow/output/release")
            .join(run_id)
            .join("delivery.json"),
    )
}

pub fn validate_output_evidence(
    project_root: impl AsRef<Path>,
    run_id: &str,
) -> Result<OutputValidationReport> {
    let root = canonical_project_root(project_root)?;
    let evidence_path = root
        .join(".agentflow/output/evidence")
        .join(format!("{run_id}.json"));
    let mut report = OutputValidationReport {
        valid: true,
        warnings: Vec::new(),
        errors: Vec::new(),
    };

    let evidence = match read_json::<OutputEvidence>(&evidence_path) {
        Ok(value) => value,
        Err(error) => {
            report.valid = false;
            report
                .errors
                .push(format!("evidence {run_id} unreadable: {error}"));
            return Ok(report);
        }
    };

    if evidence.version != OUTPUT_EVIDENCE_VERSION {
        report
            .warnings
            .push(format!("unexpected evidence version {}", evidence.version));
    }
    if evidence.run_id.is_empty()
        || evidence.issue_id.is_empty()
        || evidence.source_spec_id.is_empty()
    {
        report.valid = false;
        report
            .errors
            .push("evidence must include runId, issueId, and sourceSpecId".to_string());
    }
    if evidence.run_id != run_id {
        report.valid = false;
        report.errors.push(format!(
            "evidence runId mismatch: requested {run_id}, found {}",
            evidence.run_id
        ));
    }

    for (label, relative_path) in [
        ("run.json", Some(evidence.execute.run.as_str())),
        ("preflight.json", Some(evidence.execute.preflight.as_str())),
        ("result.json", Some(evidence.execute.result.as_str())),
        ("checkpoint", evidence.execute.checkpoint.as_deref()),
        ("diff", evidence.execute.diff.as_deref()),
        ("changed-files", evidence.execute.changed_files.as_deref()),
    ] {
        if let Some(path) = relative_path {
            if !root.join(path).is_file() {
                report.valid = false;
                report
                    .errors
                    .push(format!("evidence references missing {label}: {path}"));
            }
        }
    }

    for command in &evidence.commands {
        if !root.join(&command.record_path).is_file() {
            report.valid = false;
            report.errors.push(format!(
                "evidence references missing command record: {}",
                command.record_path
            ));
        }
    }

    let result_path = root.join(&evidence.execute.result);
    if result_path.is_file() {
        let result: Value = read_json(&result_path)?;
        let result_passed = result
            .get("validation")
            .and_then(|validation| validation.get("passed"))
            .and_then(Value::as_bool);
        if evidence.validation.passed && result_passed != Some(true) {
            report.valid = false;
            report.errors.push(
                "evidence validation.passed=true must match result.validation.passed=true"
                    .to_string(),
            );
        }
    }

    Ok(report)
}

pub fn validate_release_delivery(
    project_root: impl AsRef<Path>,
    run_id: &str,
) -> Result<OutputValidationReport> {
    let root = canonical_project_root(project_root)?;
    let delivery_path = root
        .join(".agentflow/output/release")
        .join(run_id)
        .join("delivery.json");
    let mut report = OutputValidationReport {
        valid: true,
        warnings: Vec::new(),
        errors: Vec::new(),
    };

    let delivery = match read_json::<OutputReleaseDelivery>(&delivery_path) {
        Ok(value) => value,
        Err(error) => {
            report.valid = false;
            report
                .errors
                .push(format!("release delivery {run_id} unreadable: {error}"));
            return Ok(report);
        }
    };

    if delivery.version != OUTPUT_RELEASE_DELIVERY_VERSION {
        report.warnings.push(format!(
            "unexpected release delivery version {}",
            delivery.version
        ));
    }
    if delivery.run_id != run_id {
        report.valid = false;
        report.errors.push(format!(
            "delivery runId mismatch: requested {run_id}, found {}",
            delivery.run_id
        ));
    }
    if delivery.created_by != "Build Agent" {
        report.valid = false;
        report
            .errors
            .push("delivery.createdBy must be Build Agent".to_string());
    }
    if !root.join(&delivery.evidence_path).is_file() {
        report.valid = false;
        report.errors.push(format!(
            "release delivery references missing evidence: {}",
            delivery.evidence_path
        ));
    }
    if !root.join(&delivery.execute_result_path).is_file() {
        report.valid = false;
        report.errors.push(format!(
            "release delivery references missing execute result: {}",
            delivery.execute_result_path
        ));
    }
    if let Some(path) = &delivery.diff_summary_path {
        if !root.join(path).is_file() {
            report.valid = false;
            report.errors.push(format!(
                "release delivery references missing diff summary: {path}"
            ));
        }
    }

    for (label, path) in [
        ("pr-draft.md", &delivery.artifacts.pr_draft),
        ("pr-metadata.json", &delivery.artifacts.pr_metadata),
        ("review-checklist.md", &delivery.artifacts.review_checklist),
        ("changelog.md", &delivery.artifacts.changelog),
        ("release-note.md", &delivery.artifacts.release_note),
    ] {
        if !root.join(path).is_file() {
            report.valid = false;
            report
                .errors
                .push(format!("release delivery missing {label}: {path}"));
        }
    }

    Ok(report)
}
