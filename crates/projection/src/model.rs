use serde::{Deserialize, Serialize};

pub const TASK_PROJECTION_VERSION: &str = "task-projection.v2";
pub const PROJECT_PROJECTION_VERSION: &str = "project-projection.v3";
pub const ISSUE_STATUS_INDEX_VERSION: &str = "issue-status-index.v3";
pub const REQUIREMENT_PREVIEW_PROJECTION_VERSION: &str = "requirement-preview-projection.v1";
pub const REQUIREMENT_PREVIEW_INDEX_VERSION: &str = "requirement-preview-index.v1";
pub const COMPLETION_DECISION_PROJECTION_VERSION: &str = "completion-decision-projection.v1";
pub const COMPLETION_DECISION_INDEX_VERSION: &str = "completion-decision-index.v1";

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

fn default_projection_runtime_status() -> String {
    "missing".to_string()
}

fn default_projection_delivery_status() -> String {
    "missing".to_string()
}

fn default_projection_audit_status() -> String {
    "not-requested".to_string()
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectionRuntimeSummary {
    pub run_id: Option<String>,
    #[serde(default = "default_projection_runtime_status")]
    pub run_status: String,
    pub branch_name: Option<String>,
    #[serde(default)]
    pub checkpoint_count: usize,
    pub latest_checkpoint_id: Option<String>,
    pub latest_checkpoint_state: Option<String>,
    pub latest_checkpoint_summary: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectionSessionSummary {
    pub provider: Option<String>,
    pub session_id: Option<String>,
    pub status: Option<String>,
    pub launch_requested_at: Option<u64>,
    pub claimed_at: Option<u64>,
    pub created_at: Option<u64>,
    pub updated_at: Option<u64>,
    pub launch_request_path: Option<String>,
    pub plan_path: Option<String>,
    pub log_path: Option<String>,
    pub branch_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectionDeliverySummary {
    #[serde(default = "default_projection_delivery_status")]
    pub status: String,
    #[serde(default = "default_projection_delivery_status")]
    pub evidence_status: String,
    pub evidence_path: Option<String>,
    pub pr_url: Option<String>,
    pub merge_commit: Option<String>,
    pub public_record_path: Option<String>,
    #[serde(default)]
    pub summary_line: String,
    #[serde(default)]
    pub public_record_items: Vec<String>,
    #[serde(default)]
    pub missing_public_records: Vec<String>,
    #[serde(default)]
    pub current_issue_id: Option<String>,
    #[serde(default)]
    pub published_count: usize,
    #[serde(default)]
    pub ready_count: usize,
    #[serde(default)]
    pub missing_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectionAuditSummary {
    #[serde(default = "default_projection_audit_status")]
    pub status: String,
    pub latest_audit_id: Option<String>,
    #[serde(default)]
    pub source_issue_id: Option<String>,
    pub report_path: Option<String>,
    pub requested_at: Option<u64>,
    #[serde(default)]
    pub summary_line: String,
    #[serde(default)]
    pub findings_count: usize,
    #[serde(default)]
    pub findings: Vec<String>,
    #[serde(default)]
    pub evidence_gaps: Vec<String>,
    #[serde(default)]
    pub repair_recommendations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectBrainProjection {
    pub project_path: String,
    pub goal_path: String,
    pub plan_path: String,
    pub decisions_path: String,
    pub health_path: String,
    pub brain_status: String,
    pub goal_status: String,
    pub plan_status: String,
    pub decision_status: String,
    pub health_status: String,
    pub missing_documents: Vec<String>,
    pub open_questions: Vec<String>,
    pub next_recommended_action: String,
    pub next_recommended_action_label: String,
    pub next_recommended_action_reason: String,
    pub readonly: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectCompletionProjection {
    pub current_state: String,
    pub latest_outcome: Option<String>,
    pub next_recommended_action: String,
    pub next_recommended_action_label: String,
    pub next_recommended_action_reason: String,
    pub total_issue_count: usize,
    pub completed_issue_count: usize,
    pub canceled_issue_count: usize,
    pub remaining_issue_count: usize,
    pub blocked_issue_count: usize,
    pub open_questions: Vec<String>,
    pub rationale: Vec<String>,
    pub updated_at: u64,
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
    #[serde(default)]
    pub runtime: ProjectionRuntimeSummary,
    #[serde(default)]
    pub session: ProjectionSessionSummary,
    #[serde(default)]
    pub delivery: ProjectionDeliverySummary,
    #[serde(default)]
    pub audit: ProjectionAuditSummary,
    pub updated_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectIssueLanes {
    pub current: Vec<String>,
    pub past: Vec<String>,
    pub future: Vec<String>,
    pub blocked: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectBlockerSummary {
    pub issue_id: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectProjection {
    pub version: String,
    pub project_id: String,
    pub title: String,
    pub objective: String,
    pub status: String,
    #[serde(default)]
    pub stage_key: String,
    #[serde(default)]
    pub stage_label: String,
    #[serde(default)]
    pub stage_summary: String,
    pub issue_ids: Vec<String>,
    pub current_issue_id: Option<String>,
    #[serde(default)]
    pub lanes: ProjectIssueLanes,
    #[serde(default)]
    pub next_action: String,
    #[serde(default)]
    pub next_action_label: String,
    #[serde(default)]
    pub next_action_reason: String,
    #[serde(default)]
    pub blockers: Vec<ProjectBlockerSummary>,
    #[serde(default)]
    pub completion_hint: String,
    #[serde(default)]
    pub completion: Option<ProjectCompletionProjection>,
    #[serde(default)]
    pub delivery: Option<ProjectionDeliverySummary>,
    #[serde(default)]
    pub audit: Option<ProjectionAuditSummary>,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequirementPreviewProjection {
    pub version: String,
    pub requirement_id: String,
    pub requirement_path: String,
    pub project_id: String,
    pub project_title: String,
    pub lifecycle: String,
    pub current_state: String,
    pub goal_status: String,
    pub plan_status: Option<String>,
    pub next_recommended_action: String,
    pub next_recommended_action_label: String,
    pub next_recommended_action_reason: String,
    pub issue_contract_draft_count: usize,
    pub materialized_project_id: Option<String>,
    pub materialized_issue_ids: Vec<String>,
    pub updated_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequirementPreviewIndexEntry {
    pub requirement_id: String,
    pub project_id: String,
    pub current_state: String,
    pub lifecycle: String,
    pub next_recommended_action: String,
    pub projection_path: String,
    pub updated_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequirementPreviewIndex {
    pub version: String,
    pub updated_at: u64,
    pub previews: Vec<RequirementPreviewIndexEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompletionDecisionProjection {
    pub version: String,
    pub project_id: String,
    pub project_title: String,
    pub current_state: String,
    pub latest_outcome: Option<String>,
    pub next_recommended_action: String,
    pub next_recommended_action_label: String,
    pub next_recommended_action_reason: String,
    pub total_issue_count: usize,
    pub completed_issue_count: usize,
    pub canceled_issue_count: usize,
    pub remaining_issue_count: usize,
    pub blocked_issue_count: usize,
    pub open_questions: Vec<String>,
    pub rationale: Vec<String>,
    pub projection_path: String,
    pub updated_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompletionDecisionIndexEntry {
    pub project_id: String,
    pub current_state: String,
    pub latest_outcome: Option<String>,
    pub next_recommended_action: String,
    pub projection_path: String,
    pub updated_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompletionDecisionIndex {
    pub version: String,
    pub updated_at: u64,
    pub decisions: Vec<CompletionDecisionIndexEntry>,
}
