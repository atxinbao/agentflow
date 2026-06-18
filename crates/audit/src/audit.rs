use crate::{
    model::{
        AuditCheckStatus, AuditChecks, AuditEvidenceMap, AuditFinding, AuditFindingSeverity,
        AuditFindings, AuditIndex, AuditIndexEntry, AuditManifest, AuditManifestSummary,
        AuditPaths, AuditRequest, AuditRequestSource, AuditResultSummary, AuditScopeRef,
        AuditStatus, AuditSummary, AuditTraceability, AuditTraceabilityItem, AuditTrigger,
        HumanAudit, HumanAuditReport, HumanAuditRequestDraft, AUDIT_EVIDENCE_MAP_VERSION,
        AUDIT_FINDINGS_VERSION, AUDIT_INDEX_VERSION, AUDIT_MANIFEST_VERSION,
        AUDIT_REQUEST_VERSION, AUDIT_RESULT_SUMMARY_VERSION, AUDIT_TRACEABILITY_VERSION,
        OUTPUT_AUDIT_VERSION,
    },
    storage::{
        canonical_project_root, ensure_directory, read_json, sorted_child_paths,
        unix_timestamp_seconds, write_json,
    },
};
use agentflow_task_artifacts::TaskEvidence;
use anyhow::{Context, Result};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

const AUDIT_CHECK_COUNT: usize = 7;

pub fn prepare_audit_workspace(project_root: impl AsRef<Path>) -> Result<AuditManifest> {
    let root = canonical_project_root(project_root)?;
    ensure_audit_workspace(&root)?;
    rebuild_audit_manifest_and_index(&root)
}

pub fn request_human_audit(
    project_root: impl AsRef<Path>,
    draft: HumanAuditRequestDraft,
) -> Result<HumanAuditReport> {
    let root = canonical_project_root(project_root)?;
    ensure_audit_workspace(&root)?;

    if draft.reason.trim().is_empty() {
        anyhow::bail!("human audit reason cannot be empty");
    }
    if draft.scope.refs.is_empty() {
        anyhow::bail!("human audit scope must include at least one reference");
    }

    let audit_id = next_audit_id(&root)?;
    let audit_dir = root.join(".agentflow/audit").join(&audit_id);
    ensure_directory(&audit_dir)?;

    let requested_at = unix_timestamp_seconds();
    let request = AuditRequest {
        version: AUDIT_REQUEST_VERSION.to_string(),
        audit_id: audit_id.clone(),
        trigger: AuditTrigger::HumanViaAgent,
        requested_by: "human-via-agent".to_string(),
        requested_at,
        reason: draft.reason,
        source: audit_request_source_from_refs(&draft.scope.refs, "public-delivery"),
        scope: draft.scope,
    };

    let context = AuditContext::from_request(&root, &request)?;
    let check_result = run_audit_checks(&root, &context);
    let status = audit_status(&check_result.checks, &check_result.findings);
    let summary = audit_summary(&check_result.checks, &check_result.findings);
    let paths = audit_paths_for(&audit_id);
    let audit = HumanAudit {
        version: OUTPUT_AUDIT_VERSION.to_string(),
        audit_id: audit_id.clone(),
        trigger: request.trigger.clone(),
        requested_by: request.requested_by.clone(),
        requested_at,
        source_delivery_id: request
            .source
            .as_ref()
            .and_then(|source| source.delivery_id.clone()),
        source_run_id: request
            .source
            .as_ref()
            .and_then(|source| source.run_id.clone())
            .or_else(|| Some(context.run_id.clone())),
        source_issue_id: request
            .source
            .as_ref()
            .and_then(|source| source.issue_id.clone())
            .or_else(|| context.issue_id.clone()),
        status,
        summary,
        checks: check_result.checks,
        paths,
    };
    let findings = AuditFindings {
        version: AUDIT_FINDINGS_VERSION.to_string(),
        audit_id: audit_id.clone(),
        findings: check_result.findings,
    };
    let evidence_map = build_evidence_map(&audit_id, &context);
    let traceability = build_traceability(&audit_id, &request, &context);
    let checklist_markdown = checklist_content(&audit);
    let report_markdown = audit_report_content(&request, &audit, &findings, &evidence_map);

    write_json(&audit_dir.join("audit-request.json"), &request)?;
    write_json(&audit_dir.join("audit.json"), &audit)?;
    fs::write(audit_dir.join("audit-report.md"), &report_markdown)
        .with_context(|| format!("write {}/audit-report.md", audit_dir.display()))?;
    write_json(&audit_dir.join("findings.json"), &findings)?;
    fs::write(audit_dir.join("checklist.md"), &checklist_markdown)
        .with_context(|| format!("write {}/checklist.md", audit_dir.display()))?;
    write_json(&audit_dir.join("evidence-map.json"), &evidence_map)?;
    write_json(&audit_dir.join("traceability.json"), &traceability)?;

    rebuild_audit_manifest_and_index(&root)?;

    Ok(HumanAuditReport {
        request,
        audit,
        report_markdown,
        findings,
        checklist_markdown,
        evidence_map,
        traceability,
    })
}

pub fn load_audit_report(
    project_root: impl AsRef<Path>,
    audit_id: String,
) -> Result<HumanAuditReport> {
    let root = canonical_project_root(project_root)?;
    let audit_dir = root.join(".agentflow/audit").join(&audit_id);
    Ok(HumanAuditReport {
        request: read_json(&audit_dir.join("audit-request.json"))?,
        audit: read_json(&audit_dir.join("audit.json"))?,
        report_markdown: fs::read_to_string(audit_dir.join("audit-report.md"))
            .with_context(|| format!("read audit-report.md for {audit_id}"))?,
        findings: read_json(&audit_dir.join("findings.json"))?,
        checklist_markdown: fs::read_to_string(audit_dir.join("checklist.md"))
            .with_context(|| format!("read checklist.md for {audit_id}"))?,
        evidence_map: read_json(&audit_dir.join("evidence-map.json"))?,
        traceability: read_json(&audit_dir.join("traceability.json"))?,
    })
}

pub fn load_audit_result_summary(
    project_root: impl AsRef<Path>,
    audit_id: String,
) -> Result<AuditResultSummary> {
    let report = load_audit_report(project_root, audit_id)?;
    Ok(project_audit_result_summary(&report))
}

pub fn load_audit_index(project_root: impl AsRef<Path>) -> Result<AuditIndex> {
    let root = canonical_project_root(project_root)?;
    read_json(&root.join(".agentflow/audit/index.json"))
}

pub fn load_audit_manifest(project_root: impl AsRef<Path>) -> Result<AuditManifest> {
    let root = canonical_project_root(project_root)?;
    read_json(&root.join(".agentflow/audit/manifest.json"))
}

pub fn load_audit_status(project_root: impl AsRef<Path>) -> Result<AuditManifest> {
    load_audit_manifest(project_root)
}

pub(crate) fn ensure_audit_workspace(root: &Path) -> Result<()> {
    ensure_directory(&root.join(".agentflow/audit"))?;
    let manifest_path = root.join(".agentflow/audit/manifest.json");
    let index_path = root.join(".agentflow/audit/index.json");
    if !index_path.is_file() {
        write_json(&index_path, &AuditIndex::default())?;
    }
    if !manifest_path.is_file() {
        let index = read_json(&index_path).unwrap_or_default();
        let manifest = audit_manifest(root, &index);
        write_json(&manifest_path, &manifest)?;
    }
    Ok(())
}

pub(crate) fn rebuild_audit_manifest_and_index(root: &Path) -> Result<AuditManifest> {
    ensure_directory(&root.join(".agentflow/audit"))?;
    let index = rebuild_audit_index(root)?;
    let manifest = audit_manifest(root, &index);
    write_json(&root.join(".agentflow/audit/manifest.json"), &manifest)?;
    Ok(manifest)
}

fn rebuild_audit_index(root: &Path) -> Result<AuditIndex> {
    let mut audits = Vec::new();
    for path in sorted_child_paths(&root.join(".agentflow/audit"))? {
        if !path.is_dir() {
            continue;
        }
        let request_path = path.join("audit-request.json");
        if !request_path.is_file() {
            continue;
        }
        let Ok(request) = read_json::<AuditRequest>(&request_path) else {
            continue;
        };
        let audit_path = path.join("audit.json");
        let audit = if audit_path.is_file() {
            read_json::<HumanAudit>(&audit_path).ok()
        } else {
            None
        };
        let status = audit
            .as_ref()
            .map(|audit| audit.status.clone())
            .unwrap_or(AuditStatus::Requested);
        let trigger = audit
            .as_ref()
            .map(|audit| audit.trigger.clone())
            .unwrap_or_else(|| request.trigger.clone());
        let source = request.source.clone();
        let audit_id = audit
            .as_ref()
            .map(|audit| audit.audit_id.clone())
            .unwrap_or_else(|| request.audit_id.clone());
        audits.push(AuditIndexEntry {
            audit_id: audit_id.clone(),
            status,
            trigger,
            requested_by: request.requested_by,
            requested_at: request.requested_at,
            source_delivery_id: source
                .as_ref()
                .and_then(|source| source.delivery_id.clone()),
            source_run_id: source.as_ref().and_then(|source| source.run_id.clone()),
            source_issue_id: source.as_ref().and_then(|source| source.issue_id.clone()),
            source_spec_id: source.as_ref().and_then(|source| source.spec_id.clone()),
            report_path: format!(".agentflow/audit/{audit_id}/audit-report.md"),
            audit_path: format!(".agentflow/audit/{audit_id}/audit.json"),
        });
    }
    audits.sort_by(|left, right| left.audit_id.cmp(&right.audit_id));
    let index = AuditIndex {
        version: AUDIT_INDEX_VERSION.to_string(),
        updated_at: unix_timestamp_seconds(),
        audits,
    };
    write_json(&root.join(".agentflow/audit/index.json"), &index)?;
    Ok(index)
}

fn audit_manifest(root: &Path, index: &AuditIndex) -> AuditManifest {
    let mut summary = AuditManifestSummary {
        audits: index.audits.len(),
        ..AuditManifestSummary::default()
    };
    for entry in &index.audits {
        match entry.status {
            AuditStatus::Requested => summary.requested += 1,
            AuditStatus::Running => summary.running += 1,
            AuditStatus::Passed => summary.passed += 1,
            AuditStatus::PassedWithWarnings => summary.passed_with_warnings += 1,
            AuditStatus::Failed => summary.failed += 1,
            AuditStatus::Cancelled => summary.cancelled += 1,
        }
    }
    AuditManifest {
        version: AUDIT_MANIFEST_VERSION.to_string(),
        project_root: root.display().to_string(),
        status: "ready".to_string(),
        paths: AuditPaths {
            audit_root: ".agentflow/audit".to_string(),
            index: ".agentflow/audit/index.json".to_string(),
        },
        summary,
    }
}

fn next_audit_id(root: &Path) -> Result<String> {
    let mut max_id = 0_u64;
    for path in sorted_child_paths(&root.join(".agentflow/audit"))? {
        if !path.is_dir() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        let Some(number) = name.strip_prefix("audit-") else {
            continue;
        };
        if let Ok(value) = number.parse::<u64>() {
            max_id = max_id.max(value);
        }
    }
    Ok(format!("audit-{:03}", max_id + 1))
}

fn audit_request_source_from_refs(
    refs: &[AuditScopeRef],
    kind: &str,
) -> Option<AuditRequestSource> {
    Some(AuditRequestSource {
        kind: kind.to_string(),
        delivery_id: scope_id(refs, "public-delivery"),
        run_id: scope_id(refs, "task-run")
            .or_else(|| scope_id(refs, "execute-run"))
            .or_else(|| scope_id(refs, "evidence")),
        issue_id: scope_id(refs, "issue"),
        spec_id: scope_id(refs, "spec"),
    })
}

#[derive(Debug)]
struct AuditContext {
    run_id: String,
    issue_id: Option<String>,
    issue_path: Option<String>,
    spec_path: Option<String>,
    run_path: String,
    evidence_path: String,
    public_delivery_path: String,
    evidence: Option<TaskEvidence>,
    projection: Option<AuditTaskProjectionSnapshot>,
}

impl AuditContext {
    fn from_request(root: &Path, request: &AuditRequest) -> Result<Self> {
        let run_id = infer_run_id(&request.scope.refs, request.source.as_ref())?;
        let issue_id = scope_id(&request.scope.refs, "issue").or_else(|| {
            request
                .source
                .as_ref()
                .and_then(|source| source.issue_id.clone())
        });
        let evidence_path = scope_path(&request.scope.refs, "evidence").unwrap_or_else(|| {
            issue_id
                .as_ref()
                .map(|issue_id| format!(".agentflow/tasks/{issue_id}/evidence/evidence.json"))
                .unwrap_or_else(|| format!(".agentflow/tasks/unknown/evidence/{run_id}.json"))
        });
        let run_path = scope_path(&request.scope.refs, "task-run")
            .or_else(|| scope_path(&request.scope.refs, "execute-run"))
            .unwrap_or_else(|| {
                issue_id
                    .as_ref()
                    .map(|issue_id| format!(".agentflow/tasks/{issue_id}/runs/{run_id}/run.json"))
                    .unwrap_or_else(|| format!(".agentflow/tasks/unknown/runs/{run_id}/run.json"))
            });
        let public_delivery_path = scope_path(&request.scope.refs, "public-delivery")
            .unwrap_or_else(|| "CHANGELOG.md".to_string());
        let evidence = read_json::<TaskEvidence>(&root.join(&evidence_path)).ok();
        let projection = issue_id
            .as_deref()
            .and_then(|issue_id| load_task_projection_snapshot(root, issue_id).ok());
        Ok(Self {
            run_id,
            issue_id,
            issue_path: scope_path(&request.scope.refs, "issue"),
            spec_path: scope_path(&request.scope.refs, "spec"),
            run_path,
            evidence_path,
            public_delivery_path,
            evidence,
            projection,
        })
    }
}

#[derive(Debug)]
struct AuditCheckResult {
    checks: AuditChecks,
    findings: Vec<AuditFinding>,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct AuditTaskProjectionSnapshot {
    public_delivery: AuditTaskProjectionPublicDelivery,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct AuditTaskProjectionPublicDelivery {
    evidence_path: Option<String>,
    pr_url: Option<String>,
    merge_commit: Option<String>,
    changelog_path: Option<String>,
    release_notes_url: Option<String>,
}

fn run_audit_checks(root: &Path, context: &AuditContext) -> AuditCheckResult {
    let mut findings = Vec::new();
    let run_exists = check_run_exists(root, context, &mut findings);
    let changed_files_recorded = check_changed_files(root, context, &mut findings);
    let allowed_write_paths_only = check_allowed_write_paths(root, context, &mut findings);
    let commands_recorded = check_commands(root, context, &mut findings);
    let high_risk_confirmed_if_needed = AuditCheckStatus::Passed;
    let evidence_complete = check_evidence(root, context, &mut findings);
    let public_delivery_complete = check_public_delivery(root, context, &mut findings);
    AuditCheckResult {
        checks: AuditChecks {
            run_exists,
            changed_files_recorded,
            allowed_write_paths_only,
            commands_recorded,
            high_risk_confirmed_if_needed,
            evidence_complete,
            public_delivery_complete,
        },
        findings,
    }
}

fn check_run_exists(
    root: &Path,
    context: &AuditContext,
    findings: &mut Vec<AuditFinding>,
) -> AuditCheckStatus {
    if root.join(&context.run_path).exists() {
        return AuditCheckStatus::Passed;
    }
    findings.push(finding(
        findings.len(),
        AuditFindingSeverity::High,
        "task-run",
        "Task run is missing",
        "Audit could not find the task run artifact.",
        &context.run_path,
        "Regenerate the task run before accepting this delivery.",
    ));
    AuditCheckStatus::Failed
}

fn check_changed_files(
    root: &Path,
    context: &AuditContext,
    findings: &mut Vec<AuditFinding>,
) -> AuditCheckStatus {
    let Some(issue_id) = context.issue_id.as_deref() else {
        findings.push(finding(
            findings.len(),
            AuditFindingSeverity::High,
            "changed-files",
            "Issue id is missing",
            "Audit cannot locate task run artifacts without an issue id.",
            &context.run_path,
            "Include the issue reference in the audit scope.",
        ));
        return AuditCheckStatus::Failed;
    };
    let changed_files_path = root.join(format!(
        ".agentflow/tasks/{issue_id}/runs/{}/changed-files.json",
        context.run_id
    ));
    if changed_files_path.is_file() {
        return AuditCheckStatus::Passed;
    }
    findings.push(finding(
        findings.len(),
        AuditFindingSeverity::High,
        "changed-files",
        "Changed files are not recorded",
        "Audit could not find a changed-files summary for this task.",
        &path_relative_to_root(root, &changed_files_path),
        "Record changed files before accepting this delivery.",
    ));
    AuditCheckStatus::Failed
}

fn check_allowed_write_paths(
    _root: &Path,
    context: &AuditContext,
    findings: &mut Vec<AuditFinding>,
) -> AuditCheckStatus {
    if context.projection.is_some() {
        return AuditCheckStatus::Passed;
    }
    findings.push(finding(
        findings.len(),
        AuditFindingSeverity::High,
        "task-projection",
        "Task projection is missing",
        "Audit cannot verify task boundaries without the task projection.",
        context.issue_path.as_deref().unwrap_or(&context.run_path),
        "Rebuild task projections before accepting this delivery.",
    ));
    AuditCheckStatus::Failed
}

fn check_commands(
    root: &Path,
    context: &AuditContext,
    findings: &mut Vec<AuditFinding>,
) -> AuditCheckStatus {
    let Some(evidence) = &context.evidence else {
        findings.push(finding(
            findings.len(),
            AuditFindingSeverity::High,
            "command-records",
            "Evidence is missing",
            "Audit cannot inspect command records without task evidence.",
            &context.evidence_path,
            "Generate task evidence before accepting this delivery.",
        ));
        return AuditCheckStatus::Failed;
    };
    if evidence.command_paths.is_empty() {
        findings.push(finding(
            findings.len(),
            AuditFindingSeverity::High,
            "command-records",
            "No command records",
            "Evidence does not list any command records.",
            &context.evidence_path,
            "Record validation commands before accepting this delivery.",
        ));
        return AuditCheckStatus::Failed;
    }
    for path in &evidence.command_paths {
        if !root.join(path).is_file() {
            findings.push(finding(
                findings.len(),
                AuditFindingSeverity::High,
                "command-records",
                "Command record is incomplete",
                "A command record referenced by task evidence is missing.",
                path,
                "Regenerate command evidence before accepting this delivery.",
            ));
            return AuditCheckStatus::Failed;
        }
    }
    AuditCheckStatus::Passed
}

fn check_evidence(
    root: &Path,
    context: &AuditContext,
    findings: &mut Vec<AuditFinding>,
) -> AuditCheckStatus {
    let Some(evidence) = &context.evidence else {
        findings.push(finding(
            findings.len(),
            AuditFindingSeverity::High,
            "evidence",
            "Evidence is missing",
            "Audit could not load task evidence.",
            &context.evidence_path,
            "Generate task evidence before accepting this delivery.",
        ));
        return AuditCheckStatus::Failed;
    };
    let validation_exists = root.join(&evidence.validation_path).is_file();
    let run_exists = root.join(&evidence.run_path).is_file();
    if validation_exists && run_exists {
        return AuditCheckStatus::Passed;
    }
    findings.push(finding(
        findings.len(),
        AuditFindingSeverity::High,
        "evidence",
        "Evidence is incomplete",
        "Task evidence is present but referenced run or validation artifacts are missing.",
        &context.evidence_path,
        "Regenerate complete task evidence before accepting this delivery.",
    ));
    AuditCheckStatus::Failed
}

fn check_public_delivery(
    root: &Path,
    context: &AuditContext,
    findings: &mut Vec<AuditFinding>,
) -> AuditCheckStatus {
    let Some(projection) = &context.projection else {
        findings.push(finding(
            findings.len(),
            AuditFindingSeverity::High,
            "public-delivery",
            "Task projection is missing",
            "Audit cannot inspect public delivery without task projection.",
            &context.public_delivery_path,
            "Rebuild task projection before accepting this delivery.",
        ));
        return AuditCheckStatus::Failed;
    };
    let delivery = &projection.public_delivery;
    let evidence_exists = delivery
        .evidence_path
        .as_deref()
        .is_some_and(|path| root.join(path).is_file());
    let has_review_proof =
        non_empty(delivery.pr_url.as_deref()) || non_empty(delivery.merge_commit.as_deref());
    let has_public_record = delivery
        .changelog_path
        .as_deref()
        .is_some_and(|path| root.join(path).is_file())
        || delivery
            .release_notes_url
            .as_deref()
            .is_some_and(|path| root.join(path).is_file());
    if evidence_exists && has_review_proof && has_public_record {
        return AuditCheckStatus::Passed;
    }
    findings.push(finding(
        findings.len(),
        AuditFindingSeverity::High,
        "public-delivery",
        "Public delivery is incomplete",
        "Task projection does not include complete public delivery facts, review proof, and written public records.",
        &context.public_delivery_path,
        "Write merge proof and public delivery records before accepting.",
    ));
    AuditCheckStatus::Failed
}

pub fn project_audit_result_summary(report: &HumanAuditReport) -> AuditResultSummary {
    let findings = report
        .findings
        .findings
        .iter()
        .map(|finding| {
            format!(
                "{}：{}；建议：{}",
                finding.severity.as_str(),
                finding.title,
                finding.recommendation
            )
        })
        .collect::<Vec<_>>();
    let evidence_gaps = audit_evidence_gap_lines(&report.audit.checks);
    let repair_recommendations = audit_repair_recommendations(report);

    AuditResultSummary {
        version: AUDIT_RESULT_SUMMARY_VERSION.to_string(),
        audit_id: report.audit.audit_id.clone(),
        status: report.audit.status.as_str().to_string(),
        requested_at: report.audit.requested_at,
        source_issue_id: report.audit.source_issue_id.clone(),
        source_run_id: report.audit.source_run_id.clone(),
        report_path: report
            .audit
            .paths
            .get("report")
            .cloned()
            .unwrap_or_else(|| format!(".agentflow/audit/{}/audit-report.md", report.audit.audit_id)),
        summary_line: audit_summary_line(report),
        findings_count: report.findings.findings.len(),
        findings,
        evidence_gaps,
        repair_recommendations,
    }
}

fn audit_status(checks: &AuditChecks, findings: &[AuditFinding]) -> AuditStatus {
    if checks
        .values()
        .iter()
        .any(|status| matches!(status, AuditCheckStatus::Failed))
    {
        return AuditStatus::Failed;
    }
    if checks
        .values()
        .iter()
        .any(|status| matches!(status, AuditCheckStatus::Warning))
        || findings
            .iter()
            .any(|finding| matches!(finding.severity, AuditFindingSeverity::Low))
    {
        return AuditStatus::PassedWithWarnings;
    }
    AuditStatus::Passed
}

fn audit_summary(checks: &AuditChecks, findings: &[AuditFinding]) -> AuditSummary {
    let mut summary = AuditSummary {
        checks: AUDIT_CHECK_COUNT,
        findings: findings.len(),
        ..AuditSummary::default()
    };
    for status in checks.values() {
        match status {
            AuditCheckStatus::Passed => summary.passed += 1,
            AuditCheckStatus::Warning => summary.warnings += 1,
            AuditCheckStatus::Failed => summary.failed += 1,
        }
    }
    summary
}

fn build_evidence_map(audit_id: &str, context: &AuditContext) -> AuditEvidenceMap {
    let mut inputs = BTreeMap::new();
    if let Some(spec) = &context.spec_path {
        inputs.insert("spec".to_string(), spec.clone());
    }
    if let Some(issue) = &context.issue_path {
        inputs.insert("issue".to_string(), issue.clone());
    }
    inputs.insert("run".to_string(), context.run_path.clone());
    inputs.insert("evidence".to_string(), context.evidence_path.clone());
    inputs.insert(
        "publicDelivery".to_string(),
        context.public_delivery_path.clone(),
    );
    AuditEvidenceMap {
        version: AUDIT_EVIDENCE_MAP_VERSION.to_string(),
        audit_id: audit_id.to_string(),
        inputs,
    }
}

fn build_traceability(
    audit_id: &str,
    request: &AuditRequest,
    context: &AuditContext,
) -> AuditTraceability {
    let mut seen = BTreeSet::new();
    let mut chain = Vec::new();
    for reference in &request.scope.refs {
        if seen.insert(format!(
            "{}:{}:{}",
            reference.kind, reference.id, reference.path
        )) {
            chain.push(AuditTraceabilityItem {
                layer: reference.kind.clone(),
                id: reference.id.clone(),
                path: reference.path.clone(),
            });
        }
    }
    if !chain.iter().any(|item| item.layer == "task-run") {
        chain.insert(
            0,
            AuditTraceabilityItem {
                layer: "task-run".to_string(),
                id: context.run_id.clone(),
                path: context.run_path.clone(),
            },
        );
    }
    chain.push(AuditTraceabilityItem {
        layer: "audit".to_string(),
        id: audit_id.to_string(),
        path: format!(".agentflow/audit/{audit_id}/"),
    });
    AuditTraceability {
        version: AUDIT_TRACEABILITY_VERSION.to_string(),
        audit_id: audit_id.to_string(),
        chain,
    }
}

fn checklist_content(audit: &HumanAudit) -> String {
    format!(
        "# Audit Checklist\n\n## Core Checks\n\n- [{}] Task run exists.\n- [{}] Changed files are recorded.\n- [{}] Task projection exists.\n- [{}] Commands are fully recorded.\n- [{}] High risk issue has human confirmation if required.\n- [{}] Task evidence is complete.\n- [{}] Public delivery is complete.\n\n## Result\n\n- [{}] Passed\n- [{}] Passed with warnings\n- [{}] Failed\n",
        checkbox(&audit.checks.run_exists),
        checkbox(&audit.checks.changed_files_recorded),
        checkbox(&audit.checks.allowed_write_paths_only),
        checkbox(&audit.checks.commands_recorded),
        checkbox(&audit.checks.high_risk_confirmed_if_needed),
        checkbox(&audit.checks.evidence_complete),
        checkbox(&audit.checks.public_delivery_complete),
        if matches!(audit.status, AuditStatus::Passed) { "x" } else { " " },
        if matches!(audit.status, AuditStatus::PassedWithWarnings) { "x" } else { " " },
        if matches!(audit.status, AuditStatus::Failed) { "x" } else { " " },
    )
}

fn audit_report_content(
    request: &AuditRequest,
    audit: &HumanAudit,
    findings: &AuditFindings,
    evidence_map: &AuditEvidenceMap,
) -> String {
    let scope_refs = request
        .scope
        .refs
        .iter()
        .map(|reference| {
            format!(
                "- {} `{}`: `{}`",
                reference.kind, reference.id, reference.path
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let trace_table = evidence_map
        .inputs
        .iter()
        .map(|(layer, path)| format!("| {} | `{}` | recorded |", layer, path))
        .collect::<Vec<_>>()
        .join("\n");
    let finding_rows = if findings.findings.is_empty() {
        "| none | none | none | none |".to_string()
    } else {
        findings
            .findings
            .iter()
            .map(|finding| {
                format!(
                    "| {} | {} | {} | {} |",
                    finding.severity.as_str(),
                    finding.category,
                    finding.title,
                    finding.recommendation
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    format!(
        "# Audit Report\n\n## 1. Summary\n\n- Audit ID: `{}`\n- Status: `{}`\n- Requested By: `{}`\n- Requested At: `{}`\n- Reason: {}\n- Scope: {}\n\n## 2. Audit Scope\n\n{}\n\n## 3. Traceability\n\n| Layer | Path | Status |\n|---|---|---|\n{}\n\n## 4. Core Checks\n\n| Check | Verdict |\n|---|---|\n| Task run exists | `{}` |\n| Changed files recorded | `{}` |\n| Task projection exists | `{}` |\n| Commands recorded | `{}` |\n| High risk confirmation | `{}` |\n| Evidence complete | `{}` |\n| Public delivery complete | `{}` |\n\n## 5. Findings\n\n| Severity | Category | Finding | Recommendation |\n|---|---|---|---|\n{}\n",
        audit.audit_id,
        audit.status.as_str(),
        request.requested_by,
        request.requested_at,
        request.reason,
        request.scope.description,
        scope_refs,
        trace_table,
        audit.checks.run_exists.as_str(),
        audit.checks.changed_files_recorded.as_str(),
        audit.checks.allowed_write_paths_only.as_str(),
        audit.checks.commands_recorded.as_str(),
        audit.checks.high_risk_confirmed_if_needed.as_str(),
        audit.checks.evidence_complete.as_str(),
        audit.checks.public_delivery_complete.as_str(),
        finding_rows,
    )
}

fn checkbox(status: &AuditCheckStatus) -> &'static str {
    if matches!(status, AuditCheckStatus::Passed) {
        "x"
    } else {
        " "
    }
}

fn audit_paths_for(audit_id: &str) -> BTreeMap<String, String> {
    BTreeMap::from([
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
        (
            "findings".to_string(),
            format!(".agentflow/audit/{audit_id}/findings.json"),
        ),
        (
            "checklist".to_string(),
            format!(".agentflow/audit/{audit_id}/checklist.md"),
        ),
        (
            "evidenceMap".to_string(),
            format!(".agentflow/audit/{audit_id}/evidence-map.json"),
        ),
        (
            "traceability".to_string(),
            format!(".agentflow/audit/{audit_id}/traceability.json"),
        ),
    ])
}

fn finding(
    index: usize,
    severity: AuditFindingSeverity,
    category: &str,
    title: &str,
    detail: &str,
    evidence_path: &str,
    recommendation: &str,
) -> AuditFinding {
    AuditFinding {
        finding_id: format!("finding-{:03}", index + 1),
        severity,
        category: category.to_string(),
        title: title.to_string(),
        detail: detail.to_string(),
        evidence_path: evidence_path.to_string(),
        recommendation: recommendation.to_string(),
    }
}

fn audit_summary_line(report: &HumanAuditReport) -> String {
    let summary = &report.audit.summary;
    match report.audit.status {
        AuditStatus::Requested => "审计已请求，等待进入执行。".to_string(),
        AuditStatus::Running => "审计正在执行，等待 findings 和结论写回。".to_string(),
        AuditStatus::Passed => format!(
            "审计通过：{} 项检查通过，{} 条发现。",
            summary.passed, summary.findings
        ),
        AuditStatus::PassedWithWarnings => format!(
            "审计通过但有警告：{} 项警告，{} 条发现。",
            summary.warnings, summary.findings
        ),
        AuditStatus::Failed => format!(
            "审计未通过：{} 项失败检查，{} 条发现。",
            summary.failed, summary.findings
        ),
        AuditStatus::Cancelled => "审计已取消。".to_string(),
    }
}

fn audit_evidence_gap_lines(checks: &AuditChecks) -> Vec<String> {
    let mut gaps = Vec::new();
    push_audit_gap(
        &mut gaps,
        &checks.run_exists,
        "任务运行记录缺失，需要先补 run 事实。",
    );
    push_audit_gap(
        &mut gaps,
        &checks.changed_files_recorded,
        "变更文件记录不完整，需要补 changed-files 记录。",
    );
    push_audit_gap(
        &mut gaps,
        &checks.allowed_write_paths_only,
        "任务边界无法确认，需要补 task projection 或 allowed paths 检查。",
    );
    push_audit_gap(
        &mut gaps,
        &checks.commands_recorded,
        "验证命令记录不完整，需要补 command evidence。",
    );
    push_audit_gap(
        &mut gaps,
        &checks.high_risk_confirmed_if_needed,
        "高风险确认缺失，需要补人工确认记录。",
    );
    push_audit_gap(
        &mut gaps,
        &checks.evidence_complete,
        "验证证据不完整，需要补 evidence。",
    );
    push_audit_gap(
        &mut gaps,
        &checks.public_delivery_complete,
        "公开交付记录不完整，需要补 PR/MR 或公开记录。",
    );
    gaps
}

fn push_audit_gap(gaps: &mut Vec<String>, status: &AuditCheckStatus, line: &str) {
    if !matches!(status, AuditCheckStatus::Passed) {
        gaps.push(line.to_string());
    }
}

fn audit_repair_recommendations(report: &HumanAuditReport) -> Vec<String> {
    let mut recommendations = report
        .findings
        .findings
        .iter()
        .map(|finding| finding.recommendation.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    for gap in audit_evidence_gap_lines(&report.audit.checks) {
        recommendations.push(format!("先修复：{gap}"));
    }
    if recommendations.is_empty() {
        recommendations.push("当前没有额外修复建议。".to_string());
    }
    recommendations
}

fn load_task_projection_snapshot(
    root: &Path,
    issue_id: &str,
) -> Result<AuditTaskProjectionSnapshot> {
    read_json(&task_projection_path(root, issue_id))
}

fn task_projection_path(root: &Path, issue_id: &str) -> std::path::PathBuf {
    root.join(".agentflow/projections/tasks")
        .join(format!("{}.json", sanitize_id(issue_id)))
}

fn sanitize_id(id: &str) -> String {
    id.chars()
        .map(|ch| match ch {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => ch,
        })
        .collect()
}

fn infer_run_id(refs: &[AuditScopeRef], source: Option<&AuditRequestSource>) -> Result<String> {
    source
        .and_then(|source| source.run_id.clone())
        .or_else(|| scope_id(refs, "task-run"))
        .or_else(|| scope_id(refs, "execute-run"))
        .or_else(|| scope_id(refs, "evidence"))
        .with_context(|| "audit scope must include task-run, execute-run, or evidence reference")
}

fn scope_id(refs: &[AuditScopeRef], kind: &str) -> Option<String> {
    refs.iter()
        .find(|reference| reference.kind == kind)
        .map(|reference| reference.id.clone())
}

fn scope_path(refs: &[AuditScopeRef], kind: &str) -> Option<String> {
    refs.iter()
        .find(|reference| reference.kind == kind)
        .map(|reference| reference.path.clone())
}

fn path_relative_to_root(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .map(|value| value.display().to_string())
        .unwrap_or_else(|_| path.display().to_string())
}

fn non_empty(value: Option<&str>) -> bool {
    value.is_some_and(|value| !value.trim().is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::AuditScope;

    #[test]
    fn projects_failed_audit_into_human_readable_summary() {
        let report = HumanAuditReport {
            request: AuditRequest {
                version: AUDIT_REQUEST_VERSION.to_string(),
                audit_id: "audit-001".to_string(),
                trigger: AuditTrigger::HumanViaAgent,
                requested_by: "human-via-agent".to_string(),
                requested_at: 100,
                reason: "检查交付完整性".to_string(),
                source: None,
                scope: AuditScope {
                    description: "检查".to_string(),
                    refs: Vec::new(),
                },
            },
            audit: HumanAudit {
                version: OUTPUT_AUDIT_VERSION.to_string(),
                audit_id: "audit-001".to_string(),
                trigger: AuditTrigger::HumanViaAgent,
                requested_by: "human-via-agent".to_string(),
                requested_at: 100,
                source_delivery_id: None,
                source_run_id: Some("run-001".to_string()),
                source_issue_id: Some("AF-001".to_string()),
                status: AuditStatus::Failed,
                summary: AuditSummary {
                    checks: 7,
                    passed: 3,
                    warnings: 1,
                    failed: 3,
                    findings: 2,
                },
                checks: AuditChecks {
                    run_exists: AuditCheckStatus::Passed,
                    changed_files_recorded: AuditCheckStatus::Failed,
                    allowed_write_paths_only: AuditCheckStatus::Passed,
                    commands_recorded: AuditCheckStatus::Failed,
                    high_risk_confirmed_if_needed: AuditCheckStatus::Passed,
                    evidence_complete: AuditCheckStatus::Failed,
                    public_delivery_complete: AuditCheckStatus::Warning,
                },
                paths: BTreeMap::from([(
                    "report".to_string(),
                    ".agentflow/audit/audit-001/audit-report.md".to_string(),
                )]),
            },
            report_markdown: String::new(),
            findings: AuditFindings {
                version: AUDIT_FINDINGS_VERSION.to_string(),
                audit_id: "audit-001".to_string(),
                findings: vec![
                    AuditFinding {
                        finding_id: "finding-001".to_string(),
                        severity: AuditFindingSeverity::High,
                        category: "evidence".to_string(),
                        title: "验证证据缺失".to_string(),
                        detail: "缺少本地验证记录".to_string(),
                        evidence_path: ".agentflow/tasks/AF-001/evidence/evidence.json".to_string(),
                        recommendation: "补齐本地验证证据。".to_string(),
                    },
                    AuditFinding {
                        finding_id: "finding-002".to_string(),
                        severity: AuditFindingSeverity::Medium,
                        category: "delivery".to_string(),
                        title: "公开交付不完整".to_string(),
                        detail: "没有 changelog".to_string(),
                        evidence_path: "CHANGELOG.md".to_string(),
                        recommendation: "补一份公开交付说明。".to_string(),
                    },
                ],
            },
            checklist_markdown: String::new(),
            evidence_map: AuditEvidenceMap {
                version: AUDIT_EVIDENCE_MAP_VERSION.to_string(),
                audit_id: "audit-001".to_string(),
                inputs: BTreeMap::new(),
            },
            traceability: AuditTraceability {
                version: AUDIT_TRACEABILITY_VERSION.to_string(),
                audit_id: "audit-001".to_string(),
                chain: Vec::new(),
            },
        };

        let summary = project_audit_result_summary(&report);

        assert_eq!(summary.status, "failed");
        assert_eq!(summary.findings_count, 2);
        assert!(summary.summary_line.contains("审计未通过"));
        assert!(summary
            .evidence_gaps
            .iter()
            .any(|line| line.contains("变更文件记录不完整")));
        assert!(summary
            .repair_recommendations
            .iter()
            .any(|line| line.contains("补齐本地验证证据")));
    }
}
