use crate::{
    model::{StateTimelineEvent, StateTimelineEventDraft, STATE_TIMELINE_EVENT_VERSION},
    storage::{read_jsonl, unix_timestamp_seconds, write_jsonl},
};
use anyhow::Result;
use std::path::Path;

pub fn append_state_event(
    project_root: impl AsRef<Path>,
    draft: StateTimelineEventDraft,
) -> Result<StateTimelineEvent> {
    let root = crate::storage::canonical_project_root(project_root)?;
    let event = StateTimelineEvent {
        version: STATE_TIMELINE_EVENT_VERSION.to_string(),
        ts: unix_timestamp_seconds(),
        event: draft.event,
        project_root: root.display().to_string(),
        details: draft.details,
    };
    write_jsonl(&root.join(".agentflow/state/events/timeline.jsonl"), &event)?;
    Ok(event)
}

pub fn load_state_timeline(project_root: impl AsRef<Path>) -> Result<Vec<StateTimelineEvent>> {
    let root = crate::storage::canonical_project_root(project_root)?;
    read_jsonl(&root.join(".agentflow/state/events/timeline.jsonl"))
}
