use crate::{
    checkpoint::create_execute_checkpoint,
    delivery::prepare_release_delivery,
    lease::acquire_execute_lease,
    manager::{create_execute_run, load_issue_for_run},
    model::{
        BuildAgentCompletion, BuildAgentCompletionRequest, BuildAgentValidationCommand,
        ExecuteChangedFiles, ExecuteCommandRecord, ExecutePlanDraft, ExecuteRunStatus,
    },
    plan::write_execute_plan,
    preflight::execute_run_preflight,
    storage::{
        canonical_project_root, next_named_id, read_run, rebuild_index, run_dir,
        unix_timestamp_seconds, update_run_status, write_json,
    },
    validation::validate_execute_run,
};
use anyhow::{Context, Result};
use std::{fs, path::Path};

pub fn complete_build_agent_issue(
    project_root: impl AsRef<Path>,
    request: BuildAgentCompletionRequest,
) -> Result<BuildAgentCompletion> {
    let root = canonical_project_root(project_root)?;
    let issue_id = request.issue_id.trim();
    if issue_id.is_empty() {
        anyhow::bail!("build agent completion requires issueId");
    }
    if request.validation_commands.is_empty() {
        anyhow::bail!("build agent completion requires validation command results");
    }

    let run = create_execute_run(&root, issue_id.to_string())?;
    let preflight = execute_run_preflight(&root, run.run_id.clone())?;
    if preflight.status != "ready" {
        anyhow::bail!(
            "build agent completion requires ready preflight for {}: {}",
            run.run_id,
            preflight
                .blocked_reason
                .unwrap_or_else(|| preflight.status.clone())
        );
    }
    acquire_execute_lease(&root, run.run_id.clone())?;

    let run = read_run(&root, &run.run_id)?;
    let issue = load_issue_for_run(&root, &run)?;
    let allowed_write_paths = if issue.allowed_paths.is_empty() {
        issue.scope.clone()
    } else {
        issue.allowed_paths.clone()
    };
    if allowed_write_paths.is_empty() {
        anyhow::bail!("build agent completion requires issue allowedPaths or scope");
    }

    write_execute_plan(
        &root,
        run.run_id.clone(),
        ExecutePlanDraft {
            steps: Vec::new(),
            allowed_write_paths,
            allowed_commands: allowed_commands(&request.validation_commands),
        },
    )?;
    create_execute_checkpoint(&root, run.run_id.clone())?;
    write_changed_files(&root, &run.run_id, request.changed_files)?;
    write_validation_command_records(&root, &run.run_id, &request.validation_commands)?;
    update_run_status(&root, &run.run_id, ExecuteRunStatus::Running)?;

    let result = validate_execute_run(&root, run.run_id.clone())?;
    let delivery = prepare_release_delivery(&root, run.run_id.clone())?;
    let run = read_run(&root, &run.run_id)?;
    rebuild_index(&root)?;

    Ok(BuildAgentCompletion {
        run,
        result,
        delivery,
    })
}

fn allowed_commands(commands: &[BuildAgentValidationCommand]) -> Vec<String> {
    commands
        .iter()
        .map(|command| normalize_command(&command.program, &command.args))
        .collect()
}

fn write_changed_files(
    root: &Path,
    run_id: &str,
    changed_files: Vec<crate::model::ExecuteChangedFile>,
) -> Result<()> {
    write_json(
        &run_dir(root, run_id).join("patches/changed-files.json"),
        &ExecuteChangedFiles {
            version: "execute-changed-files.v1".to_string(),
            run_id: run_id.to_string(),
            files: changed_files,
        },
    )
}

fn write_validation_command_records(
    root: &Path,
    run_id: &str,
    commands: &[BuildAgentValidationCommand],
) -> Result<()> {
    let command_dir = run_dir(root, run_id).join("commands");
    for command in commands {
        let command_id = next_named_id(&command_dir, "cmd-")?;
        let now = unix_timestamp_seconds();
        let stdout_path = command_dir.join(format!("{command_id}.stdout.txt"));
        let stderr_path = command_dir.join(format!("{command_id}.stderr.txt"));
        fs::write(&stdout_path, command.stdout.clone().unwrap_or_default())
            .with_context(|| format!("write {}", stdout_path.display()))?;
        fs::write(&stderr_path, command.stderr.clone().unwrap_or_default())
            .with_context(|| format!("write {}", stderr_path.display()))?;
        write_json(
            &command_dir.join(format!("{command_id}.json")),
            &ExecuteCommandRecord {
                version: crate::model::EXECUTE_COMMAND_VERSION.to_string(),
                command_id: command_id.clone(),
                run_id: run_id.to_string(),
                label: non_empty_or_command(&command.label, command),
                program: command.program.clone(),
                args: command.args.clone(),
                cwd: root.display().to_string(),
                source: command
                    .source
                    .clone()
                    .unwrap_or_else(|| "buildAgentCompletion.validationCommands".to_string()),
                started_at: now,
                finished_at: now,
                exit_code: command.exit_code,
                stdout_path: format!(
                    ".agentflow/execute/runs/{run_id}/commands/{command_id}.stdout.txt"
                ),
                stderr_path: format!(
                    ".agentflow/execute/runs/{run_id}/commands/{command_id}.stderr.txt"
                ),
            },
        )?;
    }
    Ok(())
}

fn non_empty_or_command(label: &str, command: &BuildAgentValidationCommand) -> String {
    if label.trim().is_empty() {
        normalize_command(&command.program, &command.args)
    } else {
        label.to_string()
    }
}

fn normalize_command(program: &str, args: &[String]) -> String {
    std::iter::once(program.to_string())
        .chain(args.iter().cloned())
        .collect::<Vec<_>>()
        .join(" ")
}
