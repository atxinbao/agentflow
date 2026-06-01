use notify::{Event, EventKind};
use std::path::{Component, Path};

pub(crate) const IGNORED_ENTRIES: &[&str] = &[
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

pub(crate) fn relevant_event_kind(root: &Path, event: &Event) -> Option<String> {
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

pub(crate) fn is_supported_event_kind(kind: &EventKind) -> bool {
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

pub(crate) fn should_skip_entry(file_name: &str) -> bool {
    IGNORED_ENTRIES.contains(&file_name)
}
