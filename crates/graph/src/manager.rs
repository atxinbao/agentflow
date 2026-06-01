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
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

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
    exports: PathBuf,
}

pub fn prepare_project_graph(
    project_root: impl AsRef<Path>,
    mode: GraphPrepareMode,
) -> Result<GraphStatusSnapshot> {
    let paths = prepare_paths(project_root.as_ref())?;
    let project_root_string = paths.root.display().to_string();
    let existing = load_project_graph_status(&paths.root).unwrap_or_else(|_| GraphStatusSnapshot {
        version: "graph-status.v1".to_string(),
        project_root: project_root_string.clone(),
        status: GraphStatus::Missing,
        file_count: 0,
        symbol_count: 0,
        relation_count: 0,
        updated_at: None,
        last_error: None,
        watcher_status: None,
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
    let run_id = format!("graph-run-{started_at}");
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
                version: "graph-status.v1".to_string(),
                project_root: paths.root.display().to_string(),
                status: GraphStatus::Failed,
                file_count: 0,
                symbol_count: 0,
                relation_count: 0,
                updated_at: Some(finished_at),
                last_error: Some(message),
                watcher_status: watcher::watcher_status(&paths.root),
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
            version: "graph-status.v1".to_string(),
            project_root: paths.root.display().to_string(),
            status: GraphStatus::Missing,
            file_count: 0,
            symbol_count: 0,
            relation_count: 0,
            updated_at: None,
            last_error: None,
            watcher_status: watcher::watcher_status(&paths.root),
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
        version: "graph-status.v1".to_string(),
        project_root: paths.root.display().to_string(),
        status: status.clone(),
        file_count: meta.file_count,
        symbol_count: meta.symbol_count,
        relation_count: meta.relation_count,
        updated_at: Some(meta.updated_at),
        last_error: meta.last_error,
        watcher_status: watcher::watcher_status(&paths.root),
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
        version: "graph-manifest.v1".to_string(),
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
    fs::create_dir_all(&paths.exports)
        .with_context(|| format!("create {}", paths.exports.display()))?;
    Ok(paths)
}

fn graph_paths(project_root: &Path) -> Result<GraphPaths> {
    let root = project_root
        .canonicalize()
        .with_context(|| format!("canonicalize {}", project_root.display()))?;
    let graph_dir = root.join(".agentflow/output/graph");
    Ok(GraphPaths {
        root,
        graph_db: graph_dir.join("graph.db"),
        meta: graph_dir.join("meta.json"),
        context_packs: graph_dir.join("context-packs"),
        exports: graph_dir.join("exports"),
        graph_dir,
    })
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
        version: "graph.v1".to_string(),
        status,
        project_root: paths.root.display().to_string(),
        graph_db: ".agentflow/output/graph/graph.db".to_string(),
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
    let manifest = manifest_from_files(&paths.root, &index.files);
    fs::write(
        paths.exports.join("manifest.json"),
        serde_json::to_string_pretty(&manifest)?,
    )?;
    write_jsonl(&paths.exports.join("files.jsonl"), &index.files)?;
    write_jsonl(&paths.exports.join("symbols.jsonl"), &index.symbols)?;
    write_jsonl(&paths.exports.join("relations.jsonl"), &index.relations)?;
    write_jsonl(&paths.exports.join("chunks.jsonl"), &index.chunks)?;
    Ok(())
}

fn write_jsonl<T: serde::Serialize>(path: &Path, records: &[T]) -> Result<()> {
    let mut content = String::new();
    for record in records {
        content.push_str(&serde_json::to_string(record)?);
        content.push('\n');
    }
    fs::write(path, content).with_context(|| format!("write {}", path.display()))?;
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
        version: "graph-manifest.v1".to_string(),
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
    status.protection_status = check_graph_git_protection(&paths.root)
        .ok()
        .map(|snapshot| snapshot.status);
    status.preflight_status = Some(preflight_status_label(&status.status));
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
                "src/main.rs"
                    | "main.go"
                    | "lib/main.dart"
                    | "Package.swift"
                    | "AndroidManifest.xml"
                    | "Info.plist"
            ) || file.name == "main.py"
                || file.name == "main.ts"
                || file.name == "main.tsx"
                || file.name == "App.swift"
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
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn index_project_graph_writes_db_meta_and_exports() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("src")).unwrap();
        fs::write(dir.path().join("src/lib.rs"), "pub struct Lease {}\n").unwrap();
        fs::write(dir.path().join("README.md"), "# Demo\n").unwrap();

        let status = index_project_graph(dir.path()).unwrap();

        assert_eq!(status.status, GraphStatus::Ready);
        assert!(dir
            .path()
            .join(".agentflow/output/graph/graph.db")
            .is_file());
        assert!(dir
            .path()
            .join(".agentflow/output/graph/meta.json")
            .is_file());
        assert!(dir
            .path()
            .join(".agentflow/output/graph/exports/files.jsonl")
            .is_file());
        assert_eq!(status.file_count, 2);
        assert!(status.symbol_count >= 2);
    }
}
