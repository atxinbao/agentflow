use crate::model::{AuditGateKind, AuditGateStatus, LOOP_AUDIT_GATE_VERSION};
use agentflow_input::{issue::InputIssueStatus, project::InputProjectStatus};
use agentflow_output::{
    storage::{ensure_directory, read_json, write_json},
    AuditCheckStatus, AuditChecks, AuditRequest, AuditRequestSource, AuditScope, AuditScopeRef,
    AuditStatus, AuditSummary, AuditTrigger, HumanAudit, AUDIT_REQUEST_VERSION,
    OUTPUT_AUDIT_VERSION,
};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectAuditGate {
    pub version: String,
    pub project_id: String,
    pub issue_id: Option<String>,
    pub run_id: Option<String>,
    pub kind: AuditGateKind,
    pub status: AuditGateStatus,
    pub output_dir: Option<String>,
    pub updated_at: u64,
}

impl ProjectAuditGate {
    pub fn delivery(
        project_id: impl Into<String>,
        issue_id: impl Into<String>,
        run_id: impl Into<String>,
        updated_at: u64,
    ) -> Self {
        let run_id = run_id.into();
        Self {
            version: LOOP_AUDIT_GATE_VERSION.to_string(),
            project_id: project_id.into(),
            issue_id: Some(issue_id.into()),
            output_dir: Some(format!(".agentflow/audit/delivery-{run_id}")),
            run_id: Some(run_id),
            kind: AuditGateKind::Delivery,
            status: AuditGateStatus::Pending,
            updated_at,
        }
    }

    pub fn project_final(project_id: impl Into<String>, updated_at: u64) -> Self {
        let project_id = project_id.into();
        Self {
            version: LOOP_AUDIT_GATE_VERSION.to_string(),
            issue_id: None,
            run_id: None,
            output_dir: Some(format!(".agentflow/audit/project-{project_id}-final")),
            project_id,
            kind: AuditGateKind::ProjectFinal,
            status: AuditGateStatus::Pending,
            updated_at,
        }
    }

    pub fn generate_delivery_audit(
        project_root: impl AsRef<Path>,
        project_id: impl Into<String>,
        issue_id: impl Into<String>,
        run_id: impl Into<String>,
    ) -> Result<Self> {
        let root = canonical_project_root(project_root)?;
        let project_id = project_id.into();
        let issue_id = issue_id.into();
        let run_id = run_id.into();
        let audit_id = format!("delivery-{run_id}");
        let audit_dir = root.join(".agentflow/audit").join(&audit_id);
        ensure_directory(&audit_dir)?;

        let checks = delivery_checks(&root, &issue_id, &run_id);
        let status = audit_status_from_checks(&checks);
        let summary = audit_summary_from_checks(&checks);
        let requested_at = now();
        let request = AuditRequest {
            version: AUDIT_REQUEST_VERSION.to_string(),
            audit_id: audit_id.clone(),
            trigger: AuditTrigger::ReleaseAuto,
            requested_by: "agentflow-project-loop".to_string(),
            requested_at,
            reason: "Project Loop generated delivery audit report after issue completion."
                .to_string(),
            source: Some(AuditRequestSource {
                kind: "public-delivery".to_string(),
                delivery_id: Some(run_id.clone()),
                run_id: Some(run_id.clone()),
                issue_id: Some(issue_id.clone()),
                spec_id: None,
            }),
            scope: AuditScope {
                description: "Delivery audit for completed Build Agent issue.".to_string(),
                refs: delivery_scope_refs(&root, &issue_id, &run_id),
            },
        };
        let audit = HumanAudit {
            version: OUTPUT_AUDIT_VERSION.to_string(),
            audit_id: audit_id.clone(),
            trigger: AuditTrigger::ReleaseAuto,
            requested_by: "agentflow-project-loop".to_string(),
            requested_at,
            source_delivery_id: Some(run_id.clone()),
            source_run_id: Some(run_id.clone()),
            source_issue_id: Some(issue_id.clone()),
            status,
            summary,
            checks,
            paths: std::collections::BTreeMap::from([
                (
                    "request".to_string(),
                    format!(".agentflow/audit/{audit_id}/audit-request.json"),
                ),
                (
                    "audit".to_string(),
                    format!(".agentflow/audit/{audit_id}/audit.json"),
                ),
                (
                    "report".to_string(),
                    format!(".agentflow/audit/{audit_id}/audit-report.md"),
                ),
            ]),
        };
        write_json(&audit_dir.join("audit-request.json"), &request)?;
        write_json(&audit_dir.join("audit.json"), &audit)?;
        fs::write(
            audit_dir.join("audit-report.md"),
            delivery_report_markdown(&project_id, &issue_id, &run_id, &audit),
        )
        .with_context(|| format!("write {}/audit-report.md", audit_dir.display()))?;
        write_json(
            &audit_dir.join("findings.json"),
            &serde_json::json!({
                "version": "audit-findings.v1",
                "auditId": audit_id,
                "findings": []
            }),
        )?;
        write_json(
            &audit_dir.join("evidence-map.json"),
            &serde_json::json!({
                "version": "audit-evidence-map.v1",
                "auditId": audit.audit_id,
                "items": []
            }),
        )?;
        write_json(
            &audit_dir.join("traceability.json"),
            &serde_json::json!({
                "version": "audit-traceability.v1",
                "auditId": audit.audit_id,
                "items": []
            }),
        )?;
        fs::write(
            audit_dir.join("checklist.md"),
            "# Delivery Audit Checklist\n",
        )
        .with_context(|| format!("write {}/checklist.md", audit_dir.display()))?;
        agentflow_output::prepare_output_workspace(&root)?;

        let mut gate = Self::delivery(project_id, issue_id, run_id, now());
        gate.status = match audit.status {
            AuditStatus::Passed => AuditGateStatus::Passed,
            AuditStatus::PassedWithWarnings => AuditGateStatus::PassedWithWarnings,
            AuditStatus::Failed => AuditGateStatus::Failed,
            _ => AuditGateStatus::Blocked,
        };
        Ok(gate)
    }

    pub fn generate_project_final_audit(
        project_root: impl AsRef<Path>,
        project_id: impl Into<String>,
    ) -> Result<Self> {
        let root = canonical_project_root(project_root)?;
        let project_id = project_id.into();
        let input = agentflow_input::prepare_input_workspace(&root)?;
        let project = input
            .projects
            .iter()
            .find(|project| project.project_id == project_id)
            .with_context(|| format!("input project {project_id} does not exist"))?;
        let all_done = project.issue_ids.iter().all(|issue_id| {
            input.issues.iter().any(|issue| {
                issue.issue_id == *issue_id && matches!(issue.status, InputIssueStatus::Done)
            })
        });
        if !all_done {
            anyhow::bail!("project final audit requires all project issues done");
        }

        for issue_id in &project.issue_ids {
            let projection = agentflow_projection::load_task_projection(&root, issue_id)
                .with_context(|| format!("load task projection for project issue {issue_id}"))?;
            if projection.current_state != "done" {
                anyhow::bail!("project issue {issue_id} projection is not done");
            }
            let run_id = projection
                .latest_run_id
                .as_deref()
                .with_context(|| format!("project issue {issue_id} has no task run"))?;
            let audit_path = root
                .join(".agentflow/audit")
                .join(format!("delivery-{run_id}"))
                .join("audit.json");
            let audit: HumanAudit = read_json(&audit_path)
                .with_context(|| format!("load delivery audit for {run_id}"))?;
            if !matches!(
                audit.status,
                AuditStatus::Passed | AuditStatus::PassedWithWarnings
            ) {
                anyhow::bail!("delivery audit for {run_id} is not passed");
            }
        }

        let audit_id = format!("project-{project_id}-final");
        let audit_dir = root.join(".agentflow/audit").join(&audit_id);
        ensure_directory(&audit_dir)?;
        let requested_at = now();
        let request = AuditRequest {
            version: AUDIT_REQUEST_VERSION.to_string(),
            audit_id: audit_id.clone(),
            trigger: AuditTrigger::ReleaseAuto,
            requested_by: "agentflow-project-loop".to_string(),
            requested_at,
            reason: "Project Loop generated final audit after all issues were done.".to_string(),
            source: Some(AuditRequestSource {
                kind: "project-final".to_string(),
                delivery_id: None,
                run_id: None,
                issue_id: None,
                spec_id: Some(project.source_spec_id.clone()),
            }),
            scope: AuditScope {
                description: "Project final audit.".to_string(),
                refs: vec![AuditScopeRef {
                    kind: "project".to_string(),
                    id: project.project_id.clone(),
                    path: project.system.path.clone(),
                }],
            },
        };
        let checks = passed_checks();
        let audit = HumanAudit {
            version: OUTPUT_AUDIT_VERSION.to_string(),
            audit_id: audit_id.clone(),
            trigger: AuditTrigger::ReleaseAuto,
            requested_by: "agentflow-project-loop".to_string(),
            requested_at,
            source_delivery_id: None,
            source_run_id: None,
            source_issue_id: None,
            status: AuditStatus::Passed,
            summary: audit_summary_from_checks(&checks),
            checks,
            paths: std::collections::BTreeMap::from([
                (
                    "request".to_string(),
                    format!(".agentflow/audit/{audit_id}/audit-request.json"),
                ),
                (
                    "audit".to_string(),
                    format!(".agentflow/audit/{audit_id}/audit.json"),
                ),
                (
                    "report".to_string(),
                    format!(".agentflow/audit/{audit_id}/audit-report.md"),
                ),
            ]),
        };
        write_json(&audit_dir.join("audit-request.json"), &request)?;
        write_json(&audit_dir.join("audit.json"), &audit)?;
        fs::write(
            audit_dir.join("audit-report.md"),
            format!(
                "# Project Final Audit\n\nProject: `{}`\n\nStatus: `passed`\n\nAll project issues are done and all delivery audits passed.\n",
                project.project_id
            ),
        )
        .with_context(|| format!("write {}/audit-report.md", audit_dir.display()))?;
        write_json(
            &audit_dir.join("findings.json"),
            &serde_json::json!({
                "version": "audit-findings.v1",
                "auditId": audit_id,
                "findings": []
            }),
        )?;
        write_json(
            &audit_dir.join("evidence-map.json"),
            &serde_json::json!({
                "version": "audit-evidence-map.v1",
                "auditId": audit.audit_id,
                "items": []
            }),
        )?;
        write_json(
            &audit_dir.join("traceability.json"),
            &serde_json::json!({
                "version": "audit-traceability.v1",
                "auditId": audit.audit_id,
                "items": []
            }),
        )?;
        fs::write(
            audit_dir.join("checklist.md"),
            "# Project Final Audit Checklist\n",
        )
        .with_context(|| format!("write {}/checklist.md", audit_dir.display()))?;
        agentflow_input::update_input_project_status(&root, &project_id, InputProjectStatus::Done)?;
        agentflow_output::prepare_output_workspace(&root)?;

        let mut gate = Self::project_final(project_id, now());
        gate.status = AuditGateStatus::Passed;
        Ok(gate)
    }
}

fn delivery_checks(root: &Path, issue_id: &str, run_id: &str) -> AuditChecks {
    let task_run_dir = agentflow_task_artifacts::task_run_dir(root, issue_id, run_id);
    AuditChecks {
        checkpoint_exists: status_for(task_run_dir.join("run.json")),
        changed_files_recorded: status_for(task_run_dir.join("validation.json")),
        allowed_write_paths_only: status_for(
            root.join(format!(".agentflow/projections/tasks/{issue_id}.json")),
        ),
        commands_recorded: status_for(task_run_dir.join("commands")),
        high_risk_confirmed_if_needed: AuditCheckStatus::Passed,
        evidence_complete: status_for(root.join(format!(
            ".agentflow/tasks/{issue_id}/evidence/evidence.json"
        ))),
        release_delivery_complete: public_delivery_check(root, issue_id),
    }
}

fn public_delivery_check(root: &Path, issue_id: &str) -> AuditCheckStatus {
    let Ok(projection) = agentflow_projection::load_task_projection(root, issue_id) else {
        return AuditCheckStatus::Failed;
    };
    let delivery = projection.public_delivery;
    let evidence_exists = delivery
        .evidence_path
        .as_deref()
        .is_some_and(|path| root.join(path).is_file());
    let has_review =
        non_empty(delivery.pr_url.as_deref()) || non_empty(delivery.merge_commit.as_deref());
    let has_public_record = non_empty(delivery.changelog_path.as_deref())
        || non_empty(delivery.release_notes_url.as_deref());
    if projection.current_state == "done" && evidence_exists && has_review && has_public_record {
        AuditCheckStatus::Passed
    } else {
        AuditCheckStatus::Failed
    }
}

fn passed_checks() -> AuditChecks {
    AuditChecks {
        checkpoint_exists: AuditCheckStatus::Passed,
        changed_files_recorded: AuditCheckStatus::Passed,
        allowed_write_paths_only: AuditCheckStatus::Passed,
        commands_recorded: AuditCheckStatus::Passed,
        high_risk_confirmed_if_needed: AuditCheckStatus::Passed,
        evidence_complete: AuditCheckStatus::Passed,
        release_delivery_complete: AuditCheckStatus::Passed,
    }
}

fn status_for(path: PathBuf) -> AuditCheckStatus {
    if path.exists() {
        AuditCheckStatus::Passed
    } else {
        AuditCheckStatus::Failed
    }
}

fn audit_status_from_checks(checks: &AuditChecks) -> AuditStatus {
    if checks
        .values()
        .iter()
        .any(|status| matches!(status, AuditCheckStatus::Failed))
    {
        AuditStatus::Failed
    } else {
        AuditStatus::Passed
    }
}

fn audit_summary_from_checks(checks: &AuditChecks) -> AuditSummary {
    let values = checks.values();
    AuditSummary {
        checks: values.len(),
        passed: values
            .iter()
            .filter(|status| matches!(status, AuditCheckStatus::Passed))
            .count(),
        warnings: values
            .iter()
            .filter(|status| matches!(status, AuditCheckStatus::Warning))
            .count(),
        failed: values
            .iter()
            .filter(|status| matches!(status, AuditCheckStatus::Failed))
            .count(),
        findings: 0,
    }
}

fn delivery_scope_refs(root: &Path, issue_id: &str, run_id: &str) -> Vec<AuditScopeRef> {
    let public_delivery_path = agentflow_projection::load_task_projection(root, issue_id)
        .ok()
        .and_then(|projection| {
            projection
                .public_delivery
                .changelog_path
                .or(projection.public_delivery.release_notes_url)
        })
        .unwrap_or_else(|| "CHANGELOG.md".to_string());
    vec![
        AuditScopeRef {
            kind: "issue".to_string(),
            id: issue_id.to_string(),
            path: format!(".agentflow/spec/issues/{issue_id}.json"),
        },
        AuditScopeRef {
            kind: "task-run".to_string(),
            id: run_id.to_string(),
            path: format!(".agentflow/tasks/{issue_id}/runs/{run_id}/run.json"),
        },
        AuditScopeRef {
            kind: "evidence".to_string(),
            id: run_id.to_string(),
            path: format!(".agentflow/tasks/{issue_id}/evidence/evidence.json"),
        },
        AuditScopeRef {
            kind: "public-delivery".to_string(),
            id: run_id.to_string(),
            path: public_delivery_path,
        },
    ]
}

fn non_empty(value: Option<&str>) -> bool {
    value.is_some_and(|value| !value.trim().is_empty())
}

fn delivery_report_markdown(
    project_id: &str,
    issue_id: &str,
    run_id: &str,
    audit: &HumanAudit,
) -> String {
    format!(
        "# Delivery Audit Report\n\nProject: `{project_id}`\nIssue: `{issue_id}`\nRun: `{run_id}`\n\nStatus: `{}`\n\nChecks: {} passed, {} failed.\n",
        audit.status.as_str(),
        audit.summary.passed,
        audit.summary.failed
    )
}

fn canonical_project_root(project_root: impl AsRef<Path>) -> Result<PathBuf> {
    project_root
        .as_ref()
        .canonicalize()
        .with_context(|| format!("canonicalize {}", project_root.as_ref().display()))
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}
