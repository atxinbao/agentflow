use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const STATE_MANIFEST_VERSION: &str = "state-manifest.v1";
pub const STATE_INDEX_VERSION: &str = "state-index.v1";
pub const STATE_HEALTH_VERSION: &str = "state-health.v1";
pub const STATE_WORKFLOW_GATES_VERSION: &str = "state-workflow-gates.v1";
pub const STATE_NEXT_ACTIONS_VERSION: &str = "state-next-actions.v1";
pub const STATE_BLOCKERS_VERSION: &str = "state-blockers.v1";
pub const STATE_SESSION_VERSION: &str = "state-session.v1";
pub const STATE_LOCKS_VERSION: &str = "state-locks.v1";
pub const STATE_TIMELINE_EVENT_VERSION: &str = "state-timeline-event.v1";
pub const STATE_STATUS_VERSION: &str = "state-status.v1";
pub const STATE_WORKSPACE_STATUS_INDEX_VERSION: &str = "state-workspace-status-index.v1";
pub const STATE_ISSUE_STATUS_INDEX_VERSION: &str = "state-issue-status-index.v1";
pub const STATE_RUN_STATUS_INDEX_VERSION: &str = "state-run-status-index.v1";
pub const STATE_OUTPUT_STATUS_INDEX_VERSION: &str = "state-output-status-index.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StateWorkspaceStatus {
    Missing,
    Ready,
    Degraded,
    Failed,
    Blocked,
}

impl StateWorkspaceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Missing => "missing",
            Self::Ready => "ready",
            Self::Degraded => "degraded",
            Self::Failed => "failed",
            Self::Blocked => "blocked",
        }
    }
}

impl Default for StateWorkspaceStatus {
    fn default() -> Self {
        Self::Missing
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WorkflowStage {
    WorkspaceMissing,
    WorkspaceBlocked,
    WorkspaceReady,
    PanelReady,
    InputReady,
    IssueReady,
    ExecuteReady,
    ExecuteRunning,
    ExecuteBlocked,
    ExecuteCompleted,
    EvidenceReady,
    DeliveryReady,
    AuditRequested,
    AuditRunning,
    AuditCompleted,
    Failed,
}

impl WorkflowStage {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::WorkspaceMissing => "workspace-missing",
            Self::WorkspaceBlocked => "workspace-blocked",
            Self::WorkspaceReady => "workspace-ready",
            Self::PanelReady => "panel-ready",
            Self::InputReady => "input-ready",
            Self::IssueReady => "issue-ready",
            Self::ExecuteReady => "execute-ready",
            Self::ExecuteRunning => "execute-running",
            Self::ExecuteBlocked => "execute-blocked",
            Self::ExecuteCompleted => "execute-completed",
            Self::EvidenceReady => "evidence-ready",
            Self::DeliveryReady => "delivery-ready",
            Self::AuditRequested => "audit-requested",
            Self::AuditRunning => "audit-running",
            Self::AuditCompleted => "audit-completed",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WorkflowAuditStatus {
    NotRequested,
    Requested,
    Running,
    Passed,
    PassedWithWarnings,
    Failed,
    Cancelled,
}

impl WorkflowAuditStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::NotRequested => "not-requested",
            Self::Requested => "requested",
            Self::Running => "running",
            Self::Passed => "passed",
            Self::PassedWithWarnings => "passed-with-warnings",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl Default for WorkflowAuditStatus {
    fn default() -> Self {
        Self::NotRequested
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StateManifestSummary {
    pub health_ready: bool,
    pub current_stage: String,
    pub allowed_next_actions: usize,
    pub blocked_actions: usize,
    pub active_sessions: usize,
    pub active_locks: usize,
    pub stale_locks: usize,
    pub audit_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StateManifest {
    pub version: String,
    pub project_root: String,
    pub status: StateWorkspaceStatus,
    pub updated_at: u64,
    pub paths: BTreeMap<String, String>,
    pub summary: StateManifestSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StateIndexEntry {
    pub id: String,
    pub status: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StateEventIndex {
    pub timeline: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StateIndex {
    pub version: String,
    pub updated_at: u64,
    pub health: Vec<StateIndexEntry>,
    pub sessions: Vec<StateIndexEntry>,
    pub locks: Vec<StateIndexEntry>,
    pub events: StateEventIndex,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowHealthSnapshot {
    pub version: String,
    pub module: String,
    pub status: String,
    pub ready: bool,
    pub source_path: String,
    pub checked_at: u64,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowBlockedAction {
    pub action: String,
    pub reason: String,
    pub source_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowGateSnapshot {
    pub version: String,
    pub current_stage: WorkflowStage,
    pub audit_status: WorkflowAuditStatus,
    pub active_issue_id: Option<String>,
    pub active_run_id: Option<String>,
    pub latest_evidence_path: Option<String>,
    pub latest_delivery_path: Option<String>,
    pub allowed_next_actions: Vec<String>,
    pub blocked_actions: Vec<WorkflowBlockedAction>,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowNextAction {
    pub action: String,
    pub label: String,
    pub allowed: bool,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowNextActionsSnapshot {
    pub version: String,
    pub actions: Vec<WorkflowNextAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowBlockersSnapshot {
    pub version: String,
    pub blockers: Vec<WorkflowBlockedAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StateSession {
    pub version: String,
    pub session_id: String,
    pub project_root: String,
    pub active_role: String,
    pub active_issue_id: Option<String>,
    pub active_run_id: Option<String>,
    pub status: String,
    pub waiting_for_human: bool,
    pub last_action: String,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StateSessionUpdate {
    pub session_id: String,
    pub active_role: Option<String>,
    pub active_issue_id: Option<String>,
    pub active_run_id: Option<String>,
    pub status: Option<String>,
    pub waiting_for_human: Option<bool>,
    pub last_action: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StateLockEntry {
    pub kind: String,
    pub issue_id: Option<String>,
    pub run_id: Option<String>,
    pub source_path: String,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StateLockSnapshot {
    pub version: String,
    pub active: Vec<StateLockEntry>,
    pub stale: Vec<StateLockEntry>,
    pub cleanup_candidates: Vec<StateLockEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StateTimelineEvent {
    pub version: String,
    pub ts: u64,
    pub event: String,
    pub project_root: String,
    pub details: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StateTimelineEventDraft {
    pub event: String,
    pub details: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StateStatusSnapshot {
    pub version: String,
    pub project_root: String,
    pub status: StateWorkspaceStatus,
    pub current_stage: WorkflowStage,
    pub audit_status: WorkflowAuditStatus,
    pub active_issue_id: Option<String>,
    pub active_run_id: Option<String>,
    pub health: BTreeMap<String, String>,
    pub next_actions: Vec<String>,
    pub blockers: Vec<WorkflowBlockedAction>,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceStatusIndex {
    pub version: String,
    pub updated_at: u64,
    pub health: Vec<WorkflowHealthSnapshot>,
    pub current_stage: WorkflowStage,
    pub audit_status: WorkflowAuditStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueStatusIndexEntry {
    pub issue_id: String,
    pub risk_level: String,
    pub latest_run_id: Option<String>,
    pub execute_status: Option<String>,
    pub evidence_status: String,
    pub delivery_status: String,
    pub audit_status: WorkflowAuditStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueStatusIndex {
    pub version: String,
    pub updated_at: u64,
    pub issues: Vec<IssueStatusIndexEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunStatusIndexEntry {
    pub run_id: String,
    pub issue_id: String,
    pub execute_status: String,
    pub evidence_status: String,
    pub delivery_status: String,
    pub audit_status: WorkflowAuditStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunStatusIndex {
    pub version: String,
    pub updated_at: u64,
    pub runs: Vec<RunStatusIndexEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputStatusIndex {
    pub version: String,
    pub updated_at: u64,
    pub evidence: usize,
    pub release_deliveries: usize,
    pub audits: usize,
    pub audit_status: WorkflowAuditStatus,
}
