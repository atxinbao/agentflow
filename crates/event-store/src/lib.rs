//! AgentFlow task event store.
//!
//! This crate owns the append-only JSONL event fact layer. It does not rebuild
//! projections, update spec issues, execute workflow actions, or serve Desktop
//! UI data directly.

pub mod model;
pub mod storage;

pub use model::{
    ContextPackFailedPayload, ContextPackReadyPayload, ContextPackRequestedPayload, EventActor,
    EventStateTransition, IssueReadyPayload, ReplayFilter, TaskEvent, TaskEventConsumerState,
    TaskEventDeadLetter, TaskEventDraft, TaskEventManifest, TaskEventSummary, CONSUMER_PANEL,
    EVENT_TYPE_PANEL_CONTEXT_PACK_FAILED, EVENT_TYPE_PANEL_CONTEXT_PACK_READY,
    EVENT_TYPE_PANEL_CONTEXT_PACK_REQUESTED, EVENT_TYPE_SPEC_ISSUE_READY,
    TASK_EVENT_CONSUMER_VERSION, TASK_EVENT_DEAD_LETTER_VERSION, TASK_EVENT_MANIFEST_VERSION,
    TASK_EVENT_STREAM_PATH, TASK_EVENT_VERSION,
};
pub use storage::{
    append_task_dead_letter, append_task_event, append_task_event_once, load_pending_task_events,
    load_task_events, mark_task_event_consumed, prepare_event_store, replay_task_events,
};
