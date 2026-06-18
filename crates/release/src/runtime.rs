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

pub fn sync_project_release(
    project_root: impl AsRef<Path>,
    project_id: impl AsRef<str>,
) -> Result<ProjectReleaseFacts> {
    let root = canonicalize_project_root(project_root)?;
    let project_id = ProjectId::parse(project_id.as_ref())?;
    let projection = load_project_projection_snapshot(&root, project_id.as_str())?;
    let target = default_project_release_target(project_id.as_str())?;
    let existing = load_project_release_facts(&root, project_id.as_str()).ok();
    let gate = evaluate_release_gate(&projection);

    let now = unix_timestamp_seconds();
    let mut current_state = existing
        .as_ref()
        .map(|facts| facts.current_state.clone())
        .unwrap_or_else(|| {
            if gate.gate_status == "ready" {
                "pending".to_string()
            } else {
                gate.current_state.clone()
            }
        });
    let mut latest_event_id = existing
        .as_ref()
        .and_then(|facts| facts.latest_event_id.clone());
    let mut published_at = existing.as_ref().and_then(|facts| facts.published_at);
    let mut entry_count = existing
        .as_ref()
        .map(|facts| facts.entry_count)
        .unwrap_or(0);

    if current_state == "published" {
        let summary = collect_public_release_summary_for_project(&root, Some(project_id.as_str()))?;
        let paths = write_public_release_documents(&root, &summary, &target)?;
        entry_count = summary.entries.len();
        published_at = published_at.or(Some(now));
        let facts = ProjectReleaseFacts {
            version: PROJECT_RELEASE_FACTS_VERSION.to_string(),
            project_id: projection.project_id,
            project_title: projection.title,
            current_state,
            gate_status: "ready".to_string(),
            gate_reason: "Release 已经发布。".to_string(),
            completion_state: gate.completion_state,
            completion_outcome: gate.completion_outcome,
            delivery_status: gate.delivery_status,
            changelog_path: paths.changelog_path,
            release_notes_path: paths.release_notes_path,
            entry_count,
            summary_line: format!(
                "Release 已发布，公开记录写入 {} 和 {}。",
                target.changelog_path.display(),
                target.release_notes_path.display()
            ),
            latest_event_id,
            published_at,
            updated_at: now,
        };
        write_project_release_facts(&root, &facts)?;
        write_project_release_index(&root)?;
        sync_project_external_review_surface(&root, &facts)?;
        return Ok(facts);
    }

    if gate.gate_status == "ready" {
        let summary = collect_public_release_summary_for_project(&root, Some(project_id.as_str()))?;
        if summary.entries.is_empty() {
            current_state = "blocked".to_string();
            let facts = ProjectReleaseFacts {
                version: PROJECT_RELEASE_FACTS_VERSION.to_string(),
                project_id: projection.project_id,
                project_title: projection.title,
                current_state,
                gate_status: "blocked".to_string(),
                gate_reason: "项目已进入 release 阶段，但当前还没有可汇总的已完成任务交付。"
                    .to_string(),
                completion_state: gate.completion_state,
                completion_outcome: gate.completion_outcome,
                delivery_status: gate.delivery_status,
                changelog_path: target.changelog_path.display().to_string(),
                release_notes_path: target.release_notes_path.display().to_string(),
                entry_count: 0,
                summary_line: "Release 已阻断，原因是没有可发布的任务交付。".to_string(),
                latest_event_id,
                published_at,
                updated_at: now,
            };
            write_project_release_facts(&root, &facts)?;
            write_project_release_index(&root)?;
            return Ok(facts);
        }

        if current_state == "blocked" {
            current_state = "pending".to_string();
        }
        if current_state == "pending" {
            latest_event_id = transition_release_state(
                &root,
                project_id.as_str(),
                "pending",
                "delivery.ready",
                &target,
                summary.entries.len(),
                &gate,
            )?
            .or(latest_event_id);
            current_state = "ready".to_string();
        }
        if current_state == "ready" {
            latest_event_id = transition_release_state(
                &root,
                project_id.as_str(),
                "ready",
                "delivery.started",
                &target,
                summary.entries.len(),
                &gate,
            )?
            .or(latest_event_id);
            current_state = "in_progress".to_string();
        }

        let paths = write_public_release_documents(&root, &summary, &target)?;
        entry_count = summary.entries.len();
        if current_state == "in_progress" {
            latest_event_id = transition_release_state(
                &root,
                project_id.as_str(),
                "in_progress",
                "delivery.published",
                &PublicReleaseDocumentTarget {
                    changelog_path: PathBuf::from(paths.changelog_path.clone()),
                    release_notes_path: PathBuf::from(paths.release_notes_path.clone()),
                },
                entry_count,
                &gate,
            )?
            .or(latest_event_id);
            current_state = "published".to_string();
            published_at = Some(now);
        }
    } else if current_state != "published" {
        current_state = gate.current_state.clone();
    }

    let summary_line = match current_state.as_str() {
        "published" => format!(
            "Release 已发布，公开记录写入 {} 和 {}。",
            target.changelog_path.display(),
            target.release_notes_path.display()
        ),
        "in_progress" => "Release 正在整理公开说明。".to_string(),
        "ready" => "Release 已就绪，下一步将生成公开说明。".to_string(),
        "blocked" => format!("Release 已阻断：{}。", gate.gate_reason),
        _ => gate.gate_reason.clone(),
    };

    let facts = ProjectReleaseFacts {
        version: PROJECT_RELEASE_FACTS_VERSION.to_string(),
        project_id: projection.project_id,
        project_title: projection.title,
        current_state,
        gate_status: gate.gate_status,
        gate_reason: gate.gate_reason,
        completion_state: gate.completion_state,
        completion_outcome: gate.completion_outcome,
        delivery_status: gate.delivery_status,
        changelog_path: target.changelog_path.display().to_string(),
        release_notes_path: target.release_notes_path.display().to_string(),
        entry_count,
        summary_line,
        latest_event_id,
        published_at,
        updated_at: now,
    };
    write_project_release_facts(&root, &facts)?;
    write_project_release_index(&root)?;
    sync_project_external_review_surface(&root, &facts)?;
    Ok(facts)
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
