use anyhow::{Context, Result};
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IssueId(String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProjectId(String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ReleaseId(String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RunId(String);

impl IssueId {
    pub fn parse(value: impl AsRef<str>) -> Result<Self> {
        let value = value.as_ref();
        validate_safe_local_id("issueId", value)?;
        let Some((prefix, number)) = value.rsplit_once('-') else {
            anyhow::bail!("issueId must use <prefix>-<number>, found {value}");
        };
        if prefix.trim().is_empty() || number.trim().is_empty() {
            anyhow::bail!("issueId must use <prefix>-<number>, found {value}");
        }
        if !number.chars().all(|ch| ch.is_ascii_digit()) {
            anyhow::bail!("issueId numeric suffix must be digits, found {value}");
        }
        Ok(Self(value.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl ProjectId {
    pub fn parse(value: impl AsRef<str>) -> Result<Self> {
        let value = value.as_ref();
        validate_safe_local_id("projectId", value)?;
        Ok(Self(value.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl ReleaseId {
    pub fn parse(value: impl AsRef<str>) -> Result<Self> {
        let value = value.as_ref();
        validate_safe_local_id("releaseId", value)?;
        Ok(Self(value.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl RunId {
    pub fn parse(value: impl AsRef<str>) -> Result<Self> {
        let value = value.as_ref();
        validate_safe_local_id("runId", value)?;
        Ok(Self(value.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

pub fn validate_safe_local_id(field: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        anyhow::bail!("{field} is required");
    }
    if value != value.trim() {
        anyhow::bail!("{field} must not contain leading or trailing whitespace");
    }
    if value == "." || value == ".." {
        anyhow::bail!("{field} must be a safe local id, found {value}");
    }
    if value.contains("..")
        || value.contains('/')
        || value.contains('\\')
        || value.contains(':')
        || value.contains('*')
        || value.contains('?')
        || value.contains('"')
        || value.contains('<')
        || value.contains('>')
        || value.contains('|')
    {
        anyhow::bail!("{field} must be a safe local id, found {value}");
    }
    if !value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.'))
    {
        anyhow::bail!("{field} must use only letters, digits, '.', '-' or '_', found {value}");
    }
    Ok(())
}

pub fn canonicalize_project_root(project_root: impl AsRef<Path>) -> Result<PathBuf> {
    let root = project_root.as_ref();
    if root.exists() {
        return root
            .canonicalize()
            .with_context(|| format!("canonicalize {}", root.display()));
    }
    Ok(root.to_path_buf())
}

pub fn normalize_relative_path(path: impl AsRef<Path>) -> Result<PathBuf> {
    let path = path.as_ref();
    if path.as_os_str().is_empty() {
        anyhow::bail!("relative path is required");
    }
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => normalized.push(part),
            Component::CurDir => {}
            Component::ParentDir => {
                anyhow::bail!("relative path must not contain parent traversal")
            }
            Component::RootDir | Component::Prefix(_) => {
                anyhow::bail!("relative path must not be absolute")
            }
        }
    }
    if normalized.as_os_str().is_empty() {
        anyhow::bail!("relative path is required");
    }
    Ok(normalized)
}

pub fn normalize_relative_path_string(path: impl AsRef<Path>) -> Result<String> {
    Ok(path_to_forward_slashes(&normalize_relative_path(path)?))
}

pub fn join_relative_path(root: &Path, relative: impl AsRef<Path>) -> Result<PathBuf> {
    Ok(root.join(normalize_relative_path(relative)?))
}

pub fn normalize_relative_to_root(root: &Path, path: impl AsRef<Path>) -> Result<String> {
    let path = path.as_ref();
    let absolute = if path.is_absolute() {
        canonicalize_with_fallback(path)?
    } else {
        join_relative_path(root, path)?
    };
    let relative = absolute
        .strip_prefix(root)
        .with_context(|| format!("{} is outside {}", absolute.display(), root.display()))?;
    Ok(path_to_forward_slashes(relative))
}

fn canonicalize_with_fallback(path: &Path) -> Result<PathBuf> {
    if path.exists() {
        return path
            .canonicalize()
            .with_context(|| format!("canonicalize {}", path.display()));
    }
    if let Some(parent) = path.parent() {
        if parent.exists() {
            let canonical_parent = parent
                .canonicalize()
                .with_context(|| format!("canonicalize {}", parent.display()))?;
            if let Some(file_name) = path.file_name() {
                return Ok(canonical_parent.join(file_name));
            }
        }
    }
    Ok(path.to_path_buf())
}

fn path_to_forward_slashes(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn issue_id_requires_numeric_suffix() {
        let err = IssueId::parse("AF-TASK-XYZ").unwrap_err().to_string();
        assert!(err.contains("numeric suffix"));
    }

    #[test]
    fn safe_local_ids_reject_escape_fragments() {
        let err = ProjectId::parse("../project").unwrap_err().to_string();
        assert!(err.contains("safe local id"));
        let err = RunId::parse("run/001").unwrap_err().to_string();
        assert!(err.contains("safe local id"));
    }

    #[test]
    fn release_id_allows_version_dots() {
        let release_id = ReleaseId::parse("release-v0.3.0").unwrap();
        assert_eq!(release_id.as_str(), "release-v0.3.0");
    }

    #[test]
    fn normalize_relative_path_rejects_absolute_and_parent_segments() {
        let err = normalize_relative_path("../escape.json")
            .unwrap_err()
            .to_string();
        assert!(err.contains("parent traversal"));
        let err = normalize_relative_path("/tmp/outside.json")
            .unwrap_err()
            .to_string();
        assert!(err.contains("must not be absolute"));
    }

    #[test]
    fn join_relative_path_and_normalize_to_root_round_trip() {
        let dir = tempdir().unwrap();
        let root = canonicalize_project_root(dir.path()).unwrap();
        let joined = join_relative_path(
            &root,
            PathBuf::from(".agentflow")
                .join("tasks")
                .join("AF-TASK-001")
                .join("runs")
                .join("run-001")
                .join("run.json"),
        )
        .unwrap();
        assert_eq!(
            normalize_relative_to_root(&root, &joined).unwrap(),
            ".agentflow/tasks/AF-TASK-001/runs/run-001/run.json"
        );
    }

    #[test]
    fn normalize_relative_to_root_rejects_outside_absolute_path() {
        let root = canonicalize_project_root(tempdir().unwrap().path()).unwrap();
        let outside = tempdir().unwrap();
        let outside_path = outside.path().join("escape.json");
        std::fs::write(&outside_path, "{}\n").unwrap();
        let err = normalize_relative_to_root(&root, &outside_path)
            .unwrap_err()
            .to_string();
        assert!(err.contains("outside"));
    }
}
