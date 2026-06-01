use super::*;
use crate::manager::{index_project_graph, load_project_graph_status};
use notify::{
    event::{CreateKind, DataChange, ModifyKind, RemoveKind, RenameMode},
    EventKind,
};
use std::{fs, thread, time::Duration};
use tempfile::tempdir;

#[test]
fn fingerprint_ignores_agentflow_and_target_runtime_files() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("src.rs"), "fn a() {}\n").unwrap();
    fs::create_dir_all(dir.path().join(".agentflow/output/graph")).unwrap();
    fs::create_dir_all(dir.path().join("target")).unwrap();

    let before = fallback::project_fingerprint(dir.path()).unwrap();
    fs::write(dir.path().join(".agentflow/output/graph/meta.json"), "{}").unwrap();
    fs::write(dir.path().join("target/generated.rs"), "ignored").unwrap();
    let after = fallback::project_fingerprint(dir.path()).unwrap();

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
            filter::should_ignore_graph_event(dir.path(), &dir.path().join(ignored)),
            "expected ignored path {ignored}"
        );
    }
    assert!(!filter::should_ignore_graph_event(
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
    assert_eq!(snapshot.ignored_path_count, filter::IGNORED_ENTRIES.len());
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

    state::record_fallback(&root_key, "forced fallback".to_string());
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
    state::record_fallback(&root_key, "forced fallback".to_string());

    let status = index_project_graph(dir.path()).unwrap();

    assert_eq!(status.status, crate::model::GraphStatus::Degraded);
    assert_eq!(status.watcher_status.as_deref(), Some("fallback"));
    assert_eq!(status.watcher_backend.as_deref(), Some("fingerprint"));
    assert!(status
        .degraded_reasons
        .iter()
        .any(|reason| reason.contains("forced fallback")));
}
