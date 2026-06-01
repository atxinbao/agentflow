use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use super::{directory::read_project_file_child_entries, model::ProjectFileSearchResult};

pub(crate) const PROJECT_FILE_SEARCH_DEFAULT_LIMIT: usize = 80;
pub(crate) const PROJECT_FILE_SEARCH_MAX_LIMIT: usize = 300;

pub(crate) fn search_project_file_entries(
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

#[allow(clippy::too_many_arguments)]
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
