//! AgentFlow projection read model.
//!
//! This crate rebuilds task and project read models from `.agentflow/spec/**`
//! and `.agentflow/events/task-events.jsonl`. Desktop should read these
//! projections instead of runtime write layers.

pub mod model;
pub mod projector;
pub mod storage;

pub use model::{
    IssueStatusIndex, IssueStatusIndexEntry, ProjectProjection, ProjectionPhase,
    ProjectionPublicDelivery, ProjectionSummary, TaskProjection, TaskTimelineItem,
    ISSUE_STATUS_INDEX_VERSION, PROJECT_PROJECTION_VERSION, TASK_PROJECTION_VERSION,
};
pub use projector::rebuild_projections;
pub use storage::{load_project_projection, load_task_projection, prepare_projection_workspace};
