use crate::model::{
    classify_task_event, ReplayFilter, TaskEvent, TaskEventConsumerState, TaskEventDeadLetter,
    TaskEventDraft, TaskEventManifest, TaskEventSummary, TaskReplayCursor,
    TASK_EVENT_CONSUMER_VERSION, TASK_EVENT_DEAD_LETTER_VERSION, TASK_EVENT_MANIFEST_VERSION,
    TASK_EVENT_STREAM_PATH, TASK_EVENT_VERSION,
};
use anyhow::{Context, Result};
use std::collections::BTreeSet;
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    thread,
    time::{SystemTime, UNIX_EPOCH},
};

const EVENT_STORE_LOCK_RETRY_LIMIT: usize = 200;
const EVENT_STORE_LOCK_RETRY_DELAY_MS: u64 = 10;

pub fn prepare_event_store(project_root: impl AsRef<Path>) -> Result<TaskEventManifest> {
    let root = canonical_project_root(project_root)?;
    ensure_directory(&root.join(".agentflow/events"))?;
    ensure_directory(&root.join(".agentflow/events/consumers"))?;
    ensure_directory(&root.join(".agentflow/events/dead-letter"))?;
    ensure_directory(&root.join(".agentflow/events/locks"))?;
    ensure_directory(&root.join(".agentflow/events/sequences"))?;
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
            consumers: count_json_files(&root.join(".agentflow/events/consumers"))?,
            dead_letters: count_json_files(&root.join(".agentflow/events/dead-letter"))?,
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
    let event = append_task_event_locked(&root, draft, false)?;
    let _ = prepare_event_store(&root);
    Ok(event)
}

pub fn append_task_event_once(
    project_root: impl AsRef<Path>,
    draft: TaskEventDraft,
) -> Result<TaskEvent> {
    let root = canonical_project_root(project_root)?;
    prepare_event_store(&root)?;
    let event = append_task_event_locked(&root, draft, true)?;
    let _ = prepare_event_store(&root);
    Ok(event)
}

pub fn allocate_task_sequence(project_root: impl AsRef<Path>, namespace: &str) -> Result<u64> {
    let root = canonical_project_root(project_root)?;
    prepare_event_store(&root)?;
    with_event_store_lock(&root, "sequence", |root| {
        next_sequence_value_unlocked(root, namespace)
    })
}

pub fn claim_task_event<F, G>(
    project_root: impl AsRef<Path>,
    selector: F,
    draft_builder: G,
) -> Result<Option<(TaskEvent, TaskEvent)>>
where
    F: Fn(&TaskEvent, &[TaskEvent]) -> bool,
    G: Fn(&TaskEvent, &[TaskEvent]) -> Result<TaskEventDraft>,
{
    let root = canonical_project_root(project_root)?;
    prepare_event_store(&root)?;
    let claimed = with_event_store_lock(&root, "claim", |root| {
        let events = load_task_events(root)?;
        let Some(requested) = events
            .iter()
            .find(|event| selector(event, &events))
            .cloned()
        else {
            return Ok(None);
        };
        let draft = draft_builder(&requested, &events)?;
        let claimed = append_task_event_once_unlocked(root, draft)?;
        Ok(Some((requested, claimed)))
    })?;
    let _ = prepare_event_store(&root);
    Ok(claimed)
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
        if let Some(expected) = filter.flow_type.as_ref() {
            if &event.flow_type != expected {
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
        if let Some(expected) = filter.run_id.as_ref() {
            if event.run_id.as_ref() != Some(expected) {
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

pub fn replay_task_events_from_cursor(
    project_root: impl AsRef<Path>,
    cursor: &TaskReplayCursor,
) -> Result<Vec<TaskEvent>> {
    replay_task_events(
        project_root,
        ReplayFilter {
            flow_type: Some(cursor.flow_type),
            aggregate_type: Some(cursor.aggregate_type.clone()),
            aggregate_id: Some(cursor.aggregate_id.clone()),
            issue_id: cursor.issue_id.clone(),
            project_id: cursor.project_id.clone(),
            run_id: cursor.run_id.clone(),
            event_types: Vec::new(),
            after_event_id: Some(cursor.after_event_id.clone()),
        },
    )
}

pub fn load_pending_task_events(
    project_root: impl AsRef<Path>,
    consumer_id: &str,
    event_types: &[&str],
) -> Result<Vec<TaskEvent>> {
    let root = canonical_project_root(project_root)?;
    prepare_event_store(&root)?;
    let consumer = load_consumer_state(&root, consumer_id)?;
    let consumed = consumer
        .consumed_event_ids
        .into_iter()
        .collect::<BTreeSet<_>>();
    let allowed_types = event_types.iter().copied().collect::<BTreeSet<_>>();
    Ok(load_task_events(&root)?
        .into_iter()
        .filter(|event| allowed_types.contains(event.event_type.as_str()))
        .filter(|event| !consumed.contains(&event.event_id))
        .collect())
}

pub fn mark_task_event_consumed(
    project_root: impl AsRef<Path>,
    consumer_id: &str,
    event_id: &str,
) -> Result<TaskEventConsumerState> {
    let root = canonical_project_root(project_root)?;
    prepare_event_store(&root)?;
    let mut consumer = load_consumer_state(&root, consumer_id)?;
    if !consumer
        .consumed_event_ids
        .iter()
        .any(|consumed_id| consumed_id == event_id)
    {
        consumer.consumed_event_ids.push(event_id.to_string());
    }
    consumer.updated_at = unix_timestamp_seconds();
    write_json(&consumer_path(&root, consumer_id), &consumer)?;
    Ok(consumer)
}

pub fn append_task_dead_letter(
    project_root: impl AsRef<Path>,
    consumer_id: &str,
    event: &TaskEvent,
    error: impl Into<String>,
) -> Result<TaskEventDeadLetter> {
    let root = canonical_project_root(project_root)?;
    prepare_event_store(&root)?;
    let dead_letter = TaskEventDeadLetter {
        version: TASK_EVENT_DEAD_LETTER_VERSION.to_string(),
        consumer_id: consumer_id.to_string(),
        event_id: event.event_id.clone(),
        event_type: event.event_type.clone(),
        subject_id: event.aggregate_id.clone(),
        error: error.into(),
        created_at: unix_timestamp_seconds(),
    };
    write_json(
        &root
            .join(".agentflow/events/dead-letter")
            .join(format!("{consumer_id}-{}.json", event.event_id)),
        &dead_letter,
    )?;
    let _ = prepare_event_store(&root);
    Ok(dead_letter)
}

fn load_consumer_state(root: &Path, consumer_id: &str) -> Result<TaskEventConsumerState> {
    let path = consumer_path(root, consumer_id);
    if !path.is_file() {
        return Ok(TaskEventConsumerState {
            version: TASK_EVENT_CONSUMER_VERSION.to_string(),
            consumer_id: consumer_id.to_string(),
            consumed_event_ids: Vec::new(),
            updated_at: unix_timestamp_seconds(),
        });
    }
    read_json(&path)
}

fn consumer_path(root: &Path, consumer_id: &str) -> PathBuf {
    root.join(".agentflow/events/consumers")
        .join(format!("{consumer_id}.json"))
}

fn append_task_event_locked(
    root: &Path,
    draft: TaskEventDraft,
    enforce_idempotency: bool,
) -> Result<TaskEvent> {
    validate_draft(&draft)?;
    with_event_store_lock(root, "append", move |root| {
        if enforce_idempotency {
            if let Some(existing) =
                find_existing_idempotent_event(root, draft.idempotency_key.as_ref())?
            {
                return Ok(existing);
            }
        }
        let event = materialize_event_unlocked(root, draft)?;
        append_event_line(root, &event)?;
        Ok(event)
    })
}

fn append_task_event_once_unlocked(root: &Path, draft: TaskEventDraft) -> Result<TaskEvent> {
    validate_draft(&draft)?;
    if let Some(existing) = find_existing_idempotent_event(root, draft.idempotency_key.as_ref())? {
        return Ok(existing);
    }
    let event = materialize_event_unlocked(root, draft)?;
    append_event_line(root, &event)?;
    Ok(event)
}

fn find_existing_idempotent_event(root: &Path, key: Option<&String>) -> Result<Option<TaskEvent>> {
    let Some(key) = key else {
        return Ok(None);
    };
    Ok(load_task_events(root)?
        .into_iter()
        .find(|event| event.idempotency_key.as_ref() == Some(key)))
}

fn materialize_event_unlocked(root: &Path, draft: TaskEventDraft) -> Result<TaskEvent> {
    let timestamp = unix_timestamp_seconds();
    let event_id = next_event_id_unlocked(root)?;
    Ok(TaskEvent {
        event_id,
        event_version: TASK_EVENT_VERSION.to_string(),
        flow_type: draft.flow_type,
        aggregate_type: draft.aggregate_type,
        aggregate_id: draft.aggregate_id.clone(),
        project_id: draft.project_id,
        issue_id: draft.issue_id,
        run_id: draft.run_id,
        event_type: draft.event_type,
        timestamp,
        authority_role: draft.authority_role,
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
    if classify_task_event(&draft.event_type).as_str() == "unknown" {
        anyhow::bail!(
            "task event type {} is not in the runtime taxonomy",
            draft.event_type
        );
    }
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

fn next_event_id_unlocked(root: &Path) -> Result<String> {
    Ok(format!(
        "evt-{:06}",
        next_sequence_value_unlocked(root, "event-id")?
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

fn with_event_store_lock<T, F>(root: &Path, scope: &str, action: F) -> Result<T>
where
    F: FnOnce(&Path) -> Result<T>,
{
    let lock_path = event_store_lock_path(root, scope);
    ensure_directory(lock_path.parent().expect("lock path should have parent"))?;
    for _ in 0..EVENT_STORE_LOCK_RETRY_LIMIT {
        match fs::create_dir(&lock_path) {
            Ok(()) => {
                let result = action(root);
                let cleanup = fs::remove_dir(&lock_path)
                    .with_context(|| format!("remove {}", lock_path.display()));
                return match (result, cleanup) {
                    (Ok(value), Ok(())) => Ok(value),
                    (Err(error), Ok(())) => Err(error),
                    (Ok(_), Err(error)) => Err(error),
                    (Err(error), Err(_)) => Err(error),
                };
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                thread::sleep(std::time::Duration::from_millis(
                    EVENT_STORE_LOCK_RETRY_DELAY_MS,
                ));
            }
            Err(error) => {
                return Err(error).with_context(|| format!("create {}", lock_path.display()));
            }
        }
    }
    anyhow::bail!(
        "timed out waiting for event store lock {}",
        lock_path.display()
    )
}

fn next_sequence_value_unlocked(root: &Path, namespace: &str) -> Result<u64> {
    let path = event_store_sequence_path(root, namespace);
    let current = if path.is_file() {
        fs::read_to_string(&path)
            .with_context(|| format!("read {}", path.display()))?
            .trim()
            .parse::<u64>()
            .with_context(|| format!("parse {}", path.display()))?
    } else {
        0
    };
    let next = current + 1;
    fs::write(&path, format!("{next}\n")).with_context(|| format!("write {}", path.display()))?;
    Ok(next)
}

fn event_store_lock_path(root: &Path, scope: &str) -> PathBuf {
    root.join(".agentflow/events/locks")
        .join(sanitize_namespace(scope))
}

fn event_store_sequence_path(root: &Path, namespace: &str) -> PathBuf {
    root.join(".agentflow/events/sequences")
        .join(format!("{}.seq", sanitize_namespace(namespace)))
}

fn sanitize_namespace(value: &str) -> String {
    value
        .chars()
        .map(|ch| match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => ch,
            _ => '_',
        })
        .collect()
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

fn count_json_files(path: &Path) -> Result<usize> {
    if !path.exists() {
        return Ok(0);
    }
    Ok(fs::read_dir(path)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().and_then(|value| value.to_str()) == Some("json"))
        .count())
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
    use crate::model::{
        EventActor, EventStateTransition, TaskReplayCursor, EVENT_TYPE_SPEC_ISSUE_READY,
    };
    use agentflow_workflow_core::{WorkflowAgentRole, WorkflowFlowType};
    use serde_json::json;
    use std::sync::Arc;
    use std::thread;
    use tempfile::tempdir;

    fn issue_scheduled_draft(issue_id: &str) -> TaskEventDraft {
        TaskEventDraft {
            flow_type: WorkflowFlowType::Work,
            aggregate_type: "issue".to_string(),
            aggregate_id: issue_id.to_string(),
            project_id: Some("project-task-workflow-v1".to_string()),
            issue_id: Some(issue_id.to_string()),
            run_id: None,
            event_type: "issue.scheduled".to_string(),
            authority_role: Some(WorkflowAgentRole::WorkAgent),
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
        assert_eq!(event.flow_type, WorkflowFlowType::Work);
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
    fn append_once_is_atomic_across_threads() {
        let dir = tempdir().unwrap();
        let root = Arc::new(dir.path().to_path_buf());
        let handles = (0..8)
            .map(|_| {
                let root = Arc::clone(&root);
                thread::spawn(move || {
                    append_task_event_once(root.as_path(), issue_scheduled_draft("AF-TASK-001"))
                        .unwrap()
                        .event_id
                })
            })
            .collect::<Vec<_>>();

        let event_ids = handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect::<Vec<_>>();

        assert!(event_ids.windows(2).all(|pair| pair[0] == pair[1]));
        assert_eq!(load_task_events(root.as_path()).unwrap().len(), 1);
    }

    #[test]
    fn sequence_allocation_is_monotonic_across_threads() {
        let dir = tempdir().unwrap();
        let root = Arc::new(dir.path().to_path_buf());
        let handles = (0..8)
            .map(|_| {
                let root = Arc::clone(&root);
                thread::spawn(move || {
                    allocate_task_sequence(root.as_path(), "run-id:AF-TASK-001").unwrap()
                })
            })
            .collect::<Vec<_>>();

        let mut values = handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect::<Vec<_>>();
        values.sort_unstable();

        assert_eq!(values, vec![1, 2, 3, 4, 5, 6, 7, 8]);
    }

    #[test]
    fn replay_filters_by_issue_and_after_event() {
        let dir = tempdir().unwrap();
        let first = append_task_event(dir.path(), issue_scheduled_draft("AF-TASK-001")).unwrap();
        append_task_event(dir.path(), issue_scheduled_draft("AF-TASK-002")).unwrap();
        let mut third = issue_scheduled_draft("AF-TASK-001");
        third.event_type = "run.created".to_string();
        third.causation_id = Some(first.event_id.clone());
        third.run_id = Some("run-001".to_string());
        append_task_event(dir.path(), third).unwrap();

        let mut filter = ReplayFilter::issue("AF-TASK-001");
        filter.after_event_id = Some(first.event_id);
        let events = replay_task_events(dir.path(), filter).unwrap();

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "run.created");
    }

    #[test]
    fn replay_filters_by_run_and_cursor() {
        let dir = tempdir().unwrap();
        let first = append_task_event(dir.path(), issue_scheduled_draft("AF-TASK-001")).unwrap();
        let mut second = issue_scheduled_draft("AF-TASK-001");
        second.event_type = "run.created".to_string();
        second.run_id = Some("run-001".to_string());
        second.authority_role = Some(WorkflowAgentRole::WorkAgent);
        let second = append_task_event(dir.path(), second).unwrap();

        let mut third = issue_scheduled_draft("AF-TASK-001");
        third.event_type = "checkpoint.created".to_string();
        third.run_id = Some("run-001".to_string());
        third.causation_id = Some(second.event_id.clone());
        append_task_event(dir.path(), third).unwrap();

        let by_run =
            replay_task_events(dir.path(), ReplayFilter::run("AF-TASK-001", "run-001")).unwrap();
        assert_eq!(by_run.len(), 2);

        let cursor = TaskReplayCursor {
            flow_type: WorkflowFlowType::Work,
            aggregate_type: "issue".to_string(),
            aggregate_id: "AF-TASK-001".to_string(),
            project_id: Some("project-task-workflow-v1".to_string()),
            issue_id: Some("AF-TASK-001".to_string()),
            run_id: Some("run-001".to_string()),
            after_event_id: second.event_id,
        };
        let replayed = replay_task_events_from_cursor(dir.path(), &cursor).unwrap();
        assert_eq!(replayed.len(), 1);
        assert_eq!(replayed[0].event_type, "checkpoint.created");
        assert_eq!(first.event_type, "issue.scheduled");
    }

    #[test]
    fn consumer_reads_pending_events_and_marks_consumed() {
        let dir = tempdir().unwrap();
        let mut draft = issue_scheduled_draft("AF-TASK-001");
        draft.event_type = EVENT_TYPE_SPEC_ISSUE_READY.to_string();
        let event = append_task_event_once(dir.path(), draft).unwrap();

        let pending =
            load_pending_task_events(dir.path(), "panel", &[EVENT_TYPE_SPEC_ISSUE_READY]).unwrap();
        assert_eq!(pending.len(), 1);

        mark_task_event_consumed(dir.path(), "panel", &event.event_id).unwrap();
        assert!(
            load_pending_task_events(dir.path(), "panel", &[EVENT_TYPE_SPEC_ISSUE_READY])
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn dead_letter_is_recorded() {
        let dir = tempdir().unwrap();
        let event =
            append_task_event_once(dir.path(), issue_scheduled_draft("AF-TASK-001")).unwrap();

        append_task_dead_letter(dir.path(), "panel", &event, "failed").unwrap();

        assert_eq!(
            prepare_event_store(dir.path())
                .unwrap()
                .summary
                .dead_letters,
            1
        );
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

    #[test]
    fn rejects_unknown_event_type_taxonomy() {
        let dir = tempdir().unwrap();
        let mut draft = issue_scheduled_draft("AF-TASK-001");
        draft.event_type = "mystery.event".to_string();
        let err = append_task_event(dir.path(), draft)
            .unwrap_err()
            .to_string();
        assert!(err.contains("runtime taxonomy"));
    }
}
