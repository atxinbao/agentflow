use crate::model::{IssueLoopProjection, ProjectLoopSnapshot};
use anyhow::{Context, Result};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub fn prepare_loop_workspace(project_root: impl AsRef<Path>) -> Result<()> {
    let root = canonical_project_root(project_root)?;
    ensure_directory(&root.join(".agentflow/state/loops/projects"))?;
    ensure_directory(&root.join(".agentflow/state/loops/issues"))?;
    Ok(())
}

pub fn write_project_loop_snapshot(
    project_root: impl AsRef<Path>,
    snapshot: &ProjectLoopSnapshot,
) -> Result<PathBuf> {
    let root = canonical_project_root(project_root)?;
    prepare_loop_workspace(&root)?;
    let path = project_loop_path(&root, &snapshot.project_id);
    write_json(&path, snapshot)?;
    Ok(path)
}

pub fn read_project_loop_snapshot(
    project_root: impl AsRef<Path>,
    project_id: &str,
) -> Result<ProjectLoopSnapshot> {
    let root = canonical_project_root(project_root)?;
    read_json(&project_loop_path(&root, project_id))
}

pub fn write_issue_loop_projection(
    project_root: impl AsRef<Path>,
    projection: &IssueLoopProjection,
) -> Result<PathBuf> {
    let root = canonical_project_root(project_root)?;
    prepare_loop_workspace(&root)?;
    let path = issue_loop_path(&root, &projection.issue_id);
    write_json(&path, projection)?;
    Ok(path)
}

pub fn read_issue_loop_projection(
    project_root: impl AsRef<Path>,
    issue_id: &str,
) -> Result<IssueLoopProjection> {
    let root = canonical_project_root(project_root)?;
    read_json(&issue_loop_path(&root, issue_id))
}

fn project_loop_path(root: &Path, project_id: &str) -> PathBuf {
    root.join(".agentflow/state/loops/projects")
        .join(format!("{}.json", sanitize_id(project_id)))
}

fn issue_loop_path(root: &Path, issue_id: &str) -> PathBuf {
    root.join(".agentflow/state/loops/issues")
        .join(format!("{}.json", sanitize_id(issue_id)))
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
    project_root
        .as_ref()
        .canonicalize()
        .with_context(|| format!("canonicalize {}", project_root.as_ref().display()))
}

fn ensure_directory(path: &Path) -> Result<()> {
    fs::create_dir_all(path).with_context(|| format!("create {}", path.display()))
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

fn write_json<T: serde::Serialize>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        ensure_directory(parent)?;
    }
    fs::write(path, serde_json::to_string_pretty(value)?)
        .with_context(|| format!("write {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        model::{IssueLoopStage, ProjectLoopStatus},
        IssueLoop, ProjectLoop,
    };
    use tempfile::tempdir;

    #[test]
    fn writes_and_reads_project_loop_snapshot() {
        let dir = tempdir().unwrap();
        let mut snapshot = ProjectLoop::new("project-001").snapshot(100);
        snapshot.status = ProjectLoopStatus::Scheduling;

        let path = write_project_loop_snapshot(dir.path(), &snapshot).unwrap();
        assert!(path.ends_with(".agentflow/state/loops/projects/project-001.json"));

        let loaded = read_project_loop_snapshot(dir.path(), "project-001").unwrap();
        assert_eq!(loaded, snapshot);
    }

    #[test]
    fn writes_and_reads_issue_loop_projection() {
        let dir = tempdir().unwrap();
        let mut projection = IssueLoop::new("project-001", "AF-v020-001").projection(101);
        projection.stage = IssueLoopStage::Todo;
        projection.branch_name = Some("agentflow/project-001/AF-v020-001".to_string());

        let path = write_issue_loop_projection(dir.path(), &projection).unwrap();
        assert!(path.ends_with(".agentflow/state/loops/issues/AF-v020-001.json"));

        let loaded = read_issue_loop_projection(dir.path(), "AF-v020-001").unwrap();
        assert_eq!(loaded, projection);
    }
}
