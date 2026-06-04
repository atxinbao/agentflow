use anyhow::{Context, Result};
use serde::{de::DeserializeOwned, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::Write,
    path::{Component, Path, PathBuf},
    process::{Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::model::{
    ExecuteCommandRecord, ExecuteIndex, ExecuteLease, ExecuteLeaseIndexEntry, ExecuteRun,
    ExecuteRunIndexEntry, ExecuteRunStatus,
};

pub const EXECUTE_DIRECTORIES: &[&str] = &[
    ".agentflow/execute",
    ".agentflow/execute/runs",
    ".agentflow/execute/leases",
    ".agentflow/execute/queue",
    ".agentflow/output",
    ".agentflow/output/evidence",
];

pub const EXECUTE_REQUIRED_FILES: &[&str] = &[
    ".agentflow/execute/manifest.json",
    ".agentflow/execute/index.json",
    ".agentflow/execute/queue/pending.json",
    ".agentflow/execute/queue/active.json",
    ".agentflow/execute/queue/blocked.json",
];

pub fn canonical_project_root(project_root: impl AsRef<Path>) -> Result<PathBuf> {
    let root = project_root.as_ref();
    if root.exists() {
        return root
            .canonicalize()
            .with_context(|| format!("canonicalize {}", root.display()));
    }
    Ok(root.to_path_buf())
}

pub fn ensure_directory(path: &Path) -> Result<()> {
    if path.exists() {
        if path.is_dir() {
            return Ok(());
        }
        anyhow::bail!("{} exists but is not a directory", path.display());
    }
    fs::create_dir_all(path).with_context(|| format!("create {}", path.display()))
}

pub fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        ensure_directory(parent)?;
    }
    let content = serde_json::to_string_pretty(value)? + "\n";
    fs::write(path, content).with_context(|| format!("write {}", path.display()))
}

pub fn write_json_if_missing<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    if path.exists() {
        if path.is_file() {
            return Ok(());
        }
        anyhow::bail!("{} exists but is not a file", path.display());
    }
    write_json(path, value)
}

pub fn read_json<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

pub fn read_json_files<T: DeserializeOwned>(directory: &Path) -> Result<Vec<T>> {
    if !directory.exists() {
        return Ok(Vec::new());
    }
    let mut entries = fs::read_dir(directory)
        .with_context(|| format!("read directory {}", directory.display()))?
        .collect::<Result<Vec<_>, _>>()
        .with_context(|| format!("collect directory {}", directory.display()))?;
    entries.sort_by_key(|entry| entry.path());

    entries
        .into_iter()
        .filter(|entry| entry.path().extension().and_then(|value| value.to_str()) == Some("json"))
        .map(|entry| read_json::<T>(&entry.path()))
        .collect()
}

pub fn unix_timestamp_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

pub fn relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

pub fn run_dir(root: &Path, run_id: &str) -> PathBuf {
    root.join(".agentflow/execute/runs").join(run_id)
}

pub fn read_run(root: &Path, run_id: &str) -> Result<ExecuteRun> {
    read_json(&run_dir(root, run_id).join("run.json"))
}

pub fn write_run(root: &Path, run: &ExecuteRun) -> Result<()> {
    write_json(&run_dir(root, &run.run_id).join("run.json"), run)
}

pub fn update_run_status(
    root: &Path,
    run_id: &str,
    status: ExecuteRunStatus,
) -> Result<ExecuteRun> {
    let mut run = read_run(root, run_id)?;
    run.status = status;
    run.updated_at = unix_timestamp_seconds();
    write_run(root, &run)?;
    Ok(run)
}

pub fn next_run_id(root: &Path) -> Result<String> {
    let runs_dir = root.join(".agentflow/execute/runs");
    ensure_directory(&runs_dir)?;
    let mut max_id = 0_u64;
    for entry in fs::read_dir(&runs_dir).with_context(|| format!("read {}", runs_dir.display()))? {
        let entry = entry?;
        if !entry.path().is_dir() {
            continue;
        }
        let Some(name) = entry.file_name().to_str().map(str::to_string) else {
            continue;
        };
        if let Some(raw) = name.strip_prefix("run-") {
            if let Ok(number) = raw.parse::<u64>() {
                max_id = max_id.max(number);
            }
        }
    }
    Ok(format!("run-{next:03}", next = max_id + 1))
}

pub fn next_named_id(directory: &Path, prefix: &str) -> Result<String> {
    ensure_directory(directory)?;
    let mut max_id = 0_u64;
    for entry in fs::read_dir(directory).with_context(|| format!("read {}", directory.display()))? {
        let entry = entry?;
        let Some(name) = entry.file_name().to_str().map(str::to_string) else {
            continue;
        };
        if let Some(raw) = name
            .strip_prefix(prefix)
            .and_then(|value| value.strip_suffix(".json"))
        {
            if let Ok(number) = raw.parse::<u64>() {
                max_id = max_id.max(number);
            }
        }
    }
    Ok(format!("{prefix}{next:03}", next = max_id + 1))
}

pub fn rebuild_index(root: &Path) -> Result<ExecuteIndex> {
    let runs = load_runs(root)?
        .into_iter()
        .map(|run| ExecuteRunIndexEntry {
            run_id: run.run_id.clone(),
            issue_id: run.issue_id.clone(),
            source_spec_id: run.source_spec_id.clone(),
            status: run.status.clone(),
            path: format!(".agentflow/execute/runs/{}/run.json", run.run_id),
            updated_at: run.updated_at,
        })
        .collect();
    let leases = load_leases(root)?
        .into_iter()
        .map(|lease| ExecuteLeaseIndexEntry {
            issue_id: lease.issue_id.clone(),
            run_id: lease.run_id.clone(),
            status: lease.status.clone(),
            path: format!(".agentflow/execute/leases/{}.json", lease.issue_id),
        })
        .collect();
    let index = ExecuteIndex {
        updated_at: unix_timestamp_seconds(),
        runs,
        leases,
        ..ExecuteIndex::default()
    };
    write_json(&root.join(".agentflow/execute/index.json"), &index)?;
    Ok(index)
}

pub fn load_runs(root: &Path) -> Result<Vec<ExecuteRun>> {
    let runs_dir = root.join(".agentflow/execute/runs");
    if !runs_dir.exists() {
        return Ok(Vec::new());
    }
    let mut entries = fs::read_dir(&runs_dir)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.path());
    entries
        .into_iter()
        .filter(|entry| entry.path().is_dir())
        .filter(|entry| entry.path().join("run.json").is_file())
        .map(|entry| read_json::<ExecuteRun>(&entry.path().join("run.json")))
        .collect()
}

pub fn load_leases(root: &Path) -> Result<Vec<ExecuteLease>> {
    read_json_files(&root.join(".agentflow/execute/leases"))
}

pub fn load_command_records(root: &Path, run_id: &str) -> Result<Vec<ExecuteCommandRecord>> {
    read_json_files(&run_dir(root, run_id).join("commands"))
}

pub fn path_is_safe_relative(path: &str) -> bool {
    let path = Path::new(path);
    !path.is_absolute()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_) | Component::CurDir))
}

pub fn path_allowed(path: &str, allowed_paths: &[String]) -> bool {
    path_is_safe_relative(path)
        && allowed_paths.iter().any(|allowed| {
            let normalized_allowed = allowed.trim_end_matches('/');
            path == normalized_allowed || path.starts_with(&format!("{normalized_allowed}/"))
        })
}

pub fn file_sha256(path: &Path) -> Result<Option<String>> {
    if !path.exists() || !path.is_file() {
        return Ok(None);
    }
    let content = fs::read(path).with_context(|| format!("read {}", path.display()))?;
    let mut hasher = Sha256::new();
    hasher.update(content);
    Ok(Some(format!("{:x}", hasher.finalize())))
}

pub fn command_output(root: &Path, program: &str, args: &[&str]) -> Result<Option<String>> {
    let output = Command::new(program).args(args).current_dir(root).output();
    match output {
        Ok(output) if output.status.success() => Ok(Some(
            String::from_utf8_lossy(&output.stdout).trim().to_string(),
        )),
        _ => Ok(None),
    }
}

pub fn run_process(
    root: &Path,
    program: &str,
    args: &[String],
    stdin: Option<&str>,
) -> Result<(Option<i32>, String, String)> {
    let mut command = Command::new(program);
    command
        .args(args)
        .current_dir(root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if stdin.is_some() {
        command.stdin(Stdio::piped());
    }
    let mut child = command
        .spawn()
        .with_context(|| format!("spawn command {program}"))?;
    if let Some(input) = stdin {
        if let Some(mut child_stdin) = child.stdin.take() {
            child_stdin
                .write_all(input.as_bytes())
                .with_context(|| format!("write stdin for {program}"))?;
        }
    }
    let output = child
        .wait_with_output()
        .with_context(|| format!("wait command {program}"))?;
    Ok((
        output.status.code(),
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    ))
}
