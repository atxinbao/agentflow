use serde::{Deserialize, Serialize};

pub const SPEC_MANIFEST_VERSION: &str = "agentflow-spec-manifest.v1";
pub const SPEC_INDEX_VERSION: &str = "agentflow-spec-index.v1";
pub const SPEC_ISSUE_VERSION: &str = "agentflow-spec-issue.v1";
pub const SPEC_PROJECT_VERSION: &str = "agentflow-spec-project.v1";
pub const PROJECT_BRAIN_DOCUMENT_SET_VERSION: &str = "agentflow-project-brain-document-set.v1";
pub const PROJECT_BRAIN_SNAPSHOT_VERSION: &str = "agentflow-project-brain-snapshot.v1";
pub const DEFAULT_WORKFLOW_REF: &str = "build-agent.issue-loop@v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SpecIssueCategory {
    Spec,
    Audit,
}

impl Default for SpecIssueCategory {
    fn default() -> Self {
        Self::Spec
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SpecRequiredAgentRole {
    BuildAgent,
    AuditAgent,
}

impl Default for SpecRequiredAgentRole {
    fn default() -> Self {
        Self::BuildAgent
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpecIssueStatus {
    #[serde(rename = "backlog")]
    Backlog,
    #[serde(rename = "todo")]
    Todo,
    #[serde(rename = "in_progress")]
    InProgress,
    #[serde(rename = "in_review")]
    InReview,
    #[serde(rename = "done")]
    Done,
    #[serde(rename = "blocked")]
    Blocked,
    #[serde(rename = "cancel")]
    Cancel,
}

impl Default for SpecIssueStatus {
    fn default() -> Self {
        Self::Backlog
    }
}

impl SpecIssueStatus {
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
pub enum SpecPriority {
    #[serde(rename = "P0", alias = "p0")]
    P0,
    #[serde(rename = "P1", alias = "p1")]
    P1,
    #[serde(rename = "P2", alias = "p2")]
    P2,
    #[serde(rename = "P3", alias = "p3")]
    P3,
}

impl Default for SpecPriority {
    fn default() -> Self {
        Self::P2
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SpecProjectStatus {
    Planned,
    Active,
    Done,
    Blocked,
    Cancel,
}

impl Default for SpecProjectStatus {
    fn default() -> Self {
        Self::Planned
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProjectBrainDocumentStatus {
    Missing,
    Draft,
    NeedsConfirmation,
    Confirmed,
    Stale,
    Blocked,
}

impl Default for ProjectBrainDocumentStatus {
    fn default() -> Self {
        Self::Missing
    }
}

impl ProjectBrainDocumentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Missing => "missing",
            Self::Draft => "draft",
            Self::NeedsConfirmation => "needs-confirmation",
            Self::Confirmed => "confirmed",
            Self::Stale => "stale",
            Self::Blocked => "blocked",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProjectBrainStatus {
    NotInitialized,
    NeedsGoal,
    NeedsPlan,
    NeedsConfirmation,
    ReadyForProjectLoop,
    NeedsRecheck,
    Blocked,
}

impl Default for ProjectBrainStatus {
    fn default() -> Self {
        Self::NotInitialized
    }
}

impl ProjectBrainStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::NotInitialized => "not-initialized",
            Self::NeedsGoal => "needs-goal",
            Self::NeedsPlan => "needs-plan",
            Self::NeedsConfirmation => "needs-confirmation",
            Self::ReadyForProjectLoop => "ready-for-project-loop",
            Self::NeedsRecheck => "needs-recheck",
            Self::Blocked => "blocked",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectBrainDocumentSet {
    pub version: String,
    pub project_id: String,
    pub root_path: String,
    pub goal_path: String,
    pub plan_path: String,
    pub decisions_path: String,
    pub health_path: String,
    pub goal_exists: bool,
    pub plan_exists: bool,
    pub decisions_exists: bool,
    pub health_exists: bool,
    pub goal_updated_at: Option<u64>,
    pub plan_updated_at: Option<u64>,
    pub decisions_updated_at: Option<u64>,
    pub health_updated_at: Option<u64>,
    pub missing_documents: Vec<String>,
    pub readonly: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectBrainSnapshot {
    pub version: String,
    pub project_id: String,
    pub project_title: String,
    pub project_path: String,
    pub goal_document: String,
    pub plan_document: String,
    pub decisions_document: String,
    pub health_document: String,
    pub goal_status: ProjectBrainDocumentStatus,
    pub plan_status: ProjectBrainDocumentStatus,
    pub decision_status: ProjectBrainDocumentStatus,
    pub health_status: ProjectBrainDocumentStatus,
    pub brain_status: ProjectBrainStatus,
    pub missing_documents: Vec<String>,
    pub open_questions: Vec<String>,
    pub next_recommended_action: String,
    pub next_recommended_action_label: String,
    pub next_recommended_action_reason: String,
    pub readonly: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicDeliveryRecord {
    pub pr_or_mr_body: bool,
    pub changelog_or_release_notes: String,
}

impl Default for PublicDeliveryRecord {
    fn default() -> Self {
        Self {
            pr_or_mr_body: true,
            changelog_or_release_notes: "required-when-release-visible".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpecExpectedOutputs {
    pub task_run_dir: String,
    pub evidence_path: String,
    pub public_delivery_record: PublicDeliveryRecord,
}

impl SpecExpectedOutputs {
    pub fn for_issue(issue_id: &str) -> Self {
        Self {
            task_run_dir: format!(".agentflow/tasks/{issue_id}/runs/<run-id>"),
            evidence_path: format!(".agentflow/tasks/{issue_id}/evidence/evidence.json"),
            public_delivery_record: PublicDeliveryRecord::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpecSystemRecord {
    pub created_by: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub path: String,
    pub public_requirement_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequirementDocument {
    pub requirement_id: String,
    pub path: String,
    pub title: String,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpecIssue {
    pub version: String,
    pub issue_id: String,
    pub issue_category: SpecIssueCategory,
    pub required_agent_role: SpecRequiredAgentRole,
    pub status: SpecIssueStatus,
    pub workflow_ref: String,
    pub source_requirement_id: String,
    pub source_requirement_path: String,
    pub source_spec_id: String,
    pub project_id: Option<String>,
    pub title: String,
    pub summary: String,
    pub priority: SpecPriority,
    pub blocked_by: Vec<String>,
    pub allowed_paths: Vec<String>,
    pub forbidden_paths: Vec<String>,
    pub validation_commands: Vec<String>,
    pub expected_outputs: SpecExpectedOutputs,
    pub system: SpecSystemRecord,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpecProject {
    pub version: String,
    pub project_id: String,
    pub source_requirement_id: String,
    pub source_requirement_path: String,
    pub title: String,
    pub summary: String,
    pub objective: String,
    pub issue_ids: Vec<String>,
    pub status: SpecProjectStatus,
    pub system: SpecSystemRecord,
}

#[derive(Debug, Clone)]
pub struct SpecIssueDraft {
    pub issue_id: String,
    pub title: Option<String>,
    pub summary: Option<String>,
    pub workflow_ref: String,
    pub source_spec_id: Option<String>,
    pub project_id: Option<String>,
    pub priority: SpecPriority,
    pub blocked_by: Vec<String>,
    pub allowed_paths: Vec<String>,
    pub forbidden_paths: Vec<String>,
    pub validation_commands: Vec<String>,
}

impl SpecIssueDraft {
    pub fn new(issue_id: impl Into<String>) -> Self {
        Self {
            issue_id: issue_id.into(),
            title: None,
            summary: None,
            workflow_ref: DEFAULT_WORKFLOW_REF.to_string(),
            source_spec_id: None,
            project_id: None,
            priority: SpecPriority::default(),
            blocked_by: Vec::new(),
            allowed_paths: Vec::new(),
            forbidden_paths: vec![".agentflow/**".to_string()],
            validation_commands: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SpecProjectDraft {
    pub project_id: String,
    pub title: Option<String>,
    pub summary: Option<String>,
    pub objective: Option<String>,
    pub issue_ids: Vec<String>,
}

impl SpecProjectDraft {
    pub fn new(project_id: impl Into<String>) -> Self {
        Self {
            project_id: project_id.into(),
            title: None,
            summary: None,
            objective: None,
            issue_ids: Vec::new(),
        }
    }
}
