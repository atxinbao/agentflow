use super::{
    base_release::initialize_base_release_project,
    git::{canonical_project_root, relative_or_display},
    ignore::{protect_agentflow_from_git, protect_agents_md_from_gitignore},
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
    app_locale: Option<String>,
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
            agents_gitignore_path: None,
            protected_agents_gitignore: false,
            agents_md_tracked_by_git: false,
            agents_md_git_warning: None,
            ownership: ownership.clone(),
            agent_manual_status,
            state_status: None,
            initialization_status: None,
        });
    }

    let agent_manual_status =
        agentflow_agent_manual::prepare_agent_working_manual_with_locale(&root, app_locale.clone())
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
            agents_gitignore_path: None,
            protected_agents_gitignore: false,
            agents_md_tracked_by_git: false,
            agents_md_git_warning: None,
            ownership: agent_manual_status.ownership.clone(),
            agent_manual_status,
            state_status: None,
            initialization_status: None,
        });
    }

    agentflow_spec::prepare_spec_workspace(&root)
        .map_err(|error| format!("prepare spec workspace: {error}"))?;
    agentflow_projection::prepare_projection_workspace(&root)
        .map_err(|error| format!("prepare projection workspace: {error}"))?;

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
    let agents_gitignore = protect_agents_md_from_gitignore(&root)?;
    let ownership = agent_manual_status.ownership.clone();
    let initialization_status = initialize_base_release_project(&root)
        .map_err(|error| format!("initialize base release workspace: {error}"))?;
    let state_status = agentflow_state::prepare_state_workspace(&root)
        .map_err(|error| format!("prepare workflow state: {error}"))?;

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
        agents_gitignore_path: Some(agents_gitignore.gitignore_path.display().to_string()),
        protected_agents_gitignore: agents_gitignore.protected,
        agents_md_tracked_by_git: agents_gitignore.tracked_by_git,
        agents_md_git_warning: agents_gitignore.warning,
        ownership,
        agent_manual_status,
        state_status: Some(state_status),
        initialization_status: Some(initialization_status),
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
    "version: config.v1\nmode: local\nagentflowDir: .agentflow\nworkflow:\n  define: define\n  panel: panel\n  spec: spec\n  events: events\n  projections: projections\n  tasks: tasks\n  state: state\n".to_string()
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
    use std::process::Command;
    use tempfile::tempdir;

    #[test]
    fn prepare_workspace_creates_agentflow_three_stage_tree_and_git_exclude() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".git/info")).unwrap();
        fs::write(dir.path().join(".git/info/exclude"), "*.log\n").unwrap();

        let summary =
            prepare_local_project_workspace_at(&dir.path().display().to_string(), None).unwrap();

        assert!(summary.created_agentflow);
        assert!(summary.protected_git_exclude);
        let initialization_status = summary.initialization_status.as_ref().unwrap();
        assert_eq!(initialization_status.project_kind, "existing");
        assert!(!initialization_status.demo_data_created);
        assert_eq!(initialization_status.demo_issue_count, 0);
        assert_eq!(initialization_status.demo_delivery_count, 0);
        assert_eq!(initialization_status.demo_audit_count, 0);
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
        assert!(!dir.path().join(".agentflow/input").exists());
        assert!(!dir
            .path()
            .join(".agentflow/spec/issues/AF-DEMO-001.json")
            .exists());
        assert!(!dir
            .path()
            .join(".agentflow/tasks/DEL-DEMO-001/evidence/evidence.json")
            .exists());
        assert!(!dir
            .path()
            .join(".agentflow/audit/AUD-DEMO-001/audit-report.md")
            .exists());
        assert!(dir
            .path()
            .join(".agentflow/state/indexes/base-release-initialization.json")
            .is_file());
        assert!(dir.path().join(".agentflow/spec/manifest.json").is_file());
        assert!(dir.path().join(".agentflow/spec/index.json").is_file());
        assert!(dir.path().join(".agentflow/spec/projects").is_dir());
        assert!(dir.path().join(".agentflow/spec/issues").is_dir());
        assert!(dir.path().join(".agentflow/projections/tasks").is_dir());
        assert!(dir.path().join(".agentflow/projections/projects").is_dir());
        assert!(dir.path().join(".agentflow/indexes").is_dir());
        assert!(dir.path().join(".agentflow/tasks").is_dir());
        assert!(!dir.path().join(".agentflow/goal-tree").exists());
        assert!(dir.path().join(".agentflow/panel/context-packs").is_dir());
        assert!(!dir.path().join(".agentflow/execute").exists());
        assert!(!dir.path().join(".agentflow/output/evidence").exists());
        assert!(dir.path().join(".agentflow/audit").is_dir());
        assert!(!dir.path().join(".agentflow/output/release").exists());
        assert!(!dir.path().join(".agentflow/output/logs").exists());
        assert!(!dir.path().join(".agentflow/output/cache").exists());
        assert!(!dir.path().join(".agentflow/output/tmp").exists());
        assert!(dir.path().join(".agentflow/state/health").is_dir());
        assert!(dir.path().join(".agentflow/state/manifest.json").is_file());
        assert!(dir.path().join(".agentflow/state/index.json").is_file());
        assert!(dir
            .path()
            .join(".agentflow/state/gates/workflow.json")
            .is_file());
        assert!(dir
            .path()
            .join(".agentflow/state/events/timeline.jsonl")
            .is_file());
        assert!(summary.state_status.is_some());
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
        assert!(fs::read_to_string(dir.path().join(".gitignore"))
            .unwrap()
            .contains("AGENTS.md"));
    }

    #[test]
    fn prepare_workspace_existing_git_project_starts_with_empty_task_context() {
        let dir = tempdir().unwrap();
        initialize_git_repo_with_commit(dir.path());
        prepare_local_project_workspace_at(&dir.path().display().to_string(), None).unwrap();
        let indexes_dir = dir.path().join(".agentflow/state/indexes");
        fs::create_dir_all(&indexes_dir).unwrap();
        fs::write(indexes_dir.join("git-context.json"), "{}\n").unwrap();
        fs::write(indexes_dir.join("recent-project-context.json"), "{}\n").unwrap();

        let summary =
            prepare_local_project_workspace_at(&dir.path().display().to_string(), None).unwrap();

        let initialization_status = summary.initialization_status.as_ref().unwrap();
        assert_eq!(initialization_status.project_kind, "existing");
        assert!(!initialization_status.git_context_loaded);
        assert_eq!(initialization_status.recent_context_count, 0);
        assert!(initialization_status.recent_context.is_empty());
        assert_eq!(initialization_status.demo_issue_count, 0);
        assert_eq!(initialization_status.demo_delivery_count, 0);
        assert_eq!(initialization_status.demo_audit_count, 0);
        assert!(!dir
            .path()
            .join(".agentflow/state/indexes/git-context.json")
            .exists());
        assert!(!dir
            .path()
            .join(".agentflow/state/indexes/recent-project-context.json")
            .exists());
        assert!(!dir.path().join(".agentflow/input").exists());
        assert!(!dir
            .path()
            .join(".agentflow/spec/issues/AF-DEMO-001.json")
            .exists());
    }

    #[test]
    fn prepare_workspace_agentflow_repo_starts_with_empty_issue_list() {
        let dir = tempdir().unwrap();
        initialize_git_repo_with_commit(dir.path());
        write_agentflow_project_markers(dir.path());

        let summary =
            prepare_local_project_workspace_at(&dir.path().display().to_string(), None).unwrap();

        let initialization_status = summary.initialization_status.as_ref().unwrap();
        assert_eq!(initialization_status.project_kind, "existing");
        assert!(!initialization_status.demo_data_created);
        assert_eq!(initialization_status.demo_issue_count, 0);
        assert!(!dir.path().join(".agentflow/input").exists());
        assert!(!dir
            .path()
            .join(".agentflow/spec/issues/AF-DOGFOOD-001.json")
            .exists());
        assert!(!dir
            .path()
            .join(".agentflow/spec/issues/AF-DEMO-001.json")
            .exists());
    }

    #[test]
    fn prepare_workspace_reuses_existing_files_without_overwriting() {
        let dir = tempdir().unwrap();
        prepare_local_project_workspace_at(&dir.path().display().to_string(), None).unwrap();
        fs::write(dir.path().join(".agentflow/config.yaml"), "custom: true\n").unwrap();

        let summary =
            prepare_local_project_workspace_at(&dir.path().display().to_string(), None).unwrap();

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
            prepare_local_project_workspace_at(&dir.path().display().to_string(), None).unwrap();

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
            prepare_local_project_workspace_at(&dir.path().display().to_string(), None).unwrap();

        assert!(summary.created_agentflow);
        assert!(!summary.protected_git_exclude);
        assert!(summary.git_exclude_path.is_none());
        assert!(dir.path().join(".agentflow/workspace.yaml").is_file());
    }

    fn initialize_git_repo_with_commit(root: &Path) {
        run_git(root, &["init"]);
        run_git(root, &["config", "user.email", "agentflow@example.local"]);
        run_git(root, &["config", "user.name", "AgentFlow Test"]);
        fs::write(root.join("README.md"), "# Test Project\n").unwrap();
        run_git(root, &["add", "README.md"]);
        run_git(root, &["commit", "-m", "Initial project state"]);
    }

    fn write_agentflow_project_markers(root: &Path) {
        fs::create_dir_all(root.join("apps/desktop")).unwrap();
        fs::write(
            root.join("apps/desktop/package.json"),
            "{\n  \"name\": \"agentflow-desktop\"\n}\n",
        )
        .unwrap();
        fs::create_dir_all(root.join("crates/agent-manual")).unwrap();
        fs::write(
            root.join("crates/agent-manual/Cargo.toml"),
            "[package]\nname = \"agentflow-agent-manual\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
    }

    fn run_git(root: &Path, args: &[&str]) {
        let output = Command::new("git")
            .args(args)
            .current_dir(root)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "git {:?} failed: {}{}",
            args,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
