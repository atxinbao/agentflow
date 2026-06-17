//! AgentFlow task artifact store.
//!
//! This crate owns local task execution artifacts under
//! `.agentflow/tasks/<issue-id>/**`. It writes run, command, validation, and
//! evidence records. Public delivery records are intentionally out of scope.

pub mod model;
pub mod storage;

pub use model::{
    TaskCommandInput, TaskCommandRecord, TaskEvidence, TaskRun, TaskRunCheckpoint, TaskRunStatus,
    TaskValidationRecord, TASK_COMMAND_VERSION, TASK_EVIDENCE_VERSION, TASK_RUN_CHECKPOINT_VERSION,
    TASK_RUN_VERSION, TASK_VALIDATION_VERSION,
};
pub use storage::{
    checkpoint_replay_cursor, create_task_run, latest_task_run_checkpoint, load_task_evidence,
    load_task_run, load_task_run_checkpoints, prepare_task_artifact_workspace, task_evidence_dir,
    task_run_dir, update_task_run_status, write_task_command_record, write_task_evidence,
    write_task_run_checkpoint, write_task_validation,
};
