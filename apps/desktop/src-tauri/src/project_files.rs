use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
use serde::Serialize;
use std::{
    collections::HashSet,
    fs,
    io::Read,
    path::{Component, Path, PathBuf},
    time::UNIX_EPOCH,
};

const PROJECT_FILE_PREVIEW_LIMIT_BYTES: u64 = 512 * 1024;
const PROJECT_BINARY_PREVIEW_LIMIT_BYTES: usize = 4096;
const DIRECTORY_CHILD_LIMIT: usize = 80;
const DIRECTORY_PAGE_DEFAULT_LIMIT: usize = 120;
const DIRECTORY_PAGE_MAX_LIMIT: usize = 400;
const PROJECT_FILE_SEARCH_DEFAULT_LIMIT: usize = 80;
const PROJECT_FILE_SEARCH_MAX_LIMIT: usize = 300;
const PROJECT_FILE_TEXT_RANGE_DEFAULT_LINES: usize = 240;
const PROJECT_FILE_TEXT_RANGE_MAX_LINES: usize = 1200;

const SOURCE_VIEW_EXCLUDED_DIRS: &[&str] = &[
    ".git",
    ".agentflow",
    ".codex",
    "node_modules",
    "target",
    "dist",
    "build",
    ".next",
    ".turbo",
    "graphify-out",
];

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProjectFilesSnapshot {
    version: String,
    project_root: String,
    entries: Vec<ProjectFileEntry>,
    selected_path: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProjectFileEntry {
    name: String,
    relative_path: String,
    kind: String,
    created_at: Option<u64>,
    modified_at: Option<u64>,
    size_bytes: Option<u64>,
    extension: Option<String>,
    child_count: Option<usize>,
    is_symlink: bool,
    children: Vec<ProjectFileChild>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProjectFileChild {
    name: String,
    relative_path: String,
    kind: String,
    created_at: Option<u64>,
    modified_at: Option<u64>,
    size_bytes: Option<u64>,
    extension: Option<String>,
    child_count: Option<usize>,
    is_symlink: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProjectFileContent {
    relative_path: String,
    name: String,
    kind: String,
    created_at: Option<u64>,
    modified_at: Option<u64>,
    size_bytes: Option<u64>,
    extension: Option<String>,
    mime_type: Option<String>,
    language: String,
    content: Option<String>,
    binary_preview: Option<String>,
    data_url: Option<String>,
    truncated: bool,
    directory_children: Vec<ProjectFileChild>,
    unsupported_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProjectDirectoryPage {
    version: String,
    project_root: String,
    directory_path: String,
    entries: Vec<ProjectFileChild>,
    next_cursor: Option<String>,
    total_children: usize,
    limit: usize,
    view_mode: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProjectFileSearchSnapshot {
    version: String,
    project_root: String,
    query: String,
    view_mode: String,
    results: Vec<ProjectFileSearchResult>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProjectFileSearchResult {
    name: String,
    relative_path: String,
    kind: String,
    extension: Option<String>,
    modified_at: Option<u64>,
    size_bytes: Option<u64>,
    score: u32,
    match_reason: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProjectFileTextRange {
    version: String,
    project_root: String,
    relative_path: String,
    start_line: usize,
    end_line: usize,
    total_lines: usize,
    content: String,
    truncated: bool,
}

#[tauri::command]
pub(crate) fn load_project_files_snapshot(
    project_root: Option<String>,
    view_mode: Option<String>,
) -> Result<ProjectFilesSnapshot, String> {
    let root = resolve_agentflow_project_root(project_root)?;
    let view_mode = normalize_project_file_view_mode(view_mode.as_deref());
    let entries = read_project_file_entries(&root, &root, &view_mode)?;
    let selected_path = preferred_project_file_selection(&entries);

    Ok(ProjectFilesSnapshot {
        version: "project-files.v1".to_string(),
        project_root: root.display().to_string(),
        entries,
        selected_path,
    })
}

#[tauri::command]
pub(crate) fn load_project_file_content(
    relative_path: String,
    project_root: Option<String>,
) -> Result<ProjectFileContent, String> {
    let root = resolve_agentflow_project_root(project_root)?;
    let target = sanitize_project_relative_path(&root, &relative_path)?;
    read_project_file_content(&root, &target)
}

#[tauri::command]
pub(crate) fn load_project_directory_page(
    directory_path: Option<String>,
    cursor: Option<String>,
    limit: Option<usize>,
    view_mode: Option<String>,
    project_root: Option<String>,
) -> Result<ProjectDirectoryPage, String> {
    let root = resolve_agentflow_project_root(project_root)?;
    let view_mode = normalize_project_file_view_mode(view_mode.as_deref());
    let relative_path = directory_path.unwrap_or_default();
    let directory = if relative_path.trim().is_empty() {
        root.clone()
    } else {
        sanitize_project_relative_path(&root, &relative_path)?
    };
    if !directory.is_dir() {
        return Err("project directory page target is not a directory".to_string());
    }

    let offset = cursor
        .as_deref()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(0);
    let limit = limit
        .unwrap_or(DIRECTORY_PAGE_DEFAULT_LIMIT)
        .clamp(1, DIRECTORY_PAGE_MAX_LIMIT);
    let all_entries = read_project_file_child_entries(&root, &directory, &view_mode)?;
    let total_children = all_entries.len();
    let entries = all_entries
        .iter()
        .skip(offset)
        .take(limit)
        .cloned()
        .collect::<Vec<_>>();
    let next_offset = offset + entries.len();
    let next_cursor = if next_offset < total_children {
        Some(next_offset.to_string())
    } else {
        None
    };

    Ok(ProjectDirectoryPage {
        version: "project-directory-page.v1".to_string(),
        project_root: root.display().to_string(),
        directory_path: relative_project_path(&root, &directory),
        entries,
        next_cursor,
        total_children,
        limit,
        view_mode,
    })
}

#[tauri::command]
pub(crate) fn search_project_files(
    query: String,
    view_mode: Option<String>,
    limit: Option<usize>,
    project_root: Option<String>,
) -> Result<ProjectFileSearchSnapshot, String> {
    let root = resolve_agentflow_project_root(project_root)?;
    let view_mode = normalize_project_file_view_mode(view_mode.as_deref());
    let query = query.trim().to_string();
    let limit = limit
        .unwrap_or(PROJECT_FILE_SEARCH_DEFAULT_LIMIT)
        .clamp(1, PROJECT_FILE_SEARCH_MAX_LIMIT);
    let results = if query.is_empty() {
        Vec::new()
    } else {
        search_project_file_entries(&root, &query, &view_mode, limit)?
    };

    Ok(ProjectFileSearchSnapshot {
        version: "project-file-search.v1".to_string(),
        project_root: root.display().to_string(),
        query,
        view_mode,
        results,
    })
}

#[tauri::command]
pub(crate) fn load_project_file_text_range(
    relative_path: String,
    start_line: Option<usize>,
    line_count: Option<usize>,
    project_root: Option<String>,
) -> Result<ProjectFileTextRange, String> {
    let root = resolve_agentflow_project_root(project_root)?;
    let target = sanitize_project_relative_path(&root, &relative_path)?;
    if !target.is_file() {
        return Err("text range target is not a file".to_string());
    }
    let content = fs::read_to_string(&target)
        .map_err(|error| format!("read text range {}: {error}", target.display()))?;
    let lines = content.lines().collect::<Vec<_>>();
    let total_lines = lines.len();
    let start_line = start_line.unwrap_or(1).max(1);
    let line_count = line_count
        .unwrap_or(PROJECT_FILE_TEXT_RANGE_DEFAULT_LINES)
        .clamp(1, PROJECT_FILE_TEXT_RANGE_MAX_LINES);
    let start_index = start_line.saturating_sub(1).min(total_lines);
    let end_index = (start_index + line_count).min(total_lines);
    let range_content = lines[start_index..end_index].join("\n");

    Ok(ProjectFileTextRange {
        version: "project-file-text-range.v1".to_string(),
        project_root: root.display().to_string(),
        relative_path: relative_project_path(&root, &target),
        start_line,
        end_line: end_index,
        total_lines,
        content: range_content,
        truncated: end_index < total_lines,
    })
}

#[tauri::command]
pub(crate) fn choose_existing_project_folder() -> Result<Option<String>, String> {
    let Some(folder) = rfd::FileDialog::new()
        .set_title("选择现有项目文件夹")
        .pick_folder()
    else {
        return Ok(None);
    };

    let canonical_folder = folder
        .canonicalize()
        .map_err(|error| format!("canonicalize selected project folder: {error}"))?;

    if !canonical_folder.is_dir() {
        return Err("selected project path is not a directory".to_string());
    }

    Ok(Some(canonical_folder.to_string_lossy().into_owned()))
}

fn resolve_agentflow_project_root(project_root: Option<String>) -> Result<PathBuf, String> {
    if let Some(project_root) = project_root.filter(|value| !value.trim().is_empty()) {
        let requested_root = PathBuf::from(project_root);
        let canonical_root = requested_root
            .canonicalize()
            .map_err(|error| format!("canonicalize selected project root: {error}"))?;
        if !canonical_root.is_dir() {
            return Err("selected project root is not a directory".to_string());
        }
        return Ok(canonical_root);
    }

    let cwd = std::env::current_dir().map_err(|error| error.to_string())?;
    let mut cursor = Some(cwd.as_path());

    while let Some(path) = cursor {
        if path.join(".agentflow").is_dir() {
            return path
                .canonicalize()
                .map_err(|error| format!("canonicalize project root: {error}"));
        }
        cursor = path.parent();
    }

    cwd.canonicalize()
        .map_err(|error| format!("canonicalize current directory: {error}"))
}

fn sanitize_project_relative_path(root: &Path, relative_path: &str) -> Result<PathBuf, String> {
    let relative = Path::new(relative_path);
    if relative.is_absolute()
        || relative.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err("invalid project file path".to_string());
    }

    let target = root.join(relative);
    let canonical_root = root
        .canonicalize()
        .map_err(|error| format!("canonicalize project root: {error}"))?;
    let canonical_target = target
        .canonicalize()
        .map_err(|error| format!("canonicalize project file: {error}"))?;

    if !canonical_target.starts_with(&canonical_root) {
        return Err("project file path escapes workspace".to_string());
    }

    Ok(canonical_target)
}

#[derive(Debug)]
struct ProjectFileNode {
    metadata: fs::Metadata,
    is_symlink: bool,
}

fn project_file_node(root: &Path, path: &Path) -> Result<ProjectFileNode, String> {
    let symlink_metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("symlink metadata {}: {error}", path.display()))?;
    let is_symlink = symlink_metadata.file_type().is_symlink();
    let canonical_root = root
        .canonicalize()
        .map_err(|error| format!("canonicalize project root: {error}"))?;
    let canonical_target = path
        .canonicalize()
        .map_err(|error| format!("canonicalize project file: {error}"))?;
    if !canonical_target.starts_with(&canonical_root) {
        return Err("project file path escapes workspace".to_string());
    }
    let metadata =
        fs::metadata(path).map_err(|error| format!("metadata {}: {error}", path.display()))?;
    Ok(ProjectFileNode {
        metadata,
        is_symlink,
    })
}

fn project_file_child_from_path(
    root: &Path,
    path: &Path,
    view_mode: &str,
) -> Result<ProjectFileChild, String> {
    let node = project_file_node(root, path)?;
    let metadata = &node.metadata;
    let kind = if metadata.is_dir() {
        "directory"
    } else {
        "file"
    }
    .to_string();

    Ok(ProjectFileChild {
        name: path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("")
            .to_string(),
        relative_path: relative_project_path(root, path),
        kind,
        created_at: created_at_seconds(metadata),
        modified_at: modified_at_seconds(metadata),
        size_bytes: if metadata.is_file() {
            Some(metadata.len())
        } else {
            None
        },
        extension: path
            .extension()
            .and_then(|extension| extension.to_str())
            .map(str::to_string),
        child_count: if metadata.is_dir() {
            Some(project_directory_child_count(root, path, view_mode)?)
        } else {
            None
        },
        is_symlink: node.is_symlink,
    })
}

fn project_directory_child_count(
    root: &Path,
    directory: &Path,
    view_mode: &str,
) -> Result<usize, String> {
    let read_dir = match fs::read_dir(directory) {
        Ok(read_dir) => read_dir,
        Err(_) => return Ok(0),
    };
    let mut count = 0;
    for entry_result in read_dir {
        let Ok(entry) = entry_result else {
            continue;
        };
        if should_skip_project_file_path(root, &entry.path(), view_mode) {
            continue;
        }
        count += 1;
    }
    Ok(count)
}

fn normalize_project_file_view_mode(view_mode: Option<&str>) -> String {
    match view_mode.unwrap_or("source").trim() {
        "all" | "all-files" => "all".to_string(),
        "recent" => "recent".to_string(),
        _ => "source".to_string(),
    }
}

fn should_skip_project_file_path(root: &Path, path: &Path, view_mode: &str) -> bool {
    if view_mode == "all" || view_mode == "recent" {
        return false;
    }

    let relative_path = relative_project_path(root, path);
    relative_path.split('/').any(|part| {
        SOURCE_VIEW_EXCLUDED_DIRS
            .iter()
            .any(|excluded| excluded == &part)
    })
}

fn read_project_file_entries(
    root: &Path,
    directory: &Path,
    view_mode: &str,
) -> Result<Vec<ProjectFileEntry>, String> {
    let mut entries = Vec::new();
    let read_dir = fs::read_dir(directory)
        .map_err(|error| format!("read {}: {error}", directory.display()))?;

    for entry_result in read_dir {
        let entry = entry_result.map_err(|error| error.to_string())?;
        let path = entry.path();
        if should_skip_project_file_path(root, &path, view_mode) {
            continue;
        }
        if let Ok(project_entry) = project_file_entry_from_path(root, &path, view_mode) {
            entries.push(project_entry);
        }
    }

    entries.sort_by(|left, right| {
        let left_dir = left.kind == "directory";
        let right_dir = right.kind == "directory";
        right_dir
            .cmp(&left_dir)
            .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
    });

    Ok(entries)
}

fn project_file_entry_from_path(
    root: &Path,
    path: &Path,
    view_mode: &str,
) -> Result<ProjectFileEntry, String> {
    let node = project_file_node(root, path)?;
    let metadata = &node.metadata;
    let kind = if metadata.is_dir() {
        "directory"
    } else {
        "file"
    }
    .to_string();
    let children = if metadata.is_dir() {
        read_project_file_children(root, path, view_mode)?
    } else {
        Vec::new()
    };
    let child_count = if metadata.is_dir() {
        Some(project_directory_child_count(root, path, view_mode)?)
    } else {
        None
    };

    Ok(ProjectFileEntry {
        name: path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("")
            .to_string(),
        relative_path: relative_project_path(root, path),
        kind,
        created_at: created_at_seconds(metadata),
        modified_at: modified_at_seconds(metadata),
        size_bytes: if metadata.is_file() {
            Some(metadata.len())
        } else {
            None
        },
        extension: path
            .extension()
            .and_then(|extension| extension.to_str())
            .map(str::to_string),
        child_count,
        is_symlink: node.is_symlink,
        children,
    })
}

fn read_project_file_children(
    root: &Path,
    directory: &Path,
    view_mode: &str,
) -> Result<Vec<ProjectFileChild>, String> {
    Ok(read_project_file_child_entries(root, directory, view_mode)?
        .into_iter()
        .take(DIRECTORY_CHILD_LIMIT)
        .collect())
}

fn read_project_file_child_entries(
    root: &Path,
    directory: &Path,
    view_mode: &str,
) -> Result<Vec<ProjectFileChild>, String> {
    let mut children = Vec::new();
    let read_dir = match fs::read_dir(directory) {
        Ok(read_dir) => read_dir,
        Err(_) => return Ok(children),
    };

    for entry_result in read_dir {
        let entry = entry_result.map_err(|error| error.to_string())?;
        let path = entry.path();
        if should_skip_project_file_path(root, &path, view_mode) {
            continue;
        }
        if let Ok(child) = project_file_child_from_path(root, &path, view_mode) {
            children.push(child);
        }
    }

    children.sort_by(|left, right| {
        let left_dir = left.kind == "directory";
        let right_dir = right.kind == "directory";
        right_dir
            .cmp(&left_dir)
            .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
    });
    Ok(children)
}

fn search_project_file_entries(
    root: &Path,
    query: &str,
    view_mode: &str,
    limit: usize,
) -> Result<Vec<ProjectFileSearchResult>, String> {
    let normalized_query = parse_project_file_search_query(query);
    let mut results = Vec::new();
    let mut visited_directories = HashSet::new();
    search_project_file_entries_in_directory(
        root,
        root,
        &normalized_query.term,
        normalized_query.extension.as_deref(),
        view_mode,
        limit,
        &mut results,
        &mut visited_directories,
    )?;
    results.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.relative_path.cmp(&right.relative_path))
    });
    results.truncate(limit);
    Ok(results)
}

struct ProjectFileSearchQuery {
    term: String,
    extension: Option<String>,
}

fn parse_project_file_search_query(query: &str) -> ProjectFileSearchQuery {
    let normalized = query.trim().to_lowercase();
    if let Some(extension) = normalized.strip_prefix("ext:") {
        let extension = extension.trim_start_matches('.').trim();
        return ProjectFileSearchQuery {
            term: String::new(),
            extension: (!extension.is_empty()).then(|| extension.to_string()),
        };
    }
    if normalized.starts_with('.') && !normalized.contains('/') {
        let extension = normalized.trim_start_matches('.').trim();
        return ProjectFileSearchQuery {
            term: String::new(),
            extension: (!extension.is_empty()).then(|| extension.to_string()),
        };
    }

    ProjectFileSearchQuery {
        term: normalized,
        extension: None,
    }
}

fn search_project_file_entries_in_directory(
    root: &Path,
    directory: &Path,
    query: &str,
    extension_filter: Option<&str>,
    view_mode: &str,
    limit: usize,
    results: &mut Vec<ProjectFileSearchResult>,
    visited_directories: &mut HashSet<PathBuf>,
) -> Result<(), String> {
    if results.len() >= limit.saturating_mul(4) {
        return Ok(());
    }
    let canonical_directory = match directory.canonicalize() {
        Ok(path) => path,
        Err(_) => return Ok(()),
    };
    if !visited_directories.insert(canonical_directory) {
        return Ok(());
    }

    let children = read_project_file_child_entries(root, directory, view_mode)?;
    for child in children {
        let normalized_path = child.relative_path.to_lowercase();
        let normalized_name = child.name.to_lowercase();
        let child_extension = child.extension.as_deref().unwrap_or("").to_lowercase();
        let extension_matches =
            extension_filter.is_some_and(|extension| child_extension == extension);
        let text_score = if query.is_empty() {
            0
        } else if normalized_name == query {
            120
        } else if normalized_name.starts_with(query) {
            100
        } else if normalized_name.contains(query) {
            80
        } else if normalized_path.contains(query) {
            60
        } else if child_extension == query {
            45
        } else {
            0
        };
        let score = if extension_matches {
            text_score.max(95)
        } else {
            text_score
        };

        if score > 0 {
            results.push(ProjectFileSearchResult {
                name: child.name.clone(),
                relative_path: child.relative_path.clone(),
                kind: child.kind.clone(),
                extension: child.extension.clone(),
                modified_at: child.modified_at,
                size_bytes: child.size_bytes,
                score,
                match_reason: if extension_matches {
                    "extension".to_string()
                } else if normalized_name.contains(query) {
                    "name".to_string()
                } else {
                    "path".to_string()
                },
            });
        }

        if child.kind == "directory" && results.len() < limit.saturating_mul(4) {
            let child_path = root.join(&child.relative_path);
            let _ = search_project_file_entries_in_directory(
                root,
                &child_path,
                query,
                extension_filter,
                view_mode,
                limit,
                results,
                visited_directories,
            );
        }
    }
    Ok(())
}

fn read_project_file_content(root: &Path, path: &Path) -> Result<ProjectFileContent, String> {
    let node = project_file_node(root, path)?;
    let metadata = node.metadata;
    let relative_path = relative_project_path(root, path);
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("")
        .to_string();
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_string);
    let language = file_language(&name, extension.as_deref());
    let mime_type = file_mime_type(&name, extension.as_deref());

    if metadata.is_dir() {
        return Ok(ProjectFileContent {
            relative_path,
            name,
            kind: "directory".to_string(),
            created_at: created_at_seconds(&metadata),
            modified_at: modified_at_seconds(&metadata),
            size_bytes: None,
            extension,
            mime_type: Some("inode/directory".to_string()),
            language: "directory".to_string(),
            content: None,
            binary_preview: None,
            data_url: None,
            truncated: false,
            directory_children: read_project_file_children(root, path, "all")?,
            unsupported_reason: None,
        });
    }

    let truncated = metadata.len() > PROJECT_FILE_PREVIEW_LIMIT_BYTES;
    let mut file =
        fs::File::open(path).map_err(|error| format!("open {}: {error}", path.display()))?;
    let mut bytes = Vec::new();
    file.by_ref()
        .take(PROJECT_FILE_PREVIEW_LIMIT_BYTES)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("read {}: {error}", path.display()))?;

    if bytes.iter().any(|byte| *byte == 0) {
        let binary_preview = hex_preview(&bytes);
        let data_url = preview_data_url(mime_type.as_deref(), &bytes, truncated);
        return Ok(ProjectFileContent {
            relative_path,
            name,
            kind: "file".to_string(),
            created_at: created_at_seconds(&metadata),
            modified_at: modified_at_seconds(&metadata),
            size_bytes: Some(metadata.len()),
            extension,
            mime_type,
            language,
            content: None,
            binary_preview: Some(binary_preview),
            data_url,
            truncated,
            directory_children: Vec::new(),
            unsupported_reason: Some(binary_unsupported_reason(truncated)),
        });
    }

    let content = match String::from_utf8(bytes) {
        Ok(content) => content,
        Err(error) => {
            let bytes = error.into_bytes();
            let binary_preview = hex_preview(&bytes);
            let data_url = preview_data_url(mime_type.as_deref(), &bytes, truncated);
            return Ok(ProjectFileContent {
                relative_path,
                name,
                kind: "file".to_string(),
                created_at: created_at_seconds(&metadata),
                modified_at: modified_at_seconds(&metadata),
                size_bytes: Some(metadata.len()),
                extension,
                mime_type,
                language,
                content: None,
                binary_preview: Some(binary_preview),
                data_url,
                truncated,
                directory_children: Vec::new(),
                unsupported_reason: Some(
                    "文件不是 UTF-8 文本，已展示 metadata 和十六进制预览。".to_string(),
                ),
            });
        }
    };
    Ok(ProjectFileContent {
        relative_path,
        name,
        kind: "file".to_string(),
        created_at: created_at_seconds(&metadata),
        modified_at: modified_at_seconds(&metadata),
        size_bytes: Some(metadata.len()),
        extension,
        mime_type,
        language,
        content: Some(content),
        binary_preview: None,
        data_url: None,
        truncated,
        directory_children: Vec::new(),
        unsupported_reason: if truncated {
            Some("文件较大，当前展示前 512KB 文本预览。".to_string())
        } else {
            None
        },
    })
}

fn preferred_project_file_selection(entries: &[ProjectFileEntry]) -> Option<String> {
    [
        "README.md",
        "design.md",
        "Cargo.toml",
        "package.json",
        ".gitignore",
    ]
    .iter()
    .find_map(|candidate| {
        entries
            .iter()
            .find(|entry| entry.kind == "file" && entry.name == *candidate)
            .map(|entry| entry.relative_path.clone())
    })
    .or_else(|| {
        entries
            .iter()
            .find(|entry| entry.kind == "file")
            .map(|entry| entry.relative_path.clone())
    })
    .or_else(|| entries.first().map(|entry| entry.relative_path.clone()))
}

fn relative_project_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn modified_at_seconds(metadata: &fs::Metadata) -> Option<u64> {
    metadata
        .modified()
        .ok()
        .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs())
}

fn created_at_seconds(metadata: &fs::Metadata) -> Option<u64> {
    metadata
        .created()
        .ok()
        .and_then(|created| created.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs())
}

fn file_language(name: &str, extension: Option<&str>) -> String {
    match name {
        "Cargo.toml" | "Tauri.toml" => return "toml".to_string(),
        "package.json" | "tsconfig.json" => return "json".to_string(),
        "Dockerfile" | "dockerfile" => return "dockerfile".to_string(),
        "Makefile" | "makefile" => return "makefile".to_string(),
        ".gitignore" | ".env" | ".env.example" => return "config".to_string(),
        _ => {}
    }

    match extension.unwrap_or("").to_lowercase().as_str() {
        "md" | "markdown" => "markdown",
        "json" => "json",
        "jsonc" => "json",
        "toml" => "toml",
        "yaml" | "yml" => "yaml",
        "rs" => "rust",
        "ts" | "tsx" => "typescript",
        "js" | "jsx" => "javascript",
        "css" => "css",
        "html" => "html",
        "py" => "python",
        "go" => "go",
        "java" => "java",
        "kt" | "kts" => "kotlin",
        "swift" => "swift",
        "dart" => "dart",
        "c" | "h" => "c",
        "cc" | "cpp" | "cxx" | "hpp" | "hh" => "cpp",
        "cs" => "csharp",
        "php" => "php",
        "rb" => "ruby",
        "sql" => "sql",
        "ps1" | "psm1" => "powershell",
        "m" | "mm" => "objective-c",
        "gradle" => "gradle",
        "plist" => "xml",
        "xml" => "xml",
        "dockerfile" => "dockerfile",
        "sh" | "bash" | "zsh" => "shell",
        "txt" => "text",
        _ => "text",
    }
    .to_string()
}

fn file_mime_type(name: &str, extension: Option<&str>) -> Option<String> {
    let normalized_name = name.to_lowercase();
    let mime = match extension.unwrap_or("").to_lowercase().as_str() {
        "md" | "markdown" => "text/markdown",
        "json" | "jsonc" => "application/json",
        "toml" => "application/toml",
        "yaml" | "yml" => "application/yaml",
        "rs" | "ts" | "tsx" | "js" | "jsx" | "css" | "html" | "py" | "go" | "java" | "kt"
        | "kts" | "swift" | "dart" | "c" | "h" | "cc" | "cpp" | "cxx" | "hpp" | "hh" | "cs"
        | "php" | "rb" | "sql" | "ps1" | "psm1" | "m" | "mm" | "gradle" | "plist" | "xml"
        | "sh" | "bash" | "zsh" | "txt" => "text/plain",
        "csv" => "text/csv",
        "pdf" => "application/pdf",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "ogg" => "audio/ogg",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        _ if normalized_name == ".gitignore" => "text/plain",
        _ if normalized_name == "dockerfile" || normalized_name == "makefile" => "text/plain",
        _ => return None,
    };
    Some(mime.to_string())
}

fn preview_data_url(mime_type: Option<&str>, bytes: &[u8], truncated: bool) -> Option<String> {
    let mime_type = mime_type?;
    if truncated {
        return None;
    }
    if !(mime_type.starts_with("image/")
        || mime_type.starts_with("audio/")
        || mime_type.starts_with("video/")
        || mime_type == "application/pdf"
        || mime_type == "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
        || mime_type == "application/vnd.openxmlformats-officedocument.wordprocessingml.document")
    {
        return None;
    }
    Some(format!(
        "data:{};base64,{}",
        mime_type,
        BASE64_STANDARD.encode(bytes)
    ))
}

fn hex_preview(bytes: &[u8]) -> String {
    bytes
        .iter()
        .take(PROJECT_BINARY_PREVIEW_LIMIT_BYTES)
        .enumerate()
        .map(|(index, byte)| {
            if index % 16 == 0 {
                format!("\n{:08x}  {:02x}", index, byte)
            } else {
                format!(" {:02x}", byte)
            }
        })
        .collect::<String>()
        .trim()
        .to_string()
}

fn binary_unsupported_reason(truncated: bool) -> String {
    if truncated {
        "二进制文件较大，已展示 metadata 和前 4KB 十六进制预览。".to_string()
    } else {
        "二进制文件已展示 metadata 和十六进制预览。".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::SystemTime;

    fn temp_project_root(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time after unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("agentflow-project-files-{label}-{nonce}"));
        fs::create_dir_all(&root).expect("create temporary project root");
        root
    }

    fn cleanup_project_root(root: &Path) {
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn project_file_content_reads_text_metadata() {
        let root = temp_project_root("text");
        let readme = root.join("README.md");
        fs::write(&readme, "# AgentFlow\n\n只读文件阅读器。").expect("write readme");

        let content = read_project_file_content(&root, &readme).expect("read text project file");

        assert_eq!(content.relative_path, "README.md");
        assert_eq!(content.kind, "file");
        assert_eq!(content.language, "markdown");
        assert_eq!(content.mime_type.as_deref(), Some("text/markdown"));
        assert_eq!(content.data_url, None);
        assert_eq!(content.binary_preview, None);
        assert!(!content.truncated);
        assert!(content.content.expect("text content").contains("AgentFlow"));

        cleanup_project_root(&root);
    }

    #[test]
    fn project_directory_content_lists_children() {
        let root = temp_project_root("directory");
        let docs = root.join("docs");
        fs::create_dir_all(&docs).expect("create docs dir");
        fs::write(docs.join("guide.md"), "# Guide").expect("write guide");

        let content = read_project_file_content(&root, &docs).expect("read directory content");

        assert_eq!(content.relative_path, "docs");
        assert_eq!(content.kind, "directory");
        assert_eq!(content.mime_type.as_deref(), Some("inode/directory"));
        assert_eq!(content.language, "directory");
        assert_eq!(content.directory_children.len(), 1);
        assert_eq!(content.directory_children[0].relative_path, "docs/guide.md");

        cleanup_project_root(&root);
    }

    #[test]
    fn project_file_content_returns_binary_hex_fallback() {
        let root = temp_project_root("binary");
        let binary_file = root.join("artifact.bin");
        fs::write(&binary_file, [0_u8, 1, 2, 3, 255]).expect("write binary file");

        let content =
            read_project_file_content(&root, &binary_file).expect("read binary project file");

        assert_eq!(content.content, None);
        assert_eq!(content.data_url, None);
        assert!(content
            .binary_preview
            .expect("hex preview")
            .contains("00000000"));
        assert!(content
            .unsupported_reason
            .expect("binary unsupported reason")
            .contains("二进制文件"));

        cleanup_project_root(&root);
    }

    #[test]
    fn project_file_content_returns_data_urls_for_small_previewable_binary_files() {
        let root = temp_project_root("data-url");
        let cases: [(&str, &[u8], &str); 4] = [
            ("picture.png", b"\x89PNG\r\n\x1a\n\0", "data:image/png;base64,"),
            ("report.pdf", b"%PDF-1.4\n\0", "data:application/pdf;base64,"),
            (
                "sheet.xlsx",
                b"PK\x03\x04\0",
                "data:application/vnd.openxmlformats-officedocument.spreadsheetml.sheet;base64,",
            ),
            (
                "document.docx",
                b"PK\x03\x04\0",
                "data:application/vnd.openxmlformats-officedocument.wordprocessingml.document;base64,",
            ),
        ];

        for (name, bytes, expected_prefix) in cases {
            let path = root.join(name);
            fs::write(&path, bytes).expect("write previewable binary file");
            let content =
                read_project_file_content(&root, &path).expect("read previewable binary file");
            let data_url = content.data_url.expect("preview data url");
            assert!(
                data_url.starts_with(expected_prefix),
                "{name} data URL should start with {expected_prefix}, got {data_url}"
            );
            assert!(content.binary_preview.is_some());
            assert_eq!(content.content, None);
        }

        cleanup_project_root(&root);
    }

    #[test]
    fn project_file_content_marks_large_text_as_truncated() {
        let root = temp_project_root("large-text");
        let large_log = root.join("large.log");
        fs::write(&large_log, "AgentFlow large line\n".repeat(40_000))
            .expect("write large text file");

        let content = read_project_file_content(&root, &large_log).expect("read large text file");

        assert!(content.truncated);
        assert!(content
            .content
            .expect("large text preview")
            .contains("AgentFlow large line"));
        assert!(content
            .unsupported_reason
            .expect("large text reason")
            .contains("512KB"));

        cleanup_project_root(&root);
    }

    #[test]
    fn project_file_path_rejects_escape() {
        let root = temp_project_root("path");

        let result = sanitize_project_relative_path(&root, "../outside.txt");

        assert!(result.is_err());

        cleanup_project_root(&root);
    }

    #[test]
    fn project_directory_page_paginates_large_directories() {
        let root = temp_project_root("paging");
        for index in 0..150 {
            fs::write(root.join(format!("file-{index:03}.txt")), "page").expect("write paged file");
        }

        let first_page = load_project_directory_page(
            None,
            None,
            Some(80),
            Some("all".to_string()),
            Some(root.display().to_string()),
        )
        .expect("load first directory page");
        let second_page = load_project_directory_page(
            None,
            first_page.next_cursor.clone(),
            Some(80),
            Some("all".to_string()),
            Some(root.display().to_string()),
        )
        .expect("load second directory page");

        assert_eq!(first_page.entries.len(), 80);
        assert_eq!(first_page.total_children, 150);
        assert!(first_page.next_cursor.is_some());
        assert_eq!(second_page.entries.len(), 70);
        assert_eq!(second_page.next_cursor, None);

        cleanup_project_root(&root);
    }

    #[test]
    fn project_file_search_finds_matching_paths() {
        let root = temp_project_root("search");
        fs::create_dir_all(root.join("apps/desktop")).expect("create nested dir");
        fs::write(
            root.join("apps/desktop/App.tsx"),
            "export const App = null;",
        )
        .expect("write app");

        let snapshot = search_project_files(
            "App".to_string(),
            Some("all".to_string()),
            Some(20),
            Some(root.display().to_string()),
        )
        .expect("search project files");

        assert!(snapshot
            .results
            .iter()
            .any(|result| result.relative_path == "apps/desktop/App.tsx"));

        cleanup_project_root(&root);
    }

    #[test]
    fn project_file_search_filters_by_extension_query() {
        let root = temp_project_root("search-extension");
        fs::write(root.join("main.rs"), "fn main() {}").expect("write rust file");
        fs::write(root.join("README.md"), "# AgentFlow").expect("write markdown file");

        let snapshot = search_project_files(
            "ext:rs".to_string(),
            Some("all".to_string()),
            Some(20),
            Some(root.display().to_string()),
        )
        .expect("search by extension");

        assert!(snapshot
            .results
            .iter()
            .any(|result| result.relative_path == "main.rs" && result.match_reason == "extension"));
        assert!(!snapshot
            .results
            .iter()
            .any(|result| result.relative_path == "README.md"));

        cleanup_project_root(&root);
    }

    #[test]
    fn project_file_language_recognizes_mobile_and_backend_configs() {
        let cases = [
            ("AndroidManifest.xml", Some("xml"), "xml"),
            ("Info.plist", Some("plist"), "xml"),
            ("pubspec.yaml", Some("yaml"), "yaml"),
            ("build.gradle", Some("gradle"), "gradle"),
            ("query.sql", Some("sql"), "sql"),
            ("App.swift", Some("swift"), "swift"),
            ("main.dart", Some("dart"), "dart"),
            ("server.py", Some("py"), "python"),
            ("main.go", Some("go"), "go"),
            ("ViewController.m", Some("m"), "objective-c"),
        ];

        for (name, extension, expected) in cases {
            assert_eq!(file_language(name, extension), expected, "{name}");
        }
    }

    #[test]
    fn project_file_text_range_reads_selected_lines() {
        let root = temp_project_root("range");
        let file = root.join("long.txt");
        fs::write(&file, "one\ntwo\nthree\nfour\nfive\n").expect("write ranged file");

        let range = load_project_file_text_range(
            "long.txt".to_string(),
            Some(2),
            Some(2),
            Some(root.display().to_string()),
        )
        .expect("load text range");

        assert_eq!(range.start_line, 2);
        assert_eq!(range.end_line, 3);
        assert_eq!(range.total_lines, 5);
        assert_eq!(range.content, "two\nthree");
        assert!(range.truncated);

        cleanup_project_root(&root);
    }

    #[test]
    fn source_view_hides_generated_directories() {
        let root = temp_project_root("source-view");
        fs::create_dir_all(root.join("target")).expect("create target dir");
        fs::write(root.join("README.md"), "# Readme").expect("write readme");

        let entries = read_project_file_entries(&root, &root, "source").expect("read source view");

        assert!(entries.iter().any(|entry| entry.name == "README.md"));
        assert!(!entries.iter().any(|entry| entry.name == "target"));

        cleanup_project_root(&root);
    }

    #[cfg(unix)]
    #[test]
    fn symlink_inside_root_is_allowed_and_outside_root_is_rejected() {
        use std::os::unix::fs::symlink;

        let root = temp_project_root("symlink");
        let outside = temp_project_root("symlink-outside");
        fs::write(root.join("inside.txt"), "inside").expect("write inside");
        fs::write(outside.join("outside.txt"), "outside").expect("write outside");
        symlink(root.join("inside.txt"), root.join("inside-link.txt"))
            .expect("create inside symlink");
        symlink(outside.join("outside.txt"), root.join("outside-link.txt"))
            .expect("create outside symlink");

        let inside = sanitize_project_relative_path(&root, "inside-link.txt");
        let outside_result = sanitize_project_relative_path(&root, "outside-link.txt");

        assert!(inside.is_ok());
        assert!(outside_result.is_err());

        cleanup_project_root(&root);
        cleanup_project_root(&outside);
    }
}
