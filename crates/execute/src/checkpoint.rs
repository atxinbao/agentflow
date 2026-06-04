use crate::{
    model::{ExecuteCheckpoint, ExecuteFileHash, ExecutePlan, ExecuteRunStatus},
    storage::{
        canonical_project_root, command_output, file_sha256, next_named_id, read_json, read_run,
        rebuild_index, run_dir, update_run_status, write_json,
    },
};
use anyhow::Result;
use std::path::Path;

pub fn create_execute_checkpoint(
    project_root: impl AsRef<Path>,
    run_id: String,
) -> Result<ExecuteCheckpoint> {
    let root = canonical_project_root(project_root)?;
    let run = read_run(&root, &run_id)?;
    let plan: ExecutePlan = read_json(&run_dir(&root, &run_id).join("plan.json"))?;
    let checkpoint_dir = run_dir(&root, &run_id).join("checkpoints");
    let checkpoint_id = next_named_id(&checkpoint_dir, "chk-")?;

    let file_hashes_before = plan
        .allowed_write_paths
        .iter()
        .filter_map(|relative_path| {
            file_sha256(&root.join(relative_path))
                .ok()
                .flatten()
                .map(|hash| ExecuteFileHash {
                    path: relative_path.clone(),
                    hash,
                })
        })
        .collect();
    let dirty_files_before = command_output(&root, "git", &["status", "--short"])?
        .unwrap_or_default()
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect();
    let checkpoint = ExecuteCheckpoint {
        version: "execute-checkpoint.v1".to_string(),
        checkpoint_id: checkpoint_id.clone(),
        run_id: run.run_id,
        created_at: crate::storage::unix_timestamp_seconds(),
        git_head: command_output(&root, "git", &["rev-parse", "HEAD"])?,
        dirty_files_before,
        panel_snapshot_id: run.input.panel_snapshot_id.clone(),
        file_hashes_before,
    };
    write_json(
        &checkpoint_dir.join(format!("{checkpoint_id}.json")),
        &checkpoint,
    )?;
    update_run_status(&root, &run_id, ExecuteRunStatus::Checkpointed)?;
    rebuild_index(&root)?;
    Ok(checkpoint)
}
