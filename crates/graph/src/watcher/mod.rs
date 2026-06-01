use crate::{
    model::{GraphWatcherDetail, GraphWatcherSnapshot},
    watcher::state::WatcherState,
};
use anyhow::{Context, Result};
use std::{path::Path, path::PathBuf, thread};

mod debounce;
mod fallback;
mod filter;
mod native;
mod state;

#[cfg(test)]
mod tests;

pub fn ensure_graph_watcher(project_root: impl AsRef<Path>) -> Result<GraphWatcherSnapshot> {
    let root = project_root
        .as_ref()
        .canonicalize()
        .with_context(|| format!("canonicalize {}", project_root.as_ref().display()))?;
    let root_key = root.display().to_string();
    let (snapshot, inserted) = state::ensure_starting_state(&root, &root_key);

    if inserted {
        thread::spawn(move || run_watcher(root));
    }

    Ok(snapshot)
}

pub(crate) fn watcher_status(project_root: impl AsRef<Path>) -> Option<String> {
    state::watcher_state(project_root).map(|state| state.status)
}

pub(crate) fn watcher_backend(project_root: impl AsRef<Path>) -> Option<String> {
    state::watcher_state(project_root).map(|state| state.backend)
}

pub(crate) fn watcher_detail(project_root: impl AsRef<Path>) -> Option<GraphWatcherDetail> {
    state::watcher_state(project_root).map(|state| GraphWatcherDetail {
        platform: state::platform_name().to_string(),
        recursive: state.recursive,
        ignored_path_count: state.ignored_path_count,
        last_event_at: state.last_event_at,
        last_event_kind: state.last_event_kind,
        last_error: state.last_error,
    })
}

fn run_watcher(root: PathBuf) {
    let root_key = root.display().to_string();
    if fallback::forced_fallback() {
        let reason = "用户或测试显式启用 fingerprint fallback。".to_string();
        state::record_fallback(&root_key, reason);
        fallback::run_fingerprint_watcher(root);
        return;
    }

    if let Err(error) = native::run_native_watcher(root.clone()) {
        state::record_fallback(
            &root_key,
            format!("OS native watcher 不可用，已降级到 fingerprint fallback：{error}"),
        );
        fallback::run_fingerprint_watcher(root);
    }
}

fn _assert_watcher_state_is_cloneable(_: WatcherState) {}
