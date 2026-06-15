use crate::{
    manager::{assert_build_agent_run, ensure_task_run_for_execute_run, sync_task_run_status},
    model::{ExecuteCommandRecord, ExecuteResult, ExecuteRunStatus},
    storage::{canonical_project_root, load_command_records, read_run},
};
use agentflow_task_artifacts::{
    load_task_evidence, write_task_command_record, write_task_evidence, write_task_validation,
    TaskCommandInput, TaskEvidence, TaskRunStatus,
};
use anyhow::Result;
use std::fs;
use std::path::Path;

pub fn write_execute_evidence(
    project_root: impl AsRef<Path>,
    run_id: String,
    result: ExecuteResult,
) -> Result<Option<TaskEvidence>> {
    let root = canonical_project_root(project_root)?;
    let run = read_run(&root, &run_id)?;
    let issue = assert_build_agent_run(&root, &run)?;
    ensure_task_run_for_execute_run(&root, &issue, &run, None)?;
    let command_records = load_command_records(&root, &run_id)?;
    write_task_evidence_for_execute_result(&root, &run, &result, &command_records)?;
    Ok(load_task_evidence(&root, &run.issue_id).ok())
}

fn write_task_evidence_for_execute_result(
    root: &Path,
    run: &crate::model::ExecuteRun,
    result: &ExecuteResult,
    command_records: &[ExecuteCommandRecord],
) -> Result<()> {
    if load_task_evidence(root, &run.issue_id).is_ok() {
        return Ok(());
    }
    if command_records.is_empty() {
        sync_task_run_status(root, run, TaskRunStatus::Failed);
        return Ok(());
    }
    sync_task_run_status(root, run, TaskRunStatus::Validating);
    for record in command_records {
        write_task_command_record(
            root,
            &run.issue_id,
            &run.run_id,
            TaskCommandInput {
                label: record.label.clone(),
                program: record.program.clone(),
                args: record.args.clone(),
                exit_code: record.exit_code,
                stdout: read_optional_text(root, &record.stdout_path),
                stderr: read_optional_text(root, &record.stderr_path),
            },
        )?;
    }
    let validation = write_task_validation(root, &run.issue_id, &run.run_id)?;
    write_task_evidence(
        root,
        &run.issue_id,
        &run.run_id,
        format!(
            "执行 run {} 完成，验证命令 {} 条，{}。",
            run.run_id,
            validation.command_ids.len(),
            if validation.passed {
                "全部通过"
            } else {
                "存在失败"
            }
        ),
    )?;
    sync_task_run_status(
        root,
        run,
        if matches!(result.status, ExecuteRunStatus::Completed) {
            TaskRunStatus::Completed
        } else {
            TaskRunStatus::Failed
        },
    );
    Ok(())
}

fn read_optional_text(root: &Path, relative_path: &str) -> String {
    fs::read_to_string(root.join(relative_path)).unwrap_or_default()
}
