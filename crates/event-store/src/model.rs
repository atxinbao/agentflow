use agentflow_workflow_core::{WorkflowAgentRole, WorkflowFlowType};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const TASK_EVENT_VERSION: &str = "task-event.v2";
pub const TASK_EVENT_MANIFEST_VERSION: &str = "task-event-store-manifest.v1";
pub const TASK_EVENT_STREAM_PATH: &str = ".agentflow/events/task-events.jsonl";
pub const TASK_EVENT_CONSUMER_VERSION: &str = "task-event-consumer.v1";
pub const TASK_EVENT_DEAD_LETTER_VERSION: &str = "task-event-dead-letter.v1";
pub const TASK_EVENT_CLAIM_LEASE_VERSION: &str = "task-event-claim-lease.v1";

pub const EVENT_TYPE_SPEC_ISSUE_READY: &str = "spec.issue.ready";
pub const EVENT_TYPE_PANEL_CONTEXT_PACK_REQUESTED: &str = "panel.context-pack.requested";
pub const EVENT_TYPE_PANEL_CONTEXT_PACK_READY: &str = "panel.context-pack.ready";
pub const EVENT_TYPE_PANEL_CONTEXT_PACK_FAILED: &str = "panel.context-pack.failed";

pub const CONSUMER_PANEL: &str = "panel";

fn default_work_flow_type() -> WorkflowFlowType {
    WorkflowFlowType::Work
}

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
    pub flow_type: WorkflowFlowType,
    pub aggregate_type: String,
    pub aggregate_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issue_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    #[serde(rename = "type")]
    pub event_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authority_role: Option<WorkflowAgentRole>,
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
    #[serde(default = "default_work_flow_type")]
    pub flow_type: WorkflowFlowType,
    pub aggregate_type: String,
    pub aggregate_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issue_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    #[serde(rename = "type")]
    pub event_type: String,
    pub timestamp: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authority_role: Option<WorkflowAgentRole>,
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
    pub consumers: usize,
    pub dead_letters: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskEventConsumerState {
    pub version: String,
    pub consumer_id: String,
    pub consumed_event_ids: Vec<String>,
    pub updated_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskEventDeadLetter {
    pub version: String,
    pub consumer_id: String,
    pub event_id: String,
    pub event_type: String,
    pub subject_id: String,
    pub error: String,
    pub created_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TaskEventClaimStatus {
    Active,
    Released,
    Expired,
}

impl TaskEventClaimStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Released => "released",
            Self::Expired => "expired",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskEventClaimLease {
    pub version: String,
    pub owner_id: String,
    pub issue_id: String,
    pub run_id: String,
    pub requested_event_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub claim_event_id: Option<String>,
    pub lease_id: String,
    pub fencing_token: u64,
    pub status: TaskEventClaimStatus,
    pub claimed_at: u64,
    pub heartbeat_at: u64,
    pub expires_at: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub released_at: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub release_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ReplayFilter {
    pub flow_type: Option<WorkflowFlowType>,
    pub aggregate_type: Option<String>,
    pub aggregate_id: Option<String>,
    pub issue_id: Option<String>,
    pub project_id: Option<String>,
    pub run_id: Option<String>,
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

    pub fn run(issue_id: impl Into<String>, run_id: impl Into<String>) -> Self {
        Self {
            flow_type: Some(WorkflowFlowType::Work),
            issue_id: Some(issue_id.into()),
            run_id: Some(run_id.into()),
            ..Self::default()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TaskEventCategory {
    Contract,
    Workflow,
    Runtime,
    Session,
    ReviewDelivery,
    Completion,
    Unknown,
}

impl TaskEventCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Contract => "contract",
            Self::Workflow => "workflow",
            Self::Runtime => "runtime",
            Self::Session => "session",
            Self::ReviewDelivery => "review-delivery",
            Self::Completion => "completion",
            Self::Unknown => "unknown",
        }
    }
}

pub fn classify_task_event(event_type: &str) -> TaskEventCategory {
    match event_type {
        value if value.starts_with("spec.") => TaskEventCategory::Contract,
        value
            if value.starts_with("project.state.")
                || value.starts_with("project.intake.")
                || value.starts_with("project.loop.")
                || value.starts_with("goal.draft.")
                || value.starts_with("plan.draft.")
                || value.starts_with("work.state.")
                || value.starts_with("audit.state.")
                || value.starts_with("delivery.state.")
                || matches!(
                    value,
                    "issue.scheduled"
                        | "issue.blocked"
                        | "issue.cancelled"
                        | "issue.preflight.passed"
                        | "issue.preflight.failed"
                ) =>
        {
            TaskEventCategory::Workflow
        }
        value
            if value.starts_with("context.pack.")
                || value.starts_with("panel.context-pack.")
                || value.starts_with("run.")
                || value.starts_with("checkpoint.")
                || value.starts_with("verification.")
                || matches!(
                    value,
                    "RequirementSubmitted"
                        | "RequirementNormalized"
                        | "RequirementClassified"
                        | "SpecDrafted"
                        | "SpecApproved"
                        | "ProjectCreated"
                        | "IssueCreated"
                        | "IssueActivated"
                        | "RunStarted"
                        | "EvidenceSubmitted"
                        | "ArtifactSubmitted"
                        | "IssueMarkedDone"
                        | "DecisionRecorded"
                        | "AuditRequested"
                        | "FindingCreated"
                        | "FixIssueLinked"
                        | "ObjectStateChanged"
                        | "ActionRejectedRecorded"
                ) =>
        {
            TaskEventCategory::Runtime
        }
        value if value.starts_with("agent.launch.") || value.starts_with("agent.session.") => {
            TaskEventCategory::Session
        }
        value if value.starts_with("issue.lease.") => TaskEventCategory::Session,
        value
            if value.starts_with("review.")
                || value.starts_with("audit.")
                || value.starts_with("delivery.")
                || value.starts_with("issue.acceptance.")
                || value.starts_with("issue.validation.")
                || value.starts_with("issue.review.")
                || value.starts_with("issue.pr.")
                || value == "issue.closeout.proof.recorded" =>
        {
            TaskEventCategory::ReviewDelivery
        }
        value
            if value.starts_with("merge.")
                || value == "issue.completed"
                || value.starts_with("project.goal_recheck.")
                || value == "project.accepted" =>
        {
            TaskEventCategory::Completion
        }
        _ => TaskEventCategory::Unknown,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskReplayCursor {
    pub flow_type: WorkflowFlowType,
    pub aggregate_type: String,
    pub aggregate_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issue_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    pub after_event_id: String,
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
