use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const TASK_EVENT_VERSION: &str = "task-event.v1";
pub const TASK_EVENT_MANIFEST_VERSION: &str = "task-event-store-manifest.v1";
pub const TASK_EVENT_STREAM_PATH: &str = ".agentflow/events/task-events.jsonl";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventActor {
    pub role: String,
    pub kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventStateTransition {
    #[serde(rename = "from")]
    pub from_state: String,
    #[serde(rename = "to")]
    pub to_state: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskEventDraft {
    pub aggregate_type: String,
    pub aggregate_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issue_id: Option<String>,
    #[serde(rename = "type")]
    pub event_type: String,
    pub actor: EventActor,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<EventStateTransition>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub causation_id: Option<String>,
    #[serde(default)]
    pub payload: Value,
    #[serde(default)]
    pub artifact_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskEvent {
    pub event_id: String,
    pub event_version: String,
    pub aggregate_type: String,
    pub aggregate_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issue_id: Option<String>,
    #[serde(rename = "type")]
    pub event_type: String,
    pub timestamp: u64,
    pub actor: EventActor,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<EventStateTransition>,
    pub correlation_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub causation_id: Option<String>,
    #[serde(default)]
    pub payload: Value,
    #[serde(default)]
    pub artifact_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskEventManifest {
    pub version: String,
    pub project_root: String,
    pub stream_path: String,
    pub summary: TaskEventSummary,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskEventSummary {
    pub events: usize,
    pub imported_legacy_events: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ReplayFilter {
    pub aggregate_type: Option<String>,
    pub aggregate_id: Option<String>,
    pub issue_id: Option<String>,
    pub project_id: Option<String>,
    pub event_types: Vec<String>,
    pub after_event_id: Option<String>,
}

impl ReplayFilter {
    pub fn aggregate(aggregate_type: impl Into<String>, aggregate_id: impl Into<String>) -> Self {
        Self {
            aggregate_type: Some(aggregate_type.into()),
            aggregate_id: Some(aggregate_id.into()),
            ..Self::default()
        }
    }

    pub fn issue(issue_id: impl Into<String>) -> Self {
        Self {
            issue_id: Some(issue_id.into()),
            ..Self::default()
        }
    }
}
