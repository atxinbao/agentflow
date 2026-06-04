use crate::{
    manager::rebuild_output_index,
    model::{OutputAudit, OutputAuditChecks, OUTPUT_AUDIT_VERSION},
    storage::{
        canonical_project_root, ensure_directory, read_json, unix_timestamp_seconds, write_json,
    },
};
use anyhow::{Context, Result};
use std::{fs, path::Path};

pub fn create_audit_skeleton(
    project_root: impl AsRef<Path>,
    run_id: String,
) -> Result<OutputAudit> {
    let root = canonical_project_root(project_root)?;
    let evidence_path = root
        .join(".agentflow/output/evidence")
        .join(format!("{run_id}.json"));
    let (issue_id, source_spec_id) = if evidence_path.is_file() {
        let evidence: crate::model::OutputEvidence = read_json(&evidence_path)?;
        (evidence.issue_id, evidence.source_spec_id)
    } else {
        ("unknown".to_string(), "unknown".to_string())
    };
    let audit_dir = root.join(".agentflow/output/audit").join(&run_id);
    ensure_directory(&audit_dir)?;
    let audit = OutputAudit {
        version: OUTPUT_AUDIT_VERSION.to_string(),
        run_id: run_id.clone(),
        issue_id,
        source_spec_id,
        status: "pending".to_string(),
        created_by: "Audit Agent".to_string(),
        created_at: unix_timestamp_seconds(),
        checks: OutputAuditChecks::default(),
        findings: Vec::new(),
    };
    write_json(&audit_dir.join("audit.json"), &audit)?;
    write_json(&audit_dir.join("findings.json"), &Vec::<String>::new())?;
    fs::write(audit_dir.join("checklist.md"), audit_checklist_content())
        .with_context(|| format!("write {}/checklist.md", audit_dir.display()))?;
    fs::write(audit_dir.join("audit-report.md"), audit_report_content())
        .with_context(|| format!("write {}/audit-report.md", audit_dir.display()))?;
    rebuild_output_index(&root)?;
    Ok(audit)
}

pub fn load_audit_output(project_root: impl AsRef<Path>, run_id: String) -> Result<OutputAudit> {
    let root = canonical_project_root(project_root)?;
    read_json(
        &root
            .join(".agentflow/output/audit")
            .join(run_id)
            .join("audit.json"),
    )
}

fn audit_report_content() -> &'static str {
    "# Audit Report\n\nStatus: pending\n\nThis audit report is a skeleton for future Audit Agent execution.\n\n## Inputs\n\n- Approved SPEC:\n- Issue:\n- Execute Run:\n- Evidence:\n- Release Delivery:\n\n## Checks\n\n- SPEC alignment:\n- Issue acceptance coverage:\n- Allowed paths:\n- Evidence completeness:\n- Release delivery completeness:\n\n## Findings\n\nNone yet.\n"
}

fn audit_checklist_content() -> &'static str {
    "# Audit Checklist\n\n- [ ] SPEC alignment\n- [ ] Issue acceptance coverage\n- [ ] Allowed paths only\n- [ ] Evidence complete\n- [ ] Release delivery complete\n\nAudit Agent is not authorized yet; this checklist is a pending skeleton.\n"
}
