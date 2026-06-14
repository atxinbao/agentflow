//! AgentFlow task loop scheduler.
//!
//! This crate reads `.agentflow/spec/**`, appends task events, and creates
//! issue-scoped launch requests. It does not execute provider sessions.

pub mod model;
pub mod scheduler;

pub use model::{
    AgentLaunchPayload, TaskLoopLaunch, TaskLoopSchedule, TaskLoopTick, AGENT_LAUNCH_REQUESTED,
    ISSUE_SCHEDULED, TASK_LOOP_LAUNCH_REQUEST_VERSION,
};
pub use scheduler::TaskLoop;
