//! AgentFlow task event store.
//!
//! This crate owns the append-only JSONL event fact layer. It does not rebuild
//! projections, update spec issues, execute workflow actions, or serve Desktop
//! UI data directly.

pub mod model;
pub mod runtime;
pub mod storage;

pub use model::{
    classify_task_event, ContextPackFailedPayload, ContextPackReadyPayload,
    ContextPackRequestedPayload, EventActor, EventStateTransition, IssueReadyPayload, ReplayFilter,
    TaskEvent, TaskEventCategory, TaskEventClaimLease, TaskEventClaimStatus,
    TaskEventConsumerState, TaskEventDeadLetter, TaskEventDraft, TaskEventManifest,
    TaskEventSummary, TaskReplayCursor, CONSUMER_PANEL, EVENT_TYPE_PANEL_CONTEXT_PACK_FAILED,
    EVENT_TYPE_PANEL_CONTEXT_PACK_READY, EVENT_TYPE_PANEL_CONTEXT_PACK_REQUESTED,
    EVENT_TYPE_SPEC_ISSUE_READY, TASK_EVENT_CLAIM_LEASE_VERSION, TASK_EVENT_CONSUMER_VERSION,
    TASK_EVENT_DEAD_LETTER_VERSION, TASK_EVENT_MANIFEST_VERSION, TASK_EVENT_STREAM_PATH,
    TASK_EVENT_VERSION,
};
pub use runtime::{
    append_accepted_action_event, build_runtime_event_envelope, map_task_event_to_runtime_event,
    replay_runtime_events, AcceptedActionAppendContext, CompatibilityRuntimeEvent,
    RuntimeEventEnvelope, RUNTIME_EVENT_ENVELOPE_VERSION,
};
pub use storage::{
    allocate_task_sequence, append_task_dead_letter, append_task_event, append_task_event_once,
    claim_task_event, load_pending_task_events, load_task_claim_lease, load_task_claim_leases,
    load_task_events, mark_task_event_consumed, prepare_event_store, release_task_claim,
    renew_task_claim, replay_task_events, replay_task_events_from_cursor, task_claim_is_active,
};
