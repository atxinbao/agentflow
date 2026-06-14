use crate::model::{
    IssueStatusIndex, ProjectProjection, TaskProjection, PROJECT_PROJECTION_VERSION,
};
use anyhow::{Context, Result};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub fn prepare_projection_workspace(project_root: impl AsRef<Path>) -> Result<()> {
    let root = canonical_project_root(project_root)?;
    ensure_directory(&root.join(".agentflow/projections/tasks"))?;
    ensure_directory(&root.join(".agentflow/projections/projects"))?;
    ensure_directory(&root.join(".agentflow/indexes"))?;
    Ok(())
}

pub fn write_task_projection(
    project_root: impl AsRef<Path>,
    projection: &TaskProjection,
) -> Result<PathBuf> {
    let root = canonical_project_root(project_root)?;
    prepare_projection_workspace(&root)?;
    let path = task_projection_path(&root, &projection.issue_id);
    write_json(&path, projection)?;
    Ok(path)
}

pub fn write_project_projection(
    project_root: impl AsRef<Path>,
    projection: &ProjectProjection,
) -> Result<PathBuf> {
    let root = canonical_project_root(project_root)?;
    prepare_projection_workspace(&root)?;
    let path = project_projection_path(&root, &projection.project_id);
    write_json(&path, projection)?;
    Ok(path)
}

pub fn write_issue_status_index(
    project_root: impl AsRef<Path>,
    index: &IssueStatusIndex,
) -> Result<PathBuf> {
    let root = canonical_project_root(project_root)?;
    prepare_projection_workspace(&root)?;
    let path = root.join(".agentflow/indexes/issue-status.json");
    write_json(&path, index)?;
    Ok(path)
}

pub fn load_task_projection(
    project_root: impl AsRef<Path>,
    issue_id: &str,
) -> Result<TaskProjection> {
    let root = canonical_project_root(project_root)?;
    read_json(&task_projection_path(&root, issue_id))
}

pub fn load_project_projection(
    project_root: impl AsRef<Path>,
    project_id: &str,
) -> Result<ProjectProjection> {
    let root = canonical_project_root(project_root)?;
    read_json(&project_projection_path(&root, project_id))
}

fn task_projection_path(root: &Path, issue_id: &str) -> PathBuf {
    root.join(".agentflow/projections/tasks")
        .join(format!("{}.json", sanitize_id(issue_id)))
}

fn project_projection_path(root: &Path, project_id: &str) -> PathBuf {
    root.join(".agentflow/projections/projects")
        .join(format!("{}.json", sanitize_id(project_id)))
}

fn sanitize_id(id: &str) -> String {
    id.chars()
        .map(|ch| match ch {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => ch,
        })
        .collect()
}

fn canonical_project_root(project_root: impl AsRef<Path>) -> Result<PathBuf> {
    let root = project_root.as_ref();
    if root.exists() {
        return root
            .canonicalize()
            .with_context(|| format!("canonicalize {}", root.display()));
    }
    Ok(root.to_path_buf())
}

fn ensure_directory(path: &Path) -> Result<()> {
    fs::create_dir_all(path).with_context(|| format!("create {}", path.display()))
}

fn write_json<T: serde::Serialize>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        ensure_directory(parent)?;
    }
    fs::write(path, serde_json::to_string_pretty(value)? + "\n")
        .with_context(|| format!("write {}", path.display()))
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

#[allow(dead_code)]
fn _assert_project_version_used() -> &'static str {
    PROJECT_PROJECTION_VERSION
}
