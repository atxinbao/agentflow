use std::{fs, path::Path};

use super::{
    model::{ProjectFileChild, ProjectFileEntry},
    path_guard::{
        created_at_seconds, modified_at_seconds, project_file_node, relative_project_path,
    },
};

const DIRECTORY_CHILD_LIMIT: usize = 80;
pub(crate) const DIRECTORY_PAGE_DEFAULT_LIMIT: usize = 120;
pub(crate) const DIRECTORY_PAGE_MAX_LIMIT: usize = 400;

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
    "agent-artifacts",
];

pub(crate) fn normalize_project_file_view_mode(view_mode: Option<&str>) -> String {
    match view_mode.unwrap_or("source").trim() {
        "all" | "all-files" => "all".to_string(),
        "recent" => "recent".to_string(),
        _ => "source".to_string(),
    }
}

pub(crate) fn read_project_file_entries(
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

pub(crate) fn read_project_file_children(
    root: &Path,
    directory: &Path,
    view_mode: &str,
) -> Result<Vec<ProjectFileChild>, String> {
    Ok(read_project_file_child_entries(root, directory, view_mode)?
        .into_iter()
        .take(DIRECTORY_CHILD_LIMIT)
        .collect())
}

pub(crate) fn read_project_file_child_entries(
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

pub(crate) fn preferred_project_file_selection(entries: &[ProjectFileEntry]) -> Option<String> {
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
