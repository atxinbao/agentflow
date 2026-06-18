use crate::model::{
    ProjectReleaseFacts, ProjectReleaseIndex, ProjectReleaseIndexEntry,
    PublicReleaseDocumentTarget, PROJECT_RELEASE_FACTS_VERSION, PROJECT_RELEASE_INDEX_VERSION,
};
use crate::public_delivery::{
    collect_public_release_summary_for_project, write_public_release_documents,
};
use crate::review_surface::sync_project_external_review_surface;
use agentflow_event_store::EventActor;
use agentflow_workflow_core::{canonicalize_project_root, join_relative_path, ProjectId};
use agentflow_workflow_runtime::{
    apply_canonical_workflow_event, RuntimeContext, StaticActionRegistry, StaticGuardRegistry,
    WorkflowFlowType,
};
use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::json;
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProjectProjectionSnapshot {
    project_id: String,
    title: String,
    #[serde(default)]
    completion: Option<CompletionProjectionSnapshot>,
    #[serde(default)]
    delivery: Option<ProjectDeliverySnapshot>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CompletionProjectionSnapshot {
    current_state: String,
    latest_outcome: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProjectDeliverySnapshot {
    status: String,
    #[serde(default)]
    missing_count: usize,
    #[serde(default)]
    summary_line: String,
}

#[derive(Debug, Clone)]
struct ReleaseGate {
    current_state: String,
    gate_status: String,
    gate_reason: String,
    completion_state: String,
    completion_outcome: Option<String>,
    delivery_status: String,
}

#[derive(Debug, Clone)]
struct ReleaseRuntimeContext {
    root: PathBuf,
    project_id: String,
    projection: ProjectProjectionSnapshot,
    target: PublicReleaseDocumentTarget,
    gate: ReleaseGate,
    existing: Option<ProjectReleaseFacts>,
}

pub fn prepare_project_release(
    project_root: impl AsRef<Path>,
    project_id: impl AsRef<str>,
) -> Result<ProjectReleaseFacts> {
    let context = build_release_runtime_context(project_root, project_id)?;
    let now = unix_timestamp_seconds();

    if context
        .existing
        .as_ref()
        .is_some_and(|facts| facts.current_state == "published")
    {
        return refresh_published_release(&context, now);
    }

    if context.gate.gate_status != "ready" {
        return write_release_gate_state(
            &context,
            context.gate.current_state.clone(),
            context.gate.gate_status.clone(),
            context.gate.gate_reason.clone(),
            0,
            context
                .existing
                .as_ref()
                .and_then(|facts| facts.latest_event_id.clone()),
            context
                .existing
                .as_ref()
                .and_then(|facts| facts.published_at),
            now,
        );
    }

    let summary = collect_public_release_summary_for_project(
        &context.root,
        Some(context.project_id.as_str()),
    )?;
    if summary.entries.is_empty() {
        return write_release_gate_state(
            &context,
            "blocked".to_string(),
            "blocked".to_string(),
            "项目已进入 release 阶段，但当前还没有可汇总的已完成任务交付。".to_string(),
            0,
            context
                .existing
                .as_ref()
                .and_then(|facts| facts.latest_event_id.clone()),
            context
                .existing
                .as_ref()
                .and_then(|facts| facts.published_at),
            now,
        );
    }

    let existing_state = context
        .existing
        .as_ref()
        .map(|facts| facts.current_state.as_str())
        .unwrap_or("pending");
    let current_state = match existing_state {
        "in_progress" => "in_progress",
        "ready" => "ready",
        "published" => "published",
        _ => "pending",
    };
    let mut latest_event_id = context
        .existing
        .as_ref()
        .and_then(|facts| facts.latest_event_id.clone());
    if current_state == "pending" {
        latest_event_id = transition_release_state(
            &context.root,
            &context.project_id,
            "pending",
            "delivery.ready",
            &context.target,
            summary.entries.len(),
            &context.gate,
        )?
        .or(latest_event_id);
    }

    let persisted_state = if current_state == "in_progress" {
        "in_progress"
    } else {
        "ready"
    };
    let persisted_reason = if persisted_state == "in_progress" {
        "Release 说明已确认，正在等待正式发布。"
    } else {
        "Release 已就绪，下一步可以确认并生成公开说明。"
    };

    write_release_gate_state(
        &context,
        persisted_state.to_string(),
        "ready".to_string(),
        persisted_reason.to_string(),
        summary.entries.len(),
        latest_event_id,
        context
            .existing
            .as_ref()
            .and_then(|facts| facts.published_at),
        now,
    )
}

pub fn confirm_project_release(
    project_root: impl AsRef<Path>,
    project_id: impl AsRef<str>,
) -> Result<ProjectReleaseFacts> {
    let context = build_release_runtime_context(project_root, project_id)?;
    let now = unix_timestamp_seconds();

    if context
        .existing
        .as_ref()
        .is_some_and(|facts| facts.current_state == "published")
    {
        return refresh_published_release(&context, now);
    }

    let prepared = prepare_project_release(&context.root, &context.project_id)?;
    if prepared.current_state != "ready" && prepared.current_state != "in_progress" {
        anyhow::bail!(
            "release confirm requires ready state, found {}",
            prepared.current_state
        );
    }
    if prepared.current_state == "in_progress" {
        return Ok(prepared);
    }

    let summary = collect_public_release_summary_for_project(
        &context.root,
        Some(context.project_id.as_str()),
    )?;
    let paths = write_public_release_documents(&context.root, &summary, &context.target)?;
    let latest_event_id = transition_release_state(
        &context.root,
        &context.project_id,
        "ready",
        "delivery.started",
        &PublicReleaseDocumentTarget {
            changelog_path: PathBuf::from(paths.changelog_path.clone()),
            release_notes_path: PathBuf::from(paths.release_notes_path.clone()),
        },
        summary.entries.len(),
        &context.gate,
    )?
    .or(prepared.latest_event_id.clone());

    write_release_gate_state(
        &context,
        "in_progress".to_string(),
        "ready".to_string(),
        "Release 说明已确认，公开记录已生成，下一步可以正式发布。".to_string(),
        summary.entries.len(),
        latest_event_id,
        prepared.published_at,
        now,
    )
}

pub fn publish_project_release(
    project_root: impl AsRef<Path>,
    project_id: impl AsRef<str>,
) -> Result<ProjectReleaseFacts> {
    let context = build_release_runtime_context(project_root, project_id)?;
    let now = unix_timestamp_seconds();

    if context
        .existing
        .as_ref()
        .is_some_and(|facts| facts.current_state == "published")
    {
        return refresh_published_release(&context, now);
    }

    let confirmed = confirm_project_release(&context.root, &context.project_id)?;
    if confirmed.current_state != "in_progress" {
        anyhow::bail!(
            "release publish requires in_progress state, found {}",
            confirmed.current_state
        );
    }

    let summary = collect_public_release_summary_for_project(
        &context.root,
        Some(context.project_id.as_str()),
    )?;
    let paths = write_public_release_documents(&context.root, &summary, &context.target)?;
    let latest_event_id = transition_release_state(
        &context.root,
        &context.project_id,
        "in_progress",
        "delivery.published",
        &PublicReleaseDocumentTarget {
            changelog_path: PathBuf::from(paths.changelog_path.clone()),
            release_notes_path: PathBuf::from(paths.release_notes_path.clone()),
        },
        summary.entries.len(),
        &context.gate,
    )?
    .or(confirmed.latest_event_id.clone());

    write_release_gate_state(
        &context,
        "published".to_string(),
        "ready".to_string(),
        "Release 已发布。".to_string(),
        summary.entries.len(),
        latest_event_id,
        Some(now),
        now,
    )
}

pub fn sync_project_release(
    project_root: impl AsRef<Path>,
    project_id: impl AsRef<str>,
) -> Result<ProjectReleaseFacts> {
    let prepared = prepare_project_release(&project_root, project_id.as_ref())?;
    if prepared.current_state != "ready" {
        return Ok(prepared);
    }
    let confirmed = confirm_project_release(&project_root, project_id.as_ref())?;
    if confirmed.current_state != "in_progress" {
        return Ok(confirmed);
    }
    publish_project_release(project_root, project_id)
}

pub fn load_project_release_facts(
    project_root: impl AsRef<Path>,
    project_id: &str,
) -> Result<ProjectReleaseFacts> {
    let root = canonicalize_project_root(project_root)?;
    read_json(&project_release_facts_path(&root, project_id)?)
}

pub fn load_project_release_index(project_root: impl AsRef<Path>) -> Result<ProjectReleaseIndex> {
    let root = canonicalize_project_root(project_root)?;
    read_json(&root.join(".agentflow/indexes/releases.json"))
}

fn build_release_runtime_context(
    project_root: impl AsRef<Path>,
    project_id: impl AsRef<str>,
) -> Result<ReleaseRuntimeContext> {
    let root = canonicalize_project_root(project_root)?;
    let project_id = ProjectId::parse(project_id.as_ref())?;
    let projection = load_project_projection_snapshot(&root, project_id.as_str())?;
    let target = default_project_release_target(project_id.as_str())?;
    let existing = load_project_release_facts(&root, project_id.as_str()).ok();
    let gate = evaluate_release_gate(&projection);
    Ok(ReleaseRuntimeContext {
        root,
        project_id: project_id.as_str().to_string(),
        projection,
        target,
        gate,
        existing,
    })
}

fn refresh_published_release(
    context: &ReleaseRuntimeContext,
    now: u64,
) -> Result<ProjectReleaseFacts> {
    let summary = collect_public_release_summary_for_project(
        &context.root,
        Some(context.project_id.as_str()),
    )?;
    let paths = write_public_release_documents(&context.root, &summary, &context.target)?;
    write_release_gate_state(
        context,
        "published".to_string(),
        "ready".to_string(),
        "Release 已经发布。".to_string(),
        summary.entries.len(),
        context
            .existing
            .as_ref()
            .and_then(|facts| facts.latest_event_id.clone()),
        context
            .existing
            .as_ref()
            .and_then(|facts| facts.published_at)
            .or(Some(now)),
        now,
    )
    .map(|mut facts| {
        facts.changelog_path = paths.changelog_path;
        facts.release_notes_path = paths.release_notes_path;
        facts
    })
}

fn write_release_gate_state(
    context: &ReleaseRuntimeContext,
    current_state: String,
    gate_status: String,
    gate_reason: String,
    entry_count: usize,
    latest_event_id: Option<String>,
    published_at: Option<u64>,
    now: u64,
) -> Result<ProjectReleaseFacts> {
    let summary_line = match current_state.as_str() {
        "published" => format!(
            "Release 已发布，公开记录写入 {} 和 {}。",
            context.target.changelog_path.display(),
            context.target.release_notes_path.display()
        ),
        "in_progress" => "Release 已确认，正在等待正式发布。".to_string(),
        "ready" => "Release 已准备好，等待确认发布内容。".to_string(),
        "blocked" => format!("Release 已阻断：{}。", gate_reason),
        _ => gate_reason.clone(),
    };

    let facts = ProjectReleaseFacts {
        version: PROJECT_RELEASE_FACTS_VERSION.to_string(),
        project_id: context.projection.project_id.clone(),
        project_title: context.projection.title.clone(),
        current_state,
        gate_status,
        gate_reason,
        completion_state: context.gate.completion_state.clone(),
        completion_outcome: context.gate.completion_outcome.clone(),
        delivery_status: context.gate.delivery_status.clone(),
        changelog_path: context.target.changelog_path.display().to_string(),
        release_notes_path: context.target.release_notes_path.display().to_string(),
        entry_count,
        summary_line,
        latest_event_id,
        published_at,
        updated_at: now,
    };
    write_project_release_facts(&context.root, &facts)?;
    write_project_release_index(&context.root)?;
    sync_project_external_review_surface(&context.root, &facts)?;
    Ok(facts)
}

fn transition_release_state(
    root: &Path,
    project_id: &str,
    current_state: &str,
    event_type: &str,
    target: &PublicReleaseDocumentTarget,
    entry_count: usize,
    gate: &ReleaseGate,
) -> Result<Option<String>> {
    let guards = match event_type {
        "delivery.ready" => StaticGuardRegistry::all_pass(["delivery.input.ready"]),
        "delivery.started" => StaticGuardRegistry::all_pass(["delivery.public_record.ready"]),
        "delivery.published" => StaticGuardRegistry::all_pass(["delivery.publish.confirmed"]),
        _ => StaticGuardRegistry::default(),
    };
    let actions = match event_type {
        "delivery.ready" => StaticActionRegistry::all_complete(["delivery.ready.write"]),
        "delivery.started" => StaticActionRegistry::all_complete(["delivery.summary.write"]),
        "delivery.published" => StaticActionRegistry::all_complete(["delivery.publish.write"]),
        _ => StaticActionRegistry::default(),
    };
    let payload = json!({
        "projectId": project_id,
        "completionState": gate.completion_state,
        "completionOutcome": gate.completion_outcome,
        "deliveryStatus": gate.delivery_status,
        "changelogPath": target.changelog_path.display().to_string(),
        "releaseNotesUrl": target.release_notes_path.display().to_string(),
        "releaseEntryCount": entry_count,
    });
    let transition = apply_canonical_workflow_event(
        root,
        WorkflowFlowType::Delivery,
        current_state,
        event_type,
        RuntimeContext {
            actor: EventActor {
                role: "release-runtime".to_string(),
                kind: "system".to_string(),
            },
            payload,
            ..RuntimeContext::project(
                project_id.to_string(),
                EventActor {
                    role: "release-runtime".to_string(),
                    kind: "system".to_string(),
                },
            )
        },
        &guards,
        &actions,
    )?;
    Ok(transition.event_id)
}

fn evaluate_release_gate(projection: &ProjectProjectionSnapshot) -> ReleaseGate {
    let completion_state = projection
        .completion
        .as_ref()
        .map(|completion| completion.current_state.clone())
        .unwrap_or_else(|| "missing".to_string());
    let completion_outcome = projection
        .completion
        .as_ref()
        .and_then(|completion| completion.latest_outcome.clone());
    let delivery_status = projection
        .delivery
        .as_ref()
        .map(|delivery| delivery.status.clone())
        .unwrap_or_else(|| "missing".to_string());

    let completion_accepted = projection.completion.as_ref().is_some_and(|completion| {
        completion.current_state == "accepted"
            || completion.latest_outcome.as_deref() == Some("accept")
    });
    if !completion_accepted {
        return ReleaseGate {
            current_state: "pending".to_string(),
            gate_status: "waiting-completion".to_string(),
            gate_reason: "项目还没有通过 completion accept，release 不能开始。".to_string(),
            completion_state,
            completion_outcome,
            delivery_status,
        };
    }

    let delivery = projection.delivery.as_ref();
    if delivery.is_none() {
        return ReleaseGate {
            current_state: "blocked".to_string(),
            gate_status: "blocked".to_string(),
            gate_reason: "项目缺少公开交付摘要，release 不能开始。".to_string(),
            completion_state,
            completion_outcome,
            delivery_status,
        };
    }
    let delivery = delivery.unwrap();
    if delivery.missing_count > 0 {
        return ReleaseGate {
            current_state: "blocked".to_string(),
            gate_status: "blocked".to_string(),
            gate_reason: if delivery.summary_line.trim().is_empty() {
                "项目公开交付记录还没整理完整。".to_string()
            } else {
                delivery.summary_line.clone()
            },
            completion_state,
            completion_outcome,
            delivery_status,
        };
    }

    ReleaseGate {
        current_state: "ready".to_string(),
        gate_status: "ready".to_string(),
        gate_reason: "项目已经完成 completion accept，公开交付记录也已齐备，可以进入正式 release。"
            .to_string(),
        completion_state,
        completion_outcome,
        delivery_status,
    }
}

fn load_project_projection_snapshot(
    root: &Path,
    project_id: &str,
) -> Result<ProjectProjectionSnapshot> {
    let project_id = ProjectId::parse(project_id)?;
    read_json(&join_relative_path(
        root,
        PathBuf::from(".agentflow")
            .join("projections")
            .join("projects")
            .join(format!("{}.json", project_id.as_str())),
    )?)
}

fn write_project_release_facts(root: &Path, facts: &ProjectReleaseFacts) -> Result<()> {
    ensure_directory(&root.join(".agentflow/release/projects"))?;
    write_json(&project_release_facts_path(root, &facts.project_id)?, facts)
}

fn write_project_release_index(root: &Path) -> Result<()> {
    ensure_directory(&root.join(".agentflow/release/projects"))?;
    ensure_directory(&root.join(".agentflow/indexes"))?;
    let mut releases = Vec::new();
    for entry in fs::read_dir(root.join(".agentflow/release/projects"))
        .with_context(|| "read .agentflow/release/projects".to_string())?
    {
        let entry = entry?;
        if entry.path().extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let facts: ProjectReleaseFacts = read_json(&entry.path())?;
        releases.push(ProjectReleaseIndexEntry {
            project_id: facts.project_id,
            current_state: facts.current_state,
            gate_status: facts.gate_status,
            changelog_path: facts.changelog_path,
            release_notes_path: facts.release_notes_path,
            published_at: facts.published_at,
            updated_at: facts.updated_at,
        });
    }
    releases.sort_by(|left, right| left.project_id.cmp(&right.project_id));
    write_json(
        &root.join(".agentflow/indexes/releases.json"),
        &ProjectReleaseIndex {
            version: PROJECT_RELEASE_INDEX_VERSION.to_string(),
            updated_at: unix_timestamp_seconds(),
            releases,
        },
    )
}

fn project_release_facts_path(root: &Path, project_id: &str) -> Result<PathBuf> {
    let project_id = ProjectId::parse(project_id)?;
    join_relative_path(
        root,
        PathBuf::from(".agentflow")
            .join("release")
            .join("projects")
            .join(format!("{}.json", project_id.as_str())),
    )
}

fn default_project_release_target(project_id: &str) -> Result<PublicReleaseDocumentTarget> {
    let project_id = ProjectId::parse(project_id)?;
    Ok(PublicReleaseDocumentTarget {
        changelog_path: PathBuf::from("CHANGELOG.md"),
        release_notes_path: PathBuf::from(format!("docs/release-notes/{}.md", project_id.as_str())),
    })
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

#[cfg(test)]
mod tests {
    use super::*;
    use agentflow_event_store::load_task_events;
    use agentflow_spec::{
        issue_from_requirement, project_from_requirement, write_spec_issue, write_spec_project,
        SpecIssueDraft, SpecProjectDraft,
    };
    use tempfile::tempdir;

    fn write_requirement(root: &Path) -> PathBuf {
        let path = root.join("docs/requirements/054-release-runtime-test.md");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(
            &path,
            "# Release Runtime Test\n\n用于 release runtime 测试。\n",
        )
        .unwrap();
        path
    }

    fn write_project_fixture(root: &Path) -> String {
        let requirement = write_requirement(root);
        let mut project_draft = SpecProjectDraft::new("project-release-runtime");
        project_draft.title = Some("Release Runtime Project".to_string());
        project_draft.summary = Some("release runtime test".to_string());
        project_draft.objective = Some("验证 release runtime".to_string());
        project_draft.issue_ids = vec!["AF-REL-001".to_string()];
        let project = project_from_requirement(root, &requirement, project_draft).unwrap();
        write_spec_project(root, &project).unwrap();

        let mut issue_draft = SpecIssueDraft::new("AF-REL-001");
        issue_draft.project_id = Some(project.project_id.clone());
        issue_draft.title = Some("完成公开交付".to_string());
        issue_draft.summary = Some("整理公开交付记录。".to_string());
        let issue = issue_from_requirement(root, &requirement, issue_draft).unwrap();
        write_spec_issue(root, &issue).unwrap();
        project.project_id
    }

    fn write_project_projection(
        root: &Path,
        project_id: &str,
        completion_state: &str,
        completion_outcome: Option<&str>,
        delivery_status: &str,
        missing_count: usize,
    ) {
        let projection = serde_json::json!({
            "projectId": project_id,
            "title": "Release Runtime Project",
            "completion": {
                "currentState": completion_state,
                "latestOutcome": completion_outcome,
            },
            "delivery": {
                "status": delivery_status,
                "missingCount": missing_count,
                "summaryLine": if missing_count == 0 { "公开交付已统一写入 PR/MR body、CHANGELOG.md。"} else {"项目公开交付仍缺少 CHANGELOG.md 或 release notes。"}
            }
        });
        let path = root.join(".agentflow/projections/projects");
        fs::create_dir_all(&path).unwrap();
        fs::write(
            path.join(format!("{project_id}.json")),
            serde_json::to_string_pretty(&projection).unwrap(),
        )
        .unwrap();
    }

    fn write_task_projection(root: &Path, issue_id: &str, project_id: &str) {
        let projection = serde_json::json!({
            "issueId": issue_id,
            "projectId": project_id,
            "currentState": "done",
            "publicDelivery": {
                "prUrl": "https://github.com/acme/repo/pull/1",
                "mergeCommit": "merge-1",
                "changelogPath": "CHANGELOG.md",
                "releaseNotesUrl": "docs/release-notes/project-release-runtime.md"
            },
            "delivery": {
                "status": "published",
                "evidenceStatus": "ready",
                "evidencePath": ".agentflow/tasks/AF-REL-001/evidence/evidence.json",
                "prUrl": "https://github.com/acme/repo/pull/1",
                "mergeCommit": "merge-1",
                "publicRecordPath": "CHANGELOG.md"
            },
            "updatedAt": 10
        });
        let path = root.join(".agentflow/projections/tasks");
        fs::create_dir_all(&path).unwrap();
        fs::write(
            path.join(format!("{issue_id}.json")),
            serde_json::to_string_pretty(&projection).unwrap(),
        )
        .unwrap();
    }

    #[test]
    fn sync_project_release_publishes_release_facts_and_public_docs() {
        let dir = tempdir().unwrap();
        let project_id = write_project_fixture(dir.path());
        write_task_projection(dir.path(), "AF-REL-001", &project_id);
        write_project_projection(
            dir.path(),
            &project_id,
            "accepted",
            Some("accept"),
            "published",
            0,
        );

        let facts = sync_project_release(dir.path(), &project_id).unwrap();
        let events = load_task_events(dir.path()).unwrap();
        let delivery_events = events
            .into_iter()
            .filter(|event| event.project_id.as_deref() == Some(project_id.as_str()))
            .map(|event| event.event_type)
            .collect::<Vec<_>>();

        assert_eq!(facts.current_state, "published");
        assert_eq!(facts.gate_status, "ready");
        assert_eq!(facts.entry_count, 1);
        assert_eq!(
            delivery_events,
            vec![
                "delivery.ready".to_string(),
                "delivery.started".to_string(),
                "delivery.published".to_string(),
            ]
        );
        assert!(dir.path().join("CHANGELOG.md").is_file());
        assert!(dir
            .path()
            .join("docs/release-notes/project-release-runtime.md")
            .is_file());
        assert!(dir
            .path()
            .join("docs/reviews/project-release-runtime.md")
            .is_file());
        assert!(dir
            .path()
            .join(".agentflow/release/reviews/project-release-runtime.json")
            .is_file());
        let review =
            crate::review_surface::load_project_external_review_surface(dir.path(), &project_id)
                .unwrap();
        assert_eq!(review.review_status, "ready");
        assert_eq!(review.total_entries, 1);
        let index = load_project_release_index(dir.path()).unwrap();
        assert_eq!(index.releases.len(), 1);
    }

    #[test]
    fn sync_project_release_blocks_when_completion_is_not_accepted() {
        let dir = tempdir().unwrap();
        let project_id = write_project_fixture(dir.path());
        write_task_projection(dir.path(), "AF-REL-001", &project_id);
        write_project_projection(
            dir.path(),
            &project_id,
            "continue",
            Some("continue"),
            "ready",
            0,
        );

        let facts = sync_project_release(dir.path(), &project_id).unwrap();

        assert_eq!(facts.current_state, "pending");
        assert_eq!(facts.gate_status, "waiting-completion");
        assert!(facts.gate_reason.contains("completion accept"));
    }

    #[test]
    fn sync_project_release_blocks_when_public_delivery_is_incomplete() {
        let dir = tempdir().unwrap();
        let project_id = write_project_fixture(dir.path());
        write_task_projection(dir.path(), "AF-REL-001", &project_id);
        write_project_projection(
            dir.path(),
            &project_id,
            "accepted",
            Some("accept"),
            "ready",
            1,
        );

        let facts = sync_project_release(dir.path(), &project_id).unwrap();

        assert_eq!(facts.current_state, "blocked");
        assert_eq!(facts.gate_status, "blocked");
        assert!(facts.gate_reason.contains("公开交付"));
    }

    #[test]
    fn sync_project_release_rejects_path_like_project_ids() {
        let dir = tempdir().unwrap();
        let err = sync_project_release(dir.path(), "../bad")
            .unwrap_err()
            .to_string();
        assert!(err.contains("safe local id"));
    }
}
