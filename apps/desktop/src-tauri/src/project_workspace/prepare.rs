use super::{
    git::{canonical_project_root, relative_or_display},
    ignore::protect_agentflow_from_git,
    model::ProjectWorkspaceSummary,
};
use std::{
    fs,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

const AGENTFLOW_DIR: &str = ".agentflow";

pub(crate) fn prepare_local_project_workspace_at(
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
    let ownership = agentflow_agent_manual::check_agentflow_workspace_ownership(&root)
        .map_err(|error| format!("check workspace ownership: {error}"))?;

    if !ownership.ready_for_prepare {
        let agent_manual_status = agentflow_agent_manual::validate_agent_working_manual(&root)
            .map_err(|error| format!("validate agent working manual: {error}"))?;
        return Ok(ProjectWorkspaceSummary {
            version: "project-workspace.v0".to_string(),
            id: format!("local:{}", root.display()),
            name,
            root: root.display().to_string(),
            agentflow_path: agentflow_path.display().to_string(),
            workspace_path: agentflow_path.join("workspace.yaml").display().to_string(),
            config_path: agentflow_path.join("config.yaml").display().to_string(),
            created_agentflow: false,
            created_paths,
            reused_paths,
            git_exclude_path: None,
            protected_git_exclude: false,
            ownership: ownership.clone(),
            agent_manual_status,
        });
    }

    let agent_manual_status = agentflow_agent_manual::prepare_agent_working_manual(&root)
        .map_err(|error| format!("prepare agent working manual: {error}"))?;
    if !agent_manual_status.ready {
        return Ok(ProjectWorkspaceSummary {
            version: "project-workspace.v0".to_string(),
            id: format!("local:{}", root.display()),
            name,
            root: root.display().to_string(),
            agentflow_path: agentflow_path.display().to_string(),
            workspace_path: agentflow_path.join("workspace.yaml").display().to_string(),
            config_path: agentflow_path.join("config.yaml").display().to_string(),
            created_agentflow: false,
            created_paths,
            reused_paths,
            git_exclude_path: None,
            protected_git_exclude: false,
            ownership: agent_manual_status.ownership.clone(),
            agent_manual_status,
        });
    }

    ensure_directory(
        &agentflow_path,
        &root,
        &mut created_paths,
        &mut reused_paths,
    )?;
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
    let ownership = agent_manual_status.ownership.clone();

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
        ownership,
        agent_manual_status,
    })
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

fn workspace_yaml(name: &str, root: &Path) -> String {
    format!(
        "version: workspace.v0\nname: {}\nprojectRoot: {}\ncreatedBy: AgentFlow Desktop\ncreatedAt: {}\n",
        yaml_quote(name),
        yaml_quote(&root.display().to_string()),
        unix_timestamp_seconds()
    )
}

fn config_yaml() -> String {
    "version: config.v1\nmode: local\nagentflowDir: .agentflow\nworkflow:\n  define: define\n  spec: spec\n  goalTree: goal-tree\n  panel: panel\n  execute: execute\n  output: output\n  state: state\n".to_string()
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
        assert!(dir
            .path()
            .join(".agentflow/workspace-manifest.json")
            .is_file());
        assert!(dir.path().join(".agentflow/define/spec/SPEC.md").is_file());
        assert!(dir.path().join(".agentflow/define/tdd/TDD.md").is_file());
        assert!(dir
            .path()
            .join(".agentflow/define/release/RELEASE.md")
            .is_file());
        assert!(dir
            .path()
            .join(".agentflow/define/audit/AUDIT.md")
            .is_file());
        assert!(dir.path().join(".agentflow/spec/changes").is_dir());
        assert!(dir.path().join(".agentflow/goal-tree/goals").is_dir());
        assert!(dir.path().join(".agentflow/panel/context-packs").is_dir());
        assert!(dir.path().join(".agentflow/execute/leases").is_dir());
        assert!(dir.path().join(".agentflow/execute/runs").is_dir());
        assert!(dir.path().join(".agentflow/execute/commands").is_dir());
        assert!(dir.path().join(".agentflow/output/evidence").is_dir());
        assert!(dir.path().join(".agentflow/output/audit").is_dir());
        assert!(dir.path().join(".agentflow/output/logs").is_dir());
        assert!(dir.path().join(".agentflow/output/cache").is_dir());
        assert!(dir.path().join(".agentflow/output/tmp").is_dir());
        assert!(dir.path().join(".agentflow/state/health").is_dir());
        assert!(dir.path().join("AGENTS.md").is_file());
        assert!(dir
            .path()
            .join(".agentflow/define/agent/Agentflow.md")
            .is_file());
        assert!(dir
            .path()
            .join(".agentflow/define/agent/skills-lock.json")
            .is_file());
        assert!(summary.agent_manual_status.ready);
        assert!(summary.agent_manual_status.workspace_manifest.valid);
        assert!(summary.agent_manual_status.layout.ready);
        assert!(fs::read_to_string(dir.path().join(".git/info/exclude"))
            .unwrap()
            .contains(".agentflow/"));
    }

    #[test]
    fn prepare_workspace_reuses_existing_files_without_overwriting() {
        let dir = tempdir().unwrap();
        prepare_local_project_workspace_at(&dir.path().display().to_string()).unwrap();
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
        assert!(summary.agent_manual_status.ready);
    }

    #[test]
    fn prepare_workspace_blocks_foreign_agentflow_without_writing() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".agentflow")).unwrap();
        fs::write(dir.path().join(".agentflow/config.yaml"), "foreign: true\n").unwrap();

        let summary =
            prepare_local_project_workspace_at(&dir.path().display().to_string()).unwrap();

        assert!(!summary.agent_manual_status.ready);
        assert_eq!(
            summary.ownership.status,
            agentflow_agent_manual::model::WorkspaceOwnershipState::Foreign
        );
        assert!(!dir.path().join("AGENTS.md").exists());
        assert!(!dir
            .path()
            .join(".agentflow/workspace-manifest.json")
            .exists());
        assert_eq!(
            fs::read_to_string(dir.path().join(".agentflow/config.yaml")).unwrap(),
            "foreign: true\n"
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
