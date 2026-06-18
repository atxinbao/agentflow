use crate::model::{
    PublicReleaseDocumentPaths, PublicReleaseDocumentTarget, PublicReleaseEntry,
    PublicReleaseSummary, PUBLIC_RELEASE_SUMMARY_VERSION,
};
use agentflow_projection::TaskProjection;
use agentflow_spec::read_spec_issue;
use anyhow::{Context, Result};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

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
    let projection_dir = root.join(".agentflow/projections/tasks");
    if !projection_dir.exists() {
        return Ok(Vec::new());
    }
    let mut entries = Vec::new();
    for entry in fs::read_dir(&projection_dir)
        .with_context(|| format!("read {}", projection_dir.display()))?
    {
        let entry = entry?;
        if entry.path().extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let projection: TaskProjection = read_json(&entry.path())?;
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
    use agentflow_projection::{
        ProjectionPhase, ProjectionPublicDelivery, TaskProjection, TaskTimelineEvent,
        TaskTimelineItem, TASK_PROJECTION_VERSION,
    };
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
        let projection = TaskProjection {
            version: TASK_PROJECTION_VERSION.to_string(),
            issue_id: issue_id.to_string(),
            project_id: Some("project-release".to_string()),
            workflow_ref: "build-agent.issue-loop@v1".to_string(),
            current_state: state.to_string(),
            display_status: state.to_string(),
            current_transition: None,
            latest_run_id: Some("run-001".to_string()),
            branch_name: Some(format!("agentflow/project-release/{issue_id}")),
            timeline: vec![TaskTimelineItem {
                state: state.to_string(),
                phase: ProjectionPhase::Past,
                entered_at: Some(updated_at),
                events: vec![TaskTimelineEvent {
                    event_id: format!("evt-{issue_id}"),
                    event_type: "issue.completed".to_string(),
                    timestamp: updated_at,
                    actor_role: "build-agent".to_string(),
                    actor_kind: "system".to_string(),
                    summary: "任务 Done 写回完成。".to_string(),
                    artifact_refs: Vec::new(),
                }],
                summary: "任务已完成。".to_string(),
                live_refs: Vec::new(),
            }],
            public_delivery: ProjectionPublicDelivery {
                evidence_path: Some(format!(
                    ".agentflow/tasks/{issue_id}/evidence/evidence.json"
                )),
                pr_url: Some(format!("https://github.com/acme/repo/pull/{updated_at}")),
                merge_commit: Some(format!("merge-{updated_at}")),
                changelog_path: None,
                release_notes_url: None,
            },
            runtime: agentflow_projection::ProjectionRuntimeSummary {
                run_id: Some("run-001".to_string()),
                run_status: "completed".to_string(),
                branch_name: Some(format!("agentflow/project-release/{issue_id}")),
                checkpoint_count: 1,
                latest_checkpoint_id: Some("checkpoint-001".to_string()),
                latest_checkpoint_state: Some(state.to_string()),
                latest_checkpoint_summary: Some("交付回放测试。".to_string()),
            },
            delivery: agentflow_projection::ProjectionDeliverySummary {
                status: "published".to_string(),
                evidence_status: "ready".to_string(),
                evidence_path: Some(format!(
                    ".agentflow/tasks/{issue_id}/evidence/evidence.json"
                )),
                pr_url: Some(format!("https://github.com/acme/repo/pull/{updated_at}")),
                merge_commit: Some(format!("merge-{updated_at}")),
                public_record_path: None,
            },
            audit: agentflow_projection::ProjectionAuditSummary {
                status: "not-requested".to_string(),
                latest_audit_id: None,
                report_path: None,
                requested_at: None,
            },
            updated_at,
        };
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
