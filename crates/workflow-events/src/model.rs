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
pub const EVENT_TYPE_BUILD_AGENT_LAUNCH_REQUESTED: &str = "build-agent.launch.requested";
pub const EVENT_TYPE_BUILD_AGENT_LAUNCH_CLAIMED: &str = "build-agent.launch.claimed";
pub const EVENT_TYPE_BUILD_AGENT_SESSION_RUNNING: &str = "build-agent.session.running";
pub const EVENT_TYPE_BUILD_AGENT_SESSION_REVIEW_READY: &str = "build-agent.session.review-ready";
pub const EVENT_TYPE_BUILD_AGENT_MERGE_CONFIRMED: &str = "build-agent.merge.confirmed";
pub const EVENT_TYPE_BUILD_AGENT_WRITEBACK_COMPLETED: &str = "build-agent.writeback.completed";

pub const CONSUMER_PANEL: &str = "panel";
pub const CONSUMER_BUILD_AGENT: &str = "build-agent";
pub const CONSUMER_PROVIDER_BRIDGE: &str = "provider-bridge";

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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildAgentLaunchRequestedPayload {
    pub issue_id: String,
    pub project_id: String,
    pub run_id: String,
    pub branch_name: Option<String>,
    pub issue_path: String,
    pub context_pack_path: String,
    pub launch_request_path: String,
    pub display_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildAgentLaunchClaimedPayload {
    pub issue_id: String,
    pub project_id: Option<String>,
    pub run_id: String,
    pub session_id: String,
    pub provider: String,
    pub branch_name: Option<String>,
    pub launch_request_path: String,
    pub log_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildAgentSessionRunningPayload {
    pub issue_id: String,
    pub project_id: Option<String>,
    pub run_id: String,
    pub session_id: String,
    pub provider: String,
    pub branch_name: Option<String>,
    pub log_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildAgentSessionReviewReadyPayload {
    pub issue_id: String,
    pub project_id: Option<String>,
    pub run_id: String,
    pub provider: String,
    pub delivery_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildAgentMergeConfirmedPayload {
    pub issue_id: String,
    pub project_id: Option<String>,
    pub run_id: String,
    pub provider: String,
    pub merge_mode: String,
    pub remote_url: Option<String>,
    pub merged: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildAgentWritebackCompletedPayload {
    pub issue_id: String,
    pub project_id: Option<String>,
    pub run_id: String,
    pub provider: String,
    pub delivery_path: Option<String>,
    pub next_issue_id: Option<String>,
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
