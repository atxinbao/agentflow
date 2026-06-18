use crate::model::{
    ExternalReviewAuditSummary, ExternalReviewEvidenceEntry, ProjectExternalReviewIndex,
    ProjectExternalReviewIndexEntry, ProjectExternalReviewSurface, ProjectReleaseFacts,
    PublicReleaseSummary, PROJECT_EXTERNAL_REVIEW_INDEX_VERSION,
    PROJECT_EXTERNAL_REVIEW_SURFACE_VERSION,
};
use crate::public_delivery::collect_public_release_summary_for_project;
use agentflow_audit::load_project_audit_review_summary;
use agentflow_spec::{read_spec_project, SpecProject};
use anyhow::{Context, Result};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

pub fn sync_project_external_review_surface(
    project_root: impl AsRef<Path>,
    release_facts: &ProjectReleaseFacts,
) -> Result<ProjectExternalReviewSurface> {
    let root = canonical_project_root(project_root)?;
    let project = read_spec_project(&root, &release_facts.project_id)?;
    let public_summary =
        collect_public_release_summary_for_project(&root, Some(&release_facts.project_id))?;
    let audit_summary =
        load_project_audit_review_summary(&root, &project.project_id, &project.issue_ids)?;
    let handoff_path = project_review_handoff_path(&project.project_id);
    let evidence_entries = public_summary
        .entries
        .iter()
        .map(|entry| ExternalReviewEvidenceEntry {
            issue_id: entry.issue_id.clone(),
            title: entry.title.clone(),
            summary: entry.summary.clone(),
            evidence_status: entry.evidence_status.clone(),
            evidence_path: entry.evidence_path.clone(),
            validation_command_count: entry.validation_command_count,
            public_record_targets: entry.public_record_targets.clone(),
            pr_url: entry.pr_url.clone(),
            merge_commit: entry.merge_commit.clone(),
            updated_at: entry.updated_at,
        })
        .collect::<Vec<_>>();
    let audit_summary = audit_summary.map(|summary| ExternalReviewAuditSummary {
        latest_audit_id: summary.latest_audit_id,
        latest_status: summary.latest_status,
        latest_report_path: summary.latest_report_path,
        total_count: summary.total_count,
        findings_count: summary.findings_count,
        summary_line: summary.summary_line,
        findings: summary.findings,
        evidence_gaps: summary.evidence_gaps,
        repair_recommendations: summary.repair_recommendations,
    });
    let risk_items = build_risk_items(release_facts, &evidence_entries, audit_summary.as_ref());
    let review_status = derive_review_status(release_facts, audit_summary.as_ref(), &risk_items);
    let summary_line = build_review_summary_line(
        release_facts,
        &evidence_entries,
        audit_summary.as_ref(),
        &risk_items,
    );
    let surface = ProjectExternalReviewSurface {
        version: PROJECT_EXTERNAL_REVIEW_SURFACE_VERSION.to_string(),
        project_id: project.project_id.clone(),
        project_title: project.title.clone(),
        source_requirement_id: project.source_requirement_id.clone(),
        source_requirement_path: project.source_requirement_path.clone(),
        objective: project.objective.clone(),
        review_status,
        release_state: release_facts.current_state.clone(),
        release_summary_line: release_facts.summary_line.clone(),
        handoff_path: handoff_path.display().to_string(),
        total_entries: evidence_entries.len(),
        evidence_entries,
        audit_summary,
        risk_items,
        summary_line,
        generated_at: unix_timestamp_seconds(),
    };
    write_review_handoff_markdown(&root, &project, &surface, &public_summary)?;
    write_project_external_review_surface(&root, &surface)?;
    write_project_external_review_index(&root)?;
    Ok(surface)
}

pub fn load_project_external_review_surface(
    project_root: impl AsRef<Path>,
    project_id: &str,
) -> Result<ProjectExternalReviewSurface> {
    let root = canonical_project_root(project_root)?;
    read_json(&project_external_review_surface_path(&root, project_id))
}

fn write_project_external_review_surface(
    root: &Path,
    surface: &ProjectExternalReviewSurface,
) -> Result<()> {
    ensure_directory(&root.join(".agentflow/release/reviews"))?;
    write_json(
        &project_external_review_surface_path(root, &surface.project_id),
        surface,
    )
}

fn write_project_external_review_index(root: &Path) -> Result<()> {
    ensure_directory(&root.join(".agentflow/release/reviews"))?;
    ensure_directory(&root.join(".agentflow/indexes"))?;
    let mut reviews = Vec::new();
    for entry in fs::read_dir(root.join(".agentflow/release/reviews"))
        .with_context(|| "read .agentflow/release/reviews".to_string())?
    {
        let entry = entry?;
        if entry.path().extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let surface: ProjectExternalReviewSurface = read_json(&entry.path())?;
        reviews.push(ProjectExternalReviewIndexEntry {
            project_id: surface.project_id,
            review_status: surface.review_status,
            handoff_path: surface.handoff_path,
            generated_at: surface.generated_at,
        });
    }
    reviews.sort_by(|left, right| left.project_id.cmp(&right.project_id));
    write_json(
        &root.join(".agentflow/indexes/external-reviews.json"),
        &ProjectExternalReviewIndex {
            version: PROJECT_EXTERNAL_REVIEW_INDEX_VERSION.to_string(),
            updated_at: unix_timestamp_seconds(),
            reviews,
        },
    )
}

fn write_review_handoff_markdown(
    root: &Path,
    project: &SpecProject,
    surface: &ProjectExternalReviewSurface,
    public_summary: &PublicReleaseSummary,
) -> Result<()> {
    let path = root.join(project_review_handoff_path(&project.project_id));
    if let Some(parent) = path.parent() {
        ensure_directory(parent)?;
    }
    fs::write(
        &path,
        render_review_handoff_markdown(project, surface, public_summary)?,
    )
    .with_context(|| format!("write {}", path.display()))
}

fn render_review_handoff_markdown(
    project: &SpecProject,
    surface: &ProjectExternalReviewSurface,
    public_summary: &PublicReleaseSummary,
) -> Result<String> {
    let mut markdown = String::new();
    markdown.push_str(&format!("# {} External Review Handoff\n\n", project.title));
    markdown.push_str("## Project\n\n");
    markdown.push_str(&format!("- Project ID: `{}`\n", project.project_id));
    markdown.push_str(&format!(
        "- Requirement: `{}` ({})\n",
        surface.source_requirement_id, surface.source_requirement_path
    ));
    markdown.push_str(&format!("- Objective: {}\n", project.objective));
    markdown.push_str(&format!("- Review status: `{}`\n", surface.review_status));
    markdown.push_str(&format!("- Release state: `{}`\n", surface.release_state));
    markdown.push_str(&format!(
        "- Release summary: {}\n\n",
        surface.release_summary_line
    ));

    markdown.push_str("## Included Deliveries\n\n");
    markdown.push_str("| Issue | Title | Validation | Review Proof | Public Targets |\n");
    markdown.push_str("| --- | --- | --- | --- | --- |\n");
    for entry in &surface.evidence_entries {
        let proof = entry
            .pr_url
            .as_deref()
            .map(|url| url.to_string())
            .or_else(|| {
                entry
                    .merge_commit
                    .as_ref()
                    .map(|commit| format!("merge `{commit}`"))
            })
            .unwrap_or_else(|| "待补充".to_string());
        let targets = if entry.public_record_targets.is_empty() {
            "待补充".to_string()
        } else {
            entry.public_record_targets.join("<br/>")
        };
        markdown.push_str(&format!(
            "| `{}` | {} | {} 条验证命令 / 证据 `{}` | {} | {} |\n",
            entry.issue_id,
            entry.title,
            entry.validation_command_count,
            entry.evidence_status,
            proof,
            targets
        ));
    }
    markdown.push('\n');

    markdown.push_str("## Public Notes\n\n");
    markdown.push_str(&format!(
        "- Changelog template: `{}`\n",
        public_summary.changelog_template_version
    ));
    markdown.push_str(&format!(
        "- Release notes template: `{}`\n\n",
        public_summary.release_notes_template_version
    ));

    markdown.push_str("## Audit Summary\n\n");
    if let Some(audit) = surface.audit_summary.as_ref() {
        markdown.push_str(&format!("- Summary: {}\n", audit.summary_line));
        markdown.push_str(&format!("- Total audits: {}\n", audit.total_count));
        markdown.push_str(&format!(
            "- Latest status: `{}`\n",
            audit.latest_status.as_deref().unwrap_or("missing")
        ));
        if let Some(report_path) = audit.latest_report_path.as_deref() {
            markdown.push_str(&format!("- Latest report: `{}`\n", report_path));
        }
        if !audit.findings.is_empty() {
            markdown.push_str("\n### Key Findings\n\n");
            for finding in &audit.findings {
                markdown.push_str(&format!("- {}\n", finding));
            }
        }
        if !audit.evidence_gaps.is_empty() {
            markdown.push_str("\n### Evidence Gaps\n\n");
            for gap in &audit.evidence_gaps {
                markdown.push_str(&format!("- {}\n", gap));
            }
        }
    } else {
        markdown.push_str("- 当前没有关联审计记录。\n");
    }
    markdown.push('\n');

    markdown.push_str("## Reviewer Risks\n\n");
    if surface.risk_items.is_empty() {
        markdown.push_str("- 当前没有额外风险提示。\n");
    } else {
        for risk in &surface.risk_items {
            markdown.push_str(&format!("- {}\n", risk));
        }
    }
    markdown.push('\n');

    markdown.push_str("## Reviewer Decision Hint\n\n");
    markdown.push_str(&format!("{}\n", surface.summary_line));
    Ok(markdown)
}

fn derive_review_status(
    release_facts: &ProjectReleaseFacts,
    audit_summary: Option<&ExternalReviewAuditSummary>,
    risk_items: &[String],
) -> String {
    if release_facts.current_state != "published" {
        return "not-ready".to_string();
    }
    if audit_summary
        .and_then(|summary| summary.latest_status.as_deref())
        .is_some_and(|status| status == "failed")
    {
        return "blocked".to_string();
    }
    if !risk_items.is_empty() {
        return "needs-attention".to_string();
    }
    "ready".to_string()
}

fn build_review_summary_line(
    release_facts: &ProjectReleaseFacts,
    evidence_entries: &[ExternalReviewEvidenceEntry],
    audit_summary: Option<&ExternalReviewAuditSummary>,
    risk_items: &[String],
) -> String {
    if release_facts.current_state != "published" {
        return "项目 release 还没正式发布，外部 review 仍然不完整。".to_string();
    }
    if let Some(audit) = audit_summary {
        if audit.latest_status.as_deref() == Some("failed") {
            return "Release 已发布，但最近一次审计失败，不能直接对外接受。".to_string();
        }
    }
    if !risk_items.is_empty() {
        return format!(
            "Release 已发布，已整理 {} 条交付，但 reviewer 仍需先处理风险提示。",
            evidence_entries.len()
        );
    }
    format!(
        "Release 已发布，外部 reviewer 可以直接使用这份 handoff package 审阅 {} 条交付。",
        evidence_entries.len()
    )
}

fn build_risk_items(
    release_facts: &ProjectReleaseFacts,
    evidence_entries: &[ExternalReviewEvidenceEntry],
    audit_summary: Option<&ExternalReviewAuditSummary>,
) -> Vec<String> {
    let mut risk_items = Vec::new();
    if release_facts.current_state != "published" {
        risk_items.push(release_facts.summary_line.clone());
    }
    for entry in evidence_entries {
        if entry.evidence_status != "ready" {
            risk_items.push(format!(
                "{} 的本地证据状态仍是 `{}`。",
                entry.issue_id, entry.evidence_status
            ));
        }
        if entry.pr_url.is_none() && entry.merge_commit.is_none() {
            risk_items.push(format!(
                "{} 还没有明确的 PR/MR 或 merge 证明。",
                entry.issue_id
            ));
        }
    }
    if let Some(audit) = audit_summary {
        if audit.latest_status.as_deref() == Some("failed") {
            risk_items.push(audit.summary_line.clone());
        }
        risk_items.extend(audit.evidence_gaps.iter().cloned().take(3));
        risk_items.extend(audit.repair_recommendations.iter().cloned().take(3));
    }
    risk_items
}

fn project_review_handoff_path(project_id: &str) -> PathBuf {
    PathBuf::from(format!("docs/reviews/{}.md", sanitize_id(project_id)))
}

fn project_external_review_surface_path(root: &Path, project_id: &str) -> PathBuf {
    root.join(".agentflow/release/reviews")
        .join(format!("{}.json", sanitize_id(project_id)))
}

fn sanitize_id(id: &str) -> String {
    id.chars()
        .map(|ch| match ch {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => ch,
        })
        .collect()
}

fn canonical_project_root(project_root: impl AsRef<Path>) -> Result<PathBuf> {
    let root = project_root.as_ref();
    if root.exists() {
        return root
            .canonicalize()
            .with_context(|| format!("canonicalize {}", root.display()));
    }
    Ok(root.to_path_buf())
}

fn ensure_directory(path: &Path) -> Result<()> {
    fs::create_dir_all(path).with_context(|| format!("create {}", path.display()))
}

fn write_json<T: serde::Serialize>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        ensure_directory(parent)?;
    }
    fs::write(path, serde_json::to_string_pretty(value)? + "\n")
        .with_context(|| format!("write {}", path.display()))
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

fn unix_timestamp_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}
