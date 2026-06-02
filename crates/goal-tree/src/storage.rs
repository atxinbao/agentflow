use crate::model::{GoalRecord, GoalTreeIndex, IssueRecord, MilestoneRecord};
use anyhow::{Context, Result};
use serde::{de::DeserializeOwned, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

pub(crate) const GOAL_TREE_JSON: &str = ".agentflow/define/goal-tree.json";

pub(crate) struct GoalTreePaths {
    pub(crate) root: PathBuf,
    pub(crate) define: PathBuf,
    pub(crate) goals: PathBuf,
    pub(crate) milestones: PathBuf,
    pub(crate) issues: PathBuf,
    pub(crate) exports: PathBuf,
    pub(crate) output_context_packs: PathBuf,
    pub(crate) index: PathBuf,
}

pub(crate) fn resolve_project_root(project_root: impl AsRef<Path>) -> Result<PathBuf> {
    let root = project_root
        .as_ref()
        .canonicalize()
        .with_context(|| format!("canonicalize {}", project_root.as_ref().display()))?;
    if !root.is_dir() {
        anyhow::bail!("project root is not a directory: {}", root.display());
    }
    Ok(root)
}

pub(crate) fn paths_for(project_root: impl AsRef<Path>) -> Result<GoalTreePaths> {
    let root = resolve_project_root(project_root)?;
    let define = root.join(".agentflow/define");
    Ok(GoalTreePaths {
        goals: define.join("goals"),
        milestones: define.join("milestones"),
        issues: define.join("issues"),
        exports: define.join("exports"),
        output_context_packs: root.join(".agentflow/output/graph/context-packs"),
        index: root.join(GOAL_TREE_JSON),
        define,
        root,
    })
}

pub(crate) fn ensure_goal_tree_dirs(paths: &GoalTreePaths) -> Result<()> {
    for directory in [
        &paths.define,
        &paths.goals,
        &paths.milestones,
        &paths.issues,
        &paths.exports,
        &paths.output_context_packs,
    ] {
        fs::create_dir_all(directory).with_context(|| format!("create {}", directory.display()))?;
    }
    Ok(())
}

pub(crate) fn load_index(paths: &GoalTreePaths) -> Result<GoalTreeIndex> {
    if !paths.index.is_file() {
        return Ok(default_index(paths));
    }
    read_json(&paths.index)
}

pub(crate) fn save_index(paths: &GoalTreePaths, index: &GoalTreeIndex) -> Result<()> {
    write_json_atomic(&paths.index, index)
}

pub(crate) fn default_index(paths: &GoalTreePaths) -> GoalTreeIndex {
    GoalTreeIndex {
        version: "goal-tree.v1".to_string(),
        project_root: paths.root.display().to_string(),
        active_goal_id: None,
        goal_order: Vec::new(),
        milestone_order_by_goal: BTreeMap::new(),
        issue_order_by_milestone: BTreeMap::new(),
        updated_at: unix_timestamp_seconds(),
    }
}

pub(crate) fn load_goals(paths: &GoalTreePaths) -> Result<Vec<GoalRecord>> {
    load_records(&paths.goals)
}

pub(crate) fn load_milestones(paths: &GoalTreePaths) -> Result<Vec<MilestoneRecord>> {
    load_records(&paths.milestones)
}

pub(crate) fn load_issues(paths: &GoalTreePaths) -> Result<Vec<IssueRecord>> {
    load_records(&paths.issues)
}

pub(crate) fn read_goal(paths: &GoalTreePaths, goal_id: &str) -> Result<GoalRecord> {
    read_json(&paths.goals.join(format!("{goal_id}.json")))
}

pub(crate) fn read_milestone(paths: &GoalTreePaths, milestone_id: &str) -> Result<MilestoneRecord> {
    read_json(&paths.milestones.join(format!("{milestone_id}.json")))
}

pub(crate) fn read_issue(paths: &GoalTreePaths, issue_id: &str) -> Result<IssueRecord> {
    read_json(&paths.issues.join(format!("{issue_id}.json")))
}

pub(crate) fn save_goal(paths: &GoalTreePaths, goal: &GoalRecord) -> Result<()> {
    write_json_atomic(&paths.goals.join(format!("{}.json", goal.id)), goal)
}

pub(crate) fn save_milestone(paths: &GoalTreePaths, milestone: &MilestoneRecord) -> Result<()> {
    write_json_atomic(
        &paths.milestones.join(format!("{}.json", milestone.id)),
        milestone,
    )
}

pub(crate) fn save_issue(paths: &GoalTreePaths, issue: &IssueRecord) -> Result<()> {
    write_json_atomic(&paths.issues.join(format!("{}.json", issue.id)), issue)
}

pub(crate) fn relative_record_path(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .ok()
        .and_then(|value| value.to_str())
        .map(str::to_string)
        .unwrap_or_else(|| path.display().to_string())
}

pub(crate) fn unix_timestamp_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

pub(crate) fn bump_record_revision(created_at: u64, revision: u64) -> (u64, u64, String) {
    let updated_at = unix_timestamp_seconds().max(created_at);
    (
        updated_at,
        revision.saturating_add(1),
        "agent-system".to_string(),
    )
}

fn load_records<T>(directory: &Path) -> Result<Vec<T>>
where
    T: DeserializeOwned,
{
    if !directory.is_dir() {
        return Ok(Vec::new());
    }
    let mut files = fs::read_dir(directory)
        .with_context(|| format!("read {}", directory.display()))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("json"))
        .collect::<Vec<_>>();
    files.sort();
    files.into_iter().map(|path| read_json(&path)).collect()
}

fn read_json<T>(path: &Path) -> Result<T>
where
    T: DeserializeOwned,
{
    let content = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&content).with_context(|| format!("parse {}", path.display()))
}

fn write_json_atomic<T>(path: &Path, value: &T) -> Result<()>
where
    T: Serialize,
{
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let tmp_path = path.with_extension("json.tmp");
    let content = serde_json::to_string_pretty(value)?;
    fs::write(&tmp_path, format!("{content}\n"))
        .with_context(|| format!("write {}", tmp_path.display()))?;
    fs::rename(&tmp_path, path)
        .with_context(|| format!("rename {} -> {}", tmp_path.display(), path.display()))?;
    Ok(())
}
