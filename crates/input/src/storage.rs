use anyhow::{Context, Result};
use serde::{de::DeserializeOwned, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

pub const INPUT_DIRECTORIES: &[&str] = &[
    ".agentflow/input",
    ".agentflow/input/intake",
    ".agentflow/input/specs",
    ".agentflow/input/specs/drafts",
    ".agentflow/input/specs/approved",
    ".agentflow/input/specs/archive",
    ".agentflow/input/projects",
    ".agentflow/input/issues",
    ".agentflow/input/relations",
    ".agentflow/input/views",
];

pub const INPUT_REQUIRED_FILES: &[&str] = &[
    ".agentflow/input/manifest.json",
    ".agentflow/input/index.json",
    ".agentflow/input/relations/issue-relations.json",
    ".agentflow/input/relations/dependency-graph.json",
    ".agentflow/input/views/active.json",
    ".agentflow/input/views/blocked.json",
    ".agentflow/input/views/by-spec.json",
    ".agentflow/input/views/by-project.json",
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

pub fn write_json_if_missing<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    if path.exists() {
        if path.is_file() {
            return Ok(());
        }
        anyhow::bail!("{} exists but is not a file", path.display());
    }

    write_json(path, value)
}

pub fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        ensure_directory(parent)?;
    }
    let content = serde_json::to_string_pretty(value)? + "\n";
    fs::write(path, content).with_context(|| format!("write {}", path.display()))
}

pub fn write_json_if_changed<T: Serialize>(path: &Path, value: &T) -> Result<bool> {
    if let Some(parent) = path.parent() {
        ensure_directory(parent)?;
    }
    let content = serde_json::to_string_pretty(value)? + "\n";
    if path.is_file() {
        let existing =
            fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
        if existing == content {
            return Ok(false);
        }
    }
    fs::write(path, content).with_context(|| format!("write {}", path.display()))?;
    Ok(true)
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
