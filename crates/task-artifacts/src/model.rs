use agentflow_workflow_core::WorkflowFlowType;
use serde::{Deserialize, Serialize};

pub const TASK_RUN_VERSION: &str = "task-run.v1";
pub const TASK_COMMAND_VERSION: &str = "task-command.v1";
pub const TASK_CHANGED_FILES_VERSION: &str = "task-changed-files.v1";
pub const TASK_VALIDATION_VERSION: &str = "task-validation.v1";
pub const TASK_EVIDENCE_VERSION: &str = "task-evidence.v1";
pub const TASK_RUN_CHECKPOINT_VERSION: &str = "task-run-checkpoint.v1";
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
    pub branch_name: Option<String>,
    pub created_at: u64,
    pub updated_at: u64,
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
    pub created_at: u64,
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
