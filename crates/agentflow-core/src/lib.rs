use anyhow::{anyhow, bail, Context, Result};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

pub const AGENTFLOW_DIR: &str = ".agentflow";
pub const VERSION: &str = "0.0.1";
const FIRST_CANDIDATE: &str = "Goal Compiler + Core/CLI Bootstrap v0";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectStatus {
    Draft,
    Active,
    Paused,
    Completed,
    Canceled,
}

impl ProjectStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Active => "active",
            Self::Paused => "paused",
            Self::Completed => "completed",
            Self::Canceled => "canceled",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IssueStatus {
    Backlog,
    Todo,
    InProgress,
    InReview,
    Done,
    Canceled,
}

impl IssueStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Backlog => "backlog",
            Self::Todo => "todo",
            Self::InProgress => "in_progress",
            Self::InReview => "in_review",
            Self::Done => "done",
            Self::Canceled => "canceled",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectGoal {
    pub version: String,
    pub objective: String,
    pub success_criteria: Vec<String>,
    pub non_goals: Vec<String>,
    pub constraints: Vec<String>,
    pub first_candidate: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDefinition {
    pub version: String,
    pub source_goal: String,
    pub phase: String,
    pub status: String,
    pub outputs: Vec<ProjectDefinitionOutput>,
    pub rules: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDefinitionOutput {
    pub id: String,
    pub name: String,
    pub path: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AgentScopeState {
    pub version: String,
    pub wip_limit: u32,
    pub active_issue_id: Option<String>,
    pub current_phase: String,
    pub execution_authorized: bool,
    pub authorization_source: String,
    pub boundaries: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub version: String,
    pub project_name: String,
    pub default_model_provider: String,
    pub model_providers: Vec<ModelProvider>,
    pub validation_commands: Vec<String>,
    pub data_policy: DataPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ModelProvider {
    pub id: String,
    #[serde(rename = "type")]
    pub provider_type: String,
    pub api_key_env: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DataPolicy {
    pub local_first: bool,
    pub upload_code_by_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectIndex {
    pub version: String,
    pub next_issue_number: u32,
    pub next_run_number: u32,
    pub issues: Vec<IssueIndexEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct IssueIndexEntry {
    pub id: String,
    pub title: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectContext {
    pub version: String,
    pub root: String,
    pub detected_stacks: Vec<String>,
    pub validation_commands: Vec<String>,
    pub files: Vec<ContextFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ContextFile {
    pub path: String,
    pub kind: String,
    pub bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct IssueContract {
    pub id: String,
    pub title: String,
    pub status: String,
    pub intent: String,
    #[serde(default)]
    pub risk_level: String,
    pub scope: Vec<String>,
    pub non_goals: Vec<String>,
    pub context: IssueContext,
    pub execution_plan: Vec<String>,
    pub validation: ValidationSpec,
    pub evidence_requirements: Vec<String>,
    #[serde(default)]
    pub rollback_plan: Vec<String>,
    pub human_gate: HumanGate,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_link: Option<IssueProjectLink>,
    #[serde(default)]
    pub aep: AepIssueProtocol,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct IssueProjectLink {
    pub team_id: String,
    pub project_id: String,
    pub milestone_id: String,
    pub link_source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct IssueProjectLinkPreview {
    pub version: String,
    pub initialized: bool,
    pub project_root: String,
    pub issue_id: String,
    pub issue_title: String,
    pub issue_json_path: String,
    pub issue_markdown_path: String,
    pub action: String,
    pub project_link: IssueProjectLink,
    pub confirmation_gates: Vec<String>,
    pub writes_required: bool,
    pub sources: Vec<String>,
    pub boundary: WorkbenchBoundary,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssueProjectLinkWriteSummary {
    pub preview: IssueProjectLinkPreview,
    pub written_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct IssueContext {
    pub repo: String,
    pub files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ValidationSpec {
    pub commands: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HumanGate {
    pub before_file_edits: bool,
    pub before_external_network: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct AepIssueProtocol {
    pub phase: String,
    pub stop_condition: String,
    pub fastest_feedback_loop: Vec<String>,
    pub vertical_slice: String,
    pub tracer_bullet_plan: Vec<String>,
    pub diagnose_plan: Vec<String>,
    pub graphify_context_status: String,
    pub docs_claim_trace: Vec<String>,
    pub boundary_confirmation: Vec<String>,
    pub pr_handoff_requirements: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitSummary {
    pub project_dir: PathBuf,
    pub goal_json: PathBuf,
    pub index_json: PathBuf,
    pub project_definition_json: PathBuf,
    pub scope_state_json: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanSummary {
    pub issue_id: String,
    pub issue_markdown: PathBuf,
    pub issue_json: PathBuf,
    pub project_link: Option<IssueProjectLink>,
    pub updated_project_seed_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextSummary {
    pub context_json: PathBuf,
    pub context_markdown: PathBuf,
    pub file_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AgentRun {
    pub id: String,
    pub issue_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub milestone_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lease_id: Option<String>,
    pub status: String,
    pub mode: String,
    #[serde(default)]
    pub run_plan: ControlledRunPlan,
    pub validation_commands: Vec<CommandRecord>,
    pub outputs: RunOutputs,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct ControlledRunPlan {
    pub goal: String,
    pub non_goals: Vec<String>,
    pub expected_files: Vec<String>,
    pub blocked_files: Vec<String>,
    pub planned_steps: Vec<String>,
    pub validation_commands: Vec<String>,
    pub evidence_requirements: Vec<String>,
    pub rollback_plan: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RunOutputs {
    pub transcript: String,
    pub commands: String,
    pub diff_summary: String,
    pub evidence: Option<String>,
    pub review: Option<String>,
    pub update: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CommandRecord {
    pub command: String,
    pub exit_code: i32,
    pub status: String,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunSummary {
    pub run_id: String,
    pub run_dir: PathBuf,
    pub run_json: PathBuf,
    pub run: AgentRun,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationSummary {
    pub issue_id: String,
    pub run_id: String,
    pub passed: bool,
    pub commands: Vec<CommandRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewSummary {
    pub issue_id: String,
    pub run_id: String,
    pub passed: bool,
    pub evidence_path: PathBuf,
    pub review_path: PathBuf,
    pub update_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SavedView {
    pub version: String,
    pub id: String,
    pub name: String,
    pub filter: SavedViewFilter,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct SavedViewFilter {
    pub issue_status: Option<String>,
    pub run_status: Option<String>,
    pub validation_status: Option<String>,
    pub issue_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct IndexedIssue {
    pub id: String,
    pub title: String,
    pub status: String,
    pub intent: String,
    pub json_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct IndexedRun {
    pub id: String,
    pub issue_id: String,
    pub status: String,
    pub mode: String,
    pub validation_status: String,
    pub json_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct IndexedUpdate {
    pub path: String,
    pub source_issue: String,
    pub source_run: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexSummary {
    pub sqlite_path: PathBuf,
    pub issue_count: usize,
    pub run_count: usize,
    pub update_count: usize,
    pub saved_view_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SavedViewSummary {
    pub view_id: String,
    pub view_path: PathBuf,
    pub sqlite_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewResult {
    pub view: SavedView,
    pub issues: Vec<IndexedIssue>,
    pub runs: Vec<IndexedRun>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectSummaryResult {
    pub summary_path: PathBuf,
    pub issue_count: usize,
    pub completed_issue_count: usize,
    pub run_count: usize,
    pub update_count: usize,
    pub saved_view_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewAssistantSummary {
    pub issue_id: String,
    pub assistant_path: PathBuf,
    pub ready: bool,
    pub checks: Vec<ReviewAssistantCheck>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewAssistantCheck {
    pub name: String,
    pub status: String,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoalBootstrapSummary {
    pub project_definition_json: PathBuf,
    pub scope_state_json: PathBuf,
    pub files_written: usize,
    pub files_checked: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoalReadinessSummary {
    pub objective: String,
    pub first_candidate: String,
    pub ready: bool,
    pub checks: Vec<GoalReadinessCheck>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoalReadinessCheck {
    pub name: String,
    pub path: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GoalLoopState {
    pub version: String,
    pub goal_ready: bool,
    pub active_issue_id: Option<String>,
    pub incomplete_issues: Vec<GoalLoopIssueRef>,
    pub next_action: String,
    pub recommended_issue_intent: String,
    pub recommended_command: String,
    pub rationale: Vec<String>,
    pub counts: GoalLoopCounts,
    pub sources: GoalLoopSources,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GoalLoopIssueRef {
    pub id: String,
    pub title: String,
    pub status: String,
    pub next_action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GoalLoopCounts {
    pub issues: usize,
    pub completed_issues: usize,
    pub runs: usize,
    pub evidence_reports: usize,
    pub reviews: usize,
    pub project_updates: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GoalLoopSources {
    pub goal: String,
    pub project_definition: String,
    pub scope_state: String,
    pub index: String,
    pub roadmap: String,
    pub project_summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoalLoopNextSummary {
    pub goal_loop_json: PathBuf,
    pub summary_path: PathBuf,
    pub goal_ready: bool,
    pub active_issue_id: Option<String>,
    pub next_action: String,
    pub recommended_issue_intent: String,
    pub recommended_command: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DesktopWorkbenchSnapshot {
    pub version: String,
    pub initialized: bool,
    pub project_root: String,
    pub project_summary_markdown: Option<String>,
    pub goal_loop_summary_markdown: Option<String>,
    pub goal_loop: Option<GoalLoopState>,
    pub issues: Vec<IssueContract>,
    pub runs: Vec<AgentRun>,
    pub saved_views: Vec<SavedView>,
    pub evidence: Vec<WorkbenchTextArtifact>,
    pub reviews: Vec<WorkbenchTextArtifact>,
    pub project_updates: Vec<WorkbenchTextArtifact>,
    pub counts: WorkbenchCounts,
    pub boundary: WorkbenchBoundary,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct WorkbenchCounts {
    pub issues: usize,
    pub completed_issues: usize,
    pub runs: usize,
    pub passed_runs: usize,
    pub evidence_reports: usize,
    pub reviews: usize,
    pub project_updates: usize,
    pub saved_views: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WorkbenchTextArtifact {
    pub path: String,
    pub title: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WorkbenchBoundary {
    pub read_only: bool,
    pub disallowed_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LocalMetricsSnapshot {
    pub version: String,
    pub initialized: bool,
    pub project_root: String,
    pub issues: LocalIssueMetrics,
    pub runs: LocalRunMetrics,
    pub artifacts: LocalArtifactMetrics,
    pub goal_ready: bool,
    pub active_issue_id: Option<String>,
    pub next_action: String,
    pub recommended_command: String,
    pub latest_run: Option<LocalMetricRunRef>,
    pub latest_evidence: Option<LocalMetricArtifactRef>,
    pub latest_review: Option<LocalMetricArtifactRef>,
    pub sources: Vec<String>,
    pub boundary: WorkbenchBoundary,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct LocalIssueMetrics {
    pub total: usize,
    pub completed: usize,
    pub planned: usize,
    pub active: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct LocalRunMetrics {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub missing_validation: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct LocalArtifactMetrics {
    pub evidence_reports: usize,
    pub reviews: usize,
    pub project_updates: usize,
    pub saved_views: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LocalMetricRunRef {
    pub id: String,
    pub issue_id: String,
    pub status: String,
    pub validation_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LocalMetricArtifactRef {
    pub path: String,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LocalSearchQuery {
    pub query: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LocalSearchResult {
    pub source_type: String,
    pub entity_kind: String,
    pub entity_id: Option<String>,
    pub path: String,
    pub title: String,
    pub field: String,
    pub line: usize,
    pub snippet: String,
    pub score: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LocalSearchSnapshot {
    pub version: String,
    pub initialized: bool,
    pub project_root: String,
    pub query: LocalSearchQuery,
    pub results: Vec<LocalSearchResult>,
    pub searched_paths: Vec<String>,
    pub excluded_paths: Vec<String>,
    pub boundary: WorkbenchBoundary,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LocalProjectModelSnapshot {
    pub version: String,
    pub initialized: bool,
    pub project_root: String,
    pub workspace: Option<LocalWorkspace>,
    pub teams: Vec<LocalTeam>,
    pub projects: Vec<LocalProject>,
    pub issue_refs: Vec<LocalProjectIssueRef>,
    pub goal_loop_selection: GoalLoopSelection,
    pub sources: Vec<String>,
    pub boundary: WorkbenchBoundary,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LocalProjectSeedPreview {
    pub version: String,
    pub initialized: bool,
    pub project_root: String,
    pub files: Vec<LocalProjectSeedFile>,
    pub confirmation_gates: Vec<String>,
    pub writes_required: bool,
    pub sources: Vec<String>,
    pub boundary: WorkbenchBoundary,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LocalProjectSeedFile {
    pub path: String,
    pub kind: String,
    pub action: String,
    pub content: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalProjectSeedWriteSummary {
    pub preview: LocalProjectSeedPreview,
    pub written_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProductFeatureDraft {
    pub feature_goal: String,
    pub team_id: String,
    pub project_title: String,
    pub non_goals: Vec<String>,
    pub success_criteria: Vec<String>,
    pub risk_level: String,
    pub scope_boundaries: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProductFeatureProject {
    pub id: String,
    pub title: String,
    pub team_id: String,
    pub active_milestone_id: String,
    pub milestone_ids: Vec<String>,
    pub issue_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProductFeatureMilestoneDraft {
    pub id: String,
    pub title: String,
    pub goal: String,
    pub status: String,
    pub issue_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProductFeatureIssueDraft {
    pub id: String,
    pub title: String,
    pub status: String,
    pub intent: String,
    pub milestone_id: String,
    pub scope: Vec<String>,
    pub non_goals: Vec<String>,
    pub validation_commands: Vec<String>,
    pub evidence_requirements: Vec<String>,
    pub rollback_plan: Vec<String>,
    pub risk_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProductFeatureCreationSnapshot {
    pub version: String,
    pub initialized: bool,
    pub project_root: String,
    pub mode: String,
    pub draft: ProductFeatureDraft,
    pub project: ProductFeatureProject,
    pub milestones: Vec<ProductFeatureMilestoneDraft>,
    pub issues: Vec<ProductFeatureIssueDraft>,
    pub writes_required: bool,
    pub written_paths: Vec<String>,
    pub recommended_command: String,
    pub sources: Vec<String>,
    pub boundary: WorkbenchBoundary,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductFeatureCreationSummary {
    pub snapshot: ProductFeatureCreationSnapshot,
    pub summary_path: PathBuf,
    pub written_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TeamDraft {
    pub name: String,
    pub team_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDraft {
    pub title: String,
    pub project_id: Option<String>,
    pub team_id: Option<String>,
    pub status: String,
    pub goal: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MilestoneDraft {
    pub title: String,
    pub milestone_id: Option<String>,
    pub project_id: Option<String>,
    pub description: Option<String>,
    pub target: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct IssueDraft {
    pub title: String,
    pub project_id: Option<String>,
    pub milestone_id: Option<String>,
    pub team_id: Option<String>,
    pub risk_level: String,
    pub scope: Vec<String>,
    pub non_goals: Vec<String>,
    pub validation_commands: Vec<String>,
    pub evidence_requirements: Vec<String>,
    pub rollback_plan: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CreationPreview {
    pub version: String,
    pub initialized: bool,
    pub project_root: String,
    pub mode: String,
    pub kind: String,
    pub entity_id: String,
    pub title: String,
    pub action: String,
    pub writes_required: bool,
    pub files: Vec<CreationPreviewFile>,
    pub v1_contract: CreationV1ContractPreview,
    pub confirmation_gates: Vec<String>,
    pub recommended_command: String,
    pub sources: Vec<String>,
    pub boundary: WorkbenchBoundary,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CreationPreviewFile {
    pub path: String,
    pub kind: String,
    pub action: String,
    pub content: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CreationV1ContractPreview {
    pub model: String,
    pub relation: String,
    pub team: Option<TeamCreationV1Preview>,
    pub project_charter: Option<ProjectCharterV1Preview>,
    pub milestone_gate: Option<MilestoneGateV1Preview>,
    pub issue_contract: Option<IssueContractV1Preview>,
    pub view_filter: Option<ViewFilterV1Preview>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TeamCreationV1Preview {
    pub team_id: String,
    pub name: String,
    pub project_ids: Vec<String>,
    pub issue_ids: Vec<String>,
    pub queue_rule: Vec<String>,
    pub boundary: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectCharterV1Preview {
    pub project_id: String,
    pub team_id: String,
    pub status: String,
    pub goal: String,
    pub scope: Vec<String>,
    pub non_goals: Vec<String>,
    pub success_criteria: Vec<String>,
    pub milestones: Vec<String>,
    pub issue_order: Vec<String>,
    pub validation_gate: Vec<String>,
    pub evidence_requirements: Vec<String>,
    pub queue_rule: Vec<String>,
    pub closure_gate: Vec<String>,
    pub boundary: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MilestoneGateV1Preview {
    pub project_id: String,
    pub milestone_id: String,
    pub goal: String,
    pub entry_criteria: Vec<String>,
    pub scope: Vec<String>,
    pub non_goals: Vec<String>,
    pub issues: Vec<String>,
    pub exit_criteria: Vec<String>,
    pub validation: Vec<String>,
    pub evidence_required: Vec<String>,
    pub next_milestone_gate: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct IssueContractV1Preview {
    pub issue_id: String,
    pub project_id: String,
    pub milestone_id: String,
    pub team_id: String,
    pub goal: String,
    pub scope: Vec<String>,
    pub non_goals: Vec<String>,
    pub dependencies: Vec<String>,
    pub codex_instructions: Vec<String>,
    pub acceptance_criteria: Vec<String>,
    pub validation_commands: Vec<String>,
    pub evidence_required: Vec<String>,
    pub allowed_files: Vec<String>,
    pub forbidden_files: Vec<String>,
    pub boundary: Vec<String>,
    pub initial_state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ViewFilterV1Preview {
    pub entity: String,
    pub filters: Vec<String>,
    pub sort: Vec<String>,
    pub layout: String,
    pub boundary: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreationWriteSummary {
    pub preview: CreationPreview,
    pub written_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProductFeatureExecutionSnapshot {
    pub version: String,
    pub initialized: bool,
    pub project_root: String,
    pub feature_ready: bool,
    pub project_id: String,
    pub project_title: String,
    pub project_status: String,
    pub project_canonical_status: String,
    pub project_goal: String,
    pub active_milestone_id: String,
    pub milestones: Vec<ProductFeatureExecutionMilestone>,
    pub current_issue: Option<ProductFeatureExecutionIssue>,
    pub issues: Vec<ProductFeatureExecutionIssue>,
    pub next_action: String,
    pub recommended_command: String,
    pub rationale: Vec<String>,
    pub sources: Vec<String>,
    pub boundary: WorkbenchBoundary,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProductFeatureExecutionMilestone {
    pub id: String,
    pub title: String,
    pub status: String,
    pub progress: MilestoneDerivedProgress,
    pub issue_ids: Vec<String>,
    pub completed_issue_ids: Vec<String>,
    pub next_issue_intent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProductFeatureExecutionIssue {
    pub id: String,
    pub title: String,
    pub status: String,
    pub canonical_status: String,
    pub milestone_id: Option<String>,
    pub ready: bool,
    pub eligible: bool,
    pub leased: bool,
    pub failure_reasons: Vec<String>,
    pub next_action: String,
    pub recommended_command: String,
    pub dry_run_recorded: bool,
    pub latest_run_plan: Vec<String>,
    pub expected_files: Vec<String>,
    pub blocked_files: Vec<String>,
    pub validation_commands: Vec<String>,
    pub evidence_requirements: Vec<String>,
    pub latest_run_id: Option<String>,
    pub latest_run_status: Option<String>,
    pub validation_status: String,
    pub execution_state: String,
    pub evidence_path: Option<String>,
    pub review_path: Option<String>,
    pub project_update_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LocalWorkspace {
    pub version: String,
    pub id: String,
    pub name: String,
    pub default_team_id: String,
    pub active_project_id: String,
    pub team_ids: Vec<String>,
    pub project_ids: Vec<String>,
    pub issue_count: usize,
    pub completed_issue_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LocalTeam {
    pub version: String,
    pub id: String,
    pub name: String,
    pub workflow: Vec<String>,
    pub default_validation_commands: Vec<String>,
    pub wip_limit: u32,
    pub issue_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LocalProject {
    pub version: String,
    pub id: String,
    pub name: String,
    pub status: String,
    pub canonical_status: String,
    pub goal: String,
    pub team_ids: Vec<String>,
    pub active_milestone_id: String,
    pub milestones: Vec<LocalMilestone>,
    pub issue_ids: Vec<String>,
    pub issue_count: usize,
    pub completed_issue_count: usize,
    pub next_issue_intent: Option<String>,
    pub recommended_command: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LocalMilestone {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub sort_order: usize,
    pub target: Option<String>,
    pub status: String,
    pub progress: MilestoneDerivedProgress,
    pub issue_ids: Vec<String>,
    pub completed_issue_ids: Vec<String>,
    pub next_issue_intent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MilestoneDerivedProgress {
    pub done_issue_count: usize,
    pub total_issue_count: usize,
    pub non_canceled_issue_count: usize,
    pub canceled_issue_count: usize,
    pub percent: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LocalProjectIssueRef {
    pub id: String,
    pub title: String,
    pub status: String,
    pub canonical_status: String,
    pub next_action: String,
    pub latest_run_id: Option<String>,
    pub latest_run_status: Option<String>,
    pub validation_status: String,
    pub execution_state: String,
    pub evidence_path: Option<String>,
    pub review_path: Option<String>,
    pub project_update_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GoalLoopSelection {
    pub active_project_id: Option<String>,
    pub source: String,
    pub next_action: String,
    pub next_issue_intent: Option<String>,
    pub recommended_command: String,
    pub rationale: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectMilestoneIssueViewModelSnapshot {
    pub version: String,
    pub initialized: bool,
    pub project_root: String,
    pub workspace: Option<V1WorkspaceRef>,
    pub teams: Vec<V1TeamRef>,
    pub projects: Vec<V1Project>,
    pub issues: Vec<V1Issue>,
    pub views: Vec<V1View>,
    pub invariants: Vec<String>,
    pub sources: Vec<String>,
    pub boundary: WorkbenchBoundary,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct V1WorkspaceRef {
    pub id: String,
    pub name: String,
    pub active_project_id: String,
    pub team_ids: Vec<String>,
    pub project_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct V1TeamRef {
    pub id: String,
    pub name: String,
    pub project_ids: Vec<String>,
    pub issue_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct V1Project {
    pub id: String,
    pub name: String,
    pub status: String,
    pub raw_status: String,
    pub goal: String,
    pub target_maturity: Option<String>,
    pub target_layers: Vec<String>,
    pub scope: Vec<String>,
    pub non_goals: Vec<String>,
    pub success_criteria: Vec<String>,
    pub milestones: Vec<V1Milestone>,
    pub issue_order: Vec<String>,
    pub validation_gate: Vec<String>,
    pub evidence_required: Vec<String>,
    pub queue_rule: Vec<String>,
    pub closure_gate: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct V1Milestone {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub status: String,
    pub raw_status: String,
    pub goal: String,
    pub entry_criteria: Vec<String>,
    pub scope: Vec<String>,
    pub non_goals: Vec<String>,
    pub issue_ids: Vec<String>,
    pub exit_criteria: Vec<String>,
    pub validation: Vec<String>,
    pub evidence_required: Vec<String>,
    pub next_milestone_gate: String,
    pub progress: MilestoneDerivedProgress,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct V1Issue {
    pub id: String,
    pub project_id: Option<String>,
    pub milestone_id: Option<String>,
    pub title: String,
    pub status: String,
    pub raw_status: String,
    pub goal: String,
    pub scope: Vec<String>,
    pub non_goals: Vec<String>,
    pub dependencies: Vec<String>,
    pub codex_instructions: Vec<String>,
    pub acceptance_criteria: Vec<String>,
    pub validation_commands: Vec<String>,
    pub evidence_required: Vec<String>,
    pub allowed_files: Vec<String>,
    pub forbidden_files: Vec<String>,
    pub boundary: Vec<String>,
    pub risk_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct V1View {
    pub id: String,
    pub name: String,
    pub entity: String,
    pub filter: V1ViewFilter,
    pub sort: Vec<V1ViewSort>,
    pub layout: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct V1ViewFilter {
    pub issue_status: Option<String>,
    pub run_status: Option<String>,
    pub validation_status: Option<String>,
    pub issue_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct V1ViewSort {
    pub field: String,
    pub direction: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GoalLoopDecision {
    next_action: String,
    recommended_issue_intent: String,
    recommended_command: String,
    rationale: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LocalSearchDocument {
    path: String,
    content: String,
    entity_kind: String,
    entity_id: Option<String>,
    title: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CodeAuditSourceDocument {
    path: String,
    content: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CodeAuditLineHit {
    path: String,
    line: usize,
    snippet: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DocsRefreshDocumentSpec {
    path: &'static str,
    category: &'static str,
    required: bool,
    anchors: &'static [&'static str],
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PlanProjectSeedUpdate {
    project_link: IssueProjectLink,
    team_path: PathBuf,
    team_seed: serde_json::Value,
    project_path: PathBuf,
    project_seed: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ActiveMilestoneQueue {
    project_id: String,
    milestone_id: String,
    issue_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowStateSnapshot {
    pub version: String,
    pub initialized: bool,
    pub project_root: String,
    pub ready: bool,
    pub counts: WorkflowStateCounts,
    pub checks: Vec<WorkflowStateCheck>,
    pub transition_guards: Vec<WorkflowTransitionGuard>,
    pub sources: Vec<String>,
    pub boundary: WorkbenchBoundary,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowStateCounts {
    pub projects: usize,
    pub milestones: usize,
    pub issues: usize,
    pub errors: usize,
    pub warnings: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowStateCheck {
    pub id: String,
    pub entity_kind: String,
    pub entity_id: String,
    pub severity: String,
    pub status: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowTransitionGuard {
    pub entity_kind: String,
    pub from: String,
    pub to: String,
    pub allowed: bool,
    pub guard: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowStateCheckSummary {
    pub snapshot: WorkflowStateSnapshot,
    pub snapshot_path: PathBuf,
    pub summary_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowEligibilitySnapshot {
    pub version: String,
    pub initialized: bool,
    pub project_root: String,
    pub active_project_id: Option<String>,
    pub active_milestone_id: Option<String>,
    pub eligible_issue_id: Option<String>,
    pub candidates: Vec<WorkflowEligibilityCandidate>,
    pub summary: WorkflowEligibilitySummary,
    pub sources: Vec<String>,
    pub boundary: WorkbenchBoundary,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowEligibilityCandidate {
    pub issue_id: String,
    pub title: String,
    pub issue_status: String,
    pub project_id: Option<String>,
    pub milestone_id: Option<String>,
    pub ready: bool,
    pub eligible: bool,
    pub leased: bool,
    pub code_changing: bool,
    pub active_lease_id: Option<String>,
    pub failure_reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowEligibilitySummary {
    pub ready_issue_count: usize,
    pub eligible_issue_count: usize,
    pub blocked_issue_count: usize,
    pub next_action: String,
    pub recommended_command: String,
    pub rationale: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowEligibilityCheckSummary {
    pub snapshot: WorkflowEligibilitySnapshot,
    pub snapshot_path: PathBuf,
    pub summary_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowLeaseRecord {
    pub version: String,
    pub id: String,
    pub issue_id: String,
    pub project_id: Option<String>,
    pub milestone_id: Option<String>,
    pub owner_agent_id: String,
    pub status: String,
    pub leased_at_epoch_seconds: u64,
    pub expires_at_epoch_seconds: u64,
    pub released_at_epoch_seconds: Option<u64>,
    pub stale_recovery_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowLeaseSnapshot {
    pub version: String,
    pub initialized: bool,
    pub project_root: String,
    pub active_leases: Vec<WorkflowLeaseRecord>,
    pub stale_leases: Vec<WorkflowLeaseRecord>,
    pub recommended_command: String,
    pub boundary: WorkbenchBoundary,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowLeaseSummary {
    pub snapshot: WorkflowLeaseSnapshot,
    pub snapshot_path: PathBuf,
    pub summary_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectClosureStateSnapshot {
    pub version: String,
    pub initialized: bool,
    pub project_root: String,
    pub active_project_id: Option<String>,
    pub project_status: Option<String>,
    pub active_milestone_id: Option<String>,
    pub closure_state: String,
    pub can_mark_done: bool,
    pub counts: ProjectClosureCounts,
    pub gates: Vec<ProjectClosureGate>,
    pub completed_milestone_ids: Vec<String>,
    pub incomplete_milestone_ids: Vec<String>,
    pub missing_milestone_summary_ids: Vec<String>,
    pub done_blocked_reasons: Vec<String>,
    pub recommended_command: String,
    pub sources: Vec<String>,
    pub boundary: WorkbenchBoundary,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectClosureCounts {
    pub milestones: usize,
    pub completed_milestones: usize,
    pub issues: usize,
    pub completed_issues: usize,
    pub runs: usize,
    pub evidence_reports: usize,
    pub reviews: usize,
    pub project_updates: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectClosureGate {
    pub id: String,
    pub name: String,
    pub status: String,
    pub required: bool,
    pub path: Option<String>,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectClosureStateSummary {
    pub snapshot: ProjectClosureStateSnapshot,
    pub snapshot_path: PathBuf,
    pub summary_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectCodeAuditSnapshot {
    pub version: String,
    pub initialized: bool,
    pub project_root: String,
    pub active_project_id: Option<String>,
    pub active_milestone_id: Option<String>,
    pub closure_state: String,
    pub audit_state: String,
    pub counts: ProjectCodeAuditCounts,
    pub checks: Vec<ProjectCodeAuditCheck>,
    pub blockers: Vec<String>,
    pub recommended_command: String,
    pub sources: Vec<String>,
    pub boundary: WorkbenchBoundary,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectCodeAuditCounts {
    pub projects: usize,
    pub milestones: usize,
    pub issues: usize,
    pub completed_issues: usize,
    pub runs: usize,
    pub evidence_reports: usize,
    pub reviews: usize,
    pub project_updates: usize,
    pub source_files: usize,
    pub findings: usize,
    pub blockers: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectCodeAuditCheck {
    pub id: String,
    pub name: String,
    pub status: String,
    pub candidate_count: usize,
    pub detail: String,
    pub findings: Vec<ProjectCodeAuditFinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectCodeAuditFinding {
    pub id: String,
    pub path: Option<String>,
    pub line: Option<usize>,
    pub severity: String,
    pub snippet: String,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectCodeAuditSummary {
    pub snapshot: ProjectCodeAuditSnapshot,
    pub snapshot_path: PathBuf,
    pub summary_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDocsRefreshSnapshot {
    pub version: String,
    pub initialized: bool,
    pub project_root: String,
    pub active_project_id: Option<String>,
    pub active_milestone_id: Option<String>,
    pub closure_state: String,
    pub docs_refresh_state: String,
    pub counts: ProjectDocsRefreshCounts,
    pub checked_docs: Vec<ProjectDocsRefreshCheckedDoc>,
    pub required_updates: Vec<ProjectDocsRefreshRequiredUpdate>,
    pub blockers: Vec<String>,
    pub recommended_command: String,
    pub sources: Vec<String>,
    pub boundary: WorkbenchBoundary,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDocsRefreshCounts {
    pub checked_docs: usize,
    pub current_docs: usize,
    pub update_needed_docs: usize,
    pub missing_docs: usize,
    pub intentionally_absent_docs: usize,
    pub required_updates: usize,
    pub blockers: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDocsRefreshCheckedDoc {
    pub path: String,
    pub category: String,
    pub status: String,
    pub reason: String,
    pub anchors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDocsRefreshRequiredUpdate {
    pub path: String,
    pub summary: String,
    pub follow_up_issue_intent: String,
    pub severity: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectDocsRefreshSummary {
    pub snapshot: ProjectDocsRefreshSnapshot,
    pub snapshot_path: PathBuf,
    pub summary_path: PathBuf,
}

pub fn compile_goal_from_markdown(markdown: &str) -> ProjectGoal {
    let objective = section_body(markdown, "Objective")
        .and_then(first_paragraph)
        .unwrap_or_else(|| "Build the local AgentFlow execution spine.".to_string());
    let success_criteria = section_body(markdown, "Success")
        .map(bullets)
        .filter(|items| !items.is_empty())
        .unwrap_or_else(|| vec!["Initialize a local project from GOAL.md.".to_string()]);
    let non_goals = section_body(markdown, "Out Of Scope")
        .map(bullets)
        .filter(|items| !items.is_empty())
        .unwrap_or_else(|| vec!["Do not upload source code by default.".to_string()]);

    ProjectGoal {
        version: VERSION.to_string(),
        objective,
        success_criteria,
        non_goals,
        constraints: vec!["local-first".to_string(), "no-code-upload".to_string()],
        first_candidate: FIRST_CANDIDATE.to_string(),
    }
}

pub fn init_from_goal(repo: &Path, goal_path: &Path, force: bool) -> Result<InitSummary> {
    let goal_markdown = fs::read_to_string(goal_path)
        .with_context(|| format!("read goal file {}", goal_path.display()))?;
    let goal = compile_goal_from_markdown(&goal_markdown);
    let project_dir = repo.join(AGENTFLOW_DIR);

    ensure_dir(&project_dir)?;
    for dir in [
        "issues",
        "runs",
        "evidence",
        "reviews",
        "views",
        "updates",
        "tmp",
        "bootstrap",
    ] {
        ensure_dir(&project_dir.join(dir))?;
    }

    write_new(&project_dir.join("goal.md"), &goal_markdown, force)?;
    write_json(&project_dir.join("goal.json"), &goal, force)?;
    write_new(
        &project_dir.join("environment.md"),
        &environment_markdown(),
        force,
    )?;
    write_new(
        &project_dir.join("architecture.md"),
        &architecture_markdown(),
        force,
    )?;
    write_new(&project_dir.join("roadmap.md"), &roadmap_markdown(), force)?;
    write_json(
        &project_dir.join("settings.json"),
        &default_settings(repo),
        force,
    )?;
    write_json(&project_dir.join("index.json"), &default_index(), force)?;
    write_new(
        &project_dir.join("evidence/FLOW-0-initialization.md"),
        &initialization_evidence(goal_path, &goal),
        force,
    )?;
    let bootstrap = write_goal_bootstrap_artifacts(&project_dir, goal_path, &goal, force)?;

    Ok(InitSummary {
        project_dir: project_dir.clone(),
        goal_json: project_dir.join("goal.json"),
        index_json: project_dir.join("index.json"),
        project_definition_json: bootstrap.project_definition_json,
        scope_state_json: bootstrap.scope_state_json,
    })
}

pub fn bootstrap_goal_protocol(repo: &Path, force: bool) -> Result<GoalBootstrapSummary> {
    let project_dir = required_project_dir(repo)?;
    let goal_path = project_dir.join("goal.md");
    let goal: ProjectGoal = read_json(&project_dir.join("goal.json"))?;
    write_goal_bootstrap_artifacts(&project_dir, &goal_path, &goal, force)
}

pub fn check_goal_readiness(repo: &Path) -> Result<GoalReadinessSummary> {
    let project_dir = required_project_dir(repo)?;
    let goal: ProjectGoal = read_json(&project_dir.join("goal.json"))?;
    let mut checks = Vec::new();
    for (name, path) in goal_readiness_paths() {
        let full_path = project_dir.join(&path);
        checks.push(GoalReadinessCheck {
            name,
            path,
            status: if full_path.exists() {
                "pass"
            } else {
                "missing"
            }
            .to_string(),
        });
    }
    let ready = checks.iter().all(|check| check.status == "pass");
    Ok(GoalReadinessSummary {
        objective: goal.objective,
        first_candidate: goal.first_candidate,
        ready,
        checks,
    })
}

pub fn write_goal_next(repo: &Path) -> Result<GoalLoopNextSummary> {
    let project_dir = required_project_dir(repo)?;
    let readiness = check_goal_readiness(repo)?;
    let scope_state = read_scope_state_or_default(&project_dir.join("scope-state.json"))?;
    let project_definition_path = project_dir.join("project-definition.json");
    if project_definition_path.exists() {
        let _project_definition: ProjectDefinition = read_json(&project_definition_path)?;
    }
    let _index: ProjectIndex = read_json(&project_dir.join("index.json"))?;
    rebuild_index(repo)?;

    let issues = load_indexed_issues(repo, &project_dir)?;
    let runs = load_indexed_runs(repo, &project_dir)?;
    let updates = load_indexed_updates(repo, &project_dir)?;
    let evidence_reports = markdown_files(&project_dir.join("evidence"))?;
    let reviews = markdown_files(&project_dir.join("reviews"))?;
    let incomplete_issues = goal_loop_incomplete_issues(&project_dir, &issues)?;
    let decision = goal_loop_decision(repo, &project_dir, &readiness, &scope_state, &issues)?;
    let state = GoalLoopState {
        version: VERSION.to_string(),
        goal_ready: readiness.ready,
        active_issue_id: scope_state.active_issue_id.clone(),
        incomplete_issues,
        next_action: decision.next_action.clone(),
        recommended_issue_intent: decision.recommended_issue_intent.clone(),
        recommended_command: decision.recommended_command.clone(),
        rationale: decision.rationale,
        counts: GoalLoopCounts {
            issues: issues.len(),
            completed_issues: issues
                .iter()
                .filter(|issue| issue_state_done(&issue.status))
                .count(),
            runs: runs.len(),
            evidence_reports: evidence_reports.len(),
            reviews: reviews.len(),
            project_updates: updates.len(),
        },
        sources: GoalLoopSources {
            goal: ".agentflow/goal.json".to_string(),
            project_definition: ".agentflow/project-definition.json".to_string(),
            scope_state: ".agentflow/scope-state.json".to_string(),
            index: ".agentflow/index.json".to_string(),
            roadmap: ".agentflow/roadmap.md".to_string(),
            project_summary: ".agentflow/updates/PROJECT-SUMMARY.md".to_string(),
        },
    };
    let goal_loop_json = project_dir.join("goal-loop.json");
    let summary_path = project_dir.join("updates/GOAL-LOOP-SUMMARY.md");
    write_json(&goal_loop_json, &state, true)?;
    write_new(&summary_path, &goal_loop_summary_markdown(&state), true)?;

    Ok(GoalLoopNextSummary {
        goal_loop_json,
        summary_path,
        goal_ready: state.goal_ready,
        active_issue_id: state.active_issue_id,
        next_action: state.next_action,
        recommended_issue_intent: state.recommended_issue_intent,
        recommended_command: state.recommended_command,
    })
}

pub fn read_desktop_workbench_snapshot(start: &Path) -> Result<DesktopWorkbenchSnapshot> {
    let project_root = discover_project_root(start)?;
    let Some(project_dir) = project_root
        .as_ref()
        .map(|root| root.join(AGENTFLOW_DIR))
        .filter(|path| path.exists())
    else {
        return Ok(empty_desktop_snapshot(start));
    };
    let root = project_root.expect("project root is set when project_dir exists");
    let issues = read_issue_contracts(&project_dir)?;
    let runs = read_agent_runs(&project_dir)?;
    let saved_views = load_saved_views(&project_dir)?;
    let evidence = read_text_artifacts(&root, &project_dir.join("evidence"))?;
    let reviews = read_text_artifacts(&root, &project_dir.join("reviews"))?;
    let project_updates = read_project_update_artifacts(&root, &project_dir.join("updates"))?;
    let goal_loop = read_optional_json(&project_dir.join("goal-loop.json"))?;
    let project_summary_markdown =
        read_optional_string(&project_dir.join("updates/PROJECT-SUMMARY.md"))?;
    let goal_loop_summary_markdown =
        read_optional_string(&project_dir.join("updates/GOAL-LOOP-SUMMARY.md"))?;
    let completed_issues = issues
        .iter()
        .filter(|issue| issue_state_done(&issue.status))
        .count();
    let passed_runs = runs
        .iter()
        .filter(|run| {
            !run.validation_commands.is_empty()
                && run
                    .validation_commands
                    .iter()
                    .all(|record| record.exit_code == 0)
        })
        .count();

    Ok(DesktopWorkbenchSnapshot {
        version: VERSION.to_string(),
        initialized: true,
        project_root: root.display().to_string(),
        project_summary_markdown,
        goal_loop_summary_markdown,
        goal_loop,
        counts: WorkbenchCounts {
            issues: issues.len(),
            completed_issues,
            runs: runs.len(),
            passed_runs,
            evidence_reports: evidence.len(),
            reviews: reviews.len(),
            project_updates: project_updates.len(),
            saved_views: saved_views.len(),
        },
        issues,
        runs,
        saved_views,
        evidence,
        reviews,
        project_updates,
        boundary: workbench_boundary(),
    })
}

pub fn read_local_metrics_snapshot(start: &Path) -> Result<LocalMetricsSnapshot> {
    let snapshot = read_desktop_workbench_snapshot(start)?;
    if !snapshot.initialized {
        return Ok(empty_local_metrics_snapshot(&snapshot));
    }

    let root = PathBuf::from(&snapshot.project_root);
    let project_updates = project_update_markdown_count(&root.join(AGENTFLOW_DIR))?;
    let active_issue_id = snapshot
        .goal_loop
        .as_ref()
        .and_then(|state| state.active_issue_id.clone());
    let goal_ready = snapshot
        .goal_loop
        .as_ref()
        .map(|state| state.goal_ready)
        .unwrap_or(false);
    let next_action = snapshot
        .goal_loop
        .as_ref()
        .map(|state| state.next_action.clone())
        .unwrap_or_else(|| "wait-human".to_string());
    let recommended_command = snapshot
        .goal_loop
        .as_ref()
        .map(|state| state.recommended_command.clone())
        .unwrap_or_else(|| "agentflow goal next".to_string());

    Ok(LocalMetricsSnapshot {
        version: VERSION.to_string(),
        initialized: true,
        project_root: snapshot.project_root.clone(),
        issues: LocalIssueMetrics {
            total: snapshot.issues.len(),
            completed: snapshot
                .issues
                .iter()
                .filter(|issue| issue_state_done(&issue.status))
                .count(),
            planned: snapshot
                .issues
                .iter()
                .filter(|issue| canonical_issue_status(&issue.status) == IssueStatus::Todo)
                .count(),
            active: usize::from(active_issue_id.is_some()),
        },
        runs: LocalRunMetrics {
            total: snapshot.runs.len(),
            passed: snapshot
                .runs
                .iter()
                .filter(|run| validation_status(&run.validation_commands) == "passed")
                .count(),
            failed: snapshot
                .runs
                .iter()
                .filter(|run| validation_status(&run.validation_commands) == "failed")
                .count(),
            missing_validation: snapshot
                .runs
                .iter()
                .filter(|run| run.validation_commands.is_empty())
                .count(),
        },
        artifacts: LocalArtifactMetrics {
            evidence_reports: snapshot.evidence.len(),
            reviews: snapshot.reviews.len(),
            project_updates,
            saved_views: snapshot.saved_views.len(),
        },
        goal_ready,
        active_issue_id,
        next_action,
        recommended_command,
        latest_run: snapshot.runs.last().map(local_metric_run_ref),
        latest_evidence: snapshot.evidence.last().map(local_metric_artifact_ref),
        latest_review: snapshot.reviews.last().map(local_metric_artifact_ref),
        sources: vec![
            ".agentflow/goal-loop.json".to_string(),
            ".agentflow/issues/*.json".to_string(),
            ".agentflow/runs/*/run.json".to_string(),
            ".agentflow/evidence/*.md".to_string(),
            ".agentflow/reviews/*.md".to_string(),
            ".agentflow/updates/PROJECT-UPDATE-*.md".to_string(),
            ".agentflow/views/*.json".to_string(),
        ],
        boundary: workbench_boundary(),
    })
}

pub fn read_local_search_snapshot(start: &Path, query: &str) -> Result<LocalSearchSnapshot> {
    let trimmed_query = query.trim();
    if trimmed_query.is_empty() {
        bail!("search query is required");
    }

    let Some(root) = discover_project_root(start)? else {
        return Ok(empty_local_search_snapshot(start, trimmed_query));
    };
    let project_dir = root.join(AGENTFLOW_DIR);
    if !project_dir.exists() {
        return Ok(empty_local_search_snapshot(start, trimmed_query));
    }

    let documents = local_search_documents(&root, &project_dir)?;
    let mut results = Vec::new();
    for document in &documents {
        results.extend(search_document(document, trimmed_query));
    }
    results.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.path.cmp(&right.path))
            .then_with(|| left.line.cmp(&right.line))
    });

    Ok(LocalSearchSnapshot {
        version: VERSION.to_string(),
        initialized: true,
        project_root: root.display().to_string(),
        query: LocalSearchQuery {
            query: trimmed_query.to_string(),
        },
        searched_paths: documents
            .iter()
            .map(|document| document.path.clone())
            .collect(),
        results,
        excluded_paths: local_search_excluded_paths(),
        boundary: workbench_boundary(),
    })
}

pub fn read_local_project_model_snapshot(start: &Path) -> Result<LocalProjectModelSnapshot> {
    let Some(root) = discover_project_root(start)? else {
        return Ok(empty_local_project_model_snapshot(start));
    };
    let project_dir = root.join(AGENTFLOW_DIR);
    if !project_dir.exists() {
        return Ok(empty_local_project_model_snapshot(start));
    }

    let goal: ProjectGoal = read_json(&project_dir.join("goal.json"))?;
    let settings: Settings = read_json(&project_dir.join("settings.json"))?;
    let scope_state = read_scope_state_or_default(&project_dir.join("scope-state.json"))?;
    if project_dir.join("project-definition.json").exists() {
        let _project_definition: ProjectDefinition =
            read_json(&project_dir.join("project-definition.json"))?;
    }
    let readiness = check_goal_readiness(&root)?;
    let issues = read_issue_contracts(&project_dir)?;
    let indexed_issues = load_indexed_issues(&root, &project_dir)?;
    let issue_refs = local_project_issue_refs(&project_dir, &issues)?;
    let issue_ids = issue_refs
        .iter()
        .map(|issue| issue.id.clone())
        .collect::<Vec<_>>();
    let completed_issue_ids = issue_refs
        .iter()
        .filter(|issue| canonical_issue_status(&issue.status) == IssueStatus::Done)
        .map(|issue| issue.id.clone())
        .collect::<Vec<_>>();
    let team_id = "core".to_string();
    let project_id = "agentflow-local-execution".to_string();
    let milestone_id = "current-roadmap".to_string();
    let decision = goal_loop_decision(
        &root,
        &project_dir,
        &readiness,
        &scope_state,
        &indexed_issues,
    )?;
    let selection = goal_loop_selection(&project_id, decision);
    let next_issue_intent = selection.next_issue_intent.clone();

    let derived_workspace = LocalWorkspace {
        version: VERSION.to_string(),
        id: "default".to_string(),
        name: settings.project_name.clone(),
        default_team_id: team_id.clone(),
        active_project_id: project_id.clone(),
        team_ids: vec![team_id.clone()],
        project_ids: vec![project_id.clone()],
        issue_count: issue_refs.len(),
        completed_issue_count: completed_issue_ids.len(),
    };
    let derived_team = LocalTeam {
        version: VERSION.to_string(),
        id: team_id.clone(),
        name: "Core".to_string(),
        workflow: vec![
            "backlog".to_string(),
            "todo".to_string(),
            "in_progress".to_string(),
            "in_review".to_string(),
            "done".to_string(),
            "canceled".to_string(),
        ],
        default_validation_commands: settings.validation_commands,
        wip_limit: scope_state.wip_limit,
        issue_ids: issue_ids.clone(),
    };
    let derived_milestone = LocalMilestone {
        id: milestone_id.clone(),
        name: "Current Roadmap".to_string(),
        description: Some(
            "Derived from the current local roadmap and issue contracts.".to_string(),
        ),
        sort_order: 0,
        target: next_issue_intent.clone(),
        status: if scope_state.active_issue_id.is_some() {
            "active".to_string()
        } else {
            "planned".to_string()
        },
        progress: milestone_derived_progress(&issue_ids, &issue_refs),
        issue_ids: issue_ids.clone(),
        completed_issue_ids: completed_issue_ids.clone(),
        next_issue_intent: next_issue_intent.clone(),
    };
    let derived_project = LocalProject {
        version: VERSION.to_string(),
        id: project_id.clone(),
        name: settings.project_name,
        status: "active".to_string(),
        canonical_status: canonical_project_status_string("active"),
        goal: goal.objective,
        team_ids: vec![team_id],
        active_milestone_id: milestone_id,
        milestones: vec![derived_milestone],
        issue_count: issue_refs.len(),
        completed_issue_count: completed_issue_ids.len(),
        issue_ids,
        next_issue_intent,
        recommended_command: Some(selection.recommended_command.clone()),
    };
    let (workspace, teams, projects) = local_project_seed_snapshot(
        &project_dir,
        derived_workspace,
        derived_team,
        derived_project,
        &issue_refs,
        &selection,
    )?;

    Ok(LocalProjectModelSnapshot {
        version: VERSION.to_string(),
        initialized: true,
        project_root: root.display().to_string(),
        workspace: Some(workspace),
        teams,
        projects,
        issue_refs,
        goal_loop_selection: selection,
        sources: local_project_model_sources(),
        boundary: workbench_boundary(),
    })
}

pub fn read_project_milestone_issue_view_model_snapshot(
    start: &Path,
) -> Result<ProjectMilestoneIssueViewModelSnapshot> {
    let snapshot = read_local_project_model_snapshot(start)?;
    if !snapshot.initialized {
        return Ok(empty_project_milestone_issue_view_model_snapshot(&snapshot));
    }

    let root = PathBuf::from(&snapshot.project_root);
    let project_dir = root.join(AGENTFLOW_DIR);
    let issues = if project_dir.exists() {
        read_issue_contracts(&project_dir)?
    } else {
        Vec::new()
    };
    let views = if project_dir.exists() {
        load_saved_views(&project_dir)?
    } else {
        Vec::new()
    };

    Ok(project_milestone_issue_view_model_from_snapshot(
        &snapshot, &issues, &views,
    ))
}

fn read_workflow_control_project_snapshot(
    root: &Path,
    project_dir: &Path,
) -> Result<LocalProjectModelSnapshot> {
    let goal: ProjectGoal = read_json(&project_dir.join("goal.json"))?;
    let settings: Settings = read_json(&project_dir.join("settings.json"))?;
    let scope_state = read_scope_state_or_default(&project_dir.join("scope-state.json"))?;
    let issues = read_issue_contracts(project_dir)?;
    let issue_refs = local_project_issue_refs(project_dir, &issues)?;
    let issue_ids = issue_refs
        .iter()
        .map(|issue| issue.id.clone())
        .collect::<Vec<_>>();
    let completed_issue_ids = issue_refs
        .iter()
        .filter(|issue| canonical_issue_status(&issue.status) == IssueStatus::Done)
        .map(|issue| issue.id.clone())
        .collect::<Vec<_>>();
    let team_id = "core".to_string();
    let project_id = "agentflow-local-execution".to_string();
    let milestone_id = "current-roadmap".to_string();
    let fallback_intent = current_candidate_intent(root, project_dir).ok();
    let selection = GoalLoopSelection {
        active_project_id: Some(project_id.clone()),
        source: "workflow-control-core".to_string(),
        next_action: fallback_intent
            .as_ref()
            .map(|_| "plan".to_string())
            .unwrap_or_else(|| "wait-human".to_string()),
        next_issue_intent: fallback_intent.clone(),
        recommended_command: fallback_intent
            .as_ref()
            .map(|intent| format!("agentflow plan \"{intent}\""))
            .unwrap_or_else(|| "agentflow projects".to_string()),
        rationale: vec![
            "Workflow control snapshot avoids GoalLoop recursion.".to_string(),
            "Eligibility must be computed before GoalLoop recommends run.".to_string(),
        ],
    };

    let derived_workspace = LocalWorkspace {
        version: VERSION.to_string(),
        id: "default".to_string(),
        name: settings.project_name.clone(),
        default_team_id: team_id.clone(),
        active_project_id: project_id.clone(),
        team_ids: vec![team_id.clone()],
        project_ids: vec![project_id.clone()],
        issue_count: issue_refs.len(),
        completed_issue_count: completed_issue_ids.len(),
    };
    let derived_team = LocalTeam {
        version: VERSION.to_string(),
        id: team_id.clone(),
        name: "Core".to_string(),
        workflow: vec![
            "backlog".to_string(),
            "todo".to_string(),
            "in_progress".to_string(),
            "in_review".to_string(),
            "done".to_string(),
            "canceled".to_string(),
        ],
        default_validation_commands: settings.validation_commands,
        wip_limit: scope_state.wip_limit,
        issue_ids: issue_ids.clone(),
    };
    let derived_milestone = LocalMilestone {
        id: milestone_id.clone(),
        name: "Current Roadmap".to_string(),
        description: Some(
            "Derived from the current local roadmap and issue contracts.".to_string(),
        ),
        sort_order: 0,
        target: fallback_intent.clone(),
        status: if scope_state.active_issue_id.is_some() {
            "active".to_string()
        } else {
            "planned".to_string()
        },
        progress: milestone_derived_progress(&issue_ids, &issue_refs),
        issue_ids: issue_ids.clone(),
        completed_issue_ids: completed_issue_ids.clone(),
        next_issue_intent: fallback_intent.clone(),
    };
    let derived_project = LocalProject {
        version: VERSION.to_string(),
        id: project_id.clone(),
        name: settings.project_name,
        status: "active".to_string(),
        canonical_status: canonical_project_status_string("active"),
        goal: goal.objective,
        team_ids: vec![team_id],
        active_milestone_id: milestone_id,
        milestones: vec![derived_milestone],
        issue_count: issue_refs.len(),
        completed_issue_count: completed_issue_ids.len(),
        issue_ids,
        next_issue_intent: fallback_intent,
        recommended_command: Some(selection.recommended_command.clone()),
    };
    let (workspace, teams, projects) = local_project_seed_snapshot(
        project_dir,
        derived_workspace,
        derived_team,
        derived_project,
        &issue_refs,
        &selection,
    )?;

    Ok(LocalProjectModelSnapshot {
        version: VERSION.to_string(),
        initialized: true,
        project_root: root.display().to_string(),
        workspace: Some(workspace),
        teams,
        projects,
        issue_refs,
        goal_loop_selection: selection,
        sources: local_project_model_sources(),
        boundary: workbench_boundary(),
    })
}

pub fn read_local_project_seed_preview(start: &Path) -> Result<LocalProjectSeedPreview> {
    let snapshot = read_local_project_model_snapshot(start)?;
    if !snapshot.initialized {
        return Ok(empty_local_project_seed_preview(&snapshot));
    }

    let root = PathBuf::from(&snapshot.project_root);
    let project_dir = root.join(AGENTFLOW_DIR);
    let files = local_project_seed_files(&project_dir, &snapshot)?;
    let writes_required = files.iter().any(|file| file.action == "create");

    Ok(LocalProjectSeedPreview {
        version: VERSION.to_string(),
        initialized: true,
        project_root: snapshot.project_root,
        files,
        confirmation_gates: local_project_seed_confirmation_gates(),
        writes_required,
        sources: local_project_seed_sources(),
        boundary: workbench_boundary(),
    })
}

pub fn write_local_project_seed(start: &Path, yes: bool) -> Result<LocalProjectSeedWriteSummary> {
    if !yes {
        bail!("project seed write requires explicit --yes confirmation");
    }

    let preview = read_local_project_seed_preview(start)?;
    if !preview.initialized {
        bail!("project seed write requires an initialized .agentflow directory");
    }

    for file in &preview.files {
        if file.action != "create" {
            bail!(
                "{} already exists; Local Project Seed v0 refuses overwrite",
                file.path
            );
        }
    }

    let root = PathBuf::from(&preview.project_root);
    let mut prepared_files = Vec::new();
    for file in &preview.files {
        let path = seed_absolute_path(&root, &file.path)?;
        let content = serde_json::to_string_pretty(&file.content)? + "\n";
        prepared_files.push((path, content));
    }

    let mut written_paths = Vec::new();
    for (path, content) in prepared_files {
        match write_new(&path, &content, false) {
            Ok(()) => written_paths.push(path),
            Err(error) => {
                rollback_project_seed_write(&root, &written_paths);
                return Err(error);
            }
        }
    }

    Ok(LocalProjectSeedWriteSummary {
        preview,
        written_paths,
    })
}

pub fn read_issue_project_link_preview(
    start: &Path,
    issue_id: &str,
) -> Result<IssueProjectLinkPreview> {
    let snapshot = read_local_project_model_snapshot(start)?;
    if !snapshot.initialized {
        bail!("issue project link preview requires an initialized .agentflow directory");
    }

    let root = PathBuf::from(&snapshot.project_root);
    let project_dir = root.join(AGENTFLOW_DIR);
    let issue = read_issue(&project_dir, issue_id)?;
    let project_link = default_issue_project_link(&snapshot)?;
    let action = if issue.project_link.is_some() {
        "exists"
    } else {
        "write"
    }
    .to_string();

    Ok(IssueProjectLinkPreview {
        version: VERSION.to_string(),
        initialized: true,
        project_root: snapshot.project_root,
        issue_id: issue.id.clone(),
        issue_title: issue.title,
        issue_json_path: format!(".agentflow/issues/{}.json", issue.id),
        issue_markdown_path: format!(".agentflow/issues/{}.md", issue.id),
        writes_required: action == "write",
        action,
        project_link,
        confirmation_gates: issue_project_link_confirmation_gates(),
        sources: issue_project_link_sources(),
        boundary: workbench_boundary(),
    })
}

pub fn write_issue_project_link(
    start: &Path,
    issue_id: &str,
    yes: bool,
) -> Result<IssueProjectLinkWriteSummary> {
    if !yes {
        bail!("issue project link write requires explicit --yes confirmation");
    }

    let preview = read_issue_project_link_preview(start, issue_id)?;
    if preview.action != "write" {
        bail!(
            "{} already has projectLink; Issue Project Link Writer v0 refuses overwrite",
            preview.issue_id
        );
    }

    let root = PathBuf::from(&preview.project_root);
    let project_dir = root.join(AGENTFLOW_DIR);
    let mut issue = read_issue(&project_dir, &preview.issue_id)?;
    if issue.project_link.is_some() {
        bail!(
            "{} already has projectLink; Issue Project Link Writer v0 refuses overwrite",
            issue.id
        );
    }

    issue.project_link = Some(preview.project_link.clone());
    write_issue(&project_dir, &issue)?;

    Ok(IssueProjectLinkWriteSummary {
        preview,
        written_paths: vec![
            project_dir.join(format!("issues/{}.json", issue.id)),
            project_dir.join(format!("issues/{}.md", issue.id)),
        ],
    })
}

pub fn create_product_feature(
    repo: &Path,
    draft: ProductFeatureDraft,
    write: bool,
    yes: bool,
) -> Result<ProductFeatureCreationSummary> {
    if write && !yes {
        bail!("feature create write requires explicit --yes confirmation");
    }

    let project_dir = required_project_dir(repo)?;
    let mut snapshot = product_feature_creation_snapshot(repo, &project_dir, draft, write)?;
    let summary_path = project_dir.join("updates/FEATURE-CREATION-SUMMARY.md");
    let mut written_paths = Vec::new();

    if write {
        let issue_ids = snapshot
            .issues
            .iter()
            .map(|issue| issue.id.clone())
            .collect::<Vec<_>>();
        let issue_contracts = product_feature_issue_contracts(
            &project_dir,
            &snapshot.draft,
            &snapshot.project,
            &snapshot.issues,
        )?;
        let project_path = project_dir.join(format!("projects/{}.json", snapshot.project.id));
        let team_path = project_dir.join(format!("teams/{}.json", snapshot.draft.team_id));
        let workspace_path = project_dir.join("workspace.json");
        let mut index: ProjectIndex = read_json(&project_dir.join("index.json"))?;
        let project_seed = product_feature_project_seed(&snapshot);
        let workspace_seed =
            product_feature_workspace_seed(&project_dir, &snapshot.draft, &snapshot.project)?;
        let team_seed = product_feature_team_seed(
            &project_dir,
            &snapshot.draft,
            &snapshot.project,
            &issue_ids,
        )?;

        if project_path.exists() {
            bail!(
                "{} already exists; Product Feature Creation Flow v0 refuses overwrite",
                project_path.display()
            );
        }

        write_json(&project_path, &project_seed, false)?;
        written_paths.push(project_path);

        for contract in &issue_contracts {
            write_issue(&project_dir, contract)?;
            written_paths.push(project_dir.join(format!("issues/{}.json", contract.id)));
            written_paths.push(project_dir.join(format!("issues/{}.md", contract.id)));
        }

        for issue in &snapshot.issues {
            index.issues.push(IssueIndexEntry {
                id: issue.id.clone(),
                title: issue.title.clone(),
                status: IssueStatus::Todo.as_str().to_string(),
            });
        }
        index.next_issue_number += u32::try_from(snapshot.issues.len())
            .map_err(|_| anyhow!("feature issue count overflow"))?;
        write_json(&project_dir.join("index.json"), &index, true)?;
        written_paths.push(project_dir.join("index.json"));

        write_json(&team_path, &team_seed, true)?;
        written_paths.push(team_path);
        write_json(&workspace_path, &workspace_seed, true)?;
        written_paths.push(workspace_path);

        snapshot.mode = "write".to_string();
        snapshot.written_paths = written_paths
            .iter()
            .map(|path| feature_relative_path(repo, path))
            .collect();
        write_new(
            &summary_path,
            &product_feature_creation_summary_markdown(&snapshot),
            true,
        )?;
        written_paths.push(summary_path.clone());
    }

    Ok(ProductFeatureCreationSummary {
        snapshot,
        summary_path,
        written_paths,
    })
}

pub fn create_team(
    repo: &Path,
    draft: TeamDraft,
    write: bool,
    yes: bool,
) -> Result<CreationWriteSummary> {
    if write && !yes {
        bail!("team create write requires explicit --yes confirmation");
    }

    let project_dir = required_project_dir(repo)?;
    let draft = normalize_team_draft(draft)?;
    let team_id = draft_seed_id(
        &project_dir,
        "teams",
        draft.team_id.as_deref(),
        &draft.name,
        "team",
    )?;
    let team_path = project_dir.join("teams").join(format!("{team_id}.json"));
    let team_action = seed_file_action(&team_path);
    let team_seed = team_creation_seed(&team_id, &draft.name);
    let workspace_path = project_dir.join("workspace.json");
    let workspace_seed = workspace_with_team(&project_dir, &team_id)?;

    let mut preview = CreationPreview {
        version: VERSION.to_string(),
        initialized: true,
        project_root: repo.display().to_string(),
        mode: if write { "write" } else { "preview" }.to_string(),
        kind: "team".to_string(),
        entity_id: team_id.clone(),
        title: draft.name.clone(),
        action: team_action.clone(),
        writes_required: team_action == "create",
        files: vec![
            CreationPreviewFile {
                path: feature_relative_path(repo, &team_path),
                kind: "team".to_string(),
                action: team_action.clone(),
                content: team_seed.clone(),
            },
            CreationPreviewFile {
                path: feature_relative_path(repo, &workspace_path),
                kind: "workspace".to_string(),
                action: "update".to_string(),
                content: workspace_seed.clone(),
            },
        ],
        v1_contract: creation_v1_team_contract(&team_id, &draft.name),
        confirmation_gates: creation_confirmation_gates(),
        recommended_command: format!("agentflow team create \"{}\" --write --yes", draft.name),
        sources: creation_sources("team"),
        boundary: creation_boundary(write),
    };

    let mut written_paths = Vec::new();
    if write {
        if team_path.exists() {
            bail!(
                "{} already exists; Team Writer v0 refuses overwrite",
                team_path.display()
            );
        }
        write_json(&team_path, &team_seed, false)?;
        written_paths.push(team_path);
        write_json(&workspace_path, &workspace_seed, true)?;
        written_paths.push(workspace_path);
        preview.mode = "write".to_string();
    }

    Ok(CreationWriteSummary {
        preview,
        written_paths,
    })
}

pub fn create_project(
    repo: &Path,
    draft: ProjectDraft,
    write: bool,
    yes: bool,
) -> Result<CreationWriteSummary> {
    if write && !yes {
        bail!("project create write requires explicit --yes confirmation");
    }

    let project_dir = required_project_dir(repo)?;
    let draft = normalize_project_draft(&project_dir, draft)?;
    let project_id = draft_seed_id(
        &project_dir,
        "projects",
        draft.project_id.as_deref(),
        &draft.title,
        "project",
    )?;
    let team_id = draft
        .team_id
        .clone()
        .ok_or_else(|| anyhow!("project create requires a team id"))?;
    let project_path = project_dir
        .join("projects")
        .join(format!("{project_id}.json"));
    let team_path = project_dir.join("teams").join(format!("{team_id}.json"));
    let workspace_path = project_dir.join("workspace.json");
    if !team_path.exists() {
        bail!("team `{team_id}` is missing; run `agentflow team create \"{team_id}\" --write --yes` first");
    }
    let project_action = seed_file_action(&project_path);
    let project_seed = project_creation_seed(&project_id, &team_id, &draft);
    let team_seed = team_with_project(&team_path, &project_id)?;
    let workspace_seed = workspace_with_project(&project_dir, &project_id)?;

    let mut preview = CreationPreview {
        version: VERSION.to_string(),
        initialized: true,
        project_root: repo.display().to_string(),
        mode: if write { "write" } else { "preview" }.to_string(),
        kind: "project".to_string(),
        entity_id: project_id.clone(),
        title: draft.title.clone(),
        action: project_action.clone(),
        writes_required: project_action == "create",
        files: vec![
            CreationPreviewFile {
                path: feature_relative_path(repo, &project_path),
                kind: "project".to_string(),
                action: project_action.clone(),
                content: project_seed.clone(),
            },
            CreationPreviewFile {
                path: feature_relative_path(repo, &team_path),
                kind: "team".to_string(),
                action: "update".to_string(),
                content: team_seed.clone(),
            },
            CreationPreviewFile {
                path: feature_relative_path(repo, &workspace_path),
                kind: "workspace".to_string(),
                action: "update".to_string(),
                content: workspace_seed.clone(),
            },
        ],
        v1_contract: creation_v1_project_contract(&project_id, &team_id, &draft),
        confirmation_gates: creation_confirmation_gates(),
        recommended_command: format!(
            "agentflow project create \"{}\" --team-id {} --write --yes",
            draft.title, team_id
        ),
        sources: creation_sources("project"),
        boundary: creation_boundary(write),
    };

    let mut written_paths = Vec::new();
    if write {
        if project_path.exists() {
            bail!(
                "{} already exists; Project Writer v0 refuses overwrite",
                project_path.display()
            );
        }
        write_json(&project_path, &project_seed, false)?;
        written_paths.push(project_path);
        write_json(&team_path, &team_seed, true)?;
        written_paths.push(team_path);
        write_json(&workspace_path, &workspace_seed, true)?;
        written_paths.push(workspace_path);
        preview.mode = "write".to_string();
    }

    Ok(CreationWriteSummary {
        preview,
        written_paths,
    })
}

pub fn create_milestone(
    repo: &Path,
    draft: MilestoneDraft,
    write: bool,
    yes: bool,
) -> Result<CreationWriteSummary> {
    if write && !yes {
        bail!("milestone create write requires explicit --yes confirmation");
    }

    let project_dir = required_project_dir(repo)?;
    let draft = normalize_milestone_draft(&project_dir, draft)?;
    let project_id = draft
        .project_id
        .clone()
        .ok_or_else(|| anyhow!("milestone create requires a project id"))?;
    validate_project_link_id(&project_id)?;
    let project_path = project_dir
        .join("projects")
        .join(format!("{project_id}.json"));
    if !project_path.exists() {
        bail!("project `{project_id}` is missing; run `agentflow project create` first");
    }

    let mut project_seed: serde_json::Value = read_json(&project_path)?;
    let milestone_id = draft_milestone_id(&project_seed, draft.milestone_id.as_deref(), &draft)?;
    let milestone_exists = seed_milestone_exists(&project_seed, &milestone_id);
    let action = if milestone_exists { "exists" } else { "update" }.to_string();
    if !milestone_exists {
        append_milestone_seed(&mut project_seed, &milestone_id, &draft)?;
    }

    let mut preview = CreationPreview {
        version: VERSION.to_string(),
        initialized: true,
        project_root: repo.display().to_string(),
        mode: if write { "write" } else { "preview" }.to_string(),
        kind: "milestone".to_string(),
        entity_id: milestone_id.clone(),
        title: draft.title.clone(),
        action: action.clone(),
        writes_required: action == "update",
        files: vec![CreationPreviewFile {
            path: feature_relative_path(repo, &project_path),
            kind: "project".to_string(),
            action: action.clone(),
            content: project_seed.clone(),
        }],
        v1_contract: creation_v1_milestone_contract(&project_id, &milestone_id, &draft),
        confirmation_gates: creation_confirmation_gates(),
        recommended_command: format!(
            "agentflow milestone create \"{}\" --project-id {} --write --yes",
            draft.title, project_id
        ),
        sources: creation_sources("milestone"),
        boundary: creation_boundary(write),
    };

    let mut written_paths = Vec::new();
    if write {
        if milestone_exists {
            bail!(
                "milestone `{milestone_id}` already exists in project `{project_id}`; Milestone Writer v0 refuses overwrite"
            );
        }
        write_json(&project_path, &project_seed, true)?;
        written_paths.push(project_path);
        preview.mode = "write".to_string();
    }

    Ok(CreationWriteSummary {
        preview,
        written_paths,
    })
}

pub fn create_issue(
    repo: &Path,
    draft: IssueDraft,
    write: bool,
    yes: bool,
) -> Result<CreationWriteSummary> {
    if write && !yes {
        bail!("issue create write requires explicit --yes confirmation");
    }

    let project_dir = required_project_dir(repo)?;
    let draft = normalize_issue_draft(&project_dir, draft)?;
    let mut index: ProjectIndex = read_json(&project_dir.join("index.json"))?;
    let issue_id = format!("ISSUE-{:04}", index.next_issue_number);
    let issue_json_path = project_dir.join(format!("issues/{issue_id}.json"));
    let issue_markdown_path = project_dir.join(format!("issues/{issue_id}.md"));
    if issue_json_path.exists() || issue_markdown_path.exists() {
        bail!(
            "{} already exists; Issue Writer v0 refuses stale index overwrite",
            issue_id
        );
    }

    let project_id = draft
        .project_id
        .clone()
        .ok_or_else(|| anyhow!("issue create requires a project id"))?;
    let milestone_id = draft
        .milestone_id
        .clone()
        .ok_or_else(|| anyhow!("issue create requires a milestone id"))?;
    let team_id = draft
        .team_id
        .clone()
        .ok_or_else(|| anyhow!("issue create requires a team id"))?;
    validate_project_link_id(&project_id)?;
    validate_project_link_id(&milestone_id)?;
    validate_project_link_id(&team_id)?;

    let project_path = project_dir
        .join("projects")
        .join(format!("{project_id}.json"));
    let team_path = project_dir.join("teams").join(format!("{team_id}.json"));
    if !project_path.exists() {
        bail!("project `{project_id}` is missing; run `agentflow project create` first");
    }
    if !team_path.exists() {
        bail!("team `{team_id}` is missing; run `agentflow team create` first");
    }

    let contract = issue_creation_contract(&project_dir, &draft, &issue_id)?;
    let issue_markdown = issue_markdown_body(&contract);
    let mut project_seed: serde_json::Value = read_json(&project_path)?;
    if !seed_milestone_exists(&project_seed, &milestone_id) {
        bail!("milestone `{milestone_id}` is missing from project `{project_id}`");
    }
    append_unique_json_string_array(&mut project_seed, "issueIds", &issue_id)?;
    append_issue_to_seed_milestone(&mut project_seed, &milestone_id, &issue_id, &draft.title)?;
    let mut team_seed: serde_json::Value = read_json(&team_path)?;
    append_unique_json_string_array(&mut team_seed, "issueIds", &issue_id)?;
    index.issues.push(IssueIndexEntry {
        id: issue_id.clone(),
        title: draft.title.clone(),
        status: IssueStatus::Todo.as_str().to_string(),
    });
    index.next_issue_number += 1;

    let mut preview = CreationPreview {
        version: VERSION.to_string(),
        initialized: true,
        project_root: repo.display().to_string(),
        mode: if write { "write" } else { "preview" }.to_string(),
        kind: "issue".to_string(),
        entity_id: issue_id.clone(),
        title: draft.title.clone(),
        action: "create".to_string(),
        writes_required: true,
        files: vec![
            CreationPreviewFile {
                path: feature_relative_path(repo, &issue_json_path),
                kind: "issue-json".to_string(),
                action: "create".to_string(),
                content: serde_json::to_value(&contract)?,
            },
            CreationPreviewFile {
                path: feature_relative_path(repo, &issue_markdown_path),
                kind: "issue-markdown".to_string(),
                action: "create".to_string(),
                content: serde_json::Value::String(issue_markdown.clone()),
            },
            CreationPreviewFile {
                path: feature_relative_path(repo, &project_path),
                kind: "project".to_string(),
                action: "update".to_string(),
                content: project_seed.clone(),
            },
            CreationPreviewFile {
                path: feature_relative_path(repo, &team_path),
                kind: "team".to_string(),
                action: "update".to_string(),
                content: team_seed.clone(),
            },
            CreationPreviewFile {
                path: feature_relative_path(repo, &project_dir.join("index.json")),
                kind: "index".to_string(),
                action: "update".to_string(),
                content: serde_json::to_value(&index)?,
            },
        ],
        v1_contract: creation_v1_issue_contract(&contract),
        confirmation_gates: creation_confirmation_gates(),
        recommended_command: format!(
            "agentflow issue create \"{}\" --project-id {} --milestone-id {} --team-id {} --write --yes",
            draft.title, project_id, milestone_id, team_id
        ),
        sources: creation_sources("issue"),
        boundary: creation_boundary(write),
    };

    let mut written_paths = Vec::new();
    if write {
        write_issue(&project_dir, &contract)?;
        written_paths.push(issue_json_path);
        written_paths.push(issue_markdown_path);
        write_json(&project_path, &project_seed, true)?;
        written_paths.push(project_path);
        write_json(&team_path, &team_seed, true)?;
        written_paths.push(team_path);
        write_json(&project_dir.join("index.json"), &index, true)?;
        written_paths.push(project_dir.join("index.json"));
        preview.mode = "write".to_string();
    }

    Ok(CreationWriteSummary {
        preview,
        written_paths,
    })
}

pub fn read_product_feature_execution_status(
    repo: &Path,
) -> Result<ProductFeatureExecutionSnapshot> {
    product_feature_execution_snapshot(repo)
}

pub fn read_product_feature_execution_next(repo: &Path) -> Result<ProductFeatureExecutionSnapshot> {
    product_feature_execution_snapshot(repo)
}

fn prepare_plan_project_seed_update(
    repo: &Path,
    project_dir: &Path,
    issue_id: &str,
    intent: &str,
) -> Result<Option<PlanProjectSeedUpdate>> {
    if !project_dir.join("workspace.json").exists() {
        return Ok(None);
    }

    let snapshot = read_local_project_model_snapshot(repo)?;
    if !snapshot.initialized {
        return Ok(None);
    }

    let mut project_link = default_issue_project_link(&snapshot)?;
    project_link.link_source = "milestone-aware-issue-planning-v0".to_string();

    let team_path = project_dir
        .join("teams")
        .join(format!("{}.json", project_link.team_id));
    let project_path = project_dir
        .join("projects")
        .join(format!("{}.json", project_link.project_id));
    if !team_path.exists() {
        bail!(
            "Milestone-aware issue planning requires team seed {}",
            team_path.display()
        );
    }
    if !project_path.exists() {
        bail!(
            "Milestone-aware issue planning requires project seed {}",
            project_path.display()
        );
    }

    let mut team_seed: serde_json::Value = read_json(&team_path)?;
    append_unique_json_string_array(&mut team_seed, "issueIds", issue_id)?;

    let mut project_seed: serde_json::Value = read_json(&project_path)?;
    append_unique_json_string_array(&mut project_seed, "issueIds", issue_id)?;
    append_issue_to_seed_milestone(
        &mut project_seed,
        &project_link.milestone_id,
        issue_id,
        intent,
    )?;

    Ok(Some(PlanProjectSeedUpdate {
        project_link,
        team_path,
        team_seed,
        project_path,
        project_seed,
    }))
}

pub fn plan_issue(repo: &Path, intent: &str) -> Result<PlanSummary> {
    let project_dir = repo.join(AGENTFLOW_DIR);
    if !project_dir.exists() {
        bail!(
            "{} is missing; run agentflow init --from-goal first",
            AGENTFLOW_DIR
        );
    }

    let goal: ProjectGoal = read_json(&project_dir.join("goal.json"))?;
    let settings: Settings = read_json(&project_dir.join("settings.json"))?;
    let project_context = read_optional_context(&project_dir)?;
    let mut index: ProjectIndex = read_json(&project_dir.join("index.json"))?;
    let issue_id = format!("ISSUE-{:04}", index.next_issue_number);
    let title = issue_title(intent);
    let context_files = selected_context_files(&project_context, intent);
    let project_seed_update =
        prepare_plan_project_seed_update(repo, &project_dir, &issue_id, intent)?;
    let project_link = project_seed_update
        .as_ref()
        .map(|update| update.project_link.clone());
    let contract = IssueContract {
        id: issue_id.clone(),
        title: title.clone(),
        status: IssueStatus::Todo.as_str().to_string(),
        intent: intent.trim().to_string(),
        risk_level: "medium".to_string(),
        scope: vec![
            format!("围绕本次请求形成有边界的实现计划：{intent}"),
            "只在当前 issue contract 授权范围内修改。".to_string(),
        ],
        non_goals: goal.non_goals.iter().take(5).cloned().collect(),
        context: IssueContext {
            repo: ".".to_string(),
            files: context_files,
        },
        execution_plan: vec![
            "读取 goal 和当前本地项目事实源。".to_string(),
            "读取 `.agentflow/context.json` 并只保留本 issue 必需的上下文。".to_string(),
            "按最小范围完成实现。".to_string(),
            "运行配置的验证命令。".to_string(),
            "生成 evidence 和 review 输出。".to_string(),
        ],
        validation: ValidationSpec {
            commands: settings.validation_commands.clone(),
        },
        evidence_requirements: vec![
            "transcript".to_string(),
            "command-output".to_string(),
            "diff-summary".to_string(),
            "known-limitations".to_string(),
            "aep-contract-checklist".to_string(),
            "docs-claim-trace".to_string(),
        ],
        rollback_plan: vec![
            "保留变更 diff，验证失败时回退本 issue 修改。".to_string(),
            "不修改未列入 scope 的模块；若必须扩大范围，停止并请求人工确认。".to_string(),
        ],
        human_gate: HumanGate {
            before_file_edits: false,
            before_external_network: true,
        },
        project_link: project_link.clone(),
        aep: aep_issue_protocol(&settings),
    };

    let issue_markdown = project_dir.join(format!("issues/{issue_id}.md"));
    let issue_json = project_dir.join(format!("issues/{issue_id}.json"));
    write_new(&issue_markdown, &issue_markdown_body(&contract), false)?;
    write_json(&issue_json, &contract, false)?;

    index.next_issue_number += 1;
    index.issues.push(IssueIndexEntry {
        id: issue_id.clone(),
        title,
        status: IssueStatus::Todo.as_str().to_string(),
    });
    write_json(&project_dir.join("index.json"), &index, true)?;

    let updated_project_seed_paths = if let Some(update) = project_seed_update {
        write_json(&update.team_path, &update.team_seed, true)?;
        write_json(&update.project_path, &update.project_seed, true)?;
        vec![update.team_path, update.project_path]
    } else {
        Vec::new()
    };

    Ok(PlanSummary {
        issue_id,
        issue_markdown,
        issue_json,
        project_link,
        updated_project_seed_paths,
    })
}

pub fn write_context(repo: &Path) -> Result<ContextSummary> {
    let project_dir = repo.join(AGENTFLOW_DIR);
    if !project_dir.exists() {
        bail!(
            "{} is missing; run agentflow init --from-goal first",
            AGENTFLOW_DIR
        );
    }
    let context = collect_context(repo)?;
    let context_json = project_dir.join("context.json");
    let context_markdown = project_dir.join("context.md");
    write_json(&context_json, &context, true)?;
    write_new(&context_markdown, &context_markdown_body(&context), true)?;
    Ok(ContextSummary {
        context_json,
        context_markdown,
        file_count: context.files.len(),
    })
}

pub fn run_issue(repo: &Path, issue_id: &str) -> Result<RunSummary> {
    start_run(repo, issue_id)
}

pub fn verify_issue(repo: &Path, issue_id: &str) -> Result<ValidationSummary> {
    let project_dir = required_project_dir(repo)?;
    let issue = read_issue(&project_dir, issue_id)?;
    if issue.validation.commands.is_empty() {
        bail!("{issue_id} has no validation commands");
    }

    let run_dir = match latest_run_dir_for_issue(&project_dir, issue_id)? {
        Some(run_dir) => run_dir,
        None => start_run(repo, issue_id)?.run_dir,
    };
    let run_json = run_dir.join("run.json");
    let mut run: AgentRun = read_json(&run_json)?;
    let mut records = Vec::new();

    for command in &issue.validation.commands {
        let output = Command::new("sh")
            .arg("-c")
            .arg(command)
            .current_dir(repo)
            .output()
            .with_context(|| format!("run validation command `{command}`"))?;
        let exit_code = output.status.code().unwrap_or(-1);
        let record = CommandRecord {
            command: command.clone(),
            exit_code,
            status: if exit_code == 0 { "passed" } else { "failed" }.to_string(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        };
        append_jsonl(&run_dir.join("commands.jsonl"), &record)?;
        records.push(record);
    }

    let passed = records.iter().all(|record| record.exit_code == 0);
    run.validation_commands.extend(records.clone());
    run.status = if passed { "validated" } else { "failed" }.to_string();
    write_json(&run_json, &run, true)?;

    Ok(ValidationSummary {
        issue_id: issue_id.to_string(),
        run_id: run.id,
        passed,
        commands: records,
    })
}

pub fn review_issue(repo: &Path, issue_id: &str) -> Result<ReviewSummary> {
    let project_dir = required_project_dir(repo)?;
    let mut issue = hydrate_issue_protocol(&project_dir, read_issue(&project_dir, issue_id)?)?;
    let run_dir = latest_run_dir_for_issue(&project_dir, issue_id)?
        .ok_or_else(|| anyhow!("{issue_id} has no run; run agentflow run first"))?;
    let run_json = run_dir.join("run.json");
    let mut run: AgentRun = read_json(&run_json)?;
    if run.validation_commands.is_empty() {
        bail!("{issue_id} has no validation output; run agentflow verify first");
    }

    let passed = run
        .validation_commands
        .iter()
        .all(|record| record.exit_code == 0);
    let evidence_path = project_dir.join(format!("evidence/{issue_id}-evidence.md"));
    let review_path = project_dir.join(format!("reviews/{issue_id}-review.md"));
    let update_path = next_project_update_path(&project_dir)?;

    write_new(
        &evidence_path,
        &evidence_markdown(&issue, &run, passed),
        true,
    )?;
    write_new(&review_path, &review_markdown(&issue, &run, passed), true)?;
    write_new(
        &update_path,
        &project_update_markdown(&issue, &run, passed),
        false,
    )?;

    run.status = if passed {
        "completed"
    } else {
        "reviewed-with-failures"
    }
    .to_string();
    run.outputs.evidence = Some(format!("../../evidence/{issue_id}-evidence.md"));
    run.outputs.review = Some(format!("../../reviews/{issue_id}-review.md"));
    run.outputs.update = Some(format!(
        "../../updates/{}",
        update_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("PROJECT-UPDATE.md")
    ));
    write_json(&run_json, &run, true)?;

    if passed {
        issue.status = IssueStatus::Done.as_str().to_string();
        write_issue(&project_dir, &issue)?;
        update_issue_status(&project_dir, issue_id, IssueStatus::Done.as_str())?;
        write_milestone_summary_if_complete(&project_dir, &issue, &run)?;
    }
    release_workflow_lease_after_review(&project_dir, &run)?;
    release_scope_state_after_review(&project_dir, issue_id, passed)?;

    Ok(ReviewSummary {
        issue_id: issue_id.to_string(),
        run_id: run.id,
        passed,
        evidence_path,
        review_path,
        update_path,
    })
}

pub fn rebuild_index(repo: &Path) -> Result<IndexSummary> {
    let project_dir = required_project_dir(repo)?;
    let sqlite_path = project_dir.join("index.sqlite");
    if sqlite_path.exists() {
        fs::remove_file(&sqlite_path)
            .with_context(|| format!("remove sqlite index {}", sqlite_path.display()))?;
    }

    let connection = Connection::open(&sqlite_path)
        .with_context(|| format!("open sqlite index {}", sqlite_path.display()))?;
    create_index_schema(&connection)?;

    let issues = load_indexed_issues(repo, &project_dir)?;
    for issue in &issues {
        connection.execute(
            "INSERT INTO issues (id, title, status, intent, json_path) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                issue.id,
                issue.title,
                issue.status,
                issue.intent,
                issue.json_path
            ],
        )?;
    }

    let runs = load_indexed_runs(repo, &project_dir)?;
    for (run, records) in &runs {
        connection.execute(
            "INSERT INTO runs (id, issue_id, status, mode, validation_status, json_path) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                run.id,
                run.issue_id,
                run.status,
                run.mode,
                run.validation_status,
                run.json_path
            ],
        )?;
        for record in records {
            connection.execute(
                "INSERT INTO commands (run_id, command, exit_code, status) VALUES (?1, ?2, ?3, ?4)",
                params![run.id, record.command, record.exit_code, record.status],
            )?;
        }
    }

    let updates = load_indexed_updates(repo, &project_dir)?;
    for update in &updates {
        connection.execute(
            "INSERT INTO updates (path, source_issue, source_run, status) VALUES (?1, ?2, ?3, ?4)",
            params![
                update.path,
                update.source_issue,
                update.source_run,
                update.status
            ],
        )?;
    }

    let views = load_saved_views(&project_dir)?;
    for view in &views {
        insert_saved_view(&connection, view, &view_file_path(&project_dir, &view.id))?;
    }

    Ok(IndexSummary {
        sqlite_path,
        issue_count: issues.len(),
        run_count: runs.len(),
        update_count: updates.len(),
        saved_view_count: views.len(),
    })
}

pub fn save_view(repo: &Path, name: &str, filter: SavedViewFilter) -> Result<SavedViewSummary> {
    let project_dir = required_project_dir(repo)?;
    ensure_dir(&project_dir.join("views"))?;
    let view_id = saved_view_id(name, &project_dir)?;
    let view_path = view_file_path(&project_dir, &view_id);
    let view = SavedView {
        version: VERSION.to_string(),
        id: view_id.clone(),
        name: name.trim().to_string(),
        filter,
    };
    write_json(&view_path, &view, true)?;
    let summary = rebuild_index(repo)?;
    Ok(SavedViewSummary {
        view_id,
        view_path,
        sqlite_path: summary.sqlite_path,
    })
}

pub fn show_view(repo: &Path, name_or_id: &str) -> Result<ViewResult> {
    let project_dir = required_project_dir(repo)?;
    let sqlite_path = project_dir.join("index.sqlite");
    if !sqlite_path.exists() {
        rebuild_index(repo)?;
    }
    let view_path = find_saved_view_path(&project_dir, name_or_id)?;
    let view: SavedView = read_json(&view_path)?;
    let connection = Connection::open(&sqlite_path)
        .with_context(|| format!("open sqlite index {}", sqlite_path.display()))?;
    let issues = query_indexed_issues(&connection, &view.filter)?;
    let runs = query_indexed_runs(&connection, &view.filter)?;
    Ok(ViewResult { view, issues, runs })
}

pub fn write_project_summary(repo: &Path) -> Result<ProjectSummaryResult> {
    let project_dir = required_project_dir(repo)?;
    let index_summary = rebuild_index(repo)?;
    let issues = load_indexed_issues(repo, &project_dir)?;
    let runs = load_indexed_runs(repo, &project_dir)?;
    let updates = load_indexed_updates(repo, &project_dir)?;
    let views = load_saved_views(&project_dir)?;
    let summary_path = project_dir.join("updates/PROJECT-SUMMARY.md");
    let completed_issue_count = issues
        .iter()
        .filter(|issue| issue_state_done(&issue.status))
        .count();

    write_new(
        &summary_path,
        &project_summary_markdown(&issues, &runs, &updates, &views, &index_summary),
        true,
    )?;

    Ok(ProjectSummaryResult {
        summary_path,
        issue_count: issues.len(),
        completed_issue_count,
        run_count: runs.len(),
        update_count: updates.len(),
        saved_view_count: views.len(),
    })
}

pub fn write_review_assistant(repo: &Path, issue_id: &str) -> Result<ReviewAssistantSummary> {
    let project_dir = required_project_dir(repo)?;
    rebuild_index(repo)?;
    let issue = hydrate_issue_protocol(&project_dir, read_issue(&project_dir, issue_id)?)?;
    let updates = load_indexed_updates(repo, &project_dir)?;
    let run = latest_run_for_issue(&project_dir, issue_id)?;
    let evidence_path = project_dir.join(format!("evidence/{issue_id}-evidence.md"));
    let review_path = project_dir.join(format!("reviews/{issue_id}-review.md"));
    let assistant_path = project_dir.join(format!("reviews/{issue_id}-assistant.md"));
    let has_update = updates.iter().any(|update| update.source_issue == issue_id);
    let scope_state_path = project_dir.join("scope-state.json");
    let scope_state_exists = scope_state_path.exists();

    let mut checks = Vec::new();
    checks.push(assistant_check(
        "Issue contract",
        true,
        format!("{} exists with status `{}`.", issue.id, issue.status),
    ));
    checks.push(assistant_check(
        "Scope and non-goals",
        !issue.scope.is_empty() && !issue.non_goals.is_empty(),
        format!(
            "{} scope items, {} non-goals.",
            issue.scope.len(),
            issue.non_goals.len()
        ),
    ));
    checks.push(assistant_check(
        "Validation contract",
        !issue.validation.commands.is_empty(),
        format!("{} validation commands.", issue.validation.commands.len()),
    ));
    checks.push(assistant_check(
        "AEP protocol fields",
        aep_protocol_complete(&issue.aep),
        format!(
            "phase `{}`, {} feedback commands, {} docs claims.",
            issue.aep.phase,
            issue.aep.fastest_feedback_loop.len(),
            issue.aep.docs_claim_trace.len()
        ),
    ));
    checks.push(assistant_check(
        "Boundary confirmation",
        !issue.aep.boundary_confirmation.is_empty(),
        format!("{} boundary claims.", issue.aep.boundary_confirmation.len()),
    ));
    checks.push(assistant_check(
        "Docs claim trace",
        !issue.aep.docs_claim_trace.is_empty()
            && issue
                .aep
                .docs_claim_trace
                .iter()
                .all(|path| docs_claim_path_exists(repo, &project_dir, path)),
        format!("{} traced docs.", issue.aep.docs_claim_trace.len()),
    ));
    checks.push(assistant_check(
        "Graphify context status",
        !issue.aep.graphify_context_status.is_empty(),
        issue.aep.graphify_context_status.clone(),
    ));
    checks.push(assistant_check(
        "Scope state",
        scope_state_exists,
        if scope_state_exists {
            ".agentflow/scope-state.json exists.".to_string()
        } else {
            ".agentflow/scope-state.json is missing.".to_string()
        },
    ));
    let (goal_loop_ready, goal_loop_detail) = goal_loop_assistant_status(&project_dir)?;
    checks.push(assistant_check(
        "Goal Loop readiness",
        goal_loop_ready,
        goal_loop_detail,
    ));

    if let Some(run) = &run {
        checks.push(assistant_check(
            "Latest run",
            true,
            format!("{} status `{}`.", run.id, run.status),
        ));
        checks.push(assistant_check(
            "Validation results",
            !run.validation_commands.is_empty()
                && run
                    .validation_commands
                    .iter()
                    .all(|record| record.exit_code == 0),
            validation_assistant_detail(run),
        ));
    } else {
        checks.push(assistant_check(
            "Latest run",
            false,
            "No run has been recorded for this issue.".to_string(),
        ));
        checks.push(assistant_check(
            "Validation results",
            false,
            "No validation output is available.".to_string(),
        ));
    }

    checks.push(assistant_check(
        "Evidence artifact",
        evidence_path.exists(),
        format!("{}.", relative_path(repo, &evidence_path)?),
    ));
    checks.push(assistant_check(
        "Review artifact",
        review_path.exists(),
        format!("{}.", relative_path(repo, &review_path)?),
    ));
    checks.push(assistant_check(
        "Project update",
        has_update,
        if has_update {
            "Numbered project update exists.".to_string()
        } else {
            "No numbered project update references this issue.".to_string()
        },
    ));
    checks.push(assistant_check(
        "SQLite index",
        project_dir.join("index.sqlite").exists(),
        ".agentflow/index.sqlite rebuilt from local facts.".to_string(),
    ));

    let ready = checks.iter().all(|check| check.status == "pass");
    write_new(
        &assistant_path,
        &review_assistant_markdown(&issue, run.as_ref(), ready, &checks),
        true,
    )?;

    Ok(ReviewAssistantSummary {
        issue_id: issue_id.to_string(),
        assistant_path,
        ready,
        checks,
    })
}

pub fn write_workflow_state_check(repo: &Path) -> Result<WorkflowStateCheckSummary> {
    let project_dir = required_project_dir(repo)?;
    let snapshot = read_local_project_model_snapshot(repo)?;
    let issues = read_issue_contracts(&project_dir)?;
    let mut checks = Vec::new();

    checks.push(workflow_state_check(
        "workflow-contract",
        "contract",
        "agentflow-ai-delivery-workflow-contract-v1",
        project_dir
            .parent()
            .unwrap_or(repo)
            .join("docs/contracts/agentflow-ai-delivery-workflow-contract-v1.md")
            .exists(),
        "AgentFlow AI Delivery Workflow Contract v1 exists.",
        "Canonical workflow contract is missing.",
        "error",
    ));

    if !snapshot.initialized {
        checks.push(workflow_state_check(
            "local-project-seed",
            "workspace",
            "default",
            false,
            "Local workspace / project seed exists.",
            "Local workspace / project seed is missing.",
            "error",
        ));
    }

    let mut milestone_count = 0usize;
    for project in &snapshot.projects {
        milestone_count += project.milestones.len();
        validate_workflow_project(project, &issues, &mut checks);
    }

    for issue in &issues {
        validate_workflow_issue(&project_dir, &snapshot, issue, &mut checks);
    }

    let errors = checks
        .iter()
        .filter(|check| check.severity == "error" && check.status == "fail")
        .count();
    let warnings = checks
        .iter()
        .filter(|check| check.severity == "warning" && check.status == "fail")
        .count();
    let state = WorkflowStateSnapshot {
        version: VERSION.to_string(),
        initialized: snapshot.initialized,
        project_root: snapshot.project_root.clone(),
        ready: errors == 0,
        counts: WorkflowStateCounts {
            projects: snapshot.projects.len(),
            milestones: milestone_count,
            issues: issues.len(),
            errors,
            warnings,
        },
        checks,
        transition_guards: workflow_transition_guards(),
        sources: vec![
            ".agentflow/workspace.json".to_string(),
            ".agentflow/teams/*.json".to_string(),
            ".agentflow/projects/*.json".to_string(),
            ".agentflow/issues/*.json".to_string(),
            "docs/contracts/agentflow-ai-delivery-workflow-contract-v1.md".to_string(),
        ],
        boundary: WorkbenchBoundary {
            read_only: false,
            disallowed_actions: vec![
                "execute-run".to_string(),
                "call-model".to_string(),
                "create-remote-issue".to_string(),
                "create-remote-pr".to_string(),
                "modify-desktop-state".to_string(),
            ],
        },
    };

    let state_dir = project_dir.join("state");
    ensure_dir(&state_dir)?;
    let snapshot_path = state_dir.join("workflow-state.json");
    let summary_path = project_dir.join("updates/WORKFLOW-STATE-SUMMARY.md");
    write_json(&snapshot_path, &state, true)?;
    write_new(
        &summary_path,
        &workflow_state_summary_markdown(&state),
        true,
    )?;

    Ok(WorkflowStateCheckSummary {
        snapshot: state,
        snapshot_path,
        summary_path,
    })
}

pub fn write_workflow_eligibility(
    repo: &Path,
    issue_id: Option<&str>,
) -> Result<WorkflowEligibilityCheckSummary> {
    let project_dir = required_project_dir(repo)?;
    let snapshot = workflow_eligibility_snapshot(repo, issue_id)?;
    let state_dir = project_dir.join("state");
    ensure_dir(&state_dir)?;
    let snapshot_path = state_dir.join("eligibility.json");
    let summary_path = project_dir.join("updates/ELIGIBILITY-SUMMARY.md");
    write_json(&snapshot_path, &snapshot, true)?;
    write_new(
        &summary_path,
        &workflow_eligibility_summary_markdown(&snapshot),
        true,
    )?;

    Ok(WorkflowEligibilityCheckSummary {
        snapshot,
        snapshot_path,
        summary_path,
    })
}

pub fn write_workflow_lease_snapshot(repo: &Path) -> Result<WorkflowLeaseSummary> {
    let project_dir = required_project_dir(repo)?;
    let snapshot = workflow_lease_snapshot(repo)?;
    let state_dir = project_dir.join("state");
    ensure_dir(&state_dir)?;
    let snapshot_path = state_dir.join("leases.json");
    let summary_path = project_dir.join("updates/LEASE-SUMMARY.md");
    write_json(&snapshot_path, &snapshot, true)?;
    write_new(
        &summary_path,
        &workflow_lease_summary_markdown(&snapshot),
        true,
    )?;

    Ok(WorkflowLeaseSummary {
        snapshot,
        snapshot_path,
        summary_path,
    })
}

pub fn write_project_closure_state(repo: &Path) -> Result<ProjectClosureStateSummary> {
    let project_dir = required_project_dir(repo)?;
    let snapshot = project_closure_state_snapshot(repo)?;
    let state_dir = project_dir.join("state");
    ensure_dir(&state_dir)?;
    let snapshot_path = state_dir.join("project-closure.json");
    let summary_path = project_dir.join("updates/PROJECT-CLOSURE-SUMMARY.md");
    write_json(&snapshot_path, &snapshot, true)?;
    write_new(
        &summary_path,
        &project_closure_summary_markdown(&snapshot),
        true,
    )?;

    Ok(ProjectClosureStateSummary {
        snapshot,
        snapshot_path,
        summary_path,
    })
}

pub fn write_project_code_audit_snapshot(repo: &Path) -> Result<ProjectCodeAuditSummary> {
    let project_dir = required_project_dir(repo)?;
    let snapshot = project_code_audit_snapshot(repo)?;
    let state_dir = project_dir.join("state");
    ensure_dir(&state_dir)?;
    let snapshot_path = project_code_audit_snapshot_path(&project_dir);
    let summary_path = project_dir.join("updates/PROJECT-CODE-AUDIT-SUMMARY.md");
    write_json(&snapshot_path, &snapshot, true)?;
    write_new(
        &summary_path,
        &project_code_audit_summary_markdown(&snapshot),
        true,
    )?;

    Ok(ProjectCodeAuditSummary {
        snapshot,
        snapshot_path,
        summary_path,
    })
}

pub fn write_project_docs_refresh_snapshot(repo: &Path) -> Result<ProjectDocsRefreshSummary> {
    let project_dir = required_project_dir(repo)?;
    let snapshot = project_docs_refresh_snapshot(repo)?;
    let state_dir = project_dir.join("state");
    ensure_dir(&state_dir)?;
    let snapshot_path = project_docs_refresh_snapshot_path(&project_dir);
    let summary_path = project_dir.join("updates/PROJECT-DOCS-REFRESH-SUMMARY.md");
    write_json(&snapshot_path, &snapshot, true)?;
    write_new(
        &summary_path,
        &project_docs_refresh_summary_markdown(&snapshot),
        true,
    )?;

    Ok(ProjectDocsRefreshSummary {
        snapshot,
        snapshot_path,
        summary_path,
    })
}

fn workflow_eligibility_snapshot(
    repo: &Path,
    issue_id: Option<&str>,
) -> Result<WorkflowEligibilitySnapshot> {
    let project_dir = required_project_dir(repo)?;
    let project_snapshot = read_workflow_control_project_snapshot(repo, &project_dir)?;
    let issues = read_issue_contracts(&project_dir)?;
    let scope_state = read_scope_state_or_default(&project_dir.join("scope-state.json"))?;
    let leases = read_workflow_leases(&project_dir)?;
    let now = current_epoch_seconds();
    let strict_project_seed = project_dir.join("workspace.json").exists();
    let active_project_id = project_snapshot
        .workspace
        .as_ref()
        .map(|workspace| workspace.active_project_id.clone())
        .or_else(|| {
            project_snapshot
                .projects
                .first()
                .map(|project| project.id.clone())
        });
    let active_project = active_project_id.as_ref().and_then(|project_id| {
        project_snapshot
            .projects
            .iter()
            .find(|project| &project.id == project_id)
    });
    let active_milestone_id = active_project.map(|project| project.active_milestone_id.clone());
    let selected_issues = selected_eligibility_issues(
        issue_id,
        active_project,
        active_milestone_id.as_deref(),
        &issues,
    )?;
    let active_milestone_open_count = active_project
        .and_then(|project| active_milestone_id.as_deref().map(|id| (project, id)))
        .and_then(|(project, milestone_id)| {
            project
                .milestones
                .iter()
                .find(|milestone| milestone.id == milestone_id)
        })
        .map(|milestone| {
            milestone
                .issue_ids
                .iter()
                .filter(|id| {
                    issues
                        .iter()
                        .find(|issue| issue.id == **id)
                        .is_some_and(|issue| issue_status_open(&issue.status))
                })
                .count()
        })
        .unwrap_or(0);

    let mut candidates = selected_issues
        .iter()
        .map(|issue| {
            evaluate_workflow_eligibility(
                &project_snapshot,
                strict_project_seed,
                active_project_id.as_deref(),
                active_milestone_id.as_deref(),
                active_milestone_open_count,
                &scope_state,
                &leases,
                now,
                issue,
            )
        })
        .collect::<Vec<_>>();

    candidates.sort_by(|left, right| left.issue_id.cmp(&right.issue_id));
    let eligible = candidates
        .iter()
        .filter(|candidate| candidate.eligible)
        .collect::<Vec<_>>();
    let eligible_issue_id = if eligible.len() == 1 {
        Some(eligible[0].issue_id.clone())
    } else {
        None
    };
    let summary = workflow_eligibility_summary(&project_snapshot, &candidates, &eligible);

    Ok(WorkflowEligibilitySnapshot {
        version: VERSION.to_string(),
        initialized: project_snapshot.initialized,
        project_root: project_snapshot.project_root,
        active_project_id,
        active_milestone_id,
        eligible_issue_id,
        candidates,
        summary,
        sources: vec![
            ".agentflow/projects/*.json".to_string(),
            ".agentflow/issues/*.json".to_string(),
            ".agentflow/leases/*.json".to_string(),
            ".agentflow/scope-state.json".to_string(),
            ".agentflow/state/workflow-state.json".to_string(),
        ],
        boundary: WorkbenchBoundary {
            read_only: false,
            disallowed_actions: vec![
                "execute-run".to_string(),
                "call-model".to_string(),
                "create-remote-issue".to_string(),
                "create-remote-pr".to_string(),
                "mark-issue-eligible-manually".to_string(),
            ],
        },
    })
}

fn selected_eligibility_issues<'a>(
    issue_id: Option<&str>,
    active_project: Option<&LocalProject>,
    active_milestone_id: Option<&str>,
    issues: &'a [IssueContract],
) -> Result<Vec<&'a IssueContract>> {
    if let Some(issue_id) = issue_id {
        let issue = issues
            .iter()
            .find(|issue| issue.id == issue_id)
            .ok_or_else(|| anyhow!("{issue_id} does not exist"))?;
        return Ok(vec![issue]);
    }

    if let Some((project, milestone_id)) = active_project
        .and_then(|project| active_milestone_id.map(|milestone_id| (project, milestone_id)))
    {
        if let Some(milestone) = project
            .milestones
            .iter()
            .find(|milestone| milestone.id == milestone_id)
        {
            let selected = milestone
                .issue_ids
                .iter()
                .filter_map(|issue_id| issues.iter().find(|issue| issue.id == *issue_id))
                .filter(|issue| issue_status_open(&issue.status))
                .collect::<Vec<_>>();
            return Ok(selected);
        }
    }

    Ok(issues
        .iter()
        .filter(|issue| issue_status_open(&issue.status))
        .collect())
}

fn evaluate_workflow_eligibility(
    snapshot: &LocalProjectModelSnapshot,
    strict_project_seed: bool,
    active_project_id: Option<&str>,
    active_milestone_id: Option<&str>,
    active_milestone_open_count: usize,
    scope_state: &AgentScopeState,
    leases: &[WorkflowLeaseRecord],
    now: u64,
    issue: &IssueContract,
) -> WorkflowEligibilityCandidate {
    let mut failure_reasons = Vec::new();
    let project_id = issue
        .project_link
        .as_ref()
        .map(|link| link.project_id.clone());
    let milestone_id = issue
        .project_link
        .as_ref()
        .map(|link| link.milestone_id.clone());
    let code_changing = issue_code_changing(issue);

    if issue_state_done(&issue.status) {
        failure_reasons.push("issue_completed".to_string());
    }
    if !issue_status_can_enter_execution(&issue.status) {
        failure_reasons.push(format!("issue_status_not_ready:{}", issue.status));
    }
    if issue.scope.is_empty() {
        failure_reasons.push("missing_scope".to_string());
    }
    if issue.non_goals.is_empty() {
        failure_reasons.push("missing_non_goals".to_string());
    }
    if issue.execution_plan.is_empty() {
        failure_reasons.push("missing_execution_plan".to_string());
    }
    if issue.risk_level.trim().is_empty() {
        failure_reasons.push("missing_risk_level".to_string());
    }
    if issue.validation.commands.is_empty() {
        failure_reasons.push("missing_validation_commands".to_string());
    }
    if issue.evidence_requirements.is_empty() {
        failure_reasons.push("missing_evidence_requirements".to_string());
    }
    if issue.rollback_plan.is_empty() {
        failure_reasons.push("missing_rollback_plan".to_string());
    }

    if strict_project_seed {
        let Some(link) = issue.project_link.as_ref() else {
            failure_reasons.push("missing_project_link".to_string());
            return eligibility_candidate(
                issue,
                project_id,
                milestone_id,
                false,
                false,
                true,
                None,
                failure_reasons,
            );
        };

        if active_project_id != Some(link.project_id.as_str()) {
            failure_reasons.push("issue_not_in_active_project".to_string());
        }
        if active_milestone_id != Some(link.milestone_id.as_str()) {
            failure_reasons.push("issue_not_in_active_milestone".to_string());
        }
        let project = snapshot
            .projects
            .iter()
            .find(|project| project.id == link.project_id);
        match project {
            Some(project) => {
                if canonical_project_status(&project.status) != ProjectStatus::Active {
                    failure_reasons.push(format!("project_not_active:{}", project.status));
                }
                let milestone = project
                    .milestones
                    .iter()
                    .find(|milestone| milestone.id == link.milestone_id);
                match milestone {
                    Some(milestone) => {
                        if !milestone.issue_ids.iter().any(|id| id == &issue.id) {
                            failure_reasons.push("issue_not_listed_in_milestone".to_string());
                        }
                    }
                    None => failure_reasons.push("milestone_missing".to_string()),
                }
            }
            None => failure_reasons.push("project_missing".to_string()),
        }

        if active_milestone_open_count > 1 {
            failure_reasons.push("active_milestone_multiple_open_issues".to_string());
        }
    }

    if let Some(active_issue_id) = &scope_state.active_issue_id {
        if active_issue_id != &issue.id {
            failure_reasons.push(format!("scope_active_issue_blocks_run:{active_issue_id}"));
        }
    }

    let mut active_lease_id = None;
    let mut leased = false;
    for lease in leases.iter().filter(|lease| lease.status == "active") {
        if lease.expires_at_epoch_seconds <= now {
            if lease.issue_id == issue.id {
                active_lease_id = Some(lease.id.clone());
                failure_reasons.push("stale_lease_requires_human_recovery".to_string());
            }
            continue;
        }
        if lease.issue_id == issue.id {
            active_lease_id = Some(lease.id.clone());
            leased = true;
            continue;
        }
        if code_changing
            && lease.project_id.as_deref() == project_id.as_deref()
            && lease.project_id.is_some()
        {
            failure_reasons.push(format!(
                "active_project_lease_blocks_code_changing_issue:{}",
                lease.issue_id
            ));
        }
    }

    let ready = issue_contract_ready(issue);
    let eligible = ready && failure_reasons.is_empty();
    eligibility_candidate(
        issue,
        project_id,
        milestone_id,
        ready,
        eligible,
        leased,
        active_lease_id,
        failure_reasons,
    )
}

fn eligibility_candidate(
    issue: &IssueContract,
    project_id: Option<String>,
    milestone_id: Option<String>,
    ready: bool,
    eligible: bool,
    leased: bool,
    active_lease_id: Option<String>,
    failure_reasons: Vec<String>,
) -> WorkflowEligibilityCandidate {
    WorkflowEligibilityCandidate {
        issue_id: issue.id.clone(),
        title: issue.title.clone(),
        issue_status: issue.status.clone(),
        project_id,
        milestone_id,
        ready,
        eligible,
        leased,
        code_changing: issue_code_changing(issue),
        active_lease_id,
        failure_reasons,
    }
}

fn issue_contract_ready(issue: &IssueContract) -> bool {
    issue_status_can_enter_execution(&issue.status)
        && !issue.scope.is_empty()
        && !issue.non_goals.is_empty()
        && !issue.execution_plan.is_empty()
        && !issue.risk_level.trim().is_empty()
        && !issue.validation.commands.is_empty()
        && !issue.evidence_requirements.is_empty()
        && !issue.rollback_plan.is_empty()
}

fn issue_code_changing(issue: &IssueContract) -> bool {
    let text = format!("{} {} {}", issue.title, issue.intent, issue.scope.join(" ")).to_lowercase();
    !(text.contains("read-only")
        || text.contains("只读")
        || text.contains("boundary")
        || text.contains("边界")
        || text.contains("docs-only")
        || text.contains("文档"))
}

fn workflow_eligibility_summary(
    snapshot: &LocalProjectModelSnapshot,
    candidates: &[WorkflowEligibilityCandidate],
    eligible: &[&WorkflowEligibilityCandidate],
) -> WorkflowEligibilitySummary {
    let ready_issue_count = candidates
        .iter()
        .filter(|candidate| candidate.ready)
        .count();
    let eligible_issue_count = eligible.len();
    let blocked_issue_count = candidates
        .iter()
        .filter(|candidate| !candidate.eligible)
        .count();

    if eligible_issue_count == 1 {
        let issue_id = &eligible[0].issue_id;
        return WorkflowEligibilitySummary {
            ready_issue_count,
            eligible_issue_count,
            blocked_issue_count,
            next_action: "run".to_string(),
            recommended_command: format!("agentflow run {issue_id} --dry-run"),
            rationale: vec![
                "Exactly one issue is eligible in the workflow control core.".to_string(),
                "Run still performs lease acquisition before creating Execution Run.".to_string(),
            ],
        };
    }

    if eligible_issue_count > 1 {
        return WorkflowEligibilitySummary {
            ready_issue_count,
            eligible_issue_count,
            blocked_issue_count,
            next_action: "wait-human".to_string(),
            recommended_command: "agentflow projects".to_string(),
            rationale: vec![
                format!("{eligible_issue_count} issues are eligible."),
                "MVP keeps one code-changing issue at a time; human must narrow the queue."
                    .to_string(),
            ],
        };
    }

    if candidates.is_empty() {
        let next_intent = snapshot_active_milestone_intent(snapshot)
            .or_else(|| snapshot.goal_loop_selection.next_issue_intent.clone());
        let command = next_intent
            .as_ref()
            .map(|intent| format!("agentflow plan \"{intent}\""))
            .unwrap_or_else(|| "agentflow projects".to_string());
        return WorkflowEligibilitySummary {
            ready_issue_count,
            eligible_issue_count,
            blocked_issue_count,
            next_action: if command.starts_with("agentflow plan") {
                "plan".to_string()
            } else {
                "wait-human".to_string()
            },
            recommended_command: command,
            rationale: vec![
                "Active milestone has no open issue candidate.".to_string(),
                "Eligibility cannot be computed until an IssueContract exists in the active milestone."
                    .to_string(),
            ],
        };
    }

    WorkflowEligibilitySummary {
        ready_issue_count,
        eligible_issue_count,
        blocked_issue_count,
        next_action: "wait-human".to_string(),
        recommended_command: "agentflow eligibility".to_string(),
        rationale: vec![
            "No issue is currently eligible.".to_string(),
            "Inspect failure reasons before run; eligible is computed, not manually assigned."
                .to_string(),
        ],
    }
}

fn snapshot_active_milestone_intent(snapshot: &LocalProjectModelSnapshot) -> Option<String> {
    let active_project_id = snapshot
        .workspace
        .as_ref()
        .map(|workspace| workspace.active_project_id.as_str())?;
    let project = snapshot
        .projects
        .iter()
        .find(|project| project.id == active_project_id)?;
    project
        .milestones
        .iter()
        .find(|milestone| milestone.id == project.active_milestone_id)
        .and_then(|milestone| milestone.next_issue_intent.clone())
        .or_else(|| project.next_issue_intent.clone())
}

fn workflow_lease_snapshot(repo: &Path) -> Result<WorkflowLeaseSnapshot> {
    let project_dir = required_project_dir(repo)?;
    let leases = read_workflow_leases(&project_dir)?;
    let now = current_epoch_seconds();
    let mut active_leases = Vec::new();
    let mut stale_leases = Vec::new();
    for lease in leases {
        if lease.status == "active" && lease.expires_at_epoch_seconds <= now {
            stale_leases.push(lease);
        } else if lease.status == "active" {
            active_leases.push(lease);
        }
    }
    active_leases.sort_by(|left, right| left.id.cmp(&right.id));
    stale_leases.sort_by(|left, right| left.id.cmp(&right.id));

    Ok(WorkflowLeaseSnapshot {
        version: VERSION.to_string(),
        initialized: project_dir.exists(),
        project_root: repo.display().to_string(),
        active_leases,
        stale_leases,
        recommended_command: "agentflow eligibility".to_string(),
        boundary: WorkbenchBoundary {
            read_only: false,
            disallowed_actions: vec![
                "auto-recover-stale-lease".to_string(),
                "create-remote-issue".to_string(),
                "create-remote-pr".to_string(),
                "execute-model".to_string(),
            ],
        },
    })
}

fn project_closure_state_snapshot(repo: &Path) -> Result<ProjectClosureStateSnapshot> {
    let project_dir = required_project_dir(repo)?;
    let project_snapshot = read_workflow_control_project_snapshot(repo, &project_dir)?;
    let issues = read_issue_contracts(&project_dir)?;
    let runs = read_agent_runs(&project_dir)?;
    let evidence = read_text_artifacts(repo, &project_dir.join("evidence"))?;
    let reviews = read_text_artifacts(repo, &project_dir.join("reviews"))?;
    let project_updates = read_project_update_artifacts(repo, &project_dir.join("updates"))?;
    let active_project_id = project_snapshot
        .workspace
        .as_ref()
        .map(|workspace| workspace.active_project_id.clone())
        .or_else(|| {
            project_snapshot
                .projects
                .first()
                .map(|project| project.id.clone())
        });
    let active_project = active_project_id.as_ref().and_then(|project_id| {
        project_snapshot
            .projects
            .iter()
            .find(|project| &project.id == project_id)
    });

    let (project_status, active_milestone_id, project_milestones) =
        if let Some(project) = active_project {
            (
                Some(project.status.clone()),
                Some(project.active_milestone_id.clone()),
                project.milestones.clone(),
            )
        } else {
            (None, None, Vec::new())
        };

    let closure_relevant = project_milestones
        .iter()
        .filter(|milestone| !is_closure_milestone(milestone))
        .collect::<Vec<_>>();
    let completed_milestone_ids = closure_relevant
        .iter()
        .filter(|milestone| milestone.status == "completed")
        .map(|milestone| milestone.id.clone())
        .collect::<Vec<_>>();
    let incomplete_milestone_ids = closure_relevant
        .iter()
        .filter(|milestone| milestone.status != "completed")
        .map(|milestone| milestone.id.clone())
        .collect::<Vec<_>>();
    let missing_milestone_summary_ids = closure_relevant
        .iter()
        .filter(|milestone| !milestone.issue_ids.is_empty())
        .filter(|milestone| !milestone_summary_path(&project_dir, &milestone.id).exists())
        .map(|milestone| milestone.id.clone())
        .collect::<Vec<_>>();

    let all_milestones_completed = active_project.is_some() && incomplete_milestone_ids.is_empty();
    let milestone_summaries_exist = missing_milestone_summary_ids.is_empty();
    let code_audit_path = project_dir.join(format!(
        "audits/{}-code-audit.md",
        active_project_id.as_deref().unwrap_or("unknown-project")
    ));
    let code_audit_snapshot_path = project_code_audit_snapshot_path(&project_dir);
    let docs_refresh_path = project_dir.join(format!(
        "audits/{}-docs-refresh.md",
        active_project_id.as_deref().unwrap_or("unknown-project")
    ));
    let docs_refresh_snapshot_path = project_docs_refresh_snapshot_path(&project_dir);
    let final_summary_path = project_dir.join(format!(
        "evidence/PROJECT-{}-final-evidence-summary.md",
        active_project_id.as_deref().unwrap_or("unknown-project")
    ));
    let final_approval_path = project_dir.join(format!(
        "approvals/{}-final-approval.json",
        active_project_id.as_deref().unwrap_or("unknown-project")
    ));

    let code_audit_exists = code_audit_path.exists()
        || code_audit_path.with_extension("json").exists()
        || project_dir
            .join(format!(
                "audits/{}-code-audit.json",
                active_project_id.as_deref().unwrap_or("unknown-project")
            ))
            .exists();
    let code_audit_snapshot_exists = code_audit_snapshot_path.exists();
    let docs_refresh_exists = docs_refresh_path.exists()
        || docs_refresh_path.with_extension("json").exists()
        || project_dir
            .join(format!(
                "audits/{}-docs-refresh.json",
                active_project_id.as_deref().unwrap_or("unknown-project")
            ))
            .exists();
    let docs_refresh_snapshot_exists = docs_refresh_snapshot_path.exists();
    let final_summary_exists = final_summary_path.exists()
        || final_summary_path.with_extension("json").exists()
        || project_dir
            .join(format!(
                "evidence/PROJECT-{}-final-evidence-summary.json",
                active_project_id.as_deref().unwrap_or("unknown-project")
            ))
            .exists();
    let human_final_approval_exists = final_approval_path.exists();

    let mut gates = Vec::new();
    gates.push(project_closure_gate(
        "all-milestones-completed",
        "All milestones completed",
        all_milestones_completed,
        true,
        None,
        if all_milestones_completed {
            "All non-closure delivery milestones are completed.".to_string()
        } else {
            format!(
                "Incomplete milestones: {}",
                if incomplete_milestone_ids.is_empty() {
                    "none".to_string()
                } else {
                    incomplete_milestone_ids.join(", ")
                }
            )
        },
    ));
    gates.push(project_closure_gate(
        "milestone-evidence-summaries",
        "Milestone evidence summaries exist",
        milestone_summaries_exist,
        true,
        None,
        if milestone_summaries_exist {
            "Milestone summaries are present or no issue-backed milestone requires one.".to_string()
        } else {
            format!(
                "Missing milestone summaries: {}",
                missing_milestone_summary_ids.join(", ")
            )
        },
    ));
    let code_audit_gate_path = if code_audit_snapshot_exists && !code_audit_exists {
        &code_audit_snapshot_path
    } else {
        &code_audit_path
    };
    let code_audit_gate_status = if code_audit_exists {
        "pass"
    } else if code_audit_snapshot_exists {
        "snapshot-ready"
    } else {
        "blocked"
    };
    let code_audit_gate_detail = if code_audit_exists {
        "Final Code Audit artifact exists.".to_string()
    } else if code_audit_snapshot_exists {
        "Project Code Audit Snapshot exists as a read-only input package; final Code Audit approval remains missing.".to_string()
    } else {
        "Code Audit is a future closure artifact; run `agentflow project code-audit` to generate the read-only snapshot first.".to_string()
    };
    gates.push(project_closure_gate_with_status(
        "code-audit",
        "Code Audit exists",
        code_audit_gate_status,
        true,
        Some(agentflow_relative_path(&project_dir, code_audit_gate_path)),
        code_audit_gate_detail,
    ));
    let docs_refresh_gate_path = if docs_refresh_snapshot_exists && !docs_refresh_exists {
        &docs_refresh_snapshot_path
    } else {
        &docs_refresh_path
    };
    let docs_refresh_gate_status = if docs_refresh_exists {
        "pass"
    } else if docs_refresh_snapshot_exists {
        "snapshot-ready"
    } else {
        "blocked"
    };
    let docs_refresh_gate_detail = if docs_refresh_exists {
        "Final Root Docs Refresh artifact exists.".to_string()
    } else if docs_refresh_snapshot_exists {
        "Root Docs Refresh Snapshot exists as a read-only input package; final docs refresh approval remains missing.".to_string()
    } else {
        "Root Docs Refresh is a future closure artifact; run `agentflow project docs-refresh` to generate the read-only snapshot first.".to_string()
    };
    gates.push(project_closure_gate_with_status(
        "docs-refresh",
        "Root Docs Refresh exists",
        docs_refresh_gate_status,
        true,
        Some(agentflow_relative_path(
            &project_dir,
            docs_refresh_gate_path,
        )),
        docs_refresh_gate_detail,
    ));
    gates.push(project_closure_gate(
        "final-evidence-summary",
        "Final Evidence Summary exists",
        final_summary_exists,
        true,
        Some(agentflow_relative_path(&project_dir, &final_summary_path)),
        "Final Evidence Summary must be generated before Human Final Approval.".to_string(),
    ));
    gates.push(project_closure_gate(
        "human-final-approval",
        "Human Final Approval exists",
        human_final_approval_exists,
        true,
        Some(agentflow_relative_path(&project_dir, &final_approval_path)),
        "AgentFlow can prepare an approval packet, but cannot approve on behalf of the user."
            .to_string(),
    ));

    let done_blocked_reasons = gates
        .iter()
        .filter(|gate| gate.required && gate.status != "pass")
        .map(|gate| format!("{}: {}", gate.id, gate.detail))
        .collect::<Vec<_>>();
    let can_mark_done = done_blocked_reasons.is_empty();
    let closure_state = project_closure_state(
        project_status.as_deref(),
        all_milestones_completed,
        code_audit_exists,
        code_audit_snapshot_exists,
        docs_refresh_exists,
        final_summary_exists,
        human_final_approval_exists,
        can_mark_done,
    );
    let recommended_command = project_closure_recommended_command(&closure_state);

    Ok(ProjectClosureStateSnapshot {
        version: VERSION.to_string(),
        initialized: project_snapshot.initialized,
        project_root: repo.display().to_string(),
        active_project_id,
        project_status,
        active_milestone_id,
        closure_state,
        can_mark_done,
        counts: ProjectClosureCounts {
            milestones: closure_relevant.len(),
            completed_milestones: completed_milestone_ids.len(),
            issues: issues.len(),
            completed_issues: issues
                .iter()
                .filter(|issue| issue_state_done(&issue.status))
                .count(),
            runs: runs.len(),
            evidence_reports: evidence.len(),
            reviews: reviews.len(),
            project_updates: project_updates.len(),
        },
        gates,
        completed_milestone_ids,
        incomplete_milestone_ids,
        missing_milestone_summary_ids,
        done_blocked_reasons,
        recommended_command,
        sources: vec![
            ".agentflow/projects/{project-id}.json".to_string(),
            ".agentflow/issues/*.json".to_string(),
            ".agentflow/runs/*/run.json".to_string(),
            ".agentflow/evidence/*.md".to_string(),
            ".agentflow/reviews/*.md".to_string(),
            ".agentflow/updates/*.md".to_string(),
            "docs/specs/project-audit-docs-refresh-boundary.md".to_string(),
        ],
        boundary: WorkbenchBoundary {
            read_only: false,
            disallowed_actions: vec![
                "auto-generate-code-audit".to_string(),
                "auto-run-docs-refresh".to_string(),
                "mark-project-done".to_string(),
                "create-agentflow-audits-dir".to_string(),
                "call-model".to_string(),
                "create-remote-pr".to_string(),
                "create-remote-issue".to_string(),
                "modify-desktop-ui".to_string(),
            ],
        },
    })
}

fn is_closure_milestone(milestone: &LocalMilestone) -> bool {
    let text = format!("{} {}", milestone.id, milestone.name).to_lowercase();
    text.contains("closure") || text.contains("audit") || text.contains("docs-refresh")
}

fn milestone_summary_path(project_dir: &Path, milestone_id: &str) -> PathBuf {
    project_dir.join(format!(
        "evidence/MILESTONE-{milestone_id}-evidence-summary.md"
    ))
}

fn project_closure_gate(
    id: &str,
    name: &str,
    passed: bool,
    required: bool,
    path: Option<String>,
    detail: String,
) -> ProjectClosureGate {
    project_closure_gate_with_status(
        id,
        name,
        if passed { "pass" } else { "blocked" },
        required,
        path,
        detail,
    )
}

fn project_closure_gate_with_status(
    id: &str,
    name: &str,
    status: &str,
    required: bool,
    path: Option<String>,
    detail: String,
) -> ProjectClosureGate {
    ProjectClosureGate {
        id: id.to_string(),
        name: name.to_string(),
        status: status.to_string(),
        required,
        path,
        detail,
    }
}

fn agentflow_relative_path(project_dir: &Path, path: &Path) -> String {
    path.strip_prefix(project_dir)
        .map(|relative| format!(".agentflow/{}", relative.display()))
        .unwrap_or_else(|_| path.display().to_string())
}

fn project_closure_state(
    project_status: Option<&str>,
    all_milestones_completed: bool,
    code_audit_exists: bool,
    code_audit_snapshot_exists: bool,
    docs_refresh_exists: bool,
    final_summary_exists: bool,
    human_final_approval_exists: bool,
    can_mark_done: bool,
) -> String {
    if project_status == Some("done") && can_mark_done {
        return "done".to_string();
    }
    if project_status == Some("done") && !can_mark_done {
        return "done-blocked".to_string();
    }
    if !all_milestones_completed {
        return "active".to_string();
    }
    if !code_audit_exists {
        return if code_audit_snapshot_exists {
            "audit".to_string()
        } else {
            "audit-ready".to_string()
        };
    }
    if !docs_refresh_exists {
        return "docs-refresh".to_string();
    }
    if !final_summary_exists || !human_final_approval_exists {
        return "final-review".to_string();
    }
    if can_mark_done {
        "done-blocked".to_string()
    } else {
        "final-review".to_string()
    }
}

fn project_closure_recommended_command(closure_state: &str) -> String {
    match closure_state {
        "active" => "agentflow projects".to_string(),
        "audit-ready" => "agentflow project closure".to_string(),
        "audit" => "agentflow project closure".to_string(),
        "docs-refresh" => "agentflow project closure".to_string(),
        "final-review" => "agentflow project closure".to_string(),
        "done-blocked" => "agentflow project closure".to_string(),
        "done" => "agentflow projects".to_string(),
        _ => "agentflow project closure".to_string(),
    }
}

fn project_code_audit_snapshot_path(project_dir: &Path) -> PathBuf {
    project_dir.join("state/project-code-audit.json")
}

fn project_docs_refresh_snapshot_path(project_dir: &Path) -> PathBuf {
    project_dir.join("state/project-docs-refresh.json")
}

fn project_code_audit_snapshot(repo: &Path) -> Result<ProjectCodeAuditSnapshot> {
    let project_dir = required_project_dir(repo)?;
    let closure_state_path = project_dir.join("state/project-closure.json");
    let closure: ProjectClosureStateSnapshot = if closure_state_path.exists() {
        read_json(&closure_state_path)?
    } else {
        project_closure_state_snapshot(repo)?
    };
    let project_snapshot = read_workflow_control_project_snapshot(repo, &project_dir)?;
    let issues = read_issue_contracts(&project_dir)?;
    let runs = read_agent_runs(&project_dir)?;
    let evidence = read_text_artifacts(repo, &project_dir.join("evidence"))?;
    let reviews = read_text_artifacts(repo, &project_dir.join("reviews"))?;
    let project_updates = read_text_artifacts(repo, &project_dir.join("updates"))?;
    let source_docs = code_audit_source_documents(repo)?;
    let checks =
        project_code_audit_checks(&source_docs, &issues, &runs, &evidence, &reviews, &closure);
    let findings = checks
        .iter()
        .map(|check| check.findings.len())
        .sum::<usize>();
    let mut blockers = closure.done_blocked_reasons.clone();

    if closure.closure_state == "active" {
        blockers.push(
            "project_not_audit_ready: all non-closure milestones must complete before code audit."
                .to_string(),
        );
    }
    if !project_code_audit_final_artifact_exists(&project_dir, closure.active_project_id.as_deref())
    {
        blockers.push(
            "final_code_audit_not_passed: Project Code Audit Snapshot is read-only input, not final audit approval."
                .to_string(),
        );
    }
    blockers.sort();
    blockers.dedup();

    let audit_state = if closure.closure_state == "active" {
        "blocked"
    } else {
        "snapshot-ready"
    }
    .to_string();

    Ok(ProjectCodeAuditSnapshot {
        version: VERSION.to_string(),
        initialized: true,
        project_root: repo.display().to_string(),
        active_project_id: closure.active_project_id.clone(),
        active_milestone_id: closure.active_milestone_id.clone(),
        closure_state: closure.closure_state.clone(),
        audit_state,
        counts: ProjectCodeAuditCounts {
            projects: project_snapshot.projects.len(),
            milestones: closure.counts.milestones,
            issues: issues.len(),
            completed_issues: issues
                .iter()
                .filter(|issue| issue_state_done(&issue.status))
                .count(),
            runs: runs.len(),
            evidence_reports: evidence.len(),
            reviews: reviews.len(),
            project_updates: project_updates.len(),
            source_files: source_docs.len(),
            findings,
            blockers: blockers.len(),
        },
        checks,
        blockers,
        recommended_command: "agentflow project closure".to_string(),
        sources: vec![
            ".agentflow/state/project-closure.json".to_string(),
            ".agentflow/projects/{project-id}.json".to_string(),
            ".agentflow/issues/*.json".to_string(),
            ".agentflow/runs/*/run.json".to_string(),
            ".agentflow/evidence/*.md".to_string(),
            ".agentflow/reviews/*.md".to_string(),
            ".agentflow/updates/*.md".to_string(),
            "repo source tree read-only scan".to_string(),
            "docs/specs/project-audit-docs-refresh-boundary.md".to_string(),
        ],
        boundary: WorkbenchBoundary {
            read_only: true,
            disallowed_actions: vec![
                "create-agentflow-audits-dir".to_string(),
                "auto-fix-audit-findings".to_string(),
                "modify-code".to_string(),
                "modify-docs".to_string(),
                "mark-project-done".to_string(),
                "call-model".to_string(),
                "create-remote-pr".to_string(),
                "create-remote-issue".to_string(),
                "modify-desktop-ui".to_string(),
            ],
        },
    })
}

fn project_docs_refresh_snapshot(repo: &Path) -> Result<ProjectDocsRefreshSnapshot> {
    let project_dir = required_project_dir(repo)?;
    let closure_state_path = project_dir.join("state/project-closure.json");
    let closure: ProjectClosureStateSnapshot = if closure_state_path.exists() {
        read_json(&closure_state_path)?
    } else {
        project_closure_state_snapshot(repo)?
    };
    let code_audit_snapshot_exists = project_code_audit_snapshot_path(&project_dir).exists();
    let checked_docs = root_docs_refresh_checked_docs(repo)?;
    let required_updates = root_docs_refresh_required_updates(&checked_docs);
    let mut blockers = closure.done_blocked_reasons.clone();

    if closure.closure_state == "active" {
        blockers.push(
            "project_not_docs_refresh_ready: all non-closure milestones must complete before Root Docs Refresh."
                .to_string(),
        );
    }
    if !code_audit_snapshot_exists {
        blockers.push(
            "code_audit_snapshot_missing: run `agentflow project code-audit` before Root Docs Refresh Snapshot."
                .to_string(),
        );
    }
    if !required_updates.is_empty() {
        blockers.push(format!(
            "root_docs_updates_required: {} doc(s) need IssueContract-backed updates before final docs refresh.",
            required_updates.len()
        ));
    }
    if !project_docs_refresh_final_artifact_exists(
        &project_dir,
        closure.active_project_id.as_deref(),
    ) {
        blockers.push(
            "final_docs_refresh_not_passed: Root Docs Refresh Snapshot is read-only input, not final docs refresh approval."
                .to_string(),
        );
    }
    blockers.sort();
    blockers.dedup();

    let docs_refresh_state = if closure.closure_state == "active" || !code_audit_snapshot_exists {
        "blocked"
    } else {
        "snapshot-ready"
    }
    .to_string();

    Ok(ProjectDocsRefreshSnapshot {
        version: VERSION.to_string(),
        initialized: true,
        project_root: repo.display().to_string(),
        active_project_id: closure.active_project_id.clone(),
        active_milestone_id: closure.active_milestone_id.clone(),
        closure_state: closure.closure_state.clone(),
        docs_refresh_state,
        counts: ProjectDocsRefreshCounts {
            checked_docs: checked_docs.len(),
            current_docs: checked_docs
                .iter()
                .filter(|doc| doc.status == "current")
                .count(),
            update_needed_docs: checked_docs
                .iter()
                .filter(|doc| doc.status == "updated-needed")
                .count(),
            missing_docs: checked_docs
                .iter()
                .filter(|doc| doc.status == "missing")
                .count(),
            intentionally_absent_docs: checked_docs
                .iter()
                .filter(|doc| doc.status == "intentionally-absent")
                .count(),
            required_updates: required_updates.len(),
            blockers: blockers.len(),
        },
        checked_docs,
        required_updates,
        blockers,
        recommended_command: "agentflow project closure".to_string(),
        sources: vec![
            ".agentflow/state/project-closure.json".to_string(),
            ".agentflow/state/project-code-audit.json".to_string(),
            "README.md".to_string(),
            "ROADMAP.md".to_string(),
            "docs/specs/*.md".to_string(),
            "docs/contracts/*.md".to_string(),
            "docs/architecture/*.md".to_string(),
            "docs/validation/*.md".to_string(),
            "verification.md".to_string(),
            "docs/specs/project-audit-docs-refresh-boundary.md".to_string(),
        ],
        boundary: WorkbenchBoundary {
            read_only: true,
            disallowed_actions: vec![
                "create-agentflow-audits-dir".to_string(),
                "modify-code".to_string(),
                "modify-docs".to_string(),
                "refresh-docs".to_string(),
                "mark-project-done".to_string(),
                "call-model".to_string(),
                "create-remote-pr".to_string(),
                "create-remote-issue".to_string(),
                "modify-desktop-ui".to_string(),
            ],
        },
    })
}

fn project_code_audit_final_artifact_exists(
    project_dir: &Path,
    active_project_id: Option<&str>,
) -> bool {
    let project_id = active_project_id.unwrap_or("unknown-project");
    project_dir
        .join(format!("audits/{project_id}-code-audit.md"))
        .exists()
        || project_dir
            .join(format!("audits/{project_id}-code-audit.json"))
            .exists()
}

fn project_docs_refresh_final_artifact_exists(
    project_dir: &Path,
    active_project_id: Option<&str>,
) -> bool {
    let project_id = active_project_id.unwrap_or("unknown-project");
    project_dir
        .join(format!("audits/{project_id}-docs-refresh.md"))
        .exists()
        || project_dir
            .join(format!("audits/{project_id}-docs-refresh.json"))
            .exists()
}

fn root_docs_refresh_checked_docs(repo: &Path) -> Result<Vec<ProjectDocsRefreshCheckedDoc>> {
    root_docs_refresh_document_specs()
        .iter()
        .map(|spec| root_docs_refresh_checked_doc(repo, spec))
        .collect()
}

fn root_docs_refresh_checked_doc(
    repo: &Path,
    spec: &DocsRefreshDocumentSpec,
) -> Result<ProjectDocsRefreshCheckedDoc> {
    let path = repo.join(spec.path);
    if !path.exists() {
        return Ok(ProjectDocsRefreshCheckedDoc {
            path: spec.path.to_string(),
            category: spec.category.to_string(),
            status: if spec.required {
                "missing"
            } else {
                "intentionally-absent"
            }
            .to_string(),
            reason: if spec.required {
                "Required Root Docs Refresh input is missing.".to_string()
            } else {
                "Optional runbook / known limitations doc is not present in this MVP stage."
                    .to_string()
            },
            anchors: spec
                .anchors
                .iter()
                .map(|anchor| anchor.to_string())
                .collect(),
        });
    }

    let content = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let missing_anchors = spec
        .anchors
        .iter()
        .filter(|anchor| !content.contains(**anchor))
        .map(|anchor| anchor.to_string())
        .collect::<Vec<_>>();
    Ok(ProjectDocsRefreshCheckedDoc {
        path: spec.path.to_string(),
        category: spec.category.to_string(),
        status: if missing_anchors.is_empty() {
            "current"
        } else {
            "updated-needed"
        }
        .to_string(),
        reason: if missing_anchors.is_empty() {
            "Required anchors are present for the read-only docs refresh snapshot.".to_string()
        } else {
            format!("Missing anchors: {}", missing_anchors.join(", "))
        },
        anchors: spec
            .anchors
            .iter()
            .map(|anchor| anchor.to_string())
            .collect(),
    })
}

fn root_docs_refresh_required_updates(
    checked_docs: &[ProjectDocsRefreshCheckedDoc],
) -> Vec<ProjectDocsRefreshRequiredUpdate> {
    checked_docs
        .iter()
        .filter(|doc| doc.status == "missing" || doc.status == "updated-needed")
        .map(|doc| ProjectDocsRefreshRequiredUpdate {
            path: doc.path.clone(),
            summary: if doc.status == "missing" {
                format!("Create required root documentation input `{}`.", doc.path)
            } else {
                format!(
                    "Refresh `{}` so it reflects the current workflow state.",
                    doc.path
                )
            },
            follow_up_issue_intent: format!("Root Docs Refresh follow-up for {}", doc.path),
            severity: if doc.status == "missing" {
                "high"
            } else {
                "medium"
            }
            .to_string(),
        })
        .collect()
}

fn root_docs_refresh_document_specs() -> Vec<DocsRefreshDocumentSpec> {
    vec![
        DocsRefreshDocumentSpec {
            path: "README.md",
            category: "root-doc",
            required: true,
            anchors: &[
                "Root Docs Refresh Snapshot v0",
                "Project Code Audit Snapshot v0",
            ],
        },
        DocsRefreshDocumentSpec {
            path: "ROADMAP.md",
            category: "root-doc",
            required: true,
            anchors: &[
                "Root Docs Refresh Snapshot v0",
                "Project Final Evidence Summary",
            ],
        },
        DocsRefreshDocumentSpec {
            path: "docs/specs/mvp-spec.md",
            category: "spec",
            required: true,
            anchors: &[
                "Root Docs Refresh Snapshot v0",
                "Project Code Audit Snapshot v0",
            ],
        },
        DocsRefreshDocumentSpec {
            path: "docs/specs/project-audit-docs-refresh-boundary.md",
            category: "spec",
            required: true,
            anchors: &["Root Docs Refresh Snapshot v0", "Root Docs Refresh"],
        },
        DocsRefreshDocumentSpec {
            path: "docs/specs/workflow-control-core-v0.md",
            category: "spec",
            required: true,
            anchors: &[
                "Project Audit / Docs Refresh v0",
                "Project Closure State v0",
            ],
        },
        DocsRefreshDocumentSpec {
            path: "docs/contracts/agentflow-ai-delivery-workflow-contract-v1.md",
            category: "contract",
            required: true,
            anchors: &["Project Audit / Docs Refresh", "Root Docs Refresh"],
        },
        DocsRefreshDocumentSpec {
            path: "docs/architecture/architecture.md",
            category: "architecture",
            required: true,
            anchors: &["AgentFlow", "本地"],
        },
        DocsRefreshDocumentSpec {
            path: "docs/architecture/architecture-decisions.md",
            category: "architecture",
            required: true,
            anchors: &["AgentFlow", "决策"],
        },
        DocsRefreshDocumentSpec {
            path: "docs/planning/construction-plan.md",
            category: "planning",
            required: true,
            anchors: &[
                "Root Docs Refresh Snapshot v0",
                "Project Code Audit Snapshot v0",
            ],
        },
        DocsRefreshDocumentSpec {
            path: "docs/planning/mvp-productization-project.md",
            category: "planning",
            required: true,
            anchors: &[
                "Root Docs Refresh Snapshot v0",
                "Project Code Audit Snapshot v0",
            ],
        },
        DocsRefreshDocumentSpec {
            path: "docs/validation/latest-verification-summary.md",
            category: "validation",
            required: true,
            anchors: &[
                "Root Docs Refresh Snapshot v0",
                "Project Code Audit Snapshot v0",
            ],
        },
        DocsRefreshDocumentSpec {
            path: "verification.md",
            category: "validation",
            required: true,
            anchors: &[
                "Root Docs Refresh Snapshot v0",
                "Project Code Audit Snapshot v0",
            ],
        },
        DocsRefreshDocumentSpec {
            path: "docs/runbook.md",
            category: "runbook",
            required: false,
            anchors: &[],
        },
        DocsRefreshDocumentSpec {
            path: "docs/known-limitations.md",
            category: "known-limitations",
            required: false,
            anchors: &[],
        },
    ]
}

fn project_code_audit_checks(
    source_docs: &[CodeAuditSourceDocument],
    issues: &[IssueContract],
    runs: &[AgentRun],
    evidence: &[WorkbenchTextArtifact],
    reviews: &[WorkbenchTextArtifact],
    closure: &ProjectClosureStateSnapshot,
) -> Vec<ProjectCodeAuditCheck> {
    vec![
        project_code_audit_check(
            "duplicate-code-candidate",
            "Duplicate code candidate",
            duplicate_code_findings(source_docs),
            "No repeated long source lines were found by the read-only heuristic.",
        ),
        project_code_audit_check(
            "temporary-code-candidate",
            "Temporary code candidate",
            keyword_findings(
                source_docs,
                "temporary-code-candidate",
                &["temporary", "temp ", "hack", "workaround", "临时"],
                "medium",
                "Temporary or workaround wording requires review before final audit.",
                24,
            ),
            "No temporary-code markers were found by the read-only heuristic.",
        ),
        project_code_audit_check(
            "unused-dead-code-candidate",
            "Unused / dead code candidate",
            keyword_findings(
                source_docs,
                "unused-dead-code-candidate",
                &["allow(dead_code", "dead_code", "allow(unused", "unused", "todo!(", "unimplemented!("],
                "medium",
                "Unused, dead-code, todo, or unimplemented markers require audit review.",
                24,
            ),
            "No unused/dead-code markers were found by the read-only heuristic.",
        ),
        project_code_audit_check(
            "todo-fixme-candidate",
            "TODO / FIXME candidate",
            keyword_findings(
                source_docs,
                "todo-fixme-candidate",
                &["todo", "fixme", "xxx"],
                "medium",
                "TODO/FIXME markers require closure review or explicit deferral.",
                24,
            ),
            "No TODO/FIXME markers were found by the read-only heuristic.",
        ),
        project_code_audit_check(
            "security-auth-permission-risk-candidate",
            "Security / auth / permission risk candidate",
            keyword_findings(
                source_docs,
                "security-auth-permission-risk-candidate",
                &["password", "secret", "token", "api_key", "credential", "auth", "permission", "sandbox"],
                "high",
                "Sensitive boundary wording requires audit review; this snapshot does not classify it as a vulnerability.",
                24,
            ),
            "No security/auth/permission markers were found by the read-only heuristic.",
        ),
        project_code_audit_check(
            "performance-risk-candidate",
            "Performance risk candidate",
            keyword_findings(
                source_docs,
                "performance-risk-candidate",
                &["o(n^2)", "performance", "perf", "slow", "cache", "large file"],
                "medium",
                "Performance-related wording requires audit review before final closure.",
                24,
            ),
            "No performance-risk markers were found by the read-only heuristic.",
        ),
        project_code_audit_check(
            "architecture-drift-candidate",
            "Architecture drift candidate",
            architecture_drift_findings(source_docs),
            "No architecture drift candidates were found by the read-only heuristic.",
        ),
        project_code_audit_check(
            "test-gap-candidate",
            "Test gap candidate",
            test_gap_findings(issues, runs, evidence, reviews, closure),
            "Completed issues have run, evidence, review, and milestone-summary coverage by the read-only heuristic.",
        ),
        project_code_audit_check(
            "unexpected-public-api-change-candidate",
            "Unexpected public API change candidate",
            public_api_findings(source_docs),
            "No public API/export markers were found by the read-only heuristic.",
        ),
    ]
}

fn project_code_audit_check(
    id: &str,
    name: &str,
    findings: Vec<ProjectCodeAuditFinding>,
    clear_detail: &str,
) -> ProjectCodeAuditCheck {
    let candidate_count = findings.len();
    ProjectCodeAuditCheck {
        id: id.to_string(),
        name: name.to_string(),
        status: if candidate_count == 0 {
            "clear"
        } else {
            "candidate"
        }
        .to_string(),
        candidate_count,
        detail: if candidate_count == 0 {
            clear_detail.to_string()
        } else {
            format!("{candidate_count} candidate(s) require human audit review.")
        },
        findings,
    }
}

fn duplicate_code_findings(
    source_docs: &[CodeAuditSourceDocument],
) -> Vec<ProjectCodeAuditFinding> {
    let mut repeated: HashMap<String, Vec<CodeAuditLineHit>> = HashMap::new();
    for document in source_docs {
        if document.path.ends_with(".md") || document.path.ends_with(".json") {
            continue;
        }
        for (index, line) in document.content.lines().enumerate() {
            let normalized = line.split_whitespace().collect::<Vec<_>>().join(" ");
            if normalized.len() < 80 || normalized.starts_with("//") || normalized.starts_with('#')
            {
                continue;
            }
            repeated
                .entry(normalized.clone())
                .or_default()
                .push(CodeAuditLineHit {
                    path: document.path.clone(),
                    line: index + 1,
                    snippet: audit_snippet(&normalized),
                });
        }
    }

    let mut findings = Vec::new();
    for (line, hits) in repeated {
        let mut unique_paths = hits.iter().map(|hit| hit.path.clone()).collect::<Vec<_>>();
        unique_paths.sort();
        unique_paths.dedup();
        if hits.len() < 2 || unique_paths.len() < 2 {
            continue;
        }
        if let Some(hit) = hits.first() {
            findings.push(ProjectCodeAuditFinding {
                id: format!("duplicate-code-candidate-{}", findings.len() + 1),
                path: Some(hit.path.clone()),
                line: Some(hit.line),
                severity: "medium".to_string(),
                snippet: hit.snippet.clone(),
                detail: format!(
                    "Repeated long source line appears {} times across {} files: {}",
                    hits.len(),
                    unique_paths.len(),
                    audit_snippet(&line)
                ),
            });
        }
        if findings.len() >= 24 {
            break;
        }
    }
    findings
}

fn keyword_findings(
    source_docs: &[CodeAuditSourceDocument],
    check_id: &str,
    keywords: &[&str],
    severity: &str,
    detail: &str,
    limit: usize,
) -> Vec<ProjectCodeAuditFinding> {
    let mut findings = Vec::new();
    for document in source_docs {
        for (index, line) in document.content.lines().enumerate() {
            let line_lower = line.to_lowercase();
            if !keywords
                .iter()
                .any(|keyword| line_lower.contains(&keyword.to_lowercase()))
            {
                continue;
            }
            findings.push(ProjectCodeAuditFinding {
                id: format!("{check_id}-{}", findings.len() + 1),
                path: Some(document.path.clone()),
                line: Some(index + 1),
                severity: severity.to_string(),
                snippet: audit_snippet(line),
                detail: detail.to_string(),
            });
            if findings.len() >= limit {
                return findings;
            }
        }
    }
    findings
}

fn architecture_drift_findings(
    source_docs: &[CodeAuditSourceDocument],
) -> Vec<ProjectCodeAuditFinding> {
    let mut findings = Vec::new();
    for document in source_docs {
        let line_count = document.content.lines().count();
        if line_count >= 2_000 {
            findings.push(ProjectCodeAuditFinding {
                id: format!("architecture-drift-candidate-{}", findings.len() + 1),
                path: Some(document.path.clone()),
                line: None,
                severity: "medium".to_string(),
                snippet: format!("{line_count} lines"),
                detail:
                    "Large source file may indicate architecture drift; review before final audit."
                        .to_string(),
            });
        }
        if document.path.ends_with("crates/agentflow-core/src/lib.rs") {
            findings.push(ProjectCodeAuditFinding {
                id: format!("architecture-drift-candidate-{}", findings.len() + 1),
                path: Some(document.path.clone()),
                line: None,
                severity: "medium".to_string(),
                snippet: "core workflow logic is concentrated in agentflow-core/src/lib.rs"
                    .to_string(),
                detail: "Centralized workflow core should be reviewed before Project closure."
                    .to_string(),
            });
        }
        if findings.len() >= 12 {
            break;
        }
    }
    findings
}

fn test_gap_findings(
    issues: &[IssueContract],
    runs: &[AgentRun],
    evidence: &[WorkbenchTextArtifact],
    reviews: &[WorkbenchTextArtifact],
    closure: &ProjectClosureStateSnapshot,
) -> Vec<ProjectCodeAuditFinding> {
    let mut findings = Vec::new();
    for issue in issues
        .iter()
        .filter(|issue| issue_state_done(&issue.status))
    {
        if !runs.iter().any(|run| run.issue_id == issue.id) {
            findings.push(ProjectCodeAuditFinding {
                id: format!("test-gap-candidate-{}", findings.len() + 1),
                path: Some(format!(".agentflow/issues/{}.json", issue.id)),
                line: None,
                severity: "high".to_string(),
                snippet: issue.title.clone(),
                detail: "Completed issue has no recorded execution run.".to_string(),
            });
        }
        if !artifact_mentions(evidence, &issue.id) {
            findings.push(ProjectCodeAuditFinding {
                id: format!("test-gap-candidate-{}", findings.len() + 1),
                path: Some(format!(".agentflow/issues/{}.json", issue.id)),
                line: None,
                severity: "high".to_string(),
                snippet: issue.title.clone(),
                detail: "Completed issue has no evidence artifact mentioning the issue id."
                    .to_string(),
            });
        }
        if !artifact_mentions(reviews, &issue.id) {
            findings.push(ProjectCodeAuditFinding {
                id: format!("test-gap-candidate-{}", findings.len() + 1),
                path: Some(format!(".agentflow/issues/{}.json", issue.id)),
                line: None,
                severity: "medium".to_string(),
                snippet: issue.title.clone(),
                detail: "Completed issue has no review artifact mentioning the issue id."
                    .to_string(),
            });
        }
        if findings.len() >= 24 {
            return findings;
        }
    }
    for milestone_id in &closure.missing_milestone_summary_ids {
        findings.push(ProjectCodeAuditFinding {
            id: format!("test-gap-candidate-{}", findings.len() + 1),
            path: Some(format!(
                ".agentflow/evidence/MILESTONE-{milestone_id}-evidence-summary.md"
            )),
            line: None,
            severity: "high".to_string(),
            snippet: milestone_id.clone(),
            detail: "Milestone evidence summary is missing before Project closure.".to_string(),
        });
        if findings.len() >= 24 {
            break;
        }
    }
    findings
}

fn public_api_findings(source_docs: &[CodeAuditSourceDocument]) -> Vec<ProjectCodeAuditFinding> {
    let mut findings = Vec::new();
    for document in source_docs {
        if !is_public_api_source(&document.path) {
            continue;
        }
        for (index, line) in document.content.lines().enumerate() {
            let trimmed = line.trim_start();
            let is_public_marker = trimmed.starts_with("pub struct ")
                || trimmed.starts_with("pub enum ")
                || trimmed.starts_with("pub fn ")
                || trimmed.starts_with("pub const ")
                || trimmed.starts_with("export interface ")
                || trimmed.starts_with("export type ")
                || trimmed.starts_with("export function ")
                || trimmed.starts_with("export const ");
            if !is_public_marker {
                continue;
            }
            findings.push(ProjectCodeAuditFinding {
                id: format!("unexpected-public-api-change-candidate-{}", findings.len() + 1),
                path: Some(document.path.clone()),
                line: Some(index + 1),
                severity: "medium".to_string(),
                snippet: audit_snippet(trimmed),
                detail: "Public API/export marker should be reviewed against the workflow contract before closure."
                    .to_string(),
            });
            if findings.len() >= 24 {
                return findings;
            }
        }
    }
    findings
}

fn artifact_mentions(artifacts: &[WorkbenchTextArtifact], needle: &str) -> bool {
    artifacts
        .iter()
        .any(|artifact| artifact.path.contains(needle) || artifact.content.contains(needle))
}

fn code_audit_source_documents(repo: &Path) -> Result<Vec<CodeAuditSourceDocument>> {
    let mut documents = Vec::new();
    collect_code_audit_source_documents(repo, repo, &mut documents)?;
    documents.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(documents)
}

fn collect_code_audit_source_documents(
    repo: &Path,
    dir: &Path,
    documents: &mut Vec<CodeAuditSourceDocument>,
) -> Result<()> {
    for entry in fs::read_dir(dir).with_context(|| format!("read {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        let file_name = entry.file_name();
        let name = file_name.to_string_lossy();
        let relative = relative_path(repo, &path)?;
        if path.is_dir() {
            if should_skip_code_audit_dir(&relative, &name) {
                continue;
            }
            collect_code_audit_source_documents(repo, &path, documents)?;
            continue;
        }
        if !path.is_file() || should_skip_code_audit_file(&relative, &name) {
            continue;
        }
        let metadata = fs::metadata(&path)?;
        if metadata.len() > 512 * 1024 || !code_audit_text_file(&relative) {
            continue;
        }
        let Ok(content) = fs::read_to_string(&path) else {
            continue;
        };
        documents.push(CodeAuditSourceDocument {
            path: relative,
            content,
        });
    }
    Ok(())
}

fn should_skip_code_audit_dir(relative: &str, name: &str) -> bool {
    matches!(
        name,
        ".git" | ".agentflow" | "target" | "node_modules" | ".next" | "dist" | "build" | "gen"
    ) || relative == "apps/desktop/dist"
        || relative.ends_with("/target")
}

fn should_skip_code_audit_file(relative: &str, name: &str) -> bool {
    name == ".DS_Store"
        || name.starts_with(".env")
        || matches!(
            name,
            "Cargo.lock" | "package-lock.json" | "pnpm-lock.yaml" | "yarn.lock"
        )
        || relative.starts_with(".agentflow/")
        || relative.starts_with(".git/")
        || relative.starts_with("target/")
        || relative.contains("/node_modules/")
        || relative.contains("/dist/")
}

fn code_audit_text_file(relative: &str) -> bool {
    let Some(extension) = Path::new(relative)
        .extension()
        .and_then(|extension| extension.to_str())
    else {
        return false;
    };
    matches!(
        extension,
        "rs" | "ts"
            | "tsx"
            | "js"
            | "jsx"
            | "css"
            | "html"
            | "toml"
            | "json"
            | "sh"
            | "yaml"
            | "yml"
    )
}

fn is_public_api_source(path: &str) -> bool {
    path.ends_with(".rs")
        || path.ends_with(".ts")
        || path.ends_with(".tsx")
        || path.ends_with(".js")
        || path.ends_with(".jsx")
}

fn audit_snippet(line: &str) -> String {
    let trimmed = line.trim();
    if trimmed.chars().count() <= 160 {
        trimmed.to_string()
    } else {
        format!("{}...", trimmed.chars().take(160).collect::<String>())
    }
}

fn acquire_workflow_lease(
    repo: &Path,
    issue: &IssueContract,
    owner_agent_id: &str,
) -> Result<WorkflowLeaseRecord> {
    let project_dir = required_project_dir(repo)?;
    let eligibility = workflow_eligibility_snapshot(repo, Some(&issue.id))?;
    let candidate = eligibility
        .candidates
        .first()
        .ok_or_else(|| anyhow!("{} has no eligibility candidate", issue.id))?;
    if !candidate.eligible {
        bail!(
            "{} is not eligible: {}",
            issue.id,
            candidate.failure_reasons.join(", ")
        );
    }

    let now = current_epoch_seconds();
    for lease in read_workflow_leases(&project_dir)? {
        if lease.status != "active" {
            continue;
        }
        if lease.expires_at_epoch_seconds <= now {
            if lease.issue_id == issue.id {
                bail!(
                    "{} has stale lease {}; run `agentflow lease` and resolve manually",
                    issue.id,
                    lease.id
                );
            }
            continue;
        }
        if lease.issue_id == issue.id {
            return Ok(lease);
        }
        if lease.project_id.as_deref() == candidate.project_id.as_deref()
            && lease.project_id.is_some()
            && candidate.code_changing
        {
            bail!(
                "active lease {} on {} blocks code-changing issue {}",
                lease.id,
                lease.issue_id,
                issue.id
            );
        }
    }

    let lease = WorkflowLeaseRecord {
        version: VERSION.to_string(),
        id: format!("LEASE-{}-{now}", issue.id),
        issue_id: issue.id.clone(),
        project_id: candidate.project_id.clone(),
        milestone_id: candidate.milestone_id.clone(),
        owner_agent_id: owner_agent_id.to_string(),
        status: "active".to_string(),
        leased_at_epoch_seconds: now,
        expires_at_epoch_seconds: now + 45 * 60,
        released_at_epoch_seconds: None,
        stale_recovery_reason: None,
    };
    let leases_dir = project_dir.join("leases");
    ensure_dir(&leases_dir)?;
    write_json(
        &leases_dir.join(format!("{}.json", lease.id)),
        &lease,
        false,
    )?;
    Ok(lease)
}

fn release_workflow_lease_after_review(project_dir: &Path, run: &AgentRun) -> Result<()> {
    let Some(lease_id) = run.lease_id.as_deref() else {
        return Ok(());
    };
    release_workflow_lease_by_id(project_dir, lease_id, "review-completed")
}

fn release_workflow_lease_by_id(project_dir: &Path, lease_id: &str, reason: &str) -> Result<()> {
    let lease_path = project_dir.join("leases").join(format!("{lease_id}.json"));
    if !lease_path.exists() {
        return Ok(());
    }
    let mut lease: WorkflowLeaseRecord = read_json(&lease_path)?;
    lease.status = "released".to_string();
    lease.released_at_epoch_seconds = Some(current_epoch_seconds());
    lease.stale_recovery_reason = Some(reason.to_string());
    write_json(&lease_path, &lease, true)
}

fn read_workflow_leases(project_dir: &Path) -> Result<Vec<WorkflowLeaseRecord>> {
    let leases_dir = project_dir.join("leases");
    if !leases_dir.exists() {
        return Ok(Vec::new());
    }
    let mut leases: Vec<WorkflowLeaseRecord> = Vec::new();
    for entry in
        fs::read_dir(&leases_dir).with_context(|| format!("read {}", leases_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        leases.push(read_json(&path)?);
    }
    leases.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(leases)
}

fn current_epoch_seconds() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn validate_workflow_project(
    project: &LocalProject,
    issues: &[IssueContract],
    checks: &mut Vec<WorkflowStateCheck>,
) {
    checks.push(workflow_state_check(
        "project-status-known",
        "project",
        &project.id,
        project_state_known(&project.status),
        format!("Project status `{}` is recognized.", project.status),
        format!("Project status `{}` is not recognized.", project.status),
        "error",
    ));

    let active_milestones = project
        .milestones
        .iter()
        .filter(|milestone| milestone_state_active(&milestone.status))
        .collect::<Vec<_>>();
    checks.push(workflow_state_check(
        "project-single-active-milestone",
        "project",
        &project.id,
        active_milestones.len() <= 1,
        "Project has at most one active milestone.",
        format!(
            "Project has {} active milestones; MVP allows only one.",
            active_milestones.len()
        ),
        "error",
    ));

    let active_milestone_exists = project
        .milestones
        .iter()
        .any(|milestone| milestone.id == project.active_milestone_id);
    checks.push(workflow_state_check(
        "project-active-milestone-exists",
        "project",
        &project.id,
        project.active_milestone_id.is_empty() || active_milestone_exists,
        format!(
            "Project active milestone `{}` exists.",
            project.active_milestone_id
        ),
        format!(
            "Project active milestone `{}` is not present in milestones.",
            project.active_milestone_id
        ),
        "error",
    ));

    if project_state_done(&project.status) {
        checks.push(workflow_state_check(
            "project-done-all-milestones-done",
            "project",
            &project.id,
            project
                .milestones
                .iter()
                .all(|milestone| milestone_state_done(&milestone.status)),
            "Done project has all milestones done.",
            "Done project still has non-done milestones.",
            "error",
        ));
    }

    let mut active_open_issues = 0usize;
    for milestone in &project.milestones {
        validate_workflow_milestone(project, milestone, issues, checks);
        if milestone_state_active(&milestone.status) {
            active_open_issues += milestone
                .issue_ids
                .iter()
                .filter(|issue_id| {
                    issues
                        .iter()
                        .find(|issue| &issue.id == *issue_id)
                        .is_some_and(|issue| !issue_state_done(&issue.status))
                })
                .count();
        }
    }

    checks.push(workflow_state_check(
        "project-single-open-active-milestone-issue",
        "project",
        &project.id,
        active_open_issues <= 1,
        "Active milestone has at most one open issue.",
        format!("Active milestone has {active_open_issues} open issues; WIP=1 is required."),
        "error",
    ));
}

fn validate_workflow_milestone(
    project: &LocalProject,
    milestone: &LocalMilestone,
    issues: &[IssueContract],
    checks: &mut Vec<WorkflowStateCheck>,
) {
    checks.push(workflow_state_check(
        "milestone-status-known",
        "milestone",
        &milestone.id,
        milestone_state_known(&milestone.status),
        format!("Milestone status `{}` is recognized.", milestone.status),
        format!("Milestone status `{}` is not recognized.", milestone.status),
        "error",
    ));

    let completed_subset = milestone
        .completed_issue_ids
        .iter()
        .all(|issue_id| milestone.issue_ids.iter().any(|id| id == issue_id));
    checks.push(workflow_state_check(
        "milestone-completed-issues-subset",
        "milestone",
        &milestone.id,
        completed_subset,
        "Milestone completedIssueIds are included in issueIds.",
        "Milestone completedIssueIds contains an issue outside issueIds.",
        "error",
    ));

    if milestone_state_done(&milestone.status) {
        let all_issues_done = milestone.issue_ids.iter().all(|issue_id| {
            issues
                .iter()
                .find(|issue| &issue.id == issue_id)
                .is_some_and(|issue| issue_state_done(&issue.status))
        });
        checks.push(workflow_state_check(
            "milestone-done-all-issues-done",
            "milestone",
            &milestone.id,
            all_issues_done,
            "Done milestone has only done issues.",
            "Done milestone contains an open or missing issue.",
            "error",
        ));
    }

    if milestone_state_active(&milestone.status) {
        checks.push(workflow_state_check(
            "milestone-active-project-link",
            "milestone",
            &milestone.id,
            project.active_milestone_id == milestone.id,
            "Active milestone matches project activeMilestoneId.",
            "Milestone is active but does not match project activeMilestoneId.",
            "error",
        ));
    }
}

fn validate_workflow_issue(
    project_dir: &Path,
    snapshot: &LocalProjectModelSnapshot,
    issue: &IssueContract,
    checks: &mut Vec<WorkflowStateCheck>,
) {
    checks.push(workflow_state_check(
        "issue-status-known",
        "issue",
        &issue.id,
        issue_state_known(&issue.status),
        format!("Issue status `{}` is recognized.", issue.status),
        format!("Issue status `{}` is not recognized.", issue.status),
        "error",
    ));

    checks.push(workflow_state_check(
        "issue-contract-complete",
        "issue",
        &issue.id,
        !issue.scope.is_empty()
            && !issue.non_goals.is_empty()
            && !issue.validation.commands.is_empty()
            && !issue.evidence_requirements.is_empty(),
        "Issue contract contains scope, non-goals, validation, and evidence requirements.",
        "Issue contract is missing scope, non-goals, validation, or evidence requirements.",
        "error",
    ));

    let Some(project_link) = issue.project_link.as_ref() else {
        checks.push(workflow_state_check(
            "issue-project-link",
            "issue",
            &issue.id,
            false,
            "Issue has projectLink.",
            "Issue is missing projectLink; workflow state cannot place it in Project / Milestone.",
            "warning",
        ));
        return;
    };

    let linked_project = snapshot
        .projects
        .iter()
        .find(|project| project.id == project_link.project_id);
    let linked_milestone = linked_project.and_then(|project| {
        project
            .milestones
            .iter()
            .find(|milestone| milestone.id == project_link.milestone_id)
    });
    checks.push(workflow_state_check(
        "issue-project-link-target",
        "issue",
        &issue.id,
        linked_project.is_some() && linked_milestone.is_some(),
        "Issue projectLink points to an existing project and milestone.",
        "Issue projectLink points to a missing project or milestone.",
        "error",
    ));

    if let Some(milestone) = linked_milestone {
        checks.push(workflow_state_check(
            "issue-listed-in-linked-milestone",
            "issue",
            &issue.id,
            milestone
                .issue_ids
                .iter()
                .any(|issue_id| issue_id == &issue.id),
            "Issue is listed in its linked milestone.",
            "Issue projectLink milestone does not include this issue id.",
            "error",
        ));
        checks.push(workflow_state_check(
            "issue-open-not-in-done-milestone",
            "issue",
            &issue.id,
            issue_state_done(&issue.status) || !milestone_state_done(&milestone.status),
            "Open issue is not attached to a done milestone.",
            "Open issue is attached to a done milestone.",
            "error",
        ));
    }

    if issue_state_done(&issue.status) {
        let evidence_path = project_dir.join(format!("evidence/{}-evidence.md", issue.id));
        let review_path = project_dir.join(format!("reviews/{}-review.md", issue.id));
        checks.push(workflow_state_check(
            "issue-done-evidence",
            "issue",
            &issue.id,
            evidence_path.exists() && review_path.exists(),
            "Done issue has evidence and review artifacts.",
            "Done issue is missing evidence or review artifacts.",
            "error",
        ));
    }
}

fn workflow_state_check(
    id: &str,
    entity_kind: &str,
    entity_id: &str,
    passed: bool,
    pass_message: impl Into<String>,
    fail_message: impl Into<String>,
    severity: &str,
) -> WorkflowStateCheck {
    WorkflowStateCheck {
        id: id.to_string(),
        entity_kind: entity_kind.to_string(),
        entity_id: entity_id.to_string(),
        severity: severity.to_string(),
        status: if passed { "pass" } else { "fail" }.to_string(),
        message: if passed {
            pass_message.into()
        } else {
            fail_message.into()
        },
    }
}

fn project_state_known(status: &str) -> bool {
    matches!(
        status,
        "draft"
            | "confirmed"
            | "active"
            | "audit"
            | "docs-refresh"
            | "final-review"
            | "done"
            | "blocked"
            | "paused"
            | "canceled"
            | "cancelled"
            | "failed"
            | "completed"
    )
}

fn project_state_done(status: &str) -> bool {
    canonical_project_status(status) == ProjectStatus::Completed
}

fn milestone_state_known(status: &str) -> bool {
    matches!(
        status,
        "draft"
            | "ready"
            | "active"
            | "review"
            | "done"
            | "blocked"
            | "paused"
            | "canceled"
            | "cancelled"
            | "failed"
            | "planned"
            | "completed"
    )
}

fn milestone_state_active(status: &str) -> bool {
    status == "active"
}

fn milestone_state_done(status: &str) -> bool {
    matches!(status, "done" | "completed")
}

fn issue_state_known(status: &str) -> bool {
    matches!(
        status,
        "draft"
            | "ready"
            | "eligible"
            | "leased"
            | "in-progress"
            | "pr"
            | "checks-passing"
            | "merged"
            | "evidence-captured"
            | "done"
            | "blocked"
            | "failed"
            | "canceled"
            | "cancelled"
            | "needs-human-review"
            | "planned"
            | "active"
            | "todo"
            | "in_progress"
            | "in_review"
            | "completed"
    )
}

fn issue_state_done(status: &str) -> bool {
    canonical_issue_status(status) == IssueStatus::Done
}

pub fn canonical_project_status(status: &str) -> ProjectStatus {
    match normalized_status(status).as_str() {
        "active" | "audit" | "docs_refresh" | "final_review" => ProjectStatus::Active,
        "paused" | "blocked" | "failed" => ProjectStatus::Paused,
        "completed" | "done" => ProjectStatus::Completed,
        "canceled" | "cancelled" => ProjectStatus::Canceled,
        "draft" | "planned" | "ready" | "confirmed" => ProjectStatus::Draft,
        _ => ProjectStatus::Draft,
    }
}

pub fn canonical_project_status_string(status: &str) -> String {
    canonical_project_status(status).as_str().to_string()
}

pub fn canonical_issue_status(status: &str) -> IssueStatus {
    match normalized_status(status).as_str() {
        "backlog" | "blocked" | "draft" | "failed" => IssueStatus::Backlog,
        "todo" | "planned" | "ready" => IssueStatus::Todo,
        "in_progress" | "active" | "eligible" | "leased" => IssueStatus::InProgress,
        "in_review" | "review" | "pr" | "checks_passing" | "merged" | "evidence_captured"
        | "needs_human_review" => IssueStatus::InReview,
        "done" | "completed" => IssueStatus::Done,
        "canceled" | "cancelled" => IssueStatus::Canceled,
        _ => IssueStatus::Backlog,
    }
}

pub fn canonical_issue_status_string(status: &str) -> String {
    canonical_issue_status(status).as_str().to_string()
}

fn issue_status_can_enter_execution(status: &str) -> bool {
    matches!(
        canonical_issue_status(status),
        IssueStatus::Todo | IssueStatus::InProgress
    )
}

fn issue_status_open(status: &str) -> bool {
    !matches!(
        canonical_issue_status(status),
        IssueStatus::Done | IssueStatus::Canceled
    )
}

fn normalized_status(status: &str) -> String {
    status.trim().to_ascii_lowercase().replace('-', "_")
}

fn workflow_transition_guards() -> Vec<WorkflowTransitionGuard> {
    vec![
        workflow_transition_guard("project", "draft", "confirmed", true, "human approval"),
        workflow_transition_guard(
            "project",
            "confirmed",
            "active",
            true,
            "ready milestone exists",
        ),
        workflow_transition_guard("project", "active", "audit", true, "all milestones done"),
        workflow_transition_guard(
            "project",
            "active",
            "done",
            false,
            "audit and docs refresh required",
        ),
        workflow_transition_guard("milestone", "draft", "ready", true, "gate criteria exists"),
        workflow_transition_guard(
            "milestone",
            "ready",
            "active",
            true,
            "project active and no active sibling",
        ),
        workflow_transition_guard("milestone", "active", "review", true, "all issues done"),
        workflow_transition_guard(
            "milestone",
            "active",
            "done",
            false,
            "milestone review required",
        ),
        workflow_transition_guard(
            "issue",
            "draft",
            "ready",
            true,
            "contract completeness gate",
        ),
        workflow_transition_guard(
            "issue",
            "ready",
            "eligible",
            true,
            "eligibility engine passes",
        ),
        workflow_transition_guard("issue", "eligible", "leased", true, "lease acquired"),
        workflow_transition_guard("issue", "ready", "leased", false, "eligibility required"),
        workflow_transition_guard(
            "issue",
            "leased",
            "done",
            false,
            "run, checks, merge, and evidence required",
        ),
        workflow_transition_guard(
            "issue",
            "merged",
            "done",
            false,
            "evidence capture required",
        ),
    ]
}

fn workflow_transition_guard(
    entity_kind: &str,
    from: &str,
    to: &str,
    allowed: bool,
    guard: &str,
) -> WorkflowTransitionGuard {
    WorkflowTransitionGuard {
        entity_kind: entity_kind.to_string(),
        from: from.to_string(),
        to: to.to_string(),
        allowed,
        guard: guard.to_string(),
    }
}

fn workflow_state_summary_markdown(state: &WorkflowStateSnapshot) -> String {
    let failed_checks = state
        .checks
        .iter()
        .filter(|check| check.status == "fail")
        .map(|check| {
            format!(
                "| `{}` | `{}` | `{}` | `{}` | {} |",
                check.severity,
                check.entity_kind,
                check.entity_id,
                check.id,
                check.message.replace('|', "\\|")
            )
        })
        .collect::<Vec<_>>();
    let failed_section = if failed_checks.is_empty() {
        "No failed checks.".to_string()
    } else {
        format!(
            "| Severity | Entity | ID | Check | Message |\n| --- | --- | --- | --- | --- |\n{}",
            failed_checks.join("\n")
        )
    };

    format!(
        "# Workflow State Summary\n\n- Generated by: Codex\n- Ready: `{}`\n- Projects: {}\n- Milestones: {}\n- Issues: {}\n- Errors: {}\n- Warnings: {}\n\n## Failed Checks\n\n{}\n\n## Transition Guards\n\n{}\n\n## Boundary\n\n- This check validates local `.agentflow/` workflow facts.\n- It writes only `.agentflow/state/workflow-state.json` and this summary.\n- It does not execute run / verify / review, call models, create remote issues, or create PRs.\n",
        state.ready,
        state.counts.projects,
        state.counts.milestones,
        state.counts.issues,
        state.counts.errors,
        state.counts.warnings,
        failed_section,
        workflow_transition_guard_table(&state.transition_guards)
    )
}

fn workflow_transition_guard_table(guards: &[WorkflowTransitionGuard]) -> String {
    let rows = guards
        .iter()
        .map(|guard| {
            format!(
                "| `{}` | `{}` | `{}` | `{}` | {} |",
                guard.entity_kind,
                guard.from,
                guard.to,
                guard.allowed,
                guard.guard.replace('|', "\\|")
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!("| Entity | From | To | Allowed | Guard |\n| --- | --- | --- | --- | --- |\n{rows}")
}

fn workflow_eligibility_summary_markdown(snapshot: &WorkflowEligibilitySnapshot) -> String {
    let rows = if snapshot.candidates.is_empty() {
        "| Issue | Ready | Eligible | Leased | Project | Milestone | Failure reasons |\n| --- | --- | --- | --- | --- | --- | --- |\n| none | - | - | - | - | - | no active milestone issue |".to_string()
    } else {
        snapshot
            .candidates
            .iter()
            .map(|candidate| {
                format!(
                    "| `{}` | `{}` | `{}` | `{}` | `{}` | `{}` | {} |",
                    candidate.issue_id,
                    candidate.ready,
                    candidate.eligible,
                    candidate.leased,
                    candidate.project_id.as_deref().unwrap_or("none"),
                    candidate.milestone_id.as_deref().unwrap_or("none"),
                    if candidate.failure_reasons.is_empty() {
                        "none".to_string()
                    } else {
                        candidate.failure_reasons.join(", ").replace('|', "\\|")
                    }
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    format!(
        "# Workflow Eligibility Summary\n\n- Generated by: Codex\n- Active project: `{}`\n- Active milestone: `{}`\n- Eligible issue: `{}`\n- Next action: `{}`\n- Recommended command: `{}`\n\n## Counts\n\n| Item | Count |\n| --- | ---: |\n| Ready issues | {} |\n| Eligible issues | {} |\n| Blocked issues | {} |\n\n## Candidates\n\n{}\n\n## Rationale\n\n{}\n\n## Boundary\n\n- Eligibility is computed locally from `.agentflow/` facts.\n- Ready is not Eligible; Lease is not Done.\n- This command does not execute run / verify / review, call models, create remote issues, or create PRs.\n",
        snapshot.active_project_id.as_deref().unwrap_or("none"),
        snapshot.active_milestone_id.as_deref().unwrap_or("none"),
        snapshot.eligible_issue_id.as_deref().unwrap_or("none"),
        snapshot.summary.next_action,
        snapshot.summary.recommended_command,
        snapshot.summary.ready_issue_count,
        snapshot.summary.eligible_issue_count,
        snapshot.summary.blocked_issue_count,
        rows,
        markdown_list(&snapshot.summary.rationale)
    )
}

fn workflow_lease_summary_markdown(snapshot: &WorkflowLeaseSnapshot) -> String {
    let active_rows = lease_table(&snapshot.active_leases);
    let stale_rows = lease_table(&snapshot.stale_leases);
    format!(
        "# Workflow Lease Summary\n\n- Generated by: Codex\n- Active leases: {}\n- Stale leases: {}\n- Recommended command: `{}`\n\n## Active Leases\n\n{}\n\n## Stale Leases\n\n{}\n\n## Boundary\n\n- Lease state is local-only under `.agentflow/leases/`.\n- Stale leases are detected but not automatically recovered.\n- This command does not run code, call models, create remote issues, or create PRs.\n",
        snapshot.active_leases.len(),
        snapshot.stale_leases.len(),
        snapshot.recommended_command,
        active_rows,
        stale_rows
    )
}

fn project_closure_summary_markdown(snapshot: &ProjectClosureStateSnapshot) -> String {
    format!(
        "# Project Closure Summary\n\n- Generated by: Codex\n- Active project: `{}`\n- Project status: `{}`\n- Active milestone: `{}`\n- Closure state: `{}`\n- Can mark done: `{}`\n- Recommended command: `{}`\n\n## Counts\n\n| Item | Count |\n| --- | ---: |\n| Milestones | {} |\n| Completed milestones | {} |\n| Issues | {} |\n| Completed issues | {} |\n| Runs | {} |\n| Evidence reports | {} |\n| Reviews | {} |\n| Project updates | {} |\n\n## Gates\n\n{}\n\n## Done Blocked Reasons\n\n{}\n\n## Boundary\n\n- This command only writes `.agentflow/state/project-closure.json` and `.agentflow/updates/PROJECT-CLOSURE-SUMMARY.md`.\n- It does not create `.agentflow/audits/`.\n- It does not generate Code Audit, execute Root Docs Refresh, approve final review, call models, create remote PRs, or mark Project done.\n",
        snapshot.active_project_id.as_deref().unwrap_or("none"),
        snapshot.project_status.as_deref().unwrap_or("none"),
        snapshot.active_milestone_id.as_deref().unwrap_or("none"),
        snapshot.closure_state,
        snapshot.can_mark_done,
        snapshot.recommended_command,
        snapshot.counts.milestones,
        snapshot.counts.completed_milestones,
        snapshot.counts.issues,
        snapshot.counts.completed_issues,
        snapshot.counts.runs,
        snapshot.counts.evidence_reports,
        snapshot.counts.reviews,
        snapshot.counts.project_updates,
        project_closure_gate_table(&snapshot.gates),
        if snapshot.done_blocked_reasons.is_empty() {
            "No blockers remain.".to_string()
        } else {
            markdown_list(&snapshot.done_blocked_reasons)
        }
    )
}

fn project_code_audit_summary_markdown(snapshot: &ProjectCodeAuditSnapshot) -> String {
    format!(
        "# Project Code Audit Summary\n\n- Generated by: Codex\n- Active project: `{}`\n- Active milestone: `{}`\n- Closure state: `{}`\n- Audit state: `{}`\n- Recommended command: `{}`\n\n## Counts\n\n| Item | Count |\n| --- | ---: |\n| Projects | {} |\n| Milestones | {} |\n| Issues | {} |\n| Completed issues | {} |\n| Runs | {} |\n| Evidence reports | {} |\n| Reviews | {} |\n| Project updates | {} |\n| Source files scanned | {} |\n| Findings | {} |\n| Blockers | {} |\n\n## Checks\n\n{}\n\n## Blockers\n\n{}\n\n## Boundary\n\n- This command only writes `.agentflow/state/project-code-audit.json` and `.agentflow/updates/PROJECT-CODE-AUDIT-SUMMARY.md`.\n- It does not create `.agentflow/audits/`.\n- It does not modify code or docs, auto-fix findings, call models, create remote PRs, or mark Project done.\n- `snapshot-ready` is not final Code Audit approval.\n",
        snapshot.active_project_id.as_deref().unwrap_or("none"),
        snapshot.active_milestone_id.as_deref().unwrap_or("none"),
        snapshot.closure_state,
        snapshot.audit_state,
        snapshot.recommended_command,
        snapshot.counts.projects,
        snapshot.counts.milestones,
        snapshot.counts.issues,
        snapshot.counts.completed_issues,
        snapshot.counts.runs,
        snapshot.counts.evidence_reports,
        snapshot.counts.reviews,
        snapshot.counts.project_updates,
        snapshot.counts.source_files,
        snapshot.counts.findings,
        snapshot.counts.blockers,
        project_code_audit_check_table(&snapshot.checks),
        if snapshot.blockers.is_empty() {
            "No blockers remain.".to_string()
        } else {
            markdown_list(&snapshot.blockers)
        }
    )
}

fn project_docs_refresh_summary_markdown(snapshot: &ProjectDocsRefreshSnapshot) -> String {
    format!(
        "# Project Docs Refresh Summary\n\n- Generated by: Codex\n- Active project: `{}`\n- Active milestone: `{}`\n- Closure state: `{}`\n- Docs refresh state: `{}`\n- Recommended command: `{}`\n\n## Counts\n\n| Item | Count |\n| --- | ---: |\n| Checked docs | {} |\n| Current docs | {} |\n| Update-needed docs | {} |\n| Missing docs | {} |\n| Intentionally absent docs | {} |\n| Required updates | {} |\n| Blockers | {} |\n\n## Checked Docs\n\n{}\n\n## Required Updates\n\n{}\n\n## Blockers\n\n{}\n\n## Boundary\n\n- This command only writes `.agentflow/state/project-docs-refresh.json` and `.agentflow/updates/PROJECT-DOCS-REFRESH-SUMMARY.md`.\n- It does not create `.agentflow/audits/`.\n- It does not modify docs, refresh docs, call models, create remote PRs, or mark Project done.\n- `snapshot-ready` is not final Root Docs Refresh approval.\n",
        snapshot.active_project_id.as_deref().unwrap_or("none"),
        snapshot.active_milestone_id.as_deref().unwrap_or("none"),
        snapshot.closure_state,
        snapshot.docs_refresh_state,
        snapshot.recommended_command,
        snapshot.counts.checked_docs,
        snapshot.counts.current_docs,
        snapshot.counts.update_needed_docs,
        snapshot.counts.missing_docs,
        snapshot.counts.intentionally_absent_docs,
        snapshot.counts.required_updates,
        snapshot.counts.blockers,
        project_docs_refresh_doc_table(&snapshot.checked_docs),
        project_docs_refresh_required_update_table(&snapshot.required_updates),
        if snapshot.blockers.is_empty() {
            "No blockers remain.".to_string()
        } else {
            markdown_list(&snapshot.blockers)
        }
    )
}

fn project_closure_gate_table(gates: &[ProjectClosureGate]) -> String {
    let rows = gates
        .iter()
        .map(|gate| {
            format!(
                "| `{}` | {} | `{}` | `{}` | `{}` | {} |",
                gate.id,
                gate.name.replace('|', "\\|"),
                gate.status,
                gate.required,
                gate.path.as_deref().unwrap_or("none"),
                gate.detail.replace('|', "\\|")
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!("| Gate | Name | Status | Required | Path | Detail |\n| --- | --- | --- | --- | --- | --- |\n{rows}")
}

fn project_code_audit_check_table(checks: &[ProjectCodeAuditCheck]) -> String {
    let rows = checks
        .iter()
        .map(|check| {
            let first_finding = check
                .findings
                .first()
                .map(|finding| {
                    format!(
                        "{}{}: {}",
                        finding.path.as_deref().unwrap_or("none"),
                        finding
                            .line
                            .map(|line| format!(":{line}"))
                            .unwrap_or_default(),
                        finding.snippet.replace('|', "\\|")
                    )
                })
                .unwrap_or_else(|| "-".to_string());
            format!(
                "| `{}` | {} | `{}` | {} | {} | {} |",
                check.id,
                check.name.replace('|', "\\|"),
                check.status,
                check.candidate_count,
                check.detail.replace('|', "\\|"),
                first_finding
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!("| Check | Name | Status | Candidates | Detail | First finding |\n| --- | --- | --- | ---: | --- | --- |\n{rows}")
}

fn project_docs_refresh_doc_table(docs: &[ProjectDocsRefreshCheckedDoc]) -> String {
    let rows = docs
        .iter()
        .map(|doc| {
            format!(
                "| `{}` | `{}` | `{}` | {} |",
                doc.path,
                doc.category,
                doc.status,
                doc.reason.replace('|', "\\|")
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!("| Doc | Category | Status | Reason |\n| --- | --- | --- | --- |\n{rows}")
}

fn project_docs_refresh_required_update_table(
    updates: &[ProjectDocsRefreshRequiredUpdate],
) -> String {
    if updates.is_empty() {
        return "| Doc | Severity | Summary | Follow-up issue intent |\n| --- | --- | --- | --- |\n| none | - | - | - |".to_string();
    }
    let rows = updates
        .iter()
        .map(|update| {
            format!(
                "| `{}` | `{}` | {} | {} |",
                update.path,
                update.severity,
                update.summary.replace('|', "\\|"),
                update.follow_up_issue_intent.replace('|', "\\|")
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "| Doc | Severity | Summary | Follow-up issue intent |\n| --- | --- | --- | --- |\n{rows}"
    )
}

fn lease_table(leases: &[WorkflowLeaseRecord]) -> String {
    if leases.is_empty() {
        return "| Lease | Issue | Project | Milestone | Owner | Expires |\n| --- | --- | --- | --- | --- | --- |\n| none | - | - | - | - | - |".to_string();
    }
    let rows = leases
        .iter()
        .map(|lease| {
            format!(
                "| `{}` | `{}` | `{}` | `{}` | `{}` | `{}` |",
                lease.id,
                lease.issue_id,
                lease.project_id.as_deref().unwrap_or("none"),
                lease.milestone_id.as_deref().unwrap_or("none"),
                lease.owner_agent_id,
                lease.expires_at_epoch_seconds
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "| Lease | Issue | Project | Milestone | Owner | Expires |\n| --- | --- | --- | --- | --- | --- |\n{rows}"
    )
}

pub fn collect_context(repo: &Path) -> Result<ProjectContext> {
    let settings = read_optional_settings(repo)?;
    let mut files = Vec::new();
    collect_files(repo, repo, &mut files)?;
    files.sort_by(|left, right| left.path.cmp(&right.path));
    let detected_stacks = detected_stacks(&files);
    let validation_commands = settings
        .map(|settings| settings.validation_commands)
        .unwrap_or_else(|| inferred_validation_commands(&detected_stacks));

    Ok(ProjectContext {
        version: VERSION.to_string(),
        root: ".".to_string(),
        detected_stacks,
        validation_commands,
        files,
    })
}

fn default_settings(repo: &Path) -> Settings {
    let project_name = repo
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("agentflow-project")
        .to_string();
    Settings {
        version: VERSION.to_string(),
        project_name,
        default_model_provider: "deepseek".to_string(),
        model_providers: vec![ModelProvider {
            id: "deepseek".to_string(),
            provider_type: "openai-compatible".to_string(),
            api_key_env: "DEEPSEEK_API_KEY".to_string(),
        }],
        validation_commands: vec!["cargo test".to_string(), "git diff --check".to_string()],
        data_policy: DataPolicy {
            local_first: true,
            upload_code_by_default: false,
        },
    }
}

fn default_index() -> ProjectIndex {
    ProjectIndex {
        version: VERSION.to_string(),
        next_issue_number: 1,
        next_run_number: 1,
        issues: Vec::new(),
    }
}

fn write_goal_bootstrap_artifacts(
    project_dir: &Path,
    goal_path: &Path,
    goal: &ProjectGoal,
    force: bool,
) -> Result<GoalBootstrapSummary> {
    ensure_dir(&project_dir.join("bootstrap"))?;
    let outputs = project_definition_outputs();
    let definition = ProjectDefinition {
        version: VERSION.to_string(),
        source_goal: goal_path.display().to_string(),
        phase: "AEP Stage 1 / Goal Initialization".to_string(),
        status: "initialized".to_string(),
        outputs: outputs.clone(),
        rules: vec![
            "/goal 是项目初始化入口。".to_string(),
            "Project Definition 初始化合同不授权代码执行。".to_string(),
            "IssueContract 是唯一执行授权。".to_string(),
            "Roadmap、candidate issue、saved view 和 project update 都不能直接授权执行。"
                .to_string(),
        ],
    };
    let scope_state = default_scope_state();
    let project_definition_json = project_dir.join("project-definition.json");
    let scope_state_json = project_dir.join("scope-state.json");
    let mut files_written = 0;

    files_written += write_maybe(
        &project_definition_json,
        &(serde_json::to_string_pretty(&definition)? + "\n"),
        force,
    )? as usize;
    files_written += write_maybe(
        &scope_state_json,
        &(serde_json::to_string_pretty(&scope_state)? + "\n"),
        force,
    )? as usize;
    for output in &outputs {
        files_written += write_maybe(
            &project_dir.join(&output.path),
            &bootstrap_output_markdown(output, goal),
            force,
        )? as usize;
    }

    Ok(GoalBootstrapSummary {
        project_definition_json,
        scope_state_json,
        files_written,
        files_checked: outputs.len() + 2,
    })
}

fn project_definition_outputs() -> Vec<ProjectDefinitionOutput> {
    [
        ("goal", "GOAL.md copy", "goal.md"),
        (
            "blueprint-seed",
            "Root Blueprint Seed",
            "bootstrap/complete-blueprint-design.md",
        ),
        ("architecture", "Architecture Map", "architecture.md"),
        ("roadmap", "Capability Roadmap", "roadmap.md"),
        (
            "project-bootstrap-sequence",
            "Project Bootstrap Sequence",
            "bootstrap/project-bootstrap-sequence.md",
        ),
        (
            "product-surface-map",
            "Product Surface Map",
            "bootstrap/product-surface-map.md",
        ),
        (
            "frontend-wireframe",
            "Frontend Surface Wireframe",
            "bootstrap/frontend-surface-wireframe.md",
        ),
        (
            "frontend-viewmodel-contract",
            "Frontend ViewModel Contract",
            "bootstrap/frontend-viewmodel-contract.md",
        ),
        (
            "backend-usecase-contract",
            "Backend Use Case Contract",
            "bootstrap/backend-usecase-contract.md",
        ),
        (
            "persistence-boundary",
            "Persistence Boundary",
            "bootstrap/persistence-boundary.md",
        ),
        (
            "read-model-projection",
            "Read Model Projection",
            "bootstrap/read-model-projection.md",
        ),
        ("api-contract", "API Contract", "bootstrap/api-contract.md"),
        (
            "linear-project-draft",
            "Linear Project Draft",
            "bootstrap/linear-project-draft.md",
        ),
        (
            "linear-issue-draft",
            "Linear Issue Draft",
            "bootstrap/linear-issue-draft.md",
        ),
        (
            "agent-sandbox-profile",
            "Agent Sandbox Profile",
            "bootstrap/agent-sandbox-profile.md",
        ),
    ]
    .into_iter()
    .map(|(id, name, path)| ProjectDefinitionOutput {
        id: id.to_string(),
        name: name.to_string(),
        path: path.to_string(),
        status: "initialized".to_string(),
    })
    .collect()
}

fn goal_readiness_paths() -> Vec<(String, String)> {
    let mut paths = vec![
        ("ProjectGoal".to_string(), "goal.json".to_string()),
        (
            "ProjectDefinition".to_string(),
            "project-definition.json".to_string(),
        ),
        ("ScopeState".to_string(), "scope-state.json".to_string()),
        ("Environment".to_string(), "environment.md".to_string()),
        ("Architecture".to_string(), "architecture.md".to_string()),
        ("Roadmap".to_string(), "roadmap.md".to_string()),
        (
            "InitializationEvidence".to_string(),
            "evidence/FLOW-0-initialization.md".to_string(),
        ),
    ];
    paths.extend(
        project_definition_outputs()
            .into_iter()
            .map(|output| (output.name, output.path)),
    );
    paths.sort_by(|left, right| left.1.cmp(&right.1));
    paths.dedup_by(|left, right| left.1 == right.1);
    paths
}

fn default_scope_state() -> AgentScopeState {
    AgentScopeState {
        version: VERSION.to_string(),
        wip_limit: 1,
        active_issue_id: None,
        current_phase: "goal-initialization".to_string(),
        execution_authorized: false,
        authorization_source: "issue-contract".to_string(),
        boundaries: vec![
            "Project Definition 初始化合同不授权执行。".to_string(),
            "每次执行必须先绑定唯一 IssueContract。".to_string(),
            "WIP limit is 1 for active agent execution.".to_string(),
            "本地 v0 不创建远程 PR、Linear issue 或团队 workspace 变更。".to_string(),
        ],
    }
}

fn bootstrap_output_markdown(output: &ProjectDefinitionOutput, goal: &ProjectGoal) -> String {
    format!(
        "# {}\n\n- Generated: 2026-05-22\n- Executor: Codex\n- AEP phase: Goal Initialization\n- Status: `{}`\n\n## Source Goal\n\n{}\n\n## Boundary\n\n- This artifact is initialized from `/goal`.\n- It does not authorize execution by itself.\n- A concrete `IssueContract` is required before any run.\n\n## Initial Notes\n\n{}\n",
        output.name,
        output.status,
        goal.objective,
        bootstrap_output_notes(&output.id, goal)
    )
}

fn bootstrap_output_notes(id: &str, goal: &ProjectGoal) -> String {
    match id {
        "project-bootstrap-sequence" => numbered_list(&[
            "Compile `/goal` into ProjectGoal.".to_string(),
            "Initialize Flow 0.1 / 0.2 / 0.3 artifacts.".to_string(),
            "Run `agentflow goal check` before planning executable issues.".to_string(),
            "Create an IssueContract only after scope and validation are explicit.".to_string(),
        ]),
        "agent-sandbox-profile" => markdown_list(&[
            "Local-first filesystem facts.".to_string(),
            "No source upload by default.".to_string(),
            "External network requires explicit gate in the issue contract.".to_string(),
            "Only issue-bound runs can produce completed evidence.".to_string(),
        ]),
        "linear-issue-draft" => markdown_list(&[
            "Local v0 keeps this as a draft only.".to_string(),
            "Remote Linear creation is not implemented.".to_string(),
            "Execution authorization remains local IssueContract.".to_string(),
        ]),
        "blueprint-seed" => markdown_list(&[
            "Final product: local AI engineering execution workbench.".to_string(),
            "Current construction scope: `/goal` to local evidence chain.".to_string(),
            "Future construction zones: desktop shell, integrations, team workflow.".to_string(),
        ]),
        "product-surface-map" => markdown_list(&[
            "Project overview.".to_string(),
            "Issue contract list and detail.".to_string(),
            "Run, validation, evidence, review, and project update views.".to_string(),
        ]),
        "persistence-boundary" => markdown_list(&[
            ".agentflow JSON and Markdown are canonical facts.".to_string(),
            "SQLite is a rebuildable query index.".to_string(),
            "ProjectUpdate is derived from evidence, not an independent truth source.".to_string(),
        ]),
        "read-model-projection" => markdown_list(&[
            "Project summary reads issue, run, update, and saved view facts.".to_string(),
            "Desktop v0 must stay read-only over `.agentflow/` facts.".to_string(),
        ]),
        "api-contract" => markdown_list(&[
            "Core Rust functions are the first API boundary.".to_string(),
            "CLI commands are the local user-facing API.".to_string(),
            "Tauri commands remain future work after the read-only shell boundary.".to_string(),
        ]),
        _ => {
            let mut notes = vec!["Initialized from `/goal`.".to_string()];
            notes.extend(goal.success_criteria.iter().take(3).cloned());
            markdown_list(&notes)
        }
    }
}

fn aep_issue_protocol(settings: &Settings) -> AepIssueProtocol {
    AepIssueProtocol {
        phase: "AEP Issue Execution".to_string(),
        stop_condition:
            "Stop after local validation, evidence, review, and project update are recorded."
                .to_string(),
        fastest_feedback_loop: settings.validation_commands.clone(),
        vertical_slice: "Deliver one locally verifiable slice tied to this issue contract."
            .to_string(),
        tracer_bullet_plan: vec![
            "Create or update the smallest artifact that proves the contract path.".to_string(),
            "Run the fastest deterministic validation command first.".to_string(),
            "Record command output and known limitations in evidence.".to_string(),
        ],
        diagnose_plan: vec![
            "On failure, keep the issue non-completed.".to_string(),
            "Record failing command, exit code, stdout, and stderr.".to_string(),
            "Narrow the next attempt to the failing contract boundary.".to_string(),
        ],
        graphify_context_status: "not-integrated-v0-local-context-only".to_string(),
        docs_claim_trace: vec![
            ".agentflow/goal.json".to_string(),
            ".agentflow/project-definition.json".to_string(),
            ".agentflow/architecture.md".to_string(),
            ".agentflow/roadmap.md".to_string(),
        ],
        boundary_confirmation: vec![
            "Roadmap and candidate issues are not execution authorization.".to_string(),
            "IssueContract is the only execution input.".to_string(),
            "No remote PR, Linear issue, merge, or team workspace mutation in local v0."
                .to_string(),
        ],
        pr_handoff_requirements: vec![
            "Local review assistant only.".to_string(),
            "Handoff text must point to evidence and review artifacts.".to_string(),
            "Remote PR automation remains future work.".to_string(),
        ],
    }
}

fn required_project_dir(repo: &Path) -> Result<PathBuf> {
    let project_dir = repo.join(AGENTFLOW_DIR);
    if !project_dir.exists() {
        bail!(
            "{} is missing; run agentflow init --from-goal first",
            AGENTFLOW_DIR
        );
    }
    Ok(project_dir)
}

fn start_run(repo: &Path, issue_id: &str) -> Result<RunSummary> {
    let project_dir = required_project_dir(repo)?;
    let issue = hydrate_issue_protocol(&project_dir, read_issue(&project_dir, issue_id)?)?;
    let lease = acquire_workflow_lease(repo, &issue, "codex-local")?;
    if let Err(error) = claim_scope_state_for_run(&project_dir, issue_id) {
        release_workflow_lease_by_id(&project_dir, &lease.id, "scope-state-claim-failed")?;
        return Err(error);
    }

    if let Some((run_dir, existing_run)) =
        latest_reusable_dry_run_for_issue(&project_dir, issue_id, &lease.id)?
    {
        return Ok(RunSummary {
            run_id: existing_run.id.clone(),
            run_json: run_dir.join("run.json"),
            run_dir,
            run: existing_run,
        });
    }

    let mut index: ProjectIndex = read_json(&project_dir.join("index.json"))?;
    let run_id = format!("RUN-{:04}", index.next_run_number);
    let run_dir = project_dir.join("runs").join(&run_id);
    ensure_dir(&run_dir)?;

    let run = AgentRun {
        id: run_id.clone(),
        issue_id: issue.id.clone(),
        project_id: lease.project_id.clone(),
        milestone_id: lease.milestone_id.clone(),
        lease_id: Some(lease.id.clone()),
        status: "dry-run".to_string(),
        mode: "dry-run".to_string(),
        run_plan: controlled_run_plan(&issue),
        validation_commands: Vec::new(),
        outputs: RunOutputs {
            transcript: "transcript.md".to_string(),
            commands: "commands.jsonl".to_string(),
            diff_summary: "diff-summary.md".to_string(),
            evidence: None,
            review: None,
            update: None,
        },
    };

    write_json(&run_dir.join("run.json"), &run, false)?;
    write_new(
        &run_dir.join("transcript.md"),
        &run_transcript(&issue, &run),
        false,
    )?;
    write_new(&run_dir.join("commands.jsonl"), "", false)?;
    write_new(
        &run_dir.join("diff-summary.md"),
        &dry_run_diff_summary(&issue, &run),
        false,
    )?;

    index.next_run_number += 1;
    write_json(&project_dir.join("index.json"), &index, true)?;
    let mut active_issue = issue.clone();
    if matches!(
        canonical_issue_status(&active_issue.status),
        IssueStatus::Todo | IssueStatus::Backlog
    ) {
        active_issue.status = IssueStatus::InProgress.as_str().to_string();
        write_issue(&project_dir, &active_issue)?;
        update_issue_status(&project_dir, issue_id, IssueStatus::InProgress.as_str())?;
    }

    Ok(RunSummary {
        run_id: run_id.clone(),
        run_json: run_dir.join("run.json"),
        run_dir,
        run,
    })
}

fn latest_reusable_dry_run_for_issue(
    project_dir: &Path,
    issue_id: &str,
    lease_id: &str,
) -> Result<Option<(PathBuf, AgentRun)>> {
    let Some(run_dir) = latest_run_dir_for_issue(project_dir, issue_id)? else {
        return Ok(None);
    };
    let run: AgentRun = read_json(&run_dir.join("run.json"))?;
    let reusable = run.mode == "dry-run"
        && run.validation_commands.is_empty()
        && run.outputs.evidence.is_none()
        && run.outputs.review.is_none()
        && run.lease_id.as_deref() == Some(lease_id);
    if reusable {
        Ok(Some((run_dir, run)))
    } else {
        Ok(None)
    }
}

fn controlled_run_plan(issue: &IssueContract) -> ControlledRunPlan {
    ControlledRunPlan {
        goal: issue.intent.clone(),
        non_goals: issue.non_goals.clone(),
        expected_files: issue.context.files.clone(),
        blocked_files: blocked_files_from_issue(issue),
        planned_steps: issue.execution_plan.clone(),
        validation_commands: issue.validation.commands.clone(),
        evidence_requirements: issue.evidence_requirements.clone(),
        rollback_plan: issue.rollback_plan.clone(),
    }
}

fn blocked_files_from_issue(issue: &IssueContract) -> Vec<String> {
    let mut blocked = issue
        .non_goals
        .iter()
        .filter(|item| {
            let text = item.to_lowercase();
            text.contains("不创建")
                || text.contains("不从")
                || text.contains("不调用")
                || text.contains("不绕过")
                || text.contains("remote")
                || text.contains("github")
                || text.contains("linear")
                || text.contains("desktop")
                || text.contains("model")
        })
        .cloned()
        .collect::<Vec<_>>();
    if issue.human_gate.before_external_network {
        blocked.push("external-network-without-human-confirmation".to_string());
    }
    if blocked.is_empty() {
        blocked.push("none-declared".to_string());
    }
    blocked
}

fn read_issue(project_dir: &Path, issue_id: &str) -> Result<IssueContract> {
    read_json(&project_dir.join(format!("issues/{issue_id}.json")))
}

fn write_issue(project_dir: &Path, issue: &IssueContract) -> Result<()> {
    write_json(
        &project_dir.join(format!("issues/{}.json", issue.id)),
        issue,
        true,
    )?;
    write_new(
        &project_dir.join(format!("issues/{}.md", issue.id)),
        &issue_markdown_body(issue),
        true,
    )
}

fn hydrate_issue_protocol(project_dir: &Path, mut issue: IssueContract) -> Result<IssueContract> {
    let mut changed = false;
    if issue.aep.phase.is_empty() {
        let settings: Settings = read_json(&project_dir.join("settings.json"))?;
        issue.aep = aep_issue_protocol(&settings);
        changed = true;
    }
    for requirement in ["aep-contract-checklist", "docs-claim-trace"] {
        if !issue
            .evidence_requirements
            .iter()
            .any(|item| item == requirement)
        {
            issue.evidence_requirements.push(requirement.to_string());
            changed = true;
        }
    }
    if changed {
        write_issue(project_dir, &issue)?;
    }
    Ok(issue)
}

fn claim_scope_state_for_run(project_dir: &Path, issue_id: &str) -> Result<()> {
    let path = project_dir.join("scope-state.json");
    let mut state = read_scope_state_or_default(&path)?;
    if let Some(active_issue_id) = &state.active_issue_id {
        if active_issue_id != issue_id {
            bail!(
                "active issue `{}` blocks `{}` because WIP limit is {}",
                active_issue_id,
                issue_id,
                state.wip_limit
            );
        }
    }
    state.active_issue_id = Some(issue_id.to_string());
    state.current_phase = "issue-execution".to_string();
    state.execution_authorized = true;
    write_json(&path, &state, true)
}

fn release_scope_state_after_review(
    project_dir: &Path,
    issue_id: &str,
    passed: bool,
) -> Result<()> {
    let path = project_dir.join("scope-state.json");
    let mut state = read_scope_state_or_default(&path)?;
    if state.active_issue_id.as_deref() == Some(issue_id) && passed {
        state.active_issue_id = None;
        state.current_phase = "project-update".to_string();
        state.execution_authorized = false;
        write_json(&path, &state, true)?;
    }
    Ok(())
}

fn read_scope_state_or_default(path: &Path) -> Result<AgentScopeState> {
    if path.exists() {
        read_json(path)
    } else {
        Ok(default_scope_state())
    }
}

fn latest_run_dir_for_issue(project_dir: &Path, issue_id: &str) -> Result<Option<PathBuf>> {
    let runs_dir = project_dir.join("runs");
    if !runs_dir.exists() {
        return Ok(None);
    }

    let mut candidates = Vec::new();
    for entry in fs::read_dir(&runs_dir).with_context(|| format!("read {}", runs_dir.display()))? {
        let entry = entry?;
        if !entry.path().is_dir() {
            continue;
        }
        let run_json = entry.path().join("run.json");
        if !run_json.exists() {
            continue;
        }
        let run: AgentRun = read_json(&run_json)?;
        if run.issue_id == issue_id {
            candidates.push((run.id, entry.path()));
        }
    }
    candidates.sort_by(|left, right| left.0.cmp(&right.0));
    Ok(candidates.pop().map(|(_, path)| path))
}

fn append_jsonl<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("open jsonl {}", path.display()))?;
    let line = serde_json::to_string(value)?;
    writeln!(file, "{line}").with_context(|| format!("append jsonl {}", path.display()))
}

fn update_issue_status(project_dir: &Path, issue_id: &str, status: &str) -> Result<()> {
    let index_path = project_dir.join("index.json");
    let mut index: ProjectIndex = read_json(&index_path)?;
    for entry in &mut index.issues {
        if entry.id == issue_id {
            entry.status = status.to_string();
        }
    }
    write_json(&index_path, &index, true)
}

fn next_project_update_path(project_dir: &Path) -> Result<PathBuf> {
    let updates_dir = project_dir.join("updates");
    ensure_dir(&updates_dir)?;
    let mut number = 1;
    loop {
        let path = updates_dir.join(format!("PROJECT-UPDATE-{number:04}.md"));
        if !path.exists() {
            return Ok(path);
        }
        number += 1;
    }
}

fn create_index_schema(connection: &Connection) -> Result<()> {
    connection.execute_batch(
        "
        CREATE TABLE issues (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            status TEXT NOT NULL,
            intent TEXT NOT NULL,
            json_path TEXT NOT NULL
        );
        CREATE TABLE runs (
            id TEXT PRIMARY KEY,
            issue_id TEXT NOT NULL,
            status TEXT NOT NULL,
            mode TEXT NOT NULL,
            validation_status TEXT NOT NULL,
            json_path TEXT NOT NULL
        );
        CREATE TABLE commands (
            run_id TEXT NOT NULL,
            command TEXT NOT NULL,
            exit_code INTEGER NOT NULL,
            status TEXT NOT NULL
        );
        CREATE TABLE updates (
            path TEXT PRIMARY KEY,
            source_issue TEXT NOT NULL,
            source_run TEXT NOT NULL,
            status TEXT NOT NULL
        );
        CREATE TABLE saved_views (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            issue_status TEXT,
            run_status TEXT,
            validation_status TEXT,
            issue_id TEXT,
            json_path TEXT NOT NULL
        );
        ",
    )?;
    Ok(())
}

fn load_indexed_issues(repo: &Path, project_dir: &Path) -> Result<Vec<IndexedIssue>> {
    let issue_paths = json_files(&project_dir.join("issues"))?;
    let mut issues = Vec::new();
    for path in issue_paths {
        let issue: IssueContract = read_json(&path)?;
        issues.push(IndexedIssue {
            id: issue.id,
            title: issue.title,
            status: issue.status,
            intent: issue.intent,
            json_path: relative_path(repo, &path)?,
        });
    }
    issues.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(issues)
}

fn load_indexed_runs(
    repo: &Path,
    project_dir: &Path,
) -> Result<Vec<(IndexedRun, Vec<CommandRecord>)>> {
    let runs_dir = project_dir.join("runs");
    if !runs_dir.exists() {
        return Ok(Vec::new());
    }

    let mut run_paths = Vec::new();
    for entry in fs::read_dir(&runs_dir).with_context(|| format!("read {}", runs_dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() && path.join("run.json").exists() {
            run_paths.push(path.join("run.json"));
        }
    }
    run_paths.sort();

    let mut runs = Vec::new();
    for path in run_paths {
        let run: AgentRun = read_json(&path)?;
        let records = run.validation_commands.clone();
        runs.push((
            IndexedRun {
                id: run.id,
                issue_id: run.issue_id,
                status: run.status,
                mode: run.mode,
                validation_status: validation_status(&records),
                json_path: relative_path(repo, &path)?,
            },
            records,
        ));
    }
    Ok(runs)
}

fn load_indexed_updates(repo: &Path, project_dir: &Path) -> Result<Vec<IndexedUpdate>> {
    let update_paths = markdown_files(&project_dir.join("updates"))?;
    let mut updates = Vec::new();
    for path in update_paths {
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if !file_name.starts_with("PROJECT-UPDATE-") {
            continue;
        }
        let content =
            fs::read_to_string(&path).with_context(|| format!("read update {}", path.display()))?;
        updates.push(IndexedUpdate {
            path: relative_path(repo, &path)?,
            source_issue: markdown_backtick_value(&content, "- Source issue:")
                .unwrap_or_else(|| "unknown".to_string()),
            source_run: markdown_backtick_value(&content, "- Source run:")
                .unwrap_or_else(|| "unknown".to_string()),
            status: markdown_backtick_value(&content, "- Status:")
                .unwrap_or_else(|| "unknown".to_string()),
        });
    }
    Ok(updates)
}

fn load_saved_views(project_dir: &Path) -> Result<Vec<SavedView>> {
    let view_paths = json_files(&project_dir.join("views"))?;
    let mut views = Vec::new();
    for path in view_paths {
        views.push(read_json(&path)?);
    }
    views.sort_by(|left: &SavedView, right| left.id.cmp(&right.id));
    Ok(views)
}

fn insert_saved_view(connection: &Connection, view: &SavedView, path: &Path) -> Result<()> {
    connection.execute(
        "INSERT INTO saved_views (id, name, issue_status, run_status, validation_status, issue_id, json_path) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            view.id,
            view.name,
            view.filter.issue_status.as_deref(),
            view.filter.run_status.as_deref(),
            view.filter.validation_status.as_deref(),
            view.filter.issue_id.as_deref(),
            path.to_string_lossy().to_string()
        ],
    )?;
    Ok(())
}

fn query_indexed_issues(
    connection: &Connection,
    filter: &SavedViewFilter,
) -> Result<Vec<IndexedIssue>> {
    let mut statement = connection
        .prepare("SELECT id, title, status, intent, json_path FROM issues ORDER BY id")?;
    let rows = statement
        .query_map([], |row| {
            Ok(IndexedIssue {
                id: row.get(0)?,
                title: row.get(1)?,
                status: row.get(2)?,
                intent: row.get(3)?,
                json_path: row.get(4)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(rows
        .into_iter()
        .filter(|issue| {
            filter
                .issue_status
                .as_ref()
                .is_none_or(|status| issue_status_matches_filter(&issue.status, status))
        })
        .filter(|issue| filter.issue_id.as_ref().is_none_or(|id| &issue.id == id))
        .collect())
}

fn issue_status_matches_filter(actual: &str, expected: &str) -> bool {
    actual == expected || canonical_issue_status(actual) == canonical_issue_status(expected)
}

fn query_indexed_runs(
    connection: &Connection,
    filter: &SavedViewFilter,
) -> Result<Vec<IndexedRun>> {
    let mut statement = connection.prepare(
        "SELECT id, issue_id, status, mode, validation_status, json_path FROM runs ORDER BY id",
    )?;
    let rows = statement
        .query_map([], |row| {
            Ok(IndexedRun {
                id: row.get(0)?,
                issue_id: row.get(1)?,
                status: row.get(2)?,
                mode: row.get(3)?,
                validation_status: row.get(4)?,
                json_path: row.get(5)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(rows
        .into_iter()
        .filter(|run| {
            filter
                .run_status
                .as_ref()
                .is_none_or(|status| &run.status == status)
        })
        .filter(|run| {
            filter
                .validation_status
                .as_ref()
                .is_none_or(|status| &run.validation_status == status)
        })
        .filter(|run| {
            filter
                .issue_id
                .as_ref()
                .is_none_or(|id| &run.issue_id == id)
        })
        .collect())
}

fn json_files(dir: &Path) -> Result<Vec<PathBuf>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    files_with_extension(dir, "json")
}

fn markdown_files(dir: &Path) -> Result<Vec<PathBuf>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    files_with_extension(dir, "md")
}

fn files_with_extension(dir: &Path, extension: &str) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    for entry in fs::read_dir(dir).with_context(|| format!("read {}", dir.display()))? {
        let path = entry?.path();
        if path
            .extension()
            .and_then(|value| value.to_str())
            .is_some_and(|value| value == extension)
        {
            paths.push(path);
        }
    }
    paths.sort();
    Ok(paths)
}

fn validation_status(records: &[CommandRecord]) -> String {
    if records.is_empty() {
        "not-run"
    } else if records.iter().all(|record| record.exit_code == 0) {
        "passed"
    } else {
        "failed"
    }
    .to_string()
}

fn markdown_backtick_value(content: &str, prefix: &str) -> Option<String> {
    content.lines().find_map(|line| {
        let line = line.trim();
        if !line.starts_with(prefix) {
            return None;
        }
        let start = line.find('`')?;
        let rest = &line[start + 1..];
        let end = rest.find('`')?;
        Some(rest[..end].to_string())
    })
}

fn saved_view_id(name: &str, project_dir: &Path) -> Result<String> {
    let slug = slugify(name);
    if !slug.is_empty() {
        return Ok(slug);
    }

    let mut number = 1;
    loop {
        let candidate = format!("view-{number:04}");
        if !view_file_path(project_dir, &candidate).exists() {
            return Ok(candidate);
        }
        number += 1;
    }
}

fn slugify(value: &str) -> String {
    let mut slug = String::new();
    let mut previous_dash = false;
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            previous_dash = false;
        } else if !previous_dash && !slug.is_empty() {
            slug.push('-');
            previous_dash = true;
        }
    }
    slug.trim_matches('-').to_string()
}

fn view_file_path(project_dir: &Path, view_id: &str) -> PathBuf {
    project_dir.join("views").join(format!("{view_id}.json"))
}

fn find_saved_view_path(project_dir: &Path, name_or_id: &str) -> Result<PathBuf> {
    let direct = view_file_path(project_dir, name_or_id);
    if direct.exists() {
        return Ok(direct);
    }

    let slug = slugify(name_or_id);
    if !slug.is_empty() {
        let slug_path = view_file_path(project_dir, &slug);
        if slug_path.exists() {
            return Ok(slug_path);
        }
    }

    for path in json_files(&project_dir.join("views"))? {
        let view: SavedView = read_json(&path)?;
        if view.name == name_or_id || view.id == name_or_id {
            return Ok(path);
        }
    }

    bail!("saved view `{name_or_id}` not found")
}

fn latest_run_for_issue(project_dir: &Path, issue_id: &str) -> Result<Option<AgentRun>> {
    let Some(run_dir) = latest_run_dir_for_issue(project_dir, issue_id)? else {
        return Ok(None);
    };
    read_json(&run_dir.join("run.json")).map(Some)
}

fn assistant_check(name: &str, passed: bool, detail: String) -> ReviewAssistantCheck {
    ReviewAssistantCheck {
        name: name.to_string(),
        status: if passed { "pass" } else { "fail" }.to_string(),
        detail,
    }
}

fn aep_protocol_complete(protocol: &AepIssueProtocol) -> bool {
    !protocol.phase.is_empty()
        && !protocol.stop_condition.is_empty()
        && !protocol.fastest_feedback_loop.is_empty()
        && !protocol.vertical_slice.is_empty()
        && !protocol.tracer_bullet_plan.is_empty()
        && !protocol.diagnose_plan.is_empty()
        && !protocol.docs_claim_trace.is_empty()
        && !protocol.boundary_confirmation.is_empty()
}

fn goal_loop_assistant_status(project_dir: &Path) -> Result<(bool, String)> {
    let path = project_dir.join("goal-loop.json");
    if !path.exists() {
        return Ok((
            false,
            ".agentflow/goal-loop.json is missing; run `agentflow goal next`.".to_string(),
        ));
    }
    let state: GoalLoopState = read_json(&path)?;
    Ok((
        state.goal_ready && !state.next_action.is_empty() && !state.recommended_command.is_empty(),
        format!(
            "next action `{}`, command `{}`.",
            state.next_action, state.recommended_command
        ),
    ))
}

fn goal_loop_incomplete_issues(
    project_dir: &Path,
    issues: &[IndexedIssue],
) -> Result<Vec<GoalLoopIssueRef>> {
    let mut refs = Vec::new();
    for issue in issues
        .iter()
        .filter(|issue| issue_status_open(&issue.status))
    {
        refs.push(GoalLoopIssueRef {
            id: issue.id.clone(),
            title: issue.title.clone(),
            status: issue.status.clone(),
            next_action: issue_next_step(project_dir, issue)?.0,
        });
    }
    Ok(refs)
}

fn goal_loop_decision(
    repo: &Path,
    project_dir: &Path,
    readiness: &GoalReadinessSummary,
    scope_state: &AgentScopeState,
    issues: &[IndexedIssue],
) -> Result<GoalLoopDecision> {
    if !readiness.ready {
        return Ok(GoalLoopDecision {
            next_action: "wait-human".to_string(),
            recommended_issue_intent: "补齐 /goal 初始化协议，再重新检查 readiness。".to_string(),
            recommended_command: "agentflow goal bootstrap && agentflow goal check".to_string(),
            rationale: vec![
                "Goal readiness is not ready.".to_string(),
                "ProjectDefinition、ScopeState 或 bootstrap artifacts 缺失时不能进入 executable issue。"
                    .to_string(),
            ],
        });
    }

    if let Some(active_issue_id) = &scope_state.active_issue_id {
        if let Some(issue) = issues.iter().find(|issue| &issue.id == active_issue_id) {
            let (action, command) = issue_next_step(project_dir, issue)?;
            return Ok(GoalLoopDecision {
                next_action: action,
                recommended_issue_intent: format!(
                    "先完成 active issue {}: {}",
                    issue.id, issue.title
                ),
                recommended_command: command,
                rationale: vec![
                    format!("ScopeState active issue is `{active_issue_id}`."),
                    format!(
                        "WIP limit is {}; no new issue can start yet.",
                        scope_state.wip_limit
                    ),
                    "Goal Loop must finish the current issue before recommending new planning."
                        .to_string(),
                ],
            });
        }

        return Ok(GoalLoopDecision {
            next_action: "wait-human".to_string(),
            recommended_issue_intent: format!(
                "修复 scope-state：active issue `{active_issue_id}` 不存在。"
            ),
            recommended_command: "agentflow goal check".to_string(),
            rationale: vec![
                "ScopeState points to an active issue missing from the local index.".to_string(),
                "Human review is required before continuing.".to_string(),
            ],
        });
    }

    if let Some(queue) = active_milestone_queue(project_dir)? {
        if !queue.issue_ids.is_empty() {
            let milestone_issue_ids = queue
                .issue_ids
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>();
            let milestone_open_issues = issues
                .iter()
                .filter(|issue| {
                    issue_status_open(&issue.status)
                        && milestone_issue_ids.contains(&issue.id.as_str())
                })
                .collect::<Vec<_>>();

            let eligibility = workflow_eligibility_snapshot(repo, None)?;
            let eligible_candidates = eligibility
                .candidates
                .iter()
                .filter(|candidate| candidate.eligible)
                .collect::<Vec<_>>();

            if eligible_candidates.len() == 1 {
                let issue_id = &eligible_candidates[0].issue_id;
                let issue = issues
                    .iter()
                    .find(|issue| &issue.id == issue_id)
                    .ok_or_else(|| anyhow!("eligible issue `{issue_id}` missing from index"))?;
                let (action, command) = issue_next_step(project_dir, issue)?;
                return Ok(GoalLoopDecision {
                    next_action: action,
                    recommended_issue_intent: format!(
                        "继续当前 milestone `{}` 的唯一可执行 issue {}: {}",
                        queue.milestone_id, issue.id, issue.title
                    ),
                    recommended_command: command,
                    rationale: vec![
                        "Goal readiness is ready.".to_string(),
                        format!(
                            "Active project `{}` / milestone `{}` queue preflight passed.",
                            queue.project_id, queue.milestone_id
                        ),
                        format!(
                            "Only one eligible issue remains in the active milestone: `{}`.",
                            issue.id
                        ),
                        "Eligibility and lease state were checked before recommending run."
                            .to_string(),
                        "Goal Loop keeps execution inside the active milestone before using global backlog or roadmap fallback."
                            .to_string(),
                    ],
                });
            }

            if eligible_candidates.len() > 1 {
                return Ok(GoalLoopDecision {
                    next_action: "wait-human".to_string(),
                    recommended_issue_intent: format!(
                        "收敛 milestone `{}` 下的唯一 eligible issue。",
                        queue.milestone_id
                    ),
                    recommended_command: "agentflow eligibility".to_string(),
                    rationale: vec![
                        "Goal readiness is ready.".to_string(),
                        format!(
                            "Active milestone `{}` has {} eligible issues.",
                            queue.milestone_id,
                            eligible_candidates.len()
                        ),
                        "WIP=1 requires one eligible issue before AgentFlow can recommend the next run."
                            .to_string(),
                    ],
                });
            }

            if !milestone_open_issues.is_empty() {
                let blocked = eligibility
                    .candidates
                    .iter()
                    .filter(|candidate| !candidate.eligible)
                    .map(|candidate| {
                        format!(
                            "{}: {}",
                            candidate.issue_id,
                            if candidate.failure_reasons.is_empty() {
                                "not eligible".to_string()
                            } else {
                                candidate.failure_reasons.join(", ")
                            }
                        )
                    })
                    .collect::<Vec<_>>();
                return Ok(GoalLoopDecision {
                    next_action: "wait-human".to_string(),
                    recommended_issue_intent: format!(
                        "修复 milestone `{}` 下 issue 的 eligibility 失败原因。",
                        queue.milestone_id
                    ),
                    recommended_command: "agentflow eligibility".to_string(),
                    rationale: vec![
                        "Goal readiness is ready.".to_string(),
                        format!(
                            "Active milestone `{}` has open issues but none is eligible.",
                            queue.milestone_id
                        ),
                        blocked.join("; "),
                    ],
                });
            }

            let outside_open_issues = issues
                .iter()
                .filter(|issue| {
                    issue_status_open(&issue.status)
                        && !milestone_issue_ids.contains(&issue.id.as_str())
                })
                .collect::<Vec<_>>();
            if !outside_open_issues.is_empty() {
                return Ok(GoalLoopDecision {
                    next_action: "wait-human".to_string(),
                    recommended_issue_intent: format!(
                        "确认 active milestone `{}` 后再处理非当前 milestone 的未完成 issue。",
                        queue.milestone_id
                    ),
                    recommended_command: "agentflow projects".to_string(),
                    rationale: vec![
                        "Goal readiness is ready.".to_string(),
                        format!(
                            "Active milestone `{}` has no open issue, but {} open issue(s) exist outside it.",
                            queue.milestone_id,
                            outside_open_issues.len()
                        ),
                        "MVP execution stays within Project -> Milestone -> Issue instead of a flat global queue."
                            .to_string(),
                    ],
                });
            }
        }
    }

    if let Some(issue) = issues.iter().find(|issue| issue_status_open(&issue.status)) {
        let (action, command) = issue_next_step(project_dir, issue)?;
        return Ok(GoalLoopDecision {
            next_action: action,
            recommended_issue_intent: format!("继续未完成 issue {}: {}", issue.id, issue.title),
            recommended_command: command,
            rationale: vec![
                "Goal readiness is ready.".to_string(),
                format!(
                    "No active issue is set; first incomplete issue is `{}`.",
                    issue.id
                ),
                "Roadmap does not authorize a new issue while an existing issue remains open."
                    .to_string(),
            ],
        });
    }

    let closure = project_closure_state_snapshot(repo)?;
    if closure.closure_state != "active" && closure.closure_state != "done" {
        let code_audit_snapshot_missing = closure.closure_state == "audit-ready"
            && !project_code_audit_snapshot_path(project_dir).exists();
        let docs_refresh_snapshot_missing = project_code_audit_snapshot_path(project_dir).exists()
            && !project_docs_refresh_snapshot_path(project_dir).exists();
        if code_audit_snapshot_missing || docs_refresh_snapshot_missing {
            let next_action = if code_audit_snapshot_missing {
                "project-code-audit"
            } else {
                "project-docs-refresh"
            };
            let recommended_command = if code_audit_snapshot_missing {
                "agentflow project code-audit".to_string()
            } else {
                "agentflow project docs-refresh".to_string()
            };
            let recommended_issue_intent = if code_audit_snapshot_missing {
                "生成 Project Code Audit Snapshot v0 只读审计输入。".to_string()
            } else {
                "生成 Root Docs Refresh Snapshot v0 只读文档刷新输入。".to_string()
            };
            return Ok(GoalLoopDecision {
                next_action: next_action.to_string(),
                recommended_issue_intent,
                recommended_command,
                rationale: vec![
                    "Goal readiness is ready.".to_string(),
                    "No active issue is set and all known issues are completed.".to_string(),
                    format!(
                        "Project closure state is `{}`; Project cannot move to done directly.",
                        closure.closure_state
                    ),
                    if code_audit_snapshot_missing {
                        "Goal Loop recommends the read-only code audit snapshot before final audit approval."
                            .to_string()
                    } else {
                        "Goal Loop recommends the read-only docs refresh snapshot before final docs refresh approval."
                            .to_string()
                    },
                ],
            });
        }
    }

    if let Some(candidate) = project_aware_candidate_intent(project_dir)? {
        return Ok(GoalLoopDecision {
            next_action: "plan".to_string(),
            recommended_issue_intent: candidate.intent.clone(),
            recommended_command: format!("agentflow plan \"{}\"", candidate.intent),
            rationale: vec![
                "Goal readiness is ready.".to_string(),
                "No active issue is set.".to_string(),
                "All known issues are completed; Goal Loop can use the active project candidate."
                    .to_string(),
                format!(
                    "Active project `{}` produced next issue intent from {}.",
                    candidate.project_id, candidate.source
                ),
            ],
        });
    }

    let intent = current_candidate_intent(repo, project_dir)?;
    Ok(GoalLoopDecision {
        next_action: "plan".to_string(),
        recommended_issue_intent: intent.clone(),
        recommended_command: format!("agentflow plan \"{intent}\""),
        rationale: vec![
            "Goal readiness is ready.".to_string(),
            "No active issue is set.".to_string(),
            "All known issues are completed; Goal Loop can recommend the next roadmap intent."
                .to_string(),
        ],
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProjectAwareCandidate {
    intent: String,
    project_id: String,
    source: String,
}

fn project_aware_candidate_intent(project_dir: &Path) -> Result<Option<ProjectAwareCandidate>> {
    let project_id = active_project_id_from_seed(project_dir)?
        .unwrap_or_else(|| "agentflow-local-execution".to_string());
    validate_project_link_id(&project_id)?;

    let project_path = project_dir
        .join("projects")
        .join(format!("{project_id}.json"));
    if !project_path.exists() {
        return Ok(None);
    }

    let project: serde_json::Value = read_json(&project_path)?;
    let active_milestone_id = json_string_field(&project, "activeMilestoneId");
    if let Some(milestone_id) = active_milestone_id.as_deref() {
        if let Some(intent) = milestone_next_issue_intent(&project, milestone_id) {
            return Ok(Some(ProjectAwareCandidate {
                intent,
                project_id,
                source: format!("active milestone `{milestone_id}`"),
            }));
        }
    }

    Ok(
        json_string_field(&project, "nextIssueIntent").map(|intent| ProjectAwareCandidate {
            intent,
            project_id,
            source: format!("{}", project_path.display()),
        }),
    )
}

fn active_milestone_queue(project_dir: &Path) -> Result<Option<ActiveMilestoneQueue>> {
    let project_id = active_project_id_from_seed(project_dir)?
        .unwrap_or_else(|| "agentflow-local-execution".to_string());
    validate_project_link_id(&project_id)?;

    let project_path = project_dir
        .join("projects")
        .join(format!("{project_id}.json"));
    if !project_path.exists() {
        return Ok(None);
    }

    let project: serde_json::Value = read_json(&project_path)?;
    let Some(milestone_id) = json_string_field(&project, "activeMilestoneId") else {
        return Ok(None);
    };

    let issue_ids = project
        .get("milestones")
        .and_then(serde_json::Value::as_array)
        .and_then(|milestones| {
            milestones.iter().find(|milestone| {
                milestone
                    .get("id")
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|id| id == milestone_id)
            })
        })
        .map(|milestone| json_string_array_field(milestone, "issueIds"))
        .unwrap_or_default();

    Ok(Some(ActiveMilestoneQueue {
        project_id,
        milestone_id,
        issue_ids,
    }))
}

fn active_project_id_from_seed(project_dir: &Path) -> Result<Option<String>> {
    let workspace_path = project_dir.join("workspace.json");
    if !workspace_path.exists() {
        return Ok(None);
    }
    let workspace: serde_json::Value = read_json(&workspace_path)?;
    Ok(json_string_field(&workspace, "activeProjectId"))
}

fn milestone_next_issue_intent(project: &serde_json::Value, milestone_id: &str) -> Option<String> {
    project
        .get("milestones")?
        .as_array()?
        .iter()
        .find(|milestone| {
            milestone
                .get("id")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|id| id == milestone_id)
        })
        .and_then(|milestone| json_string_field(milestone, "nextIssueIntent"))
}

fn json_string_field(value: &serde_json::Value, field: &str) -> Option<String> {
    value
        .get(field)
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn json_string_array_field(value: &serde_json::Value, field: &str) -> Vec<String> {
    value
        .get(field)
        .and_then(serde_json::Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(serde_json::Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn local_project_seed_snapshot(
    project_dir: &Path,
    derived_workspace: LocalWorkspace,
    derived_team: LocalTeam,
    derived_project: LocalProject,
    issue_refs: &[LocalProjectIssueRef],
    selection: &GoalLoopSelection,
) -> Result<(LocalWorkspace, Vec<LocalTeam>, Vec<LocalProject>)> {
    let workspace_path = project_dir.join("workspace.json");
    if !workspace_path.exists() {
        return Ok((derived_workspace, vec![derived_team], vec![derived_project]));
    }

    let workspace_seed: serde_json::Value = read_json(&workspace_path)?;
    let mut workspace = LocalWorkspace {
        version: json_string_field(&workspace_seed, "version")
            .unwrap_or_else(|| derived_workspace.version.clone()),
        id: json_string_field(&workspace_seed, "id")
            .unwrap_or_else(|| derived_workspace.id.clone()),
        name: json_string_field(&workspace_seed, "name")
            .unwrap_or_else(|| derived_workspace.name.clone()),
        default_team_id: json_string_field(&workspace_seed, "defaultTeamId")
            .unwrap_or_else(|| derived_workspace.default_team_id.clone()),
        active_project_id: json_string_field(&workspace_seed, "activeProjectId")
            .unwrap_or_else(|| derived_workspace.active_project_id.clone()),
        team_ids: json_string_array_field(&workspace_seed, "teamIds"),
        project_ids: json_string_array_field(&workspace_seed, "projectIds"),
        issue_count: issue_refs.len(),
        completed_issue_count: issue_refs
            .iter()
            .filter(|issue| issue_state_done(&issue.status))
            .count(),
    };
    if workspace.team_ids.is_empty() {
        workspace.team_ids = derived_workspace.team_ids;
    }
    if workspace.project_ids.is_empty() {
        workspace.project_ids = derived_workspace.project_ids;
    }

    let teams = workspace
        .team_ids
        .iter()
        .map(|team_id| {
            read_seed_team(project_dir, team_id, &derived_team).unwrap_or_else(|| {
                let mut fallback = derived_team.clone();
                fallback.id = team_id.clone();
                fallback
            })
        })
        .collect::<Vec<_>>();
    let projects = workspace
        .project_ids
        .iter()
        .map(|project_id| {
            read_seed_project(
                project_dir,
                project_id,
                &derived_project,
                issue_refs,
                selection,
            )
            .unwrap_or_else(|| {
                let mut fallback = derived_project.clone();
                fallback.id = project_id.clone();
                fallback
            })
        })
        .collect::<Vec<_>>();

    Ok((workspace, teams, projects))
}

fn read_seed_team(project_dir: &Path, team_id: &str, derived: &LocalTeam) -> Option<LocalTeam> {
    let path = project_dir.join("teams").join(format!("{team_id}.json"));
    if !path.exists() {
        return None;
    }
    let seed: serde_json::Value = read_json(&path).ok()?;
    Some(LocalTeam {
        version: json_string_field(&seed, "version").unwrap_or_else(|| derived.version.clone()),
        id: json_string_field(&seed, "id").unwrap_or_else(|| team_id.to_string()),
        name: json_string_field(&seed, "name").unwrap_or_else(|| derived.name.clone()),
        workflow: non_empty_or_default(
            json_string_array_field(&seed, "workflow"),
            &derived.workflow,
        ),
        default_validation_commands: non_empty_or_default(
            json_string_array_field(&seed, "defaultValidationCommands"),
            &derived.default_validation_commands,
        ),
        wip_limit: seed
            .get("wipLimit")
            .and_then(serde_json::Value::as_u64)
            .and_then(|value| u32::try_from(value).ok())
            .unwrap_or(derived.wip_limit),
        issue_ids: json_string_array_field(&seed, "issueIds"),
    })
}

fn read_seed_project(
    project_dir: &Path,
    project_id: &str,
    derived: &LocalProject,
    issue_refs: &[LocalProjectIssueRef],
    selection: &GoalLoopSelection,
) -> Option<LocalProject> {
    let path = project_dir
        .join("projects")
        .join(format!("{project_id}.json"));
    if !path.exists() {
        return None;
    }
    let seed: serde_json::Value = read_json(&path).ok()?;
    let issue_ids = json_string_array_field(&seed, "issueIds");
    let completed_issue_ids = completed_issue_ids(&issue_ids, issue_refs);
    let milestones = seed
        .get("milestones")
        .and_then(serde_json::Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|milestone| seed_milestone(milestone, issue_refs))
                .collect::<Vec<_>>()
        })
        .filter(|items| !items.is_empty())
        .unwrap_or_else(|| derived.milestones.clone());

    let status = json_string_field(&seed, "status").unwrap_or_else(|| derived.status.clone());

    Some(LocalProject {
        version: json_string_field(&seed, "version").unwrap_or_else(|| derived.version.clone()),
        id: json_string_field(&seed, "id").unwrap_or_else(|| project_id.to_string()),
        name: json_string_field(&seed, "name").unwrap_or_else(|| derived.name.clone()),
        canonical_status: canonical_project_status_string(&status),
        status,
        goal: json_string_field(&seed, "goal").unwrap_or_else(|| derived.goal.clone()),
        team_ids: non_empty_or_default(
            json_string_array_field(&seed, "teamIds"),
            &derived.team_ids,
        ),
        active_milestone_id: json_string_field(&seed, "activeMilestoneId")
            .unwrap_or_else(|| derived.active_milestone_id.clone()),
        milestones,
        issue_count: issue_ids.len(),
        completed_issue_count: completed_issue_ids.len(),
        issue_ids,
        next_issue_intent: if seed.get("nextIssueIntent").is_some() {
            json_string_field(&seed, "nextIssueIntent")
        } else {
            selection.next_issue_intent.clone()
        },
        recommended_command: json_string_field(&seed, "recommendedCommand")
            .or_else(|| Some(selection.recommended_command.clone())),
    })
}

fn seed_milestone(
    milestone: &serde_json::Value,
    issue_refs: &[LocalProjectIssueRef],
) -> Option<LocalMilestone> {
    let id = json_string_field(milestone, "id")?;
    let issue_ids = json_string_array_field(milestone, "issueIds");
    let explicit_completed = json_string_array_field(milestone, "completedIssueIds");
    let completed_issue_ids = if explicit_completed.is_empty() {
        completed_issue_ids(&issue_ids, issue_refs)
    } else {
        explicit_completed
    };

    let progress = milestone_derived_progress(&issue_ids, issue_refs);
    let description = json_string_field(milestone, "description")
        .or_else(|| json_string_field(milestone, "goal"));
    let target = json_string_field(milestone, "target")
        .or_else(|| json_string_field(milestone, "nextIssueIntent"));

    Some(LocalMilestone {
        id,
        name: json_string_field(milestone, "name")
            .or_else(|| json_string_field(milestone, "title"))
            .unwrap_or_else(|| "Milestone".to_string()),
        description,
        sort_order: milestone
            .get("sortOrder")
            .and_then(serde_json::Value::as_u64)
            .and_then(|value| usize::try_from(value).ok())
            .unwrap_or(0),
        target,
        status: json_string_field(milestone, "status").unwrap_or_else(|| "planned".to_string()),
        progress,
        issue_ids,
        completed_issue_ids,
        next_issue_intent: json_string_field(milestone, "nextIssueIntent"),
    })
}

fn completed_issue_ids(issue_ids: &[String], issue_refs: &[LocalProjectIssueRef]) -> Vec<String> {
    issue_ids
        .iter()
        .filter(|issue_id| {
            issue_refs
                .iter()
                .any(|issue| issue.id == **issue_id && issue_state_done(&issue.status))
        })
        .cloned()
        .collect()
}

fn milestone_derived_progress(
    issue_ids: &[String],
    issue_refs: &[LocalProjectIssueRef],
) -> MilestoneDerivedProgress {
    let mut done_issue_count = 0usize;
    let mut canceled_issue_count = 0usize;

    for issue_id in issue_ids {
        if let Some(issue) = issue_refs.iter().find(|issue| issue.id == *issue_id) {
            match canonical_issue_status(&issue.status) {
                IssueStatus::Done => done_issue_count += 1,
                IssueStatus::Canceled => canceled_issue_count += 1,
                _ => {}
            }
        }
    }

    let total_issue_count = issue_ids.len();
    let non_canceled_issue_count = total_issue_count.saturating_sub(canceled_issue_count);
    let percent = if non_canceled_issue_count == 0 {
        0
    } else {
        ((done_issue_count * 100) / non_canceled_issue_count).min(100) as u8
    };

    MilestoneDerivedProgress {
        done_issue_count,
        total_issue_count,
        non_canceled_issue_count,
        canceled_issue_count,
        percent,
    }
}

fn non_empty_or_default(values: Vec<String>, fallback: &[String]) -> Vec<String> {
    if values.is_empty() {
        fallback.to_vec()
    } else {
        values
    }
}

fn append_unique_json_string_array(
    value: &mut serde_json::Value,
    field: &str,
    item: &str,
) -> Result<()> {
    let object = value
        .as_object_mut()
        .ok_or_else(|| anyhow!("seed root must be a JSON object"))?;
    object
        .entry(field.to_string())
        .or_insert_with(|| serde_json::Value::Array(Vec::new()));
    let array = object
        .get_mut(field)
        .and_then(serde_json::Value::as_array_mut)
        .ok_or_else(|| anyhow!("seed field `{field}` must be an array"))?;
    if !array
        .iter()
        .any(|value| value.as_str().is_some_and(|value| value == item))
    {
        array.push(serde_json::Value::String(item.to_string()));
    }
    Ok(())
}

fn append_issue_to_seed_milestone(
    project_seed: &mut serde_json::Value,
    milestone_id: &str,
    issue_id: &str,
    intent: &str,
) -> Result<()> {
    let milestones = project_seed
        .get_mut("milestones")
        .and_then(serde_json::Value::as_array_mut)
        .ok_or_else(|| anyhow!("project seed milestones must be an array"))?;
    let milestone = milestones
        .iter_mut()
        .find(|milestone| {
            milestone
                .get("id")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|id| id == milestone_id)
        })
        .ok_or_else(|| anyhow!("active milestone `{milestone_id}` is missing from project seed"))?;
    append_unique_json_string_array(milestone, "issueIds", issue_id)?;

    let planned_intent = intent.trim();
    if json_string_field(milestone, "nextIssueIntent").as_deref() == Some(planned_intent) {
        if let Some(object) = milestone.as_object_mut() {
            object.insert("nextIssueIntent".to_string(), serde_json::Value::Null);
        }
        if json_string_field(project_seed, "nextIssueIntent").as_deref() == Some(planned_intent) {
            if let Some(object) = project_seed.as_object_mut() {
                object.insert("nextIssueIntent".to_string(), serde_json::Value::Null);
            }
        }
    }

    Ok(())
}

fn product_feature_creation_snapshot(
    repo: &Path,
    project_dir: &Path,
    draft: ProductFeatureDraft,
    write: bool,
) -> Result<ProductFeatureCreationSnapshot> {
    let settings: Settings = read_json(&project_dir.join("settings.json"))?;
    let index: ProjectIndex = read_json(&project_dir.join("index.json"))?;
    let draft = normalize_product_feature_draft(draft)?;
    validate_project_link_id(&draft.team_id)?;

    let project_id =
        unique_feature_project_id(project_dir, &draft.project_title, index.next_issue_number);
    let issue_start = index.next_issue_number;
    let milestones = product_feature_milestone_drafts(issue_start);
    let milestone_ids = milestones
        .iter()
        .map(|milestone| milestone.id.clone())
        .collect::<Vec<_>>();
    let issue_ids = milestones
        .iter()
        .map(|milestone| milestone.issue_id.clone())
        .collect::<Vec<_>>();
    let project = ProductFeatureProject {
        id: project_id,
        title: draft.project_title.clone(),
        team_id: draft.team_id.clone(),
        active_milestone_id: "project-charter".to_string(),
        milestone_ids,
        issue_ids,
    };
    let issues = product_feature_issue_drafts(&draft, &project, &milestones, &settings);

    Ok(ProductFeatureCreationSnapshot {
        version: VERSION.to_string(),
        initialized: true,
        project_root: repo.display().to_string(),
        mode: if write {
            "write".to_string()
        } else {
            "preview".to_string()
        },
        draft,
        project,
        milestones,
        issues,
        writes_required: true,
        written_paths: Vec::new(),
        recommended_command: "agentflow goal next".to_string(),
        sources: vec![
            ".agentflow/settings.json".to_string(),
            ".agentflow/index.json".to_string(),
            ".agentflow/workspace.json".to_string(),
            ".agentflow/teams/{teamId}.json".to_string(),
            ".agentflow/projects/{featureProjectId}.json".to_string(),
            ".agentflow/issues/ISSUE-XXXX.json".to_string(),
        ],
        boundary: WorkbenchBoundary {
            read_only: !write,
            disallowed_actions: vec![
                "call-model".to_string(),
                "create-remote-issue".to_string(),
                "create-remote-pr".to_string(),
                "desktop-create-feature".to_string(),
                "mark-project-done".to_string(),
                "bypass-issue-contract".to_string(),
            ],
        },
    })
}

fn normalize_product_feature_draft(mut draft: ProductFeatureDraft) -> Result<ProductFeatureDraft> {
    draft.feature_goal = draft.feature_goal.trim().to_string();
    if draft.feature_goal.is_empty() {
        bail!("feature goal is required");
    }
    draft.team_id = if draft.team_id.trim().is_empty() {
        "core".to_string()
    } else {
        slugify(&draft.team_id)
    };
    if draft.team_id.is_empty() {
        draft.team_id = "core".to_string();
    }
    draft.project_title = if draft.project_title.trim().is_empty() {
        draft.feature_goal.clone()
    } else {
        draft.project_title.trim().to_string()
    };
    draft.risk_level = if draft.risk_level.trim().is_empty() {
        "medium".to_string()
    } else {
        draft.risk_level.trim().to_string()
    };
    draft.non_goals = non_empty_or_default(
        draft
            .non_goals
            .into_iter()
            .map(|item| item.trim().to_string())
            .filter(|item| !item.is_empty())
            .collect(),
        &[
            "不调用模型自动拆解。".to_string(),
            "不创建远程 PR / GitHub issue / Linear issue。".to_string(),
            "不从 Desktop 执行创建或运行。".to_string(),
            "不绕过 IssueContract 或 Workflow Control Core。".to_string(),
        ],
    );
    draft.success_criteria = non_empty_or_default(
        draft
            .success_criteria
            .into_iter()
            .map(|item| item.trim().to_string())
            .filter(|item| !item.is_empty())
            .collect(),
        &[
            "新 Project 成为 active project。".to_string(),
            "默认 Milestones 和 IssueContracts 已写入本地事实源。".to_string(),
            "goal next 能推荐第一条受控执行 issue。".to_string(),
            "eligibility 能解释第一条 issue 的 ready / eligible 状态。".to_string(),
        ],
    );
    draft.scope_boundaries = non_empty_or_default(
        draft
            .scope_boundaries
            .into_iter()
            .map(|item| item.trim().to_string())
            .filter(|item| !item.is_empty())
            .collect(),
        &[
            "仅写 `.agentflow/` 本地事实源。".to_string(),
            "只创建 Product Feature 对应的 Project、Milestones 和 IssueContracts。".to_string(),
            "后续执行必须继续通过 goal next / eligibility / lease / run 链路。".to_string(),
        ],
    );
    Ok(draft)
}

fn unique_feature_project_id(project_dir: &Path, title: &str, seed: u32) -> String {
    let mut base = slugify(title);
    if base.is_empty() {
        base = format!("feature-{seed:04}");
    }
    let mut candidate = base.clone();
    let mut suffix = 2;
    while project_dir
        .join("projects")
        .join(format!("{candidate}.json"))
        .exists()
    {
        candidate = format!("{base}-{suffix}");
        suffix += 1;
    }
    candidate
}

fn product_feature_milestone_drafts(issue_start: u32) -> Vec<ProductFeatureMilestoneDraft> {
    [
        (
            "project-charter",
            "Project Charter",
            "确认产品功能目标、边界、成功标准和风险。",
        ),
        (
            "milestone-plan",
            "Milestone Plan",
            "把产品功能拆成可审核的阶段边界。",
        ),
        (
            "issue-contracts",
            "Issue Contracts",
            "为每个阶段生成最小可执行 IssueContract。",
        ),
        (
            "validation-evidence",
            "Validation / Evidence",
            "定义验证命令、证据要求和回滚计划。",
        ),
    ]
    .into_iter()
    .enumerate()
    .map(|(offset, (id, title, goal))| ProductFeatureMilestoneDraft {
        id: id.to_string(),
        title: title.to_string(),
        goal: goal.to_string(),
        status: if offset == 0 { "active" } else { "planned" }.to_string(),
        issue_id: format!("ISSUE-{:04}", issue_start + offset as u32),
    })
    .collect()
}

fn product_feature_issue_drafts(
    draft: &ProductFeatureDraft,
    project: &ProductFeatureProject,
    milestones: &[ProductFeatureMilestoneDraft],
    settings: &Settings,
) -> Vec<ProductFeatureIssueDraft> {
    milestones
        .iter()
        .map(|milestone| {
            let title = format!("{}: {}", milestone.title, project.title);
            ProductFeatureIssueDraft {
                id: milestone.issue_id.clone(),
                title,
                status: IssueStatus::Todo.as_str().to_string(),
                intent: format!("{} -> {}", draft.feature_goal, milestone.goal),
                milestone_id: milestone.id.clone(),
                scope: product_feature_issue_scope(draft, milestone),
                non_goals: draft.non_goals.clone(),
                validation_commands: settings.validation_commands.clone(),
                evidence_requirements: product_feature_evidence_requirements(),
                rollback_plan: product_feature_rollback_plan(),
                risk_level: draft.risk_level.clone(),
            }
        })
        .collect()
}

fn product_feature_issue_scope(
    draft: &ProductFeatureDraft,
    milestone: &ProductFeatureMilestoneDraft,
) -> Vec<String> {
    let mut scope = vec![
        format!("产品功能目标：{}", draft.feature_goal),
        format!("当前阶段：{}。{}", milestone.title, milestone.goal),
        "只推进本 IssueContract 描述的最小交付物。".to_string(),
    ];
    scope.extend(
        draft
            .scope_boundaries
            .iter()
            .map(|boundary| format!("边界：{boundary}")),
    );
    scope.extend(
        draft
            .success_criteria
            .iter()
            .map(|criterion| format!("成功标准：{criterion}")),
    );
    scope
}

fn product_feature_evidence_requirements() -> Vec<String> {
    [
        "feature-creation-summary",
        "command-output",
        "validation-output",
        "diff-summary",
        "acceptance-criteria-coverage",
        "rollback-plan",
        "known-limitations",
        "aep-contract-checklist",
        "docs-claim-trace",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

fn product_feature_rollback_plan() -> Vec<String> {
    [
        "如果 feature project 创建错误，回退本次新增 project、issue、workspace/team/index 更新。",
        "如果后续 run 失败，释放 lease 并保留 evidence，不能直接标记 Project done。",
        "如果 scope 需要扩大，停止执行并由 Human 更新 Project Charter 或 IssueContract。",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

fn product_feature_issue_contracts(
    project_dir: &Path,
    draft: &ProductFeatureDraft,
    project: &ProductFeatureProject,
    issues: &[ProductFeatureIssueDraft],
) -> Result<Vec<IssueContract>> {
    let settings: Settings = read_json(&project_dir.join("settings.json"))?;
    Ok(issues
        .iter()
        .map(|issue| IssueContract {
            id: issue.id.clone(),
            title: issue.title.clone(),
            status: issue.status.clone(),
            intent: issue.intent.clone(),
            risk_level: issue.risk_level.clone(),
            scope: issue.scope.clone(),
            non_goals: issue.non_goals.clone(),
            context: IssueContext {
                repo: ".".to_string(),
                files: vec![
                    ".agentflow/workspace.json".to_string(),
                    format!(".agentflow/projects/{}.json", project.id),
                    format!(".agentflow/teams/{}.json", draft.team_id),
                ],
            },
            execution_plan: vec![
                "读取 Product Feature Project 和当前 Milestone。".to_string(),
                "确认 IssueContract 的 scope、non-goals、riskLevel 和 rollbackPlan。".to_string(),
                "只在当前 issue 授权范围内完成最小变更。".to_string(),
                "运行 validation commands 并记录输出。".to_string(),
                "生成 evidence、review 和 project update。".to_string(),
            ],
            validation: ValidationSpec {
                commands: issue.validation_commands.clone(),
            },
            evidence_requirements: issue.evidence_requirements.clone(),
            rollback_plan: issue.rollback_plan.clone(),
            human_gate: HumanGate {
                before_file_edits: false,
                before_external_network: true,
            },
            project_link: Some(IssueProjectLink {
                team_id: draft.team_id.clone(),
                project_id: project.id.clone(),
                milestone_id: issue.milestone_id.clone(),
                link_source: "product-feature-creation-flow-v0".to_string(),
            }),
            aep: aep_issue_protocol(&settings),
        })
        .collect())
}

fn product_feature_project_seed(snapshot: &ProductFeatureCreationSnapshot) -> serde_json::Value {
    let milestones = snapshot
        .milestones
        .iter()
        .enumerate()
        .map(|(offset, milestone)| {
            serde_json::json!({
                "id": milestone.id.clone(),
                "name": milestone.title.clone(),
                "title": milestone.title.clone(),
                "description": milestone.goal.clone(),
                "sortOrder": offset + 1,
                "target": milestone.goal.clone(),
                "status": milestone.status.clone(),
                "goal": milestone.goal.clone(),
                "issueIds": [milestone.issue_id.clone()],
                "completedIssueIds": [],
                "nextIssueIntent": null,
            })
        })
        .collect::<Vec<_>>();

    serde_json::json!({
        "version": VERSION,
        "id": snapshot.project.id.clone(),
        "name": snapshot.project.title.clone(),
        "status": "active",
        "goal": snapshot.draft.feature_goal.clone(),
        "teamIds": [snapshot.project.team_id.clone()],
        "activeMilestoneId": snapshot.project.active_milestone_id.clone(),
        "milestones": milestones,
        "issueIds": snapshot.project.issue_ids.clone(),
        "nextIssueIntent": null,
        "productFeature": {
            "generatedFrom": "product-feature-creation-flow-v0",
            "nonGoals": snapshot.draft.non_goals.clone(),
            "successCriteria": snapshot.draft.success_criteria.clone(),
            "riskLevel": snapshot.draft.risk_level.clone(),
            "scopeBoundaries": snapshot.draft.scope_boundaries.clone()
        },
        "source": {
            "kind": "product-feature-creation-flow",
            "generatedFrom": "agentflow feature create"
        }
    })
}

fn product_feature_workspace_seed(
    project_dir: &Path,
    draft: &ProductFeatureDraft,
    project: &ProductFeatureProject,
) -> Result<serde_json::Value> {
    let workspace_path = project_dir.join("workspace.json");
    let mut workspace = if workspace_path.exists() {
        read_json(&workspace_path)?
    } else {
        serde_json::json!({
            "version": VERSION,
            "id": "default",
            "name": "AgentFlow",
            "defaultTeamId": draft.team_id.clone(),
            "activeProjectId": project.id.clone(),
            "teamIds": [],
            "projectIds": [],
            "source": {
                "kind": "product-feature-creation-flow",
                "generatedFrom": "agentflow feature create"
            }
        })
    };
    if let Some(object) = workspace.as_object_mut() {
        object.insert(
            "activeProjectId".to_string(),
            serde_json::Value::String(project.id.clone()),
        );
        object
            .entry("version".to_string())
            .or_insert_with(|| serde_json::Value::String(VERSION.to_string()));
        object
            .entry("id".to_string())
            .or_insert_with(|| serde_json::Value::String("default".to_string()));
        object
            .entry("name".to_string())
            .or_insert_with(|| serde_json::Value::String("AgentFlow".to_string()));
        object
            .entry("defaultTeamId".to_string())
            .or_insert_with(|| serde_json::Value::String(draft.team_id.clone()));
    }
    append_unique_json_string_array(&mut workspace, "teamIds", &draft.team_id)?;
    append_unique_json_string_array(&mut workspace, "projectIds", &project.id)?;
    Ok(workspace)
}

fn product_feature_team_seed(
    project_dir: &Path,
    draft: &ProductFeatureDraft,
    project: &ProductFeatureProject,
    issue_ids: &[String],
) -> Result<serde_json::Value> {
    let team_path = project_dir.join(format!("teams/{}.json", draft.team_id));
    let mut team = if team_path.exists() {
        read_json(&team_path)?
    } else {
        serde_json::json!({
            "version": VERSION,
            "id": draft.team_id.clone(),
            "name": draft.team_id.clone(),
            "workflow": ["backlog", "todo", "in_progress", "in_review", "done", "canceled"],
            "defaultValidationCommands": [],
            "wipLimit": 1,
            "issueIds": [],
            "projectIds": [],
            "source": {
                "kind": "product-feature-creation-flow",
                "generatedFrom": "agentflow feature create"
            }
        })
    };
    if let Some(object) = team.as_object_mut() {
        object
            .entry("version".to_string())
            .or_insert_with(|| serde_json::Value::String(VERSION.to_string()));
        object
            .entry("id".to_string())
            .or_insert_with(|| serde_json::Value::String(draft.team_id.clone()));
        object
            .entry("name".to_string())
            .or_insert_with(|| serde_json::Value::String(draft.team_id.clone()));
        object.entry("workflow".to_string()).or_insert_with(|| {
            serde_json::json!([
                "backlog",
                "todo",
                "in_progress",
                "in_review",
                "done",
                "canceled"
            ])
        });
        object
            .entry("wipLimit".to_string())
            .or_insert_with(|| serde_json::json!(1));
    }
    append_unique_json_string_array(&mut team, "projectIds", &project.id)?;
    for issue_id in issue_ids {
        append_unique_json_string_array(&mut team, "issueIds", issue_id)?;
    }
    Ok(team)
}

fn normalize_team_draft(mut draft: TeamDraft) -> Result<TeamDraft> {
    draft.name = draft.name.trim().to_string();
    if draft.name.is_empty() {
        bail!("team name is required");
    }
    draft.team_id = normalize_optional_seed_id(draft.team_id.as_deref())?;
    Ok(draft)
}

fn normalize_project_draft(project_dir: &Path, mut draft: ProjectDraft) -> Result<ProjectDraft> {
    draft.title = draft.title.trim().to_string();
    if draft.title.is_empty() {
        bail!("project title is required");
    }
    draft.project_id = normalize_optional_seed_id(draft.project_id.as_deref())?;
    draft.team_id = match draft
        .team_id
        .as_deref()
        .map(str::trim)
        .filter(|id| !id.is_empty())
    {
        Some(team_id) => {
            let team_id = slugify(team_id);
            validate_project_link_id(&team_id)?;
            Some(team_id)
        }
        None => Some(default_team_id(project_dir)?),
    };
    draft.status = parse_project_create_status(&draft.status)?
        .as_str()
        .to_string();
    draft.goal = draft
        .goal
        .map(|goal| goal.trim().to_string())
        .filter(|goal| !goal.is_empty())
        .or_else(|| Some(draft.title.clone()));
    Ok(draft)
}

fn normalize_milestone_draft(
    project_dir: &Path,
    mut draft: MilestoneDraft,
) -> Result<MilestoneDraft> {
    draft.title = draft.title.trim().to_string();
    if draft.title.is_empty() {
        bail!("milestone title is required");
    }
    draft.milestone_id = normalize_optional_seed_id(draft.milestone_id.as_deref())?;
    draft.project_id = match draft
        .project_id
        .as_deref()
        .map(str::trim)
        .filter(|id| !id.is_empty())
    {
        Some(project_id) => {
            let project_id = slugify(project_id);
            validate_project_link_id(&project_id)?;
            Some(project_id)
        }
        None => Some(default_project_id(project_dir)?),
    };
    draft.description = trim_optional(draft.description);
    draft.target = trim_optional(draft.target).or_else(|| draft.description.clone());
    Ok(draft)
}

fn normalize_issue_draft(project_dir: &Path, mut draft: IssueDraft) -> Result<IssueDraft> {
    draft.title = draft.title.trim().to_string();
    if draft.title.is_empty() {
        bail!("issue title is required");
    }

    let project_id = match draft
        .project_id
        .as_deref()
        .map(str::trim)
        .filter(|id| !id.is_empty())
    {
        Some(project_id) => {
            let project_id = slugify(project_id);
            validate_project_link_id(&project_id)?;
            project_id
        }
        None => default_project_id(project_dir)?,
    };
    let project_path = project_dir
        .join("projects")
        .join(format!("{project_id}.json"));
    let project_seed: serde_json::Value = read_json(&project_path)?;
    let milestone_id = match draft
        .milestone_id
        .as_deref()
        .map(str::trim)
        .filter(|id| !id.is_empty())
    {
        Some(milestone_id) => {
            let milestone_id = slugify(milestone_id);
            validate_project_link_id(&milestone_id)?;
            milestone_id
        }
        None => default_milestone_id(&project_seed)?,
    };
    let team_id = match draft
        .team_id
        .as_deref()
        .map(str::trim)
        .filter(|id| !id.is_empty())
    {
        Some(team_id) => {
            let team_id = slugify(team_id);
            validate_project_link_id(&team_id)?;
            team_id
        }
        None => default_team_id_for_project(project_dir, &project_seed)?,
    };

    draft.project_id = Some(project_id);
    draft.milestone_id = Some(milestone_id);
    draft.team_id = Some(team_id);
    draft.risk_level = if draft.risk_level.trim().is_empty() {
        "medium".to_string()
    } else {
        draft.risk_level.trim().to_string()
    };
    draft.scope = non_empty_or_default(
        trimmed_vec(draft.scope),
        &[
            format!("创建本地 IssueContract：{}", draft.title),
            "只写 `.agentflow/` 本地事实源。".to_string(),
            "不自动执行 run / verify / review。".to_string(),
        ],
    );
    draft.non_goals = non_empty_or_default(
        trimmed_vec(draft.non_goals),
        &[
            "不调用模型。".to_string(),
            "不创建远程 PR / GitHub issue / Linear issue。".to_string(),
            "不绕过 preview / confirmation gate。".to_string(),
        ],
    );
    let settings: Settings = read_json(&project_dir.join("settings.json"))?;
    draft.validation_commands = non_empty_or_default(
        trimmed_vec(draft.validation_commands),
        &settings.validation_commands,
    );
    draft.evidence_requirements = non_empty_or_default(
        trimmed_vec(draft.evidence_requirements),
        &[
            "issue-contract".to_string(),
            "validation-output".to_string(),
            "evidence-summary".to_string(),
            "rollback-plan".to_string(),
        ],
    );
    draft.rollback_plan = non_empty_or_default(
        trimmed_vec(draft.rollback_plan),
        &[
            "如果 Issue 创建错误，回退新增 issue 文件以及 project/team/index 引用。".to_string(),
            "如果范围需要扩大，停止并由用户重新确认 IssueContract。".to_string(),
        ],
    );
    Ok(draft)
}

fn normalize_optional_seed_id(value: Option<&str>) -> Result<Option<String>> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };
    let id = slugify(value);
    validate_project_link_id(&id)?;
    Ok(Some(id))
}

fn parse_project_create_status(status: &str) -> Result<ProjectStatus> {
    match normalized_status(status).as_str() {
        "" | "draft" => Ok(ProjectStatus::Draft),
        "active" => Ok(ProjectStatus::Active),
        "paused" => Ok(ProjectStatus::Paused),
        "completed" => Ok(ProjectStatus::Completed),
        "canceled" | "cancelled" => Ok(ProjectStatus::Canceled),
        other => bail!(
            "project status `{other}` is invalid; use draft, active, paused, completed, or canceled"
        ),
    }
}

fn trim_optional(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn trimmed_vec(values: Vec<String>) -> Vec<String> {
    values
        .into_iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect()
}

fn draft_seed_id(
    project_dir: &Path,
    seed_dir: &str,
    provided: Option<&str>,
    title: &str,
    prefix: &str,
) -> Result<String> {
    if let Some(provided) = provided {
        let id = slugify(provided);
        validate_project_link_id(&id)?;
        return Ok(id);
    }
    let slug = slugify(title);
    if !slug.is_empty() {
        validate_project_link_id(&slug)?;
        return Ok(slug);
    }

    let dir = project_dir.join(seed_dir);
    let mut number = json_files(&dir)?.len() + 1;
    loop {
        let candidate = format!("{prefix}-{number:04}");
        validate_project_link_id(&candidate)?;
        if !dir.join(format!("{candidate}.json")).exists() {
            return Ok(candidate);
        }
        number += 1;
    }
}

fn draft_milestone_id(
    project_seed: &serde_json::Value,
    provided: Option<&str>,
    draft: &MilestoneDraft,
) -> Result<String> {
    if let Some(provided) = provided {
        let id = slugify(provided);
        validate_project_link_id(&id)?;
        return Ok(id);
    }
    let slug = slugify(&draft.title);
    if !slug.is_empty() {
        validate_project_link_id(&slug)?;
        return Ok(slug);
    }
    let count = project_seed
        .get("milestones")
        .and_then(serde_json::Value::as_array)
        .map(|items| items.len() + 1)
        .unwrap_or(1);
    Ok(format!("milestone-{count:04}"))
}

fn default_team_id(project_dir: &Path) -> Result<String> {
    let workspace_path = project_dir.join("workspace.json");
    if !workspace_path.exists() {
        return Ok("core".to_string());
    }
    let workspace: serde_json::Value = read_json(&workspace_path)?;
    let team_id = json_string_field(&workspace, "defaultTeamId")
        .or_else(|| {
            json_string_array_field(&workspace, "teamIds")
                .into_iter()
                .next()
        })
        .unwrap_or_else(|| "core".to_string());
    validate_project_link_id(&team_id)?;
    Ok(team_id)
}

fn default_project_id(project_dir: &Path) -> Result<String> {
    let workspace_path = project_dir.join("workspace.json");
    if !workspace_path.exists() {
        bail!("project create requires .agentflow/workspace.json; run `agentflow project-seed --write --yes` first");
    }
    let workspace: serde_json::Value = read_json(&workspace_path)?;
    let project_id = json_string_field(&workspace, "activeProjectId")
        .or_else(|| {
            json_string_array_field(&workspace, "projectIds")
                .into_iter()
                .next()
        })
        .ok_or_else(|| {
            anyhow!("workspace has no projectIds; run `agentflow project create` first")
        })?;
    validate_project_link_id(&project_id)?;
    Ok(project_id)
}

fn default_milestone_id(project_seed: &serde_json::Value) -> Result<String> {
    let milestone_id = json_string_field(project_seed, "activeMilestoneId")
        .filter(|id| !id.is_empty())
        .or_else(|| {
            project_seed
                .get("milestones")
                .and_then(serde_json::Value::as_array)
                .and_then(|items| items.first())
                .and_then(|milestone| json_string_field(milestone, "id"))
        })
        .ok_or_else(|| {
            anyhow!("project has no milestone; run `agentflow milestone create` first")
        })?;
    validate_project_link_id(&milestone_id)?;
    Ok(milestone_id)
}

fn default_team_id_for_project(
    project_dir: &Path,
    project_seed: &serde_json::Value,
) -> Result<String> {
    let team_id = json_string_array_field(project_seed, "teamIds")
        .into_iter()
        .next()
        .unwrap_or(default_team_id(project_dir)?);
    validate_project_link_id(&team_id)?;
    Ok(team_id)
}

fn team_creation_seed(team_id: &str, name: &str) -> serde_json::Value {
    serde_json::json!({
        "version": VERSION,
        "id": team_id,
        "name": name,
        "workflow": ["backlog", "todo", "in_progress", "in_review", "done", "canceled"],
        "defaultValidationCommands": [],
        "wipLimit": 1,
        "issueIds": [],
        "projectIds": [],
        "source": {
            "kind": "team-project-milestone-issue-writers",
            "generatedFrom": "agentflow team create"
        }
    })
}

fn project_creation_seed(
    project_id: &str,
    team_id: &str,
    draft: &ProjectDraft,
) -> serde_json::Value {
    serde_json::json!({
        "version": VERSION,
        "id": project_id,
        "name": draft.title.clone(),
        "status": draft.status.clone(),
        "goal": draft.goal.clone().unwrap_or_else(|| draft.title.clone()),
        "teamIds": [team_id],
        "activeMilestoneId": "",
        "milestones": [],
        "issueIds": [],
        "nextIssueIntent": null,
        "source": {
            "kind": "team-project-milestone-issue-writers",
            "generatedFrom": "agentflow project create"
        }
    })
}

fn workspace_with_team(project_dir: &Path, team_id: &str) -> Result<serde_json::Value> {
    let workspace_path = project_dir.join("workspace.json");
    if !workspace_path.exists() {
        bail!("team create requires .agentflow/workspace.json; run `agentflow project-seed --write --yes` first");
    }
    let mut workspace: serde_json::Value = read_json(&workspace_path)?;
    append_unique_json_string_array(&mut workspace, "teamIds", team_id)?;
    Ok(workspace)
}

fn workspace_with_project(project_dir: &Path, project_id: &str) -> Result<serde_json::Value> {
    let workspace_path = project_dir.join("workspace.json");
    if !workspace_path.exists() {
        bail!("project create requires .agentflow/workspace.json; run `agentflow project-seed --write --yes` first");
    }
    let mut workspace: serde_json::Value = read_json(&workspace_path)?;
    append_unique_json_string_array(&mut workspace, "projectIds", project_id)?;
    Ok(workspace)
}

fn team_with_project(team_path: &Path, project_id: &str) -> Result<serde_json::Value> {
    let mut team: serde_json::Value = read_json(team_path)?;
    append_unique_json_string_array(&mut team, "projectIds", project_id)?;
    Ok(team)
}

fn seed_milestone_exists(project_seed: &serde_json::Value, milestone_id: &str) -> bool {
    project_seed
        .get("milestones")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|milestones| {
            milestones.iter().any(|milestone| {
                milestone
                    .get("id")
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|id| id == milestone_id)
            })
        })
}

fn append_milestone_seed(
    project_seed: &mut serde_json::Value,
    milestone_id: &str,
    draft: &MilestoneDraft,
) -> Result<()> {
    let object = project_seed
        .as_object_mut()
        .ok_or_else(|| anyhow!("project seed must be a JSON object"))?;
    object
        .entry("milestones".to_string())
        .or_insert_with(|| serde_json::Value::Array(Vec::new()));
    let milestones = object
        .get_mut("milestones")
        .and_then(serde_json::Value::as_array_mut)
        .ok_or_else(|| anyhow!("project seed milestones must be an array"))?;
    let sort_order = milestones.len() + 1;
    milestones.push(serde_json::json!({
        "id": milestone_id,
        "name": draft.title.clone(),
        "title": draft.title.clone(),
        "description": draft.description.clone(),
        "sortOrder": sort_order,
        "target": draft.target.clone(),
        "issueIds": []
    }));
    let active_missing = object
        .get("activeMilestoneId")
        .and_then(serde_json::Value::as_str)
        .is_none_or(|id| id.is_empty());
    if active_missing {
        object.insert(
            "activeMilestoneId".to_string(),
            serde_json::Value::String(milestone_id.to_string()),
        );
    }
    Ok(())
}

fn issue_creation_contract(
    project_dir: &Path,
    draft: &IssueDraft,
    issue_id: &str,
) -> Result<IssueContract> {
    let settings: Settings = read_json(&project_dir.join("settings.json"))?;
    let team_id = draft
        .team_id
        .clone()
        .ok_or_else(|| anyhow!("issue draft missing team id"))?;
    let project_id = draft
        .project_id
        .clone()
        .ok_or_else(|| anyhow!("issue draft missing project id"))?;
    let milestone_id = draft
        .milestone_id
        .clone()
        .ok_or_else(|| anyhow!("issue draft missing milestone id"))?;
    Ok(IssueContract {
        id: issue_id.to_string(),
        title: draft.title.clone(),
        status: IssueStatus::Todo.as_str().to_string(),
        intent: draft.title.clone(),
        risk_level: draft.risk_level.clone(),
        scope: draft.scope.clone(),
        non_goals: draft.non_goals.clone(),
        context: IssueContext {
            repo: ".".to_string(),
            files: vec![
                ".agentflow/workspace.json".to_string(),
                format!(".agentflow/teams/{team_id}.json"),
                format!(".agentflow/projects/{project_id}.json"),
            ],
        },
        execution_plan: vec![
            "读取 Team / Project / Milestone / Issue 本地事实源。".to_string(),
            "确认 IssueContract 的 scope、non-goals、validation commands 和 rollback plan。"
                .to_string(),
            "保持本阶段只创建任务，不自动执行 run / verify / review。".to_string(),
        ],
        validation: ValidationSpec {
            commands: draft.validation_commands.clone(),
        },
        evidence_requirements: draft.evidence_requirements.clone(),
        rollback_plan: draft.rollback_plan.clone(),
        human_gate: HumanGate {
            before_file_edits: false,
            before_external_network: true,
        },
        project_link: Some(IssueProjectLink {
            team_id,
            project_id,
            milestone_id,
            link_source: "team-project-milestone-issue-writers-v0".to_string(),
        }),
        aep: aep_issue_protocol(&settings),
    })
}

fn empty_creation_v1_contract(kind: &str) -> CreationV1ContractPreview {
    CreationV1ContractPreview {
        model: "project-milestone-issue-view-model-v1".to_string(),
        relation: format!("Workspace / Team -> Project -> Milestone -> Issue -> View ({kind})"),
        team: None,
        project_charter: None,
        milestone_gate: None,
        issue_contract: None,
        view_filter: None,
    }
}

fn creation_v1_team_contract(team_id: &str, name: &str) -> CreationV1ContractPreview {
    let mut contract = empty_creation_v1_contract("team");
    contract.team = Some(TeamCreationV1Preview {
        team_id: team_id.to_string(),
        name: name.to_string(),
        project_ids: Vec::new(),
        issue_ids: Vec::new(),
        queue_rule: vec![
            "Team 是 Project 和 Issue 的父级归属。".to_string(),
            "Team 不执行任务，不维护产品状态。".to_string(),
            "同一 Project 默认只允许一个 code-changing Issue 进入 Todo / In Progress。".to_string(),
        ],
        boundary: vec![
            "不执行 run / verify / review。".to_string(),
            "不创建远程 PR / GitHub issue / Linear issue。".to_string(),
            "不保存 View 结果，不改变业务状态。".to_string(),
        ],
    });
    contract
}

fn creation_v1_project_contract(
    project_id: &str,
    team_id: &str,
    draft: &ProjectDraft,
) -> CreationV1ContractPreview {
    let goal = draft.goal.clone().unwrap_or_else(|| draft.title.clone());
    let mut contract = empty_creation_v1_contract("project");
    contract.project_charter = Some(ProjectCharterV1Preview {
        project_id: project_id.to_string(),
        team_id: team_id.to_string(),
        status: draft.status.clone(),
        goal: goal.clone(),
        scope: vec![
            format!(
                "围绕 `{}` 建立 Project -> Milestone -> Issue 的本地事实源。",
                draft.title
            ),
            "允许后续在 Project 下追加 Milestone 和 Issue。".to_string(),
        ],
        non_goals: vec![
            "不直接执行 Issue。".to_string(),
            "不替代 GitHub / Linear / SaaS 项目管理平台。".to_string(),
            "不自动调用模型拆解或修改代码。".to_string(),
        ],
        success_criteria: vec![
            "Project charter 可被 CLI 和 Desktop 读取。".to_string(),
            "Project 下 Milestone / Issue 关系清晰可追溯。".to_string(),
            "创建动作必须 preview-first，只有 --write --yes 才落盘。".to_string(),
        ],
        milestones: Vec::new(),
        issue_order: Vec::new(),
        validation_gate: vec![
            "cargo run -p agentflow-cli -- projects".to_string(),
            "cargo run -p agentflow-cli -- goal next".to_string(),
        ],
        evidence_requirements: vec![
            "creation-preview".to_string(),
            "local-facts-diff".to_string(),
            "validation-output".to_string(),
        ],
        queue_rule: vec![
            "Project 不执行，Issue 执行。".to_string(),
            "Queue Preflight 决定 Issue 是否可以从 Backlog / Todo 进入执行。".to_string(),
            "View 只展示，不承载业务状态。".to_string(),
        ],
        closure_gate: vec![
            "所有 Issue 完成。".to_string(),
            "Evidence 完整。".to_string(),
            "Project closure 后续仍需 audit / docs refresh。".to_string(),
        ],
        boundary: vec![
            "不自动执行 run / verify / review。".to_string(),
            "不创建远程对象。".to_string(),
            "不隐式覆盖 activeProjectId。".to_string(),
        ],
    });
    contract
}

fn creation_v1_milestone_contract(
    project_id: &str,
    milestone_id: &str,
    draft: &MilestoneDraft,
) -> CreationV1ContractPreview {
    let goal = draft
        .target
        .clone()
        .or_else(|| draft.description.clone())
        .unwrap_or_else(|| draft.title.clone());
    let mut contract = empty_creation_v1_contract("milestone");
    contract.milestone_gate = Some(MilestoneGateV1Preview {
        project_id: project_id.to_string(),
        milestone_id: milestone_id.to_string(),
        goal,
        entry_criteria: vec![
            "Project 已存在。".to_string(),
            "Milestone id 在 Project 内唯一。".to_string(),
            "Milestone 只作为阶段分组，不写独立产品状态。".to_string(),
        ],
        scope: vec![
            format!("在 Project `{project_id}` 下新增阶段 `{}`。", draft.title),
            "后续 Issue 必须显式挂到该 Milestone。".to_string(),
        ],
        non_goals: vec![
            "Milestone 不直接执行代码。".to_string(),
            "Milestone 不维护独立状态机。".to_string(),
            "不自动推进下一个 Milestone。".to_string(),
        ],
        issues: Vec::new(),
        exit_criteria: vec![
            "Milestone 下所有 Issue 达到 done。".to_string(),
            "验证输出和 evidence 可追溯。".to_string(),
            "Milestone summary / review 后续由专门流程生成。".to_string(),
        ],
        validation: vec!["cargo run -p agentflow-cli -- projects".to_string()],
        evidence_required: vec![
            "milestone-preview".to_string(),
            "project-milestone-link".to_string(),
            "validation-output".to_string(),
        ],
        next_milestone_gate: vec![
            "当前 Milestone 完成度从 Issues 派生。".to_string(),
            "进入下一阶段前必须通过后续 Queue / Review gate。".to_string(),
        ],
    });
    contract
}

fn creation_v1_issue_contract(contract: &IssueContract) -> CreationV1ContractPreview {
    let (team_id, project_id, milestone_id) = contract
        .project_link
        .as_ref()
        .map(|link| {
            (
                link.team_id.clone(),
                link.project_id.clone(),
                link.milestone_id.clone(),
            )
        })
        .unwrap_or_else(|| {
            (
                "unknown-team".to_string(),
                "unknown-project".to_string(),
                "unknown-milestone".to_string(),
            )
        });
    let mut contract_preview = empty_creation_v1_contract("issue");
    contract_preview.issue_contract = Some(IssueContractV1Preview {
        issue_id: contract.id.clone(),
        project_id,
        milestone_id,
        team_id,
        goal: contract.intent.clone(),
        scope: contract.scope.clone(),
        non_goals: contract.non_goals.clone(),
        dependencies: Vec::new(),
        codex_instructions: contract.execution_plan.clone(),
        acceptance_criteria: vec![
            "Goal 完成且不越过 Scope / Non-goals。".to_string(),
            "Validation Commands 可执行并留下输出。".to_string(),
            "Evidence Required 项完整。".to_string(),
        ],
        validation_commands: contract.validation.commands.clone(),
        evidence_required: contract.evidence_requirements.clone(),
        allowed_files: contract.context.files.clone(),
        forbidden_files: vec![
            "未声明的源码目录。".to_string(),
            ".env*".to_string(),
            ".agentflow/search 或 .agentflow/queries writer".to_string(),
        ],
        boundary: vec![
            "Issue 是唯一执行原子。".to_string(),
            "不跨 Milestone 扩范围。".to_string(),
            "不自动执行 run / verify / review。".to_string(),
            "不创建远程 PR / GitHub issue / Linear issue。".to_string(),
        ],
        initial_state: contract.status.clone(),
    });
    contract_preview
}

fn creation_confirmation_gates() -> Vec<String> {
    [
        "preview-default",
        "explicit-write-flag",
        "explicit-yes-confirmation",
        "refuse-existing-team-or-project",
        "canonical-project-status",
        "canonical-issue-status",
        "no-milestone-status",
        "local-facts-only",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

fn creation_sources(kind: &str) -> Vec<String> {
    vec![
        format!("Team / Project / Milestone / Issue Writers v0 {kind} draft"),
        ".agentflow/workspace.json".to_string(),
        ".agentflow/teams/{team-id}.json".to_string(),
        ".agentflow/projects/{project-id}.json".to_string(),
        ".agentflow/issues/ISSUE-XXXX.json".to_string(),
        ".agentflow/index.json".to_string(),
    ]
}

fn creation_boundary(write: bool) -> WorkbenchBoundary {
    WorkbenchBoundary {
        read_only: !write,
        disallowed_actions: vec![
            "run".to_string(),
            "verify".to_string(),
            "review".to_string(),
            "model-call".to_string(),
            "remote-pr-or-issue".to_string(),
            "saas-account-payment-cloud-sync".to_string(),
            "desktop-write-ui".to_string(),
            "overwrite-existing-team-or-project".to_string(),
            "bypass-preview-confirmation".to_string(),
        ],
    }
}

fn product_feature_creation_summary_markdown(snapshot: &ProductFeatureCreationSnapshot) -> String {
    format!(
        "# Product Feature Creation Summary\n\n- Mode: `{}`\n- Project: `{}`\n- Team: `{}`\n- Feature goal: {}\n- Active milestone: `{}`\n- Recommended command: `{}`\n\n## Milestones\n\n{}\n\n## Issue Contracts\n\n{}\n\n## Writes\n\n{}\n\n## Boundary\n\n- 不调用模型\n- 不创建远程 PR / GitHub issue / Linear issue\n- 不从 Desktop 执行创建\n- 不标记 Project done\n- 后续执行必须继续通过 goal next / eligibility / lease / run\n",
        snapshot.mode,
        snapshot.project.id,
        snapshot.project.team_id,
        snapshot.draft.feature_goal,
        snapshot.project.active_milestone_id,
        snapshot.recommended_command,
        snapshot
            .milestones
            .iter()
            .map(|milestone| format!(
                "- `{}` -> `{}`",
                milestone.id, milestone.issue_id
            ))
            .collect::<Vec<_>>()
            .join("\n"),
        snapshot
            .issues
            .iter()
            .map(|issue| format!(
                "- `{}` [{}] -> `{}` risk={} milestone=`{}`",
                issue.id, issue.status, issue.title, issue.risk_level, issue.milestone_id
            ))
            .collect::<Vec<_>>()
            .join("\n"),
        if snapshot.written_paths.is_empty() {
            "- preview only; no facts written".to_string()
        } else {
            snapshot
                .written_paths
                .iter()
                .map(|path| format!("- `{path}`"))
                .collect::<Vec<_>>()
                .join("\n")
        }
    )
}

fn feature_relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .ok()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| path.display().to_string())
}

fn product_feature_execution_snapshot(repo: &Path) -> Result<ProductFeatureExecutionSnapshot> {
    let project_dir = required_project_dir(repo)?;
    let project_snapshot = read_local_project_model_snapshot(repo)?;
    let workspace = project_snapshot
        .workspace
        .as_ref()
        .ok_or_else(|| anyhow!("feature execution requires .agentflow/workspace.json"))?;
    let project_id = workspace.active_project_id.clone();
    validate_project_link_id(&project_id)?;
    let project = project_snapshot
        .projects
        .iter()
        .find(|project| project.id == project_id)
        .ok_or_else(|| anyhow!("active project `{project_id}` is missing"))?;
    let project_path = project_dir
        .join("projects")
        .join(format!("{}.json", project.id));
    let project_seed: serde_json::Value = read_json(&project_path)?;
    if project_seed.get("productFeature").is_none() {
        bail!(
            "active project `{}` is not a product feature project; run `agentflow feature create` first",
            project.id,
        );
    }

    let issue_contracts = read_issue_contracts(&project_dir)?;
    let issue_by_id = issue_contracts
        .iter()
        .map(|issue| (issue.id.clone(), issue))
        .collect::<HashMap<_, _>>();
    let issue_ref_by_id = project_snapshot
        .issue_refs
        .iter()
        .map(|issue| (issue.id.clone(), issue))
        .collect::<HashMap<_, _>>();
    let eligibility = workflow_eligibility_snapshot(repo, None)?;
    let eligibility_by_id = eligibility
        .candidates
        .iter()
        .map(|candidate| (candidate.issue_id.clone(), candidate))
        .collect::<HashMap<_, _>>();

    let milestones = project
        .milestones
        .iter()
        .map(|milestone| ProductFeatureExecutionMilestone {
            id: milestone.id.clone(),
            title: milestone.name.clone(),
            status: milestone.status.clone(),
            progress: milestone.progress.clone(),
            issue_ids: milestone.issue_ids.clone(),
            completed_issue_ids: milestone.completed_issue_ids.clone(),
            next_issue_intent: milestone.next_issue_intent.clone(),
        })
        .collect::<Vec<_>>();

    let mut issues = Vec::new();
    for issue_id in &project.issue_ids {
        let Some(contract) = issue_by_id.get(issue_id).copied() else {
            continue;
        };
        let reference = issue_ref_by_id.get(issue_id).copied();
        let candidate = eligibility_by_id.get(issue_id).copied();
        issues.push(product_feature_execution_issue(
            &project_dir,
            contract,
            reference,
            candidate,
        )?);
    }

    let active_milestone = project
        .milestones
        .iter()
        .find(|milestone| milestone.id == project.active_milestone_id);
    let current_issue = active_milestone.and_then(|milestone| {
        let open_ids = milestone
            .issue_ids
            .iter()
            .filter(|issue_id| {
                issue_by_id
                    .get(*issue_id)
                    .is_some_and(|issue| issue_status_open(&issue.status))
            })
            .cloned()
            .collect::<Vec<_>>();
        if open_ids.len() == 1 {
            issues.iter().find(|issue| issue.id == open_ids[0]).cloned()
        } else {
            None
        }
    });

    let feature_ready = canonical_project_status(&project.status) == ProjectStatus::Active
        && active_milestone.is_some();
    let (next_action, recommended_command, rationale) =
        product_feature_execution_decision(feature_ready, active_milestone, &current_issue);

    Ok(ProductFeatureExecutionSnapshot {
        version: VERSION.to_string(),
        initialized: project_snapshot.initialized,
        project_root: project_snapshot.project_root,
        feature_ready,
        project_id: project.id.clone(),
        project_title: project.name.clone(),
        project_status: project.status.clone(),
        project_canonical_status: project.canonical_status.clone(),
        project_goal: project.goal.clone(),
        active_milestone_id: project.active_milestone_id.clone(),
        milestones,
        current_issue,
        issues,
        next_action,
        recommended_command,
        rationale,
        sources: vec![
            ".agentflow/workspace.json".to_string(),
            format!(".agentflow/projects/{}.json", project.id),
            ".agentflow/issues/*.json".to_string(),
            ".agentflow/runs/*/run.json".to_string(),
            ".agentflow/state/eligibility.json".to_string(),
        ],
        boundary: WorkbenchBoundary {
            read_only: true,
            disallowed_actions: vec![
                "execute-run".to_string(),
                "execute-verify".to_string(),
                "execute-review".to_string(),
                "call-model".to_string(),
                "create-remote-issue".to_string(),
                "create-remote-pr".to_string(),
                "mark-project-done".to_string(),
                "create-audits-dir".to_string(),
            ],
        },
    })
}

fn product_feature_execution_issue(
    project_dir: &Path,
    issue: &IssueContract,
    reference: Option<&LocalProjectIssueRef>,
    candidate: Option<&WorkflowEligibilityCandidate>,
) -> Result<ProductFeatureExecutionIssue> {
    let latest_run = latest_run_for_issue(project_dir, &issue.id)?;
    let indexed = IndexedIssue {
        id: issue.id.clone(),
        title: issue.title.clone(),
        status: issue.status.clone(),
        intent: issue.intent.clone(),
        json_path: format!(".agentflow/issues/{}.json", issue.id),
    };
    let (next_action, recommended_command) = if issue_state_done(&issue.status) {
        ("completed".to_string(), "completed".to_string())
    } else if let Some(candidate) = candidate {
        if !candidate.eligible && candidate.active_lease_id.is_none() {
            (
                "wait-human".to_string(),
                "agentflow eligibility".to_string(),
            )
        } else {
            issue_next_step(project_dir, &indexed)?
        }
    } else {
        issue_next_step(project_dir, &indexed)?
    };

    Ok(ProductFeatureExecutionIssue {
        id: issue.id.clone(),
        title: issue.title.clone(),
        status: issue.status.clone(),
        canonical_status: canonical_issue_status_string(&issue.status),
        milestone_id: issue
            .project_link
            .as_ref()
            .map(|link| link.milestone_id.clone()),
        ready: candidate
            .map(|candidate| candidate.ready)
            .unwrap_or_else(|| issue_contract_ready(issue)),
        eligible: candidate
            .map(|candidate| candidate.eligible)
            .unwrap_or(false),
        leased: candidate.map(|candidate| candidate.leased).unwrap_or(false),
        failure_reasons: candidate
            .map(|candidate| candidate.failure_reasons.clone())
            .unwrap_or_default(),
        next_action,
        recommended_command,
        dry_run_recorded: latest_run.as_ref().is_some_and(|run| run.mode == "dry-run"),
        latest_run_plan: latest_run
            .as_ref()
            .map(|run| run.run_plan.planned_steps.clone())
            .unwrap_or_default(),
        expected_files: latest_run
            .as_ref()
            .map(|run| run.run_plan.expected_files.clone())
            .unwrap_or_else(|| issue.context.files.clone()),
        blocked_files: latest_run
            .as_ref()
            .map(|run| run.run_plan.blocked_files.clone())
            .unwrap_or_else(|| blocked_files_from_issue(issue)),
        validation_commands: latest_run
            .as_ref()
            .map(|run| run.run_plan.validation_commands.clone())
            .unwrap_or_else(|| issue.validation.commands.clone()),
        evidence_requirements: latest_run
            .as_ref()
            .map(|run| run.run_plan.evidence_requirements.clone())
            .unwrap_or_else(|| issue.evidence_requirements.clone()),
        latest_run_id: reference.and_then(|issue| issue.latest_run_id.clone()),
        latest_run_status: reference.and_then(|issue| issue.latest_run_status.clone()),
        validation_status: reference
            .map(|issue| issue.validation_status.clone())
            .unwrap_or_else(|| "not-run".to_string()),
        execution_state: reference
            .map(|issue| issue.execution_state.clone())
            .unwrap_or_else(|| "not-started".to_string()),
        evidence_path: reference.and_then(|issue| issue.evidence_path.clone()),
        review_path: reference.and_then(|issue| issue.review_path.clone()),
        project_update_path: reference.and_then(|issue| issue.project_update_path.clone()),
    })
}

fn product_feature_execution_decision(
    feature_ready: bool,
    active_milestone: Option<&LocalMilestone>,
    current_issue: &Option<ProductFeatureExecutionIssue>,
) -> (String, String, Vec<String>) {
    if !feature_ready {
        return (
            "wait-human".to_string(),
            "agentflow projects".to_string(),
            vec!["Active product feature project is not ready.".to_string()],
        );
    }

    let Some(milestone) = active_milestone else {
        return (
            "wait-human".to_string(),
            "agentflow projects".to_string(),
            vec!["Active product feature milestone is missing.".to_string()],
        );
    };

    let Some(issue) = current_issue else {
        return (
            "wait-human".to_string(),
            "agentflow feature status".to_string(),
            vec![format!(
                "Active milestone `{}` does not have exactly one open issue.",
                milestone.id
            )],
        );
    };

    if !issue.failure_reasons.is_empty() {
        return (
            "wait-human".to_string(),
            "agentflow eligibility".to_string(),
            vec![
                format!("Current issue `{}` is not eligible.", issue.id),
                issue.failure_reasons.join(", "),
            ],
        );
    }

    (
        issue.next_action.clone(),
        issue.recommended_command.clone(),
        vec![
            format!("Active product feature milestone is `{}`.", milestone.id),
            format!("Current issue is `{}`.", issue.id),
            "Product feature execution reuses GoalLoop, Eligibility, Lease, Run, Verify, Review, Evidence, and Milestone Summary.".to_string(),
        ],
    )
}

fn issue_next_step(project_dir: &Path, issue: &IndexedIssue) -> Result<(String, String)> {
    let Some(run) = latest_run_for_issue(project_dir, &issue.id)? else {
        return Ok((
            "run".to_string(),
            format!("agentflow run {} --dry-run", issue.id),
        ));
    };

    if run.validation_commands.is_empty() {
        return Ok((
            "verify".to_string(),
            format!("agentflow verify {}", issue.id),
        ));
    }

    if run
        .validation_commands
        .iter()
        .any(|record| record.exit_code != 0)
    {
        return Ok((
            "wait-human".to_string(),
            format!("inspect validation failure for {}", issue.id),
        ));
    }

    if !issue_state_done(&issue.status)
        || run.outputs.evidence.is_none()
        || run.outputs.review.is_none()
    {
        return Ok((
            "review".to_string(),
            format!("agentflow review {}", issue.id),
        ));
    }

    Ok(("update".to_string(), "agentflow update summary".to_string()))
}

fn current_candidate_intent(repo: &Path, project_dir: &Path) -> Result<String> {
    let root_roadmap = repo.join("ROADMAP.md");
    if root_roadmap.exists() {
        let content = fs::read_to_string(&root_roadmap)
            .with_context(|| format!("read roadmap {}", root_roadmap.display()))?;
        if let Some(candidate) = markdown_backtick_value(&content, "候选施工包：") {
            return Ok(candidate);
        }
    }

    let project_roadmap = project_dir.join("roadmap.md");
    if project_roadmap.exists() {
        let content = fs::read_to_string(&project_roadmap)
            .with_context(|| format!("read roadmap {}", project_roadmap.display()))?;
        if let Some(line) = content
            .lines()
            .rev()
            .map(str::trim)
            .find(|line| line.chars().next().is_some_and(|ch| ch.is_ascii_digit()))
        {
            return Ok(line
                .split_once('.')
                .map(|(_, value)| value.trim().to_string())
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| "继续下一项 roadmap 能力".to_string()));
        }
    }

    Ok("继续下一项 roadmap 能力".to_string())
}

fn discover_project_root(start: &Path) -> Result<Option<PathBuf>> {
    let start = start
        .canonicalize()
        .with_context(|| format!("canonicalize {}", start.display()))?;
    let mut cursor = if start.is_file() {
        start.parent().map(Path::to_path_buf)
    } else {
        Some(start)
    };
    while let Some(path) = cursor {
        if path.join(AGENTFLOW_DIR).exists() {
            return Ok(Some(path));
        }
        cursor = path.parent().map(Path::to_path_buf);
    }
    Ok(None)
}

fn empty_desktop_snapshot(start: &Path) -> DesktopWorkbenchSnapshot {
    DesktopWorkbenchSnapshot {
        version: VERSION.to_string(),
        initialized: false,
        project_root: start.display().to_string(),
        project_summary_markdown: None,
        goal_loop_summary_markdown: None,
        goal_loop: None,
        issues: Vec::new(),
        runs: Vec::new(),
        saved_views: Vec::new(),
        evidence: Vec::new(),
        reviews: Vec::new(),
        project_updates: Vec::new(),
        counts: WorkbenchCounts::default(),
        boundary: workbench_boundary(),
    }
}

fn empty_local_metrics_snapshot(snapshot: &DesktopWorkbenchSnapshot) -> LocalMetricsSnapshot {
    LocalMetricsSnapshot {
        version: VERSION.to_string(),
        initialized: false,
        project_root: snapshot.project_root.clone(),
        issues: LocalIssueMetrics::default(),
        runs: LocalRunMetrics::default(),
        artifacts: LocalArtifactMetrics::default(),
        goal_ready: false,
        active_issue_id: None,
        next_action: "wait-human".to_string(),
        recommended_command: "agentflow init --from-goal GOAL.md".to_string(),
        latest_run: None,
        latest_evidence: None,
        latest_review: None,
        sources: Vec::new(),
        boundary: workbench_boundary(),
    }
}

fn empty_local_search_snapshot(start: &Path, query: &str) -> LocalSearchSnapshot {
    LocalSearchSnapshot {
        version: VERSION.to_string(),
        initialized: false,
        project_root: start.display().to_string(),
        query: LocalSearchQuery {
            query: query.to_string(),
        },
        results: Vec::new(),
        searched_paths: Vec::new(),
        excluded_paths: local_search_excluded_paths(),
        boundary: workbench_boundary(),
    }
}

fn empty_local_project_model_snapshot(start: &Path) -> LocalProjectModelSnapshot {
    LocalProjectModelSnapshot {
        version: VERSION.to_string(),
        initialized: false,
        project_root: start.display().to_string(),
        workspace: None,
        teams: Vec::new(),
        projects: Vec::new(),
        issue_refs: Vec::new(),
        goal_loop_selection: GoalLoopSelection {
            active_project_id: None,
            source: "missing-agentflow".to_string(),
            next_action: "wait-human".to_string(),
            next_issue_intent: None,
            recommended_command: "agentflow init --from-goal GOAL.md".to_string(),
            rationale: vec!["No .agentflow directory was found.".to_string()],
        },
        sources: Vec::new(),
        boundary: workbench_boundary(),
    }
}

fn empty_project_milestone_issue_view_model_snapshot(
    snapshot: &LocalProjectModelSnapshot,
) -> ProjectMilestoneIssueViewModelSnapshot {
    ProjectMilestoneIssueViewModelSnapshot {
        version: VERSION.to_string(),
        initialized: false,
        project_root: snapshot.project_root.clone(),
        workspace: None,
        teams: Vec::new(),
        projects: Vec::new(),
        issues: Vec::new(),
        views: Vec::new(),
        invariants: project_milestone_issue_view_model_invariants(),
        sources: Vec::new(),
        boundary: workbench_boundary(),
    }
}

fn project_milestone_issue_view_model_from_snapshot(
    snapshot: &LocalProjectModelSnapshot,
    issue_contracts: &[IssueContract],
    views: &[SavedView],
) -> ProjectMilestoneIssueViewModelSnapshot {
    let issue_contracts_by_id = issue_contracts
        .iter()
        .map(|issue| (issue.id.as_str(), issue))
        .collect::<HashMap<_, _>>();
    let issue_refs_by_id = snapshot
        .issue_refs
        .iter()
        .map(|issue| (issue.id.as_str(), issue))
        .collect::<HashMap<_, _>>();
    let issue_links = v1_issue_links(&snapshot.projects);

    let workspace = snapshot.workspace.as_ref().map(|workspace| V1WorkspaceRef {
        id: workspace.id.clone(),
        name: workspace.name.clone(),
        active_project_id: workspace.active_project_id.clone(),
        team_ids: workspace.team_ids.clone(),
        project_ids: workspace.project_ids.clone(),
    });

    let team_project_ids = v1_team_project_ids(&snapshot.projects);
    let teams = snapshot
        .teams
        .iter()
        .map(|team| V1TeamRef {
            id: team.id.clone(),
            name: team.name.clone(),
            project_ids: team_project_ids.get(&team.id).cloned().unwrap_or_default(),
            issue_ids: team.issue_ids.clone(),
        })
        .collect::<Vec<_>>();

    let projects = snapshot
        .projects
        .iter()
        .map(|project| {
            let project_seed = v1_project_seed_value(&snapshot.project_root, &project.id);
            let project_issue_contracts = project
                .issue_ids
                .iter()
                .filter_map(|issue_id| issue_contracts_by_id.get(issue_id.as_str()).copied())
                .collect::<Vec<_>>();
            let milestones = project
                .milestones
                .iter()
                .map(|milestone| {
                    let milestone_issue_refs = milestone
                        .issue_ids
                        .iter()
                        .filter_map(|issue_id| issue_refs_by_id.get(issue_id.as_str()).copied())
                        .collect::<Vec<_>>();
                    let milestone_issue_contracts = milestone
                        .issue_ids
                        .iter()
                        .filter_map(|issue_id| issue_contracts_by_id.get(issue_id.as_str()).copied())
                        .collect::<Vec<_>>();
                    V1Milestone {
                        id: milestone.id.clone(),
                        project_id: project.id.clone(),
                        name: milestone.name.clone(),
                        status: v1_milestone_status(project, milestone, &milestone_issue_refs),
                        raw_status: milestone.status.clone(),
                        goal: milestone
                            .target
                            .clone()
                            .or_else(|| milestone.description.clone())
                            .unwrap_or_else(|| milestone.name.clone()),
                        entry_criteria: vec!["Project boundary is confirmed.".to_string()],
                        scope: Vec::new(),
                        non_goals: Vec::new(),
                        issue_ids: milestone.issue_ids.clone(),
                        exit_criteria: v1_milestone_exit_criteria(),
                        validation: unique_issue_values(
                            &milestone_issue_contracts,
                            |issue| issue.validation.commands.clone(),
                        ),
                        evidence_required: unique_issue_values(
                            &milestone_issue_contracts,
                            |issue| issue.evidence_requirements.clone(),
                        ),
                        next_milestone_gate:
                            "All issues Done, validation passed, evidence complete, milestone review complete."
                                .to_string(),
                        progress: milestone.progress.clone(),
                    }
                })
                .collect::<Vec<_>>();
            V1Project {
                id: project.id.clone(),
                name: project.name.clone(),
                status: v1_project_status(&project.status),
                raw_status: project.status.clone(),
                goal: project.goal.clone(),
                target_maturity: v1_project_seed_string(project_seed.as_ref(), "targetMaturity"),
                target_layers: v1_project_seed_array(project_seed.as_ref(), "targetLayers"),
                scope: v1_project_template_scope(project_seed.as_ref()),
                non_goals: v1_project_template_non_goals(project_seed.as_ref()),
                success_criteria: v1_project_template_success_criteria(project_seed.as_ref()),
                milestones,
                issue_order: project.issue_ids.clone(),
                validation_gate: non_empty_vec_or_else(
                    v1_project_seed_array(project_seed.as_ref(), "validationGate"),
                    || unique_issue_values(&project_issue_contracts, |issue| {
                        issue.validation.commands.clone()
                    }),
                ),
                evidence_required: non_empty_vec_or_else(
                    v1_project_seed_array(project_seed.as_ref(), "evidenceRequired"),
                    || {
                        unique_issue_values(&project_issue_contracts, |issue| {
                            issue.evidence_requirements.clone()
                        })
                    },
                ),
                queue_rule: non_empty_vec_or_else(
                    v1_project_seed_array(project_seed.as_ref(), "queueRule"),
                    || {
                        vec![
                            "WIP=1".to_string(),
                            "同一项目同一时间只允许一个可执行任务。".to_string(),
                            "任务执行前必须通过队列预检。".to_string(),
                        ]
                    },
                ),
                closure_gate: v1_project_seed_array(project_seed.as_ref(), "closureGate"),
            }
        })
        .collect::<Vec<_>>();

    let issues = issue_contracts
        .iter()
        .map(|issue| {
            let link = issue_links.get(&issue.id).cloned().or_else(|| {
                issue
                    .project_link
                    .as_ref()
                    .map(|link| (link.project_id.clone(), Some(link.milestone_id.clone())))
            });
            V1Issue {
                id: issue.id.clone(),
                project_id: link.as_ref().map(|(project_id, _)| project_id.clone()),
                milestone_id: link.and_then(|(_, milestone_id)| milestone_id),
                title: issue.title.clone(),
                status: v1_issue_status(&issue.status),
                raw_status: issue.status.clone(),
                goal: issue.intent.clone(),
                scope: issue.scope.clone(),
                non_goals: issue.non_goals.clone(),
                dependencies: Vec::new(),
                codex_instructions: issue.execution_plan.clone(),
                acceptance_criteria: v1_default_acceptance_criteria(),
                validation_commands: issue.validation.commands.clone(),
                evidence_required: issue.evidence_requirements.clone(),
                allowed_files: issue.context.files.clone(),
                forbidden_files: Vec::new(),
                boundary: issue.non_goals.clone(),
                risk_level: if issue.risk_level.trim().is_empty() {
                    "medium".to_string()
                } else {
                    issue.risk_level.clone()
                },
            }
        })
        .collect::<Vec<_>>();

    let views = views
        .iter()
        .map(v1_view_from_saved_view)
        .collect::<Vec<_>>();

    ProjectMilestoneIssueViewModelSnapshot {
        version: VERSION.to_string(),
        initialized: true,
        project_root: snapshot.project_root.clone(),
        workspace,
        teams,
        projects,
        issues,
        views,
        invariants: project_milestone_issue_view_model_invariants(),
        sources: project_milestone_issue_view_model_sources(),
        boundary: workbench_boundary(),
    }
}

fn v1_project_seed_value(project_root: &str, project_id: &str) -> Option<serde_json::Value> {
    let path = Path::new(project_root)
        .join(AGENTFLOW_DIR)
        .join("projects")
        .join(format!("{project_id}.json"));
    read_json(&path).ok()
}

fn v1_project_seed_string(seed: Option<&serde_json::Value>, field: &str) -> Option<String> {
    seed.and_then(|seed| json_string_field(seed, field))
}

fn v1_project_seed_array(seed: Option<&serde_json::Value>, field: &str) -> Vec<String> {
    seed.map(|seed| json_string_array_field(seed, field))
        .unwrap_or_default()
}

fn v1_project_product_feature_array(seed: Option<&serde_json::Value>, field: &str) -> Vec<String> {
    seed.and_then(|seed| seed.get("productFeature"))
        .map(|product_feature| json_string_array_field(product_feature, field))
        .unwrap_or_default()
}

fn v1_project_template_scope(seed: Option<&serde_json::Value>) -> Vec<String> {
    non_empty_vec_or_else(v1_project_seed_array(seed, "scope"), || {
        v1_project_product_feature_array(seed, "scopeBoundaries")
    })
}

fn v1_project_template_non_goals(seed: Option<&serde_json::Value>) -> Vec<String> {
    non_empty_vec_or_else(v1_project_seed_array(seed, "nonGoals"), || {
        v1_project_product_feature_array(seed, "nonGoals")
    })
}

fn v1_project_template_success_criteria(seed: Option<&serde_json::Value>) -> Vec<String> {
    non_empty_vec_or_else(v1_project_seed_array(seed, "successCriteria"), || {
        v1_project_product_feature_array(seed, "successCriteria")
    })
}

fn non_empty_vec_or_else<F>(values: Vec<String>, fallback: F) -> Vec<String>
where
    F: FnOnce() -> Vec<String>,
{
    if values.is_empty() {
        fallback()
    } else {
        values
    }
}

fn empty_local_project_seed_preview(
    snapshot: &LocalProjectModelSnapshot,
) -> LocalProjectSeedPreview {
    LocalProjectSeedPreview {
        version: VERSION.to_string(),
        initialized: false,
        project_root: snapshot.project_root.clone(),
        files: Vec::new(),
        confirmation_gates: local_project_seed_confirmation_gates(),
        writes_required: false,
        sources: local_project_seed_sources(),
        boundary: workbench_boundary(),
    }
}

fn workbench_boundary() -> WorkbenchBoundary {
    WorkbenchBoundary {
        read_only: true,
        disallowed_actions: vec![
            "create-issue".to_string(),
            "run".to_string(),
            "verify".to_string(),
            "review".to_string(),
            "model-call".to_string(),
            "write-agentflow-facts".to_string(),
            "remote-pr-or-issue".to_string(),
        ],
    }
}

fn read_issue_contracts(project_dir: &Path) -> Result<Vec<IssueContract>> {
    let mut issues = Vec::new();
    for path in json_files(&project_dir.join("issues"))? {
        issues.push(read_json(&path)?);
    }
    issues.sort_by(|left: &IssueContract, right| left.id.cmp(&right.id));
    Ok(issues)
}

fn read_agent_runs(project_dir: &Path) -> Result<Vec<AgentRun>> {
    let runs_dir = project_dir.join("runs");
    if !runs_dir.exists() {
        return Ok(Vec::new());
    }
    let mut runs = Vec::new();
    for entry in fs::read_dir(&runs_dir).with_context(|| format!("read {}", runs_dir.display()))? {
        let path = entry?.path();
        let run_json = path.join("run.json");
        if path.is_dir() && run_json.exists() {
            runs.push(read_json(&run_json)?);
        }
    }
    runs.sort_by(|left: &AgentRun, right| left.id.cmp(&right.id));
    Ok(runs)
}

fn read_text_artifacts(repo: &Path, dir: &Path) -> Result<Vec<WorkbenchTextArtifact>> {
    let mut artifacts = Vec::new();
    for path in markdown_files(dir)? {
        let content =
            fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
        let relative = relative_path(repo, &path)?;
        let title = path
            .file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or("artifact")
            .to_string();
        artifacts.push(WorkbenchTextArtifact {
            path: relative,
            title,
            content,
        });
    }
    artifacts.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(artifacts)
}

fn read_project_update_artifacts(repo: &Path, dir: &Path) -> Result<Vec<WorkbenchTextArtifact>> {
    let mut artifacts = Vec::new();
    for path in markdown_files(dir)? {
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        if !file_name.starts_with("PROJECT-UPDATE-") {
            continue;
        }
        let content =
            fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
        let relative = relative_path(repo, &path)?;
        let title = path
            .file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or("project-update")
            .to_string();
        artifacts.push(WorkbenchTextArtifact {
            path: relative,
            title,
            content,
        });
    }
    artifacts.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(artifacts)
}

fn local_search_documents(root: &Path, project_dir: &Path) -> Result<Vec<LocalSearchDocument>> {
    let mut paths = Vec::new();
    for path in [
        "goal.md",
        "goal.json",
        "project-definition.json",
        "scope-state.json",
        "context.json",
        "context.md",
        "environment.md",
        "architecture.md",
        "roadmap.md",
        "index.json",
        "settings.json",
    ] {
        push_existing_file(&mut paths, project_dir.join(path));
    }

    push_files_with_extension(&mut paths, &project_dir.join("bootstrap"), "md")?;
    push_files_with_extension(&mut paths, &project_dir.join("issues"), "json")?;
    push_files_with_extension(&mut paths, &project_dir.join("issues"), "md")?;
    push_files_with_extension(&mut paths, &project_dir.join("evidence"), "md")?;
    push_files_with_extension(&mut paths, &project_dir.join("reviews"), "md")?;
    push_files_with_extension(&mut paths, &project_dir.join("updates"), "md")?;
    push_files_with_extension(&mut paths, &project_dir.join("views"), "json")?;
    push_run_search_files(&mut paths, &project_dir.join("runs"))?;

    paths.sort();
    paths.dedup();

    let mut documents = Vec::new();
    for path in paths {
        let relative = relative_path(root, &path)?;
        if !local_search_path_allowed(&relative) || local_search_path_excluded(&relative) {
            continue;
        }
        let content = fs::read_to_string(&path)
            .with_context(|| format!("read local search document {}", path.display()))?;
        let (entity_kind, entity_id, title) = local_search_metadata(&path, &relative, &content);
        documents.push(LocalSearchDocument {
            path: relative,
            content,
            entity_kind,
            entity_id,
            title,
        });
    }
    Ok(documents)
}

fn push_existing_file(paths: &mut Vec<PathBuf>, path: PathBuf) {
    if path.is_file() {
        paths.push(path);
    }
}

fn push_files_with_extension(paths: &mut Vec<PathBuf>, dir: &Path, extension: &str) -> Result<()> {
    paths.extend(files_with_extension(dir, extension)?);
    Ok(())
}

fn push_run_search_files(paths: &mut Vec<PathBuf>, runs_dir: &Path) -> Result<()> {
    if !runs_dir.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(runs_dir).with_context(|| format!("read {}", runs_dir.display()))? {
        let path = entry?.path();
        if !path.is_dir() {
            continue;
        }
        for file_name in [
            "run.json",
            "transcript.md",
            "commands.jsonl",
            "diff-summary.md",
        ] {
            push_existing_file(paths, path.join(file_name));
        }
    }
    Ok(())
}

fn search_document(document: &LocalSearchDocument, query: &str) -> Vec<LocalSearchResult> {
    let query_lower = query.to_lowercase();
    let mut current_field = local_search_default_field(&document.path);
    let mut results = Vec::new();

    for (index, line) in document.content.lines().enumerate() {
        if let Some(heading) = markdown_heading(line) {
            current_field = heading;
        }
        if !line.to_lowercase().contains(&query_lower) {
            continue;
        }
        results.push(LocalSearchResult {
            source_type: "file".to_string(),
            entity_kind: document.entity_kind.clone(),
            entity_id: document.entity_id.clone(),
            path: document.path.clone(),
            title: document.title.clone(),
            field: current_field.clone(),
            line: index + 1,
            snippet: local_search_snippet(line),
            score: local_search_score(line, query),
        });
    }

    results
}

fn local_search_metadata(
    path: &Path,
    relative: &str,
    content: &str,
) -> (String, Option<String>, String) {
    let file_stem = path
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("document")
        .to_string();
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");

    if relative == ".agentflow/goal.md" || relative == ".agentflow/goal.json" {
        return ("goal".to_string(), None, "Project Goal".to_string());
    }
    if relative == ".agentflow/project-definition.json" {
        return (
            "project-definition".to_string(),
            None,
            "Project Definition".to_string(),
        );
    }
    if relative == ".agentflow/scope-state.json" {
        return ("scope-state".to_string(), None, "Scope State".to_string());
    }
    if relative == ".agentflow/context.json" || relative == ".agentflow/context.md" {
        return ("context".to_string(), None, "Project Context".to_string());
    }
    if relative == ".agentflow/environment.md" {
        return ("environment".to_string(), None, "Environment".to_string());
    }
    if relative == ".agentflow/architecture.md" {
        return ("architecture".to_string(), None, "Architecture".to_string());
    }
    if relative == ".agentflow/roadmap.md" {
        return ("roadmap".to_string(), None, "Roadmap".to_string());
    }
    if relative == ".agentflow/index.json" {
        return (
            "index-summary".to_string(),
            None,
            "Project Index".to_string(),
        );
    }
    if relative == ".agentflow/settings.json" {
        return ("settings".to_string(), None, "Settings".to_string());
    }
    if relative.starts_with(".agentflow/bootstrap/") {
        return (
            "bootstrap".to_string(),
            None,
            markdown_title(content, &file_stem),
        );
    }
    if relative.starts_with(".agentflow/issues/") {
        let issue_id = file_stem.clone();
        return (
            "issue".to_string(),
            Some(issue_id),
            issue_title_from_content(content, &file_stem),
        );
    }
    if relative.starts_with(".agentflow/runs/") {
        let run_id = relative
            .split('/')
            .nth(2)
            .map(str::to_string)
            .unwrap_or_else(|| file_stem.clone());
        let kind = match file_name {
            "transcript.md" => "run-transcript",
            "commands.jsonl" => "run-command",
            "diff-summary.md" => "run-diff",
            _ => "run",
        };
        return (kind.to_string(), Some(run_id.clone()), run_id);
    }
    if relative.starts_with(".agentflow/evidence/") {
        return ("evidence".to_string(), Some(file_stem.clone()), file_stem);
    }
    if relative.starts_with(".agentflow/reviews/") {
        return ("review".to_string(), Some(file_stem.clone()), file_stem);
    }
    if relative.starts_with(".agentflow/updates/") {
        return (
            "project-update".to_string(),
            Some(file_stem.clone()),
            file_stem,
        );
    }
    if relative.starts_with(".agentflow/views/") {
        return ("saved-view".to_string(), Some(file_stem.clone()), file_stem);
    }

    ("file".to_string(), None, file_stem)
}

fn issue_title_from_content(content: &str, fallback: &str) -> String {
    if let Ok(issue) = serde_json::from_str::<IssueContract>(content) {
        return issue.title;
    }
    markdown_title(content, fallback)
}

fn markdown_title(content: &str, fallback: &str) -> String {
    content
        .lines()
        .find_map(|line| line.strip_prefix("# ").map(str::trim))
        .filter(|title| !title.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| fallback.to_string())
}

fn markdown_heading(line: &str) -> Option<String> {
    let trimmed = line.trim_start();
    let heading = trimmed.strip_prefix('#')?;
    let level_end = heading.find(|ch| ch != '#').unwrap_or(heading.len());
    if level_end > 5 {
        return None;
    }
    let title = heading[level_end..].trim();
    if title.is_empty() {
        None
    } else {
        Some(title.to_string())
    }
}

fn local_search_default_field(path: &str) -> String {
    if path.ends_with(".json") || path.ends_with(".jsonl") {
        "text".to_string()
    } else {
        "body".to_string()
    }
}

fn local_search_snippet(line: &str) -> String {
    let trimmed = line.trim();
    let max_chars = 180;
    if trimmed.chars().count() <= max_chars {
        trimmed.to_string()
    } else {
        let mut snippet = trimmed.chars().take(max_chars).collect::<String>();
        snippet.push_str("...");
        snippet
    }
}

fn local_search_score(line: &str, query: &str) -> u32 {
    if line.contains(query) {
        100
    } else {
        80
    }
}

fn local_search_path_allowed(relative: &str) -> bool {
    matches!(
        relative,
        ".agentflow/goal.md"
            | ".agentflow/goal.json"
            | ".agentflow/project-definition.json"
            | ".agentflow/scope-state.json"
            | ".agentflow/context.json"
            | ".agentflow/context.md"
            | ".agentflow/environment.md"
            | ".agentflow/architecture.md"
            | ".agentflow/roadmap.md"
            | ".agentflow/index.json"
            | ".agentflow/settings.json"
    ) || (relative.starts_with(".agentflow/bootstrap/") && relative.ends_with(".md"))
        || (relative.starts_with(".agentflow/issues/")
            && (relative.ends_with(".json") || relative.ends_with(".md")))
        || (relative.starts_with(".agentflow/runs/")
            && (relative.ends_with("/run.json")
                || relative.ends_with("/transcript.md")
                || relative.ends_with("/commands.jsonl")
                || relative.ends_with("/diff-summary.md")))
        || (relative.starts_with(".agentflow/evidence/") && relative.ends_with(".md"))
        || (relative.starts_with(".agentflow/reviews/") && relative.ends_with(".md"))
        || (relative.starts_with(".agentflow/updates/") && relative.ends_with(".md"))
        || (relative.starts_with(".agentflow/views/") && relative.ends_with(".json"))
}

fn local_search_path_excluded(relative: &str) -> bool {
    relative.starts_with(".git/")
        || relative.starts_with("target/")
        || relative.starts_with("node_modules/")
        || relative.starts_with("apps/desktop/dist/")
        || relative.starts_with(".agentflow/tmp/")
        || relative == ".agentflow/index.sqlite"
        || relative.starts_with(".agentflow/index.sqlite-")
        || relative.starts_with(".agentflow/search/")
        || relative.starts_with(".agentflow/queries/")
        || relative
            .rsplit('/')
            .next()
            .is_some_and(|name| name == ".env" || name.starts_with(".env."))
}

fn local_search_excluded_paths() -> Vec<String> {
    [
        ".git/",
        "target/",
        "node_modules/",
        "apps/desktop/dist/",
        ".agentflow/tmp/",
        ".agentflow/index.sqlite*",
        ".agentflow/search/",
        ".agentflow/queries/",
        ".env*",
        "binary files",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

fn project_update_markdown_count(project_dir: &Path) -> Result<usize> {
    Ok(markdown_files(&project_dir.join("updates"))?
        .iter()
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("PROJECT-UPDATE-") && name.ends_with(".md"))
        })
        .count())
}

fn local_metric_run_ref(run: &AgentRun) -> LocalMetricRunRef {
    LocalMetricRunRef {
        id: run.id.clone(),
        issue_id: run.issue_id.clone(),
        status: run.status.clone(),
        validation_status: validation_status(&run.validation_commands),
    }
}

fn local_metric_artifact_ref(artifact: &WorkbenchTextArtifact) -> LocalMetricArtifactRef {
    LocalMetricArtifactRef {
        path: artifact.path.clone(),
        title: artifact.title.clone(),
    }
}

fn local_project_issue_refs(
    project_dir: &Path,
    issues: &[IssueContract],
) -> Result<Vec<LocalProjectIssueRef>> {
    let mut refs = Vec::new();
    for issue in issues {
        let latest_run = latest_run_for_issue(project_dir, &issue.id)?;
        let next_action = if issue_state_done(&issue.status) {
            "completed".to_string()
        } else {
            let indexed = IndexedIssue {
                id: issue.id.clone(),
                title: issue.title.clone(),
                status: issue.status.clone(),
                intent: issue.intent.clone(),
                json_path: format!(".agentflow/issues/{}.json", issue.id),
            };
            issue_next_step(project_dir, &indexed)?.0
        };
        let validation = latest_run
            .as_ref()
            .map(|run| validation_status(&run.validation_commands))
            .unwrap_or_else(|| "not-run".to_string());
        let execution_state = issue_execution_state(issue, latest_run.as_ref(), &validation);
        refs.push(LocalProjectIssueRef {
            id: issue.id.clone(),
            title: issue.title.clone(),
            status: issue.status.clone(),
            canonical_status: canonical_issue_status_string(&issue.status),
            next_action,
            latest_run_id: latest_run.as_ref().map(|run| run.id.clone()),
            latest_run_status: latest_run.as_ref().map(|run| run.status.clone()),
            validation_status: validation,
            execution_state,
            evidence_path: latest_run
                .as_ref()
                .and_then(|run| agentflow_output_path(run.outputs.evidence.as_deref())),
            review_path: latest_run
                .as_ref()
                .and_then(|run| agentflow_output_path(run.outputs.review.as_deref())),
            project_update_path: latest_run
                .as_ref()
                .and_then(|run| agentflow_output_path(run.outputs.update.as_deref())),
        });
    }
    refs.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(refs)
}

fn issue_execution_state(
    issue: &IssueContract,
    latest_run: Option<&AgentRun>,
    validation_status: &str,
) -> String {
    let Some(run) = latest_run else {
        return "not-started".to_string();
    };
    if issue_state_done(&issue.status)
        && run.outputs.evidence.is_some()
        && run.outputs.review.is_some()
        && run.outputs.update.is_some()
    {
        return "completed".to_string();
    }
    if validation_status == "failed" {
        return "blocked".to_string();
    }
    if validation_status == "passed" {
        return "validated".to_string();
    }
    if run.validation_commands.is_empty() {
        return "run-created".to_string();
    }
    run.status.clone()
}

fn agentflow_output_path(output: Option<&str>) -> Option<String> {
    let output = output?.trim();
    if output.is_empty() {
        return None;
    }
    let normalized = output.trim_start_matches("../../").trim_start_matches("./");
    Some(
        normalized
            .strip_prefix(".agentflow/")
            .map(|path| format!(".agentflow/{path}"))
            .unwrap_or_else(|| {
                if normalized.starts_with("evidence/")
                    || normalized.starts_with("reviews/")
                    || normalized.starts_with("updates/")
                {
                    format!(".agentflow/{normalized}")
                } else {
                    normalized.to_string()
                }
            }),
    )
}

fn goal_loop_selection(project_id: &str, decision: GoalLoopDecision) -> GoalLoopSelection {
    GoalLoopSelection {
        active_project_id: Some(project_id.to_string()),
        source: "local-project-model-decision".to_string(),
        next_action: decision.next_action.clone(),
        next_issue_intent: if decision.next_action == "plan" {
            Some(decision.recommended_issue_intent)
        } else {
            None
        },
        recommended_command: decision.recommended_command,
        rationale: decision.rationale,
    }
}

fn local_project_model_sources() -> Vec<String> {
    [
        ".agentflow/goal.json",
        ".agentflow/project-definition.json",
        ".agentflow/scope-state.json",
        ".agentflow/settings.json",
        ".agentflow/roadmap.md",
        ".agentflow/goal-loop.json",
        ".agentflow/issues/*.json",
        ".agentflow/runs/*/run.json",
        ".agentflow/evidence/*.md",
        ".agentflow/reviews/*.md",
        ".agentflow/updates/*.md",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

fn project_milestone_issue_view_model_sources() -> Vec<String> {
    [
        "LocalProjectModelSnapshot",
        ".agentflow/workspace.json",
        ".agentflow/teams/*.json",
        ".agentflow/projects/*.json",
        ".agentflow/issues/*.json",
        ".agentflow/views/*.json",
        "docs/specs/project-milestone-issue-view-model-v1.md",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

fn project_milestone_issue_view_model_invariants() -> Vec<String> {
    [
        "Project 不执行。",
        "Milestone 不执行。",
        "Issue 执行。",
        "View 只展示。",
        "Queue Preflight 决定谁能执行。",
        "Evidence 决定是否 Done。",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

fn v1_project_status(status: &str) -> String {
    match normalized_status(status).as_str() {
        "planned" | "ready" | "confirmed" => "Planned",
        "active" => "Active",
        "audit" | "docs_refresh" | "final_review" | "closing" => "Closing",
        "completed" | "done" => "Completed",
        "paused" | "blocked" | "failed" => "Blocked",
        "canceled" | "cancelled" => "Canceled",
        "draft" => "Draft",
        _ => "Draft",
    }
    .to_string()
}

fn v1_issue_status(status: &str) -> String {
    match normalized_status(status).as_str() {
        "todo" | "planned" | "ready" => "Todo",
        "in_progress" | "active" | "eligible" | "leased" => "In Progress",
        "in_review" | "review" | "pr" | "checks_passing" | "merged" | "evidence_captured"
        | "needs_human_review" => "In Review",
        "done" | "completed" => "Done",
        "blocked" | "failed" => "Blocked",
        "repair" => "Repair",
        "canceled" | "cancelled" => "Canceled",
        "backlog" | "draft" => "Backlog",
        _ => "Backlog",
    }
    .to_string()
}

fn v1_milestone_status(
    project: &LocalProject,
    milestone: &LocalMilestone,
    issue_refs: &[&LocalProjectIssueRef],
) -> String {
    if issue_refs.iter().any(|issue| {
        matches!(
            v1_issue_status(&issue.status).as_str(),
            "Blocked" | "Repair"
        )
    }) {
        return "Blocked".to_string();
    }

    if milestone.progress.non_canceled_issue_count > 0
        && milestone.progress.done_issue_count == milestone.progress.non_canceled_issue_count
    {
        return "Done".to_string();
    }

    if issue_refs
        .iter()
        .any(|issue| v1_issue_status(&issue.status) == "In Review")
    {
        return "Review".to_string();
    }

    if project.active_milestone_id == milestone.id {
        return "Active".to_string();
    }

    if milestone.issue_ids.is_empty() {
        "Draft".to_string()
    } else {
        "Ready".to_string()
    }
}

fn v1_team_project_ids(projects: &[LocalProject]) -> HashMap<String, Vec<String>> {
    let mut ids_by_team = HashMap::<String, Vec<String>>::new();
    for project in projects {
        for team_id in &project.team_ids {
            ids_by_team
                .entry(team_id.clone())
                .or_default()
                .push(project.id.clone());
        }
    }
    ids_by_team
}

fn v1_issue_links(projects: &[LocalProject]) -> HashMap<String, (String, Option<String>)> {
    let mut links = HashMap::new();
    for project in projects {
        for issue_id in &project.issue_ids {
            links.insert(issue_id.clone(), (project.id.clone(), None));
        }
        for milestone in &project.milestones {
            for issue_id in &milestone.issue_ids {
                links.insert(
                    issue_id.clone(),
                    (project.id.clone(), Some(milestone.id.clone())),
                );
            }
        }
    }
    links
}

fn unique_issue_values<F>(issues: &[&IssueContract], values: F) -> Vec<String>
where
    F: Fn(&IssueContract) -> Vec<String>,
{
    let mut output = Vec::new();
    for issue in issues {
        for value in values(issue) {
            if !value.trim().is_empty() && !output.contains(&value) {
                output.push(value);
            }
        }
    }
    output
}

fn v1_milestone_exit_criteria() -> Vec<String> {
    [
        "All issues Done.",
        "Validation passed.",
        "Evidence complete.",
        "No blocker.",
        "Milestone review complete.",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

fn v1_default_acceptance_criteria() -> Vec<String> {
    [
        "Goal completed.",
        "Scope respected.",
        "Non-goals not expanded.",
        "Validation commands passed.",
        "Evidence complete.",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

fn v1_view_from_saved_view(view: &SavedView) -> V1View {
    V1View {
        id: view.id.clone(),
        name: view.name.clone(),
        entity: "issue".to_string(),
        filter: V1ViewFilter {
            issue_status: view.filter.issue_status.clone(),
            run_status: view.filter.run_status.clone(),
            validation_status: view.filter.validation_status.clone(),
            issue_id: view.filter.issue_id.clone(),
        },
        sort: vec![
            V1ViewSort {
                field: "milestoneOrder".to_string(),
                direction: "asc".to_string(),
            },
            V1ViewSort {
                field: "issueOrder".to_string(),
                direction: "asc".to_string(),
            },
        ],
        layout: "list".to_string(),
    }
}

fn local_project_seed_files(
    project_dir: &Path,
    snapshot: &LocalProjectModelSnapshot,
) -> Result<Vec<LocalProjectSeedFile>> {
    let workspace = snapshot
        .workspace
        .as_ref()
        .ok_or_else(|| anyhow!("LocalProjectSeed requires a workspace snapshot"))?;
    let team = snapshot
        .teams
        .first()
        .ok_or_else(|| anyhow!("LocalProjectSeed requires a team snapshot"))?;
    let project = snapshot
        .projects
        .first()
        .ok_or_else(|| anyhow!("LocalProjectSeed requires a project snapshot"))?;

    validate_seed_id(&workspace.id)?;
    validate_seed_id(&team.id)?;
    validate_seed_id(&project.id)?;

    Ok(vec![
        LocalProjectSeedFile {
            path: ".agentflow/workspace.json".to_string(),
            kind: "workspace".to_string(),
            action: seed_file_action(&project_dir.join("workspace.json")),
            content: workspace_seed_content(workspace),
        },
        LocalProjectSeedFile {
            path: format!(".agentflow/teams/{}.json", team.id),
            kind: "team".to_string(),
            action: seed_file_action(&project_dir.join("teams").join(format!("{}.json", team.id))),
            content: team_seed_content(team),
        },
        LocalProjectSeedFile {
            path: format!(".agentflow/projects/{}.json", project.id),
            kind: "project".to_string(),
            action: seed_file_action(
                &project_dir
                    .join("projects")
                    .join(format!("{}.json", project.id)),
            ),
            content: project_seed_content(project),
        },
    ])
}

fn workspace_seed_content(workspace: &LocalWorkspace) -> serde_json::Value {
    serde_json::json!({
        "version": workspace.version.clone(),
        "id": workspace.id.clone(),
        "name": workspace.name.clone(),
        "defaultTeamId": workspace.default_team_id.clone(),
        "activeProjectId": workspace.active_project_id.clone(),
        "teamIds": workspace.team_ids.clone(),
        "projectIds": workspace.project_ids.clone(),
        "source": local_project_seed_source(),
    })
}

fn team_seed_content(team: &LocalTeam) -> serde_json::Value {
    serde_json::json!({
        "version": team.version.clone(),
        "id": team.id.clone(),
        "name": team.name.clone(),
        "workflow": team.workflow.clone(),
        "defaultValidationCommands": team.default_validation_commands.clone(),
        "wipLimit": team.wip_limit,
        "issueIds": [],
        "source": local_project_seed_source(),
    })
}

fn project_seed_content(project: &LocalProject) -> serde_json::Value {
    let milestones = project
        .milestones
        .iter()
        .map(|milestone| {
            serde_json::json!({
                "id": milestone.id.clone(),
                "name": milestone.name.clone(),
                "title": milestone.name.clone(),
                "description": milestone.description.clone(),
                "sortOrder": milestone.sort_order,
                "target": milestone.target.clone(),
                "status": milestone.status.clone(),
                "issueIds": [],
                "completedIssueIds": [],
                "nextIssueIntent": milestone.next_issue_intent.clone(),
            })
        })
        .collect::<Vec<_>>();

    serde_json::json!({
        "version": project.version.clone(),
        "id": project.id.clone(),
        "name": project.name.clone(),
        "status": project.status.clone(),
        "goal": project.goal.clone(),
        "teamIds": project.team_ids.clone(),
        "activeMilestoneId": project.active_milestone_id.clone(),
        "milestones": milestones,
        "issueIds": [],
        "nextIssueIntent": project.next_issue_intent.clone(),
        "source": local_project_seed_source(),
    })
}

fn local_project_seed_source() -> serde_json::Value {
    serde_json::json!({
        "kind": "local-project-model-snapshot",
        "generatedFrom": "read_local_project_model_snapshot",
    })
}

fn seed_file_action(path: &Path) -> String {
    if path.exists() {
        "exists".to_string()
    } else {
        "create".to_string()
    }
}

fn validate_seed_id(id: &str) -> Result<()> {
    if id.is_empty() {
        bail!("seed id cannot be empty");
    }
    if !id
        .chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
    {
        bail!("seed id `{id}` must use lowercase letters, digits, and hyphen only");
    }
    Ok(())
}

fn seed_absolute_path(root: &Path, relative: &str) -> Result<PathBuf> {
    let allowed = relative == ".agentflow/workspace.json"
        || relative == ".agentflow/teams/core.json"
        || relative == ".agentflow/projects/agentflow-local-execution.json";
    if !allowed || relative.contains("..") || relative.starts_with('/') {
        bail!("Local Project Seed v0 refuses unexpected path `{relative}`");
    }
    Ok(root.join(relative))
}

fn rollback_project_seed_write(root: &Path, paths: &[PathBuf]) {
    for path in paths.iter().rev() {
        let _ = fs::remove_file(path);
    }
    let _ = fs::remove_dir(root.join(".agentflow/projects"));
    let _ = fs::remove_dir(root.join(".agentflow/teams"));
}

fn local_project_seed_confirmation_gates() -> Vec<String> {
    [
        "create-workspace-file",
        "create-team-directory",
        "create-project-directory",
        "create-team-file",
        "create-project-file",
        "overwrite-existing-seed",
        "change-default-team",
        "change-active-project",
        "link-existing-issues",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

fn local_project_seed_sources() -> Vec<String> {
    [
        "LocalProjectModelSnapshot",
        ".agentflow/goal.json",
        ".agentflow/settings.json",
        ".agentflow/scope-state.json",
        ".agentflow/roadmap.md",
        ".agentflow/issues/*.json",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

fn default_issue_project_link(snapshot: &LocalProjectModelSnapshot) -> Result<IssueProjectLink> {
    let team = snapshot
        .teams
        .first()
        .ok_or_else(|| anyhow!("IssueProjectLink requires a team snapshot"))?;
    let project = snapshot
        .projects
        .first()
        .ok_or_else(|| anyhow!("IssueProjectLink requires a project snapshot"))?;
    let milestone = project
        .milestones
        .iter()
        .find(|milestone| milestone.id == project.active_milestone_id)
        .or_else(|| project.milestones.first())
        .ok_or_else(|| anyhow!("IssueProjectLink requires a milestone snapshot"))?;

    validate_project_link_id(&team.id)?;
    validate_project_link_id(&project.id)?;
    validate_project_link_id(&milestone.id)?;

    Ok(IssueProjectLink {
        team_id: team.id.clone(),
        project_id: project.id.clone(),
        milestone_id: milestone.id.clone(),
        link_source: "issue-project-link-writer-v0".to_string(),
    })
}

fn validate_project_link_id(id: &str) -> Result<()> {
    if id.is_empty() {
        bail!("project link id cannot be empty");
    }
    if !id
        .chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
    {
        bail!("project link id `{id}` must use lowercase letters, digits, and hyphen only");
    }
    Ok(())
}

fn issue_project_link_confirmation_gates() -> Vec<String> {
    [
        "target-issue-id",
        "preview-default-link",
        "explicit-write-flag",
        "explicit-yes-confirmation",
        "refuse-existing-project-link",
        "write-json-and-markdown-only",
        "no-bulk-history-migration",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

fn issue_project_link_sources() -> Vec<String> {
    [
        "LocalProjectModelSnapshot",
        ".agentflow/issues/{issue-id}.json",
        ".agentflow/issues/{issue-id}.md",
        ".agentflow/goal.json",
        ".agentflow/settings.json",
        ".agentflow/scope-state.json",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

fn docs_claim_path_exists(repo: &Path, project_dir: &Path, claim: &str) -> bool {
    let path = Path::new(claim);
    if path.is_absolute() {
        return path.exists();
    }
    if let Some(agentflow_path) = claim.strip_prefix(".agentflow/") {
        return project_dir.join(agentflow_path).exists();
    }
    repo.join(path).exists() || project_dir.join(path).exists()
}

fn goal_loop_summary_markdown(state: &GoalLoopState) -> String {
    format!(
        "# Goal Loop Summary\n\n- Generated: 2026-05-22\n- Executor: Codex\n- Goal ready: `{}`\n- Active issue: `{}`\n- Next action: `{}`\n- Recommended intent: {}\n- Recommended command: `{}`\n\n## Counts\n\n| Item | Count |\n| --- | ---: |\n| Issues | {} |\n| Completed issues | {} |\n| Runs | {} |\n| Evidence reports | {} |\n| Reviews | {} |\n| Project updates | {} |\n\n## Incomplete Issues\n\n{}\n\n## Rationale\n\n{}\n\n## Boundary\n\n- Goal Loop is local-decision-only.\n- It does not execute code, create remote issues, call models, or bypass IssueContract.\n",
        state.goal_ready,
        state.active_issue_id.as_deref().unwrap_or("none"),
        state.next_action,
        state.recommended_issue_intent,
        state.recommended_command,
        state.counts.issues,
        state.counts.completed_issues,
        state.counts.runs,
        state.counts.evidence_reports,
        state.counts.reviews,
        state.counts.project_updates,
        goal_loop_issue_table(&state.incomplete_issues),
        markdown_list(&state.rationale)
    )
}

fn goal_loop_issue_table(issues: &[GoalLoopIssueRef]) -> String {
    if issues.is_empty() {
        return "| Issue | Status | Next action | Title |\n| --- | --- | --- | --- |\n| none | - | - | - |"
            .to_string();
    }
    let rows = issues
        .iter()
        .map(|issue| {
            format!(
                "| `{}` | {} | {} | {} |",
                issue.id, issue.status, issue.next_action, issue.title
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!("| Issue | Status | Next action | Title |\n| --- | --- | --- | --- |\n{rows}")
}

fn validation_assistant_detail(run: &AgentRun) -> String {
    if run.validation_commands.is_empty() {
        return "No validation commands were recorded.".to_string();
    }
    let passed = run
        .validation_commands
        .iter()
        .filter(|record| record.exit_code == 0)
        .count();
    format!(
        "{} / {} validation commands passed.",
        passed,
        run.validation_commands.len()
    )
}

fn project_summary_markdown(
    issues: &[IndexedIssue],
    runs: &[(IndexedRun, Vec<CommandRecord>)],
    updates: &[IndexedUpdate],
    views: &[SavedView],
    index_summary: &IndexSummary,
) -> String {
    let completed = issues
        .iter()
        .filter(|issue| issue_state_done(&issue.status))
        .count();
    let planned = issues
        .iter()
        .filter(|issue| canonical_issue_status(&issue.status) == IssueStatus::Todo)
        .count();
    let passed_runs = runs
        .iter()
        .filter(|(run, _)| run.validation_status == "passed")
        .count();
    let next_issue = issues
        .iter()
        .find(|issue| issue_status_open(&issue.status))
        .map(|issue| format!("{} - {}", issue.id, issue.title))
        .unwrap_or_else(|| "none".to_string());

    format!(
        "# Project Summary\n\n- Generated: 2026-05-22\n- Executor: Codex\n- SQLite index: `{}`\n\n## Counts\n\n| Item | Count |\n| --- | ---: |\n| Issues | {} |\n| Completed issues | {} |\n| Planned issues | {} |\n| Runs | {} |\n| Passed runs | {} |\n| Project updates | {} |\n| Saved views | {} |\n\n## Next Issue\n\n{}\n\n## Issues\n\n{}\n\n## Runs\n\n{}\n\n## Saved Views\n\n{}\n",
        index_summary.sqlite_path.display(),
        issues.len(),
        completed,
        planned,
        runs.len(),
        passed_runs,
        updates.len(),
        views.len(),
        next_issue,
        issue_summary_table(issues),
        run_summary_table(runs),
        saved_view_summary_table(views)
    )
}

fn issue_summary_table(issues: &[IndexedIssue]) -> String {
    if issues.is_empty() {
        return "| Issue | Status | Title |\n| --- | --- | --- |\n| none | - | - |".to_string();
    }
    let rows = issues
        .iter()
        .map(|issue| format!("| `{}` | {} | {} |", issue.id, issue.status, issue.title))
        .collect::<Vec<_>>()
        .join("\n");
    format!("| Issue | Status | Title |\n| --- | --- | --- |\n{rows}")
}

fn run_summary_table(runs: &[(IndexedRun, Vec<CommandRecord>)]) -> String {
    if runs.is_empty() {
        return "| Run | Issue | Status | Validation |\n| --- | --- | --- | --- |\n| none | - | - | - |"
            .to_string();
    }
    let rows = runs
        .iter()
        .map(|(run, _)| {
            format!(
                "| `{}` | `{}` | {} | {} |",
                run.id, run.issue_id, run.status, run.validation_status
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!("| Run | Issue | Status | Validation |\n| --- | --- | --- | --- |\n{rows}")
}

fn saved_view_summary_table(views: &[SavedView]) -> String {
    if views.is_empty() {
        return "| View | Issue status | Run status | Validation |\n| --- | --- | --- | --- |\n| none | - | - | - |".to_string();
    }
    let rows = views
        .iter()
        .map(|view| {
            format!(
                "| `{}` | {} | {} | {} |",
                view.id,
                option_text(&view.filter.issue_status),
                option_text(&view.filter.run_status),
                option_text(&view.filter.validation_status)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!("| View | Issue status | Run status | Validation |\n| --- | --- | --- | --- |\n{rows}")
}

fn option_text(value: &Option<String>) -> &str {
    value.as_deref().unwrap_or("-")
}

fn review_assistant_markdown(
    issue: &IssueContract,
    run: Option<&AgentRun>,
    ready: bool,
    checks: &[ReviewAssistantCheck],
) -> String {
    format!(
        "# {} Review Assistant\n\n- Generated: 2026-05-22\n- Executor: Codex\n- Issue: `{}`\n- Latest run: `{}`\n- Decision: `{}`\n\n## Checklist\n\n{}\n\n## Boundary\n\n- Local-only review assistant.\n- No remote PR operation.\n- No team workspace mutation.\n",
        issue.id,
        issue.id,
        run.map(|run| run.id.as_str()).unwrap_or("none"),
        if ready { "ready" } else { "needs-work" },
        review_assistant_table(checks)
    )
}

fn review_assistant_table(checks: &[ReviewAssistantCheck]) -> String {
    if checks.is_empty() {
        return "| Check | Status | Detail |\n| --- | --- | --- |\n| none | - | - |".to_string();
    }
    let rows = checks
        .iter()
        .map(|check| format!("| {} | {} | {} |", check.name, check.status, check.detail))
        .collect::<Vec<_>>()
        .join("\n");
    format!("| Check | Status | Detail |\n| --- | --- | --- |\n{rows}")
}

fn read_optional_settings(repo: &Path) -> Result<Option<Settings>> {
    let path = repo.join(AGENTFLOW_DIR).join("settings.json");
    if path.exists() {
        read_json(&path).map(Some)
    } else {
        Ok(None)
    }
}

fn read_optional_context(project_dir: &Path) -> Result<Option<ProjectContext>> {
    let path = project_dir.join("context.json");
    if path.exists() {
        read_json(&path).map(Some)
    } else {
        Ok(None)
    }
}

fn collect_files(repo: &Path, dir: &Path, files: &mut Vec<ContextFile>) -> Result<()> {
    for entry in fs::read_dir(dir).with_context(|| format!("read directory {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        let file_name = entry.file_name();
        let name = file_name.to_string_lossy();
        let relative = relative_path(repo, &path)?;

        if path.is_dir() {
            if should_skip_dir(&relative, &name) {
                continue;
            }
            collect_files(repo, &path, files)?;
            continue;
        }

        if !path.is_file() || should_skip_file(&relative, &name) {
            continue;
        }

        let metadata = fs::metadata(&path)?;
        files.push(ContextFile {
            kind: file_kind(&relative),
            path: relative,
            bytes: metadata.len(),
        });
    }
    Ok(())
}

fn relative_path(repo: &Path, path: &Path) -> Result<String> {
    let relative = path
        .strip_prefix(repo)
        .with_context(|| format!("strip repo prefix for {}", path.display()))?;
    Ok(relative.to_string_lossy().replace('\\', "/"))
}

fn should_skip_dir(relative: &str, name: &str) -> bool {
    matches!(
        name,
        ".git" | "target" | "node_modules" | ".next" | "dist" | "build"
    ) || matches!(relative, ".agentflow/runs" | ".agentflow/tmp")
}

fn should_skip_file(relative: &str, name: &str) -> bool {
    name == ".DS_Store"
        || name.starts_with(".env")
        || relative.starts_with(".git/")
        || relative.starts_with("target/")
        || relative == ".agentflow/index.sqlite"
        || relative.starts_with(".agentflow/index.sqlite-")
}

fn file_kind(path: &str) -> String {
    if path == "Cargo.toml" {
        "rust-manifest"
    } else if path.ends_with(".rs") {
        "rust-source"
    } else if path.ends_with(".md") {
        "markdown"
    } else if path.ends_with(".json") {
        "json"
    } else if path.ends_with(".toml") {
        "toml"
    } else if path.ends_with(".ts") || path.ends_with(".tsx") {
        "typescript"
    } else if path.ends_with(".js") || path.ends_with(".jsx") {
        "javascript"
    } else {
        "file"
    }
    .to_string()
}

fn detected_stacks(files: &[ContextFile]) -> Vec<String> {
    let mut stacks = Vec::new();
    if files.iter().any(|file| file.path == "Cargo.toml") {
        stacks.push("rust".to_string());
    }
    if files.iter().any(|file| file.path == "package.json") {
        stacks.push("node".to_string());
    }
    if files.iter().any(|file| file.path == "pyproject.toml") {
        stacks.push("python".to_string());
    }
    stacks
}

fn inferred_validation_commands(stacks: &[String]) -> Vec<String> {
    if stacks.iter().any(|stack| stack == "rust") {
        vec!["cargo test".to_string(), "git diff --check".to_string()]
    } else if stacks.iter().any(|stack| stack == "node") {
        vec!["npm test".to_string(), "git diff --check".to_string()]
    } else {
        vec!["git diff --check".to_string()]
    }
}

fn selected_context_files(project_context: &Option<ProjectContext>, intent: &str) -> Vec<String> {
    let mut selected = vec!["GOAL.md".to_string(), ".agentflow/goal.json".to_string()];
    if let Some(context) = project_context {
        selected.push(".agentflow/context.json".to_string());
        let tokens = intent_tokens(intent);
        let mut scored = context
            .files
            .iter()
            .map(|file| (context_score(file, &tokens), file.path.clone()))
            .filter(|(score, path)| *score > 0 && !selected.contains(path))
            .collect::<Vec<_>>();
        scored.sort_by(|left, right| right.0.cmp(&left.0).then_with(|| left.1.cmp(&right.1)));
        for (_, path) in scored.into_iter().take(8) {
            if !selected.contains(&path) {
                selected.push(path);
            }
        }
    }
    selected
}

fn intent_tokens(intent: &str) -> Vec<String> {
    intent
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .map(str::trim)
        .filter(|token| token.len() >= 3)
        .map(|token| token.to_ascii_lowercase())
        .collect()
}

fn context_score(file: &ContextFile, tokens: &[String]) -> u32 {
    let path = file.path.to_ascii_lowercase();
    let mut score = 0;
    for token in tokens {
        if path.contains(token) {
            score += 20;
        }
    }
    if matches!(
        file.path.as_str(),
        "Cargo.toml" | "README.md" | "ROADMAP.md" | "docs/specs/mvp-spec.md"
    ) {
        score += 10;
    }
    if path.starts_with("crates/agentflow-core/") || path.starts_with("crates/agentflow-cli/") {
        score += 8;
    }
    if file.kind == "rust-source" {
        score += 5;
    }
    if path.starts_with(".agentflow/runs/") || path.starts_with(".agentflow/tmp/") {
        0
    } else {
        score
    }
}

fn section_body<'a>(markdown: &'a str, heading: &str) -> Option<&'a str> {
    let marker = format!("## {heading}");
    let start = markdown.find(&marker)?;
    let after_heading = markdown[start + marker.len()..].trim_start_matches([' ', '\t']);
    let after_newline = after_heading.strip_prefix('\n').unwrap_or(after_heading);
    let end = after_newline
        .find("\n## ")
        .unwrap_or_else(|| after_newline.len());
    Some(after_newline[..end].trim())
}

fn first_paragraph(body: &str) -> Option<String> {
    body.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('>'))
        .map(|line| line.trim_start_matches("> ").to_string())
        .next()
}

fn bullets(body: &str) -> Vec<String> {
    body.lines()
        .map(str::trim)
        .filter_map(|line| {
            line.strip_prefix("- ")
                .or_else(|| line.strip_prefix("* "))
                .map(str::trim)
        })
        .filter(|line| !line.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn ensure_dir(path: &Path) -> Result<()> {
    fs::create_dir_all(path).with_context(|| format!("create directory {}", path.display()))
}

fn write_json<T: Serialize>(path: &Path, value: &T, force: bool) -> Result<()> {
    let json = serde_json::to_string_pretty(value)?;
    write_new(path, &(json + "\n"), force)
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T> {
    let content =
        fs::read_to_string(path).with_context(|| format!("read json {}", path.display()))?;
    serde_json::from_str(&content).with_context(|| format!("parse json {}", path.display()))
}

fn read_optional_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<Option<T>> {
    if path.exists() {
        read_json(path).map(Some)
    } else {
        Ok(None)
    }
}

fn read_optional_string(path: &Path) -> Result<Option<String>> {
    if path.exists() {
        fs::read_to_string(path)
            .with_context(|| format!("read {}", path.display()))
            .map(Some)
    } else {
        Ok(None)
    }
}

fn write_new(path: &Path, content: &str, force: bool) -> Result<()> {
    if path.exists() && !force {
        bail!(
            "{} already exists; pass --force to overwrite",
            path.display()
        );
    }
    if let Some(parent) = path.parent() {
        ensure_dir(parent)?;
    }
    fs::write(path, content).with_context(|| format!("write {}", path.display()))
}

fn write_maybe(path: &Path, content: &str, force: bool) -> Result<bool> {
    if path.exists() && !force {
        return Ok(false);
    }
    if let Some(parent) = path.parent() {
        ensure_dir(parent)?;
    }
    fs::write(path, content).with_context(|| format!("write {}", path.display()))?;
    Ok(true)
}

fn environment_markdown() -> String {
    "# Environment\n\n- Data policy: local-first.\n- Upload code by default: false.\n- Default validation: cargo test, git diff --check.\n".to_string()
}

fn architecture_markdown() -> String {
    "# Project Map\n\n- GoalCompiler\n- LocalProjectStore\n- ContextCollector\n- Planner\n- IssueContractBuilder\n- CodexRuntimeAdapter\n- ValidationRunner\n- EvidenceChain\n".to_string()
}

fn roadmap_markdown() -> String {
    "# Roadmap\n\n1. Goal Compiler\n2. Local Project Store\n3. Context Collector\n4. Planner\n5. Issue Contract Builder\n6. Codex Runtime Adapter\n7. Validation Runner\n8. Evidence Chain\n9. Review / ProjectUpdate Generator\n".to_string()
}

fn initialization_evidence(goal_path: &Path, goal: &ProjectGoal) -> String {
    format!(
        "# FLOW-0 Initialization Evidence\n\n- Source goal: `{}`\n- Objective: {}\n- First candidate: {}\n- Status: initialized\n",
        goal_path.display(),
        goal.objective,
        goal.first_candidate
    )
}

fn run_transcript(issue: &IssueContract, run: &AgentRun) -> String {
    format!(
        "# {} Transcript\n\n- Issue: `{}`\n- Project: `{}`\n- Milestone: `{}`\n- Lease: `{}`\n- Mode: `{}`\n- Status: `{}`\n\n## Adapter Decision\n\nCodex Runtime Adapter v0 only materializes the local run boundary after eligibility and lease gates. It does not invoke an external model, upload code, or edit files.\n\n## Controlled Run Plan\n\n- Goal: {}\n\n### Expected Files\n\n{}\n\n### Blocked Files / Areas\n\n{}\n\n### Planned Steps\n\n{}\n\n### Validation Commands\n\n{}\n\n### Evidence Requirements\n\n{}\n\n### Rollback Plan\n\n{}\n\n## Contract\n\n- Title: {}\n- Intent: {}\n",
        run.id,
        issue.id,
        run.project_id.as_deref().unwrap_or("none"),
        run.milestone_id.as_deref().unwrap_or("none"),
        run.lease_id.as_deref().unwrap_or("none"),
        run.mode,
        run.status,
        run.run_plan.goal,
        markdown_list(&run.run_plan.expected_files),
        markdown_list(&run.run_plan.blocked_files),
        markdown_list(&run.run_plan.planned_steps),
        markdown_list(&run.run_plan.validation_commands),
        markdown_list(&run.run_plan.evidence_requirements),
        markdown_list(&run.run_plan.rollback_plan),
        issue.title,
        issue.intent
    )
}

fn dry_run_diff_summary(issue: &IssueContract, run: &AgentRun) -> String {
    format!(
        "# {} Diff Summary\n\n- Issue: `{}`\n- Mode: `{}`\n- Project: `{}`\n- Milestone: `{}`\n- Lease: `{}`\n- Adapter file edits: none\n- Expected files: {}\n- Blocked files / areas: {}\n- Validation readiness: `{}`\n- Evidence requirements: {}\n- Note: Product Feature Controlled Run v0 records the execution boundary before local validation.\n",
        run.id,
        issue.id,
        run.mode,
        run.project_id.as_deref().unwrap_or("none"),
        run.milestone_id.as_deref().unwrap_or("none"),
        run.lease_id.as_deref().unwrap_or("none"),
        inline_list(&run.run_plan.expected_files),
        inline_list(&run.run_plan.blocked_files),
        if run.run_plan.validation_commands.is_empty() {
            "missing"
        } else {
            "ready"
        },
        inline_list(&run.run_plan.evidence_requirements)
    )
}

fn evidence_markdown(issue: &IssueContract, run: &AgentRun, passed: bool) -> String {
    format!(
        "# {} Evidence\n\n- Issue: `{}`\n- Run: `{}`\n- Project: `{}`\n- Milestone: `{}`\n- Lease: `{}`\n- Result: `{}`\n\n## Artifacts\n\n- Transcript: `.agentflow/runs/{}/transcript.md`\n- Commands: `.agentflow/runs/{}/commands.jsonl`\n- Diff summary: `.agentflow/runs/{}/diff-summary.md`\n\n## Validation\n\n{}\n\n## AEP Evidence Chain\n\n- Stop condition: {}\n- Graphify context status: {}\n\n### Docs Claim Trace\n\n{}\n\n### Boundary Confirmation\n\n{}\n\n## Known Limitations\n\n- Runtime Adapter v0 is dry-run only.\n- External model execution and apply gates remain future work.\n",
        issue.id,
        issue.id,
        run.id,
        run.project_id.as_deref().unwrap_or("none"),
        run.milestone_id.as_deref().unwrap_or("none"),
        run.lease_id.as_deref().unwrap_or("none"),
        if passed { "pass" } else { "fail" },
        run.id,
        run.id,
        run.id,
        validation_table(&run.validation_commands),
        issue.aep.stop_condition,
        issue.aep.graphify_context_status,
        markdown_list(&issue.aep.docs_claim_trace),
        markdown_list(&issue.aep.boundary_confirmation)
    )
}

fn review_markdown(issue: &IssueContract, run: &AgentRun, passed: bool) -> String {
    format!(
        "# {} Review\n\n- Issue: `{}`\n- Run: `{}`\n- Decision: `{}`\n\n## Checklist\n\n- Issue contract exists: pass\n- AEP protocol fields: {}\n- Boundary confirmation: {}\n- Docs claim trace: {}\n- Run artifact exists: pass\n- Validation commands recorded: {}\n- Evidence generated: pass\n- Project update generated: pass\n\n## Notes\n\n{}\n",
        issue.id,
        issue.id,
        run.id,
        if passed { "pass" } else { "needs-fix" },
        if aep_protocol_complete(&issue.aep) {
            "pass"
        } else {
            "fail"
        },
        if issue.aep.boundary_confirmation.is_empty() {
            "fail"
        } else {
            "pass"
        },
        if issue.aep.docs_claim_trace.is_empty() {
            "fail"
        } else {
            "pass"
        },
        if run.validation_commands.is_empty() {
            "fail"
        } else {
            "pass"
        },
        if passed {
            "All local validation commands passed."
        } else {
            "One or more local validation commands failed; keep issue open."
        }
    )
}

fn project_update_markdown(issue: &IssueContract, run: &AgentRun, passed: bool) -> String {
    format!(
        "# Project Update\n\n- Source issue: `{}`\n- Source run: `{}`\n- Status: `{}`\n\n## Summary\n\n{} now has a local run, validation, evidence, and review chain.\n\n## Evidence Links\n\n- Evidence: `.agentflow/evidence/{}-evidence.md`\n- Review: `.agentflow/reviews/{}-review.md`\n",
        issue.id,
        run.id,
        if passed { "completed" } else { "blocked" },
        issue.title,
        issue.id,
        issue.id
    )
}

fn write_milestone_summary_if_complete(
    project_dir: &Path,
    completed_issue: &IssueContract,
    completed_run: &AgentRun,
) -> Result<Option<PathBuf>> {
    let Some(link) = completed_issue.project_link.as_ref() else {
        return Ok(None);
    };
    validate_project_link_id(&link.project_id)?;
    validate_project_link_id(&link.milestone_id)?;

    let project_path = project_dir
        .join("projects")
        .join(format!("{}.json", link.project_id));
    if !project_path.exists() {
        return Ok(None);
    }

    let mut project: serde_json::Value = read_json(&project_path)?;
    let Some(milestone) = project
        .get("milestones")
        .and_then(serde_json::Value::as_array)
        .and_then(|milestones| {
            milestones.iter().find(|milestone| {
                milestone
                    .get("id")
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|id| id == link.milestone_id)
            })
        })
    else {
        return Ok(None);
    };

    let milestone_name =
        json_string_field(milestone, "name").unwrap_or_else(|| link.milestone_id.clone());
    let issue_ids = json_string_array_field(milestone, "issueIds");
    if issue_ids.is_empty()
        || !issue_ids
            .iter()
            .any(|issue_id| issue_id == &completed_issue.id)
    {
        return Ok(None);
    }

    let mut open_issue_ids = Vec::new();
    let mut issue_rows = Vec::new();
    for issue_id in &issue_ids {
        let issue = read_issue(project_dir, issue_id)?;
        if !issue_state_done(&issue.status) {
            open_issue_ids.push(issue_id.clone());
        }
        let latest_run = latest_run_for_issue(project_dir, issue_id)?;
        issue_rows.push(milestone_issue_summary_row(&issue, latest_run.as_ref()));
    }

    if !open_issue_ids.is_empty() {
        return Ok(None);
    }

    let summary_path = project_dir.join(format!(
        "evidence/MILESTONE-{}-evidence-summary.md",
        link.milestone_id
    ));
    write_new(
        &summary_path,
        &milestone_evidence_summary_markdown(
            &link.project_id,
            &link.milestone_id,
            &milestone_name,
            completed_issue,
            completed_run,
            &issue_rows,
        ),
        true,
    )?;
    mark_milestone_completed_in_project_seed(&mut project, &link.milestone_id, &issue_ids);
    write_json(&project_path, &project, true)?;

    Ok(Some(summary_path))
}

fn milestone_issue_summary_row(issue: &IssueContract, latest_run: Option<&AgentRun>) -> String {
    let run_id = latest_run
        .map(|run| run.id.as_str())
        .unwrap_or("not-run")
        .to_string();
    let validation = latest_run
        .map(|run| validation_status(&run.validation_commands))
        .unwrap_or_else(|| "not-run".to_string());
    let evidence = latest_run
        .and_then(|run| agentflow_output_path(run.outputs.evidence.as_deref()))
        .unwrap_or_else(|| "-".to_string());
    let review = latest_run
        .and_then(|run| agentflow_output_path(run.outputs.review.as_deref()))
        .unwrap_or_else(|| "-".to_string());
    let update = latest_run
        .and_then(|run| agentflow_output_path(run.outputs.update.as_deref()))
        .unwrap_or_else(|| "-".to_string());

    format!(
        "| `{}` | {} | {} | `{}` | {} | `{}` | `{}` | `{}` |",
        issue.id,
        issue.title.replace('|', "\\|"),
        issue.status,
        run_id,
        validation,
        evidence,
        review,
        update
    )
}

fn milestone_evidence_summary_markdown(
    project_id: &str,
    milestone_id: &str,
    milestone_name: &str,
    completed_issue: &IssueContract,
    completed_run: &AgentRun,
    issue_rows: &[String],
) -> String {
    format!(
        "# Milestone Evidence Summary\n\n- Project: `{project_id}`\n- Milestone: `{milestone_id}`\n- Milestone name: {milestone_name}\n- Status: `completed`\n- Completed by issue: `{}`\n- Source run: `{}`\n- Generated by: Codex\n\n## MVP Workflow\n\n1. Human 创建/确认 Project\n2. Project 下拆 Milestones\n3. 每个 Milestone 下挂 Issues\n4. AgentFlow 做 queue preflight\n5. 只推进当前 milestone 中唯一 eligible issue\n6. Issue 完成：PR / checks / merge 后置为未来远程 artifact；当前 v0 记录本地 run / validation / evidence / review\n7. Milestone 全部 Done：自动生成本 summary\n8. Project 全部 milestones Done：后续进入 Stage Code Audit + Root Docs Refresh\n\n## Issue Evidence Chain\n\n| Issue | Title | Status | Run | Validation | Evidence | Review | Project update |\n| --- | --- | --- | --- | --- | --- | --- | --- |\n{}\n\n## Boundary\n\n- 本 summary 只来自本地 `.agentflow/` 事实源。\n- 不创建远程 PR / Linear issue。\n- 不执行额外 run / verify / review。\n",
        completed_issue.id,
        completed_run.id,
        issue_rows.join("\n")
    )
}

fn mark_milestone_completed_in_project_seed(
    project: &mut serde_json::Value,
    milestone_id: &str,
    completed_issue_ids: &[String],
) {
    let active_milestone_id = json_string_field(project, "activeMilestoneId");
    let active_milestone_completed = active_milestone_id.as_deref() == Some(milestone_id);
    let mut next_active_id = None;
    let mut all_milestones_completed = false;

    if let Some(milestones) = project
        .get_mut("milestones")
        .and_then(serde_json::Value::as_array_mut)
    {
        for milestone in milestones.iter_mut() {
            if milestone
                .get("id")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|id| id == milestone_id)
            {
                milestone["status"] = serde_json::json!("completed");
                milestone["completedIssueIds"] = serde_json::json!(completed_issue_ids);
                milestone["nextIssueIntent"] = serde_json::Value::Null;
            }
        }

        if active_milestone_completed {
            for milestone in milestones.iter_mut() {
                let status =
                    json_string_field(milestone, "status").unwrap_or_else(|| "planned".to_string());
                if status != "completed" {
                    milestone["status"] = serde_json::json!("active");
                    next_active_id = json_string_field(milestone, "id");
                    break;
                }
            }
        }

        all_milestones_completed = milestones.iter().all(|milestone| {
            json_string_field(milestone, "status").as_deref() == Some("completed")
        });
    }

    if let Some(next_active_id) = next_active_id {
        project["activeMilestoneId"] = serde_json::json!(next_active_id);
        project["status"] = serde_json::json!("active");
    } else if all_milestones_completed {
        project["status"] = serde_json::json!("audit");
        project["nextIssueIntent"] = serde_json::Value::Null;
    }
}

fn validation_table(records: &[CommandRecord]) -> String {
    if records.is_empty() {
        return "| Command | Exit | Result |\n| --- | --- | --- |\n| none | - | not-run |"
            .to_string();
    }
    let rows = records
        .iter()
        .map(|record| {
            format!(
                "| `{}` | {} | {} |",
                record.command.replace('|', "\\|"),
                record.exit_code,
                record.status
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!("| Command | Exit | Result |\n| --- | --- | --- |\n{rows}")
}

fn context_markdown_body(context: &ProjectContext) -> String {
    let files = context
        .files
        .iter()
        .take(80)
        .map(|file| format!("- `{}` ({}, {} bytes)", file.path, file.kind, file.bytes))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "# Project Context\n\n## Detected Stacks\n\n{}\n\n## Validation Commands\n\n{}\n\n## Files\n\n{}\n",
        markdown_list(&context.detected_stacks),
        markdown_list(&context.validation_commands),
        if files.is_empty() {
            "- none".to_string()
        } else {
            files
        }
    )
}

fn issue_title(intent: &str) -> String {
    let trimmed = intent.trim();
    if trimmed.is_empty() {
        return "Untitled local issue".to_string();
    }
    let mut title: String = trimmed.chars().take(64).collect();
    if trimmed.chars().count() > 64 {
        title.push_str("...");
    }
    title
}

fn issue_markdown_body(contract: &IssueContract) -> String {
    let project_link = issue_project_link_markdown(&contract.project_link);
    format!(
        "# {}: {}\n\n## Intent\n\n{}\n\n## Risk Level\n\n{}\n\n## Scope\n\n{}\n\n## Non-Goals\n\n{}\n\n## Context\n\n- Repo: {}\n- Files:\n{}\n\n## Execution Plan\n\n{}\n\n## Validation\n\n{}\n\n## Evidence Requirements\n\n{}\n\n## Rollback Plan\n\n{}\n\n{}## AEP Protocol\n\n- Phase: {}\n- Stop condition: {}\n- Vertical slice: {}\n- Graphify context status: {}\n\n### Fastest Feedback Loop\n\n{}\n\n### Tracer Bullet Plan\n\n{}\n\n### Diagnose Plan\n\n{}\n\n### Docs Claim Trace\n\n{}\n\n### Boundary Confirmation\n\n{}\n\n### PR Handoff Requirements\n\n{}\n\n## Human Gate\n\n- Requires confirmation before file edits: {}\n- Requires confirmation before external network: {}\n\n## Status\n\n{}\n",
        contract.id,
        contract.title,
        contract.intent,
        contract.risk_level,
        markdown_list(&contract.scope),
        markdown_list(&contract.non_goals),
        contract.context.repo,
        indented_markdown_list(&contract.context.files),
        numbered_list(&contract.execution_plan),
        markdown_list(&contract.validation.commands),
        markdown_list(&contract.evidence_requirements),
        markdown_list(&contract.rollback_plan),
        project_link,
        contract.aep.phase,
        contract.aep.stop_condition,
        contract.aep.vertical_slice,
        contract.aep.graphify_context_status,
        markdown_list(&contract.aep.fastest_feedback_loop),
        numbered_list(&contract.aep.tracer_bullet_plan),
        numbered_list(&contract.aep.diagnose_plan),
        markdown_list(&contract.aep.docs_claim_trace),
        markdown_list(&contract.aep.boundary_confirmation),
        markdown_list(&contract.aep.pr_handoff_requirements),
        contract.human_gate.before_file_edits,
        contract.human_gate.before_external_network,
        contract.status
    )
}

fn issue_project_link_markdown(project_link: &Option<IssueProjectLink>) -> String {
    let Some(link) = project_link else {
        return String::new();
    };

    format!(
        "## Project Link\n\n- Team: `{}`\n- Project: `{}`\n- Milestone: `{}`\n- Link source: `{}`\n\n",
        link.team_id, link.project_id, link.milestone_id, link.link_source
    )
}

fn markdown_list(items: &[String]) -> String {
    if items.is_empty() {
        return "- none".to_string();
    }
    items
        .iter()
        .map(|item| format!("- {item}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn inline_list(items: &[String]) -> String {
    if items.is_empty() {
        "none".to_string()
    } else {
        items.join(", ")
    }
}

fn indented_markdown_list(items: &[String]) -> String {
    if items.is_empty() {
        return "  - none".to_string();
    }
    items
        .iter()
        .map(|item| format!("  - {item}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn numbered_list(items: &[String]) -> String {
    if items.is_empty() {
        return "1. none".to_string();
    }
    items
        .iter()
        .enumerate()
        .map(|(index, item)| format!("{}. {item}", index + 1))
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn require_goal_initialized(repo: &Path) -> Result<ProjectGoal> {
    let path = repo.join(AGENTFLOW_DIR).join("goal.json");
    if !path.exists() {
        return Err(anyhow!(
            "{} is missing; run agentflow init --from-goal first",
            path.display()
        ));
    }
    read_json(&path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    const GOAL: &str = "# Goal\n\n## Objective\n\nBuild a local execution spine.\n\n## Success\n\n- Initialize from a goal.\n- Generate issue contracts.\n\n## Out Of Scope\n\n- No cloud sync.\n- No account system.\n";

    fn write_project_candidate_seed(repo: &Path, next_issue_intent: &str) {
        let project_dir = repo
            .join(".agentflow/projects")
            .join("agentflow-local-execution.json");
        fs::create_dir_all(project_dir.parent().unwrap()).unwrap();
        let project = serde_json::json!({
            "version": VERSION,
            "id": "agentflow-local-execution",
            "name": "AgentFlow",
            "status": "active",
            "goal": "Build a local execution spine.",
            "teamIds": ["core"],
            "activeMilestoneId": "current-roadmap",
            "milestones": [
                {
                    "id": "current-roadmap",
                    "name": "Current Roadmap",
                    "status": "active",
                    "issueIds": [],
                    "completedIssueIds": [],
                    "nextIssueIntent": next_issue_intent
                }
            ],
            "issueIds": [],
            "nextIssueIntent": "Project-level fallback",
            "source": {
                "kind": "test-project-seed"
            }
        });
        write_json(&project_dir, &project, false).unwrap();
    }

    #[test]
    fn compiles_goal_sections() {
        let goal = compile_goal_from_markdown(GOAL);
        assert_eq!(goal.objective, "Build a local execution spine.");
        assert_eq!(goal.success_criteria.len(), 2);
        assert_eq!(goal.non_goals[0], "No cloud sync.");
        assert!(goal.constraints.contains(&"local-first".to_string()));
    }

    #[test]
    fn initializes_agentflow_from_goal() {
        let dir = tempdir().unwrap();
        let goal_path = dir.path().join("GOAL.md");
        fs::write(&goal_path, GOAL).unwrap();

        let summary = init_from_goal(dir.path(), &goal_path, false).unwrap();

        assert!(summary.goal_json.exists());
        assert!(dir.path().join(".agentflow/settings.json").exists());
        assert!(dir
            .path()
            .join(".agentflow/project-definition.json")
            .exists());
        assert!(dir.path().join(".agentflow/scope-state.json").exists());
        assert!(dir
            .path()
            .join(".agentflow/bootstrap/project-bootstrap-sequence.md")
            .exists());
        assert!(dir
            .path()
            .join(".agentflow/evidence/FLOW-0-initialization.md")
            .exists());
    }

    #[test]
    fn product_feature_preview_does_not_write_facts() {
        let dir = tempdir().unwrap();
        let goal_path = dir.path().join("GOAL.md");
        fs::write(&goal_path, GOAL).unwrap();
        init_from_goal(dir.path(), &goal_path, false).unwrap();
        let draft = product_feature_test_draft("示例产品功能");

        let summary = create_product_feature(dir.path(), draft, false, false).unwrap();

        assert_eq!(summary.snapshot.mode, "preview");
        assert!(summary.written_paths.is_empty());
        assert!(!summary.summary_path.exists());
        assert!(!dir
            .path()
            .join(format!(
                ".agentflow/projects/{}.json",
                summary.snapshot.project.id
            ))
            .exists());
        assert_eq!(summary.snapshot.issues.len(), 4);
    }

    #[test]
    fn product_feature_write_creates_project_milestones_and_issue_contracts() {
        let dir = tempdir().unwrap();
        let goal_path = dir.path().join("GOAL.md");
        fs::write(&goal_path, GOAL).unwrap();
        init_from_goal(dir.path(), &goal_path, false).unwrap();
        let draft = product_feature_test_draft("示例产品功能");

        let summary = create_product_feature(dir.path(), draft, true, true).unwrap();

        let project_dir = dir.path().join(".agentflow");
        let project_path =
            project_dir.join(format!("projects/{}.json", summary.snapshot.project.id));
        let workspace: serde_json::Value = read_json(&project_dir.join("workspace.json")).unwrap();
        let issue: IssueContract =
            read_json(&project_dir.join(format!("issues/{}.json", summary.snapshot.issues[0].id)))
                .unwrap();
        let project: serde_json::Value = read_json(&project_path).unwrap();
        assert!(summary.summary_path.exists());
        assert_eq!(workspace["activeProjectId"], summary.snapshot.project.id);
        assert_eq!(project["activeMilestoneId"], "project-charter");
        assert_eq!(project["status"], "active");
        assert_eq!(project["milestones"][0]["issueIds"][0], issue.id);
        assert_eq!(issue.status, "todo");
        assert_eq!(
            issue.project_link.as_ref().unwrap().link_source,
            "product-feature-creation-flow-v0"
        );
        assert_eq!(issue.risk_level, "medium");
        assert!(!issue.rollback_plan.is_empty());
        assert!(!issue.validation.commands.is_empty());
    }

    #[test]
    fn status_model_maps_legacy_status_and_derives_milestone_progress() {
        let dir = tempdir().unwrap();
        let goal_path = dir.path().join("GOAL.md");
        fs::write(&goal_path, GOAL).unwrap();
        init_from_goal(dir.path(), &goal_path, false).unwrap();
        let draft = product_feature_test_draft("状态模型验证功能");

        let summary = create_product_feature(dir.path(), draft, true, true).unwrap();
        let snapshot = read_local_project_model_snapshot(dir.path()).unwrap();
        let project = snapshot
            .projects
            .iter()
            .find(|project| project.id == summary.snapshot.project.id)
            .unwrap();
        let first_issue = snapshot
            .issue_refs
            .iter()
            .find(|issue| issue.id == summary.snapshot.issues[0].id)
            .unwrap();

        assert_eq!(canonical_project_status("planned"), ProjectStatus::Draft);
        assert_eq!(canonical_issue_status("planned"), IssueStatus::Todo);
        assert_eq!(project.canonical_status, "active");
        assert_eq!(first_issue.status, "todo");
        assert_eq!(first_issue.canonical_status, "todo");
        assert_eq!(project.milestones[0].progress.total_issue_count, 1);
        assert_eq!(project.milestones[0].progress.done_issue_count, 0);
    }

    #[test]
    fn project_milestone_issue_view_model_v1_derives_read_only_snapshot() {
        let dir = tempdir().unwrap();
        let goal_path = dir.path().join("GOAL.md");
        fs::write(&goal_path, GOAL).unwrap();
        init_from_goal(dir.path(), &goal_path, false).unwrap();
        let summary = create_product_feature(
            dir.path(),
            product_feature_test_draft("PMIV Model Alignment"),
            true,
            true,
        )
        .unwrap();
        let project_path = dir.path().join(format!(
            ".agentflow/projects/{}.json",
            summary.snapshot.project.id
        ));
        let project_before = fs::read_to_string(&project_path).unwrap();

        let snapshot = read_project_milestone_issue_view_model_snapshot(dir.path()).unwrap();
        let json = serde_json::to_value(&snapshot).unwrap();

        assert!(snapshot.initialized);
        assert!(snapshot.boundary.read_only);
        assert!(snapshot
            .invariants
            .contains(&"Project 不执行。".to_string()));
        assert_eq!(snapshot.projects.len(), 1);
        assert_eq!(snapshot.projects[0].status, "Active");
        assert_eq!(snapshot.projects[0].raw_status, "active");
        assert_eq!(snapshot.projects[0].milestones[0].status, "Active");
        assert_eq!(snapshot.issues[0].status, "Todo");
        assert_eq!(snapshot.issues[0].raw_status, "todo");
        assert_eq!(
            snapshot.issues[0].project_id.as_deref(),
            Some(summary.snapshot.project.id.as_str())
        );
        assert_eq!(
            snapshot.issues[0].milestone_id.as_deref(),
            Some("project-charter")
        );
        assert!(!snapshot.issues[0].validation_commands.is_empty());
        assert!(!snapshot.issues[0].evidence_required.is_empty());
        assert_eq!(
            json["projects"][0]["milestones"][0]["exitCriteria"][0],
            "All issues Done."
        );
        assert_eq!(fs::read_to_string(&project_path).unwrap(), project_before);
    }

    #[test]
    fn project_milestone_issue_view_model_v1_keeps_views_as_saved_filters() {
        let dir = tempdir().unwrap();
        let goal_path = dir.path().join("GOAL.md");
        fs::write(&goal_path, GOAL).unwrap();
        init_from_goal(dir.path(), &goal_path, false).unwrap();
        create_product_feature(
            dir.path(),
            product_feature_test_draft("View Filter Alignment"),
            true,
            true,
        )
        .unwrap();
        save_view(
            dir.path(),
            "当前 Todo",
            SavedViewFilter {
                issue_status: Some("todo".to_string()),
                run_status: None,
                validation_status: None,
                issue_id: None,
            },
        )
        .unwrap();

        let snapshot = read_project_milestone_issue_view_model_snapshot(dir.path()).unwrap();

        assert_eq!(snapshot.views.len(), 1);
        assert_eq!(snapshot.views[0].name, "当前 Todo");
        assert_eq!(snapshot.views[0].entity, "issue");
        assert_eq!(
            snapshot.views[0].filter.issue_status.as_deref(),
            Some("todo")
        );
        assert_eq!(snapshot.views[0].layout, "list");
        assert_eq!(snapshot.views[0].sort[0].field, "milestoneOrder");
        assert!(snapshot.invariants.contains(&"View 只展示。".to_string()));
    }

    #[test]
    fn product_feature_write_feeds_goal_next_and_eligibility() {
        let dir = tempdir().unwrap();
        let goal_path = dir.path().join("GOAL.md");
        fs::write(&goal_path, GOAL).unwrap();
        init_from_goal(dir.path(), &goal_path, false).unwrap();
        let draft = product_feature_test_draft("Product Feature Creation Flow v0");

        let summary = create_product_feature(dir.path(), draft, true, true).unwrap();
        let first_issue_id = summary.snapshot.issues[0].id.clone();
        let goal_next = write_goal_next(dir.path()).unwrap();
        let eligibility = write_workflow_eligibility(dir.path(), None).unwrap();

        assert_eq!(
            eligibility.snapshot.eligible_issue_id.as_deref(),
            Some(first_issue_id.as_str())
        );
        assert!(goal_next
            .recommended_issue_intent
            .contains(first_issue_id.as_str()));
        assert_eq!(
            goal_next.recommended_command,
            format!("agentflow run {first_issue_id} --dry-run")
        );
    }

    #[test]
    fn product_feature_execution_status_recommends_current_issue_run() {
        let dir = tempdir().unwrap();
        let goal_path = dir.path().join("GOAL.md");
        fs::write(&goal_path, GOAL).unwrap();
        init_from_goal(dir.path(), &goal_path, false).unwrap();
        create_product_feature(
            dir.path(),
            product_feature_test_draft("Product Feature Execution Flow v0"),
            true,
            true,
        )
        .unwrap();

        let snapshot = read_product_feature_execution_status(dir.path()).unwrap();

        assert!(snapshot.feature_ready);
        assert_eq!(snapshot.project_id, "product-feature-execution-flow-v0");
        assert_eq!(snapshot.active_milestone_id, "project-charter");
        assert_eq!(snapshot.current_issue.as_ref().unwrap().id, "ISSUE-0001");
        assert_eq!(snapshot.next_action, "run");
        assert_eq!(
            snapshot.recommended_command,
            "agentflow run ISSUE-0001 --dry-run"
        );
    }

    #[test]
    fn product_feature_execution_next_moves_from_run_to_verify_to_review_to_next_milestone() {
        let dir = tempdir().unwrap();
        let goal_path = dir.path().join("GOAL.md");
        fs::write(&goal_path, GOAL).unwrap();
        init_from_goal(dir.path(), &goal_path, false).unwrap();
        create_product_feature(
            dir.path(),
            product_feature_test_draft("Feature Execution Lifecycle"),
            true,
            true,
        )
        .unwrap();
        replace_validation_commands(dir.path(), "ISSUE-0001", vec!["printf ok".to_string()]);

        run_issue(dir.path(), "ISSUE-0001").unwrap();
        let after_run = read_product_feature_execution_next(dir.path()).unwrap();
        assert_eq!(after_run.next_action, "verify");
        assert_eq!(after_run.recommended_command, "agentflow verify ISSUE-0001");

        verify_issue(dir.path(), "ISSUE-0001").unwrap();
        let after_verify = read_product_feature_execution_next(dir.path()).unwrap();
        assert_eq!(after_verify.next_action, "review");
        assert_eq!(
            after_verify.recommended_command,
            "agentflow review ISSUE-0001"
        );

        review_issue(dir.path(), "ISSUE-0001").unwrap();
        let after_review = read_product_feature_execution_next(dir.path()).unwrap();
        assert_eq!(after_review.active_milestone_id, "milestone-plan");
        assert_eq!(
            after_review.current_issue.as_ref().unwrap().id,
            "ISSUE-0002"
        );
        assert_eq!(after_review.next_action, "run");
    }

    #[test]
    fn controlled_run_records_plan_and_updates_feature_status() {
        let dir = tempdir().unwrap();
        let goal_path = dir.path().join("GOAL.md");
        fs::write(&goal_path, GOAL).unwrap();
        init_from_goal(dir.path(), &goal_path, false).unwrap();
        create_product_feature(
            dir.path(),
            product_feature_test_draft("Controlled Run Flow"),
            true,
            true,
        )
        .unwrap();

        let run_summary = run_issue(dir.path(), "ISSUE-0001").unwrap();
        assert_eq!(
            run_summary.run.project_id.as_deref(),
            Some("controlled-run-flow")
        );
        assert_eq!(
            run_summary.run.milestone_id.as_deref(),
            Some("project-charter")
        );
        assert!(run_summary.run.lease_id.is_some());
        assert!(!run_summary.run.run_plan.planned_steps.is_empty());
        assert!(!run_summary.run.run_plan.expected_files.is_empty());
        assert!(!run_summary.run.run_plan.validation_commands.is_empty());
        assert!(!run_summary.run.run_plan.evidence_requirements.is_empty());
        assert!(!run_summary.run.run_plan.rollback_plan.is_empty());

        let after_run = read_product_feature_execution_status(dir.path()).unwrap();
        let issue = after_run.current_issue.as_ref().unwrap();
        assert!(issue.dry_run_recorded);
        assert_eq!(
            issue.latest_run_id.as_deref(),
            Some(run_summary.run_id.as_str())
        );
        assert_eq!(issue.next_action, "verify");
        assert_eq!(issue.recommended_command, "agentflow verify ISSUE-0001");
        assert!(!issue.latest_run_plan.is_empty());
        assert!(!issue.expected_files.is_empty());
        assert!(!issue.validation_commands.is_empty());
        assert!(!issue.evidence_requirements.is_empty());
    }

    #[test]
    fn goal_check_requires_aep_bootstrap_artifacts() {
        let dir = tempdir().unwrap();
        let goal_path = dir.path().join("GOAL.md");
        fs::write(&goal_path, GOAL).unwrap();
        init_from_goal(dir.path(), &goal_path, false).unwrap();

        let readiness = check_goal_readiness(dir.path()).unwrap();

        assert!(readiness.ready);
        assert!(readiness
            .checks
            .iter()
            .any(|check| check.path == "project-definition.json"));
        assert!(readiness
            .checks
            .iter()
            .any(|check| check.path == "scope-state.json"));
    }

    #[test]
    fn plans_issue_and_updates_index() {
        let dir = tempdir().unwrap();
        let goal_path = dir.path().join("GOAL.md");
        fs::write(&goal_path, GOAL).unwrap();
        init_from_goal(dir.path(), &goal_path, false).unwrap();

        let summary = plan_issue(dir.path(), "Create the first contract").unwrap();
        let index: ProjectIndex = read_json(&dir.path().join(".agentflow/index.json")).unwrap();

        assert_eq!(summary.issue_id, "ISSUE-0001");
        assert!(summary.issue_json.exists());
        assert_eq!(index.next_issue_number, 2);
        assert_eq!(index.issues.len(), 1);
        let issue: IssueContract = read_json(&summary.issue_json).unwrap();
        assert_eq!(issue.aep.phase, "AEP Issue Execution");
        assert!(!issue.aep.docs_claim_trace.is_empty());
    }

    #[test]
    fn goal_next_recommends_planning_when_no_issue_is_open() {
        let dir = tempdir().unwrap();
        let goal_path = dir.path().join("GOAL.md");
        fs::write(&goal_path, GOAL).unwrap();
        init_from_goal(dir.path(), &goal_path, false).unwrap();

        let summary = write_goal_next(dir.path()).unwrap();
        let state: GoalLoopState =
            read_json(&dir.path().join(".agentflow/goal-loop.json")).unwrap();

        assert!(summary.goal_ready);
        assert_eq!(summary.next_action, "plan");
        assert!(summary.recommended_command.starts_with("agentflow plan"));
        assert!(summary.active_issue_id.is_none());
        assert_eq!(state.counts.issues, 0);
    }

    #[test]
    fn goal_next_keeps_wip_one_for_active_issue() {
        let dir = tempdir().unwrap();
        let goal_path = dir.path().join("GOAL.md");
        fs::write(&goal_path, GOAL).unwrap();
        init_from_goal(dir.path(), &goal_path, false).unwrap();
        let issue = plan_issue(dir.path(), "Implement active issue flow").unwrap();
        run_issue(dir.path(), &issue.issue_id).unwrap();

        let summary = write_goal_next(dir.path()).unwrap();

        assert_eq!(summary.active_issue_id.as_deref(), Some("ISSUE-0001"));
        assert_eq!(summary.next_action, "verify");
        assert_eq!(summary.recommended_command, "agentflow verify ISSUE-0001");
        assert!(summary
            .recommended_issue_intent
            .contains("先完成 active issue"));
    }

    #[test]
    fn project_aware_goal_loop_keeps_active_issue_before_project_candidate() {
        let dir = tempdir().unwrap();
        let goal_path = dir.path().join("GOAL.md");
        fs::write(&goal_path, GOAL).unwrap();
        init_from_goal(dir.path(), &goal_path, false).unwrap();
        write_project_candidate_seed(dir.path(), "Project candidate slice");
        let issue = plan_issue(dir.path(), "Active issue wins").unwrap();
        run_issue(dir.path(), &issue.issue_id).unwrap();

        let summary = write_goal_next(dir.path()).unwrap();

        assert_eq!(summary.active_issue_id.as_deref(), Some("ISSUE-0001"));
        assert_eq!(summary.next_action, "verify");
        assert_eq!(summary.recommended_command, "agentflow verify ISSUE-0001");
        assert!(summary
            .recommended_issue_intent
            .contains("先完成 active issue"));
    }

    #[test]
    fn project_aware_goal_loop_keeps_incomplete_issue_before_project_candidate() {
        let dir = tempdir().unwrap();
        let goal_path = dir.path().join("GOAL.md");
        fs::write(&goal_path, GOAL).unwrap();
        init_from_goal(dir.path(), &goal_path, false).unwrap();
        write_project_candidate_seed(dir.path(), "Project candidate slice");
        plan_issue(dir.path(), "Incomplete issue wins").unwrap();

        let summary = write_goal_next(dir.path()).unwrap();

        assert_eq!(summary.active_issue_id, None);
        assert_eq!(summary.next_action, "run");
        assert_eq!(
            summary.recommended_command,
            "agentflow run ISSUE-0001 --dry-run"
        );
        assert!(summary
            .recommended_issue_intent
            .contains("继续未完成 issue ISSUE-0001"));
    }

    #[test]
    fn goal_next_uses_active_milestone_queue_before_outside_backlog() {
        let dir = tempdir().unwrap();
        let goal_path = dir.path().join("GOAL.md");
        fs::write(&goal_path, GOAL).unwrap();
        init_from_goal(dir.path(), &goal_path, false).unwrap();
        let outside = plan_issue(dir.path(), "Outside backlog issue").unwrap();
        write_local_project_seed(dir.path(), true).unwrap();
        let project_dir = dir.path().join(".agentflow");
        fs::write(
            project_dir.join("projects/agentflow-local-execution.json"),
            serde_json::to_string_pretty(&serde_json::json!({
                "version": VERSION,
                "id": "agentflow-local-execution",
                "name": "AgentFlow MVP Productization",
                "status": "active",
                "goal": "Ship the local-first MVP through project, milestone, and issue planning.",
                "teamIds": ["core"],
                "activeMilestoneId": "mvp-current",
                "milestones": [
                    {
                        "id": "mvp-current",
                        "name": "当前 Milestone",
                        "status": "active",
                        "issueIds": [],
                        "completedIssueIds": [],
                        "nextIssueIntent": "Inside milestone issue"
                    }
                ],
                "issueIds": [],
                "nextIssueIntent": null,
            }))
            .unwrap(),
        )
        .unwrap();
        let inside = plan_issue(dir.path(), "Inside milestone issue").unwrap();

        let summary = write_goal_next(dir.path()).unwrap();
        let state: GoalLoopState =
            read_json(&dir.path().join(".agentflow/goal-loop.json")).unwrap();

        assert_eq!(outside.issue_id, "ISSUE-0001");
        assert_eq!(inside.issue_id, "ISSUE-0002");
        assert_eq!(summary.next_action, "run");
        assert_eq!(
            summary.recommended_command,
            "agentflow run ISSUE-0002 --dry-run"
        );
        assert!(summary
            .recommended_issue_intent
            .contains("当前 milestone `mvp-current`"));
        assert!(state
            .rationale
            .iter()
            .any(|line| line.contains("queue preflight")));
    }

    #[test]
    fn project_aware_goal_loop_uses_active_project_candidate_when_no_issue_is_open() {
        let dir = tempdir().unwrap();
        let goal_path = dir.path().join("GOAL.md");
        fs::write(&goal_path, GOAL).unwrap();
        init_from_goal(dir.path(), &goal_path, false).unwrap();
        write_project_candidate_seed(dir.path(), "Desktop GoalLoop Trace v0 只读展示");

        let summary = write_goal_next(dir.path()).unwrap();
        let state: GoalLoopState =
            read_json(&dir.path().join(".agentflow/goal-loop.json")).unwrap();
        let index: ProjectIndex = read_json(&dir.path().join(".agentflow/index.json")).unwrap();

        assert_eq!(summary.active_issue_id, None);
        assert_eq!(summary.next_action, "plan");
        assert_eq!(
            summary.recommended_issue_intent,
            "Desktop GoalLoop Trace v0 只读展示"
        );
        assert_eq!(
            summary.recommended_command,
            "agentflow plan \"Desktop GoalLoop Trace v0 只读展示\""
        );
        assert!(state
            .rationale
            .iter()
            .any(|line| line.contains("active project candidate")));
        assert_eq!(index.issues.len(), 0);
        assert!(!dir
            .path()
            .join(".agentflow/issues/ISSUE-0001.json")
            .exists());
    }

    #[test]
    fn project_aware_goal_loop_falls_back_to_roadmap_when_project_candidate_is_missing() {
        let dir = tempdir().unwrap();
        let goal_path = dir.path().join("GOAL.md");
        fs::write(&goal_path, GOAL).unwrap();
        fs::write(
            dir.path().join("ROADMAP.md"),
            "## 当前下一步\n\n候选施工包：`Roadmap Fallback Slice`\n",
        )
        .unwrap();
        init_from_goal(dir.path(), &goal_path, false).unwrap();

        let summary = write_goal_next(dir.path()).unwrap();

        assert_eq!(summary.next_action, "plan");
        assert_eq!(summary.recommended_issue_intent, "Roadmap Fallback Slice");
        assert_eq!(
            summary.recommended_command,
            "agentflow plan \"Roadmap Fallback Slice\""
        );
    }

    #[test]
    fn desktop_snapshot_reads_agentflow_facts_without_writing() {
        let dir = tempdir().unwrap();
        let goal_path = dir.path().join("GOAL.md");
        fs::write(&goal_path, GOAL).unwrap();
        init_from_goal(dir.path(), &goal_path, false).unwrap();
        let issue = plan_issue(dir.path(), "Implement desktop snapshot").unwrap();
        replace_validation_commands(dir.path(), &issue.issue_id, vec!["printf ok".to_string()]);
        run_issue(dir.path(), &issue.issue_id).unwrap();
        verify_issue(dir.path(), &issue.issue_id).unwrap();
        review_issue(dir.path(), &issue.issue_id).unwrap();
        write_goal_next(dir.path()).unwrap();

        let snapshot = read_desktop_workbench_snapshot(dir.path()).unwrap();

        assert!(snapshot.initialized);
        assert_eq!(snapshot.counts.issues, 1);
        assert_eq!(snapshot.counts.runs, 1);
        assert_eq!(snapshot.counts.passed_runs, 1);
        assert_eq!(snapshot.boundary.read_only, true);
        assert!(snapshot.project_summary_markdown.is_none());
        assert!(snapshot.goal_loop.is_some());
        assert!(snapshot
            .evidence
            .iter()
            .any(|artifact| artifact.path.ends_with("ISSUE-0001-evidence.md")));
        assert!(snapshot
            .project_updates
            .iter()
            .any(|artifact| artifact.path.ends_with("PROJECT-UPDATE-0001.md")));
    }

    #[test]
    fn desktop_snapshot_reports_missing_initialization() {
        let dir = tempdir().unwrap();

        let snapshot = read_desktop_workbench_snapshot(dir.path()).unwrap();

        assert!(!snapshot.initialized);
        assert_eq!(snapshot.counts.issues, 0);
        assert!(snapshot.issues.is_empty());
        assert!(snapshot.boundary.read_only);
    }

    #[test]
    fn local_metrics_snapshot_reads_only_derived_counts() {
        let dir = tempdir().unwrap();
        let goal_path = dir.path().join("GOAL.md");
        fs::write(&goal_path, GOAL).unwrap();
        init_from_goal(dir.path(), &goal_path, false).unwrap();
        let issue = plan_issue(dir.path(), "Implement metrics snapshot").unwrap();
        replace_validation_commands(dir.path(), &issue.issue_id, vec!["printf ok".to_string()]);
        run_issue(dir.path(), &issue.issue_id).unwrap();
        verify_issue(dir.path(), &issue.issue_id).unwrap();
        review_issue(dir.path(), &issue.issue_id).unwrap();
        write_goal_next(dir.path()).unwrap();

        let metrics = read_local_metrics_snapshot(dir.path()).unwrap();

        assert!(metrics.initialized);
        assert_eq!(metrics.issues.total, 1);
        assert_eq!(metrics.issues.completed, 1);
        assert_eq!(metrics.issues.planned, 0);
        assert_eq!(metrics.issues.active, 0);
        assert_eq!(metrics.runs.total, 1);
        assert_eq!(metrics.runs.passed, 1);
        assert_eq!(metrics.runs.failed, 0);
        assert_eq!(metrics.runs.missing_validation, 0);
        assert_eq!(metrics.artifacts.project_updates, 1);
        assert_eq!(metrics.artifacts.saved_views, 0);
        assert!(metrics.goal_ready);
        assert_eq!(metrics.next_action, "plan");
        assert_eq!(metrics.latest_run.as_ref().unwrap().id, "RUN-0001");
        assert!(metrics
            .latest_evidence
            .as_ref()
            .unwrap()
            .path
            .ends_with("ISSUE-0001-evidence.md"));
        assert!(metrics.boundary.read_only);
        assert!(!dir.path().join(".agentflow/analytics").exists());
    }

    #[test]
    fn local_metrics_snapshot_reports_missing_initialization() {
        let dir = tempdir().unwrap();

        let metrics = read_local_metrics_snapshot(dir.path()).unwrap();

        assert!(!metrics.initialized);
        assert_eq!(metrics.issues.total, 0);
        assert_eq!(metrics.runs.total, 0);
        assert_eq!(metrics.next_action, "wait-human");
        assert!(metrics.boundary.read_only);
    }

    #[test]
    fn local_project_model_snapshot_reads_current_facts_without_writing() {
        let dir = tempdir().unwrap();
        let goal_path = dir.path().join("GOAL.md");
        fs::write(&goal_path, GOAL).unwrap();
        init_from_goal(dir.path(), &goal_path, false).unwrap();
        let issue = plan_issue(dir.path(), "Local Project Model v0 只读实现").unwrap();
        replace_validation_commands(dir.path(), &issue.issue_id, vec!["printf ok".to_string()]);
        run_issue(dir.path(), &issue.issue_id).unwrap();
        verify_issue(dir.path(), &issue.issue_id).unwrap();
        review_issue(dir.path(), &issue.issue_id).unwrap();
        write_goal_next(dir.path()).unwrap();

        let snapshot = read_local_project_model_snapshot(dir.path()).unwrap();

        assert!(snapshot.initialized);
        assert!(snapshot.boundary.read_only);
        assert_eq!(snapshot.workspace.as_ref().unwrap().id, "default");
        assert_eq!(snapshot.teams.len(), 1);
        assert_eq!(snapshot.teams[0].id, "core");
        assert_eq!(snapshot.projects.len(), 1);
        assert_eq!(snapshot.projects[0].id, "agentflow-local-execution");
        assert_eq!(snapshot.projects[0].issue_count, 1);
        assert_eq!(snapshot.projects[0].completed_issue_count, 1);
        assert_eq!(snapshot.projects[0].milestones.len(), 1);
        assert_eq!(snapshot.issue_refs.len(), 1);
        assert_eq!(snapshot.issue_refs[0].validation_status, "passed");
        assert_eq!(snapshot.issue_refs[0].execution_state, "completed");
        assert_eq!(
            snapshot.issue_refs[0].latest_run_status.as_deref(),
            Some("completed")
        );
        assert!(snapshot.issue_refs[0]
            .evidence_path
            .as_ref()
            .is_some_and(|path| path.ends_with("ISSUE-0001-evidence.md")));
        assert!(snapshot.issue_refs[0]
            .review_path
            .as_ref()
            .is_some_and(|path| path.ends_with("ISSUE-0001-review.md")));
        assert!(snapshot.issue_refs[0]
            .project_update_path
            .as_ref()
            .is_some_and(|path| path.ends_with("PROJECT-UPDATE-0001.md")));
        assert_eq!(snapshot.goal_loop_selection.next_action, "plan");
        assert!(!dir.path().join(".agentflow/workspace.json").exists());
        assert!(!dir.path().join(".agentflow/teams").exists());
        assert!(!dir.path().join(".agentflow/projects").exists());
    }

    #[test]
    fn local_project_model_snapshot_reports_missing_initialization() {
        let dir = tempdir().unwrap();

        let snapshot = read_local_project_model_snapshot(dir.path()).unwrap();

        assert!(!snapshot.initialized);
        assert!(snapshot.workspace.is_none());
        assert!(snapshot.teams.is_empty());
        assert!(snapshot.projects.is_empty());
        assert_eq!(snapshot.goal_loop_selection.next_action, "wait-human");
        assert!(snapshot.boundary.read_only);
    }

    #[test]
    fn local_project_seed_preview_does_not_write_files() {
        let dir = tempdir().unwrap();
        let goal_path = dir.path().join("GOAL.md");
        fs::write(&goal_path, GOAL).unwrap();
        init_from_goal(dir.path(), &goal_path, false).unwrap();

        let preview = read_local_project_seed_preview(dir.path()).unwrap();

        assert!(preview.initialized);
        assert!(preview.boundary.read_only);
        assert!(preview.writes_required);
        assert_eq!(preview.files.len(), 3);
        assert!(preview.files.iter().all(|file| file.action == "create"));
        assert!(preview
            .confirmation_gates
            .contains(&"create-workspace-file".to_string()));
        assert!(!dir.path().join(".agentflow/workspace.json").exists());
        assert!(!dir.path().join(".agentflow/teams").exists());
        assert!(!dir.path().join(".agentflow/projects").exists());
    }

    #[test]
    fn local_project_seed_write_requires_confirmation() {
        let dir = tempdir().unwrap();
        let goal_path = dir.path().join("GOAL.md");
        fs::write(&goal_path, GOAL).unwrap();
        init_from_goal(dir.path(), &goal_path, false).unwrap();

        let result = write_local_project_seed(dir.path(), false);

        assert!(result.is_err());
        assert!(!dir.path().join(".agentflow/workspace.json").exists());
        assert!(!dir.path().join(".agentflow/teams").exists());
        assert!(!dir.path().join(".agentflow/projects").exists());
    }

    #[test]
    fn local_project_seed_write_creates_seed_files_after_confirmation() {
        let dir = tempdir().unwrap();
        let goal_path = dir.path().join("GOAL.md");
        fs::write(&goal_path, GOAL).unwrap();
        init_from_goal(dir.path(), &goal_path, false).unwrap();
        let issue = plan_issue(dir.path(), "Issue Project Link v0 边界定义").unwrap();
        let issue_before = fs::read_to_string(&issue.issue_json).unwrap();

        let summary = write_local_project_seed(dir.path(), true).unwrap();

        assert_eq!(summary.written_paths.len(), 3);
        assert!(dir.path().join(".agentflow/workspace.json").exists());
        assert!(dir.path().join(".agentflow/teams/core.json").exists());
        assert!(dir
            .path()
            .join(".agentflow/projects/agentflow-local-execution.json")
            .exists());
        let workspace: serde_json::Value =
            read_json(&dir.path().join(".agentflow/workspace.json")).unwrap();
        let team: serde_json::Value =
            read_json(&dir.path().join(".agentflow/teams/core.json")).unwrap();
        let project: serde_json::Value = read_json(
            &dir.path()
                .join(".agentflow/projects/agentflow-local-execution.json"),
        )
        .unwrap();
        assert_eq!(workspace["id"], "default");
        assert_eq!(team["id"], "core");
        assert_eq!(team["issueIds"].as_array().unwrap().len(), 0);
        assert_eq!(project["id"], "agentflow-local-execution");
        assert_eq!(project["issueIds"].as_array().unwrap().len(), 0);
        assert_eq!(fs::read_to_string(&issue.issue_json).unwrap(), issue_before);
    }

    #[test]
    fn local_project_model_snapshot_prefers_seed_files_when_present() {
        let dir = tempdir().unwrap();
        let goal_path = dir.path().join("GOAL.md");
        fs::write(&goal_path, GOAL).unwrap();
        init_from_goal(dir.path(), &goal_path, false).unwrap();
        let issue = plan_issue(dir.path(), "Project Seed Fact Source v0 实现").unwrap();
        write_local_project_seed(dir.path(), true).unwrap();
        let project_dir = dir.path().join(".agentflow");
        let issue_ids = vec![issue.issue_id.clone()];
        fs::write(
            project_dir.join("teams/core.json"),
            serde_json::to_string_pretty(&serde_json::json!({
                "version": VERSION,
                "id": "core",
                "name": "Core",
                "workflow": ["planned", "active", "completed"],
                "defaultValidationCommands": ["cargo test", "git diff --check"],
                "wipLimit": 1,
                "issueIds": issue_ids,
            }))
            .unwrap(),
        )
        .unwrap();
        fs::write(
            project_dir.join("projects/agentflow-local-execution.json"),
            serde_json::to_string_pretty(&serde_json::json!({
                "version": VERSION,
                "id": "agentflow-local-execution",
                "name": "AgentFlow MVP Productization",
                "status": "active",
                "goal": "Ship the local-first MVP through project, milestone, and issue planning.",
                "teamIds": ["core"],
                "activeMilestoneId": "mvp-project-foundation",
                "milestones": [
                    {
                        "id": "mvp-archive",
                        "name": "历史完成项归档",
                        "status": "completed",
                        "issueIds": [],
                        "completedIssueIds": [],
                        "nextIssueIntent": null
                    },
                    {
                        "id": "mvp-project-foundation",
                        "name": "项目与里程碑事实源",
                        "status": "active",
                        "issueIds": [issue.issue_id.clone()],
                        "completedIssueIds": [],
                        "nextIssueIntent": "Project Seed Fact Source v0 实现"
                    }
                ],
                "issueIds": [issue.issue_id.clone()],
                "nextIssueIntent": "Project Seed Fact Source v0 实现",
            }))
            .unwrap(),
        )
        .unwrap();

        let snapshot = read_local_project_model_snapshot(dir.path()).unwrap();

        assert_eq!(snapshot.teams[0].issue_ids, vec![issue.issue_id.clone()]);
        assert_eq!(snapshot.projects[0].name, "AgentFlow MVP Productization");
        assert_eq!(
            snapshot.projects[0].active_milestone_id,
            "mvp-project-foundation"
        );
        assert_eq!(snapshot.projects[0].issue_count, 1);
        assert_eq!(snapshot.projects[0].milestones.len(), 2);
        assert_eq!(
            snapshot.projects[0].milestones[1]
                .next_issue_intent
                .as_deref(),
            Some("Project Seed Fact Source v0 实现")
        );
        let preview = read_issue_project_link_preview(dir.path(), &issue.issue_id).unwrap();
        assert_eq!(preview.project_link.milestone_id, "mvp-project-foundation");
    }

    #[test]
    fn plan_issue_links_active_project_milestone_when_seed_exists() {
        let dir = tempdir().unwrap();
        let goal_path = dir.path().join("GOAL.md");
        fs::write(&goal_path, GOAL).unwrap();
        init_from_goal(dir.path(), &goal_path, false).unwrap();
        write_local_project_seed(dir.path(), true).unwrap();
        let project_dir = dir.path().join(".agentflow");
        fs::write(
            project_dir.join("projects/agentflow-local-execution.json"),
            serde_json::to_string_pretty(&serde_json::json!({
                "version": VERSION,
                "id": "agentflow-local-execution",
                "name": "AgentFlow MVP Productization",
                "status": "active",
                "goal": "Ship the local-first MVP through project, milestone, and issue planning.",
                "teamIds": ["core"],
                "activeMilestoneId": "mvp-issue-planning",
                "milestones": [
                    {
                        "id": "mvp-issue-planning",
                        "name": "基于 Milestone 创建任务",
                        "status": "active",
                        "issueIds": [],
                        "completedIssueIds": [],
                        "nextIssueIntent": "Milestone-aware Issue Planning v0 实现"
                    }
                ],
                "issueIds": [],
                "nextIssueIntent": "Milestone-aware Issue Planning v0 实现",
            }))
            .unwrap(),
        )
        .unwrap();

        let summary = plan_issue(dir.path(), "Milestone-aware Issue Planning v0 实现").unwrap();

        let link = summary.project_link.unwrap();
        assert_eq!(link.team_id, "core");
        assert_eq!(link.project_id, "agentflow-local-execution");
        assert_eq!(link.milestone_id, "mvp-issue-planning");
        assert_eq!(link.link_source, "milestone-aware-issue-planning-v0");
        assert_eq!(summary.updated_project_seed_paths.len(), 2);
        let issue_json: serde_json::Value = read_json(&summary.issue_json).unwrap();
        assert_eq!(
            issue_json["projectLink"]["linkSource"],
            "milestone-aware-issue-planning-v0"
        );
        let team: serde_json::Value = read_json(&project_dir.join("teams/core.json")).unwrap();
        assert_eq!(
            team["issueIds"][0].as_str(),
            Some(summary.issue_id.as_str())
        );
        let project: serde_json::Value =
            read_json(&project_dir.join("projects/agentflow-local-execution.json")).unwrap();
        assert_eq!(
            project["issueIds"][0].as_str(),
            Some(summary.issue_id.as_str())
        );
        assert_eq!(project["nextIssueIntent"], serde_json::Value::Null);
        assert_eq!(
            project["milestones"][0]["issueIds"][0].as_str(),
            Some(summary.issue_id.as_str())
        );
        assert_eq!(
            project["milestones"][0]["nextIssueIntent"],
            serde_json::Value::Null
        );
    }

    #[test]
    fn local_project_seed_write_refuses_overwrite() {
        let dir = tempdir().unwrap();
        let goal_path = dir.path().join("GOAL.md");
        fs::write(&goal_path, GOAL).unwrap();
        init_from_goal(dir.path(), &goal_path, false).unwrap();
        write_local_project_seed(dir.path(), true).unwrap();

        let result = write_local_project_seed(dir.path(), true);

        assert!(result.is_err());
        assert!(format!("{:?}", result.err().unwrap()).contains("refuses overwrite"));
    }

    #[test]
    fn team_project_milestone_issue_writers_preview_without_writes() {
        let dir = tempdir().unwrap();
        let goal_path = dir.path().join("GOAL.md");
        fs::write(&goal_path, GOAL).unwrap();
        init_from_goal(dir.path(), &goal_path, false).unwrap();
        write_local_project_seed(dir.path(), true).unwrap();
        let workspace_path = dir.path().join(".agentflow/workspace.json");
        let workspace_before = fs::read_to_string(&workspace_path).unwrap();

        let preview = create_team(
            dir.path(),
            TeamDraft {
                name: "Demo Team".to_string(),
                team_id: None,
            },
            false,
            false,
        )
        .unwrap();

        assert_eq!(preview.preview.mode, "preview");
        assert_eq!(preview.preview.kind, "team");
        assert_eq!(preview.preview.entity_id, "demo-team");
        assert!(preview.preview.boundary.read_only);
        assert_eq!(
            preview.preview.v1_contract.model,
            "project-milestone-issue-view-model-v1"
        );
        assert_eq!(
            preview.preview.v1_contract.team.as_ref().unwrap().team_id,
            "demo-team"
        );
        assert!(preview.written_paths.is_empty());
        assert!(!dir.path().join(".agentflow/teams/demo-team.json").exists());
        assert_eq!(
            fs::read_to_string(&workspace_path).unwrap(),
            workspace_before
        );

        let project_preview = create_project(
            dir.path(),
            ProjectDraft {
                title: "开发文档任务".to_string(),
                project_id: Some("dev-docs".to_string()),
                team_id: Some("core".to_string()),
                status: "draft".to_string(),
                goal: Some("完成开发文档 v1 合同对齐".to_string()),
            },
            false,
            false,
        )
        .unwrap();
        let project_contract = project_preview
            .preview
            .v1_contract
            .project_charter
            .as_ref()
            .unwrap();
        assert_eq!(project_contract.project_id, "dev-docs");
        assert_eq!(project_contract.status, ProjectStatus::Draft.as_str());
        assert!(project_contract
            .queue_rule
            .contains(&"Project 不执行，Issue 执行。".to_string()));
        assert!(project_preview.written_paths.is_empty());

        let milestone_preview = create_milestone(
            dir.path(),
            MilestoneDraft {
                title: "开发文档阶段".to_string(),
                milestone_id: Some("dev-docs-stage".to_string()),
                project_id: Some("agentflow-local-execution".to_string()),
                description: Some("完成开发文档阶段门".to_string()),
                target: None,
            },
            false,
            false,
        )
        .unwrap();
        let milestone_contract = milestone_preview
            .preview
            .v1_contract
            .milestone_gate
            .as_ref()
            .unwrap();
        assert_eq!(milestone_contract.milestone_id, "dev-docs-stage");
        assert!(milestone_contract
            .entry_criteria
            .contains(&"Milestone 只作为阶段分组，不写独立产品状态。".to_string()));
        assert!(milestone_preview.written_paths.is_empty());

        let issue_preview = create_issue(
            dir.path(),
            IssueDraft {
                title: "开发文档任务".to_string(),
                project_id: None,
                milestone_id: None,
                team_id: None,
                risk_level: "medium".to_string(),
                scope: Vec::new(),
                non_goals: Vec::new(),
                validation_commands: Vec::new(),
                evidence_requirements: Vec::new(),
                rollback_plan: Vec::new(),
            },
            false,
            false,
        )
        .unwrap();
        let issue_contract = issue_preview
            .preview
            .v1_contract
            .issue_contract
            .as_ref()
            .unwrap();
        assert_eq!(issue_contract.initial_state, IssueStatus::Todo.as_str());
        assert!(issue_contract
            .boundary
            .contains(&"Issue 是唯一执行原子。".to_string()));
        assert!(issue_preview.written_paths.is_empty());
        assert_eq!(
            fs::read_to_string(&workspace_path).unwrap(),
            workspace_before
        );
    }

    #[test]
    fn team_project_milestone_issue_writers_create_local_fact_chain() {
        let dir = tempdir().unwrap();
        let goal_path = dir.path().join("GOAL.md");
        fs::write(&goal_path, GOAL).unwrap();
        init_from_goal(dir.path(), &goal_path, false).unwrap();
        write_local_project_seed(dir.path(), true).unwrap();

        create_team(
            dir.path(),
            TeamDraft {
                name: "Demo Team".to_string(),
                team_id: None,
            },
            true,
            true,
        )
        .unwrap();
        create_project(
            dir.path(),
            ProjectDraft {
                title: "Demo Project".to_string(),
                project_id: None,
                team_id: Some("demo-team".to_string()),
                status: "draft".to_string(),
                goal: Some("Demo project goal".to_string()),
            },
            true,
            true,
        )
        .unwrap();
        create_milestone(
            dir.path(),
            MilestoneDraft {
                title: "Demo Milestone".to_string(),
                milestone_id: None,
                project_id: Some("demo-project".to_string()),
                description: Some("Demo milestone description".to_string()),
                target: Some("Demo milestone target".to_string()),
            },
            true,
            true,
        )
        .unwrap();
        create_issue(
            dir.path(),
            IssueDraft {
                title: "Demo Issue".to_string(),
                project_id: Some("demo-project".to_string()),
                milestone_id: Some("demo-milestone".to_string()),
                team_id: Some("demo-team".to_string()),
                risk_level: "medium".to_string(),
                scope: Vec::new(),
                non_goals: Vec::new(),
                validation_commands: Vec::new(),
                evidence_requirements: Vec::new(),
                rollback_plan: Vec::new(),
            },
            true,
            true,
        )
        .unwrap();

        let project_dir = dir.path().join(".agentflow");
        let workspace: serde_json::Value = read_json(&project_dir.join("workspace.json")).unwrap();
        let team: serde_json::Value = read_json(&project_dir.join("teams/demo-team.json")).unwrap();
        let project: serde_json::Value =
            read_json(&project_dir.join("projects/demo-project.json")).unwrap();
        let issue: IssueContract = read_json(&project_dir.join("issues/ISSUE-0001.json")).unwrap();
        let index: ProjectIndex = read_json(&project_dir.join("index.json")).unwrap();

        assert_eq!(workspace["activeProjectId"], "agentflow-local-execution");
        assert!(json_string_array_field(&workspace, "teamIds").contains(&"demo-team".to_string()));
        assert!(
            json_string_array_field(&workspace, "projectIds").contains(&"demo-project".to_string())
        );
        assert_eq!(team["status"], serde_json::Value::Null);
        assert!(json_string_array_field(&team, "projectIds").contains(&"demo-project".to_string()));
        assert!(json_string_array_field(&team, "issueIds").contains(&"ISSUE-0001".to_string()));
        assert_eq!(project["status"], "draft");
        assert_eq!(project["activeMilestoneId"], "demo-milestone");
        assert!(project["milestones"][0].get("status").is_none());
        assert_eq!(project["milestones"][0]["id"], "demo-milestone");
        assert!(
            json_string_array_field(&project["milestones"][0], "issueIds")
                .contains(&"ISSUE-0001".to_string())
        );
        assert_eq!(issue.status, IssueStatus::Todo.as_str());
        assert_eq!(issue.project_link.as_ref().unwrap().team_id, "demo-team");
        assert_eq!(
            issue.project_link.as_ref().unwrap().project_id,
            "demo-project"
        );
        assert_eq!(
            issue.project_link.as_ref().unwrap().milestone_id,
            "demo-milestone"
        );
        assert_eq!(index.next_issue_number, 2);
        assert_eq!(index.issues[0].status, IssueStatus::Todo.as_str());

        let snapshot = read_local_project_model_snapshot(dir.path()).unwrap();
        let demo_project = snapshot
            .projects
            .iter()
            .find(|project| project.id == "demo-project")
            .unwrap();
        assert_eq!(demo_project.canonical_status, ProjectStatus::Draft.as_str());
        assert_eq!(demo_project.milestones[0].progress.total_issue_count, 1);
        let demo_issue = snapshot
            .issue_refs
            .iter()
            .find(|issue| issue.id == "ISSUE-0001")
            .unwrap();
        assert_eq!(demo_issue.canonical_status, IssueStatus::Todo.as_str());
    }

    #[test]
    fn issue_project_link_preview_does_not_write_issue_files() {
        let dir = tempdir().unwrap();
        let goal_path = dir.path().join("GOAL.md");
        fs::write(&goal_path, GOAL).unwrap();
        init_from_goal(dir.path(), &goal_path, false).unwrap();
        let issue = plan_issue(dir.path(), "Issue Project Link Writer v0 实现").unwrap();
        let json_before = fs::read_to_string(&issue.issue_json).unwrap();
        let markdown_before = fs::read_to_string(&issue.issue_markdown).unwrap();

        let preview = read_issue_project_link_preview(dir.path(), &issue.issue_id).unwrap();

        assert!(preview.initialized);
        assert!(preview.boundary.read_only);
        assert_eq!(preview.action, "write");
        assert!(preview.writes_required);
        assert_eq!(preview.project_link.team_id, "core");
        assert_eq!(preview.project_link.project_id, "agentflow-local-execution");
        assert_eq!(preview.project_link.milestone_id, "current-roadmap");
        assert_eq!(
            preview.project_link.link_source,
            "issue-project-link-writer-v0"
        );
        assert!(preview
            .confirmation_gates
            .contains(&"explicit-yes-confirmation".to_string()));
        assert_eq!(fs::read_to_string(&issue.issue_json).unwrap(), json_before);
        assert_eq!(
            fs::read_to_string(&issue.issue_markdown).unwrap(),
            markdown_before
        );
        let issue_json: serde_json::Value = read_json(&issue.issue_json).unwrap();
        assert!(issue_json.get("projectLink").is_none());
        assert!(!markdown_before.contains("## Project Link"));
    }

    #[test]
    fn issue_project_link_write_requires_confirmation() {
        let dir = tempdir().unwrap();
        let goal_path = dir.path().join("GOAL.md");
        fs::write(&goal_path, GOAL).unwrap();
        init_from_goal(dir.path(), &goal_path, false).unwrap();
        let issue = plan_issue(dir.path(), "Issue Project Link Writer v0 实现").unwrap();

        let result = write_issue_project_link(dir.path(), &issue.issue_id, false);

        assert!(result.is_err());
        assert!(format!("{:?}", result.err().unwrap()).contains("explicit --yes"));
        let issue_json: serde_json::Value = read_json(&issue.issue_json).unwrap();
        assert!(issue_json.get("projectLink").is_none());
        assert!(!fs::read_to_string(&issue.issue_markdown)
            .unwrap()
            .contains("## Project Link"));
    }

    #[test]
    fn issue_project_link_write_updates_only_target_issue() {
        let dir = tempdir().unwrap();
        let goal_path = dir.path().join("GOAL.md");
        fs::write(&goal_path, GOAL).unwrap();
        init_from_goal(dir.path(), &goal_path, false).unwrap();
        let target = plan_issue(dir.path(), "Issue Project Link Writer v0 实现").unwrap();
        let untouched = plan_issue(dir.path(), "Project-aware GoalLoop v0 边界定义").unwrap();
        let untouched_json_before = fs::read_to_string(&untouched.issue_json).unwrap();
        let untouched_markdown_before = fs::read_to_string(&untouched.issue_markdown).unwrap();

        let summary = write_issue_project_link(dir.path(), &target.issue_id, true).unwrap();

        assert_eq!(summary.written_paths.len(), 2);
        assert!(summary
            .written_paths
            .iter()
            .any(|path| path.ends_with(".agentflow/issues/ISSUE-0001.json")));
        assert!(summary
            .written_paths
            .iter()
            .any(|path| path.ends_with(".agentflow/issues/ISSUE-0001.md")));
        let target_json: serde_json::Value = read_json(&target.issue_json).unwrap();
        assert_eq!(target_json["projectLink"]["teamId"], "core");
        assert_eq!(
            target_json["projectLink"]["projectId"],
            "agentflow-local-execution"
        );
        assert_eq!(target_json["projectLink"]["milestoneId"], "current-roadmap");
        assert_eq!(
            target_json["projectLink"]["linkSource"],
            "issue-project-link-writer-v0"
        );
        let target_markdown = fs::read_to_string(&target.issue_markdown).unwrap();
        assert!(target_markdown.contains("## Project Link"));
        assert!(target_markdown.contains("- Team: `core`"));
        assert_eq!(
            fs::read_to_string(&untouched.issue_json).unwrap(),
            untouched_json_before
        );
        assert_eq!(
            fs::read_to_string(&untouched.issue_markdown).unwrap(),
            untouched_markdown_before
        );
        let untouched_json: serde_json::Value = read_json(&untouched.issue_json).unwrap();
        assert!(untouched_json.get("projectLink").is_none());
    }

    #[test]
    fn issue_project_link_write_refuses_overwrite() {
        let dir = tempdir().unwrap();
        let goal_path = dir.path().join("GOAL.md");
        fs::write(&goal_path, GOAL).unwrap();
        init_from_goal(dir.path(), &goal_path, false).unwrap();
        let issue = plan_issue(dir.path(), "Issue Project Link Writer v0 实现").unwrap();
        write_issue_project_link(dir.path(), &issue.issue_id, true).unwrap();

        let preview = read_issue_project_link_preview(dir.path(), &issue.issue_id).unwrap();
        let result = write_issue_project_link(dir.path(), &issue.issue_id, true);

        assert_eq!(preview.action, "exists");
        assert!(!preview.writes_required);
        assert!(result.is_err());
        assert!(format!("{:?}", result.err().unwrap()).contains("refuses overwrite"));
    }

    #[test]
    fn local_search_reader_returns_traceable_results() {
        let dir = tempdir().unwrap();
        let goal_path = dir.path().join("GOAL.md");
        fs::write(&goal_path, GOAL).unwrap();
        init_from_goal(dir.path(), &goal_path, false).unwrap();
        let issue = plan_issue(dir.path(), "Local Search Reader v0 只读实现").unwrap();

        let snapshot = read_local_search_snapshot(dir.path(), "Local Search").unwrap();

        assert!(snapshot.initialized);
        assert!(snapshot.boundary.read_only);
        assert!(!snapshot.results.is_empty());
        assert!(snapshot
            .searched_paths
            .iter()
            .any(|path| path == ".agentflow/issues/ISSUE-0001.md"));
        let issue_result = snapshot
            .results
            .iter()
            .find(|result| {
                result.path == ".agentflow/issues/ISSUE-0001.md"
                    && result.snippet.contains("Local Search")
            })
            .expect("search result for issue markdown");
        assert_eq!(issue.issue_id, "ISSUE-0001");
        assert_eq!(issue_result.source_type, "file");
        assert_eq!(issue_result.entity_kind, "issue");
        assert_eq!(issue_result.entity_id.as_deref(), Some("ISSUE-0001"));
        assert!(issue_result.line > 0);
        assert!(!issue_result.field.is_empty());
        assert!(issue_result.score > 0);
        assert!(!dir.path().join(".agentflow/search").exists());
        assert!(!dir.path().join(".agentflow/queries").exists());
    }

    #[test]
    fn local_search_reader_excludes_unapproved_paths() {
        let dir = tempdir().unwrap();
        let goal_path = dir.path().join("GOAL.md");
        fs::write(&goal_path, GOAL).unwrap();
        init_from_goal(dir.path(), &goal_path, false).unwrap();
        fs::write(
            dir.path().join(".agentflow/evidence/manual-evidence.md"),
            "# Manual Evidence\n\nneedle visible\n",
        )
        .unwrap();
        fs::write(dir.path().join(".agentflow/index.sqlite"), "needle hidden").unwrap();
        fs::create_dir_all(dir.path().join(".agentflow/search")).unwrap();
        fs::write(
            dir.path().join(".agentflow/search/cache.md"),
            "needle hidden",
        )
        .unwrap();
        fs::create_dir_all(dir.path().join(".agentflow/queries")).unwrap();
        fs::write(
            dir.path().join(".agentflow/queries/saved.json"),
            "needle hidden",
        )
        .unwrap();

        let snapshot = read_local_search_snapshot(dir.path(), "needle").unwrap();

        assert_eq!(snapshot.results.len(), 1);
        assert_eq!(
            snapshot.results[0].path,
            ".agentflow/evidence/manual-evidence.md"
        );
        assert!(snapshot
            .results
            .iter()
            .all(|result| !result.path.starts_with(".agentflow/search/")));
        assert!(snapshot
            .results
            .iter()
            .all(|result| !result.path.starts_with(".agentflow/queries/")));
        assert!(snapshot
            .results
            .iter()
            .all(|result| !result.path.starts_with(".agentflow/index.sqlite")));
        assert!(snapshot
            .excluded_paths
            .contains(&".agentflow/queries/".to_string()));
    }

    #[test]
    fn collects_context_and_skips_build_outputs() {
        let dir = tempdir().unwrap();
        let goal_path = dir.path().join("GOAL.md");
        fs::write(&goal_path, GOAL).unwrap();
        fs::write(dir.path().join("Cargo.toml"), "[workspace]\n").unwrap();
        fs::create_dir_all(dir.path().join("src")).unwrap();
        fs::write(dir.path().join("src/lib.rs"), "pub fn ok() {}\n").unwrap();
        fs::create_dir_all(dir.path().join("target")).unwrap();
        fs::write(dir.path().join("target/generated.rs"), "ignored").unwrap();
        init_from_goal(dir.path(), &goal_path, false).unwrap();

        let summary = write_context(dir.path()).unwrap();
        let context: ProjectContext =
            read_json(&dir.path().join(".agentflow/context.json")).unwrap();

        assert!(summary.context_json.exists());
        assert!(context.detected_stacks.contains(&"rust".to_string()));
        assert!(context.files.iter().any(|file| file.path == "Cargo.toml"));
        assert!(context.files.iter().any(|file| file.path == "src/lib.rs"));
        assert!(!context
            .files
            .iter()
            .any(|file| file.path == "target/generated.rs"));
    }

    #[test]
    fn plan_uses_collected_context() {
        let dir = tempdir().unwrap();
        let goal_path = dir.path().join("GOAL.md");
        fs::write(&goal_path, GOAL).unwrap();
        fs::write(dir.path().join("Cargo.toml"), "[workspace]\n").unwrap();
        fs::create_dir_all(dir.path().join("crates/agentflow-core/src")).unwrap();
        fs::write(
            dir.path().join("crates/agentflow-core/src/lib.rs"),
            "pub fn planner() {}\n",
        )
        .unwrap();
        init_from_goal(dir.path(), &goal_path, false).unwrap();
        write_context(dir.path()).unwrap();

        let summary = plan_issue(dir.path(), "Implement planner").unwrap();
        let issue: IssueContract = read_json(&summary.issue_json).unwrap();

        assert!(issue
            .context
            .files
            .contains(&".agentflow/context.json".to_string()));
        assert!(issue
            .context
            .files
            .iter()
            .any(|path| path.contains("agentflow-core")));
    }

    #[test]
    fn run_dry_run_creates_artifacts_and_updates_index() {
        let dir = tempdir().unwrap();
        let goal_path = dir.path().join("GOAL.md");
        fs::write(&goal_path, GOAL).unwrap();
        init_from_goal(dir.path(), &goal_path, false).unwrap();
        let issue = plan_issue(dir.path(), "Implement runtime adapter").unwrap();

        let run = run_issue(dir.path(), &issue.issue_id).unwrap();
        let index: ProjectIndex = read_json(&dir.path().join(".agentflow/index.json")).unwrap();
        let stored_run: AgentRun = read_json(&run.run_json).unwrap();

        assert_eq!(run.run_id, "RUN-0001");
        assert_eq!(index.next_run_number, 2);
        assert_eq!(stored_run.issue_id, issue.issue_id);
        assert_eq!(stored_run.mode, "dry-run");
        assert!(run.run_dir.join("transcript.md").exists());
        assert!(run.run_dir.join("commands.jsonl").exists());
        assert!(run.run_dir.join("diff-summary.md").exists());
    }

    #[test]
    fn verify_runs_issue_validation_commands() {
        let dir = tempdir().unwrap();
        let goal_path = dir.path().join("GOAL.md");
        fs::write(&goal_path, GOAL).unwrap();
        init_from_goal(dir.path(), &goal_path, false).unwrap();
        let issue = plan_issue(dir.path(), "Implement validation runner").unwrap();
        replace_validation_commands(dir.path(), &issue.issue_id, vec!["printf ok".to_string()]);
        run_issue(dir.path(), &issue.issue_id).unwrap();

        let validation = verify_issue(dir.path(), &issue.issue_id).unwrap();
        let run_dir = dir.path().join(".agentflow/runs/RUN-0001");
        let commands = fs::read_to_string(run_dir.join("commands.jsonl")).unwrap();

        assert!(validation.passed);
        assert_eq!(validation.commands.len(), 1);
        assert_eq!(validation.commands[0].exit_code, 0);
        assert!(commands.contains("\"command\":\"printf ok\""));
    }

    #[test]
    fn review_generates_evidence_update_and_completes_issue() {
        let dir = tempdir().unwrap();
        let goal_path = dir.path().join("GOAL.md");
        fs::write(&goal_path, GOAL).unwrap();
        init_from_goal(dir.path(), &goal_path, false).unwrap();
        let issue = plan_issue(dir.path(), "Implement evidence chain").unwrap();
        replace_validation_commands(dir.path(), &issue.issue_id, vec!["printf ok".to_string()]);
        run_issue(dir.path(), &issue.issue_id).unwrap();
        verify_issue(dir.path(), &issue.issue_id).unwrap();

        let review = review_issue(dir.path(), &issue.issue_id).unwrap();
        let stored_issue: IssueContract = read_json(&issue.issue_json).unwrap();
        let index: ProjectIndex = read_json(&dir.path().join(".agentflow/index.json")).unwrap();
        let stored_run: AgentRun =
            read_json(&dir.path().join(".agentflow/runs/RUN-0001/run.json")).unwrap();

        assert!(review.passed);
        assert!(review.evidence_path.exists());
        assert!(review.review_path.exists());
        assert!(review.update_path.exists());
        assert_eq!(stored_issue.status, "done");
        assert_eq!(stored_run.status, "completed");
        assert_eq!(index.issues[0].status, "done");
    }

    #[test]
    fn review_completes_milestone_and_writes_summary_when_seed_linked() {
        let dir = tempdir().unwrap();
        let goal_path = dir.path().join("GOAL.md");
        fs::write(&goal_path, GOAL).unwrap();
        init_from_goal(dir.path(), &goal_path, false).unwrap();
        write_local_project_seed(dir.path(), true).unwrap();
        let project_dir = dir.path().join(".agentflow");
        fs::write(
            project_dir.join("projects/agentflow-local-execution.json"),
            serde_json::to_string_pretty(&serde_json::json!({
                "version": VERSION,
                "id": "agentflow-local-execution",
                "name": "AgentFlow MVP Productization",
                "status": "active",
                "goal": "Ship the local-first MVP through project, milestone, and issue planning.",
                "teamIds": ["core"],
                "activeMilestoneId": "mvp-summary",
                "milestones": [
                    {
                        "id": "mvp-summary",
                        "name": "Milestone summary gate",
                        "status": "active",
                        "issueIds": [],
                        "completedIssueIds": [],
                        "nextIssueIntent": "Milestone summary issue"
                    },
                    {
                        "id": "mvp-next",
                        "name": "Next milestone",
                        "status": "planned",
                        "issueIds": [],
                        "completedIssueIds": [],
                        "nextIssueIntent": "Next milestone issue"
                    }
                ],
                "issueIds": [],
                "nextIssueIntent": null,
            }))
            .unwrap(),
        )
        .unwrap();
        let issue = plan_issue(dir.path(), "Milestone summary issue").unwrap();
        replace_validation_commands(dir.path(), &issue.issue_id, vec!["printf ok".to_string()]);
        run_issue(dir.path(), &issue.issue_id).unwrap();
        verify_issue(dir.path(), &issue.issue_id).unwrap();

        review_issue(dir.path(), &issue.issue_id).unwrap();

        let summary_path = project_dir.join("evidence/MILESTONE-mvp-summary-evidence-summary.md");
        let summary = fs::read_to_string(&summary_path).unwrap();
        let project: serde_json::Value =
            read_json(&project_dir.join("projects/agentflow-local-execution.json")).unwrap();
        assert!(summary.contains("# Milestone Evidence Summary"));
        assert!(summary.contains("只推进当前 milestone 中唯一 eligible issue"));
        assert!(summary.contains(&issue.issue_id));
        assert_eq!(project["activeMilestoneId"], "mvp-next");
        assert_eq!(project["milestones"][0]["status"], "completed");
        assert_eq!(
            project["milestones"][0]["completedIssueIds"][0].as_str(),
            Some(issue.issue_id.as_str())
        );
        assert_eq!(project["milestones"][1]["status"], "active");
    }

    #[test]
    fn rebuild_index_records_issues_runs_and_updates() {
        let dir = tempdir().unwrap();
        let goal_path = dir.path().join("GOAL.md");
        fs::write(&goal_path, GOAL).unwrap();
        init_from_goal(dir.path(), &goal_path, false).unwrap();
        let issue = plan_issue(dir.path(), "Implement sqlite index").unwrap();
        replace_validation_commands(dir.path(), &issue.issue_id, vec!["printf ok".to_string()]);
        run_issue(dir.path(), &issue.issue_id).unwrap();
        verify_issue(dir.path(), &issue.issue_id).unwrap();
        review_issue(dir.path(), &issue.issue_id).unwrap();

        let summary = rebuild_index(dir.path()).unwrap();

        assert!(summary.sqlite_path.exists());
        assert_eq!(summary.issue_count, 1);
        assert_eq!(summary.run_count, 1);
        assert_eq!(summary.update_count, 1);
        assert_eq!(summary.saved_view_count, 0);
    }

    #[test]
    fn saved_view_filters_completed_issues_and_passed_runs() {
        let dir = tempdir().unwrap();
        let goal_path = dir.path().join("GOAL.md");
        fs::write(&goal_path, GOAL).unwrap();
        init_from_goal(dir.path(), &goal_path, false).unwrap();
        let issue = plan_issue(dir.path(), "Implement saved views").unwrap();
        replace_validation_commands(dir.path(), &issue.issue_id, vec!["printf ok".to_string()]);
        run_issue(dir.path(), &issue.issue_id).unwrap();
        verify_issue(dir.path(), &issue.issue_id).unwrap();
        review_issue(dir.path(), &issue.issue_id).unwrap();

        let saved = save_view(
            dir.path(),
            "completed",
            SavedViewFilter {
                issue_status: Some("completed".to_string()),
                run_status: Some("completed".to_string()),
                validation_status: Some("passed".to_string()),
                issue_id: None,
            },
        )
        .unwrap();
        let result = show_view(dir.path(), "completed").unwrap();

        assert_eq!(saved.view_id, "completed");
        assert_eq!(result.issues.len(), 1);
        assert_eq!(result.runs.len(), 1);
        assert_eq!(result.issues[0].status, "done");
        assert_eq!(result.runs[0].validation_status, "passed");
    }

    #[test]
    fn project_summary_counts_current_facts() {
        let dir = tempdir().unwrap();
        let goal_path = dir.path().join("GOAL.md");
        fs::write(&goal_path, GOAL).unwrap();
        init_from_goal(dir.path(), &goal_path, false).unwrap();
        let issue = plan_issue(dir.path(), "Implement project summary").unwrap();
        replace_validation_commands(dir.path(), &issue.issue_id, vec!["printf ok".to_string()]);
        run_issue(dir.path(), &issue.issue_id).unwrap();
        verify_issue(dir.path(), &issue.issue_id).unwrap();
        review_issue(dir.path(), &issue.issue_id).unwrap();
        save_view(
            dir.path(),
            "completed",
            SavedViewFilter {
                issue_status: Some("completed".to_string()),
                run_status: Some("completed".to_string()),
                validation_status: Some("passed".to_string()),
                issue_id: None,
            },
        )
        .unwrap();

        let summary = write_project_summary(dir.path()).unwrap();
        let content = fs::read_to_string(&summary.summary_path).unwrap();

        assert_eq!(summary.issue_count, 1);
        assert_eq!(summary.completed_issue_count, 1);
        assert_eq!(summary.run_count, 1);
        assert_eq!(summary.update_count, 1);
        assert_eq!(summary.saved_view_count, 1);
        assert!(content.contains("Project Summary"));
        assert!(content.contains("ISSUE-0001"));
    }

    #[test]
    fn review_assistant_reports_ready_after_review_chain() {
        let dir = tempdir().unwrap();
        let goal_path = dir.path().join("GOAL.md");
        fs::write(&goal_path, GOAL).unwrap();
        init_from_goal(dir.path(), &goal_path, false).unwrap();
        let issue = plan_issue(dir.path(), "Implement review assistant").unwrap();
        replace_validation_commands(dir.path(), &issue.issue_id, vec!["printf ok".to_string()]);
        run_issue(dir.path(), &issue.issue_id).unwrap();
        verify_issue(dir.path(), &issue.issue_id).unwrap();
        review_issue(dir.path(), &issue.issue_id).unwrap();
        write_goal_next(dir.path()).unwrap();

        let assistant = write_review_assistant(dir.path(), &issue.issue_id).unwrap();
        let content = fs::read_to_string(&assistant.assistant_path).unwrap();

        assert!(assistant.ready);
        assert_eq!(assistant.checks.len(), 15);
        assert!(content.contains("Decision: `ready`"));
        assert!(content.contains("Validation results"));
        assert!(content.contains("AEP protocol fields"));
        assert!(content.contains("Goal Loop readiness"));
    }

    #[test]
    fn workflow_state_check_writes_ready_snapshot_for_active_milestone_issue() {
        let dir = tempdir().unwrap();
        let goal_path = dir.path().join("GOAL.md");
        fs::write(&goal_path, GOAL).unwrap();
        init_from_goal(dir.path(), &goal_path, false).unwrap();
        write_contract_fixture(dir.path());
        write_local_project_seed(dir.path(), true).unwrap();
        plan_issue(dir.path(), "Workflow State Machine v0 边界定义").unwrap();

        let summary = write_workflow_state_check(dir.path()).unwrap();
        let content = fs::read_to_string(&summary.summary_path).unwrap();

        assert!(summary.snapshot.ready);
        assert!(summary.snapshot_path.exists());
        assert!(content.contains("Workflow State Summary"));
        assert!(content.contains("Transition Guards"));
        assert_eq!(summary.snapshot.counts.projects, 1);
        assert_eq!(summary.snapshot.counts.issues, 1);
    }

    #[test]
    fn workflow_state_check_blocks_open_issue_in_done_milestone() {
        let dir = tempdir().unwrap();
        let goal_path = dir.path().join("GOAL.md");
        fs::write(&goal_path, GOAL).unwrap();
        init_from_goal(dir.path(), &goal_path, false).unwrap();
        write_contract_fixture(dir.path());
        write_local_project_seed(dir.path(), true).unwrap();
        let issue = plan_issue(dir.path(), "Workflow State Machine v0 边界定义").unwrap();
        let project_path = dir
            .path()
            .join(".agentflow/projects/agentflow-local-execution.json");
        let mut project: serde_json::Value = read_json(&project_path).unwrap();
        if let Some(milestone) = project["milestones"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|milestone| {
                milestone["issueIds"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|id| id.as_str() == Some(issue.issue_id.as_str()))
            })
        {
            milestone["status"] = serde_json::json!("completed");
        }
        write_json(&project_path, &project, true).unwrap();

        let summary = write_workflow_state_check(dir.path()).unwrap();

        assert!(!summary.snapshot.ready);
        assert!(summary.snapshot.checks.iter().any(|check| {
            check.status == "fail"
                && (check.id == "issue-open-not-in-done-milestone"
                    || check.id == "milestone-done-all-issues-done")
        }));
    }

    #[test]
    fn eligibility_finds_unique_active_milestone_issue() {
        let dir = tempdir().unwrap();
        let goal_path = dir.path().join("GOAL.md");
        fs::write(&goal_path, GOAL).unwrap();
        init_from_goal(dir.path(), &goal_path, false).unwrap();
        write_workflow_control_seed(dir.path(), "eligibility-core");
        let issue = plan_issue(dir.path(), "Eligibility Engine v0 边界定义").unwrap();

        let summary = write_workflow_eligibility(dir.path(), None).unwrap();

        assert_eq!(
            summary.snapshot.eligible_issue_id.as_deref(),
            Some(issue.issue_id.as_str())
        );
        assert_eq!(summary.snapshot.summary.eligible_issue_count, 1);
        assert_eq!(
            summary.snapshot.summary.recommended_command,
            format!("agentflow run {} --dry-run", issue.issue_id)
        );
        assert!(summary.summary_path.exists());
    }

    #[test]
    fn eligibility_reports_failure_reasons_when_issue_is_not_ready() {
        let dir = tempdir().unwrap();
        let goal_path = dir.path().join("GOAL.md");
        fs::write(&goal_path, GOAL).unwrap();
        init_from_goal(dir.path(), &goal_path, false).unwrap();
        write_workflow_control_seed(dir.path(), "eligibility-core");
        let issue = plan_issue(dir.path(), "Eligibility missing validation").unwrap();
        replace_validation_commands(dir.path(), &issue.issue_id, Vec::new());

        let summary = write_workflow_eligibility(dir.path(), None).unwrap();

        assert!(summary.snapshot.eligible_issue_id.is_none());
        assert_eq!(summary.snapshot.summary.next_action, "wait-human");
        assert!(summary.snapshot.candidates[0]
            .failure_reasons
            .contains(&"missing_validation_commands".to_string()));
    }

    #[test]
    fn run_acquires_lease_and_review_releases_it() {
        let dir = tempdir().unwrap();
        let goal_path = dir.path().join("GOAL.md");
        fs::write(&goal_path, GOAL).unwrap();
        init_from_goal(dir.path(), &goal_path, false).unwrap();
        write_workflow_control_seed(dir.path(), "lease-core");
        let issue = plan_issue(dir.path(), "Lease Lock v0 实现").unwrap();
        replace_validation_commands(dir.path(), &issue.issue_id, vec!["printf ok".to_string()]);

        let run_summary = run_issue(dir.path(), &issue.issue_id).unwrap();
        let run: AgentRun = read_json(&run_summary.run_json).unwrap();
        let lease_id = run.lease_id.clone().unwrap();
        let active_snapshot = workflow_lease_snapshot(dir.path()).unwrap();
        assert_eq!(active_snapshot.active_leases.len(), 1);
        assert_eq!(active_snapshot.active_leases[0].id, lease_id);

        verify_issue(dir.path(), &issue.issue_id).unwrap();
        review_issue(dir.path(), &issue.issue_id).unwrap();
        let released: WorkflowLeaseRecord = read_json(
            &dir.path()
                .join(".agentflow/leases")
                .join(format!("{lease_id}.json")),
        )
        .unwrap();

        assert_eq!(released.status, "released");
        assert!(released.released_at_epoch_seconds.is_some());
    }

    #[test]
    fn project_closure_state_reports_audit_ready_without_marking_done() {
        let dir = tempdir().unwrap();
        let goal_path = dir.path().join("GOAL.md");
        fs::write(&goal_path, GOAL).unwrap();
        init_from_goal(dir.path(), &goal_path, false).unwrap();
        write_project_closure_seed(dir.path());

        let summary = write_project_closure_state(dir.path()).unwrap();

        assert_eq!(summary.snapshot.closure_state, "audit-ready");
        assert!(!summary.snapshot.can_mark_done);
        assert!(summary
            .snapshot
            .done_blocked_reasons
            .iter()
            .any(|reason| reason.starts_with("code-audit")));
        assert!(summary.snapshot_path.exists());
        assert!(summary.summary_path.exists());
        assert!(!dir.path().join(".agentflow/audits").exists());
    }

    #[test]
    fn project_code_audit_snapshot_writes_read_only_state_without_audits_dir() {
        let dir = tempdir().unwrap();
        let goal_path = dir.path().join("GOAL.md");
        fs::write(&goal_path, GOAL).unwrap();
        init_from_goal(dir.path(), &goal_path, false).unwrap();
        write_project_closure_seed(dir.path());
        write_project_closure_state(dir.path()).unwrap();
        fs::create_dir_all(dir.path().join("src")).unwrap();
        fs::write(
            dir.path().join("src/lib.rs"),
            "pub fn visible_api() {}\n// TODO: remove temporary workaround before closure\n",
        )
        .unwrap();

        let summary = write_project_code_audit_snapshot(dir.path()).unwrap();

        assert_eq!(summary.snapshot.audit_state, "snapshot-ready");
        assert!(summary.snapshot_path.exists());
        assert!(summary.summary_path.exists());
        assert!(!dir.path().join(".agentflow/audits").exists());
        assert!(summary
            .snapshot
            .checks
            .iter()
            .any(|check| check.id == "todo-fixme-candidate" && check.candidate_count > 0));

        let closure = write_project_closure_state(dir.path()).unwrap();
        let code_audit_gate = closure
            .snapshot
            .gates
            .iter()
            .find(|gate| gate.id == "code-audit")
            .unwrap();
        assert_eq!(closure.snapshot.closure_state, "audit");
        assert_eq!(code_audit_gate.status, "snapshot-ready");
        assert!(!closure.snapshot.can_mark_done);
    }

    #[test]
    fn project_docs_refresh_snapshot_writes_read_only_state_without_audits_dir() {
        let dir = tempdir().unwrap();
        let goal_path = dir.path().join("GOAL.md");
        fs::write(&goal_path, GOAL).unwrap();
        init_from_goal(dir.path(), &goal_path, false).unwrap();
        write_project_closure_seed(dir.path());
        write_contract_fixture(dir.path());
        fs::write(
            dir.path().join("README.md"),
            "Root Docs Refresh Snapshot v0\nProject Code Audit Snapshot v0\n",
        )
        .unwrap();
        write_project_closure_state(dir.path()).unwrap();
        write_project_code_audit_snapshot(dir.path()).unwrap();

        let summary = write_project_docs_refresh_snapshot(dir.path()).unwrap();

        assert_eq!(summary.snapshot.docs_refresh_state, "snapshot-ready");
        assert!(summary.snapshot_path.exists());
        assert!(summary.summary_path.exists());
        assert!(!dir.path().join(".agentflow/audits").exists());
        assert!(summary
            .snapshot
            .checked_docs
            .iter()
            .any(|doc| doc.path == "README.md" && doc.status == "current"));

        let closure = write_project_closure_state(dir.path()).unwrap();
        let docs_refresh_gate = closure
            .snapshot
            .gates
            .iter()
            .find(|gate| gate.id == "docs-refresh")
            .unwrap();
        assert_eq!(docs_refresh_gate.status, "snapshot-ready");
        assert!(!closure.snapshot.can_mark_done);
    }

    #[test]
    fn goal_next_recommends_project_code_audit_when_snapshot_is_missing() {
        let dir = tempdir().unwrap();
        let goal_path = dir.path().join("GOAL.md");
        fs::write(&goal_path, GOAL).unwrap();
        init_from_goal(dir.path(), &goal_path, false).unwrap();
        write_contract_fixture(dir.path());
        write_project_closure_seed(dir.path());

        let summary = write_goal_next(dir.path()).unwrap();

        assert_eq!(summary.next_action, "project-code-audit");
        assert_eq!(summary.recommended_command, "agentflow project code-audit");
        assert!(summary.summary_path.exists());
    }

    #[test]
    fn goal_next_recommends_docs_refresh_when_code_audit_snapshot_exists() {
        let dir = tempdir().unwrap();
        let goal_path = dir.path().join("GOAL.md");
        fs::write(&goal_path, GOAL).unwrap();
        init_from_goal(dir.path(), &goal_path, false).unwrap();
        write_contract_fixture(dir.path());
        write_project_closure_seed(dir.path());
        write_project_closure_state(dir.path()).unwrap();
        write_project_code_audit_snapshot(dir.path()).unwrap();

        let summary = write_goal_next(dir.path()).unwrap();

        assert_eq!(summary.next_action, "project-docs-refresh");
        assert_eq!(
            summary.recommended_command,
            "agentflow project docs-refresh"
        );
        assert!(summary.summary_path.exists());
    }

    #[test]
    fn goal_next_returns_to_project_candidate_after_closure_snapshots_exist() {
        let dir = tempdir().unwrap();
        let goal_path = dir.path().join("GOAL.md");
        fs::write(&goal_path, GOAL).unwrap();
        init_from_goal(dir.path(), &goal_path, false).unwrap();
        write_contract_fixture(dir.path());
        write_project_closure_seed(dir.path());
        set_active_milestone_intent(dir.path(), "Product Feature Creation Flow v0");
        write_project_closure_state(dir.path()).unwrap();
        write_project_code_audit_snapshot(dir.path()).unwrap();
        write_project_docs_refresh_snapshot(dir.path()).unwrap();

        let summary = write_goal_next(dir.path()).unwrap();

        assert_eq!(summary.next_action, "plan");
        assert_eq!(
            summary.recommended_issue_intent,
            "Product Feature Creation Flow v0"
        );
        assert_eq!(
            summary.recommended_command,
            "agentflow plan \"Product Feature Creation Flow v0\""
        );
    }

    fn replace_validation_commands(repo: &Path, issue_id: &str, commands: Vec<String>) {
        let project_dir = repo.join(".agentflow");
        let issue_path = project_dir.join(format!("issues/{issue_id}.json"));
        let mut issue: IssueContract = read_json(&issue_path).unwrap();
        issue.validation.commands = commands;
        write_issue(&project_dir, &issue).unwrap();
    }

    fn product_feature_test_draft(goal: &str) -> ProductFeatureDraft {
        ProductFeatureDraft {
            feature_goal: goal.to_string(),
            team_id: "core".to_string(),
            project_title: goal.to_string(),
            non_goals: vec!["不创建远程对象。".to_string()],
            success_criteria: vec!["第一条 issue 可以进入 eligibility。".to_string()],
            risk_level: "medium".to_string(),
            scope_boundaries: vec!["只写本地 .agentflow 事实源。".to_string()],
        }
    }

    fn write_workflow_control_seed(repo: &Path, milestone_id: &str) {
        write_local_project_seed(repo, true).unwrap();
        let project_dir = repo.join(".agentflow");
        fs::write(
            project_dir.join("projects/agentflow-local-execution.json"),
            serde_json::to_string_pretty(&serde_json::json!({
                "version": VERSION,
                "id": "agentflow-local-execution",
                "name": "AgentFlow Workflow Control",
                "status": "active",
                "goal": "Build workflow control core.",
                "teamIds": ["core"],
                "activeMilestoneId": milestone_id,
                "milestones": [
                    {
                        "id": milestone_id,
                        "name": "Workflow Control",
                        "status": "active",
                        "issueIds": [],
                        "completedIssueIds": [],
                        "nextIssueIntent": "Workflow Control Core v0"
                    }
                ],
                "issueIds": [],
                "nextIssueIntent": null,
            }))
            .unwrap(),
        )
        .unwrap();
    }

    fn write_project_closure_seed(repo: &Path) {
        write_local_project_seed(repo, true).unwrap();
        let project_dir = repo.join(".agentflow");
        fs::write(
            project_dir.join("projects/agentflow-local-execution.json"),
            serde_json::to_string_pretty(&serde_json::json!({
                "version": VERSION,
                "id": "agentflow-local-execution",
                "name": "AgentFlow Closure",
                "status": "active",
                "goal": "Close the local AgentFlow project through audit and docs gates.",
                "teamIds": ["core"],
                "activeMilestoneId": "workflow-core-closure-gates",
                "milestones": [
                    {
                        "id": "delivery-core",
                        "name": "Delivery Core",
                        "status": "completed",
                        "issueIds": [],
                        "completedIssueIds": [],
                        "nextIssueIntent": null
                    },
                    {
                        "id": "workflow-core-closure-gates",
                        "name": "Workflow Core Closure Gates",
                        "status": "active",
                        "issueIds": [],
                        "completedIssueIds": [],
                        "nextIssueIntent": null
                    }
                ],
                "issueIds": [],
                "nextIssueIntent": null,
            }))
            .unwrap(),
        )
        .unwrap();
    }

    fn set_active_milestone_intent(repo: &Path, next_issue_intent: &str) {
        let project_path = repo
            .join(".agentflow/projects")
            .join("agentflow-local-execution.json");
        let mut project: serde_json::Value = read_json(&project_path).unwrap();
        project["milestones"][1]["nextIssueIntent"] =
            serde_json::Value::String(next_issue_intent.to_string());
        fs::write(
            &project_path,
            serde_json::to_string_pretty(&project).unwrap(),
        )
        .unwrap();
    }

    fn write_contract_fixture(repo: &Path) {
        let contract_dir = repo.join("docs/contracts");
        fs::create_dir_all(&contract_dir).unwrap();
        fs::write(
            contract_dir.join("agentflow-ai-delivery-workflow-contract-v1.md"),
            "# AgentFlow AI Delivery Workflow Contract v1\n",
        )
        .unwrap();
    }
}
