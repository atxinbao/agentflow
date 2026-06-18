use crate::model::{
    DeliverySummary, ProjectDeliverySummary, PublicReleaseDocumentPaths,
    PublicReleaseDocumentTarget, PublicReleaseEntry, PublicReleaseSummary,
    DELIVERY_SUMMARY_VERSION, PROJECT_DELIVERY_SUMMARY_VERSION, PUBLIC_RELEASE_SUMMARY_VERSION,
};
use agentflow_spec::read_spec_issue;
use anyhow::{Context, Result};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Clone, Default, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProjectionPublicDeliverySnapshot {
    evidence_path: Option<String>,
    pr_url: Option<String>,
    merge_commit: Option<String>,
    changelog_path: Option<String>,
    release_notes_url: Option<String>,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProjectionDeliverySummarySnapshot {
    status: String,
    evidence_status: String,
    evidence_path: Option<String>,
    pr_url: Option<String>,
    merge_commit: Option<String>,
    public_record_path: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct TaskProjectionSnapshot {
    issue_id: String,
    project_id: Option<String>,
    current_state: String,
    #[serde(default)]
    public_delivery: ProjectionPublicDeliverySnapshot,
    #[serde(default)]
    delivery: ProjectionDeliverySummarySnapshot,
    updated_at: u64,
}

pub fn collect_public_release_summary(
    project_root: impl AsRef<Path>,
) -> Result<PublicReleaseSummary> {
    let root = canonical_project_root(project_root)?;
    let mut entries = load_done_task_entries(&root)?;
    entries.sort_by(|left, right| {
        right
            .updated_at
            .cmp(&left.updated_at)
            .then_with(|| left.issue_id.cmp(&right.issue_id))
    });
    let changelog_markdown = render_changelog(&entries);
    let release_notes_markdown = render_release_notes(&entries);

    Ok(PublicReleaseSummary {
        version: PUBLIC_RELEASE_SUMMARY_VERSION.to_string(),
        generated_at: unix_timestamp_seconds(),
        entries,
        changelog_markdown,
        release_notes_markdown,
    })
}

pub fn load_delivery_summary(
    project_root: impl AsRef<Path>,
    issue_id: impl AsRef<str>,
) -> Result<DeliverySummary> {
    let root = canonical_project_root(project_root)?;
    let projection = load_task_projection_snapshot(&root, issue_id.as_ref())?;
    Ok(delivery_summary_from_snapshot(&projection))
}

pub fn load_project_delivery_summary(
    project_root: impl AsRef<Path>,
    project_id: impl AsRef<str>,
) -> Result<Option<ProjectDeliverySummary>> {
    let root = canonical_project_root(project_root)?;
    let project_id = project_id.as_ref();
    let mut snapshots = load_task_projection_snapshots(&root)?
        .into_iter()
        .filter(|projection| projection.project_id.as_deref() == Some(project_id))
        .collect::<Vec<_>>();
    if snapshots.is_empty() {
        return Ok(None);
    }
    snapshots.sort_by(|left, right| {
        right
            .updated_at
            .cmp(&left.updated_at)
            .then_with(|| left.issue_id.cmp(&right.issue_id))
    });
    Ok(Some(project_delivery_summary_from_snapshots(
        project_id,
        &snapshots,
    )))
}

pub fn write_public_release_documents(
    project_root: impl AsRef<Path>,
    summary: &PublicReleaseSummary,
    target: &PublicReleaseDocumentTarget,
) -> Result<PublicReleaseDocumentPaths> {
    let root = canonical_project_root(project_root)?;
    let changelog_path = root.join(&target.changelog_path);
    let release_notes_path = root.join(&target.release_notes_path);
    write_text(&changelog_path, &summary.changelog_markdown)?;
    write_text(&release_notes_path, &summary.release_notes_markdown)?;
    Ok(PublicReleaseDocumentPaths {
        changelog_path: target.changelog_path.display().to_string(),
        release_notes_path: target.release_notes_path.display().to_string(),
    })
}

fn load_done_task_entries(root: &Path) -> Result<Vec<PublicReleaseEntry>> {
    let mut entries = Vec::new();
    for projection in load_task_projection_snapshots(root)? {
        if projection.current_state != "done" {
            continue;
        }
        let issue = read_spec_issue(root, &projection.issue_id)?;
        entries.push(PublicReleaseEntry {
            issue_id: projection.issue_id,
            project_id: projection.project_id,
            title: issue.title,
            current_state: projection.current_state,
            pr_url: projection.public_delivery.pr_url,
            merge_commit: projection.public_delivery.merge_commit,
            evidence_path: projection.public_delivery.evidence_path,
            changelog_path: projection.public_delivery.changelog_path,
            release_notes_url: projection.public_delivery.release_notes_url,
            updated_at: projection.updated_at,
        });
    }
    Ok(entries)
}

fn load_task_projection_snapshot(root: &Path, issue_id: &str) -> Result<TaskProjectionSnapshot> {
    let path = task_projection_path(root, issue_id);
    read_json(&path)
}

fn load_task_projection_snapshots(root: &Path) -> Result<Vec<TaskProjectionSnapshot>> {
    let projection_dir = root.join(".agentflow/projections/tasks");
    if !projection_dir.exists() {
        return Ok(Vec::new());
    }
    let mut snapshots = Vec::new();
    for entry in fs::read_dir(&projection_dir)
        .with_context(|| format!("read {}", projection_dir.display()))?
    {
        let entry = entry?;
        if entry.path().extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        snapshots.push(read_json(&entry.path())?);
    }
    Ok(snapshots)
}

fn task_projection_path(root: &Path, issue_id: &str) -> PathBuf {
    root.join(".agentflow/projections/tasks")
        .join(format!("{issue_id}.json"))
}

fn delivery_summary_from_snapshot(snapshot: &TaskProjectionSnapshot) -> DeliverySummary {
    let evidence_path = snapshot
        .delivery
        .evidence_path
        .clone()
        .or_else(|| snapshot.public_delivery.evidence_path.clone());
    let pr_url = snapshot
        .delivery
        .pr_url
        .clone()
        .or_else(|| snapshot.public_delivery.pr_url.clone());
    let merge_commit = snapshot
        .delivery
        .merge_commit
        .clone()
        .or_else(|| snapshot.public_delivery.merge_commit.clone());
    let public_record_path = snapshot
        .delivery
        .public_record_path
        .clone()
        .or_else(|| snapshot.public_delivery.changelog_path.clone())
        .or_else(|| snapshot.public_delivery.release_notes_url.clone());
    let release_notes_url = snapshot.public_delivery.release_notes_url.clone();
    let mut public_record_items = Vec::new();
    if pr_url.is_some() {
        public_record_items.push("PR/MR body".to_string());
    }
    if let Some(path) = snapshot.public_delivery.changelog_path.clone() {
        public_record_items.push(path);
    }
    if let Some(path) = snapshot.public_delivery.release_notes_url.clone() {
        if !public_record_items.iter().any(|item| item == &path) {
            public_record_items.push(path);
        }
    }

    let mut missing_public_records = Vec::new();
    if matches!(
        snapshot.current_state.as_str(),
        "in_review" | "done" | "published"
    ) {
        if pr_url.is_none() {
            missing_public_records.push("PR/MR body".to_string());
        }
        if snapshot.public_delivery.changelog_path.is_none()
            && snapshot.public_delivery.release_notes_url.is_none()
        {
            missing_public_records.push("CHANGELOG.md 或 release notes".to_string());
        }
    }

    let status = if !snapshot.delivery.status.trim().is_empty() {
        snapshot.delivery.status.clone()
    } else if public_record_path.is_some() {
        "published".to_string()
    } else if pr_url.is_some() || merge_commit.is_some() || evidence_path.is_some() {
        "ready".to_string()
    } else {
        "missing".to_string()
    };
    let evidence_status = if !snapshot.delivery.evidence_status.trim().is_empty() {
        snapshot.delivery.evidence_status.clone()
    } else if evidence_path.is_some() {
        "ready".to_string()
    } else {
        "missing".to_string()
    };
    let summary_line = if !missing_public_records.is_empty() {
        format!(
            "公开交付正在整理，当前缺少 {}。",
            missing_public_records.join("、")
        )
    } else if !public_record_items.is_empty() {
        format!(
            "公开交付已整理到 {}。",
            public_record_items.join("、")
        )
    } else if evidence_path.is_some() || pr_url.is_some() || merge_commit.is_some() {
        "交付事实已存在，但公开交付记录还没有整理完成。".to_string()
    } else {
        "当前还没有公开交付记录。".to_string()
    };

    DeliverySummary {
        version: DELIVERY_SUMMARY_VERSION.to_string(),
        issue_id: snapshot.issue_id.clone(),
        project_id: snapshot.project_id.clone(),
        status,
        evidence_status,
        evidence_path,
        pr_url,
        merge_commit,
        public_record_path,
        release_notes_url,
        summary_line,
        public_record_items,
        missing_public_records,
        updated_at: snapshot.updated_at,
    }
}

fn project_delivery_summary_from_snapshots(
    project_id: &str,
    snapshots: &[TaskProjectionSnapshot],
) -> ProjectDeliverySummary {
    let summaries = snapshots
        .iter()
        .map(delivery_summary_from_snapshot)
        .collect::<Vec<_>>();
    let published_count = summaries
        .iter()
        .filter(|summary| summary.status == "published")
        .count();
    let ready_count = summaries
        .iter()
        .filter(|summary| summary.status == "ready")
        .count();
    let missing_count = summaries
        .iter()
        .filter(|summary| summary.status == "missing")
        .count();
    let current_issue_id = snapshots
        .iter()
        .find(|snapshot| !matches!(snapshot.current_state.as_str(), "done" | "cancel"))
        .map(|snapshot| snapshot.issue_id.clone());
    let status = if missing_count == 0 && (published_count > 0 || ready_count > 0) {
        if ready_count == 0 {
            "published".to_string()
        } else {
            "ready".to_string()
        }
    } else if published_count > 0 || ready_count > 0 {
        "ready".to_string()
    } else {
        "missing".to_string()
    };
    let updated_at = summaries.iter().map(|summary| summary.updated_at).max().unwrap_or(0);
    let public_record_items = summaries
        .iter()
        .flat_map(|summary| summary.public_record_items.clone())
        .collect::<Vec<_>>();
    let mut unique_public_record_items = Vec::new();
    for item in public_record_items {
        if !unique_public_record_items.iter().any(|existing| existing == &item) {
            unique_public_record_items.push(item);
        }
    }
    let missing_public_records = summaries
        .iter()
        .flat_map(|summary| summary.missing_public_records.clone())
        .collect::<Vec<_>>();
    let mut unique_missing_public_records = Vec::new();
    for item in missing_public_records {
        if !unique_missing_public_records
            .iter()
            .any(|existing| existing == &item)
        {
            unique_missing_public_records.push(item);
        }
    }
    let summary_line = if !unique_missing_public_records.is_empty() {
        format!(
            "项目公开交付仍缺少 {}。",
            unique_missing_public_records.join("、")
        )
    } else if !unique_public_record_items.is_empty() {
        format!(
            "项目公开交付已汇总到 {}。",
            unique_public_record_items.join("、")
        )
    } else {
        "当前项目还没有公开交付记录。".to_string()
    };

    ProjectDeliverySummary {
        version: PROJECT_DELIVERY_SUMMARY_VERSION.to_string(),
        project_id: project_id.to_string(),
        status,
        current_issue_id,
        published_count,
        ready_count,
        missing_count,
        summary_line,
        public_record_items: unique_public_record_items,
        missing_public_records: unique_missing_public_records,
        updated_at,
    }
}

fn render_changelog(entries: &[PublicReleaseEntry]) -> String {
    let mut markdown = String::from("# Changelog\n\n## Generated Public Delivery Summary\n\n");
    if entries.is_empty() {
        markdown.push_str("- No completed task deliveries found.\n");
        return markdown;
    }
    for entry in entries {
        markdown.push_str(&format!("- `{}` {}\n", entry.issue_id, entry.title));
        push_optional_line(&mut markdown, "  - PR/MR", entry.pr_url.as_deref());
        push_optional_line(&mut markdown, "  - Merge", entry.merge_commit.as_deref());
        push_optional_line(
            &mut markdown,
            "  - Evidence",
            entry.evidence_path.as_deref(),
        );
    }
    markdown
}

fn render_release_notes(entries: &[PublicReleaseEntry]) -> String {
    let mut markdown = String::from("# Release Notes\n\n## Completed Tasks\n\n");
    if entries.is_empty() {
        markdown.push_str("No completed task deliveries found.\n");
        return markdown;
    }
    for entry in entries {
        markdown.push_str(&format!("### {} {}\n\n", entry.issue_id, entry.title));
        markdown.push_str("- Status: done\n");
        push_optional_line(&mut markdown, "- PR/MR", entry.pr_url.as_deref());
        push_optional_line(&mut markdown, "- Merge", entry.merge_commit.as_deref());
        push_optional_line(&mut markdown, "- Evidence", entry.evidence_path.as_deref());
        push_optional_line(
            &mut markdown,
            "- Release notes",
            entry.release_notes_url.as_deref(),
        );
        markdown.push('\n');
    }
    markdown
}

fn push_optional_line(markdown: &mut String, label: &str, value: Option<&str>) {
    if let Some(value) = value.filter(|value| !value.trim().is_empty()) {
        markdown.push_str(&format!("{label}: {value}\n"));
    }
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

fn write_text(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(path, content).with_context(|| format!("write {}", path.display()))
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

fn unix_timestamp_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use agentflow_spec::{issue_from_requirement, write_spec_issue, SpecIssueDraft};
    use tempfile::tempdir;

    fn write_requirement(root: &Path) -> PathBuf {
        let path = root.join("docs/requirements/034-release-test.md");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "# Release Test\n\n用于 release 汇总测试。\n").unwrap();
        path
    }

    fn write_issue(root: &Path, issue_id: &str, title: &str) {
        let requirement = write_requirement(root);
        let mut draft = SpecIssueDraft::new(issue_id);
        draft.title = Some(title.to_string());
        let issue = issue_from_requirement(root, &requirement, draft).unwrap();
        write_spec_issue(root, &issue).unwrap();
    }

    fn write_projection(root: &Path, issue_id: &str, state: &str, updated_at: u64) {
        let projection = serde_json::json!({
            "issueId": issue_id.to_string(),
            "projectId": "project-release",
            "currentState": state,
            "publicDelivery": {
                "evidencePath": format!(".agentflow/tasks/{issue_id}/evidence/evidence.json"),
                "prUrl": format!("https://github.com/acme/repo/pull/{updated_at}"),
                "mergeCommit": format!("merge-{updated_at}"),
                "changelogPath": if state == "done" { Some("CHANGELOG.md") } else { None::<&str> },
                "releaseNotesUrl": if state == "done" { Some("docs/release-notes/generated.md") } else { None::<&str> },
            },
            "delivery": {
                "status": if state == "done" { "published" } else { "ready" },
                "evidenceStatus": "ready",
                "evidencePath": format!(".agentflow/tasks/{issue_id}/evidence/evidence.json"),
                "prUrl": format!("https://github.com/acme/repo/pull/{updated_at}"),
                "mergeCommit": format!("merge-{updated_at}"),
                "publicRecordPath": if state == "done" { Some("CHANGELOG.md") } else { None::<&str> },
            },
            "updatedAt": updated_at,
        });
        let dir = root.join(".agentflow/projections/tasks");
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join(format!("{issue_id}.json")),
            serde_json::to_string_pretty(&projection).unwrap(),
        )
        .unwrap();
    }

    #[test]
    fn collects_done_task_public_delivery_records() {
        let dir = tempdir().unwrap();
        write_issue(dir.path(), "AF-REL-001", "完成公开交付记录");
        write_issue(dir.path(), "AF-REL-002", "仍在执行");
        write_projection(dir.path(), "AF-REL-001", "done", 20);
        write_projection(dir.path(), "AF-REL-002", "in_progress", 30);

        let summary = collect_public_release_summary(dir.path()).unwrap();

        assert_eq!(summary.version, PUBLIC_RELEASE_SUMMARY_VERSION);
        assert_eq!(summary.entries.len(), 1);
        assert_eq!(summary.entries[0].issue_id, "AF-REL-001");
        assert!(summary.changelog_markdown.contains("完成公开交付记录"));
        assert!(summary.release_notes_markdown.contains("merge-20"));
    }

    #[test]
    fn loads_delivery_summary_with_human_readable_public_records() {
        let dir = tempdir().unwrap();
        write_issue(dir.path(), "AF-REL-001", "完成公开交付记录");
        write_projection(dir.path(), "AF-REL-001", "done", 20);

        let summary = load_delivery_summary(dir.path(), "AF-REL-001").unwrap();

        assert_eq!(summary.version, DELIVERY_SUMMARY_VERSION);
        assert_eq!(summary.status, "published");
        assert!(summary.public_record_items.contains(&"PR/MR body".to_string()));
        assert!(summary.public_record_items.contains(&"CHANGELOG.md".to_string()));
        assert!(summary.summary_line.contains("公开交付已整理到"));
    }

    #[test]
    fn aggregates_project_delivery_summary() {
        let dir = tempdir().unwrap();
        write_issue(dir.path(), "AF-REL-001", "完成公开交付记录");
        write_issue(dir.path(), "AF-REL-002", "整理评审交付");
        write_projection(dir.path(), "AF-REL-001", "done", 20);
        write_projection(dir.path(), "AF-REL-002", "in_review", 30);

        let summary = load_project_delivery_summary(dir.path(), "project-release")
            .unwrap()
            .expect("project delivery summary");

        assert_eq!(summary.version, PROJECT_DELIVERY_SUMMARY_VERSION);
        assert_eq!(summary.project_id, "project-release");
        assert_eq!(summary.current_issue_id.as_deref(), Some("AF-REL-002"));
        assert_eq!(summary.published_count, 1);
        assert_eq!(summary.ready_count, 1);
        assert!(summary.public_record_items.contains(&"PR/MR body".to_string()));
    }

    #[test]
    fn writes_public_release_documents_to_target_paths() {
        let dir = tempdir().unwrap();
        write_issue(dir.path(), "AF-REL-001", "完成公开交付记录");
        write_projection(dir.path(), "AF-REL-001", "done", 20);
        let summary = collect_public_release_summary(dir.path()).unwrap();
        let target = PublicReleaseDocumentTarget {
            changelog_path: PathBuf::from("CHANGELOG.generated.md"),
            release_notes_path: PathBuf::from("docs/release-notes/generated.md"),
        };

        let paths = write_public_release_documents(dir.path(), &summary, &target).unwrap();

        assert_eq!(paths.changelog_path, "CHANGELOG.generated.md");
        assert!(dir.path().join("CHANGELOG.generated.md").is_file());
        assert!(dir.path().join("docs/release-notes/generated.md").is_file());
    }
}
