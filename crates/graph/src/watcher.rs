use crate::{manager::index_project_graph, model::GraphWatcherSnapshot};
use anyhow::{Context, Result};
use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    fs,
    hash::{Hash, Hasher},
    path::{Component, Path, PathBuf},
    sync::{Mutex, OnceLock},
    thread,
    time::{Duration, UNIX_EPOCH},
};

#[cfg(not(test))]
const WATCH_INTERVAL_MS: u64 = 1_000;
#[cfg(test)]
const WATCH_INTERVAL_MS: u64 = 100;
#[cfg(not(test))]
const DEBOUNCE_MS: u64 = 1_500;
#[cfg(test)]
const DEBOUNCE_MS: u64 = 200;

#[derive(Debug, Clone)]
struct WatcherState {
    status: String,
    debounce_ms: u64,
    last_error: Option<String>,
}

static WATCHERS: OnceLock<Mutex<HashMap<String, WatcherState>>> = OnceLock::new();

pub fn ensure_graph_watcher(project_root: impl AsRef<Path>) -> Result<GraphWatcherSnapshot> {
    let root = project_root
        .as_ref()
        .canonicalize()
        .with_context(|| format!("canonicalize {}", project_root.as_ref().display()))?;
    let root_key = root.display().to_string();
    let registry = WATCHERS.get_or_init(|| Mutex::new(HashMap::new()));

    {
        let mut watchers = registry.lock().expect("graph watcher registry poisoned");
        if let Some(state) = watchers.get(&root_key) {
            return Ok(snapshot(&root, state));
        }
        watchers.insert(
            root_key.clone(),
            WatcherState {
                status: "watching".to_string(),
                debounce_ms: DEBOUNCE_MS,
                last_error: None,
            },
        );
    }

    let initial_fingerprint = project_fingerprint(&root)?;
    thread::spawn(move || run_watcher(root, initial_fingerprint));
    let watchers = registry.lock().expect("graph watcher registry poisoned");
    Ok(snapshot(
        Path::new(&root_key),
        watchers.get(&root_key).expect("watcher was just inserted"),
    ))
}

pub(crate) fn watcher_status(project_root: impl AsRef<Path>) -> Option<String> {
    let root = project_root.as_ref().canonicalize().ok()?;
    let root_key = root.display().to_string();
    WATCHERS.get().and_then(|registry| {
        registry
            .lock()
            .ok()?
            .get(&root_key)
            .map(|state| state.status.clone())
    })
}

fn run_watcher(root: PathBuf, mut last_fingerprint: u64) {
    let root_key = root.display().to_string();

    loop {
        thread::sleep(Duration::from_millis(WATCH_INTERVAL_MS));
        let Ok(current_fingerprint) = project_fingerprint(&root) else {
            update_state(
                &root_key,
                "failed",
                Some("无法读取项目文件变化。".to_string()),
            );
            continue;
        };
        if current_fingerprint == last_fingerprint {
            continue;
        }

        update_state(&root_key, "debouncing", None);
        thread::sleep(Duration::from_millis(DEBOUNCE_MS));
        let Ok(debounced_fingerprint) = project_fingerprint(&root) else {
            update_state(
                &root_key,
                "failed",
                Some("无法完成文件变化合并。".to_string()),
            );
            continue;
        };
        if debounced_fingerprint == last_fingerprint {
            update_state(&root_key, "watching", None);
            continue;
        }

        update_state(&root_key, "indexing", None);
        match index_project_graph(&root) {
            Ok(_) => {
                last_fingerprint = debounced_fingerprint;
                update_state(&root_key, "watching", None);
            }
            Err(error) => {
                last_fingerprint = debounced_fingerprint;
                update_state(&root_key, "failed", Some(error.to_string()));
            }
        }
    }
}

fn update_state(root_key: &str, status: &str, last_error: Option<String>) {
    let registry = WATCHERS.get_or_init(|| Mutex::new(HashMap::new()));
    if let Ok(mut watchers) = registry.lock() {
        watchers.insert(
            root_key.to_string(),
            WatcherState {
                status: status.to_string(),
                debounce_ms: DEBOUNCE_MS,
                last_error,
            },
        );
    }
}

fn snapshot(root: &Path, state: &WatcherState) -> GraphWatcherSnapshot {
    GraphWatcherSnapshot {
        version: "graph-watcher.v1".to_string(),
        project_root: root.display().to_string(),
        status: state.status.clone(),
        debounce_ms: state.debounce_ms,
        last_error: state.last_error.clone(),
    }
}

fn project_fingerprint(root: &Path) -> Result<u64> {
    let mut files = Vec::new();
    collect_file_fingerprints(root, root, &mut files)?;
    files.sort();

    let mut hasher = DefaultHasher::new();
    for file in files {
        file.hash(&mut hasher);
    }
    Ok(hasher.finish())
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
            if is_ignored_relative(relative) {
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

fn should_skip_entry(file_name: &str) -> bool {
    matches!(
        file_name,
        ".git"
            | ".agentflow"
            | "node_modules"
            | "target"
            | "dist"
            | "build"
            | "coverage"
            | ".cache"
            | "vendor"
            | ".idea"
            | ".vscode"
            | ".DS_Store"
    )
}

fn is_ignored_relative(path: &Path) -> bool {
    path.components().any(|component| {
        let Component::Normal(value) = component else {
            return false;
        };
        should_skip_entry(value.to_str().unwrap_or(""))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manager::{index_project_graph, load_project_graph_status};
    use tempfile::tempdir;

    #[test]
    fn fingerprint_ignores_agentflow_and_target_runtime_files() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("src.rs"), "fn a() {}\n").unwrap();
        fs::create_dir_all(dir.path().join(".agentflow/output/graph")).unwrap();
        fs::create_dir_all(dir.path().join("target")).unwrap();

        let before = project_fingerprint(dir.path()).unwrap();
        fs::write(dir.path().join(".agentflow/output/graph/meta.json"), "{}").unwrap();
        fs::write(dir.path().join("target/generated.rs"), "ignored").unwrap();
        let after = project_fingerprint(dir.path()).unwrap();

        assert_eq!(before, after);
    }

    #[test]
    fn watcher_refreshes_graph_after_source_file_change() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("a.rs"), "pub struct A {}\n").unwrap();
        index_project_graph(dir.path()).unwrap();
        ensure_graph_watcher(dir.path()).unwrap();

        fs::write(dir.path().join("b.rs"), "pub struct B {}\n").unwrap();

        let mut status = load_project_graph_status(dir.path()).unwrap();
        for _ in 0..20 {
            if status.file_count == 2 {
                break;
            }
            thread::sleep(Duration::from_millis(150));
            status = load_project_graph_status(dir.path()).unwrap();
        }
        assert_eq!(status.status, crate::model::GraphStatus::Ready);
        assert_eq!(status.file_count, 2);
    }
}
