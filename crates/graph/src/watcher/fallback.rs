use crate::watcher::{
    debounce::{DEBOUNCE_MS, WATCH_INTERVAL_MS},
    filter::{should_ignore_graph_event, should_skip_entry},
    state,
};
use anyhow::{Context, Result};
use std::{
    collections::hash_map::DefaultHasher,
    fs,
    hash::{Hash, Hasher},
    path::Path,
    path::PathBuf,
    thread,
    time::{Duration, UNIX_EPOCH},
};

pub(crate) fn run_fingerprint_watcher(root: PathBuf) {
    let root_key = root.display().to_string();
    let mut last_fingerprint = match project_fingerprint(&root) {
        Ok(value) => value,
        Err(error) => {
            state::update_state(&root_key, |state| {
                state.status = "failed".to_string();
                state.backend = "fingerprint".to_string();
                state.last_error = Some(error.to_string());
            });
            return;
        }
    };

    loop {
        thread::sleep(Duration::from_millis(WATCH_INTERVAL_MS));
        let Ok(current_fingerprint) = project_fingerprint(&root) else {
            state::update_state(&root_key, |state| {
                state.status = "failed".to_string();
                state.last_error = Some("无法读取项目文件变化。".to_string());
            });
            continue;
        };
        if current_fingerprint == last_fingerprint {
            continue;
        }

        state::record_event(&root_key, "debouncing", "fingerprint_change", None);
        thread::sleep(Duration::from_millis(DEBOUNCE_MS));
        let Ok(debounced_fingerprint) = project_fingerprint(&root) else {
            state::update_state(&root_key, |state| {
                state.status = "failed".to_string();
                state.last_error = Some("无法完成文件变化合并。".to_string());
            });
            continue;
        };
        if debounced_fingerprint == last_fingerprint {
            state::update_state(&root_key, |state| {
                state.status = "fallback".to_string();
                state.backend = "fingerprint".to_string();
            });
            continue;
        }

        state::refresh_graph(&root_key, &root, "fingerprint", "fingerprint_change");
        last_fingerprint = debounced_fingerprint;
    }
}

pub(crate) fn project_fingerprint(root: &Path) -> Result<u64> {
    let mut files = Vec::new();
    collect_file_fingerprints(root, root, &mut files)?;
    files.sort();

    let mut hasher = DefaultHasher::new();
    for file in files {
        file.hash(&mut hasher);
    }
    Ok(hasher.finish())
}

pub(crate) fn forced_fallback() -> bool {
    std::env::var("AGENTFLOW_GRAPH_WATCHER_FORCE_FALLBACK")
        .map(|value| matches!(value.as_str(), "1" | "true" | "yes"))
        .unwrap_or(false)
}

fn collect_file_fingerprints(root: &Path, directory: &Path, files: &mut Vec<String>) -> Result<()> {
    for entry in fs::read_dir(directory).with_context(|| format!("read {}", directory.display()))? {
        let entry = entry?;
        let path = entry.path();
        let file_name = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("");
        if should_skip_entry(file_name) {
            continue;
        }
        let metadata = entry.metadata()?;
        if metadata.is_dir() {
            collect_file_fingerprints(root, &path, files)?;
        } else if metadata.is_file() {
            let relative = path.strip_prefix(root).unwrap_or(&path);
            if should_ignore_graph_event(root, relative) {
                continue;
            }
            let modified = metadata
                .modified()
                .ok()
                .and_then(|value| value.duration_since(UNIX_EPOCH).ok())
                .map(|value| value.as_secs())
                .unwrap_or(0);
            files.push(format!(
                "{}:{}:{}",
                relative.to_string_lossy().replace('\\', "/"),
                metadata.len(),
                modified
            ));
        }
    }
    Ok(())
}
