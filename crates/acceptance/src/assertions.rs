use anyhow::{Context, Result};
use serde::Serialize;
use std::{fs, path::Path};

pub fn assert_path_exists(root: &Path, relative_path: &str) -> Result<()> {
    let path = root.join(relative_path);
    anyhow::ensure!(path.exists(), "expected path to exist: {}", path.display());
    Ok(())
}

pub fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(path, serde_json::to_string_pretty(value)?)
        .with_context(|| format!("write {}", path.display()))
}
