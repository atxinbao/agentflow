use crate::{
    checkpoint::create_execute_checkpoint,
    delivery::prepare_release_delivery,
    lease::{acquire_execute_lease, has_active_lease_for_run},
    manager::{
        assert_build_agent_run, load_issue_for_run, sync_issue_loop_projection,
        update_input_issue_status,
    },
    model::{
        BuildAgentCompletion, BuildAgentCompletionRequest, BuildAgentValidationCommand,
        ExecuteChangedFiles, ExecuteCommandRecord, ExecutePlanDraft, ExecutePreflight, ExecuteRun,
        ExecuteRunStatus,
    },
    plan::write_execute_plan,
    storage::{
        canonical_project_root, next_named_id, read_json, read_run, rebuild_index, run_dir,
        unix_timestamp_seconds, update_run_status, write_json,
    },
    validation::validate_execute_run,
};
use agentflow_input::issue::InputIssueStatus;
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
    let run_id = request
        .run_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| anyhow::anyhow!("build agent completion requires runId"))?;
    if request.validation_commands.is_empty() {
        anyhow::bail!("build agent completion requires validation command results");
    }

    let run = read_run(&root, run_id)
        .with_context(|| format!("build agent completion requires existing run {run_id}"))?;
    if run.issue_id != issue_id {
        anyhow::bail!(
            "build agent completion issueId mismatch: request {issue_id}, run {}",
            run.issue_id
        );
    }
    assert_build_agent_run(&root, &run)?;
    require_ready_preflight(&root, &run)?;
    require_branch_metadata(&root, &run)?;
    require_merge_proof(&root, &run)?;
    if !has_active_lease_for_run(&root, &run.run_id)? {
        acquire_execute_lease(&root, run.run_id.clone())?;
    }

    let run = read_run(&root, &run.run_id)?;
    let issue = load_issue_for_run(&root, &run)?;
    let result_path = run_dir(&root, &run.run_id).join("result.json");
    let delivery_path = root
        .join(".agentflow/output/release")
        .join(&run.run_id)
        .join("delivery.json");
    let result = if result_path.is_file() {
        read_json(&result_path)?
    } else {
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
        validate_execute_run(&root, run.run_id.clone())?
    };
    let delivery = if delivery_path.is_file() {
        agentflow_output::load_release_delivery(&root, run.run_id.clone())?
    } else {
        prepare_release_delivery(&root, run.run_id.clone())?
    };
    update_input_issue_status(&root, &run.issue_id, InputIssueStatus::Done)?;
    sync_issue_loop_projection(
        &root,
        &run,
        InputIssueStatus::Done,
        Some("merged".to_string()),
        Vec::new(),
    )?;
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

fn require_ready_preflight(root: &Path, run: &ExecuteRun) -> Result<()> {
    let preflight: ExecutePreflight = read_json(&run_dir(root, &run.run_id).join("preflight.json"))
        .with_context(|| format!("load ready preflight for {}", run.run_id))?;
    if preflight.status != "ready" {
        anyhow::bail!(
            "build agent completion requires ready preflight for {}: {}",
            run.run_id,
            preflight
                .blocked_reason
                .unwrap_or_else(|| preflight.status.clone())
        );
    }
    Ok(())
}

fn require_branch_metadata(root: &Path, run: &ExecuteRun) -> Result<()> {
    let branch_path = run_dir(root, &run.run_id).join("branch.json");
    if !branch_path.is_file() {
        anyhow::bail!(
            "build agent completion requires branch metadata for {}",
            run.run_id
        );
    }
    let metadata: serde_json::Value = read_json(&branch_path)?;
    let status = metadata
        .get("status")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    if status == "blocked" {
        anyhow::bail!("build agent completion requires non-blocked branch metadata");
    }
    Ok(())
}

fn require_merge_proof(root: &Path, run: &ExecuteRun) -> Result<()> {
    let proof_path = run_dir(root, &run.run_id).join("review/merge-proof.json");
    if !proof_path.is_file() {
        anyhow::bail!(
            "build agent completion requires merge proof for {}",
            run.run_id
        );
    }
    let proof: serde_json::Value = read_json(&proof_path)?;
    let merged = proof
        .get("merged")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    if !merged {
        anyhow::bail!("build agent completion requires merged PR/MR proof");
    }
    Ok(())
}

pub(crate) fn write_changed_files(
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

pub(crate) fn write_validation_command_records(
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
