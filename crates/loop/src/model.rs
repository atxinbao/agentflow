use agentflow_input::issue::DisplayStatus;
use serde::{Deserialize, Serialize};

pub const LOOP_PROJECT_SNAPSHOT_VERSION: &str = "agentflow-loop-project.v1";
pub const LOOP_ISSUE_PROJECTION_VERSION: &str = "agentflow-loop-issue.v1";
pub const LOOP_AUDIT_GATE_VERSION: &str = "agentflow-loop-audit-gate.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectLoopStatus {
    Active,
    PreflightBlocked,
    Scheduling,
    Executing,
    Auditing,
    Done,
    Blocked,
    Cancel,
}

impl Default for ProjectLoopStatus {
    fn default() -> Self {
        Self::Active
    }
}

impl ProjectLoopStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::PreflightBlocked => "preflight_blocked",
            Self::Scheduling => "scheduling",
            Self::Executing => "executing",
            Self::Auditing => "auditing",
            Self::Done => "done",
            Self::Blocked => "blocked",
            Self::Cancel => "cancel",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueLoopStage {
    Backlog,
    Todo,
    InProgress,
    InReview,
    Done,
    Blocked,
    Cancel,
}

impl Default for IssueLoopStage {
    fn default() -> Self {
        Self::Backlog
    }
}

impl IssueLoopStage {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Backlog => "backlog",
            Self::Todo => "todo",
            Self::InProgress => "in_progress",
            Self::InReview => "in_review",
            Self::Done => "done",
            Self::Blocked => "blocked",
            Self::Cancel => "cancel",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditGateKind {
    Delivery,
    ProjectFinal,
}

impl Default for AuditGateKind {
    fn default() -> Self {
        Self::Delivery
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditGateStatus {
    Pending,
    Running,
    Passed,
    PassedWithWarnings,
    Failed,
    Blocked,
}

impl Default for AuditGateStatus {
    fn default() -> Self {
        Self::Pending
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoopBlocker {
    pub code: String,
    pub reason: String,
    pub source_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectLoopSnapshot {
    pub version: String,
    pub project_id: String,
    pub status: ProjectLoopStatus,
    pub active_issue_ids: Vec<String>,
    pub blocked_issue_ids: Vec<String>,
    pub done_issue_ids: Vec<String>,
    pub audit_status: AuditGateStatus,
    pub blockers: Vec<LoopBlocker>,
    pub updated_at: u64,
}

impl ProjectLoopSnapshot {
    pub fn new(project_id: impl Into<String>, updated_at: u64) -> Self {
        Self {
            version: LOOP_PROJECT_SNAPSHOT_VERSION.to_string(),
            project_id: project_id.into(),
            status: ProjectLoopStatus::Active,
            active_issue_ids: Vec::new(),
            blocked_issue_ids: Vec::new(),
            done_issue_ids: Vec::new(),
            audit_status: AuditGateStatus::Pending,
            blockers: Vec::new(),
            updated_at,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueLoopProjection {
    pub version: String,
    pub project_id: Option<String>,
    pub issue_id: String,
    pub stage: IssueLoopStage,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_status: Option<DisplayStatus>,
    pub run_id: Option<String>,
    pub branch_name: Option<String>,
    pub review_substate: Option<String>,
    pub blockers: Vec<LoopBlocker>,
    pub updated_at: u64,
}

impl IssueLoopProjection {
    pub fn new(project_id: Option<String>, issue_id: impl Into<String>, updated_at: u64) -> Self {
        Self {
            version: LOOP_ISSUE_PROJECTION_VERSION.to_string(),
            project_id,
            issue_id: issue_id.into(),
            stage: IssueLoopStage::Backlog,
            display_status: None,
            run_id: None,
            branch_name: None,
            review_substate: None,
            blockers: Vec::new(),
            updated_at,
        }
    }
}
