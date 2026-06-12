use crate::{
    manager::{assert_build_agent_run, sync_issue_loop_projection, update_input_issue_status},
    model::{ExecutePlan, ExecutePlanDraft, ExecuteRunStatus, EXECUTE_PLAN_VERSION},
    storage::{
        canonical_project_root, read_run, rebuild_index, run_dir, update_run_status, write_json,
    },
};
use agentflow_input::issue::InputIssueStatus;
use anyhow::Result;
use std::path::Path;

pub fn write_execute_plan(
    project_root: impl AsRef<Path>,
    run_id: String,
    draft: ExecutePlanDraft,
) -> Result<ExecutePlan> {
    let root = canonical_project_root(project_root)?;
    let run = read_run(&root, &run_id)?;
    let issue = assert_build_agent_run(&root, &run)?;
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
    update_input_issue_status(&root, &run.issue_id, InputIssueStatus::InProgress)?;
    sync_issue_loop_projection(&root, &run, InputIssueStatus::InProgress, None, Vec::new())?;
    rebuild_index(&root)?;
    Ok(plan)
}
