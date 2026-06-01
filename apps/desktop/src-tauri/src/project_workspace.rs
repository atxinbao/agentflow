use serde::Serialize;
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

const AGENTFLOW_DIR: &str = ".agentflow";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProjectWorkspaceSummary {
    version: String,
    id: String,
    name: String,
    root: String,
    agentflow_path: String,
    workspace_path: String,
    config_path: String,
    created_agentflow: bool,
    created_paths: Vec<String>,
    reused_paths: Vec<String>,
    git_exclude_path: Option<String>,
    protected_git_exclude: bool,
}

#[tauri::command]
pub(crate) fn prepare_local_project_workspace(
    project_root: String,
) -> Result<ProjectWorkspaceSummary, String> {
    prepare_local_project_workspace_at(&project_root)
}

fn prepare_local_project_workspace_at(
    project_root: &str,
) -> Result<ProjectWorkspaceSummary, String> {
    let root = canonical_project_root(project_root)?;
    let name = root
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("Local Project")
        .to_string();
    let agentflow_path = root.join(AGENTFLOW_DIR);
    let created_agentflow = !agentflow_path.exists();
    let mut created_paths = Vec::new();
    let mut reused_paths = Vec::new();

    ensure_directory(
        &agentflow_path,
        &root,
        &mut created_paths,
        &mut reused_paths,
    )?;
    for relative_path in [
        "define",
        "define/goals",
        "define/milestones",
        "define/issues",
        "execute",
        "execute/leases",
        "execute/runs",
        "execute/events",
        "output",
        "output/graph",
        "output/graph/context-packs",
        "output/graph/exports",
        "output/evidence",
        "output/logs",
        "output/cache",
        "output/tmp",
    ] {
        ensure_directory(
            &agentflow_path.join(relative_path),
            &root,
            &mut created_paths,
            &mut reused_paths,
        )?;
    }

    let workspace_path = agentflow_path.join("workspace.yaml");
    write_once(
        &workspace_path,
        &workspace_yaml(&name, &root),
        &root,
        &mut created_paths,
        &mut reused_paths,
    )?;

    let config_path = agentflow_path.join("config.yaml");
    write_once(
        &config_path,
        &config_yaml(),
        &root,
        &mut created_paths,
        &mut reused_paths,
    )?;

    let (git_exclude_path, protected_git_exclude) = protect_agentflow_from_git(&root)?;

    Ok(ProjectWorkspaceSummary {
        version: "project-workspace.v0".to_string(),
        id: format!("local:{}", root.display()),
        name,
        root: root.display().to_string(),
        agentflow_path: agentflow_path.display().to_string(),
        workspace_path: workspace_path.display().to_string(),
        config_path: config_path.display().to_string(),
        created_agentflow,
        created_paths,
        reused_paths,
        git_exclude_path: git_exclude_path.map(|path| path.display().to_string()),
        protected_git_exclude,
    })
}

fn canonical_project_root(project_root: &str) -> Result<PathBuf, String> {
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

fn ensure_directory(
    path: &Path,
    root: &Path,
    created_paths: &mut Vec<String>,
    reused_paths: &mut Vec<String>,
) -> Result<(), String> {
    if path.exists() {
        if !path.is_dir() {
            return Err(format!("{} exists but is not a directory", path.display()));
        }
        reused_paths.push(relative_or_display(root, path));
        return Ok(());
    }

    fs::create_dir_all(path).map_err(|error| format!("create {}: {error}", path.display()))?;
    created_paths.push(relative_or_display(root, path));
    Ok(())
}

fn write_once(
    path: &Path,
    content: &str,
    root: &Path,
    created_paths: &mut Vec<String>,
    reused_paths: &mut Vec<String>,
) -> Result<(), String> {
    if path.exists() {
        if !path.is_file() {
            return Err(format!("{} exists but is not a file", path.display()));
        }
        reused_paths.push(relative_or_display(root, path));
        return Ok(());
    }

    fs::write(path, content).map_err(|error| format!("write {}: {error}", path.display()))?;
    created_paths.push(relative_or_display(root, path));
    Ok(())
}

fn protect_agentflow_from_git(root: &Path) -> Result<(Option<PathBuf>, bool), String> {
    let git_dir = root.join(".git");
    if !git_dir.is_dir() {
        return Ok((None, false));
    }

    let info_dir = git_dir.join("info");
    fs::create_dir_all(&info_dir)
        .map_err(|error| format!("create {}: {error}", info_dir.display()))?;
    let exclude_path = info_dir.join("exclude");
    let current = fs::read_to_string(&exclude_path).unwrap_or_default();
    let already_protected = current
        .lines()
        .map(str::trim)
        .any(|line| line == ".agentflow/" || line == ".agentflow");
    if already_protected {
        return Ok((Some(exclude_path), true));
    }

    let mut next = current;
    if !next.is_empty() && !next.ends_with('\n') {
        next.push('\n');
    }
    next.push_str("\n# AgentFlow local runtime\n.agentflow/\n");
    fs::write(&exclude_path, next)
        .map_err(|error| format!("write {}: {error}", exclude_path.display()))?;
    Ok((Some(exclude_path), true))
}

fn workspace_yaml(name: &str, root: &Path) -> String {
    format!(
        "version: workspace.v0\nname: {}\nprojectRoot: {}\ncreatedBy: AgentFlow Desktop\ncreatedAt: {}\n",
        yaml_quote(name),
        yaml_quote(&root.display().to_string()),
        unix_timestamp_seconds()
    )
}

fn config_yaml() -> String {
    "version: config.v0\nmode: local\nagentflowDir: .agentflow\nworkflow:\n  define: define\n  execute: execute\n  output: output\n".to_string()
}

fn yaml_quote(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

fn unix_timestamp_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn relative_or_display(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .ok()
        .and_then(|relative| relative.to_str())
        .map(str::to_string)
        .unwrap_or_else(|| path.display().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn prepare_workspace_creates_agentflow_three_stage_tree_and_git_exclude() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".git/info")).unwrap();
        fs::write(dir.path().join(".git/info/exclude"), "*.log\n").unwrap();

        let summary =
            prepare_local_project_workspace_at(&dir.path().display().to_string()).unwrap();

        assert!(summary.created_agentflow);
        assert!(summary.protected_git_exclude);
        assert!(dir.path().join(".agentflow/workspace.yaml").is_file());
        assert!(dir.path().join(".agentflow/config.yaml").is_file());
        assert!(dir.path().join(".agentflow/define/goals").is_dir());
        assert!(dir.path().join(".agentflow/define/milestones").is_dir());
        assert!(dir.path().join(".agentflow/define/issues").is_dir());
        assert!(dir.path().join(".agentflow/execute/leases").is_dir());
        assert!(dir.path().join(".agentflow/execute/runs").is_dir());
        assert!(dir.path().join(".agentflow/execute/events").is_dir());
        assert!(dir.path().join(".agentflow/output/evidence").is_dir());
        assert!(dir.path().join(".agentflow/output/logs").is_dir());
        assert!(dir.path().join(".agentflow/output/cache").is_dir());
        assert!(dir.path().join(".agentflow/output/tmp").is_dir());
        assert!(fs::read_to_string(dir.path().join(".git/info/exclude"))
            .unwrap()
            .contains(".agentflow/"));
    }

    #[test]
    fn prepare_workspace_reuses_existing_files_without_overwriting() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".agentflow")).unwrap();
        fs::write(dir.path().join(".agentflow/config.yaml"), "custom: true\n").unwrap();

        let summary =
            prepare_local_project_workspace_at(&dir.path().display().to_string()).unwrap();

        assert!(!summary.created_agentflow);
        assert!(summary
            .reused_paths
            .iter()
            .any(|path| path == ".agentflow/config.yaml"));
        assert_eq!(
            fs::read_to_string(dir.path().join(".agentflow/config.yaml")).unwrap(),
            "custom: true\n"
        );
    }

    #[test]
    fn prepare_workspace_allows_non_git_project() {
        let dir = tempdir().unwrap();

        let summary =
            prepare_local_project_workspace_at(&dir.path().display().to_string()).unwrap();

        assert!(summary.created_agentflow);
        assert!(!summary.protected_git_exclude);
        assert!(summary.git_exclude_path.is_none());
        assert!(dir.path().join(".agentflow/workspace.yaml").is_file());
    }
}
