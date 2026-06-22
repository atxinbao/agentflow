use agentflow_workflow_core::WorkflowFlowType;
use serde::{Deserialize, Serialize};

pub const TASK_RUN_VERSION: &str = "task-run.v1";
pub const TASK_COMMAND_VERSION: &str = "task-command.v1";
pub const TASK_CHANGED_FILES_VERSION: &str = "task-changed-files.v1";
pub const TASK_VALIDATION_VERSION: &str = "task-validation.v1";
pub const TASK_EVIDENCE_VERSION: &str = "task-evidence.v1";
pub const TASK_ACCEPTANCE_GATE_VERSION: &str = "task-acceptance-gate.v1";
pub const TASK_PREFLIGHT_VERSION: &str = "task-preflight.v1";
pub const TASK_RUN_CHECKPOINT_VERSION: &str = "task-run-checkpoint.v1";
pub const TASK_WORK_SESSION_VERSION: &str = "task-work-session.v1";
pub const TASK_WORK_SESSION_RECOVERY_VERSION: &str = "task-work-session-recovery.v1";
pub const TASK_WORK_SESSION_EVIDENCE_VERSION: &str = "task-work-session-evidence.v1";
pub const WORK_LOOP_FILESYSTEM_CONTRACT_VERSION: &str = "work-loop-filesystem-contract.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskRunStatus {
    Queued,
    InProgress,
    Validating,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskRun {
    pub version: String,
    pub issue_id: String,
    pub run_id: String,
    pub workflow_ref: String,
    pub status: TaskRunStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_commit: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_owner: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_status: Option<String>,
    pub branch_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub working_directory: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace_root: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub worktree_root: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime_root: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temp_root: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_root: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence_root: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub launch_request_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plan_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub log_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_message_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exit_proof_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub merge_proof_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub started_at: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_heartbeat_at: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attempt_count: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retry_policy: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_attempts: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resumed_from_attempt: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retryable: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recovery_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub merge_state: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub writeback_state: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub terminal_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exited_at: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskWorkSessionStatus {
    Queued,
    Claimed,
    Starting,
    Running,
    InReview,
    Done,
    Interrupted,
    Failed,
    Cancelled,
}

impl TaskWorkSessionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Claimed => "claimed",
            Self::Starting => "starting",
            Self::Running => "running",
            Self::InReview => "in_review",
            Self::Done => "done",
            Self::Interrupted => "interrupted",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskWorkSessionRecord {
    pub version: String,
    pub issue_id: String,
    pub run_id: String,
    pub session_id: String,
    pub provider: String,
    pub session_owner: String,
    pub status: TaskWorkSessionStatus,
    pub attempt_count: u32,
    pub working_directory: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace_root: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub worktree_root: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime_root: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temp_root: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_root: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence_root: Option<String>,
    pub launch_request_path: String,
    pub plan_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub log_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_message_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exit_proof_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub merge_proof_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch_name: Option<String>,
    pub started_at: u64,
    pub last_heartbeat_at: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retry_policy: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_attempts: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resumed_from_attempt: Option<u32>,
    pub retryable: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recovery_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub merge_state: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub writeback_state: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub terminal_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exited_at: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskWorkSessionRecoverySummary {
    pub version: String,
    pub issue_id: String,
    pub run_id: String,
    pub session_id: String,
    pub provider: String,
    pub session_owner: String,
    pub status: TaskWorkSessionStatus,
    pub attempt_count: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resumed_from_attempt: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recovery_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retry_policy: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_attempts: Option<u32>,
    pub retryable: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub terminal_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
    pub updated_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskWorkSessionEvidence {
    pub version: String,
    pub issue_id: String,
    pub run_id: String,
    pub session_id: String,
    pub provider: String,
    pub session_owner: String,
    pub status: TaskWorkSessionStatus,
    pub summary: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub merge_state: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub writeback_state: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub terminal_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub refs: Vec<String>,
    pub generated_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskCommandInput {
    pub label: String,
    pub program: String,
    pub args: Vec<String>,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskCommandRecord {
    pub version: String,
    pub issue_id: String,
    pub run_id: String,
    pub command_id: String,
    pub label: String,
    pub program: String,
    pub args: Vec<String>,
    pub exit_code: Option<i32>,
    pub stdout_path: String,
    pub stderr_path: String,
    pub recorded_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskChangedFile {
    pub path: String,
    pub change_type: String,
    pub insertions: usize,
    pub deletions: usize,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sources: Vec<TaskChangedFileSource>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskChangedFileSource {
    Committed,
    WorkingTree,
    Untracked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskChangedFilesRecord {
    pub version: String,
    pub issue_id: String,
    pub run_id: String,
    pub files: Vec<TaskChangedFile>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_commit: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub head_commit: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tree_sha: Option<String>,
    pub working_tree_hash: String,
    pub patch_sha256: String,
    pub file_content_sha256: String,
    pub changed_file_hash: String,
    pub collected_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskValidationRecord {
    pub version: String,
    pub issue_id: String,
    pub run_id: String,
    pub passed: bool,
    pub command_ids: Vec<String>,
    pub failed_command_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub boundary_failures: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub changed_files_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validation_command_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validation_output_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub patch_sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_content_sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tree_sha: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub changed_file_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validation_result_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_commit: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub head_commit: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub working_tree_hash: Option<String>,
    pub checked_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskEvidence {
    pub version: String,
    pub issue_id: String,
    pub run_id: String,
    pub status: String,
    pub summary: String,
    pub run_path: String,
    pub command_paths: Vec<String>,
    pub validation_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub changed_files_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validation_command_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validation_output_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub patch_sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_content_sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tree_sha: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub changed_file_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validation_result_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_commit: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub head_commit: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub working_tree_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generated_at: Option<u64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub entries: Vec<TaskEvidenceEntry>,
    pub created_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskEvidenceEntryStatus {
    Ready,
    Missing,
    Failed,
    Manual,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskEvidenceEntry {
    pub evidence_type: String,
    pub required: bool,
    pub status: TaskEvidenceEntryStatus,
    pub summary: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub manual_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub manual_risk: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskAcceptanceGateKind {
    Verification,
    Evidence,
    Contract,
    State,
}

impl TaskAcceptanceGateKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Verification => "verification",
            Self::Evidence => "evidence",
            Self::Contract => "contract",
            Self::State => "state",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskAcceptanceSubGateDecision {
    pub gate: TaskAcceptanceGateKind,
    pub passed: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inputs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub outputs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub failure_reasons: Vec<String>,
    pub repair_suggestion: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskAcceptanceOutcome {
    Accepted,
    Rejected,
    HumanReviewRequired,
}

impl TaskAcceptanceOutcome {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Accepted => "accepted",
            Self::Rejected => "rejected",
            Self::HumanReviewRequired => "human_review_required",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskAcceptanceTraceability {
    pub issue_id: String,
    pub run_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_owner: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch_name: Option<String>,
    pub acceptance_decision_path: String,
    pub evidence_path: String,
    pub validation_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub changed_files_path: Option<String>,
    pub closeout_proof_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pr_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub merge_commit_sha: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskAcceptanceGateDecision {
    pub version: String,
    pub issue_id: String,
    pub run_id: String,
    pub passed: bool,
    pub outcome: TaskAcceptanceOutcome,
    pub summary: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sub_gates: Vec<TaskAcceptanceSubGateDecision>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_evidence_types: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence_entries: Vec<TaskEvidenceEntry>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub blockers: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub failure_reasons: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub next_steps: Vec<String>,
    pub traceability: TaskAcceptanceTraceability,
    pub checked_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskPreflightCheckStatus {
    Passed,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskPreflightCheck {
    pub key: String,
    pub status: TaskPreflightCheckStatus,
    pub summary: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub details: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskPreflightDecision {
    pub version: String,
    pub issue_id: String,
    pub run_id: String,
    pub workflow_ref: String,
    pub issue_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub work_command_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub work_action_proposals_path: Option<String>,
    pub passed: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub blockers: Vec<String>,
    pub checks: Vec<TaskPreflightCheck>,
    pub checked_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskRunCheckpoint {
    pub version: String,
    pub issue_id: String,
    pub run_id: String,
    pub checkpoint_id: String,
    pub flow_type: WorkflowFlowType,
    pub state: String,
    pub event_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<String>,
    pub summary: String,
    pub created_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkLoopStage {
    Command,
    Proposal,
    Preflight,
    Session,
    Evidence,
    Handoff,
    Delivery,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkLoopArtifactClass {
    Authority,
    DerivedArtifact,
    TransportSnapshot,
    ReadModel,
    PublicRecord,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkLoopArtifactContract {
    pub key: String,
    pub stage: WorkLoopStage,
    pub class: WorkLoopArtifactClass,
    pub location_ref: String,
    pub description: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub traces_to: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkLoopStageContract {
    pub stage: WorkLoopStage,
    pub issue_statuses: Vec<String>,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkLoopRoleAlias {
    pub canonical_role: String,
    pub accepted_aliases: Vec<String>,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkLoopFilesystemContract {
    pub version: String,
    pub issue_id: String,
    pub workflow_ref: String,
    pub contract_path: String,
    pub role_aliases: Vec<WorkLoopRoleAlias>,
    pub stages: Vec<WorkLoopStageContract>,
    pub artifacts: Vec<WorkLoopArtifactContract>,
    pub generated_at: u64,
}
