//! AgentFlow local message bus contract.
//!
//! The bus is an in-memory fanout / refresh signal surface. It does not store
//! authority facts. Durable replay always comes from `agentflow-event-store`.

use std::collections::VecDeque;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use agentflow_event_store::{load_task_events, ReplayFilter, TaskEvent};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub const MESSAGE_BUS_ENVELOPE_VERSION: &str = "agentflow-message-bus-envelope.v1";
pub const MESSAGE_BUS_POLICY_VERSION: &str = "agentflow-message-bus-policy.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MessageBusChannel {
    Runtime,
    Projection,
    Command,
    Worker,
    Audit,
}

impl MessageBusChannel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Runtime => "runtime",
            Self::Projection => "projection",
            Self::Command => "command",
            Self::Worker => "worker",
            Self::Audit => "audit",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MessageReplaySource {
    EventStore,
    LiveFanout,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageBusEnvelope {
    pub version: String,
    pub message_id: String,
    pub correlation_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub causation_id: Option<String>,
    pub idempotency_key: String,
    pub channel: MessageBusChannel,
    pub topic: String,
    pub subject_type: String,
    pub subject_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_ref: Option<String>,
    pub replay_source: MessageReplaySource,
    pub created_at: u64,
    #[serde(default)]
    pub payload: Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageBusAuthorityPolicy {
    pub version: String,
    pub stores_authority: bool,
    pub durable_replay_source: String,
    pub allowed_channels: Vec<MessageBusChannel>,
}

pub fn local_message_bus_policy() -> MessageBusAuthorityPolicy {
    MessageBusAuthorityPolicy {
        version: MESSAGE_BUS_POLICY_VERSION.to_string(),
        stores_authority: false,
        durable_replay_source: "event-store".to_string(),
        allowed_channels: vec![
            MessageBusChannel::Runtime,
            MessageBusChannel::Projection,
            MessageBusChannel::Command,
            MessageBusChannel::Worker,
            MessageBusChannel::Audit,
        ],
    }
}

#[derive(Debug, Default, Clone)]
pub struct LocalMessageBus {
    messages: VecDeque<MessageBusEnvelope>,
}

impl LocalMessageBus {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn publish(&mut self, envelope: MessageBusEnvelope) {
        self.messages.push_back(envelope);
    }

    pub fn publish_projection_refresh(
        &mut self,
        subject_type: impl Into<String>,
        subject_id: impl Into<String>,
        reason: impl Into<String>,
    ) -> MessageBusEnvelope {
        let subject_type = subject_type.into();
        let subject_id = subject_id.into();
        let envelope = live_envelope(
            MessageBusChannel::Projection,
            "projection.refresh.requested",
            subject_type,
            subject_id,
            json!({ "reason": reason.into() }),
        );
        self.publish(envelope.clone());
        envelope
    }

    pub fn publish_console_refresh(
        &mut self,
        surface: impl Into<String>,
        subject_id: impl Into<String>,
        reason: impl Into<String>,
    ) -> MessageBusEnvelope {
        let surface = surface.into();
        let envelope = live_envelope(
            MessageBusChannel::Command,
            "console.refresh.requested",
            "console-surface",
            subject_id.into(),
            json!({ "surface": surface, "reason": reason.into() }),
        );
        self.publish(envelope.clone());
        envelope
    }

    pub fn drain_channel(&mut self, channel: MessageBusChannel) -> Vec<MessageBusEnvelope> {
        let mut drained = Vec::new();
        let mut retained = VecDeque::new();
        while let Some(message) = self.messages.pop_front() {
            if message.channel == channel {
                drained.push(message);
            } else {
                retained.push_back(message);
            }
        }
        self.messages = retained;
        drained
    }

    pub fn messages(&self) -> Vec<MessageBusEnvelope> {
        self.messages.iter().cloned().collect()
    }
}

pub fn replay_event_store_to_bus(
    project_root: impl AsRef<Path>,
    filter: ReplayFilter,
    channel: MessageBusChannel,
) -> Result<Vec<MessageBusEnvelope>> {
    let events = load_task_events(project_root)?;
    Ok(events
        .iter()
        .filter(|event| event_matches_filter(event, &filter))
        .map(|event| envelope_from_event(event, channel.clone()))
        .collect())
}

pub fn envelope_from_event(event: &TaskEvent, channel: MessageBusChannel) -> MessageBusEnvelope {
    MessageBusEnvelope {
        version: MESSAGE_BUS_ENVELOPE_VERSION.to_string(),
        message_id: format!("bus-{}", event.event_id),
        correlation_id: event.correlation_id.clone(),
        causation_id: event.causation_id.clone(),
        idempotency_key: event
            .idempotency_key
            .clone()
            .unwrap_or_else(|| format!("bus:event-store:{}", event.event_id)),
        channel,
        topic: event.event_type.clone(),
        subject_type: event.aggregate_type.clone(),
        subject_id: event.aggregate_id.clone(),
        event_ref: Some(event.event_id.clone()),
        replay_source: MessageReplaySource::EventStore,
        created_at: event.timestamp,
        payload: json!({
            "eventType": event.event_type,
            "issueId": event.issue_id,
            "runId": event.run_id,
            "correlationId": event.correlation_id,
        }),
    }
}

fn live_envelope(
    channel: MessageBusChannel,
    topic: impl Into<String>,
    subject_type: impl Into<String>,
    subject_id: impl Into<String>,
    payload: Value,
) -> MessageBusEnvelope {
    let topic = topic.into();
    let subject_id = subject_id.into();
    let now = unix_timestamp_seconds();
    let channel_name = channel.as_str();
    let idempotency_key = format!("bus:live:{channel_name}:{}:{subject_id}:{now}", topic);
    MessageBusEnvelope {
        version: MESSAGE_BUS_ENVELOPE_VERSION.to_string(),
        message_id: format!("bus-live-{channel_name}-{topic}-{subject_id}-{now}"),
        correlation_id: format!("corr-bus-{subject_id}-{now}"),
        causation_id: None,
        idempotency_key,
        channel,
        topic,
        subject_type: subject_type.into(),
        subject_id,
        event_ref: None,
        replay_source: MessageReplaySource::LiveFanout,
        created_at: now,
        payload,
    }
}

fn unix_timestamp_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn event_matches_filter(event: &TaskEvent, filter: &ReplayFilter) -> bool {
    if let Some(flow_type) = filter.flow_type {
        if event.flow_type != flow_type {
            return false;
        }
    }
    if let Some(aggregate_type) = filter.aggregate_type.as_deref() {
        if event.aggregate_type != aggregate_type {
            return false;
        }
    }
    if let Some(aggregate_id) = filter.aggregate_id.as_deref() {
        if event.aggregate_id != aggregate_id {
            return false;
        }
    }
    if let Some(issue_id) = filter.issue_id.as_deref() {
        if event.issue_id.as_deref() != Some(issue_id) {
            return false;
        }
    }
    if let Some(project_id) = filter.project_id.as_deref() {
        if event.project_id.as_deref() != Some(project_id) {
            return false;
        }
    }
    if let Some(run_id) = filter.run_id.as_deref() {
        if event.run_id.as_deref() != Some(run_id) {
            return false;
        }
    }
    if !filter.event_types.is_empty()
        && !filter
            .event_types
            .iter()
            .any(|event_type| event_type == &event.event_type)
    {
        return false;
    }
    if let Some(after_event_id) = filter.after_event_id.as_deref() {
        if event.event_id.as_str() <= after_event_id {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use agentflow_event_store::{
        append_task_event, EventActor, EventStateTransition, TaskEventDraft,
    };
    use agentflow_workflow_core::{WorkflowAgentRole, WorkflowFlowType};
    use serde_json::json;
    use tempfile::tempdir;

    fn issue_scheduled_draft(issue_id: &str) -> TaskEventDraft {
        TaskEventDraft {
            flow_type: WorkflowFlowType::Work,
            aggregate_type: "issue".to_string(),
            aggregate_id: issue_id.to_string(),
            project_id: Some("project-message-bus".to_string()),
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
            artifact_refs: Vec::new(),
            idempotency_key: Some(format!("issue.scheduled:{issue_id}")),
        }
    }

    #[test]
    fn policy_declares_bus_is_not_authority() {
        let policy = local_message_bus_policy();
        assert!(!policy.stores_authority);
        assert_eq!(policy.durable_replay_source, "event-store");
        assert!(policy
            .allowed_channels
            .contains(&MessageBusChannel::Projection));
    }

    #[test]
    fn live_bus_fanout_does_not_write_agentflow_files() {
        let dir = tempdir().unwrap();
        let mut bus = LocalMessageBus::new();
        let envelope = bus.publish_projection_refresh("issue", "AF-BUS-001", "task event appended");

        assert_eq!(envelope.channel, MessageBusChannel::Projection);
        assert_eq!(envelope.replay_source, MessageReplaySource::LiveFanout);
        assert_ne!(envelope.created_at, 0);
        assert!(envelope.message_id.starts_with("bus-live-projection-"));
        assert!(envelope.correlation_id.starts_with("corr-bus-"));
        assert!(envelope.idempotency_key.starts_with("bus:live:projection:"));
        assert_eq!(bus.messages().len(), 1);
        assert!(!dir.path().join(".agentflow").exists());
    }

    #[test]
    fn console_refresh_uses_command_channel() {
        let mut bus = LocalMessageBus::new();
        bus.publish_console_refresh("task-workbench", "AF-BUS-002", "projection refreshed");
        let messages = bus.drain_channel(MessageBusChannel::Command);

        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].topic, "console.refresh.requested");
        assert_eq!(messages[0].payload["surface"], "task-workbench");
        assert!(bus.messages().is_empty());
    }

    #[test]
    fn replay_uses_event_store_as_durable_source() {
        let dir = tempdir().unwrap();
        let event = append_task_event(dir.path(), issue_scheduled_draft("AF-BUS-003")).unwrap();
        let messages = replay_event_store_to_bus(
            dir.path(),
            ReplayFilter::issue("AF-BUS-003"),
            MessageBusChannel::Worker,
        )
        .unwrap();

        assert_eq!(messages.len(), 1);
        assert_eq!(
            messages[0].event_ref.as_deref(),
            Some(event.event_id.as_str())
        );
        assert_eq!(messages[0].replay_source, MessageReplaySource::EventStore);
        assert_eq!(messages[0].topic, "issue.scheduled");
        assert_eq!(messages[0].correlation_id, event.correlation_id);
        assert_eq!(messages[0].causation_id, event.causation_id);
        assert_eq!(messages[0].idempotency_key, "issue.scheduled:AF-BUS-003");
    }
}
