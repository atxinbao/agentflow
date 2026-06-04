use crate::{
    manager::load_issue_for_run,
    model::{ExecutePlan, ExecutePlanDraft, ExecuteRunStatus, EXECUTE_PLAN_VERSION},
    storage::{
        canonical_project_root, read_run, rebuild_index, run_dir, update_run_status, write_json,
    },
};
use anyhow::Result;
use std::path::Path;

pub fn write_execute_plan(
    project_root: impl AsRef<Path>,
    run_id: String,
    draft: ExecutePlanDraft,
) -> Result<ExecutePlan> {
    let root = canonical_project_root(project_root)?;
    let run = read_run(&root, &run_id)?;
    let issue = load_issue_for_run(&root, &run)?;
    if draft.allowed_write_paths.is_empty() {
        anyhow::bail!("execute plan requires at least one allowedWritePaths entry");
    }
    if draft.allowed_commands.is_empty() && issue.validation_hints.is_empty() {
        anyhow::bail!("execute plan requires allowedCommands or issue validation hints");
    }

    let plan = ExecutePlan {
        version: EXECUTE_PLAN_VERSION.to_string(),
        run_id: run.run_id.clone(),
        issue_id: run.issue_id.clone(),
        steps: draft.steps,
        allowed_write_paths: draft.allowed_write_paths,
        allowed_commands: draft.allowed_commands,
    };
    write_json(&run_dir(&root, &run_id).join("plan.json"), &plan)?;
    update_run_status(&root, &run_id, ExecuteRunStatus::Planned)?;
    rebuild_index(&root)?;
    Ok(plan)
}
