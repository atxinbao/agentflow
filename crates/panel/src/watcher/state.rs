use crate::{
    manager::{index_project_panel, unix_timestamp_seconds},
    model::PanelWatcherSnapshot,
    watcher::{debounce::DEBOUNCE_MS, filter::IGNORED_ENTRIES, native::native_backend_name},
};
use std::{
    collections::HashMap,
    path::Path,
    sync::{Mutex, OnceLock},
};

#[derive(Debug, Clone)]
pub(crate) struct WatcherState {
    pub(crate) status: String,
    pub(crate) backend: String,
    pub(crate) recursive: bool,
    pub(crate) debounce_ms: u64,
    pub(crate) ignored_path_count: usize,
    pub(crate) last_event_at: Option<u64>,
    pub(crate) last_event_kind: Option<String>,
    pub(crate) last_error: Option<String>,
}

static WATCHERS: OnceLock<Mutex<HashMap<String, WatcherState>>> = OnceLock::new();

pub(crate) fn ensure_starting_state(root: &Path, root_key: &str) -> (PanelWatcherSnapshot, bool) {
    let registry = WATCHERS.get_or_init(|| Mutex::new(HashMap::new()));
    let mut watchers = registry.lock().expect("panel watcher registry poisoned");
    if let Some(state) = watchers.get(root_key) {
        return (snapshot(root, state), false);
    }

    watchers.insert(root_key.to_string(), starting_state());
    (
        snapshot(
            root,
            watchers.get(root_key).expect("watcher was just inserted"),
        ),
        true,
    )
}

pub(crate) fn watcher_state(project_root: impl AsRef<Path>) -> Option<WatcherState> {
    let root = project_root.as_ref().canonicalize().ok()?;
    let root_key = root.display().to_string();
    WATCHERS
        .get()
        .and_then(|registry| registry.lock().ok()?.get(&root_key).cloned())
}

pub(crate) fn refresh_panel(root_key: &str, root: &Path, backend: &str, event_kind: &str) {
    record_event(root_key, "indexing", event_kind, None);
    match index_project_panel(root) {
        Ok(_) => {
            update_state(root_key, |state| {
                state.status = "native".to_string();
                state.backend = if backend == "native" {
                    native_backend_name().to_string()
                } else {
                    backend.to_string()
                };
                state.last_error = None;
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

pub(crate) fn record_failure(root_key: &str, reason: String) {
    update_state(root_key, |state| {
        state.status = "failed".to_string();
        state.recursive = true;
        state.last_error = Some(reason);
    });
}

pub(crate) fn record_event(root_key: &str, status: &str, event_kind: &str, error: Option<String>) {
    update_state(root_key, |state| {
        state.status = status.to_string();
        state.last_event_at = Some(unix_timestamp_seconds());
        state.last_event_kind = Some(event_kind.to_string());
        if let Some(error) = error {
            state.last_error = Some(error);
        }
    });
}

pub(crate) fn update_state(root_key: &str, update: impl FnOnce(&mut WatcherState)) {
    let registry = WATCHERS.get_or_init(|| Mutex::new(HashMap::new()));
    if let Ok(mut watchers) = registry.lock() {
        let state = watchers
            .entry(root_key.to_string())
            .or_insert_with(starting_state);
        update(state);
    }
}

pub(crate) fn platform_name() -> &'static str {
    std::env::consts::OS
}

fn starting_state() -> WatcherState {
    WatcherState {
        status: "starting".to_string(),
        backend: "unknown".to_string(),
        recursive: true,
        debounce_ms: DEBOUNCE_MS,
        ignored_path_count: IGNORED_ENTRIES.len(),
        last_event_at: None,
        last_event_kind: None,
        last_error: None,
    }
}

fn snapshot(root: &Path, state: &WatcherState) -> PanelWatcherSnapshot {
    PanelWatcherSnapshot {
        version: "panel-watcher.v1".to_string(),
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
