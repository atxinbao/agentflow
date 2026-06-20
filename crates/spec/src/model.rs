use agentflow_workflow_core::{WorkflowAgentRole, WorkflowSkillPack};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const SPEC_MANIFEST_VERSION: &str = "agentflow-spec-manifest.v1";
pub const SPEC_INDEX_VERSION: &str = "agentflow-spec-index.v1";
pub const SPEC_ISSUE_VERSION: &str = "agentflow-spec-issue.v1";
pub const SPEC_PROJECT_VERSION: &str = "agentflow-spec-project.v1";
pub const REQUIREMENT_PREVIEW_VERSION: &str = "agentflow-requirement-preview.v1";
pub const SPEC_REQUIREMENT_MANIFEST_VERSION: &str = "agentflow-spec-requirement-manifest.v1";
pub const SPEC_STAGE_ARTIFACT_VERSION: &str = "agentflow-spec-stage-artifact.v1";
pub const REQUIREMENT_CLASSIFICATION_VERSION: &str = "agentflow-requirement-classification.v1";
pub const COMPLETION_DECISION_VERSION: &str = "agentflow-completion-decision.v1";
pub const PROJECT_BRAIN_DOCUMENT_SET_VERSION: &str = "agentflow-project-brain-document-set.v1";
pub const PROJECT_BRAIN_SNAPSHOT_VERSION: &str = "agentflow-project-brain-snapshot.v1";
pub const DEFAULT_WORKFLOW_REF: &str = "work-agent.issue-loop@v1";

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
    #[serde(rename = "work-agent", alias = "build-agent")]
    BuildAgent,
    AuditAgent,
}

impl Default for SpecRequiredAgentRole {
    fn default() -> Self {
        Self::BuildAgent
    }
}

impl SpecRequiredAgentRole {
    pub fn runtime_role(&self) -> WorkflowAgentRole {
        match self {
            Self::BuildAgent => WorkflowAgentRole::WorkAgent,
            Self::AuditAgent => WorkflowAgentRole::AuditAgent,
        }
    }

    pub fn default_skill_pack(&self) -> WorkflowSkillPack {
        match self {
            Self::BuildAgent => WorkflowSkillPack::ExecutionSkills,
            Self::AuditAgent => WorkflowSkillPack::JudgmentSkills,
        }
    }

    pub fn provider_role_alias(&self) -> &'static str {
        match self {
            Self::BuildAgent => "build-agent",
            Self::AuditAgent => "audit-agent",
        }
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
pub enum RequirementIntentType {
    Product,
    Design,
    Technical,
    Repair,
    Audit,
    Understanding,
    Mixed,
    Unknown,
}

impl Default for RequirementIntentType {
    fn default() -> Self {
        Self::Unknown
    }
}

impl RequirementIntentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Product => "product",
            Self::Design => "design",
            Self::Technical => "technical",
            Self::Repair => "repair",
            Self::Audit => "audit",
            Self::Understanding => "understanding",
            Self::Mixed => "mixed",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GoalDraftStatus {
    NeedsClarification,
    ReadyForReview,
    Confirmed,
    Rejected,
    SplitRequired,
}

impl Default for GoalDraftStatus {
    fn default() -> Self {
        Self::ReadyForReview
    }
}

impl GoalDraftStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::NeedsClarification => "needs-clarification",
            Self::ReadyForReview => "ready-for-review",
            Self::Confirmed => "confirmed",
            Self::Rejected => "rejected",
            Self::SplitRequired => "split-required",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PlanDraftStatus {
    Draft,
    ReadyForReview,
    Confirmed,
    Rejected,
    NeedsRevision,
    Blocked,
}

impl Default for PlanDraftStatus {
    fn default() -> Self {
        Self::Draft
    }
}

impl PlanDraftStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::ReadyForReview => "ready-for-review",
            Self::Confirmed => "confirmed",
            Self::Rejected => "rejected",
            Self::NeedsRevision => "needs-revision",
            Self::Blocked => "blocked",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RequirementPreviewLifecycle {
    Active,
    Cancelled,
    Materialized,
}

impl Default for RequirementPreviewLifecycle {
    fn default() -> Self {
        Self::Active
    }
}

impl RequirementPreviewLifecycle {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Cancelled => "cancelled",
            Self::Materialized => "materialized",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SpecLoopStageName {
    Intake,
    Classification,
    Context,
    Boundary,
    Route,
    Preview,
    Confirmation,
    Materialization,
}

impl SpecLoopStageName {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Intake => "intake",
            Self::Classification => "classification",
            Self::Context => "context",
            Self::Boundary => "boundary",
            Self::Route => "route",
            Self::Preview => "preview",
            Self::Confirmation => "confirmation",
            Self::Materialization => "materialization",
        }
    }

    pub fn file_name(&self) -> &'static str {
        match self {
            Self::Intake => "intake.json",
            Self::Classification => "classification.json",
            Self::Context => "context.json",
            Self::Boundary => "boundary.json",
            Self::Route => "route.json",
            Self::Preview => "preview.json",
            Self::Confirmation => "confirmation.json",
            Self::Materialization => "materialization.json",
        }
    }

    pub fn all() -> &'static [SpecLoopStageName] {
        const STAGES: &[SpecLoopStageName] = &[
            SpecLoopStageName::Intake,
            SpecLoopStageName::Classification,
            SpecLoopStageName::Context,
            SpecLoopStageName::Boundary,
            SpecLoopStageName::Route,
            SpecLoopStageName::Preview,
            SpecLoopStageName::Confirmation,
            SpecLoopStageName::Materialization,
        ];
        STAGES
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SpecLoopStageStatus {
    Declared,
    Ready,
    Confirmed,
    Materialized,
    Cancelled,
}

impl SpecLoopStageStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Declared => "declared",
            Self::Ready => "ready",
            Self::Confirmed => "confirmed",
            Self::Materialized => "materialized",
            Self::Cancelled => "cancelled",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SpecArtifactAuthority {
    Derived,
    Authority,
}

impl SpecArtifactAuthority {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Derived => "derived",
            Self::Authority => "authority",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpecLoopStageFileRef {
    pub stage: SpecLoopStageName,
    pub path: String,
    pub status: SpecLoopStageStatus,
    pub authority: SpecArtifactAuthority,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpecLoopRequirementManifest {
    pub version: String,
    pub requirement_id: String,
    pub project_id: String,
    pub root_path: String,
    pub runtime_path: String,
    pub stage_files: Vec<SpecLoopStageFileRef>,
    pub updated_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpecLoopStageArtifact {
    pub version: String,
    pub requirement_id: String,
    pub project_id: String,
    pub stage: SpecLoopStageName,
    pub status: SpecLoopStageStatus,
    pub authority: SpecArtifactAuthority,
    pub current_state: Option<String>,
    pub input_refs: Vec<String>,
    pub output_refs: Vec<String>,
    pub evidence_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload: Option<Value>,
    pub summary: String,
    pub updated_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RequirementClass {
    Question,
    Research,
    Feature,
    Bug,
    Audit,
    DesignOnly,
    ExecutableIssue,
    Release,
    Maintenance,
    Cleanup,
}

impl RequirementClass {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Question => "question",
            Self::Research => "research",
            Self::Feature => "feature",
            Self::Bug => "bug",
            Self::Audit => "audit",
            Self::DesignOnly => "design-only",
            Self::ExecutableIssue => "executable-issue",
            Self::Release => "release",
            Self::Maintenance => "maintenance",
            Self::Cleanup => "cleanup",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RequirementExecutionPermission {
    AnswerOnly,
    PreviewOnly,
    SpecLoop,
    AuditLoop,
    ReleaseCloseout,
}

impl RequirementExecutionPermission {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AnswerOnly => "answer-only",
            Self::PreviewOnly => "preview-only",
            Self::SpecLoop => "spec-loop",
            Self::AuditLoop => "audit-loop",
            Self::ReleaseCloseout => "release-closeout",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RequirementFactImpact {
    ReadOnly,
    RequirementPreview,
    SpecAuthority,
    RuntimeProposal,
    AuditSurface,
    ReleaseSurface,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RequirementRiskLevel {
    Low,
    Medium,
    High,
}

impl RequirementRiskLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RequirementTargetObject {
    Requirement,
    SpecProject,
    SpecIssue,
    Code,
    Design,
    Audit,
    Release,
    Documentation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequirementClassificationResult {
    pub version: String,
    pub primary_type: RequirementClass,
    pub basic_types: Vec<RequirementClass>,
    pub intent_type: RequirementIntentType,
    pub execution_permission: RequirementExecutionPermission,
    pub fact_impacts: Vec<RequirementFactImpact>,
    pub risk_level: RequirementRiskLevel,
    pub target_objects: Vec<RequirementTargetObject>,
    pub confirmation_required: bool,
    pub ambiguous: bool,
    pub conflicting: bool,
    pub reasons: Vec<String>,
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
pub struct RequirementIntakeResult {
    pub requirement_id: String,
    pub project_id: String,
    pub raw_text: String,
    #[serde(default = "default_agent_locale")]
    pub agent_locale: String,
    #[serde(default)]
    pub referenced_files: Vec<String>,
    #[serde(default)]
    pub referenced_urls: Vec<String>,
    #[serde(default)]
    pub referenced_versions: Vec<String>,
    #[serde(default)]
    pub referenced_releases: Vec<String>,
    #[serde(default)]
    pub referenced_branches: Vec<String>,
    #[serde(default)]
    pub referenced_issues: Vec<String>,
    #[serde(default)]
    pub referenced_pull_requests: Vec<String>,
    #[serde(default)]
    pub explicit_actions: Vec<String>,
    #[serde(default)]
    pub input_sources: Vec<String>,
    pub detected_intent: RequirementIntentType,
    pub detected_scope: Vec<String>,
    pub detected_non_goals: Vec<String>,
    pub detected_deliverables: Vec<String>,
    pub detected_constraints: Vec<String>,
    pub missing_information: Vec<String>,
    pub clarification_questions: Vec<String>,
    pub confidence: u8,
    pub next_action: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoalDraftPreview {
    pub goal_draft_id: String,
    pub project_id: String,
    pub source_requirement_id: String,
    pub title: String,
    pub intent_type: RequirementIntentType,
    pub outcome: String,
    pub target_user: String,
    pub expected_deliverables: Vec<String>,
    pub scope: Vec<String>,
    pub non_goals: Vec<String>,
    pub success_criteria: Vec<String>,
    pub constraints: Vec<String>,
    pub open_questions: Vec<String>,
    pub risk_hints: Vec<String>,
    pub confidence: u8,
    pub status: GoalDraftStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MilestoneDraftPreview {
    pub milestone_id: String,
    pub title: String,
    pub goal: String,
    pub depends_on: Vec<String>,
    pub expected_outputs: Vec<String>,
    pub validation_need: String,
    pub evidence_need: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueContractDraftPreview {
    pub issue_draft_id: String,
    pub title: String,
    pub goal: String,
    pub scope: Vec<String>,
    pub non_goals: Vec<String>,
    pub dependencies: Vec<String>,
    pub acceptance_criteria: Vec<String>,
    pub validation_commands: Vec<String>,
    pub evidence_requirements: Vec<String>,
    pub boundary: Vec<String>,
    pub priority: SpecPriority,
    pub suggested_agent_role: SpecRequiredAgentRole,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanDraftPreview {
    pub plan_draft_id: String,
    pub project_id: String,
    pub source_goal_id: String,
    pub plan_type: RequirementIntentType,
    pub stage_plan: Vec<String>,
    pub milestone_drafts: Vec<MilestoneDraftPreview>,
    pub issue_contract_drafts: Vec<IssueContractDraftPreview>,
    pub validation_strategy: Vec<String>,
    pub evidence_strategy: Vec<String>,
    pub human_confirmation_points: Vec<String>,
    pub risk_list: Vec<String>,
    pub blockers: Vec<String>,
    pub next_recommended_action: String,
    pub status: PlanDraftStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewConfirmationRecord {
    pub timestamp: u64,
    pub actor: String,
    pub target_type: String,
    pub target_id: String,
    pub summary: String,
    pub decision: String,
    pub impact: String,
    pub next_action: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequirementPreviewRuntime {
    pub version: String,
    pub requirement_id: String,
    pub requirement_path: String,
    pub project_id: String,
    pub project_title: String,
    pub revision: u32,
    pub lifecycle: RequirementPreviewLifecycle,
    pub current_state: String,
    pub intake: RequirementIntakeResult,
    pub goal_draft: GoalDraftPreview,
    pub plan_draft: Option<PlanDraftPreview>,
    pub confirmation_records: Vec<PreviewConfirmationRecord>,
    pub materialized_project_id: Option<String>,
    pub materialized_issue_ids: Vec<String>,
    pub next_recommended_action: String,
    pub next_recommended_action_label: String,
    pub next_recommended_action_reason: String,
    pub readonly: bool,
    pub updated_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CompletionDecisionState {
    GoalRecheck,
    Continue,
    Adjust,
    Pause,
    Accepted,
    NextStage,
}

impl Default for CompletionDecisionState {
    fn default() -> Self {
        Self::GoalRecheck
    }
}

impl CompletionDecisionState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::GoalRecheck => "goal-recheck",
            Self::Continue => "continue",
            Self::Adjust => "adjust",
            Self::Pause => "pause",
            Self::Accepted => "accepted",
            Self::NextStage => "next-stage",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CompletionDecisionOutcome {
    Continue,
    Adjust,
    Pause,
    Accept,
    NextStage,
}

impl CompletionDecisionOutcome {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Continue => "continue",
            Self::Adjust => "adjust",
            Self::Pause => "pause",
            Self::Accept => "accept",
            Self::NextStage => "next-stage",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompletionDecisionFacts {
    pub total_issue_count: usize,
    pub completed_issue_count: usize,
    pub canceled_issue_count: usize,
    pub remaining_issue_count: usize,
    pub blocked_issue_count: usize,
    #[serde(default)]
    pub task_evidence_ready_count: usize,
    #[serde(default)]
    pub task_evidence_missing_count: usize,
    #[serde(default = "default_completion_delivery_status")]
    pub delivery_status: String,
    #[serde(default)]
    pub delivery_missing_count: usize,
    #[serde(default)]
    pub audit_required: bool,
    #[serde(default = "default_completion_audit_status")]
    pub audit_status: String,
    #[serde(default)]
    pub audit_blocking_findings: usize,
    #[serde(default = "default_completion_goal_recheck_status")]
    pub goal_recheck_status: String,
    #[serde(default = "default_completion_project_health_status")]
    pub project_health_status: String,
    #[serde(default = "default_completion_release_readiness")]
    pub release_readiness: String,
}

fn default_completion_delivery_status() -> String {
    "missing".to_string()
}

fn default_completion_audit_status() -> String {
    "not-requested".to_string()
}

fn default_completion_goal_recheck_status() -> String {
    "not-ready".to_string()
}

fn default_completion_project_health_status() -> String {
    "missing".to_string()
}

fn default_completion_release_readiness() -> String {
    "blocked".to_string()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompletionDecisionRecord {
    pub actor: String,
    pub outcome: CompletionDecisionOutcome,
    pub summary: String,
    pub rationale: Vec<String>,
    pub decided_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompletionDecisionRuntime {
    pub version: String,
    pub project_id: String,
    pub project_title: String,
    pub source_requirement_id: String,
    pub current_state: CompletionDecisionState,
    pub latest_outcome: Option<CompletionDecisionOutcome>,
    pub facts: CompletionDecisionFacts,
    pub open_questions: Vec<String>,
    pub rationale: Vec<String>,
    pub history: Vec<CompletionDecisionRecord>,
    pub next_recommended_action: String,
    pub next_recommended_action_label: String,
    pub next_recommended_action_reason: String,
    pub readonly: bool,
    pub updated_at: u64,
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
    pub raw_text: String,
}

fn default_agent_locale() -> String {
    "zh-CN".to_string()
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
