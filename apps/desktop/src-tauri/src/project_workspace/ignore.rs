use std::{
    fs,
    path::{Path, PathBuf},
};

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
