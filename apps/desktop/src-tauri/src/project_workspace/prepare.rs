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
            input_status: None,
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
            input_status: None,
            state_status: None,
            initialization_status: None,
        });
    }

    agentflow_input::prepare_input_workspace(&root)
        .map_err(|error| format!("prepare input workspace: {error}"))?;
    agentflow_execute::prepare_execute_workspace(&root)
        .map_err(|error| format!("prepare execute workspace: {error}"))?;

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
    let input_snapshot = agentflow_input::prepare_input_workspace(&root)
        .map_err(|error| format!("refresh input workspace: {error}"))?;
    agentflow_output::prepare_output_workspace(&root)
        .map_err(|error| format!("refresh output workspace: {error}"))?;
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
        input_status: Some(input_snapshot.status),
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
    "version: config.v1\nmode: local\nagentflowDir: .agentflow\nworkflow:\n  define: define\n  panel: panel\n  input: input\n  execute: execute\n  output: output\n  state: state\nlegacy:\n  spec: spec\n  goalTree: goal-tree\n".to_string()
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
        assert_eq!(initialization_status.project_kind, "new");
        assert!(initialization_status.demo_data_created);
        assert_eq!(initialization_status.demo_issue_count, 5);
        assert_eq!(initialization_status.demo_delivery_count, 1);
        assert_eq!(initialization_status.demo_audit_count, 1);
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
        assert!(dir.path().join(".agentflow/input/intake").is_dir());
        assert!(dir.path().join(".agentflow/input/specs/drafts").is_dir());
        assert!(dir.path().join(".agentflow/input/specs/approved").is_dir());
        assert!(dir.path().join(".agentflow/input/manifest.json").is_file());
        assert!(dir.path().join(".agentflow/input/index.json").is_file());
        assert!(dir.path().join(".agentflow/input/projects").is_dir());
        assert!(dir.path().join(".agentflow/input/issues").is_dir());
        assert!(dir
            .path()
            .join(".agentflow/input/issues/AF-DEMO-001.json")
            .is_file());
        assert!(dir
            .path()
            .join(".agentflow/output/release/DEL-DEMO-001/delivery.json")
            .is_file());
        assert!(dir
            .path()
            .join(".agentflow/output/audit/AUD-DEMO-001/audit-report.md")
            .is_file());
        assert!(dir
            .path()
            .join(".agentflow/state/indexes/base-release-initialization.json")
            .is_file());
        assert!(summary
            .input_status
            .as_ref()
            .is_some_and(|status| status.ready));
        assert!(!dir.path().join(".agentflow/spec").exists());
        assert!(!dir.path().join(".agentflow/goal-tree").exists());
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
    fn prepare_workspace_existing_git_project_uses_recent_context_without_demo_data() {
        let dir = tempdir().unwrap();
        initialize_git_repo_with_commit(dir.path());

        let summary =
            prepare_local_project_workspace_at(&dir.path().display().to_string(), None).unwrap();

        let initialization_status = summary.initialization_status.as_ref().unwrap();
        assert_eq!(initialization_status.project_kind, "existing");
        assert!(initialization_status.git_context_loaded);
        assert_eq!(initialization_status.demo_issue_count, 0);
        assert_eq!(initialization_status.demo_delivery_count, 0);
        assert_eq!(initialization_status.demo_audit_count, 0);
        assert!(dir
            .path()
            .join(".agentflow/state/indexes/git-context.json")
            .is_file());
        assert!(dir
            .path()
            .join(".agentflow/state/indexes/recent-project-context.json")
            .is_file());
        assert!(!dir
            .path()
            .join(".agentflow/input/intake/git-context.json")
            .exists());
        assert!(!dir
            .path()
            .join(".agentflow/input/projects/context-prs.json")
            .exists());
        assert!(!dir
            .path()
            .join(".agentflow/input/issues/AF-DEMO-001.json")
            .exists());
    }

    #[test]
    fn prepare_workspace_agentflow_repo_seeds_dogfood_cutover_issue() {
        let dir = tempdir().unwrap();
        initialize_git_repo_with_commit(dir.path());
        write_agentflow_project_markers(dir.path());

        let summary =
            prepare_local_project_workspace_at(&dir.path().display().to_string(), None).unwrap();

        let initialization_status = summary.initialization_status.as_ref().unwrap();
        assert_eq!(initialization_status.project_kind, "existing");
        assert!(!initialization_status.demo_data_created);
        assert_eq!(initialization_status.demo_issue_count, 0);
        assert!(dir
            .path()
            .join(".agentflow/input/specs/approved/dogfood-cutover-v1/spec.json")
            .is_file());
        let dogfood_issue_path = dir
            .path()
            .join(".agentflow/input/issues/AF-DOGFOOD-001.json");
        assert!(dogfood_issue_path.is_file());
        let dogfood_issue: agentflow_input::issue::InputIssue =
            serde_json::from_str(&fs::read_to_string(&dogfood_issue_path).unwrap()).unwrap();
        assert_eq!(dogfood_issue.source_spec_id, "dogfood-cutover-v1");
        assert_eq!(dogfood_issue.issue_category.as_str(), "spec");
        assert_eq!(dogfood_issue.required_agent_role.as_str(), "build-agent");
        assert_eq!(dogfood_issue.display_status.as_str(), "ready");
        assert!(matches!(
            dogfood_issue.risk_level,
            agentflow_input::issue::InputRiskLevel::Medium
        ));
        assert!(!dir
            .path()
            .join(".agentflow/input/issues/AF-DEMO-001.json")
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
