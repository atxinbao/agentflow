use crate::{
    lease::has_active_lease_for_run,
    manager::assert_build_agent_run,
    model::{ExecuteCommandRecord, ExecuteCommandRequest, ExecutePlan, ExecuteRunStatus},
    storage::{
        canonical_project_root, next_named_id, read_json, rebuild_index, run_dir, run_process,
        update_run_status, write_json,
    },
};
use anyhow::Result;
use std::{fs, path::Path};

pub fn run_execute_command(
    project_root: impl AsRef<Path>,
    run_id: String,
    request: ExecuteCommandRequest,
) -> Result<ExecuteCommandRecord> {
    let root = canonical_project_root(project_root)?;
    if is_dangerous_command(&request.program, &request.args) {
        anyhow::bail!("dangerous command is blocked: {}", request.label);
    }
    if !has_active_lease_for_run(&root, &run_id)? {
        anyhow::bail!("run {} must acquire an active lease before command", run_id);
    }
    let run = crate::storage::read_run(&root, &run_id)?;
    assert_build_agent_run(&root, &run)?;
    let plan: ExecutePlan = read_json(&run_dir(&root, &run_id).join("plan.json"))?;
    let normalized = normalize_command(&request.program, &request.args);
    if !plan
        .allowed_commands
        .iter()
        .any(|command| command == &normalized || command == &request.label)
    {
        anyhow::bail!("command is not allowed by run plan: {normalized}");
    }
    let checkpoint_exists = run_dir(&root, &run_id)
        .join("checkpoints")
        .read_dir()
        .map(|mut entries| {
            entries.any(|entry| entry.map(|item| item.path().is_file()).unwrap_or(false))
        })
        .unwrap_or(false);
    if !checkpoint_exists {
        anyhow::bail!("run {} must create checkpoint before command", run_id);
    }

    update_run_status(&root, &run_id, ExecuteRunStatus::Running)?;
    let command_dir = run_dir(&root, &run_id).join("commands");
    let command_id = next_named_id(&command_dir, "cmd-")?;
    let started_at = crate::storage::unix_timestamp_seconds();
    let (exit_code, stdout, stderr) = run_process(&root, &request.program, &request.args, None)?;
    let finished_at = crate::storage::unix_timestamp_seconds();
    let stdout_path = command_dir.join(format!("{command_id}.stdout.txt"));
    let stderr_path = command_dir.join(format!("{command_id}.stderr.txt"));
    fs::write(&stdout_path, stdout)?;
    fs::write(&stderr_path, stderr)?;
    let record = ExecuteCommandRecord {
        version: crate::model::EXECUTE_COMMAND_VERSION.to_string(),
        command_id: command_id.clone(),
        run_id: run_id.clone(),
        label: request.label,
        program: request.program,
        args: request.args,
        cwd: root.display().to_string(),
        source: request
            .source
            .unwrap_or_else(|| "runPlan.allowedCommands".to_string()),
        started_at,
        finished_at,
        exit_code,
        stdout_path: format!(".agentflow/execute/runs/{run_id}/commands/{command_id}.stdout.txt"),
        stderr_path: format!(".agentflow/execute/runs/{run_id}/commands/{command_id}.stderr.txt"),
    };
    write_json(&command_dir.join(format!("{command_id}.json")), &record)?;
    rebuild_index(&root)?;
    Ok(record)
}

fn normalize_command(program: &str, args: &[String]) -> String {
    std::iter::once(program.to_string())
        .chain(args.iter().cloned())
        .collect::<Vec<_>>()
        .join(" ")
}

fn is_dangerous_command(program: &str, args: &[String]) -> bool {
    let normalized = normalize_command(program, args);
    if matches!(program, "sh" | "bash") && args.iter().any(|arg| arg == "-c") {
        return true;
    }
    if program == "git" {
        if let Some(subcommand) = args.first().map(String::as_str) {
            if matches!(
                subcommand,
                "push" | "commit" | "checkout" | "reset" | "clean"
            ) {
                return true;
            }
        }
    }
    if program == "rm"
        && args
            .iter()
            .any(|arg| arg.contains("rf") || arg.contains("fr"))
    {
        return true;
    }
    [
        "rm -rf",
        "git reset --hard",
        "git clean -fd",
        "git push",
        "git commit",
        "git checkout",
        "deploy",
        "release",
        "curl ",
    ]
    .iter()
    .any(|danger| normalized.contains(danger))
}
