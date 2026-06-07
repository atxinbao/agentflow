use crate::{
    manager::rebuild_output_index,
    model::{
        AuditCheckStatus, AuditChecks, AuditEvidenceMap, AuditFinding, AuditFindingSeverity,
        AuditFindings, AuditIndex, AuditIndexEntry, AuditManifest, AuditManifestSummary,
        AuditPaths, AuditRequest, AuditRequestSource, AuditScope, AuditScopeRef, AuditStatus,
        AuditSummary, AuditTraceability, AuditTraceabilityItem, AuditTrigger, HumanAudit,
        HumanAuditReport, HumanAuditRequestDraft, OutputEvidence, OutputReleaseDelivery,
        AUDIT_EVIDENCE_MAP_VERSION, AUDIT_FINDINGS_VERSION, AUDIT_INDEX_VERSION,
        AUDIT_MANIFEST_VERSION, AUDIT_REQUEST_VERSION, AUDIT_TRACEABILITY_VERSION,
        OUTPUT_AUDIT_VERSION,
    },
    storage::{
        canonical_project_root, ensure_directory, read_json, sorted_child_paths,
        unix_timestamp_seconds, write_json,
    },
    validate::{validate_output_evidence, validate_release_delivery},
};
use agentflow_input::issue::{
    audit_expected_outputs, AgentRole, DisplayStatus, InputIssue, InputIssueAudit, InputIssueKind,
    InputIssueModel, InputIssueRelations, InputIssueStatus, InputPanelLink, InputPriority,
    InputRiskLevel, InputSystemRecord, IssueCategory,
};
use anyhow::{Context, Result};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
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
    let audit_dir = root.join(".agentflow/output/audit").join(&audit_id);
    ensure_directory(&audit_dir)?;

    let requested_at = unix_timestamp_seconds();
    let request = AuditRequest {
        version: AUDIT_REQUEST_VERSION.to_string(),
        audit_id: audit_id.clone(),
        trigger: AuditTrigger::HumanViaAgent,
        requested_by: "human-via-agent".to_string(),
        requested_at,
        reason: draft.reason,
        source: audit_request_source_from_refs(&draft.scope.refs, "release-delivery"),
        scope: draft.scope,
    };

    let context = AuditContext::from_request(&root, &request)?;
    let check_result = run_audit_checks(&root, &context)?;
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
            .and_then(|source| source.issue_id.clone()),
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
    rebuild_output_index(&root)?;

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

pub fn ensure_release_auto_audits(project_root: impl AsRef<Path>) -> Result<AuditIndex> {
    let root = canonical_project_root(project_root)?;
    ensure_audit_workspace(&root)?;

    let releases = release_deliveries(&root)?;
    for release in releases {
        if release_auto_audit_exists(&root, &release)? {
            continue;
        }
        write_release_auto_audit_request(&root, &release)?;
    }
    backfill_release_auto_audit_issues_from_requests(&root)?;

    rebuild_audit_manifest_and_index(&root)?;
    rebuild_output_index(&root)?;
    load_audit_index(&root)
}

pub fn load_audit_report(
    project_root: impl AsRef<Path>,
    audit_id: String,
) -> Result<HumanAuditReport> {
    let root = canonical_project_root(project_root)?;
    let audit_dir = root.join(".agentflow/output/audit").join(&audit_id);
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

pub fn load_audit_index(project_root: impl AsRef<Path>) -> Result<AuditIndex> {
    let root = canonical_project_root(project_root)?;
    read_json(&root.join(".agentflow/output/audit/index.json"))
}

pub fn load_audit_manifest(project_root: impl AsRef<Path>) -> Result<AuditManifest> {
    let root = canonical_project_root(project_root)?;
    read_json(&root.join(".agentflow/output/audit/manifest.json"))
}

pub fn load_audit_status(project_root: impl AsRef<Path>) -> Result<AuditManifest> {
    load_audit_manifest(project_root)
}

pub(crate) fn ensure_audit_workspace(root: &Path) -> Result<()> {
    ensure_directory(&root.join(".agentflow/output/audit"))?;
    let manifest_path = root.join(".agentflow/output/audit/manifest.json");
    let index_path = root.join(".agentflow/output/audit/index.json");
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
    ensure_directory(&root.join(".agentflow/output/audit"))?;
    let index = rebuild_audit_index(root)?;
    let manifest = audit_manifest(root, &index);
    write_json(
        &root.join(".agentflow/output/audit/manifest.json"),
        &manifest,
    )?;
    Ok(manifest)
}

fn rebuild_audit_index(root: &Path) -> Result<AuditIndex> {
    let mut audits = Vec::new();
    for path in sorted_child_paths(&root.join(".agentflow/output/audit"))? {
        if !path.is_dir() {
            continue;
        }
        let audit_path = path.join("audit.json");
        let request_path = path.join("audit-request.json");
        if !request_path.is_file() {
            continue;
        }
        let Ok(request) = read_json::<AuditRequest>(&request_path) else {
            continue;
        };
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
            report_path: format!(".agentflow/output/audit/{audit_id}/audit-report.md"),
            audit_path: format!(".agentflow/output/audit/{audit_id}/audit.json"),
        });
    }
    audits.sort_by(|left, right| left.audit_id.cmp(&right.audit_id));
    let index = AuditIndex {
        version: AUDIT_INDEX_VERSION.to_string(),
        updated_at: unix_timestamp_seconds(),
        audits,
    };
    write_json(&root.join(".agentflow/output/audit/index.json"), &index)?;
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
            audit_root: ".agentflow/output/audit".to_string(),
            index: ".agentflow/output/audit/index.json".to_string(),
        },
        summary,
    }
}

fn next_audit_id(root: &Path) -> Result<String> {
    let mut max_id = 0_u64;
    for path in sorted_child_paths(&root.join(".agentflow/output/audit"))? {
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

fn backfill_release_auto_audit_issues_from_requests(root: &Path) -> Result<()> {
    for path in sorted_child_paths(&root.join(".agentflow/output/audit"))? {
        let request_path = path.join("audit-request.json");
        if !request_path.is_file() {
            continue;
        }
        let Ok(request) = read_json::<AuditRequest>(&request_path) else {
            continue;
        };
        if !matches!(request.trigger, AuditTrigger::ReleaseAuto) {
            continue;
        }
        let Some(release_id) = release_id_from_audit_request(&request) else {
            continue;
        };
        let source_spec_id = request
            .source
            .as_ref()
            .and_then(|source| source.spec_id.clone())
            .or_else(|| scope_id(&request.scope.refs, "spec"))
            .unwrap_or_else(|| "unknown-spec".to_string());
        let source_delivery_path = request
            .scope
            .refs
            .iter()
            .find(|reference| reference.kind == "release-delivery" && reference.id == release_id)
            .map(|reference| reference.path.clone())
            .unwrap_or_else(|| format!(".agentflow/output/release/{release_id}/delivery.json"));
        write_release_auto_audit_issue_record(
            root,
            &release_id,
            &source_spec_id,
            &request.audit_id,
            &source_delivery_path,
        )?;
    }
    Ok(())
}

fn release_id_from_audit_request(request: &AuditRequest) -> Option<String> {
    request
        .source
        .as_ref()
        .and_then(|source| source.delivery_id.clone().or_else(|| source.run_id.clone()))
        .or_else(|| scope_id(&request.scope.refs, "release-delivery"))
        .or_else(|| scope_id(&request.scope.refs, "execute-run"))
}

fn write_release_auto_audit_issue_record(
    root: &Path,
    release_id: &str,
    source_spec_id: &str,
    audit_id: &str,
    source_delivery_path: &str,
) -> Result<()> {
    let issue_id = release_auto_audit_issue_id(release_id);
    let issue_dir = root.join(".agentflow/input/issues");
    ensure_directory(&issue_dir)?;
    let issue_path = issue_dir.join(format!("{issue_id}.json"));
    if issue_path.is_file() {
        let mut issue: InputIssue = read_json(&issue_path)?;
        let before = serde_json::to_value(&issue)?;
        issue.issue_category = IssueCategory::Audit;
        issue.required_agent_role = AgentRole::AuditAgent;
        issue.display_status = DisplayStatus::Ready;
        issue.risk_level = InputRiskLevel::High;
        issue.audit = Some(release_auto_issue_audit(
            release_id,
            audit_id,
            source_delivery_path,
        ));
        issue.normalize_execution_metadata();
        if serde_json::to_value(&issue)? != before {
            write_json(&issue_path, &issue)?;
        }
        return Ok(());
    }

    let now = unix_timestamp_seconds();
    let issue = InputIssue {
        version: "input-issue.v1".to_string(),
        issue_id: issue_id.clone(),
        issue_model: InputIssueModel::Direct,
        issue_category: IssueCategory::Audit,
        required_agent_role: AgentRole::AuditAgent,
        source_spec_id: source_spec_id.to_string(),
        project_id: None,
        title: format!("审计 Release {release_id}"),
        summary: format!(
            "检查 {} 是否符合已确认需求、任务、证据和交付边界。",
            release_id
        ),
        kind: InputIssueKind::Validation,
        priority: InputPriority::High,
        status: InputIssueStatus::ReadyForExecute,
        display_status: DisplayStatus::Ready,
        risk_level: InputRiskLevel::High,
        scope: vec![
            format!("读取 {release_id} 的 delivery.json"),
            "读取关联 SPEC / Issue / Evidence".to_string(),
            "检查验证命令和结果".to_string(),
            "检查是否有越界改动".to_string(),
            "生成审计报告".to_string(),
        ],
        non_goals: vec![
            "不修改用户源码".to_string(),
            "不创建远程 PR".to_string(),
            "不发布 GitHub Release".to_string(),
            "不自动修复问题".to_string(),
        ],
        acceptance_criteria: vec![
            format!(".agentflow/output/audit/{audit_id}/audit.json 存在"),
            format!(".agentflow/output/audit/{audit_id}/audit-report.md 存在"),
            format!(".agentflow/output/audit/{audit_id}/findings.json 存在"),
            format!(".agentflow/output/audit/{audit_id}/evidence-map.json 存在"),
            format!(".agentflow/output/audit/{audit_id}/traceability.json 存在"),
        ],
        validation_hints: vec![
            "读取 release delivery、evidence、execute result 和 diff summary。".to_string(),
            "只写 .agentflow/output/audit/**。".to_string(),
        ],
        relations: InputIssueRelations::default(),
        panel: InputPanelLink::default(),
        audit: Some(release_auto_issue_audit(
            release_id,
            audit_id,
            source_delivery_path,
        )),
        system: InputSystemRecord {
            created_by: "agentflow-release-auto".to_string(),
            created_at: now,
            updated_at: now,
            path: format!(".agentflow/input/issues/{issue_id}.json"),
            revision: 1,
        },
        ..InputIssue::default()
    };
    let mut issue = issue;
    issue.normalize_execution_metadata();
    write_json(&issue_path, &issue)
}

fn release_auto_issue_audit(
    release_id: &str,
    audit_id: &str,
    source_delivery_path: &str,
) -> InputIssueAudit {
    let audit_output_dir = format!(".agentflow/output/audit/{audit_id}");
    InputIssueAudit {
        audit_id: audit_id.to_string(),
        trigger: "release-auto".to_string(),
        source_release_id: release_id.to_string(),
        source_run_id: Some(release_id.to_string()),
        source_delivery_path: source_delivery_path.to_string(),
        audit_output_dir: audit_output_dir.clone(),
        expected_outputs: audit_expected_outputs(&audit_output_dir),
    }
}

fn release_auto_audit_issue_id(release_id: &str) -> String {
    format!("audit-{release_id}")
}

fn release_deliveries(root: &Path) -> Result<Vec<OutputReleaseDelivery>> {
    let mut releases = Vec::new();
    for path in sorted_child_paths(&root.join(".agentflow/output/release"))? {
        let delivery_path = path.join("delivery.json");
        if !delivery_path.is_file() {
            continue;
        }
        if let Ok(release) = read_json::<OutputReleaseDelivery>(&delivery_path) {
            releases.push(release);
        }
    }
    Ok(releases)
}

fn release_auto_audit_exists(root: &Path, release: &OutputReleaseDelivery) -> Result<bool> {
    for path in sorted_child_paths(&root.join(".agentflow/output/audit"))? {
        let request_path = path.join("audit-request.json");
        if !request_path.is_file() {
            continue;
        }
        let Ok(request) = read_json::<AuditRequest>(&request_path) else {
            continue;
        };
        if !matches!(request.trigger, AuditTrigger::ReleaseAuto) {
            continue;
        }
        let source_matches = request.source.as_ref().is_some_and(|source| {
            source.run_id.as_deref() == Some(release.run_id.as_str())
                || source.delivery_id.as_deref() == Some(release.run_id.as_str())
        });
        let scope_matches = scope_id(&request.scope.refs, "execute-run").as_deref()
            == Some(release.run_id.as_str())
            || scope_id(&request.scope.refs, "release-delivery").as_deref()
                == Some(release.run_id.as_str());
        if source_matches || scope_matches {
            return Ok(true);
        }
    }
    Ok(false)
}

fn write_release_auto_audit_request(root: &Path, release: &OutputReleaseDelivery) -> Result<()> {
    let audit_id = next_audit_id(root)?;
    let audit_dir = root.join(".agentflow/output/audit").join(&audit_id);
    ensure_directory(&audit_dir)?;
    let requested_at = unix_timestamp_seconds();
    let request = AuditRequest {
        version: AUDIT_REQUEST_VERSION.to_string(),
        audit_id: audit_id.clone(),
        trigger: AuditTrigger::ReleaseAuto,
        requested_by: "agentflow-release-auto".to_string(),
        requested_at,
        reason: "审计请求已独立登记，用于核对交付材料。".to_string(),
        source: Some(AuditRequestSource {
            kind: "release-delivery".to_string(),
            delivery_id: Some(release.run_id.clone()),
            run_id: Some(release.run_id.clone()),
            issue_id: Some(release.issue_id.clone()),
            spec_id: Some(release.source_spec_id.clone()),
        }),
        scope: release_auto_scope(release),
    };
    write_json(&audit_dir.join("audit-request.json"), &request)
}

fn release_auto_scope(release: &OutputReleaseDelivery) -> AuditScope {
    AuditScope {
        description: "审计 release delivery 是否符合 SPEC、Issue、Evidence 和验证结果。"
            .to_string(),
        refs: vec![
            AuditScopeRef {
                kind: "spec".to_string(),
                id: release.source_spec_id.clone(),
                path: format!(
                    ".agentflow/input/specs/approved/{}/spec.json",
                    release.source_spec_id
                ),
            },
            AuditScopeRef {
                kind: "issue".to_string(),
                id: release.issue_id.clone(),
                path: format!(".agentflow/input/issues/{}.json", release.issue_id),
            },
            AuditScopeRef {
                kind: "execute-run".to_string(),
                id: release.run_id.clone(),
                path: format!(".agentflow/execute/runs/{}", release.run_id),
            },
            AuditScopeRef {
                kind: "evidence".to_string(),
                id: release.run_id.clone(),
                path: release.evidence_path.clone(),
            },
            AuditScopeRef {
                kind: "release-delivery".to_string(),
                id: release.run_id.clone(),
                path: format!(".agentflow/output/release/{}/delivery.json", release.run_id),
            },
        ],
    }
}

fn audit_request_source_from_refs(
    refs: &[AuditScopeRef],
    kind: &str,
) -> Option<AuditRequestSource> {
    Some(AuditRequestSource {
        kind: kind.to_string(),
        delivery_id: scope_id(refs, "release-delivery"),
        run_id: scope_id(refs, "execute-run").or_else(|| scope_id(refs, "evidence")),
        issue_id: scope_id(refs, "issue"),
        spec_id: scope_id(refs, "spec"),
    })
}

#[derive(Debug)]
struct AuditContext {
    run_id: String,
    issue_path: Option<String>,
    spec_path: Option<String>,
    run_path: String,
    evidence_path: String,
    release_path: String,
    evidence: Option<OutputEvidence>,
    release: Option<OutputReleaseDelivery>,
}

impl AuditContext {
    fn from_request(root: &Path, request: &AuditRequest) -> Result<Self> {
        let run_id = infer_run_id(&request.scope.refs)?;
        let evidence_path = scope_path(&request.scope.refs, "evidence")
            .unwrap_or_else(|| format!(".agentflow/output/evidence/{run_id}.json"));
        let release_path = scope_path(&request.scope.refs, "release-delivery")
            .unwrap_or_else(|| format!(".agentflow/output/release/{run_id}/delivery.json"));
        let run_path = scope_path(&request.scope.refs, "execute-run")
            .unwrap_or_else(|| format!(".agentflow/execute/runs/{run_id}/"));
        let evidence = read_json::<OutputEvidence>(&root.join(&evidence_path)).ok();
        let release = read_json::<OutputReleaseDelivery>(&root.join(&release_path)).ok();
        Ok(Self {
            run_id,
            issue_path: scope_path(&request.scope.refs, "issue"),
            spec_path: scope_path(&request.scope.refs, "spec"),
            run_path,
            evidence_path,
            release_path,
            evidence,
            release,
        })
    }
}

#[derive(Debug)]
struct AuditCheckResult {
    checks: AuditChecks,
    findings: Vec<AuditFinding>,
}

fn run_audit_checks(root: &Path, context: &AuditContext) -> Result<AuditCheckResult> {
    let mut findings = Vec::new();
    let checkpoint_exists = check_checkpoint(root, context, &mut findings);
    let changed_files_recorded = check_changed_files(root, context, &mut findings);
    let allowed_write_paths_only = check_allowed_write_paths(root, context, &mut findings);
    let commands_recorded = check_commands(root, context, &mut findings);
    let high_risk_confirmed_if_needed = check_high_risk_confirmation(root, context, &mut findings);
    let evidence_complete = check_evidence(root, context, &mut findings)?;
    let release_delivery_complete = check_release_delivery(root, context, &mut findings)?;
    Ok(AuditCheckResult {
        checks: AuditChecks {
            checkpoint_exists,
            changed_files_recorded,
            allowed_write_paths_only,
            commands_recorded,
            high_risk_confirmed_if_needed,
            evidence_complete,
            release_delivery_complete,
        },
        findings,
    })
}

fn check_checkpoint(
    root: &Path,
    context: &AuditContext,
    findings: &mut Vec<AuditFinding>,
) -> AuditCheckStatus {
    let checkpoint = context
        .evidence
        .as_ref()
        .and_then(|value| value.execute.checkpoint.as_deref());
    if let Some(path) = checkpoint {
        if root.join(path).is_file() {
            return AuditCheckStatus::Passed;
        }
    }
    findings.push(finding(
        findings.len(),
        AuditFindingSeverity::High,
        "checkpoint",
        "Checkpoint is missing",
        "Audit could not find a checkpoint captured before patch or command execution.",
        checkpoint.unwrap_or(&context.run_path),
        "Reject or manually review the delivery before accepting it.",
    ));
    AuditCheckStatus::Failed
}

fn check_changed_files(
    root: &Path,
    context: &AuditContext,
    findings: &mut Vec<AuditFinding>,
) -> AuditCheckStatus {
    let changed_files = context
        .evidence
        .as_ref()
        .and_then(|value| value.execute.changed_files.as_deref());
    if let Some(path) = changed_files {
        if root.join(path).is_file() {
            return AuditCheckStatus::Passed;
        }
    }
    findings.push(finding(
        findings.len(),
        AuditFindingSeverity::High,
        "changed-files",
        "Changed files are not recorded",
        "Audit could not find changed-files.json for this delivery.",
        changed_files.unwrap_or(&context.run_path),
        "Regenerate delivery evidence with changed file records before accepting it.",
    ));
    AuditCheckStatus::Failed
}

fn check_allowed_write_paths(
    root: &Path,
    context: &AuditContext,
    findings: &mut Vec<AuditFinding>,
) -> AuditCheckStatus {
    let plan_path = root
        .join(".agentflow/execute/runs")
        .join(&context.run_id)
        .join("plan.json");
    let changed_files_path = context
        .evidence
        .as_ref()
        .and_then(|value| value.execute.changed_files.as_deref())
        .map(|path| root.join(path));
    let Ok(plan) = read_json::<Value>(&plan_path) else {
        findings.push(finding(
            findings.len(),
            AuditFindingSeverity::High,
            "allowed-write-paths",
            "Execute plan is missing",
            "Audit could not read plan.json and cannot verify allowedWritePaths.",
            &relative_plan_path(&context.run_id),
            "Regenerate the run plan or manually reject this delivery.",
        ));
        return AuditCheckStatus::Failed;
    };
    let Some(changed_files_path) = changed_files_path else {
        findings.push(finding(
            findings.len(),
            AuditFindingSeverity::High,
            "allowed-write-paths",
            "Changed files are missing",
            "Audit cannot verify allowedWritePaths without changed-files.json.",
            &context.run_path,
            "Regenerate changed file evidence before accepting this delivery.",
        ));
        return AuditCheckStatus::Failed;
    };
    let Ok(changed_files) = read_json::<Value>(&changed_files_path) else {
        findings.push(finding(
            findings.len(),
            AuditFindingSeverity::High,
            "allowed-write-paths",
            "Changed files are unreadable",
            "Audit could not parse changed-files.json.",
            &path_relative_to_root(root, &changed_files_path),
            "Regenerate changed file evidence before accepting this delivery.",
        ));
        return AuditCheckStatus::Failed;
    };

    let allowed = plan
        .get("allowedWritePaths")
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let changed = changed_files
        .get("files")
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(|value| value.get("path").and_then(Value::as_str))
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    if allowed.is_empty() || changed.iter().any(|path| !path_allowed(path, &allowed)) {
        findings.push(finding(
            findings.len(),
            AuditFindingSeverity::High,
            "allowed-write-paths",
            "Changed file outside allowedWritePaths",
            "One or more changed files are not covered by the run plan allowedWritePaths.",
            &path_relative_to_root(root, &changed_files_path),
            "Review the patch manually and either reject it or require explicit scope approval.",
        ));
        return AuditCheckStatus::Failed;
    }

    AuditCheckStatus::Passed
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
            "Audit cannot inspect command records without output evidence.",
            &context.evidence_path,
            "Generate output evidence before accepting this delivery.",
        ));
        return AuditCheckStatus::Failed;
    };
    if evidence.commands.is_empty() {
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
    for command in &evidence.commands {
        if !root.join(&command.record_path).is_file()
            || command
                .stdout_path
                .as_ref()
                .map(|path| !root.join(path).is_file())
                .unwrap_or(true)
            || command
                .stderr_path
                .as_ref()
                .map(|path| !root.join(path).is_file())
                .unwrap_or(true)
        {
            findings.push(finding(
                findings.len(),
                AuditFindingSeverity::High,
                "command-records",
                "Command record is incomplete",
                "A command record, stdout, or stderr artifact is missing.",
                &command.record_path,
                "Regenerate command evidence before accepting this delivery.",
            ));
            return AuditCheckStatus::Failed;
        }
    }
    AuditCheckStatus::Passed
}

fn check_high_risk_confirmation(
    root: &Path,
    context: &AuditContext,
    findings: &mut Vec<AuditFinding>,
) -> AuditCheckStatus {
    let risk = context
        .evidence
        .as_ref()
        .map(|value| value.risk_level.as_str())
        .or_else(|| {
            context
                .release
                .as_ref()
                .map(|value| value.risk_level.as_str())
        })
        .unwrap_or("unknown");
    if !risk.eq_ignore_ascii_case("high") {
        return AuditCheckStatus::Passed;
    }
    let confirmation = root
        .join(".agentflow/execute/runs")
        .join(&context.run_id)
        .join("confirmations/high-risk-confirmation.json");
    if confirmation.is_file() {
        return AuditCheckStatus::Passed;
    }
    findings.push(finding(
        findings.len(),
        AuditFindingSeverity::High,
        "high-risk-confirmation",
        "High risk issue lacks human confirmation",
        "The audited delivery is high risk but no high-risk confirmation artifact exists.",
        &format!(
            ".agentflow/execute/runs/{}/confirmations/high-risk-confirmation.json",
            context.run_id
        ),
        "Require human confirmation before accepting this delivery.",
    ));
    AuditCheckStatus::Failed
}

fn check_evidence(
    root: &Path,
    context: &AuditContext,
    findings: &mut Vec<AuditFinding>,
) -> Result<AuditCheckStatus> {
    let validation = validate_output_evidence(root, &context.run_id)?;
    if validation.valid {
        return Ok(AuditCheckStatus::Passed);
    }
    findings.push(finding(
        findings.len(),
        AuditFindingSeverity::High,
        "evidence",
        "Evidence is incomplete",
        &validation.errors.join("; "),
        &context.evidence_path,
        "Regenerate complete output evidence before accepting this delivery.",
    ));
    Ok(AuditCheckStatus::Failed)
}

fn check_release_delivery(
    root: &Path,
    context: &AuditContext,
    findings: &mut Vec<AuditFinding>,
) -> Result<AuditCheckStatus> {
    let validation = validate_release_delivery(root, &context.run_id)?;
    if validation.valid {
        return Ok(AuditCheckStatus::Passed);
    }
    findings.push(finding(
        findings.len(),
        AuditFindingSeverity::High,
        "release-delivery",
        "Release delivery is incomplete",
        &validation.errors.join("; "),
        &context.release_path,
        "Regenerate release delivery artifacts before accepting this delivery.",
    ));
    Ok(AuditCheckStatus::Failed)
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
        inputs.insert("approvedSpec".to_string(), spec.clone());
    }
    if let Some(issue) = &context.issue_path {
        inputs.insert("issue".to_string(), issue.clone());
    }
    inputs.insert(
        "run".to_string(),
        format!(".agentflow/execute/runs/{}/run.json", context.run_id),
    );
    inputs.insert(
        "preflight".to_string(),
        format!(".agentflow/execute/runs/{}/preflight.json", context.run_id),
    );
    inputs.insert("plan".to_string(), relative_plan_path(&context.run_id));
    if let Some(evidence) = &context.evidence {
        if let Some(checkpoint) = &evidence.execute.checkpoint {
            inputs.insert("checkpoint".to_string(), checkpoint.clone());
        }
        if let Some(changed_files) = &evidence.execute.changed_files {
            inputs.insert("changedFiles".to_string(), changed_files.clone());
        }
        if let Some(diff) = &evidence.execute.diff {
            inputs.insert("diff".to_string(), diff.clone());
        }
    }
    inputs.insert(
        "result".to_string(),
        format!(".agentflow/execute/runs/{}/result.json", context.run_id),
    );
    inputs.insert("evidence".to_string(), context.evidence_path.clone());
    inputs.insert("releaseDelivery".to_string(), context.release_path.clone());
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
    chain.push(AuditTraceabilityItem {
        layer: "audit".to_string(),
        id: audit_id.to_string(),
        path: format!(".agentflow/output/audit/{audit_id}/"),
    });
    if !chain.iter().any(|item| item.layer == "execute-run") {
        chain.insert(
            0,
            AuditTraceabilityItem {
                layer: "execute-run".to_string(),
                id: context.run_id.clone(),
                path: context.run_path.clone(),
            },
        );
    }
    AuditTraceability {
        version: AUDIT_TRACEABILITY_VERSION.to_string(),
        audit_id: audit_id.to_string(),
        chain,
    }
}

fn checklist_content(audit: &HumanAudit) -> String {
    format!(
        "# Audit Checklist\n\n## Core Checks\n\n- [{}] Checkpoint exists before patch / command.\n- [{}] Changed files are recorded.\n- [{}] Changed files are inside allowedWritePaths.\n- [{}] Commands are fully recorded.\n- [{}] High risk issue has human confirmation if required.\n- [{}] Evidence is complete.\n- [{}] Release delivery is complete.\n\n## Result\n\n- [{}] Passed\n- [{}] Passed with warnings\n- [{}] Failed\n\n## Notes\n\nAudit Agent only read existing evidence chain and wrote this audit package.\n",
        checkbox(&audit.checks.checkpoint_exists),
        checkbox(&audit.checks.changed_files_recorded),
        checkbox(&audit.checks.allowed_write_paths_only),
        checkbox(&audit.checks.commands_recorded),
        checkbox(&audit.checks.high_risk_confirmed_if_needed),
        checkbox(&audit.checks.evidence_complete),
        checkbox(&audit.checks.release_delivery_complete),
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
        "# Audit Report\n\n## 1. Summary\n\n- Audit ID: `{}`\n- Status: `{}`\n- Requested By: `{}`\n- Requested At: `{}`\n- Reason: {}\n- Scope: {}\n- Final Verdict: `{}`\n\n## 2. Audit Scope\n\n本次审计范围：\n\n{}\n\n不在本次审计范围：\n\n- 不重新写代码\n- 不重新执行 patch\n- 不修改 input facts\n- 不修改 Approved SPEC\n- 不创建 PR\n- 不 merge\n- 不 deploy\n\n## 3. Traceability\n\n| Layer | Path | Status |\n|---|---|---|\n{}\n\n## 4. Core Checks\n\n### 4.1 Checkpoint\n\nVerdict: `{}`\n\n### 4.2 Changed Files\n\nVerdict: `{}`\n\n### 4.3 Allowed Write Paths\n\nVerdict: `{}`\n\n### 4.4 Command Records\n\nVerdict: `{}`\n\n### 4.5 High Risk Confirmation\n\nVerdict: `{}`\n\n### 4.6 Evidence Completeness\n\nVerdict: `{}`\n\n### 4.7 Release Delivery Completeness\n\nVerdict: `{}`\n\n## 5. Findings\n\n| Severity | Category | Finding | Recommendation |\n|---|---|---|---|\n{}\n\n## 6. Final Verdict\n\nStatus: `{}`\n\nReason: Audit V1 only checked Build Agent evidence chain completeness, boundary, traceability, high-risk confirmation, and release package completeness.\n\nRecommended next action: {}\n",
        audit.audit_id,
        audit.status.as_str(),
        request.requested_by,
        request.requested_at,
        request.reason,
        request.scope.description,
        audit.status.as_str(),
        scope_refs,
        trace_table,
        audit.checks.checkpoint_exists.as_str(),
        audit.checks.changed_files_recorded.as_str(),
        audit.checks.allowed_write_paths_only.as_str(),
        audit.checks.commands_recorded.as_str(),
        audit.checks.high_risk_confirmed_if_needed.as_str(),
        audit.checks.evidence_complete.as_str(),
        audit.checks.release_delivery_complete.as_str(),
        finding_rows,
        audit.status.as_str(),
        if matches!(audit.status, AuditStatus::Passed) {
            "Delivery can proceed to human acceptance."
        } else {
            "Resolve audit findings before accepting this delivery."
        },
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
            format!(".agentflow/output/audit/{audit_id}/audit-request.json"),
        ),
        (
            "report".to_string(),
            format!(".agentflow/output/audit/{audit_id}/audit-report.md"),
        ),
        (
            "findings".to_string(),
            format!(".agentflow/output/audit/{audit_id}/findings.json"),
        ),
        (
            "checklist".to_string(),
            format!(".agentflow/output/audit/{audit_id}/checklist.md"),
        ),
        (
            "evidenceMap".to_string(),
            format!(".agentflow/output/audit/{audit_id}/evidence-map.json"),
        ),
        (
            "traceability".to_string(),
            format!(".agentflow/output/audit/{audit_id}/traceability.json"),
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

fn infer_run_id(refs: &[AuditScopeRef]) -> Result<String> {
    for kind in ["execute-run", "evidence", "release-delivery"] {
        if let Some(id) = scope_id(refs, kind) {
            return Ok(id);
        }
    }
    anyhow::bail!("human audit scope must include execute-run, evidence, or release-delivery ref")
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

fn relative_plan_path(run_id: &str) -> String {
    format!(".agentflow/execute/runs/{run_id}/plan.json")
}

fn path_relative_to_root(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .map(|value| value.display().to_string())
        .unwrap_or_else(|_| path.display().to_string())
}

fn path_allowed(path: &str, allowed: &[String]) -> bool {
    allowed.iter().any(|candidate| {
        let normalized = candidate
            .trim()
            .trim_end_matches("/*")
            .trim_end_matches('/');
        path == normalized || path.starts_with(&format!("{normalized}/"))
    })
}

#[allow(dead_code)]
fn _assert_send_sync(_: &PathBuf) {}
