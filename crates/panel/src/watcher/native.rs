use crate::watcher::{debounce::DEBOUNCE_MS, filter::relevant_event_kind, state};
use anyhow::Result;
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::{
    path::PathBuf,
    sync::mpsc::{self, RecvTimeoutError},
    time::Duration,
};

pub(crate) fn run_native_watcher(root: PathBuf) -> Result<()> {
    let root_key = root.display().to_string();
    let (sender, receiver) = mpsc::channel::<notify::Result<Event>>();
    let mut watcher: RecommendedWatcher = notify::recommended_watcher(move |event| {
        let _ = sender.send(event);
    })?;
    watcher.watch(&root, RecursiveMode::Recursive)?;
    state::update_state(&root_key, |state| {
        state.status = "native".to_string();
        state.backend = native_backend_name().to_string();
        state.recursive = true;
        state.last_error = None;
    });

    loop {
        let event = match receiver.recv() {
            Ok(Ok(event)) => event,
            Ok(Err(error)) => {
                state::update_state(&root_key, |state| {
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
        state::record_event(&root_key, "debouncing", &kind, None);
        let mut saw_relevant_event = true;
        let mut last_kind = kind;

        loop {
            match receiver.recv_timeout(Duration::from_millis(DEBOUNCE_MS)) {
                Ok(Ok(next_event)) => {
                    if let Some(kind) = relevant_event_kind(&root, &next_event) {
                        saw_relevant_event = true;
                        last_kind = kind;
                        state::record_event(&root_key, "debouncing", &last_kind, None);
                    }
                }
                Ok(Err(error)) => {
                    state::update_state(&root_key, |state| {
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
            state::refresh_panel(&root_key, &root, "native", &last_kind);
        }
    }
}

pub(crate) fn native_backend_name() -> &'static str {
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
