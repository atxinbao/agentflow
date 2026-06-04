use crate::model::GraphProtectionSnapshot;
use anyhow::{Context, Result};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub fn check_graph_git_protection(
    project_root: impl AsRef<Path>,
) -> Result<GraphProtectionSnapshot> {
    let root = project_root
        .as_ref()
        .canonicalize()
        .with_context(|| format!("canonicalize {}", project_root.as_ref().display()))?;
    let graph_output_root = root.join(".agentflow/panel");
    let git_dir = resolve_git_dir(&root);

    let Some(git_dir) = git_dir else {
        return Ok(GraphProtectionSnapshot {
            version: "panel-protection.v1".to_string(),
            project_root: root.display().to_string(),
            status: "ready".to_string(),
            graph_output_root: graph_output_root.display().to_string(),
            git_exclude_path: None,
            protected_by_info_exclude: true,
            writes_only_graph_output: true,
            reason: "非 Git 项目，无需 Git exclude 保护。".to_string(),
        });
    };

    let exclude_path = git_dir.join("info/exclude");
    let exclude_content = fs::read_to_string(&exclude_path).unwrap_or_default();
    let protected_by_info_exclude = exclude_content.lines().map(str::trim).any(|line| {
        line == ".agentflow/"
            || line == ".agentflow"
            || line == ".agentflow/panel/"
            || line == ".agentflow/panel"
            || line == ".agentflow/output/graph/"
            || line == ".agentflow/output/graph"
    });
    let status = if protected_by_info_exclude {
        "ready"
    } else {
        "warning"
    };
    let reason = if protected_by_info_exclude {
        ".git/info/exclude 已保护 .agentflow/ 本地运行目录。"
    } else {
        ".git/info/exclude 缺少 .agentflow/ 保护，Panel 产物可能被误加入 Git。"
    };

    Ok(GraphProtectionSnapshot {
        version: "panel-protection.v1".to_string(),
        project_root: root.display().to_string(),
        status: status.to_string(),
        graph_output_root: graph_output_root.display().to_string(),
        git_exclude_path: Some(exclude_path.display().to_string()),
        protected_by_info_exclude,
        writes_only_graph_output: true,
        reason: reason.to_string(),
    })
}

fn resolve_git_dir(root: &Path) -> Option<PathBuf> {
    let git_path = root.join(".git");
    if git_path.is_dir() {
        return Some(git_path);
    }
    if git_path.is_file() {
        let git_file = fs::read_to_string(&git_path).ok()?;
        let path_value = git_file.trim().strip_prefix("gitdir:")?.trim();
        let candidate = PathBuf::from(path_value);
        return Some(if candidate.is_absolute() {
            candidate
        } else {
            root.join(candidate)
        });
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn protection_warns_when_git_exclude_missing_agentflow() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".git/info")).unwrap();
        fs::write(dir.path().join(".git/info/exclude"), "*.log\n").unwrap();

        let snapshot = check_graph_git_protection(dir.path()).unwrap();

        assert_eq!(snapshot.status, "warning");
        assert!(!snapshot.protected_by_info_exclude);
    }

    #[test]
    fn protection_is_ready_when_agentflow_is_excluded() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".git/info")).unwrap();
        fs::write(dir.path().join(".git/info/exclude"), ".agentflow/\n").unwrap();

        let snapshot = check_graph_git_protection(dir.path()).unwrap();

        assert_eq!(snapshot.status, "ready");
        assert!(snapshot.protected_by_info_exclude);
    }
}
