use crate::{
    db,
    model::{PanelIndex, PanelManifestSnapshot, PanelStatus, PanelStatusSnapshot},
    protection::check_panel_git_protection,
    scanner::scan_project,
    watcher,
};
use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

const PANEL_DIR: &str = ".agentflow/panel";
const PANEL_DB_RELATIVE_PATH: &str = ".agentflow/panel/index/panel.db";
const PANEL_MANIFEST_RELATIVE_PATH: &str = ".agentflow/panel/manifest.json";
const LARGE_FILE_BYTES: u64 = 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelPrepareMode {
    Blocking,
    Background,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PanelMeta {
    version: String,
    status: PanelStatus,
    project_root: String,
    backend: String,
    last_indexed_at: u64,
    active_snapshot_id: Option<String>,
    paths: PanelManifestPaths,
    summary: PanelSummary,
    worktree: PanelWorktree,
    watcher: PanelManifestWatcher,
    #[serde(default)]
    #[serde(alias = "degradedReasons")]
    warnings: Vec<String>,
    #[serde(default)]
    errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PanelManifestPaths {
    database: String,
    file_tree: String,
    languages: String,
    symbols: String,
    relations: String,
    diagnostics: String,
    git: String,
    tests: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PanelSummary {
    files: usize,
    languages: usize,
    symbols: usize,
    relations: usize,
    diagnostics: usize,
    tests: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PanelWorktree {
    root: String,
    git_branch: Option<String>,
    head_sha: Option<String>,
    dirty: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PanelManifestWatcher {
    status: String,
    backend: String,
}

#[derive(Debug, Clone)]
struct PanelPaths {
    root: PathBuf,
    panel_dir: PathBuf,
    panel_db: PathBuf,
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
}

#[derive(Debug, Clone)]
struct PanelExportSummary {
    warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PanelGitSnapshot {
    version: String,
    is_git_repo: bool,
    branch: Option<String>,
    head_sha: Option<String>,
    dirty: bool,
    modified_files: Vec<String>,
    untracked_files: Vec<String>,
    deleted_files: Vec<String>,
    ignored_files: Vec<String>,
    agentflow_protected: bool,
    warnings: Vec<String>,
    errors: Vec<String>,
}

pub fn prepare_project_panel(
    project_root: impl AsRef<Path>,
    mode: PanelPrepareMode,
) -> Result<PanelStatusSnapshot> {
    let project_root = project_root.as_ref();
    ensure_agentflow_owned_for_panel(project_root)?;

    let paths = prepare_paths(project_root)?;
    let project_root_string = paths.root.display().to_string();
    let existing = load_project_panel_status(&paths.root).unwrap_or_else(|_| PanelStatusSnapshot {
        version: "panel-status.v1".to_string(),
        project_root: project_root_string.clone(),
        status: PanelStatus::Missing,
        file_count: 0,
        symbol_count: 0,
        relation_count: 0,
        updated_at: None,
        last_error: None,
        watcher_status: None,
        watcher_backend: None,
        watcher_detail: None,
        preflight_status: Some(preflight_status_label(&PanelStatus::Missing)),
        protection_status: None,
        warnings: Vec::new(),
    });

    if existing.status == PanelStatus::Ready && !is_stale(&paths)? {
        if mode == PanelPrepareMode::Background {
            let _ = watcher::ensure_panel_watcher(&paths.root);
        }
        return Ok(existing);
    }

    write_manifest(&paths, PanelStatus::Indexing, None, None, Vec::new())?;
    if mode == PanelPrepareMode::Background {
        let root = paths.root.clone();
        std::thread::spawn(move || {
            let _ = index_project_panel(root);
        });
        let _ = watcher::ensure_panel_watcher(&paths.root);
        return load_project_panel_status(&paths.root);
    }

    index_project_panel(&paths.root)
}

pub fn index_project_panel(project_root: impl AsRef<Path>) -> Result<PanelStatusSnapshot> {
    ensure_agentflow_owned_for_panel(project_root.as_ref())?;
    let paths = prepare_paths(project_root.as_ref())?;
    let mut connection = db::open_panel_db(&paths.panel_db)?;
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
            let mut exports = write_exports(&paths, &index)?;
            let finished_at = unix_timestamp_seconds();
            let protection = check_panel_git_protection(&paths.root).ok();
            if let Some(protection) = &protection {
                if protection.status == "warning" && !exports.warnings.contains(&protection.reason)
                {
                    exports.warnings.push(protection.reason.clone());
                }
            }
            let panel_status = PanelStatus::Ready;
            db::finish_index_run(
                &connection,
                &run_id,
                finished_at,
                "ready",
                Some(&index),
                None,
            )?;
            write_manifest(
                &paths,
                panel_status.clone(),
                Some((&run_id, &index, git_head.as_deref())),
                None,
                exports.warnings,
            )?;
            let mut status =
                db::counts(&connection, &paths.root.display().to_string(), panel_status)?;
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
            write_manifest(
                &paths,
                PanelStatus::Failed,
                None,
                Some(&message),
                Vec::new(),
            )?;
            Ok(PanelStatusSnapshot {
                version: "panel-status.v1".to_string(),
                project_root: paths.root.display().to_string(),
                status: PanelStatus::Failed,
                file_count: 0,
                symbol_count: 0,
                relation_count: 0,
                updated_at: Some(finished_at),
                last_error: Some(message),
                watcher_status: watcher::watcher_status(&paths.root),
                watcher_backend: watcher::watcher_backend(&paths.root),
                watcher_detail: watcher::watcher_detail(&paths.root),
                preflight_status: Some(preflight_status_label(&PanelStatus::Failed)),
                protection_status: check_panel_git_protection(&paths.root)
                    .ok()
                    .map(|snapshot| snapshot.status),
                warnings: Vec::new(),
            })
        }
    }
}

pub(crate) fn ensure_agentflow_owned_for_panel(project_root: &Path) -> Result<()> {
    let ownership = agentflow_agent_manual::check_agentflow_workspace_ownership(project_root)?;
    if !ownership.ready_for_prepare {
        return Err(anyhow!(
            "AgentFlow workspace ownership blocks panel prepare: {:?}: {}",
            ownership.status,
            ownership.errors.join("; ")
        ));
    }

    let agent_status = agentflow_agent_manual::prepare_agent_working_manual(project_root)?;
    if !agent_status.ready {
        return Err(anyhow!(
            "Agent working manual is not ready for panel prepare: {:?}: {}",
            agent_status.status,
            agent_status.errors.join("; ")
        ));
    }

    Ok(())
}

pub fn load_project_panel_status(project_root: impl AsRef<Path>) -> Result<PanelStatusSnapshot> {
    let paths = panel_paths(project_root.as_ref())?;
    if !paths.meta.is_file() || !paths.panel_db.is_file() {
        return Ok(PanelStatusSnapshot {
            version: "panel-status.v1".to_string(),
            project_root: paths.root.display().to_string(),
            status: PanelStatus::Missing,
            file_count: 0,
            symbol_count: 0,
            relation_count: 0,
            updated_at: None,
            last_error: None,
            watcher_status: watcher::watcher_status(&paths.root),
            watcher_backend: watcher::watcher_backend(&paths.root),
            watcher_detail: watcher::watcher_detail(&paths.root),
            preflight_status: Some(preflight_status_label(&PanelStatus::Missing)),
            protection_status: check_panel_git_protection(&paths.root)
                .ok()
                .map(|snapshot| snapshot.status),
            warnings: Vec::new(),
        });
    }
    let meta: PanelMeta = serde_json::from_str(&fs::read_to_string(&paths.meta)?)?;
    let status = if meta.status == PanelStatus::Ready && is_stale(&paths)? {
        PanelStatus::Stale
    } else {
        meta.status
    };
    let mut warnings = meta.warnings;
    let protection_status = check_panel_git_protection(&paths.root).ok();
    if let Some(protection) = &protection_status {
        if protection.status == "warning" && !warnings.contains(&protection.reason) {
            warnings.push(protection.reason.clone());
        }
    }
    Ok(PanelStatusSnapshot {
        version: "panel-status.v1".to_string(),
        project_root: paths.root.display().to_string(),
        status: status.clone(),
        file_count: meta.summary.files,
        symbol_count: meta.summary.symbols,
        relation_count: meta.summary.relations,
        updated_at: Some(meta.last_indexed_at),
        last_error: meta.errors.first().cloned(),
        watcher_status: watcher::watcher_status(&paths.root),
        watcher_backend: watcher::watcher_backend(&paths.root),
        watcher_detail: watcher::watcher_detail(&paths.root),
        preflight_status: Some(preflight_status_label(&status)),
        protection_status: protection_status.map(|snapshot| snapshot.status),
        warnings,
    })
}

pub fn load_project_panel_manifest(
    project_root: impl AsRef<Path>,
) -> Result<PanelManifestSnapshot> {
    let paths = panel_paths(project_root.as_ref())?;
    if paths.panel_db.is_file() {
        let connection = db::open_panel_db(&paths.panel_db)?;
        let files = db::fetch_files(&connection)?;
        return Ok(manifest_from_files(&paths.root, &files));
    }
    Ok(PanelManifestSnapshot {
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

pub(crate) fn panel_db_path(project_root: impl AsRef<Path>) -> Result<PathBuf> {
    Ok(panel_paths(project_root.as_ref())?.panel_db)
}

pub(crate) fn context_pack_dir(project_root: impl AsRef<Path>) -> Result<PathBuf> {
    Ok(prepare_paths(project_root.as_ref())?.context_packs)
}

fn prepare_paths(project_root: &Path) -> Result<PanelPaths> {
    let paths = panel_paths(project_root)?;
    fs::create_dir_all(&paths.panel_dir)
        .with_context(|| format!("create {}", paths.panel_dir.display()))?;
    fs::create_dir_all(&paths.context_packs)
        .with_context(|| format!("create {}", paths.context_packs.display()))?;
    fs::create_dir_all(&paths.search)
        .with_context(|| format!("create {}", paths.search.display()))?;
    fs::create_dir_all(&paths.snapshots)
        .with_context(|| format!("create {}", paths.snapshots.display()))?;
    fs::create_dir_all(&paths.index)
        .with_context(|| format!("create {}", paths.index.display()))?;
    Ok(paths)
}

fn panel_paths(project_root: &Path) -> Result<PanelPaths> {
    let root = project_root
        .canonicalize()
        .with_context(|| format!("canonicalize {}", project_root.display()))?;
    let panel_dir = root.join(PANEL_DIR);
    let search = panel_dir.join("search");
    let snapshots = panel_dir.join("snapshots");
    let index = panel_dir.join("index");
    Ok(PanelPaths {
        panel_db: root.join(PANEL_DB_RELATIVE_PATH),
        meta: root.join(PANEL_MANIFEST_RELATIVE_PATH),
        context_packs: panel_dir.join("context-packs"),
        file_tree: panel_dir.join("file-tree.json"),
        languages: panel_dir.join("languages.json"),
        symbols: panel_dir.join("symbols.json"),
        relations: panel_dir.join("relations.json"),
        diagnostics: panel_dir.join("diagnostics.json"),
        git: panel_dir.join("git.json"),
        tests: panel_dir.join("tests.json"),
        file_index: search.join("file-index.json"),
        symbol_index: search.join("symbol-index.json"),
        content_index: search.join("content-index.json"),
        root,
        search,
        snapshots,
        index,
        panel_dir,
    })
}

fn write_manifest(
    paths: &PanelPaths,
    status: PanelStatus,
    success: Option<(&str, &PanelIndex, Option<&str>)>,
    last_error: Option<&str>,
    warnings: Vec<String>,
) -> Result<()> {
    let git = read_git_status(&paths.root);
    let (active_snapshot_id, file_count, symbol_count, relation_count, language_count, test_count) =
        if let Some((run_id, index, _git_head)) = success {
            let language_count = index
                .files
                .iter()
                .filter(|file| file.language != "unknown")
                .map(|file| file.language.clone())
                .collect::<BTreeSet<_>>()
                .len();
            (
                Some(run_id.to_string()),
                index.files.len(),
                index.symbols.len(),
                index.relations.len(),
                language_count,
                index.files.iter().filter(|file| file.is_test).count(),
            )
        } else {
            (None, 0, 0, 0, 0, 0)
        };

    let meta = PanelMeta {
        version: "panel-manifest.v1".to_string(),
        status,
        project_root: paths.root.display().to_string(),
        backend: "panel".to_string(),
        last_indexed_at: unix_timestamp_seconds(),
        active_snapshot_id,
        paths: PanelManifestPaths {
            database: PANEL_DB_RELATIVE_PATH.to_string(),
            file_tree: ".agentflow/panel/file-tree.json".to_string(),
            languages: ".agentflow/panel/languages.json".to_string(),
            symbols: ".agentflow/panel/symbols.json".to_string(),
            relations: ".agentflow/panel/relations.json".to_string(),
            diagnostics: ".agentflow/panel/diagnostics.json".to_string(),
            git: ".agentflow/panel/git.json".to_string(),
            tests: ".agentflow/panel/tests.json".to_string(),
        },
        summary: PanelSummary {
            files: file_count,
            languages: language_count,
            symbols: symbol_count,
            relations: relation_count,
            diagnostics: 0,
            tests: test_count,
        },
        worktree: PanelWorktree {
            root: paths.root.display().to_string(),
            git_branch: git.branch.clone(),
            head_sha: git.head_sha.clone(),
            dirty: git.dirty,
        },
        watcher: PanelManifestWatcher {
            status: watcher::watcher_status(&paths.root)
                .unwrap_or_else(|| "not_started".to_string()),
            backend: watcher::watcher_backend(&paths.root).unwrap_or_else(|| "unknown".to_string()),
        },
        warnings,
        errors: last_error
            .map(|error| vec![error.to_string()])
            .unwrap_or_default(),
    };
    fs::write(&paths.meta, serde_json::to_string_pretty(&meta)?)
        .with_context(|| format!("write {}", paths.meta.display()))?;
    Ok(())
}

fn write_exports(paths: &PanelPaths, index: &PanelIndex) -> Result<PanelExportSummary> {
    let git = read_git_status(&paths.root);
    let git_status_by_path = git_status_by_path(&git);
    let language_counts = index
        .files
        .iter()
        .fold(BTreeMap::new(), |mut counts, file| {
            if file.language != "unknown" {
                *counts.entry(file.language.clone()).or_insert(0usize) += 1;
            }
            counts
        });
    let diagnostics = diagnostics_snapshot(Vec::new());
    let tests = tests_snapshot(&paths.root, index);
    let test_files = index
        .files
        .iter()
        .filter(|file| file.is_test)
        .collect::<Vec<_>>();
    let snapshot_id = format!("panel-snapshot-{}", unix_timestamp_seconds());

    write_json(
        &paths.file_tree,
        &file_tree_snapshot(index, &git_status_by_path),
    )?;
    write_json(
        &paths.languages,
        &languages_snapshot(index, &language_counts),
    )?;
    write_json(&paths.symbols, &index.symbols)?;
    write_json(&paths.relations, &index.relations)?;
    write_json(&paths.diagnostics, &diagnostics)?;
    write_json(&paths.git, &git)?;
    write_json(&paths.tests, &tests)?;
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
            "manifestSummary": {
                "files": index.files.len(),
                "symbols": index.symbols.len(),
                "relations": index.relations.len()
            },
            "worktree": {
                "gitBranch": git.branch.clone(),
                "headSha": git.head_sha.clone(),
                "dirty": git.dirty
            },
            "gitSummary": {
                "modified": git.modified_files.len(),
                "untracked": git.untracked_files.len(),
                "deleted": git.deleted_files.len()
            },
            "fileSummary": {
                "source": index.files.iter().filter(|file| file.is_source).count(),
                "tests": test_files.len(),
                "docs": index.files.iter().filter(|file| file.is_doc).count(),
                "config": index.files.iter().filter(|file| file.is_config).count()
            },
            "fileHashSummary": {
                "tracked": index.files.len()
            },
            "diagnosticsSummary": diagnostics["summary"],
            "testsSummary": {
                "testFiles": test_files.len(),
                "frameworks": tests["frameworks"].clone()
            },
            "panelPaths": {
                "manifest": ".agentflow/panel/manifest.json",
                "database": ".agentflow/panel/index/panel.db",
                "fileTree": ".agentflow/panel/file-tree.json",
                "languages": ".agentflow/panel/languages.json",
                "symbols": ".agentflow/panel/symbols.json",
                "relations": ".agentflow/panel/relations.json",
                "diagnostics": ".agentflow/panel/diagnostics.json",
                "git": ".agentflow/panel/git.json",
                "tests": ".agentflow/panel/tests.json"
            }
        }),
    )?;
    Ok(PanelExportSummary {
        warnings: git.warnings.clone(),
    })
}

fn write_json<T: serde::Serialize>(path: &Path, value: &T) -> Result<()> {
    fs::write(path, format!("{}\n", serde_json::to_string_pretty(value)?))
        .with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

fn diagnostics_snapshot(items: Vec<serde_json::Value>) -> serde_json::Value {
    let mut errors = 0usize;
    let mut warnings = 0usize;
    let mut infos = 0usize;
    for item in &items {
        match item
            .get("severity")
            .and_then(|value| value.as_str())
            .unwrap_or("info")
        {
            "error" => errors += 1,
            "warning" => warnings += 1,
            _ => infos += 1,
        }
    }
    serde_json::json!({
        "version": "panel-diagnostics.v1",
        "summary": {
            "errors": errors,
            "warnings": warnings,
            "infos": infos
        },
        "items": items
    })
}

fn file_tree_snapshot(
    index: &PanelIndex,
    git_status_by_path: &BTreeMap<String, String>,
) -> serde_json::Value {
    let files = index
        .files
        .iter()
        .map(|file| {
            serde_json::json!({
                "path": file.path,
                "kind": file.kind,
                "extension": file.extension,
                "size": file.size_bytes,
                "modifiedAt": file.modified_at,
                "language": file.language,
                "mimeType": mime_type_for(file),
                "isBinary": is_binary_record(file),
                "isLarge": file.size_bytes > LARGE_FILE_BYTES,
                "isSymlink": false,
                "ignored": file.is_ignored,
                "gitStatus": git_status_by_path
                    .get(&file.path)
                    .cloned()
                    .unwrap_or_else(|| "clean".to_string()),
                "diagnosticSummary": {
                    "errors": 0,
                    "warnings": 0,
                    "infos": 0
                }
            })
        })
        .collect::<Vec<_>>();
    serde_json::json!({
        "version": "panel-file-tree.v1",
        "files": files
    })
}

fn languages_snapshot(
    index: &PanelIndex,
    language_counts: &BTreeMap<String, usize>,
) -> serde_json::Value {
    let languages = language_counts
        .iter()
        .map(|(language, file_count)| {
            let files = index
                .files
                .iter()
                .filter(|file| &file.language == language)
                .collect::<Vec<_>>();
            serde_json::json!({
                "language": language,
                "fileCount": file_count,
                "entryFiles": files
                    .iter()
                    .filter(|file| is_entry_file(&file.path, &file.name))
                    .map(|file| file.path.clone())
                    .collect::<Vec<_>>(),
                "configFiles": files
                    .iter()
                    .filter(|file| file.is_config)
                    .map(|file| file.path.clone())
                    .collect::<Vec<_>>(),
                "packageFiles": files
                    .iter()
                    .filter(|file| is_package_file(&file.name))
                    .map(|file| file.path.clone())
                    .collect::<Vec<_>>(),
                "testFrameworkHints": test_framework_hints_for_language(language, index),
                "mobilePlatformHints": mobile_platform_hints_for_language(language, index)
            })
        })
        .collect::<Vec<_>>();
    serde_json::json!({
        "version": "panel-languages.v1",
        "languages": languages
    })
}

fn tests_snapshot(root: &Path, index: &PanelIndex) -> serde_json::Value {
    let test_files = index
        .files
        .iter()
        .filter(|file| file.is_test)
        .map(|file| {
            serde_json::json!({
                "path": file.path,
                "language": file.language,
                "kind": file.kind,
            })
        })
        .collect::<Vec<_>>();
    let frameworks = detect_test_frameworks(root, index);
    let command_candidates = detect_test_command_candidates(root, index, &frameworks);
    serde_json::json!({
        "version": "panel-tests.v1",
        "testFiles": test_files,
        "frameworks": frameworks,
        "commandCandidates": command_candidates,
        "sourceToLikelyTests": source_to_likely_tests(index),
        "testToLikelySources": test_to_likely_sources(index),
        "hints": command_candidates
            .iter()
            .map(|command| {
                serde_json::json!({
                    "command": command,
                    "reason": "Panel inferred this candidate without executing tests."
                })
            })
            .collect::<Vec<_>>()
    })
}

fn read_git_status(root: &Path) -> PanelGitSnapshot {
    let is_git_repo = git_output(root, &["rev-parse", "--is-inside-work-tree"])
        .map(|value| value == "true")
        .unwrap_or(false);
    if !is_git_repo {
        return PanelGitSnapshot {
            version: "panel-git.v1".to_string(),
            is_git_repo: false,
            branch: None,
            head_sha: None,
            dirty: false,
            modified_files: Vec::new(),
            untracked_files: Vec::new(),
            deleted_files: Vec::new(),
            ignored_files: Vec::new(),
            agentflow_protected: true,
            warnings: vec!["Project is not a Git repository".to_string()],
            errors: Vec::new(),
        };
    }

    let branch = git_output(root, &["rev-parse", "--abbrev-ref", "HEAD"]).ok();
    let head_sha = git_output(root, &["rev-parse", "HEAD"]).ok();
    let porcelain =
        git_output(root, &["status", "--porcelain", "--untracked-files=all"]).unwrap_or_default();
    let ignored = git_output(root, &["status", "--ignored", "--porcelain"]).unwrap_or_default();
    let mut modified_files = Vec::new();
    let mut untracked_files = Vec::new();
    let mut deleted_files = Vec::new();
    let mut ignored_files = Vec::new();

    for line in porcelain.lines() {
        if line.len() < 4 {
            continue;
        }
        let code = &line[..2];
        let path = line[3..].to_string();
        if code == "??" {
            untracked_files.push(path);
        } else if code.contains('D') {
            deleted_files.push(path);
        } else {
            modified_files.push(path);
        }
    }
    for line in ignored.lines() {
        if let Some(path) = line.strip_prefix("!! ") {
            ignored_files.push(path.to_string());
        }
    }
    let protection = check_panel_git_protection(root).ok();
    let agentflow_protected = protection
        .as_ref()
        .map(|snapshot| snapshot.protected_by_info_exclude)
        .unwrap_or(false);
    let mut warnings = Vec::new();
    if !agentflow_protected {
        warnings.push("Project .agentflow directory is not protected by Git exclude".to_string());
    }

    PanelGitSnapshot {
        version: "panel-git.v1".to_string(),
        is_git_repo: true,
        branch,
        head_sha,
        dirty: !(modified_files.is_empty()
            && untracked_files.is_empty()
            && deleted_files.is_empty()),
        modified_files,
        untracked_files,
        deleted_files,
        ignored_files,
        agentflow_protected,
        warnings,
        errors: Vec::new(),
    }
}

fn git_output(root: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .with_context(|| format!("run git {}", args.join(" ")))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(anyhow::anyhow!(
            "{}",
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }
}

fn git_status_by_path(git: &PanelGitSnapshot) -> BTreeMap<String, String> {
    let mut status = BTreeMap::new();
    for path in &git.modified_files {
        status.insert(path.clone(), "modified".to_string());
    }
    for path in &git.untracked_files {
        status.insert(path.clone(), "untracked".to_string());
    }
    for path in &git.deleted_files {
        status.insert(path.clone(), "deleted".to_string());
    }
    for path in &git.ignored_files {
        status.insert(path.clone(), "ignored".to_string());
    }
    status
}

fn mime_type_for(file: &crate::model::PanelFileRecord) -> String {
    match file.language.as_str() {
        "markdown" => "text/markdown",
        "json" => "application/json",
        "toml" => "application/toml",
        "yaml" => "application/yaml",
        "html" => "text/html",
        "css" => "text/css",
        "xml" | "plist" => "application/xml",
        "unknown" if is_binary_record(file) => "application/octet-stream",
        _ => "text/plain",
    }
    .to_string()
}

fn is_binary_record(file: &crate::model::PanelFileRecord) -> bool {
    file.language == "unknown" && file.line_count == 0 && file.size_bytes > 0
}

fn is_entry_file(path: &str, name: &str) -> bool {
    matches!(
        path,
        "src/main.rs" | "main.go" | "lib/main.dart" | "Package.swift"
    ) || matches!(
        name,
        "main.py" | "main.ts" | "main.tsx" | "App.swift" | "AndroidManifest.xml"
    )
}

fn is_package_file(name: &str) -> bool {
    matches!(
        name,
        "Cargo.toml"
            | "package.json"
            | "pyproject.toml"
            | "go.mod"
            | "pubspec.yaml"
            | "Package.swift"
            | "pom.xml"
            | "build.gradle"
            | "build.gradle.kts"
    )
}

fn detect_test_frameworks(root: &Path, index: &PanelIndex) -> Vec<String> {
    let mut frameworks = BTreeSet::new();
    let languages = index
        .files
        .iter()
        .map(|file| file.language.as_str())
        .collect::<BTreeSet<_>>();
    if root.join("Cargo.toml").is_file() || languages.contains("rust") {
        frameworks.insert("cargo".to_string());
    }
    if root.join("package.json").is_file() {
        if let Ok(package) = fs::read_to_string(root.join("package.json")) {
            let lower = package.to_ascii_lowercase();
            for framework in ["vitest", "jest", "playwright"] {
                if lower.contains(framework) {
                    frameworks.insert(framework.to_string());
                }
            }
            if lower.contains("\"test\"") {
                frameworks.insert("npm-test".to_string());
            }
        }
    }
    if root.join("pyproject.toml").is_file() || languages.contains("python") {
        frameworks.insert("pytest".to_string());
    }
    if root.join("go.mod").is_file() || languages.contains("go") {
        frameworks.insert("go-test".to_string());
    }
    if root.join("Package.swift").is_file() || languages.contains("swift") {
        frameworks.insert("swift-test".to_string());
    }
    if root.join("pubspec.yaml").is_file() || languages.contains("dart") {
        frameworks.insert("flutter-test".to_string());
    }
    if root.join("pom.xml").is_file() {
        frameworks.insert("maven-test".to_string());
    }
    if root.join("build.gradle").is_file() || root.join("build.gradle.kts").is_file() {
        frameworks.insert("gradle-test".to_string());
    }
    frameworks.into_iter().collect()
}

fn detect_test_command_candidates(
    root: &Path,
    index: &PanelIndex,
    frameworks: &[String],
) -> Vec<String> {
    let mut commands = BTreeSet::new();
    for framework in frameworks {
        match framework.as_str() {
            "cargo" => {
                commands.insert("cargo test".to_string());
            }
            "vitest" | "jest" | "playwright" | "npm-test" => {
                commands.insert("npm test".to_string());
            }
            "pytest" => {
                commands.insert("pytest".to_string());
            }
            "go-test" => {
                commands.insert("go test ./...".to_string());
            }
            "swift-test" => {
                commands.insert("swift test".to_string());
            }
            "flutter-test" => {
                commands.insert("flutter test".to_string());
            }
            "maven-test" => {
                commands.insert("mvn test".to_string());
            }
            "gradle-test" => {
                commands.insert("./gradlew test".to_string());
            }
            _ => {}
        }
    }
    if root.join("apps/desktop/package.json").is_file()
        && index
            .files
            .iter()
            .any(|file| file.path.starts_with("apps/desktop/"))
    {
        commands.insert("npm --prefix apps/desktop test".to_string());
    }
    commands.into_iter().collect()
}

fn test_framework_hints_for_language(language: &str, index: &PanelIndex) -> Vec<String> {
    let mut hints = BTreeSet::new();
    match language {
        "rust" => {
            hints.insert("cargo".to_string());
        }
        "typescript" | "javascript" => {
            for file in &index.files {
                if matches!(file.name.as_str(), "package.json") {
                    hints.insert("npm-test".to_string());
                }
                if file.path.ends_with(".test.ts")
                    || file.path.ends_with(".spec.ts")
                    || file.path.ends_with(".test.tsx")
                    || file.path.ends_with(".spec.tsx")
                {
                    hints.insert("vitest-or-jest".to_string());
                }
            }
        }
        "python" => {
            hints.insert("pytest-or-unittest".to_string());
        }
        "go" => {
            hints.insert("go-test".to_string());
        }
        "java" | "kotlin" => {
            hints.insert("gradle-or-maven".to_string());
        }
        "swift" => {
            hints.insert("swift-test".to_string());
        }
        "dart" => {
            hints.insert("flutter-test".to_string());
        }
        _ => {}
    }
    hints.into_iter().collect()
}

fn mobile_platform_hints_for_language(language: &str, index: &PanelIndex) -> Vec<String> {
    let mut hints = BTreeSet::new();
    if matches!(language, "kotlin" | "java")
        && index
            .files
            .iter()
            .any(|file| file.path.starts_with("android/"))
    {
        hints.insert("android".to_string());
    }
    if matches!(language, "swift" | "objc" | "plist")
        && index.files.iter().any(|file| file.path.starts_with("ios/"))
    {
        hints.insert("ios".to_string());
    }
    if language == "dart" && index.files.iter().any(|file| file.path == "pubspec.yaml") {
        hints.insert("flutter".to_string());
    }
    hints.into_iter().collect()
}

fn source_to_likely_tests(index: &PanelIndex) -> serde_json::Value {
    let tests = index
        .files
        .iter()
        .filter(|file| file.is_test)
        .collect::<Vec<_>>();
    let mut map = serde_json::Map::new();
    for source in index
        .files
        .iter()
        .filter(|file| file.is_source && !file.is_test)
    {
        let stem = Path::new(&source.path)
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("");
        let likely = tests
            .iter()
            .filter(|test| {
                test.path
                    .to_ascii_lowercase()
                    .contains(&stem.to_ascii_lowercase())
            })
            .map(|test| test.path.clone())
            .collect::<Vec<_>>();
        if !likely.is_empty() {
            map.insert(source.path.clone(), serde_json::json!(likely));
        }
    }
    serde_json::Value::Object(map)
}

fn test_to_likely_sources(index: &PanelIndex) -> serde_json::Value {
    let sources = index
        .files
        .iter()
        .filter(|file| file.is_source && !file.is_test)
        .collect::<Vec<_>>();
    let mut map = serde_json::Map::new();
    for test in index.files.iter().filter(|file| file.is_test) {
        let test_lower = test.path.to_ascii_lowercase();
        let likely = sources
            .iter()
            .filter(|source| {
                let stem = Path::new(&source.path)
                    .file_stem()
                    .and_then(|value| value.to_str())
                    .unwrap_or("")
                    .to_ascii_lowercase();
                !stem.is_empty() && test_lower.contains(&stem)
            })
            .map(|source| source.path.clone())
            .collect::<Vec<_>>();
        if !likely.is_empty() {
            map.insert(test.path.clone(), serde_json::json!(likely));
        }
    }
    serde_json::Value::Object(map)
}

fn manifest_from_files(
    root: &Path,
    files: &[crate::model::PanelFileRecord],
) -> PanelManifestSnapshot {
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

    PanelManifestSnapshot {
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
    paths: &PanelPaths,
    mut status: PanelStatusSnapshot,
) -> PanelStatusSnapshot {
    status.watcher_status = watcher::watcher_status(&paths.root);
    status.watcher_backend = watcher::watcher_backend(&paths.root);
    status.watcher_detail = watcher::watcher_detail(&paths.root);
    status.protection_status = check_panel_git_protection(&paths.root)
        .ok()
        .map(|snapshot| snapshot.status);
    status.preflight_status = Some(preflight_status_label(&status.status));
    if status.watcher_status.as_deref() == Some("failed") {
        if matches!(status.status, PanelStatus::Ready | PanelStatus::Stale) {
            status.status = PanelStatus::Failed;
            status.preflight_status = Some(preflight_status_label(&status.status));
        }
        if status.last_error.is_none() {
            status.last_error = status
                .watcher_detail
                .as_ref()
                .and_then(|detail| detail.last_error.clone());
        }
    }
    status
}

fn preflight_status_label(status: &PanelStatus) -> String {
    match status {
        PanelStatus::Ready => "ready",
        PanelStatus::Indexing => "pending",
        PanelStatus::Missing | PanelStatus::Stale => "needs_prepare",
        PanelStatus::Failed => "blocked",
    }
    .to_string()
}

fn detect_platforms(files: &[crate::model::PanelFileRecord]) -> Vec<String> {
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

fn detect_entry_points(files: &[crate::model::PanelFileRecord]) -> Vec<String> {
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

fn detect_mobile_components(files: &[crate::model::PanelFileRecord]) -> Vec<String> {
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

fn detect_mobile_configs(files: &[crate::model::PanelFileRecord]) -> Vec<String> {
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

fn is_stale(paths: &PanelPaths) -> Result<bool> {
    if !paths.meta.is_file() {
        return Ok(true);
    }
    let meta: PanelMeta = serde_json::from_str(&fs::read_to_string(&paths.meta)?)?;
    Ok(meta.worktree.head_sha != git_head(&paths.root))
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
    use crate::test_recommendation::recommend_panel_tests;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn index_project_panel_writes_panel_artifacts() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("src")).unwrap();
        fs::write(dir.path().join("src/lib.rs"), "pub struct Lease {}\n").unwrap();
        fs::write(dir.path().join("README.md"), "# Demo\n").unwrap();

        let status = index_project_panel(dir.path()).unwrap();

        assert_eq!(status.status, PanelStatus::Ready);
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
        assert_eq!(status.file_count, 3);
        assert!(status.symbol_count >= 2);
    }

    #[test]
    fn index_project_panel_blocks_foreign_agentflow_without_panel_writes() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".agentflow")).unwrap();
        fs::write(dir.path().join(".agentflow/custom.txt"), "foreign\n").unwrap();
        fs::write(dir.path().join("README.md"), "# Demo\n").unwrap();

        let error = index_project_panel(dir.path()).unwrap_err();

        assert!(error
            .to_string()
            .contains("workspace ownership blocks panel prepare"));
        assert!(!dir.path().join(".agentflow/panel").exists());
        assert_eq!(
            fs::read_to_string(dir.path().join(".agentflow/custom.txt")).unwrap(),
            "foreign\n"
        );
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

        index_project_panel(dir.path()).unwrap();
        let manifest = load_project_panel_manifest(dir.path()).unwrap();
        let hints = recommend_panel_tests(dir.path(), &[], &[], &[]).unwrap();

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
