use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::Serialize;
use std::{
    collections::{BTreeSet, HashSet},
    path::{Component, Path, PathBuf},
    sync::{mpsc, Mutex, OnceLock},
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Emitter};

pub(crate) const AGENTFLOW_WORKSPACE_CHANGED_EVENT: &str = "agentflow-workspace-changed";
const WATCHER_VERSION: &str = "agentflow-workspace-watcher.v1";
const DEBOUNCE_MS: u64 = 600;

static WATCHED_ROOTS: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AgentflowWorkspaceWatcherSnapshot {
    version: String,
    project_root: String,
    agentflow_path: String,
    status: String,
    backend: String,
    recursive: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AgentflowWorkspaceChangedEvent {
    version: String,
    project_root: String,
    agentflow_path: String,
    changed_areas: Vec<String>,
    paths: Vec<String>,
    event_kind: String,
    watcher_status: String,
    watcher_backend: String,
    updated_at: u64,
}

#[tauri::command]
pub(crate) fn start_agentflow_workspace_watcher(
    project_root: String,
    app: AppHandle,
) -> Result<AgentflowWorkspaceWatcherSnapshot, String> {
    let root = PathBuf::from(&project_root)
        .canonicalize()
        .map_err(|error| format!("canonicalize project root: {error}"))?;
    let agentflow_path = root.join(".agentflow");
    if !agentflow_path.is_dir() {
        return Err(format!("{} does not exist", agentflow_path.display()));
    }

    let root_key = root.display().to_string();
    let inserted = {
        let mut watched_roots = WATCHED_ROOTS
            .get_or_init(|| Mutex::new(HashSet::new()))
            .lock()
            .map_err(|_| "agentflow watcher registry poisoned".to_string())?;
        watched_roots.insert(root_key.clone())
    };

    if inserted {
        thread::spawn(move || {
            if let Err(error) = run_agentflow_watcher(root, app) {
                eprintln!("agentflow workspace watcher stopped: {error}");
            }
        });
    }

    Ok(AgentflowWorkspaceWatcherSnapshot {
        version: WATCHER_VERSION.to_string(),
        project_root: root_key,
        agentflow_path: agentflow_path.display().to_string(),
        status: "watching".to_string(),
        backend: native_backend_name().to_string(),
        recursive: true,
    })
}

fn run_agentflow_watcher(root: PathBuf, app: AppHandle) -> notify::Result<()> {
    let agentflow_path = root.join(".agentflow");
    let (sender, receiver) = mpsc::channel::<notify::Result<Event>>();
    let mut watcher: RecommendedWatcher = notify::recommended_watcher(move |event| {
        let _ = sender.send(event);
    })?;
    watcher.watch(&agentflow_path, RecursiveMode::Recursive)?;

    loop {
        let event = match receiver.recv() {
            Ok(Ok(event)) => event,
            Ok(Err(error)) => {
                eprintln!("agentflow workspace watcher event error: {error}");
                continue;
            }
            Err(error) => return Err(notify::Error::generic(&error.to_string())),
        };

        let Some(mut pending) = PendingAgentflowChange::from_event(&root, &event) else {
            continue;
        };

        loop {
            match receiver.recv_timeout(Duration::from_millis(DEBOUNCE_MS)) {
                Ok(Ok(next_event)) => {
                    if let Some(next_pending) =
                        PendingAgentflowChange::from_event(&root, &next_event)
                    {
                        pending.merge(next_pending);
                    }
                }
                Ok(Err(error)) => {
                    eprintln!("agentflow workspace watcher event error: {error}");
                }
                Err(mpsc::RecvTimeoutError::Timeout) => break,
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    return Err(notify::Error::generic(
                        "agentflow workspace watcher event channel disconnected",
                    ));
                }
            }
        }

        let payload = pending.into_event(&root, &agentflow_path);
        let _ = app.emit(AGENTFLOW_WORKSPACE_CHANGED_EVENT, payload);
    }
}

#[derive(Debug, Clone)]
struct PendingAgentflowChange {
    changed_areas: BTreeSet<String>,
    paths: BTreeSet<String>,
    event_kinds: BTreeSet<String>,
}

impl PendingAgentflowChange {
    fn from_event(root: &Path, event: &Event) -> Option<Self> {
        if !is_supported_event_kind(&event.kind) {
            return None;
        }

        let mut pending = Self {
            changed_areas: BTreeSet::new(),
            paths: BTreeSet::new(),
            event_kinds: BTreeSet::new(),
        };
        pending
            .event_kinds
            .insert(event_kind_label(&event.kind).to_string());

        for path in &event.paths {
            let Some(relative) = agentflow_relative_path(root, path) else {
                continue;
            };
            if should_ignore_agentflow_path(&relative) {
                continue;
            }
            let Some(area) = agentflow_area(&relative) else {
                continue;
            };
            pending.changed_areas.insert(area.to_string());
            pending
                .paths
                .insert(relative.to_string_lossy().replace('\\', "/"));
        }

        if pending.changed_areas.is_empty() {
            None
        } else {
            Some(pending)
        }
    }

    fn merge(&mut self, other: Self) {
        self.changed_areas.extend(other.changed_areas);
        self.paths.extend(other.paths);
        self.event_kinds.extend(other.event_kinds);
    }

    fn into_event(self, root: &Path, agentflow_path: &Path) -> AgentflowWorkspaceChangedEvent {
        AgentflowWorkspaceChangedEvent {
            version: WATCHER_VERSION.to_string(),
            project_root: root.display().to_string(),
            agentflow_path: agentflow_path.display().to_string(),
            changed_areas: self.changed_areas.into_iter().collect(),
            paths: self.paths.into_iter().collect(),
            event_kind: self.event_kinds.into_iter().collect::<Vec<_>>().join(","),
            watcher_status: "native".to_string(),
            watcher_backend: native_backend_name().to_string(),
            updated_at: unix_timestamp_seconds(),
        }
    }
}

fn agentflow_relative_path(root: &Path, path: &Path) -> Option<PathBuf> {
    let relative = if path.is_absolute() {
        path.strip_prefix(root).ok()?.to_path_buf()
    } else {
        path.to_path_buf()
    };

    if relative.components().next() == Some(Component::Normal(".agentflow".as_ref())) {
        Some(relative)
    } else {
        None
    }
}

fn agentflow_area(relative: &Path) -> Option<&str> {
    let mut components = relative.components();
    match components.next()? {
        Component::Normal(value) if value == ".agentflow" => {}
        _ => return None,
    }
    let Component::Normal(area) = components.next()? else {
        return None;
    };
    area.to_str()
}

fn should_ignore_agentflow_path(relative: &Path) -> bool {
    let parts = relative
        .components()
        .filter_map(|component| {
            let Component::Normal(value) = component else {
                return None;
            };
            value.to_str()
        })
        .collect::<Vec<_>>();

    if parts
        .iter()
        .any(|part| matches!(*part, ".DS_Store" | "tmp" | ".tmp"))
    {
        return true;
    }

    matches!(
        parts.as_slice(),
        [".agentflow", "output", "logs", ..]
            | [".agentflow", "output", "tmp", ..]
            | [".agentflow", "output", "cache", ..]
            | [".agentflow", "panel", "index", ..]
            | [".agentflow", "panel", "snapshots", ..]
    )
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

fn unix_timestamp_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use notify::{event::ModifyKind, Event};
    use tempfile::tempdir;

    #[test]
    fn classifies_agentflow_paths_by_top_level_area() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        let event = Event::new(EventKind::Modify(ModifyKind::Data(
            notify::event::DataChange::Content,
        )))
        .add_path(root.join(".agentflow/input/issues/AF-001.json"))
        .add_path(root.join(".agentflow/output/release/run-001/release.json"))
        .add_path(root.join("src/main.rs"));

        let pending = PendingAgentflowChange::from_event(root, &event).unwrap();

        assert_eq!(
            pending.changed_areas.into_iter().collect::<Vec<_>>(),
            vec!["input".to_string(), "output".to_string()]
        );
    }

    #[test]
    fn ignores_high_churn_agentflow_paths() {
        assert!(should_ignore_agentflow_path(Path::new(
            ".agentflow/output/logs/watch.log"
        )));
        assert!(should_ignore_agentflow_path(Path::new(
            ".agentflow/output/tmp/a.json"
        )));
        assert!(should_ignore_agentflow_path(Path::new(
            ".agentflow/panel/index/panel.db"
        )));
        assert!(should_ignore_agentflow_path(Path::new(
            ".agentflow/panel/snapshots/latest.json"
        )));
        assert!(!should_ignore_agentflow_path(Path::new(
            ".agentflow/input/issues/AF-001.json"
        )));
    }
}
