use std::{
    fs,
    path::{Component, Path, PathBuf},
    time::UNIX_EPOCH,
};

#[derive(Debug)]
pub(crate) struct ProjectFileNode {
    pub(crate) metadata: fs::Metadata,
    pub(crate) is_symlink: bool,
}

pub(crate) fn resolve_agentflow_project_root(
    project_root: Option<String>,
) -> Result<PathBuf, String> {
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

pub(crate) fn sanitize_project_relative_path(
    root: &Path,
    relative_path: &str,
) -> Result<PathBuf, String> {
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

pub(crate) fn project_file_node(root: &Path, path: &Path) -> Result<ProjectFileNode, String> {
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

pub(crate) fn relative_project_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

pub(crate) fn modified_at_seconds(metadata: &fs::Metadata) -> Option<u64> {
    metadata
        .modified()
        .ok()
        .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs())
}

pub(crate) fn created_at_seconds(metadata: &fs::Metadata) -> Option<u64> {
    metadata
        .created()
        .ok()
        .and_then(|created| created.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs())
}
