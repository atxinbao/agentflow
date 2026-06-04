use anyhow::{Context, Result};
use serde::{de::DeserializeOwned, Serialize};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

pub const STATE_DIRECTORIES: &[&str] = &[
    ".agentflow/state",
    ".agentflow/state/health",
    ".agentflow/state/gates",
    ".agentflow/state/sessions",
    ".agentflow/state/locks",
    ".agentflow/state/events",
    ".agentflow/state/indexes",
];

pub const STATE_REQUIRED_FILES: &[&str] = &[
    ".agentflow/state/manifest.json",
    ".agentflow/state/index.json",
    ".agentflow/state/events/timeline.jsonl",
    ".agentflow/state/gates/workflow.json",
    ".agentflow/state/gates/next-actions.json",
    ".agentflow/state/gates/blockers.json",
    ".agentflow/state/locks/active.json",
    ".agentflow/state/locks/stale.json",
    ".agentflow/state/locks/cleanup-candidates.json",
    ".agentflow/state/indexes/workspace-status.json",
    ".agentflow/state/indexes/issue-status.json",
    ".agentflow/state/indexes/run-status.json",
    ".agentflow/state/indexes/output-status.json",
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

pub fn write_jsonl<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        ensure_directory(parent)?;
    }
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("open {}", path.display()))?;
    let content = serde_json::to_string(value)? + "\n";
    file.write_all(content.as_bytes())
        .with_context(|| format!("append {}", path.display()))
}

pub fn touch_file(path: &Path) -> Result<()> {
    if path.exists() {
        if path.is_file() {
            return Ok(());
        }
        anyhow::bail!("{} exists but is not a file", path.display());
    }
    if let Some(parent) = path.parent() {
        ensure_directory(parent)?;
    }
    fs::write(path, "").with_context(|| format!("write {}", path.display()))
}

pub fn read_json<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

pub fn read_jsonl<T: DeserializeOwned>(path: &Path) -> Result<Vec<T>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    raw.lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).context("parse timeline event"))
        .collect()
}

pub fn sorted_child_paths(path: &Path) -> Result<Vec<PathBuf>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let mut entries = fs::read_dir(path)
        .with_context(|| format!("read directory {}", path.display()))?
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(|entry| entry.path())
        .collect::<Vec<_>>();
    entries.sort();
    Ok(entries)
}

pub fn unix_timestamp_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}
