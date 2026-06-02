use anyhow::{Context, Result};
use std::{fs, path::Path};

pub(crate) fn next_id(directory: &Path, prefix: &str) -> Result<String> {
    fs::create_dir_all(directory).with_context(|| format!("create {}", directory.display()))?;
    let mut max_value = 0u64;
    for entry in fs::read_dir(directory).with_context(|| format!("read {}", directory.display()))? {
        let entry = entry?;
        let Some(file_name) = entry.file_name().to_str().map(str::to_string) else {
            continue;
        };
        let Some(stem) = file_name.strip_suffix(".json") else {
            continue;
        };
        let Some(number) = stem.strip_prefix(&format!("{prefix}-")) else {
            continue;
        };
        if let Ok(value) = number.parse::<u64>() {
            max_value = max_value.max(value);
        }
    }
    Ok(format!("{prefix}-{:03}", max_value + 1))
}
