use super::*;
use crate::manager::{index_project_panel, load_project_panel_status};
use notify::{
    event::{CreateKind, DataChange, ModifyKind, RemoveKind, RenameMode},
    EventKind,
};
use std::{fs, thread, time::Duration};
use tempfile::tempdir;

#[test]
fn panel_event_filter_ignores_runtime_and_build_paths() {
    let dir = tempdir().unwrap();
    for ignored in [
        ".agentflow/panel/manifest.json",
        ".git/index",
        "target/debug/app",
        "node_modules/pkg/index.js",
        "dist/app.js",
        ".DS_Store",
    ] {
        assert!(
            filter::should_ignore_panel_event(dir.path(), &dir.path().join(ignored)),
            "expected ignored path {ignored}"
        );
    }
    assert!(!filter::should_ignore_panel_event(
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
            filter::is_supported_event_kind(&kind),
            "event kind should refresh panel: {kind:?}"
        );
    }
}

#[test]
fn native_watcher_snapshot_reports_backend_and_recursive_mode() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("a.rs"), "pub struct A {}\n").unwrap();

    let snapshot = ensure_panel_watcher(dir.path()).unwrap();

    assert_eq!(snapshot.status, "starting");
    assert!(snapshot.recursive);
    assert_eq!(snapshot.ignored_path_count, filter::IGNORED_ENTRIES.len());
}

#[test]
fn watcher_native_event_refreshes_panel() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("a.rs"), "pub struct A {}\n").unwrap();
    index_project_panel(dir.path()).unwrap();
    ensure_panel_watcher(dir.path()).unwrap();
    for _ in 0..30 {
        if watcher_status(dir.path()).as_deref() == Some("native") {
            break;
        }
        thread::sleep(Duration::from_millis(100));
    }

    fs::write(dir.path().join("b.rs"), "pub struct B {}\n").unwrap();

    let mut status = load_project_panel_status(dir.path()).unwrap();
    for _ in 0..60 {
        let watcher_status = watcher_status(dir.path());
        if status.file_count == 2
            && status.status == crate::model::PanelStatus::Ready
            && matches!(
                watcher_status.as_deref(),
                Some("native") | Some("debouncing")
            )
        {
            break;
        }
        thread::sleep(Duration::from_millis(150));
        status = load_project_panel_status(dir.path()).unwrap();
    }
    assert_eq!(status.status, crate::model::PanelStatus::Ready);
    assert_eq!(status.file_count, 2);
    assert!(matches!(
        status.watcher_status.as_deref(),
        Some("native") | Some("debouncing")
    ));
    assert!(matches!(
        status.watcher_backend.as_deref(),
        Some("fsevents")
            | Some("read_directory_changes_w")
            | Some("inotify")
            | Some("recommended_native")
    ));
}
