use crate::{
    lease::has_active_lease_for_run,
    model::{
        ExecuteChangedFile, ExecuteChangedFiles, ExecutePatchOutcome, ExecutePlan,
        ExecutePreflight, ExecuteRunStatus,
    },
    storage::{
        canonical_project_root, path_allowed, read_json, rebuild_index, run_dir, run_process,
        update_run_status, write_json,
    },
};
use anyhow::Result;
use std::{collections::BTreeMap, fs, path::Path};

pub fn apply_execute_patch(
    project_root: impl AsRef<Path>,
    run_id: String,
    proposed_patch: String,
) -> Result<ExecutePatchOutcome> {
    let root = canonical_project_root(project_root)?;
    let run_path = run_dir(&root, &run_id);
    let preflight: ExecutePreflight = read_json(&run_path.join("preflight.json"))?;
    if preflight.status != "ready" {
        anyhow::bail!("run {} preflight is not ready", run_id);
    }
    if !has_active_lease_for_run(&root, &run_id)? {
        anyhow::bail!("run {} must acquire an active lease before patch", run_id);
    }
    let plan: ExecutePlan = read_json(&run_path.join("plan.json"))?;
    let checkpoint_exists = run_path
        .join("checkpoints")
        .read_dir()
        .map(|mut entries| {
            entries.any(|entry| entry.map(|item| item.path().is_file()).unwrap_or(false))
        })
        .unwrap_or(false);
    if !checkpoint_exists {
        anyhow::bail!("run {} must create checkpoint before patch", run_id);
    }

    let changed_files = parse_patch_changed_files(&proposed_patch);
    if changed_files.is_empty() {
        anyhow::bail!("patch does not contain changed files");
    }
    let unauthorized = changed_files
        .iter()
        .find(|file| !path_allowed(&file.path, &plan.allowed_write_paths));
    if let Some(file) = unauthorized {
        anyhow::bail!(
            "patch touches unauthorized path {}. allowedWritePaths: {:?}",
            file.path,
            plan.allowed_write_paths
        );
    }

    update_run_status(&root, &run_id, ExecuteRunStatus::Patching)?;
    let patches_dir = run_path.join("patches");
    fs::write(patches_dir.join("proposed.patch"), &proposed_patch)?;
    let (exit_code, _stdout, stderr) = run_process(
        &root,
        "git",
        &[
            "apply".to_string(),
            "--whitespace=nowarn".to_string(),
            "-".to_string(),
        ],
        Some(&proposed_patch),
    )?;
    if exit_code != Some(0) {
        anyhow::bail!("git apply failed: {stderr}");
    }
    fs::write(patches_dir.join("applied.patch"), &proposed_patch)?;
    let worktree_diff = run_process(&root, "git", &["diff".to_string()], None)
        .map(|(_, stdout, _)| stdout)
        .unwrap_or_default();
    fs::write(patches_dir.join("worktree.diff"), worktree_diff)?;
    let changed = ExecuteChangedFiles {
        version: "execute-changed-files.v1".to_string(),
        run_id: run_id.clone(),
        files: changed_files,
    };
    write_json(&patches_dir.join("changed-files.json"), &changed)?;
    update_run_status(&root, &run_id, ExecuteRunStatus::Running)?;
    rebuild_index(&root)?;
    Ok(ExecutePatchOutcome {
        run_id,
        changed_files: changed,
        proposed_patch_path: ".agentflow/execute/runs".to_string(),
        applied_patch_path: "patches/applied.patch".to_string(),
        worktree_diff_path: "patches/worktree.diff".to_string(),
    })
}

pub(crate) fn parse_patch_changed_files(patch: &str) -> Vec<ExecuteChangedFile> {
    let mut counts: BTreeMap<String, (usize, usize, String)> = BTreeMap::new();
    let mut current_path: Option<String> = None;

    for line in patch.lines() {
        if let Some(path) = parse_patch_path(line) {
            if path != "/dev/null" {
                let change_type = if line.starts_with("+++") && !line.starts_with("+++ /dev/null") {
                    "modified"
                } else {
                    "modified"
                };
                counts
                    .entry(path.clone())
                    .or_insert((0, 0, change_type.to_string()));
                current_path = Some(path);
            }
            continue;
        }

        let Some(path) = current_path.as_ref() else {
            continue;
        };
        if line.starts_with('+') && !line.starts_with("+++") {
            counts
                .entry(path.clone())
                .or_insert((0, 0, "modified".to_string()))
                .0 += 1;
        } else if line.starts_with('-') && !line.starts_with("---") {
            counts
                .entry(path.clone())
                .or_insert((0, 0, "modified".to_string()))
                .1 += 1;
        }
    }

    counts
        .into_iter()
        .map(
            |(path, (insertions, deletions, change_type))| ExecuteChangedFile {
                path,
                change_type,
                insertions,
                deletions,
            },
        )
        .collect()
}

fn parse_patch_path(line: &str) -> Option<String> {
    let raw = if let Some(path) = line.strip_prefix("+++ b/") {
        path
    } else if let Some(path) = line.strip_prefix("--- a/") {
        path
    } else {
        return None;
    };
    Some(raw.split('\t').next().unwrap_or(raw).trim().to_string())
}
