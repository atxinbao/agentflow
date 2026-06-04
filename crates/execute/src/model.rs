use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const EXECUTE_MANIFEST_VERSION: &str = "execute-manifest.v1";
pub const EXECUTE_INDEX_VERSION: &str = "execute-index.v1";
pub const EXECUTE_STATUS_VERSION: &str = "execute-status.v1";
pub const EXECUTE_SNAPSHOT_VERSION: &str = "execute-snapshot.v1";
pub const EXECUTE_RUN_VERSION: &str = "execute-run.v1";
pub const EXECUTE_PREFLIGHT_VERSION: &str = "execute-preflight.v1";
pub const EXECUTE_PLAN_VERSION: &str = "execute-plan.v1";
pub const EXECUTE_LEASE_VERSION: &str = "execute-lease.v1";
pub const EXECUTE_CHECKPOINT_VERSION: &str = "execute-checkpoint.v1";
pub const EXECUTE_COMMAND_VERSION: &str = "execute-command.v1";
pub const EXECUTE_RESULT_VERSION: &str = "execute-result.v1";
pub const OUTPUT_EVIDENCE_VERSION: &str = "output-evidence.v1";
pub const OUTPUT_RELEASE_DELIVERY_VERSION: &str = "output-release-delivery.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExecuteWorkspaceStatus {
    Missing,
    Ready,
    Degraded,
    Failed,
    Blocked,
}

impl Default for ExecuteWorkspaceStatus {
    fn default() -> Self {
        Self::Missing
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExecuteRunStatus {
    Queued,
    Preflight,
    Blocked,
    Planned,
    Checkpointed,
    Patching,
    Running,
    Validating,
    Completed,
    Failed,
    Cancelled,
}

impl Default for ExecuteRunStatus {
    fn default() -> Self {
        Self::Queued
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExecuteCheckStatus {
    Passed,
    Blocked,
    Warning,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExecuteLeaseStatus {
    Active,
    Released,
}

impl Default for ExecuteLeaseStatus {
    fn default() -> Self {
        Self::Active
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteSummary {
    pub runs: usize,
    pub active_runs: usize,
    pub blocked_runs: usize,
    pub completed_runs: usize,
    pub active_leases: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteManifest {
    pub version: String,
    pub project_root: String,
    pub status: ExecuteWorkspaceStatus,
    pub paths: BTreeMap<String, String>,
    pub summary: ExecuteSummary,
}

impl ExecuteManifest {
    pub fn new(project_root: impl Into<String>, summary: ExecuteSummary) -> Self {
        Self {
            version: EXECUTE_MANIFEST_VERSION.to_string(),
            project_root: project_root.into(),
            status: ExecuteWorkspaceStatus::Ready,
            paths: execute_paths(),
            summary,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteRunIndexEntry {
    pub run_id: String,
    pub issue_id: String,
    pub source_spec_id: String,
    pub status: ExecuteRunStatus,
    pub path: String,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteLeaseIndexEntry {
    pub issue_id: String,
    pub run_id: String,
    pub status: ExecuteLeaseStatus,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteIndex {
    pub version: String,
    pub updated_at: u64,
    pub runs: Vec<ExecuteRunIndexEntry>,
    pub leases: Vec<ExecuteLeaseIndexEntry>,
}

impl Default for ExecuteIndex {
    fn default() -> Self {
        Self {
            version: EXECUTE_INDEX_VERSION.to_string(),
            updated_at: 0,
            runs: Vec::new(),
            leases: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteStatusSnapshot {
    pub version: String,
    pub project_root: String,
    pub status: ExecuteWorkspaceStatus,
    pub ready: bool,
    pub manifest_exists: bool,
    pub index_exists: bool,
    pub summary: ExecuteSummary,
    pub missing_paths: Vec<String>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteSnapshot {
    pub version: String,
    pub project_root: String,
    pub ready: bool,
    pub status: ExecuteStatusSnapshot,
    pub manifest: ExecuteManifest,
    pub index: ExecuteIndex,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteRunInput {
    pub issue_path: String,
    pub spec_path: String,
    pub panel_snapshot_id: Option<String>,
    pub context_pack_id: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteRunPaths {
    pub preflight: String,
    pub plan: String,
    pub result: String,
    pub evidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteRun {
    pub version: String,
    pub run_id: String,
    pub issue_id: String,
    pub source_spec_id: String,
    pub project_id: Option<String>,
    pub risk_level: String,
    pub status: ExecuteRunStatus,
    pub agent_role: String,
    pub created_by: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub input: ExecuteRunInput,
    pub paths: ExecuteRunPaths,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutePreflightCheck {
    pub name: String,
    pub status: ExecuteCheckStatus,
    pub message: Option<String>,
    pub risk_level: Option<String>,
    pub human_confirmation_required: Option<bool>,
    pub confirmed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutePreflight {
    pub version: String,
    pub run_id: String,
    pub issue_id: String,
    pub status: String,
    pub checks: Vec<ExecutePreflightCheck>,
    pub blocked_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteHumanConfirmation {
    pub version: String,
    pub run_id: String,
    pub issue_id: String,
    pub risk_level: String,
    pub confirmed_by: String,
    pub confirmed_at: u64,
    pub confirmation_text: String,
    pub scope: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteLease {
    pub version: String,
    pub issue_id: String,
    pub run_id: String,
    pub status: ExecuteLeaseStatus,
    pub created_at: u64,
    pub released_at: Option<u64>,
    pub expires_at: Option<u64>,
    pub locked_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExecutePlanStepKind {
    Edit,
    Validate,
    Review,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutePlanStep {
    pub step_id: String,
    pub kind: ExecutePlanStepKind,
    pub target: Option<String>,
    pub command: Option<String>,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutePlan {
    pub version: String,
    pub run_id: String,
    pub issue_id: String,
    pub steps: Vec<ExecutePlanStep>,
    pub allowed_write_paths: Vec<String>,
    pub allowed_commands: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutePlanDraft {
    pub steps: Vec<ExecutePlanStep>,
    pub allowed_write_paths: Vec<String>,
    pub allowed_commands: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteFileHash {
    pub path: String,
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteCheckpoint {
    pub version: String,
    pub checkpoint_id: String,
    pub run_id: String,
    pub created_at: u64,
    pub git_head: Option<String>,
    pub dirty_files_before: Vec<String>,
    pub panel_snapshot_id: Option<String>,
    pub file_hashes_before: Vec<ExecuteFileHash>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteChangedFile {
    pub path: String,
    pub change_type: String,
    pub insertions: usize,
    pub deletions: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteChangedFiles {
    pub version: String,
    pub run_id: String,
    pub files: Vec<ExecuteChangedFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutePatchOutcome {
    pub run_id: String,
    pub changed_files: ExecuteChangedFiles,
    pub proposed_patch_path: String,
    pub applied_patch_path: String,
    pub worktree_diff_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteCommandRequest {
    pub label: String,
    pub program: String,
    pub args: Vec<String>,
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteCommandRecord {
    pub version: String,
    pub command_id: String,
    pub run_id: String,
    pub label: String,
    pub program: String,
    pub args: Vec<String>,
    pub cwd: String,
    pub source: String,
    pub started_at: u64,
    pub finished_at: u64,
    pub exit_code: Option<i32>,
    pub stdout_path: String,
    pub stderr_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteValidationResult {
    pub passed: bool,
    pub evidence_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteResultNext {
    pub ready_for_delivery: bool,
    pub needs_audit: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteResult {
    pub version: String,
    pub run_id: String,
    pub issue_id: String,
    pub status: ExecuteRunStatus,
    pub risk_level: String,
    pub changed_files: Vec<String>,
    pub commands: Vec<String>,
    pub validation: ExecuteValidationResult,
    pub next: ExecuteResultNext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteDiffSummary {
    pub version: String,
    pub run_id: String,
    pub changed_files: usize,
    pub insertions: usize,
    pub deletions: usize,
    pub risk_level: String,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteReviewState {
    pub version: String,
    pub run_id: String,
    pub status: String,
    pub hunk_review_enabled: bool,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputEvidence {
    pub version: String,
    pub run_id: String,
    pub issue_id: String,
    pub source_spec_id: String,
    pub risk_level: String,
    pub completed_at: u64,
    pub summary: String,
    pub changed_files: Vec<String>,
    pub commands: Vec<String>,
    pub validation_passed: bool,
    pub artifacts: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputReleaseDeliveryArtifacts {
    pub pr_draft: String,
    pub pr_metadata: String,
    pub review_checklist: String,
    pub changelog: String,
    pub release_note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputReleaseDelivery {
    pub version: String,
    pub run_id: String,
    pub issue_id: String,
    pub source_spec_id: String,
    pub risk_level: String,
    pub evidence_path: String,
    pub status: String,
    pub artifacts: OutputReleaseDeliveryArtifacts,
    pub created_by: String,
    pub created_at: u64,
}

pub fn execute_paths() -> BTreeMap<String, String> {
    BTreeMap::from([
        ("root".to_string(), ".agentflow/execute".to_string()),
        ("runs".to_string(), ".agentflow/execute/runs".to_string()),
        (
            "leases".to_string(),
            ".agentflow/execute/leases".to_string(),
        ),
        ("queue".to_string(), ".agentflow/execute/queue".to_string()),
        (
            "evidence".to_string(),
            ".agentflow/output/evidence".to_string(),
        ),
        (
            "release".to_string(),
            ".agentflow/output/release".to_string(),
        ),
    ])
}
