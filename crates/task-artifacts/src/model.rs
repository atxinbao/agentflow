use agentflow_workflow_core::WorkflowFlowType;
use serde::{Deserialize, Serialize};

pub const TASK_RUN_VERSION: &str = "task-run.v1";
pub const TASK_COMMAND_VERSION: &str = "task-command.v1";
pub const TASK_VALIDATION_VERSION: &str = "task-validation.v1";
pub const TASK_EVIDENCE_VERSION: &str = "task-evidence.v1";
pub const TASK_RUN_CHECKPOINT_VERSION: &str = "task-run-checkpoint.v1";

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
pub struct TaskValidationRecord {
    pub version: String,
    pub issue_id: String,
    pub run_id: String,
    pub passed: bool,
    pub command_ids: Vec<String>,
    pub failed_command_ids: Vec<String>,
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
