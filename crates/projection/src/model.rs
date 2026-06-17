use serde::{Deserialize, Serialize};

pub const TASK_PROJECTION_VERSION: &str = "task-projection.v1";
pub const PROJECT_PROJECTION_VERSION: &str = "project-projection.v1";
pub const ISSUE_STATUS_INDEX_VERSION: &str = "issue-status-index.v2";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProjectionPhase {
    Past,
    Current,
    Future,
    Exception,
}

impl ProjectionPhase {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Past => "past",
            Self::Current => "current",
            Self::Future => "future",
            Self::Exception => "exception",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskTimelineEvent {
    pub event_id: String,
    pub event_type: String,
    pub timestamp: u64,
    pub actor_role: String,
    pub actor_kind: String,
    pub summary: String,
    pub artifact_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskTimelineItem {
    pub state: String,
    pub phase: ProjectionPhase,
    pub entered_at: Option<u64>,
    pub events: Vec<TaskTimelineEvent>,
    pub summary: String,
    pub live_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectionPublicDelivery {
    pub evidence_path: Option<String>,
    pub pr_url: Option<String>,
    pub merge_commit: Option<String>,
    pub changelog_path: Option<String>,
    pub release_notes_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectBrainProjection {
    pub project_path: String,
    pub goal_path: String,
    pub plan_path: String,
    pub decisions_path: String,
    pub brain_status: String,
    pub goal_status: String,
    pub plan_status: String,
    pub decision_status: String,
    pub missing_documents: Vec<String>,
    pub open_questions: Vec<String>,
    pub next_recommended_action: String,
    pub readonly: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskProjection {
    pub version: String,
    pub issue_id: String,
    pub project_id: Option<String>,
    pub workflow_ref: String,
    pub current_state: String,
    pub display_status: String,
    pub current_transition: Option<String>,
    pub latest_run_id: Option<String>,
    pub branch_name: Option<String>,
    pub timeline: Vec<TaskTimelineItem>,
    pub public_delivery: ProjectionPublicDelivery,
    pub updated_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectProjection {
    pub version: String,
    pub project_id: String,
    pub title: String,
    pub objective: String,
    pub status: String,
    pub issue_ids: Vec<String>,
    pub current_issue_id: Option<String>,
    pub issue_count: usize,
    pub completed_issue_count: usize,
    pub project_brain: ProjectBrainProjection,
    pub updated_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueStatusIndexEntry {
    pub issue_id: String,
    pub project_id: Option<String>,
    pub title: String,
    pub current_state: String,
    pub display_status: String,
    pub workflow_ref: String,
    pub projection_path: String,
    pub updated_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueStatusIndex {
    pub version: String,
    pub updated_at: u64,
    pub issues: Vec<IssueStatusIndexEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectionSummary {
    pub task_count: usize,
    pub project_count: usize,
    pub index_path: String,
}
