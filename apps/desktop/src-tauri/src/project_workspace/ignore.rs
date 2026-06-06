use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

pub(crate) struct AgentsGitignoreProtection {
    pub(crate) gitignore_path: PathBuf,
    pub(crate) protected: bool,
    pub(crate) tracked_by_git: bool,
    pub(crate) warning: Option<String>,
}

pub(crate) fn protect_agentflow_from_git(root: &Path) -> Result<(Option<PathBuf>, bool), String> {
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

pub(crate) fn protect_agents_md_from_gitignore(
    root: &Path,
) -> Result<AgentsGitignoreProtection, String> {
    let gitignore_path = root.join(".gitignore");
    let current = fs::read_to_string(&gitignore_path).unwrap_or_default();
    let already_protected = current
        .lines()
        .map(str::trim)
        .any(|line| line == "AGENTS.md");

    if !already_protected {
        let mut next = current;
        if !next.is_empty() && !next.ends_with('\n') {
            next.push('\n');
        }
        next.push_str("\n# AgentFlow local agent instructions\nAGENTS.md\n");
        fs::write(&gitignore_path, next)
            .map_err(|error| format!("write {}: {error}", gitignore_path.display()))?;
    }

    let tracked_by_git = is_git_tracked(root, "AGENTS.md");
    let warning = tracked_by_git.then(|| {
        "AGENTS.md 已生成，并已加入 .gitignore。如果它之前已经被 Git 跟踪，请手动执行：git rm --cached AGENTS.md".to_string()
    });

    Ok(AgentsGitignoreProtection {
        gitignore_path,
        protected: true,
        tracked_by_git,
        warning,
    })
}

fn is_git_tracked(root: &Path, relative_path: &str) -> bool {
    if !root.join(".git").is_dir() {
        return false;
    }

    Command::new("git")
        .args(["ls-files", "--error-unmatch", relative_path])
        .current_dir(root)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}
