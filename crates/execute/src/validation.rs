use crate::{
    evidence::write_execute_evidence,
    lease::finalize_run_and_release,
    model::{ExecuteResult, ExecuteRunStatus},
    result::build_execute_result,
    storage::{canonical_project_root, load_command_records, rebuild_index, write_json},
};
use anyhow::Result;
use std::path::Path;

pub fn validate_execute_run(
    project_root: impl AsRef<Path>,
    run_id: String,
) -> Result<ExecuteResult> {
    let root = canonical_project_root(project_root)?;
    crate::storage::update_run_status(&root, &run_id, ExecuteRunStatus::Validating)?;
    let commands = load_command_records(&root, &run_id)?;
    let passed = !commands.is_empty() && commands.iter().all(|record| record.exit_code == Some(0));
    let result = build_execute_result(&root, &run_id, passed)?;
    write_json(
        &crate::storage::run_dir(&root, &run_id).join("result.json"),
        &result,
    )?;
    write_execute_evidence(&root, run_id.clone(), result.clone())?;
    finalize_run_and_release(
        &root,
        &run_id,
        if passed {
            ExecuteRunStatus::Completed
        } else {
            ExecuteRunStatus::Failed
        },
    )?;
    rebuild_index(&root)?;
    Ok(result)
}

pub fn complete_execute_run(
    project_root: impl AsRef<Path>,
    run_id: String,
) -> Result<ExecuteResult> {
    validate_execute_run(project_root, run_id)
}
