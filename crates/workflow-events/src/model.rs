use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const WORKFLOW_EVENT_VERSION: &str = "workflow-event.v1";
pub const WORKFLOW_EVENT_MANIFEST_VERSION: &str = "workflow-events-manifest.v1";
pub const WORKFLOW_CONSUMER_VERSION: &str = "workflow-event-consumer.v1";
pub const WORKFLOW_DEAD_LETTER_VERSION: &str = "workflow-event-dead-letter.v1";

pub const EVENT_TYPE_INPUT_ISSUE_READY: &str = "input.issue.ready";
pub const EVENT_TYPE_PANEL_CONTEXT_PACK_REQUESTED: &str = "panel.context-pack.requested";
pub const EVENT_TYPE_PANEL_CONTEXT_PACK_READY: &str = "panel.context-pack.ready";
pub const EVENT_TYPE_PANEL_CONTEXT_PACK_FAILED: &str = "panel.context-pack.failed";

pub const CONSUMER_PANEL: &str = "panel";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowEventDraft {
    pub event_type: String,
    pub source: String,
    pub subject_id: String,
    pub subject_path: Option<String>,
    pub dedupe_key: String,
    pub payload: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowEvent {
    pub version: String,
    pub event_id: String,
    pub event_type: String,
    pub source: String,
    pub subject_id: String,
    pub subject_path: Option<String>,
    pub dedupe_key: String,
    pub created_at: u64,
    pub payload: Value,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowEventManifest {
    pub version: String,
    pub project_root: String,
    pub stream_path: String,
    pub consumers_path: String,
    pub dead_letter_path: String,
    pub summary: WorkflowEventSummary,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowEventSummary {
    pub events: usize,
    pub consumers: usize,
    pub dead_letters: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowConsumerState {
    pub version: String,
    pub consumer_id: String,
    pub consumed_event_ids: Vec<String>,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowDeadLetter {
    pub version: String,
    pub consumer_id: String,
    pub event_id: String,
    pub event_type: String,
    pub subject_id: String,
    pub error: String,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueReadyPayload {
    pub issue_id: String,
    pub issue_path: String,
    pub issue_category: String,
    pub required_agent_role: String,
    pub display_status: String,
    pub title: String,
    pub objective: String,
    pub acceptance_criteria: Vec<String>,
    pub context_pack_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextPackRequestedPayload {
    pub issue_id: String,
    pub context_pack_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextPackReadyPayload {
    pub issue_id: String,
    pub context_pack_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextPackFailedPayload {
    pub issue_id: String,
    pub context_pack_path: Option<String>,
    pub error: String,
}

pub trait WorkflowEventPayload: Serialize {
    fn into_value(self) -> anyhow::Result<Value>
    where
        Self: Sized,
    {
        Ok(serde_json::to_value(self)?)
    }
}

impl<T> WorkflowEventPayload for T where T: Serialize {}
