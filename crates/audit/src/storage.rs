use anyhow::{Context, Result};
use serde::{de::DeserializeOwned, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

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

pub fn read_json<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

pub fn unix_timestamp_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

pub fn count_directory_entries(path: &Path) -> usize {
    fs::read_dir(path)
        .map(|entries| entries.filter_map(Result::ok).count())
        .unwrap_or(0)
}

pub fn sorted_child_paths(path: &Path) -> Result<Vec<PathBuf>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let mut entries = fs::read_dir(path)
        .with_context(|| format!("read directory {}", path.display()))?
        .collect::<Result<Vec<_>, _>>()
        .with_context(|| format!("collect directory {}", path.display()))?
        .into_iter()
        .map(|entry| entry.path())
        .collect::<Vec<_>>();
    entries.sort();
    Ok(entries)
}
