//! AgentFlow task artifact store.
//!
//! This crate owns local task execution artifacts under
//! `.agentflow/tasks/<issue-id>/**`. It writes work-loop contract, run,
//! command, validation, and evidence records. Public delivery records are
//! intentionally out of scope.

pub mod model;
pub mod storage;

pub use model::{
    TaskChangedFile, TaskChangedFileSource, TaskChangedFilesRecord, TaskCommandInput,
    TaskCommandRecord, TaskEvidence, TaskRun, TaskRunCheckpoint, TaskRunStatus,
    TaskValidationRecord, WorkLoopArtifactClass, WorkLoopArtifactContract,
    WorkLoopFilesystemContract, WorkLoopRoleAlias, WorkLoopStage, WorkLoopStageContract,
    TASK_CHANGED_FILES_VERSION, TASK_COMMAND_VERSION, TASK_EVIDENCE_VERSION,
    TASK_RUN_CHECKPOINT_VERSION, TASK_RUN_VERSION, TASK_VALIDATION_VERSION,
    WORK_LOOP_FILESYSTEM_CONTRACT_VERSION,
};
pub use storage::{
    checkpoint_replay_cursor, create_task_run, latest_task_run_checkpoint, load_task_changed_files,
    load_task_evidence, load_task_run, load_task_run_checkpoints,
    load_work_loop_filesystem_contract, prepare_task_artifact_workspace, task_changed_files_path,
    task_evidence_dir, task_launch_request_path, task_run_dir, task_work_loop_contract_path,
    update_task_run_status, write_task_changed_files, write_task_command_record,
    write_task_evidence, write_task_run_checkpoint, write_task_validation,
    write_task_validation_with_assessment, write_work_loop_filesystem_contract,
};
