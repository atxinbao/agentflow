use crate::model::{
    EventActor, ReplayFilter, TaskEvent, TaskEventDraft, TaskEventManifest, TaskEventSummary,
    TASK_EVENT_MANIFEST_VERSION, TASK_EVENT_STREAM_PATH, TASK_EVENT_VERSION,
};
use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

const LEGACY_WORKFLOW_STREAM_PATH: &str = ".agentflow/events/stream.jsonl";

pub fn prepare_event_store(project_root: impl AsRef<Path>) -> Result<TaskEventManifest> {
    let root = canonical_project_root(project_root)?;
    ensure_directory(&root.join(".agentflow/events"))?;
    let stream_path = root.join(TASK_EVENT_STREAM_PATH);
    if !stream_path.exists() {
        fs::write(&stream_path, "")?;
    }
    let events = load_task_events(&root)?;
    let manifest = TaskEventManifest {
        version: TASK_EVENT_MANIFEST_VERSION.to_string(),
        project_root: root.display().to_string(),
        stream_path: TASK_EVENT_STREAM_PATH.to_string(),
        summary: TaskEventSummary {
            events: events.len(),
            imported_legacy_events: events
                .iter()
                .filter(|event| event.payload.get("legacyWorkflowEventId").is_some())
                .count(),
        },
    };
    write_json(
        &root.join(".agentflow/events/task-events.manifest.json"),
        &manifest,
    )?;
    Ok(manifest)
}

pub fn append_task_event(
    project_root: impl AsRef<Path>,
    draft: TaskEventDraft,
) -> Result<TaskEvent> {
    let root = canonical_project_root(project_root)?;
    prepare_event_store(&root)?;
    validate_draft(&draft)?;
    let event = materialize_event(&root, draft)?;
    append_event_line(&root, &event)?;
    let _ = prepare_event_store(&root);
    Ok(event)
}

pub fn append_task_event_once(
    project_root: impl AsRef<Path>,
    draft: TaskEventDraft,
) -> Result<TaskEvent> {
    let root = canonical_project_root(project_root)?;
    prepare_event_store(&root)?;
    if let Some(key) = draft.idempotency_key.as_ref() {
        if let Some(existing) = load_task_events(&root)?
            .into_iter()
            .find(|event| event.idempotency_key.as_ref() == Some(key))
        {
            return Ok(existing);
        }
    }
    append_task_event(root, draft)
}

pub fn load_task_events(project_root: impl AsRef<Path>) -> Result<Vec<TaskEvent>> {
    let root = canonical_project_root(project_root)?;
    let stream_path = root.join(TASK_EVENT_STREAM_PATH);
    if !stream_path.is_file() {
        return Ok(Vec::new());
    }
    let raw = fs::read_to_string(&stream_path)
        .with_context(|| format!("read {}", stream_path.display()))?;
    raw.lines()
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
        .map(|(index, line)| {
            serde_json::from_str(line)
                .with_context(|| format!("parse task event line {}", index + 1))
        })
        .collect()
}

pub fn replay_task_events(
    project_root: impl AsRef<Path>,
    filter: ReplayFilter,
) -> Result<Vec<TaskEvent>> {
    let mut has_seen_after_event = filter.after_event_id.is_none();
    let allowed_types = filter.event_types;
    let events = load_task_events(project_root)?;
    let mut replayed = Vec::new();
    for event in events {
        if !has_seen_after_event {
            has_seen_after_event = filter.after_event_id.as_ref() == Some(&event.event_id);
            continue;
        }
        if let Some(expected) = filter.aggregate_type.as_ref() {
            if &event.aggregate_type != expected {
                continue;
            }
        }
        if let Some(expected) = filter.aggregate_id.as_ref() {
            if &event.aggregate_id != expected {
                continue;
            }
        }
        if let Some(expected) = filter.issue_id.as_ref() {
            if event.issue_id.as_ref() != Some(expected) {
                continue;
            }
        }
        if let Some(expected) = filter.project_id.as_ref() {
            if event.project_id.as_ref() != Some(expected) {
                continue;
            }
        }
        if !allowed_types.is_empty() && !allowed_types.contains(&event.event_type) {
            continue;
        }
        replayed.push(event);
    }
    Ok(replayed)
}

pub fn import_legacy_workflow_events(project_root: impl AsRef<Path>) -> Result<Vec<TaskEvent>> {
    let root = canonical_project_root(project_root)?;
    prepare_event_store(&root)?;
    let legacy_path = root.join(LEGACY_WORKFLOW_STREAM_PATH);
    if !legacy_path.is_file() {
        return Ok(Vec::new());
    }
    let existing = load_task_events(&root)?;
    let raw = fs::read_to_string(&legacy_path)
        .with_context(|| format!("read {}", legacy_path.display()))?;
    let mut imported = Vec::new();
    for (index, line) in raw.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let legacy: Value = serde_json::from_str(line)
            .with_context(|| format!("parse legacy workflow event line {}", index + 1))?;
        let legacy_event_id = legacy
            .get("eventId")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        let idempotency_key = format!("legacy-workflow-event:{legacy_event_id}");
        if existing
            .iter()
            .chain(imported.iter())
            .any(|event| event.idempotency_key.as_deref() == Some(idempotency_key.as_str()))
        {
            continue;
        }
        let subject_id = legacy
            .get("subjectId")
            .and_then(Value::as_str)
            .unwrap_or(legacy_event_id);
        let event_type = legacy
            .get("eventType")
            .and_then(Value::as_str)
            .unwrap_or("legacy.workflow-event.imported");
        let source = legacy
            .get("source")
            .and_then(Value::as_str)
            .unwrap_or("workflow-events");
        let draft = TaskEventDraft {
            aggregate_type: "legacy-workflow-event".to_string(),
            aggregate_id: subject_id.to_string(),
            project_id: None,
            issue_id: Some(subject_id.to_string()),
            event_type: event_type.to_string(),
            actor: EventActor {
                role: source.to_string(),
                kind: "legacy".to_string(),
            },
            state: None,
            correlation_id: Some(format!("corr-{subject_id}")),
            causation_id: None,
            payload: json!({
                "legacyWorkflowEventId": legacy_event_id,
                "legacyWorkflowEvent": legacy,
            }),
            artifact_refs: Vec::new(),
            idempotency_key: Some(idempotency_key),
        };
        imported.push(append_task_event_once(&root, draft)?);
    }
    let _ = prepare_event_store(&root);
    Ok(imported)
}

fn materialize_event(root: &Path, draft: TaskEventDraft) -> Result<TaskEvent> {
    let timestamp = unix_timestamp_seconds();
    let event_id = next_event_id(root, timestamp)?;
    Ok(TaskEvent {
        event_id,
        event_version: TASK_EVENT_VERSION.to_string(),
        aggregate_type: draft.aggregate_type,
        aggregate_id: draft.aggregate_id.clone(),
        project_id: draft.project_id,
        issue_id: draft.issue_id,
        event_type: draft.event_type,
        timestamp,
        actor: draft.actor,
        state: draft.state,
        correlation_id: draft
            .correlation_id
            .unwrap_or_else(|| format!("corr-{}", draft.aggregate_id)),
        causation_id: draft.causation_id,
        payload: draft.payload,
        artifact_refs: draft.artifact_refs,
        idempotency_key: draft.idempotency_key,
    })
}

fn append_event_line(root: &Path, event: &TaskEvent) -> Result<()> {
    let line = serde_json::to_string(event)?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(root.join(TASK_EVENT_STREAM_PATH))
        .with_context(|| format!("open {}", root.join(TASK_EVENT_STREAM_PATH).display()))?;
    writeln!(file, "{line}")?;
    Ok(())
}

fn validate_draft(draft: &TaskEventDraft) -> Result<()> {
    validate_required("aggregateType", &draft.aggregate_type)?;
    validate_required("aggregateId", &draft.aggregate_id)?;
    validate_required("type", &draft.event_type)?;
    validate_required("actor.role", &draft.actor.role)?;
    validate_required("actor.kind", &draft.actor.kind)?;
    if draft.project_id.is_none() && draft.issue_id.is_none() {
        anyhow::bail!("task event should reference projectId or issueId");
    }
    Ok(())
}

fn validate_required(field: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        anyhow::bail!("{field} is required");
    }
    Ok(())
}

fn next_event_id(root: &Path, timestamp: u64) -> Result<String> {
    Ok(format!(
        "evt-{}-{:06}",
        timestamp,
        load_task_events(root)?.len() + 1
    ))
}

fn canonical_project_root(project_root: impl AsRef<Path>) -> Result<PathBuf> {
    project_root
        .as_ref()
        .canonicalize()
        .with_context(|| format!("canonicalize {}", project_root.as_ref().display()))
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

fn unix_timestamp_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::EventStateTransition;
    use tempfile::tempdir;

    fn issue_scheduled_draft(issue_id: &str) -> TaskEventDraft {
        TaskEventDraft {
            aggregate_type: "issue".to_string(),
            aggregate_id: issue_id.to_string(),
            project_id: Some("project-task-workflow-v1".to_string()),
            issue_id: Some(issue_id.to_string()),
            event_type: "issue.scheduled".to_string(),
            actor: EventActor {
                role: "task-loop".to_string(),
                kind: "system".to_string(),
            },
            state: Some(EventStateTransition {
                from_state: "backlog".to_string(),
                to_state: "todo".to_string(),
            }),
            correlation_id: None,
            causation_id: None,
            payload: json!({"workflowRef": "build-agent.issue-loop@v1"}),
            artifact_refs: vec![".agentflow/panel/context-packs/AF-TASK-001.json".to_string()],
            idempotency_key: Some(format!("issue.scheduled:{issue_id}")),
        }
    }

    #[test]
    fn appends_and_loads_task_events_jsonl() {
        let dir = tempdir().unwrap();

        let event = append_task_event(dir.path(), issue_scheduled_draft("AF-TASK-001")).unwrap();
        let events = load_task_events(dir.path()).unwrap();

        assert_eq!(events.len(), 1);
        assert_eq!(events[0], event);
        assert!(event.event_id.starts_with("evt-"));
        assert_eq!(event.event_version, TASK_EVENT_VERSION);
        assert_eq!(event.correlation_id, "corr-AF-TASK-001");
        assert!(dir.path().join(TASK_EVENT_STREAM_PATH).is_file());
    }

    #[test]
    fn append_once_uses_idempotency_key() {
        let dir = tempdir().unwrap();
        let draft = issue_scheduled_draft("AF-TASK-001");

        let first = append_task_event_once(dir.path(), draft.clone()).unwrap();
        let second = append_task_event_once(dir.path(), draft).unwrap();

        assert_eq!(first.event_id, second.event_id);
        assert_eq!(load_task_events(dir.path()).unwrap().len(), 1);
    }

    #[test]
    fn replay_filters_by_issue_and_after_event() {
        let dir = tempdir().unwrap();
        let first = append_task_event(dir.path(), issue_scheduled_draft("AF-TASK-001")).unwrap();
        append_task_event(dir.path(), issue_scheduled_draft("AF-TASK-002")).unwrap();
        let mut third = issue_scheduled_draft("AF-TASK-001");
        third.event_type = "issue.started".to_string();
        third.causation_id = Some(first.event_id.clone());
        append_task_event(dir.path(), third).unwrap();

        let mut filter = ReplayFilter::issue("AF-TASK-001");
        filter.after_event_id = Some(first.event_id);
        let events = replay_task_events(dir.path(), filter).unwrap();

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "issue.started");
    }

    #[test]
    fn imports_legacy_workflow_event_stream() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".agentflow/events")).unwrap();
        fs::write(
            dir.path().join(LEGACY_WORKFLOW_STREAM_PATH),
            r#"{"eventId":"evt-old-0001","eventType":"input.issue.ready","source":"input","subjectId":"AF-TASK-001","payload":{"title":"Old"}}"#,
        )
        .unwrap();

        let imported = import_legacy_workflow_events(dir.path()).unwrap();
        let second_import = import_legacy_workflow_events(dir.path()).unwrap();

        assert_eq!(imported.len(), 1);
        assert!(second_import.is_empty());
        assert_eq!(imported[0].aggregate_type, "legacy-workflow-event");
        assert_eq!(imported[0].issue_id.as_deref(), Some("AF-TASK-001"));
    }

    #[test]
    fn draft_requires_project_or_issue_reference() {
        let dir = tempdir().unwrap();
        let mut draft = issue_scheduled_draft("AF-TASK-001");
        draft.project_id = None;
        draft.issue_id = None;

        let err = append_task_event(dir.path(), draft)
            .unwrap_err()
            .to_string();

        assert!(err.contains("projectId or issueId"));
    }
}
