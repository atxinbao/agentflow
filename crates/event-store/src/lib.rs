//! AgentFlow task event store.
//!
//! This crate owns the append-only JSONL event fact layer. It does not rebuild
//! projections, update spec issues, execute workflow actions, or serve Desktop
//! UI data directly.

pub mod model;
pub mod storage;

pub use model::{
    EventActor, EventStateTransition, ReplayFilter, TaskEvent, TaskEventDraft, TaskEventManifest,
    TaskEventSummary, TASK_EVENT_MANIFEST_VERSION, TASK_EVENT_STREAM_PATH, TASK_EVENT_VERSION,
};
pub use storage::{
    append_task_event, append_task_event_once, import_legacy_workflow_events, load_task_events,
    prepare_event_store, replay_task_events,
};
