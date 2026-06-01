use crate::{
    manager::{index_project_graph, unix_timestamp_seconds},
    model::{GraphWatcherDetail, GraphWatcherSnapshot},
};
use anyhow::{Context, Result};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    fs,
    hash::{Hash, Hasher},
    path::{Component, Path, PathBuf},
    sync::{
        mpsc::{self, RecvTimeoutError},
        Mutex, OnceLock,
    },
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

const IGNORED_ENTRIES: &[&str] = &[
    ".git",
    ".agentflow",
    "node_modules",
    "target",
    "dist",
    "build",
    "coverage",
    ".cache",
    "vendor",
    ".idea",
    ".vscode",
    ".DS_Store",
];

#[derive(Debug, Clone)]
struct WatcherState {
    status: String,
    backend: String,
    recursive: bool,
    debounce_ms: u64,
    ignored_path_count: usize,
    last_event_at: Option<u64>,
    last_event_kind: Option<String>,
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
                status: "starting".to_string(),
                backend: "unknown".to_string(),
                recursive: true,
                debounce_ms: DEBOUNCE_MS,
                ignored_path_count: IGNORED_ENTRIES.len(),
                last_event_at: None,
                last_event_kind: None,
                last_error: None,
            },
        );
    }

    thread::spawn(move || run_watcher(root));
    let watchers = registry.lock().expect("graph watcher registry poisoned");
    Ok(snapshot(
        Path::new(&root_key),
        watchers.get(&root_key).expect("watcher was just inserted"),
    ))
}

pub(crate) fn watcher_status(project_root: impl AsRef<Path>) -> Option<String> {
    watcher_state(project_root).map(|state| state.status)
}

pub(crate) fn watcher_backend(project_root: impl AsRef<Path>) -> Option<String> {
    watcher_state(project_root).map(|state| state.backend)
}

pub(crate) fn watcher_detail(project_root: impl AsRef<Path>) -> Option<GraphWatcherDetail> {
    watcher_state(project_root).map(|state| GraphWatcherDetail {
        platform: platform_name().to_string(),
        recursive: state.recursive,
        ignored_path_count: state.ignored_path_count,
        last_event_at: state.last_event_at,
        last_event_kind: state.last_event_kind,
        last_error: state.last_error,
    })
}

fn watcher_state(project_root: impl AsRef<Path>) -> Option<WatcherState> {
    let root = project_root.as_ref().canonicalize().ok()?;
    let root_key = root.display().to_string();
    WATCHERS
        .get()
        .and_then(|registry| registry.lock().ok()?.get(&root_key).cloned())
}

fn run_watcher(root: PathBuf) {
    let root_key = root.display().to_string();
    if forced_fallback() {
        let reason = "用户或测试显式启用 fingerprint fallback。".to_string();
        record_fallback(&root_key, reason);
        run_fingerprint_watcher(root);
        return;
    }

    if let Err(error) = run_native_watcher(root.clone()) {
        record_fallback(
            &root_key,
            format!("OS native watcher 不可用，已降级到 fingerprint fallback：{error}"),
        );
        run_fingerprint_watcher(root);
    }
}

fn run_native_watcher(root: PathBuf) -> Result<()> {
    let root_key = root.display().to_string();
    let (sender, receiver) = mpsc::channel::<notify::Result<Event>>();
    let mut watcher: RecommendedWatcher = notify::recommended_watcher(move |event| {
        let _ = sender.send(event);
    })?;
    watcher.watch(&root, RecursiveMode::Recursive)?;
    update_state(&root_key, |state| {
        state.status = "native".to_string();
        state.backend = native_backend_name().to_string();
        state.recursive = true;
        state.last_error = None;
    });

    loop {
        let event = match receiver.recv() {
            Ok(Ok(event)) => event,
            Ok(Err(error)) => {
                update_state(&root_key, |state| {
                    state.status = "failed".to_string();
                    state.last_error = Some(error.to_string());
                });
                continue;
            }
            Err(error) => return Err(error.into()),
        };

        let Some(kind) = relevant_event_kind(&root, &event) else {
            continue;
        };
        record_event(&root_key, "debouncing", &kind, None);
        let mut saw_relevant_event = true;
        let mut last_kind = kind;

        loop {
            match receiver.recv_timeout(Duration::from_millis(DEBOUNCE_MS)) {
                Ok(Ok(next_event)) => {
                    if let Some(kind) = relevant_event_kind(&root, &next_event) {
                        saw_relevant_event = true;
                        last_kind = kind;
                        record_event(&root_key, "debouncing", &last_kind, None);
                    }
                }
                Ok(Err(error)) => {
                    update_state(&root_key, |state| {
                        state.status = "failed".to_string();
                        state.last_error = Some(error.to_string());
                    });
                }
                Err(RecvTimeoutError::Timeout) => break,
                Err(RecvTimeoutError::Disconnected) => {
                    return Err(anyhow::anyhow!("native watcher event channel disconnected"));
                }
            }
        }

        if saw_relevant_event {
            refresh_graph(&root_key, &root, "native", &last_kind);
        }
    }
}

fn run_fingerprint_watcher(root: PathBuf) {
    let root_key = root.display().to_string();
    let mut last_fingerprint = match project_fingerprint(&root) {
        Ok(value) => value,
        Err(error) => {
            update_state(&root_key, |state| {
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
            update_state(&root_key, |state| {
                state.status = "failed".to_string();
                state.last_error = Some("无法读取项目文件变化。".to_string());
            });
            continue;
        };
        if current_fingerprint == last_fingerprint {
            continue;
        }

        record_event(&root_key, "debouncing", "fingerprint_change", None);
        thread::sleep(Duration::from_millis(DEBOUNCE_MS));
        let Ok(debounced_fingerprint) = project_fingerprint(&root) else {
            update_state(&root_key, |state| {
                state.status = "failed".to_string();
                state.last_error = Some("无法完成文件变化合并。".to_string());
            });
            continue;
        };
        if debounced_fingerprint == last_fingerprint {
            update_state(&root_key, |state| {
                state.status = "fallback".to_string();
                state.backend = "fingerprint".to_string();
            });
            continue;
        }

        refresh_graph(&root_key, &root, "fingerprint", "fingerprint_change");
        last_fingerprint = debounced_fingerprint;
    }
}

fn refresh_graph(root_key: &str, root: &Path, backend: &str, event_kind: &str) {
    record_event(root_key, "indexing", event_kind, None);
    match index_project_graph(root) {
        Ok(_) => {
            update_state(root_key, |state| {
                state.status = if backend == "fingerprint" {
                    "fallback".to_string()
                } else {
                    "native".to_string()
                };
                state.backend = if backend == "fingerprint" {
                    "fingerprint".to_string()
                } else {
                    native_backend_name().to_string()
                };
                state.last_error = if backend == "fingerprint" {
                    state.last_error.clone()
                } else {
                    None
                };
            });
        }
        Err(error) => {
            update_state(root_key, |state| {
                state.status = "failed".to_string();
                state.backend = backend.to_string();
                state.last_error = Some(error.to_string());
            });
        }
    }
}

fn record_fallback(root_key: &str, reason: String) {
    update_state(root_key, |state| {
        state.status = "fallback".to_string();
        state.backend = "fingerprint".to_string();
        state.recursive = true;
        state.last_error = Some(reason);
    });
}

fn record_event(root_key: &str, status: &str, event_kind: &str, error: Option<String>) {
    update_state(root_key, |state| {
        state.status = status.to_string();
        state.last_event_at = Some(unix_timestamp_seconds());
        state.last_event_kind = Some(event_kind.to_string());
        if let Some(error) = error {
            state.last_error = Some(error);
        }
    });
}

fn update_state(root_key: &str, update: impl FnOnce(&mut WatcherState)) {
    let registry = WATCHERS.get_or_init(|| Mutex::new(HashMap::new()));
    if let Ok(mut watchers) = registry.lock() {
        let state = watchers
            .entry(root_key.to_string())
            .or_insert_with(|| WatcherState {
                status: "missing".to_string(),
                backend: "unknown".to_string(),
                recursive: true,
                debounce_ms: DEBOUNCE_MS,
                ignored_path_count: IGNORED_ENTRIES.len(),
                last_event_at: None,
                last_event_kind: None,
                last_error: None,
            });
        update(state);
    }
}

fn snapshot(root: &Path, state: &WatcherState) -> GraphWatcherSnapshot {
    GraphWatcherSnapshot {
        version: "graph-watcher.v2".to_string(),
        project_root: root.display().to_string(),
        status: state.status.clone(),
        backend: state.backend.clone(),
        recursive: state.recursive,
        debounce_ms: state.debounce_ms,
        ignored_path_count: state.ignored_path_count,
        last_event_at: state.last_event_at,
        last_event_kind: state.last_event_kind.clone(),
        last_error: state.last_error.clone(),
    }
}

fn relevant_event_kind(root: &Path, event: &Event) -> Option<String> {
    if !is_supported_event_kind(&event.kind) {
        return None;
    }
    if event
        .paths
        .iter()
        .all(|path| should_ignore_graph_event(root, path))
    {
        return None;
    }
    Some(event_kind_label(&event.kind).to_string())
}

fn is_supported_event_kind(kind: &EventKind) -> bool {
    matches!(
        kind,
        EventKind::Create(_)
            | EventKind::Modify(_)
            | EventKind::Remove(_)
            | EventKind::Any
            | EventKind::Other
    )
}

fn event_kind_label(kind: &EventKind) -> &'static str {
    match kind {
        EventKind::Create(_) => "create",
        EventKind::Modify(_) => "modify",
        EventKind::Remove(_) => "remove",
        EventKind::Any => "any",
        EventKind::Other => "other",
        _ => "unknown",
    }
}

pub(crate) fn should_ignore_graph_event(root: &Path, path: &Path) -> bool {
    let relative = if path.is_absolute() {
        path.strip_prefix(root).unwrap_or(path)
    } else {
        path
    };
    relative.components().any(|component| {
        let Component::Normal(value) = component else {
            return false;
        };
        should_skip_entry(value.to_str().unwrap_or(""))
    })
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

fn should_skip_entry(file_name: &str) -> bool {
    IGNORED_ENTRIES.contains(&file_name)
}

fn forced_fallback() -> bool {
    std::env::var("AGENTFLOW_GRAPH_WATCHER_FORCE_FALLBACK")
        .map(|value| matches!(value.as_str(), "1" | "true" | "yes"))
        .unwrap_or(false)
}

fn native_backend_name() -> &'static str {
    #[cfg(target_os = "macos")]
    {
        "fsevents"
    }
    #[cfg(target_os = "windows")]
    {
        "read_directory_changes_w"
    }
    #[cfg(target_os = "linux")]
    {
        "inotify"
    }
    #[cfg(all(
        not(target_os = "macos"),
        not(target_os = "windows"),
        not(target_os = "linux")
    ))]
    {
        "recommended_native"
    }
}

fn platform_name() -> &'static str {
    std::env::consts::OS
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manager::{index_project_graph, load_project_graph_status};
    use notify::event::{CreateKind, DataChange, ModifyKind, RemoveKind, RenameMode};
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
    fn graph_event_filter_ignores_runtime_and_build_paths() {
        let dir = tempdir().unwrap();
        for ignored in [
            ".agentflow/output/graph/meta.json",
            ".git/index",
            "target/debug/app",
            "node_modules/pkg/index.js",
            "dist/app.js",
            ".DS_Store",
        ] {
            assert!(
                should_ignore_graph_event(dir.path(), &dir.path().join(ignored)),
                "expected ignored path {ignored}"
            );
        }
        assert!(!should_ignore_graph_event(
            dir.path(),
            &dir.path().join("src/lib.rs")
        ));
    }

    #[test]
    fn native_event_kinds_cover_create_modify_remove_and_rename() {
        for kind in [
            EventKind::Create(CreateKind::File),
            EventKind::Modify(ModifyKind::Data(DataChange::Any)),
            EventKind::Remove(RemoveKind::File),
            EventKind::Modify(ModifyKind::Name(RenameMode::Both)),
        ] {
            assert!(
                is_supported_event_kind(&kind),
                "event kind should refresh graph: {kind:?}"
            );
        }
    }

    #[test]
    fn native_watcher_snapshot_reports_backend_and_recursive_mode() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("a.rs"), "pub struct A {}\n").unwrap();

        let snapshot = ensure_graph_watcher(dir.path()).unwrap();

        assert_eq!(snapshot.status, "starting");
        assert!(snapshot.recursive);
        assert_eq!(snapshot.ignored_path_count, IGNORED_ENTRIES.len());
    }

    #[test]
    fn watcher_native_event_refreshes_graph() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("a.rs"), "pub struct A {}\n").unwrap();
        index_project_graph(dir.path()).unwrap();
        ensure_graph_watcher(dir.path()).unwrap();
        for _ in 0..30 {
            if watcher_status(dir.path()).as_deref() == Some("native") {
                break;
            }
            thread::sleep(Duration::from_millis(100));
        }

        fs::write(dir.path().join("b.rs"), "pub struct B {}\n").unwrap();

        let mut status = load_project_graph_status(dir.path()).unwrap();
        for _ in 0..60 {
            if status.file_count == 2 && watcher_status(dir.path()).as_deref() == Some("native") {
                break;
            }
            thread::sleep(Duration::from_millis(150));
            status = load_project_graph_status(dir.path()).unwrap();
        }
        assert_eq!(status.status, crate::model::GraphStatus::Ready);
        assert_eq!(status.file_count, 2);
        assert_eq!(status.watcher_status.as_deref(), Some("native"));
        assert!(matches!(
            status.watcher_backend.as_deref(),
            Some("fsevents")
                | Some("read_directory_changes_w")
                | Some("inotify")
                | Some("recommended_native")
        ));
    }

    #[test]
    fn fallback_snapshot_is_marked_degraded() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("a.rs"), "pub struct A {}\n").unwrap();
        let root_key = dir.path().canonicalize().unwrap().display().to_string();

        record_fallback(&root_key, "forced fallback".to_string());
        let snapshot = ensure_graph_watcher(dir.path()).unwrap();

        assert_eq!(snapshot.status, "fallback");
        assert_eq!(snapshot.backend, "fingerprint");
        assert_eq!(snapshot.last_error.as_deref(), Some("forced fallback"));
    }

    #[test]
    fn fallback_watcher_marks_graph_status_degraded() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("a.rs"), "pub struct A {}\n").unwrap();
        let root_key = dir.path().canonicalize().unwrap().display().to_string();
        record_fallback(&root_key, "forced fallback".to_string());

        let status = index_project_graph(dir.path()).unwrap();

        assert_eq!(status.status, crate::model::GraphStatus::Degraded);
        assert_eq!(status.watcher_status.as_deref(), Some("fallback"));
        assert_eq!(status.watcher_backend.as_deref(), Some("fingerprint"));
        assert!(status
            .degraded_reasons
            .iter()
            .any(|reason| reason.contains("forced fallback")));
    }
}
