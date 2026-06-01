use std::path::{Path, PathBuf};

pub(crate) fn canonical_project_root(project_root: &str) -> Result<PathBuf, String> {
    let trimmed = project_root.trim();
    if trimmed.is_empty() {
        return Err("project root is required".to_string());
    }
    let root = PathBuf::from(trimmed)
        .canonicalize()
        .map_err(|error| format!("canonicalize selected project root: {error}"))?;
    if !root.is_dir() {
        return Err("selected project root is not a directory".to_string());
    }
    Ok(root)
}

pub(crate) fn relative_or_display(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .ok()
        .and_then(|relative| relative.to_str())
        .map(str::to_string)
        .unwrap_or_else(|| path.display().to_string())
}
