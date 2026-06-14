use super::model::ProjectInitializationSummary;
use anyhow::Context;
use serde::{de::DeserializeOwned, Serialize};
use std::{fs, path::Path};

const INIT_STATUS_PATH: &str = ".agentflow/state/indexes/base-release-initialization.json";
const LEGACY_RECENT_CONTEXT_PATH: &str = ".agentflow/state/indexes/recent-project-context.json";
const LEGACY_GIT_CONTEXT_PATH: &str = ".agentflow/state/indexes/git-context.json";

pub(crate) fn initialize_base_release_project(
    root: &Path,
) -> Result<ProjectInitializationSummary, String> {
    remove_legacy_git_context_files(root)?;
    let project_kind = if root.join(".git").is_dir() || spec_issue_json_count(root) > 0 {
        "existing"
    } else {
        "new"
    };
    let summary = ProjectInitializationSummary {
        version: "base-release-initialization.v1".to_string(),
        project_kind: project_kind.to_string(),
        initialized: true,
        demo_data_created: false,
        git_context_loaded: false,
        recent_context_count: 0,
        demo_issue_count: 0,
        demo_delivery_count: 0,
        demo_audit_count: 0,
        message: if project_kind == "new" {
            "新项目已准备好。".to_string()
        } else {
            "项目已准备好。".to_string()
        },
        paths: Vec::new(),
        warnings: Vec::new(),
        recent_context: Vec::new(),
    };
    write_initialization_summary(root, &summary)?;
    Ok(summary)
}

pub(crate) fn load_project_initialization_status(
    root: &Path,
) -> Result<ProjectInitializationSummary, String> {
    let path = root.join(INIT_STATUS_PATH);
    if path.is_file() {
        return read_json(&path);
    }

    Ok(ProjectInitializationSummary {
        version: "base-release-initialization.v1".to_string(),
        project_kind: if root.join(".git").is_dir() || spec_issue_json_count(root) > 0 {
            "existing"
        } else {
            "new"
        }
        .to_string(),
        initialized: false,
        demo_data_created: false,
        git_context_loaded: false,
        recent_context_count: 0,
        demo_issue_count: 0,
        demo_delivery_count: 0,
        demo_audit_count: 0,
        message: "初始化状态尚未登记，刷新项目后会补齐。".to_string(),
        paths: Vec::new(),
        warnings: Vec::new(),
        recent_context: Vec::new(),
    })
}

fn remove_legacy_git_context_files(root: &Path) -> Result<(), String> {
    for relative_path in [LEGACY_RECENT_CONTEXT_PATH, LEGACY_GIT_CONTEXT_PATH] {
        let path = root.join(relative_path);
        if path.exists() {
            fs::remove_file(&path).map_err(|error| format!("remove {relative_path}: {error}"))?;
        }
    }
    Ok(())
}

fn write_initialization_summary(
    root: &Path,
    summary: &ProjectInitializationSummary,
) -> Result<(), String> {
    write_json(&root.join(INIT_STATUS_PATH), summary)
}

fn spec_issue_json_count(root: &Path) -> usize {
    root.join(".agentflow/spec/issues")
        .read_dir()
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .filter(|entry| {
                    entry.path().extension().and_then(|value| value.to_str()) == Some("json")
                })
                .count()
        })
        .unwrap_or(0)
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create {}", parent.display()))
            .map_err(|error| error.to_string())?;
    }
    let content = serde_json::to_string_pretty(value)
        .map_err(|error| format!("serialize {}: {error}", path.display()))?
        + "\n";
    fs::write(path, content)
        .with_context(|| format!("write {}", path.display()))
        .map_err(|error| error.to_string())
}

fn read_json<T: DeserializeOwned>(path: &Path) -> Result<T, String> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("read {}", path.display()))
        .map_err(|error| error.to_string())?;
    serde_json::from_str(&raw)
        .with_context(|| format!("parse {}", path.display()))
        .map_err(|error| error.to_string())
}
