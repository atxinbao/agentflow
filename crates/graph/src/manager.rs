use crate::{
    db,
    model::{GraphIndex, GraphManifestSnapshot, GraphStatus, GraphStatusSnapshot},
    protection::check_graph_git_protection,
    scanner::scan_project,
    watcher,
};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

const PANEL_DIR: &str = ".agentflow/panel";
const PANEL_DB_RELATIVE_PATH: &str = ".agentflow/panel/index/panel.db";
const PANEL_MANIFEST_RELATIVE_PATH: &str = ".agentflow/panel/manifest.json";
const LEGACY_GRAPH_OUTPUT_DIR: &str = ".agentflow/output/graph";
const LEGACY_GRAPH_CANONICAL_DIR: &str = ".agentflow/graph";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphPrepareMode {
    Blocking,
    Background,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GraphMeta {
    version: String,
    status: GraphStatus,
    project_root: String,
    graph_db: String,
    updated_at: u64,
    git_head: Option<String>,
    file_count: usize,
    symbol_count: usize,
    relation_count: usize,
    last_index_run_id: Option<String>,
    languages: Vec<String>,
    last_error: Option<String>,
    #[serde(default)]
    degraded_reasons: Vec<String>,
}

#[derive(Debug, Clone)]
struct GraphPaths {
    root: PathBuf,
    graph_dir: PathBuf,
    graph_db: PathBuf,
    meta: PathBuf,
    context_packs: PathBuf,
    search: PathBuf,
    snapshots: PathBuf,
    index: PathBuf,
    file_tree: PathBuf,
    languages: PathBuf,
    symbols: PathBuf,
    relations: PathBuf,
    diagnostics: PathBuf,
    git: PathBuf,
    tests: PathBuf,
    file_index: PathBuf,
    symbol_index: PathBuf,
    content_index: PathBuf,
    legacy_output_dir: PathBuf,
    legacy_canonical_dir: PathBuf,
}

pub fn prepare_project_graph(
    project_root: impl AsRef<Path>,
    mode: GraphPrepareMode,
) -> Result<GraphStatusSnapshot> {
    let paths = prepare_paths(project_root.as_ref())?;
    let project_root_string = paths.root.display().to_string();
    let existing = load_project_graph_status(&paths.root).unwrap_or_else(|_| GraphStatusSnapshot {
        version: "panel-status.v1".to_string(),
        project_root: project_root_string.clone(),
        status: GraphStatus::Missing,
        file_count: 0,
        symbol_count: 0,
        relation_count: 0,
        updated_at: None,
        last_error: None,
        watcher_status: None,
        watcher_backend: None,
        watcher_detail: None,
        preflight_status: Some(preflight_status_label(&GraphStatus::Missing)),
        protection_status: None,
        degraded_reasons: Vec::new(),
    });

    if existing.status == GraphStatus::Ready && !is_stale(&paths)? {
        if mode == GraphPrepareMode::Background {
            let _ = watcher::ensure_graph_watcher(&paths.root);
        }
        return Ok(existing);
    }

    write_meta(&paths, GraphStatus::Indexing, None, None, Vec::new())?;
    if mode == GraphPrepareMode::Background {
        let root = paths.root.clone();
        std::thread::spawn(move || {
            let _ = index_project_graph(root);
        });
        let _ = watcher::ensure_graph_watcher(&paths.root);
        return load_project_graph_status(&paths.root);
    }

    index_project_graph(&paths.root)
}

pub fn index_project_graph(project_root: impl AsRef<Path>) -> Result<GraphStatusSnapshot> {
    let paths = prepare_paths(project_root.as_ref())?;
    let mut connection = db::open_graph_db(&paths.graph_db)?;
    let started_at = unix_timestamp_seconds();
    let run_id = format!("panel-run-{started_at}");
    let git_head = git_head(&paths.root);
    db::insert_index_run_start(
        &connection,
        &run_id,
        started_at,
        &paths.root.display().to_string(),
        git_head.as_deref(),
    )?;

    match scan_project(&paths.root) {
        Ok(index) => {
            db::replace_index(&mut connection, &index)?;
            write_exports(&paths, &index)?;
            let finished_at = unix_timestamp_seconds();
            let protection = check_graph_git_protection(&paths.root).ok();
            let degraded_reasons = protection
                .as_ref()
                .filter(|snapshot| snapshot.status == "warning")
                .map(|snapshot| vec![snapshot.reason.clone()])
                .unwrap_or_default();
            let graph_status = if degraded_reasons.is_empty() {
                GraphStatus::Ready
            } else {
                GraphStatus::Degraded
            };
            db::finish_index_run(
                &connection,
                &run_id,
                finished_at,
                match graph_status {
                    GraphStatus::Ready => "ready",
                    GraphStatus::Degraded => "degraded",
                    _ => "ready",
                },
                Some(&index),
                None,
            )?;
            write_meta(
                &paths,
                graph_status.clone(),
                Some((&run_id, &index, git_head.as_deref())),
                None,
                degraded_reasons,
            )?;
            let mut status =
                db::counts(&connection, &paths.root.display().to_string(), graph_status)?;
            status.updated_at = Some(finished_at);
            status = enrich_status_with_runtime(&paths, status);
            Ok(status)
        }
        Err(error) => {
            let message = error.to_string();
            let finished_at = unix_timestamp_seconds();
            db::finish_index_run(
                &connection,
                &run_id,
                finished_at,
                "failed",
                None,
                Some(&message),
            )?;
            write_meta(
                &paths,
                GraphStatus::Failed,
                None,
                Some(&message),
                Vec::new(),
            )?;
            Ok(GraphStatusSnapshot {
                version: "panel-status.v1".to_string(),
                project_root: paths.root.display().to_string(),
                status: GraphStatus::Failed,
                file_count: 0,
                symbol_count: 0,
                relation_count: 0,
                updated_at: Some(finished_at),
                last_error: Some(message),
                watcher_status: watcher::watcher_status(&paths.root),
                watcher_backend: watcher::watcher_backend(&paths.root),
                watcher_detail: watcher::watcher_detail(&paths.root),
                preflight_status: Some(preflight_status_label(&GraphStatus::Failed)),
                protection_status: check_graph_git_protection(&paths.root)
                    .ok()
                    .map(|snapshot| snapshot.status),
                degraded_reasons: Vec::new(),
            })
        }
    }
}

pub fn load_project_graph_status(project_root: impl AsRef<Path>) -> Result<GraphStatusSnapshot> {
    let paths = graph_paths(project_root.as_ref())?;
    if !paths.meta.is_file() || !paths.graph_db.is_file() {
        return Ok(GraphStatusSnapshot {
            version: "panel-status.v1".to_string(),
            project_root: paths.root.display().to_string(),
            status: GraphStatus::Missing,
            file_count: 0,
            symbol_count: 0,
            relation_count: 0,
            updated_at: None,
            last_error: None,
            watcher_status: watcher::watcher_status(&paths.root),
            watcher_backend: watcher::watcher_backend(&paths.root),
            watcher_detail: watcher::watcher_detail(&paths.root),
            preflight_status: Some(preflight_status_label(&GraphStatus::Missing)),
            protection_status: check_graph_git_protection(&paths.root)
                .ok()
                .map(|snapshot| snapshot.status),
            degraded_reasons: Vec::new(),
        });
    }
    let meta: GraphMeta = serde_json::from_str(&fs::read_to_string(&paths.meta)?)?;
    let mut status = if meta.status == GraphStatus::Ready && is_stale(&paths)? {
        GraphStatus::Stale
    } else {
        meta.status
    };
    let mut degraded_reasons = meta.degraded_reasons;
    let protection_status = check_graph_git_protection(&paths.root).ok();
    if matches!(status, GraphStatus::Ready) {
        if let Some(protection) = &protection_status {
            if protection.status == "warning" {
                status = GraphStatus::Degraded;
                degraded_reasons.push(protection.reason.clone());
            }
        }
    }
    Ok(GraphStatusSnapshot {
        version: "panel-status.v1".to_string(),
        project_root: paths.root.display().to_string(),
        status: status.clone(),
        file_count: meta.file_count,
        symbol_count: meta.symbol_count,
        relation_count: meta.relation_count,
        updated_at: Some(meta.updated_at),
        last_error: meta.last_error,
        watcher_status: watcher::watcher_status(&paths.root),
        watcher_backend: watcher::watcher_backend(&paths.root),
        watcher_detail: watcher::watcher_detail(&paths.root),
        preflight_status: Some(preflight_status_label(&status)),
        protection_status: protection_status.map(|snapshot| snapshot.status),
        degraded_reasons,
    })
}

pub fn load_project_graph_manifest(
    project_root: impl AsRef<Path>,
) -> Result<GraphManifestSnapshot> {
    let paths = graph_paths(project_root.as_ref())?;
    if paths.graph_db.is_file() {
        let connection = db::open_graph_db(&paths.graph_db)?;
        let files = db::fetch_files(&connection)?;
        return Ok(manifest_from_files(&paths.root, &files));
    }
    Ok(GraphManifestSnapshot {
        version: "panel-manifest.v1".to_string(),
        project_root: paths.root.display().to_string(),
        languages: Vec::new(),
        top_level_dirs: Vec::new(),
        important_files: Vec::new(),
        source_files: 0,
        test_files: 0,
        doc_files: 0,
        config_files: 0,
        platforms: Vec::new(),
        entry_points: Vec::new(),
        mobile_components: Vec::new(),
        mobile_configs: Vec::new(),
        mobile_tests: Vec::new(),
    })
}

pub(crate) fn graph_db_path(project_root: impl AsRef<Path>) -> Result<PathBuf> {
    Ok(graph_paths(project_root.as_ref())?.graph_db)
}

pub(crate) fn context_pack_dir(project_root: impl AsRef<Path>) -> Result<PathBuf> {
    Ok(prepare_paths(project_root.as_ref())?.context_packs)
}

fn prepare_paths(project_root: &Path) -> Result<GraphPaths> {
    let paths = graph_paths(project_root)?;
    fs::create_dir_all(&paths.graph_dir)
        .with_context(|| format!("create {}", paths.graph_dir.display()))?;
    fs::create_dir_all(&paths.context_packs)
        .with_context(|| format!("create {}", paths.context_packs.display()))?;
    fs::create_dir_all(&paths.search)
        .with_context(|| format!("create {}", paths.search.display()))?;
    fs::create_dir_all(&paths.snapshots)
        .with_context(|| format!("create {}", paths.snapshots.display()))?;
    fs::create_dir_all(&paths.index)
        .with_context(|| format!("create {}", paths.index.display()))?;
    migrate_legacy_panel_artifacts(&paths)?;
    Ok(paths)
}

fn graph_paths(project_root: &Path) -> Result<GraphPaths> {
    let root = project_root
        .canonicalize()
        .with_context(|| format!("canonicalize {}", project_root.display()))?;
    let graph_dir = root.join(PANEL_DIR);
    let search = graph_dir.join("search");
    let snapshots = graph_dir.join("snapshots");
    let index = graph_dir.join("index");
    Ok(GraphPaths {
        graph_db: root.join(PANEL_DB_RELATIVE_PATH),
        meta: root.join(PANEL_MANIFEST_RELATIVE_PATH),
        context_packs: graph_dir.join("context-packs"),
        file_tree: graph_dir.join("file-tree.json"),
        languages: graph_dir.join("languages.json"),
        symbols: graph_dir.join("symbols.json"),
        relations: graph_dir.join("relations.json"),
        diagnostics: graph_dir.join("diagnostics.json"),
        git: graph_dir.join("git.json"),
        tests: graph_dir.join("tests.json"),
        file_index: search.join("file-index.json"),
        symbol_index: search.join("symbol-index.json"),
        content_index: search.join("content-index.json"),
        legacy_output_dir: root.join(LEGACY_GRAPH_OUTPUT_DIR),
        legacy_canonical_dir: root.join(LEGACY_GRAPH_CANONICAL_DIR),
        root,
        search,
        snapshots,
        index,
        graph_dir,
    })
}

fn migrate_legacy_panel_artifacts(paths: &GraphPaths) -> Result<()> {
    migrate_legacy_graph_dir(paths, &paths.legacy_output_dir, "meta.json", "graph.db")?;
    migrate_legacy_graph_dir(
        paths,
        &paths.legacy_canonical_dir,
        "manifest.json",
        "index/graph.db",
    )?;
    Ok(())
}

fn migrate_legacy_graph_dir(
    paths: &GraphPaths,
    legacy_dir: &Path,
    legacy_manifest_name: &str,
    legacy_db_name: &str,
) -> Result<()> {
    if !legacy_dir.is_dir() {
        return Ok(());
    }

    let legacy_manifest = legacy_dir.join(legacy_manifest_name);
    if legacy_manifest.is_file() && !paths.meta.is_file() {
        fs::copy(&legacy_manifest, &paths.meta).with_context(|| {
            format!(
                "copy legacy manifest {} to {}",
                legacy_manifest.display(),
                paths.meta.display()
            )
        })?;
    }

    let legacy_db = legacy_dir.join(legacy_db_name);
    if legacy_db.is_file() && !paths.graph_db.is_file() {
        if let Some(parent) = paths.graph_db.parent() {
            fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
        }
        fs::copy(&legacy_db, &paths.graph_db).with_context(|| {
            format!(
                "copy legacy db {} to {}",
                legacy_db.display(),
                paths.graph_db.display()
            )
        })?;
    }

    let legacy_context_packs = legacy_dir.join("context-packs");
    if legacy_context_packs.is_dir() {
        copy_legacy_files_if_missing(&legacy_context_packs, &paths.context_packs)?;
    }

    Ok(())
}

fn copy_legacy_files_if_missing(source: &Path, destination: &Path) -> Result<()> {
    fs::create_dir_all(destination).with_context(|| format!("create {}", destination.display()))?;
    for entry in fs::read_dir(source).with_context(|| format!("read {}", source.display()))? {
        let entry = entry?;
        let source_path = entry.path();
        if !source_path.is_file() {
            continue;
        }
        let destination_path = destination.join(entry.file_name());
        if !destination_path.exists() {
            fs::copy(&source_path, &destination_path).with_context(|| {
                format!(
                    "copy legacy file {} to {}",
                    source_path.display(),
                    destination_path.display()
                )
            })?;
        }
    }
    Ok(())
}

fn write_meta(
    paths: &GraphPaths,
    status: GraphStatus,
    success: Option<(&str, &GraphIndex, Option<&str>)>,
    last_error: Option<&str>,
    degraded_reasons: Vec<String>,
) -> Result<()> {
    let (last_index_run_id, file_count, symbol_count, relation_count, languages, git_head) =
        if let Some((run_id, index, git_head)) = success {
            let languages = index
                .files
                .iter()
                .filter(|file| file.language != "unknown")
                .map(|file| file.language.clone())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect();
            (
                Some(run_id.to_string()),
                index.files.len(),
                index.symbols.len(),
                index.relations.len(),
                languages,
                git_head.map(str::to_string),
            )
        } else {
            (None, 0, 0, 0, Vec::new(), git_head(&paths.root))
        };

    let meta = GraphMeta {
        version: "panel-manifest.v1".to_string(),
        status,
        project_root: paths.root.display().to_string(),
        graph_db: PANEL_DB_RELATIVE_PATH.to_string(),
        updated_at: unix_timestamp_seconds(),
        git_head,
        file_count,
        symbol_count,
        relation_count,
        last_index_run_id,
        languages,
        last_error: last_error.map(str::to_string),
        degraded_reasons,
    };
    fs::write(&paths.meta, serde_json::to_string_pretty(&meta)?)
        .with_context(|| format!("write {}", paths.meta.display()))?;
    Ok(())
}

fn write_exports(paths: &GraphPaths, index: &GraphIndex) -> Result<()> {
    let language_counts = index
        .files
        .iter()
        .fold(BTreeMap::new(), |mut counts, file| {
            if file.language != "unknown" {
                *counts.entry(file.language.clone()).or_insert(0usize) += 1;
            }
            counts
        });
    let test_files = index
        .files
        .iter()
        .filter(|file| file.is_test)
        .collect::<Vec<_>>();
    let snapshot_id = format!("panel-snapshot-{}", unix_timestamp_seconds());

    write_json(&paths.file_tree, &index.files)?;
    write_json(&paths.languages, &language_counts)?;
    write_json(&paths.symbols, &index.symbols)?;
    write_json(&paths.relations, &index.relations)?;
    write_json(&paths.diagnostics, &Vec::<serde_json::Value>::new())?;
    write_json(
        &paths.git,
        &serde_json::json!({
            "version": "panel-git.v1",
            "head": git_head(&paths.root),
            "projectRoot": paths.root.display().to_string(),
        }),
    )?;
    write_json(
        &paths.tests,
        &test_files
            .iter()
            .map(|file| {
                serde_json::json!({
                    "path": file.path,
                    "language": file.language,
                    "kind": file.kind,
                })
            })
            .collect::<Vec<_>>(),
    )?;
    write_json(
        &paths.file_index,
        &index
            .files
            .iter()
            .map(|file| {
                serde_json::json!({
                    "id": file.id,
                    "path": file.path,
                    "name": file.name,
                    "language": file.language,
                    "kind": file.kind,
                })
            })
            .collect::<Vec<_>>(),
    )?;
    write_json(&paths.symbol_index, &index.symbols)?;
    write_json(
        &paths.content_index,
        &index
            .chunks
            .iter()
            .map(|chunk| {
                serde_json::json!({
                    "id": chunk.id,
                    "fileId": chunk.file_id,
                    "path": chunk.path,
                    "startLine": chunk.start_line,
                    "endLine": chunk.end_line,
                    "text": chunk.text,
                    "contentHash": chunk.content_hash,
                })
            })
            .collect::<Vec<_>>(),
    )?;
    write_json(
        &paths.snapshots.join(format!("{snapshot_id}.json")),
        &serde_json::json!({
            "version": "panel-snapshot.v1",
            "snapshotId": snapshot_id,
            "createdAt": unix_timestamp_seconds(),
            "projectRoot": paths.root.display().to_string(),
            "fileCount": index.files.len(),
            "symbolCount": index.symbols.len(),
            "relationCount": index.relations.len(),
            "languages": language_counts.keys().cloned().collect::<Vec<_>>(),
        }),
    )?;
    Ok(())
}

fn write_json<T: serde::Serialize>(path: &Path, value: &T) -> Result<()> {
    fs::write(path, format!("{}\n", serde_json::to_string_pretty(value)?))
        .with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

fn manifest_from_files(
    root: &Path,
    files: &[crate::model::GraphFileRecord],
) -> GraphManifestSnapshot {
    let languages = files
        .iter()
        .filter(|file| file.language != "unknown")
        .map(|file| file.language.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let top_level_dirs = files
        .iter()
        .filter_map(|file| file.path.split('/').next())
        .filter(|value| !value.is_empty() && *value != ".agentflow")
        .collect::<BTreeSet<_>>()
        .into_iter()
        .map(str::to_string)
        .collect();
    let important_files = files
        .iter()
        .filter(|file| {
            matches!(
                file.name.as_str(),
                "README.md"
                    | "Cargo.toml"
                    | "package.json"
                    | "pyproject.toml"
                    | "go.mod"
                    | "pubspec.yaml"
                    | "Package.swift"
                    | "Podfile"
                    | "Dockerfile"
            )
        })
        .map(|file| file.path.clone())
        .collect();

    GraphManifestSnapshot {
        version: "panel-manifest.v1".to_string(),
        project_root: root.display().to_string(),
        languages,
        top_level_dirs,
        important_files,
        source_files: files.iter().filter(|file| file.is_source).count(),
        test_files: files.iter().filter(|file| file.is_test).count(),
        doc_files: files.iter().filter(|file| file.is_doc).count(),
        config_files: files.iter().filter(|file| file.is_config).count(),
        platforms: detect_platforms(files),
        entry_points: detect_entry_points(files),
        mobile_components: detect_mobile_components(files),
        mobile_configs: detect_mobile_configs(files),
        mobile_tests: files
            .iter()
            .filter(|file| file.is_test && is_mobile_path(&file.path))
            .map(|file| file.path.clone())
            .collect(),
    }
}

fn enrich_status_with_runtime(
    paths: &GraphPaths,
    mut status: GraphStatusSnapshot,
) -> GraphStatusSnapshot {
    status.watcher_status = watcher::watcher_status(&paths.root);
    status.watcher_backend = watcher::watcher_backend(&paths.root);
    status.watcher_detail = watcher::watcher_detail(&paths.root);
    status.protection_status = check_graph_git_protection(&paths.root)
        .ok()
        .map(|snapshot| snapshot.status);
    status.preflight_status = Some(preflight_status_label(&status.status));
    if status.watcher_status.as_deref() == Some("fallback") {
        let reason = status
            .watcher_detail
            .as_ref()
            .and_then(|detail| detail.last_error.clone())
            .unwrap_or_else(|| "Graph watcher 使用降级 fallback。".to_string());
        if !status.degraded_reasons.contains(&reason) {
            status.degraded_reasons.push(reason);
        }
        if status.status == GraphStatus::Ready {
            status.status = GraphStatus::Degraded;
            status.preflight_status = Some(preflight_status_label(&status.status));
        }
    }
    status
}

fn preflight_status_label(status: &GraphStatus) -> String {
    match status {
        GraphStatus::Ready | GraphStatus::Degraded => "ready",
        GraphStatus::Indexing => "pending",
        GraphStatus::Missing | GraphStatus::Stale => "needs_prepare",
        GraphStatus::Failed => "blocked",
    }
    .to_string()
}

fn detect_platforms(files: &[crate::model::GraphFileRecord]) -> Vec<String> {
    let mut platforms = BTreeSet::new();
    if files.iter().any(|file| {
        file.name == "AndroidManifest.xml"
            || file.path.starts_with("android/")
            || file.name == "build.gradle"
            || file.name == "build.gradle.kts"
    }) {
        platforms.insert("android".to_string());
    }
    if files.iter().any(|file| {
        file.name == "Info.plist"
            || file.name == "Package.swift"
            || file.name == "Podfile"
            || file.path.contains(".xcodeproj/")
            || file.path.contains(".xcworkspace/")
            || file.path.starts_with("ios/")
    }) {
        platforms.insert("ios".to_string());
    }
    if files
        .iter()
        .any(|file| file.name == "pubspec.yaml" || file.path == "lib/main.dart")
    {
        platforms.insert("flutter".to_string());
    }
    platforms.into_iter().collect()
}

fn detect_entry_points(files: &[crate::model::GraphFileRecord]) -> Vec<String> {
    files
        .iter()
        .filter(|file| {
            matches!(
                file.path.as_str(),
                "src/main.rs" | "main.go" | "lib/main.dart" | "Package.swift" | "Info.plist"
            ) || file.name == "main.py"
                || file.name == "main.ts"
                || file.name == "main.tsx"
                || file.name == "App.swift"
                || file.name == "AndroidManifest.xml"
        })
        .map(|file| file.path.clone())
        .collect()
}

fn detect_mobile_components(files: &[crate::model::GraphFileRecord]) -> Vec<String> {
    files
        .iter()
        .filter(|file| {
            is_mobile_path(&file.path)
                && matches!(
                    file.language.as_str(),
                    "kotlin" | "java" | "swift" | "objc" | "dart" | "xml" | "plist"
                )
        })
        .map(|file| file.path.clone())
        .take(80)
        .collect()
}

fn detect_mobile_configs(files: &[crate::model::GraphFileRecord]) -> Vec<String> {
    files
        .iter()
        .filter(|file| {
            matches!(
                file.name.as_str(),
                "AndroidManifest.xml"
                    | "Info.plist"
                    | "Package.swift"
                    | "Podfile"
                    | "pubspec.yaml"
                    | "build.gradle"
                    | "build.gradle.kts"
                    | "settings.gradle"
                    | "settings.gradle.kts"
            ) || file.path.contains(".xcodeproj/")
                || file.path.contains(".xcworkspace/")
        })
        .map(|file| file.path.clone())
        .collect()
}

fn is_mobile_path(path: &str) -> bool {
    path.starts_with("android/")
        || path.starts_with("ios/")
        || path.starts_with("lib/")
        || path.contains("AndroidManifest.xml")
        || path.contains("Info.plist")
        || path.contains(".xcodeproj")
        || path.contains(".xcworkspace")
}

fn is_stale(paths: &GraphPaths) -> Result<bool> {
    if !paths.meta.is_file() {
        return Ok(true);
    }
    let meta: GraphMeta = serde_json::from_str(&fs::read_to_string(&paths.meta)?)?;
    Ok(meta.git_head != git_head(&paths.root))
}

fn git_head(root: &Path) -> Option<String> {
    let git_path = root.join(".git");
    let git_dir = if git_path.is_dir() {
        git_path
    } else if git_path.is_file() {
        let git_file = fs::read_to_string(&git_path).ok()?;
        let path_value = git_file.trim().strip_prefix("gitdir:")?.trim();
        let candidate = PathBuf::from(path_value);
        if candidate.is_absolute() {
            candidate
        } else {
            root.join(candidate)
        }
    } else {
        return None;
    };

    let head = fs::read_to_string(git_dir.join("HEAD")).ok()?;
    let head = head.trim();
    if let Some(reference) = head.strip_prefix("ref:") {
        let reference_path = git_dir.join(reference.trim());
        if let Ok(value) = fs::read_to_string(reference_path) {
            let value = value.trim().to_string();
            if !value.is_empty() {
                return Some(value);
            }
        }
        if let Ok(packed_refs) = fs::read_to_string(git_dir.join("packed-refs")) {
            let reference = reference.trim();
            for line in packed_refs.lines() {
                if line.starts_with('#') || line.starts_with('^') {
                    continue;
                }
                let mut parts = line.split_whitespace();
                let sha = parts.next()?;
                let name = parts.next()?;
                if name == reference {
                    return Some(sha.to_string());
                }
            }
        }
        None
    } else if head.is_empty() {
        None
    } else {
        Some(head.to_string())
    }
}

pub(crate) fn unix_timestamp_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_recommendation::recommend_graph_tests;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn index_project_graph_writes_panel_artifacts() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("src")).unwrap();
        fs::write(dir.path().join("src/lib.rs"), "pub struct Lease {}\n").unwrap();
        fs::write(dir.path().join("README.md"), "# Demo\n").unwrap();

        let status = index_project_graph(dir.path()).unwrap();

        assert_eq!(status.status, GraphStatus::Ready);
        assert!(dir.path().join(".agentflow/panel/index/panel.db").is_file());
        assert!(dir.path().join(".agentflow/panel/manifest.json").is_file());
        assert!(dir.path().join(".agentflow/panel/file-tree.json").is_file());
        assert!(dir.path().join(".agentflow/panel/languages.json").is_file());
        assert!(dir.path().join(".agentflow/panel/symbols.json").is_file());
        assert!(dir.path().join(".agentflow/panel/relations.json").is_file());
        assert!(dir
            .path()
            .join(".agentflow/panel/diagnostics.json")
            .is_file());
        assert!(dir.path().join(".agentflow/panel/git.json").is_file());
        assert!(dir.path().join(".agentflow/panel/tests.json").is_file());
        assert!(dir
            .path()
            .join(".agentflow/panel/search/file-index.json")
            .is_file());
        assert!(dir
            .path()
            .join(".agentflow/panel/search/symbol-index.json")
            .is_file());
        assert!(dir
            .path()
            .join(".agentflow/panel/search/content-index.json")
            .is_file());
        assert!(dir.path().join(".agentflow/panel/snapshots").is_dir());
        assert_eq!(status.file_count, 2);
        assert!(status.symbol_count >= 2);
    }

    #[test]
    fn mobile_fixture_matrix_reports_platforms_configs_components_and_tests() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("android/app/src/main/java/com/demo")).unwrap();
        fs::create_dir_all(dir.path().join("android/app/src/androidTest/java/com/demo")).unwrap();
        fs::create_dir_all(dir.path().join("ios/App")).unwrap();
        fs::create_dir_all(dir.path().join("ios/AppTests")).unwrap();
        fs::create_dir_all(dir.path().join("lib")).unwrap();
        fs::create_dir_all(dir.path().join("test")).unwrap();
        fs::write(
            dir.path().join("build.gradle"),
            "plugins { id 'com.android.application' }\n",
        )
        .unwrap();
        fs::write(
            dir.path().join("android/app/src/main/AndroidManifest.xml"),
            "<manifest><application><activity android:name=\".MainActivity\"/></application></manifest>\n",
        )
        .unwrap();
        fs::write(
            dir.path()
                .join("android/app/src/main/java/com/demo/MainActivity.kt"),
            "package com.demo\nclass MainActivity {}\n",
        )
        .unwrap();
        fs::write(
            dir.path()
                .join("android/app/src/androidTest/java/com/demo/MainActivityTest.kt"),
            "package com.demo\nclass MainActivityTest {}\n",
        )
        .unwrap();
        fs::write(
            dir.path().join("ios/App/Info.plist"),
            "<plist><dict><key>CFBundleIdentifier</key><string>demo</string></dict></plist>\n",
        )
        .unwrap();
        fs::write(
            dir.path().join("ios/App/App.swift"),
            "import SwiftUI\nstruct RootView: View { var body: some View { Text(\"ok\") } }\n",
        )
        .unwrap();
        fs::write(
            dir.path().join("ios/AppTests/AppTests.swift"),
            "import XCTest\nfinal class AppTests: XCTestCase {}\n",
        )
        .unwrap();
        fs::write(dir.path().join("pubspec.yaml"), "name: demo\nflutter:\n").unwrap();
        fs::write(
            dir.path().join("lib/main.dart"),
            "import 'package:flutter/widgets.dart';\nclass Home extends StatelessWidget {}\n",
        )
        .unwrap();
        fs::write(
            dir.path().join("test/widget_test.dart"),
            "void main() { testWidgets('home', (tester) async {}); }\n",
        )
        .unwrap();

        index_project_graph(dir.path()).unwrap();
        let manifest = load_project_graph_manifest(dir.path()).unwrap();
        let hints = recommend_graph_tests(dir.path(), &[], &[], &[]).unwrap();

        for platform in ["android", "ios", "flutter"] {
            assert!(
                manifest.platforms.iter().any(|item| item == platform),
                "missing platform {platform}"
            );
        }
        assert!(manifest
            .entry_points
            .iter()
            .any(|path| path.ends_with("AndroidManifest.xml")));
        assert!(manifest
            .mobile_configs
            .iter()
            .any(|path| path.ends_with("Info.plist")));
        assert!(manifest
            .mobile_components
            .iter()
            .any(|path| path.ends_with("MainActivity.kt")));
        assert!(manifest
            .mobile_tests
            .iter()
            .any(|path| path.ends_with("MainActivityTest.kt")));
        for command in [
            "./gradlew connectedAndroidTest",
            "xcodebuild test",
            "flutter test",
        ] {
            assert!(
                hints.iter().any(|hint| hint.command_hint == command),
                "missing test hint {command}"
            );
        }
    }
}
