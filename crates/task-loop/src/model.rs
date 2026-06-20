use agentflow_runtime_api::WorkCommandHandoff;
use serde::{Deserialize, Serialize};

pub const ISSUE_SCHEDULED: &str = "issue.scheduled";
pub const AGENT_LAUNCH_REQUESTED: &str = "agent.launch.requested";
pub const TASK_LOOP_LAUNCH_REQUEST_VERSION: &str = "task-loop-launch-request.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskLoopSchedule {
    pub project_id: String,
    pub issue_id: String,
    pub workflow_ref: String,
    pub event_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskLoopLaunch {
    pub project_id: Option<String>,
    pub issue_id: String,
    pub run_id: String,
    pub branch_name: String,
    pub launch_request_path: String,
    pub event_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskLoopTick {
    pub schedule: Option<TaskLoopSchedule>,
    pub launch: TaskLoopLaunch,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentLaunchPayload {
    pub version: String,
    pub provider: String,
    pub issue_id: String,
    pub project_id: Option<String>,
    pub run_id: String,
    pub agent_role: String,
    pub workflow_ref: String,
    pub working_directory: String,
    pub issue_path: String,
    pub launch_request_path: String,
    pub context_pack_path: Option<String>,
    pub branch_name: String,
    pub merge_mode: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub work_command: Option<WorkCommandHandoff>,
}
