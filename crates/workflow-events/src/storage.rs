use crate::model::{
    WorkflowConsumerState, WorkflowDeadLetter, WorkflowEvent, WorkflowEventDraft,
    WorkflowEventManifest, WorkflowEventSummary, WORKFLOW_CONSUMER_VERSION,
    WORKFLOW_DEAD_LETTER_VERSION, WORKFLOW_EVENT_MANIFEST_VERSION, WORKFLOW_EVENT_VERSION,
};
use anyhow::{Context, Result};
use std::{
    collections::BTreeSet,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

pub fn prepare_events_workspace(project_root: impl AsRef<Path>) -> Result<WorkflowEventManifest> {
    let root = canonical_project_root(project_root)?;
    ensure_directory(&root.join(".agentflow/events/consumers"))?;
    ensure_directory(&root.join(".agentflow/events/dead-letter"))?;
    let stream_path = root.join(".agentflow/events/stream.jsonl");
    if !stream_path.exists() {
        fs::write(&stream_path, "")?;
    }
    let manifest = WorkflowEventManifest {
        version: WORKFLOW_EVENT_MANIFEST_VERSION.to_string(),
        project_root: root.display().to_string(),
        stream_path: ".agentflow/events/stream.jsonl".to_string(),
        consumers_path: ".agentflow/events/consumers".to_string(),
        dead_letter_path: ".agentflow/events/dead-letter".to_string(),
        summary: WorkflowEventSummary {
            events: load_events(&root)?.len(),
            consumers: count_json_files(&root.join(".agentflow/events/consumers"))?,
            dead_letters: count_json_files(&root.join(".agentflow/events/dead-letter"))?,
        },
    };
    write_json(&root.join(".agentflow/events/manifest.json"), &manifest)?;
    Ok(manifest)
}

pub fn append_event_once(
    project_root: impl AsRef<Path>,
    draft: WorkflowEventDraft,
) -> Result<WorkflowEvent> {
    let root = canonical_project_root(project_root)?;
    prepare_events_workspace(&root)?;
    if let Some(existing) = load_events(&root)?
        .into_iter()
        .find(|event| event.dedupe_key == draft.dedupe_key)
    {
        return Ok(existing);
    }

    let event = WorkflowEvent {
        version: WORKFLOW_EVENT_VERSION.to_string(),
        event_id: next_event_id(&root)?,
        event_type: draft.event_type,
        source: draft.source,
        subject_id: draft.subject_id,
        subject_path: draft.subject_path,
        dedupe_key: draft.dedupe_key,
        created_at: unix_timestamp_seconds(),
        payload: draft.payload,
    };
    let line = serde_json::to_string(&event)?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(root.join(".agentflow/events/stream.jsonl"))?;
    writeln!(file, "{line}")?;
    let _ = prepare_events_workspace(&root);
    Ok(event)
}

pub fn load_events(project_root: impl AsRef<Path>) -> Result<Vec<WorkflowEvent>> {
    let root = canonical_project_root(project_root)?;
    let stream_path = root.join(".agentflow/events/stream.jsonl");
    if !stream_path.is_file() {
        return Ok(Vec::new());
    }
    let raw = fs::read_to_string(&stream_path)
        .with_context(|| format!("read {}", stream_path.display()))?;
    raw.lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).context("parse workflow event"))
        .collect()
}

pub fn load_pending_events(
    project_root: impl AsRef<Path>,
    consumer_id: &str,
    event_types: &[&str],
) -> Result<Vec<WorkflowEvent>> {
    let root = canonical_project_root(project_root)?;
    prepare_events_workspace(&root)?;
    let consumer = load_consumer_state(&root, consumer_id)?;
    let consumed = consumer
        .consumed_event_ids
        .into_iter()
        .collect::<BTreeSet<_>>();
    let allowed_types = event_types.iter().copied().collect::<BTreeSet<_>>();
    Ok(load_events(&root)?
        .into_iter()
        .filter(|event| allowed_types.contains(event.event_type.as_str()))
        .filter(|event| !consumed.contains(&event.event_id))
        .collect())
}

pub fn mark_event_consumed(
    project_root: impl AsRef<Path>,
    consumer_id: &str,
    event_id: &str,
) -> Result<WorkflowConsumerState> {
    let root = canonical_project_root(project_root)?;
    prepare_events_workspace(&root)?;
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

pub fn append_dead_letter(
    project_root: impl AsRef<Path>,
    consumer_id: &str,
    event: &WorkflowEvent,
    error: impl Into<String>,
) -> Result<WorkflowDeadLetter> {
    let root = canonical_project_root(project_root)?;
    prepare_events_workspace(&root)?;
    let dead_letter = WorkflowDeadLetter {
        version: WORKFLOW_DEAD_LETTER_VERSION.to_string(),
        consumer_id: consumer_id.to_string(),
        event_id: event.event_id.clone(),
        event_type: event.event_type.clone(),
        subject_id: event.subject_id.clone(),
        error: error.into(),
        created_at: unix_timestamp_seconds(),
    };
    write_json(
        &root
            .join(".agentflow/events/dead-letter")
            .join(format!("{consumer_id}-{}.json", event.event_id)),
        &dead_letter,
    )?;
    let _ = prepare_events_workspace(&root);
    Ok(dead_letter)
}

fn load_consumer_state(root: &Path, consumer_id: &str) -> Result<WorkflowConsumerState> {
    let path = consumer_path(root, consumer_id);
    if !path.is_file() {
        return Ok(WorkflowConsumerState {
            version: WORKFLOW_CONSUMER_VERSION.to_string(),
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

fn next_event_id(root: &Path) -> Result<String> {
    Ok(format!(
        "evt-{}-{:04}",
        unix_timestamp_seconds(),
        load_events(root)?.len() + 1
    ))
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

fn ensure_directory(path: &Path) -> Result<()> {
    fs::create_dir_all(path).with_context(|| format!("create {}", path.display()))
}

fn canonical_project_root(project_root: impl AsRef<Path>) -> Result<PathBuf> {
    project_root
        .as_ref()
        .canonicalize()
        .with_context(|| format!("canonicalize {}", project_root.as_ref().display()))
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

fn write_json<T: serde::Serialize>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        ensure_directory(parent)?;
    }
    fs::write(path, serde_json::to_string_pretty(value)?)
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
    use crate::model::{IssueReadyPayload, EVENT_TYPE_INPUT_ISSUE_READY};
    use serde_json::json;
    use tempfile::tempdir;

    #[test]
    fn append_once_dedupes_and_consumer_marks_events() {
        let dir = tempdir().unwrap();
        let payload = IssueReadyPayload {
            issue_id: "iss-001".to_string(),
            issue_path: ".agentflow/input/issues/iss-001.json".to_string(),
            issue_category: "spec".to_string(),
            required_agent_role: "build-agent".to_string(),
            display_status: "todo".to_string(),
            title: "Build issue".to_string(),
            objective: "Build something".to_string(),
            acceptance_criteria: vec!["passes".to_string()],
            context_pack_path: Some(".agentflow/panel/context-packs/iss-001.json".to_string()),
        };
        let draft = WorkflowEventDraft {
            event_type: EVENT_TYPE_INPUT_ISSUE_READY.to_string(),
            source: "input".to_string(),
            subject_id: "iss-001".to_string(),
            subject_path: Some(".agentflow/input/issues/iss-001.json".to_string()),
            dedupe_key: "input.issue.ready:iss-001:1".to_string(),
            payload: serde_json::to_value(payload).unwrap(),
        };

        let first = append_event_once(dir.path(), draft.clone()).unwrap();
        let second = append_event_once(dir.path(), draft).unwrap();
        assert_eq!(first.event_id, second.event_id);
        assert_eq!(load_events(dir.path()).unwrap().len(), 1);

        let pending =
            load_pending_events(dir.path(), "panel", &[EVENT_TYPE_INPUT_ISSUE_READY]).unwrap();
        assert_eq!(pending.len(), 1);
        mark_event_consumed(dir.path(), "panel", &first.event_id).unwrap();
        assert!(
            load_pending_events(dir.path(), "panel", &[EVENT_TYPE_INPUT_ISSUE_READY])
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn dead_letter_is_recorded() {
        let dir = tempdir().unwrap();
        let event = append_event_once(
            dir.path(),
            WorkflowEventDraft {
                event_type: EVENT_TYPE_INPUT_ISSUE_READY.to_string(),
                source: "input".to_string(),
                subject_id: "iss-001".to_string(),
                subject_path: None,
                dedupe_key: "input.issue.ready:iss-001:dead-letter".to_string(),
                payload: json!({ "issueId": "iss-001" }),
            },
        )
        .unwrap();

        append_dead_letter(dir.path(), "panel", &event, "failed").unwrap();
        assert_eq!(
            prepare_events_workspace(dir.path())
                .unwrap()
                .summary
                .dead_letters,
            1
        );
    }
}
