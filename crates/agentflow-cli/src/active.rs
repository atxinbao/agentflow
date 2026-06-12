//! Active CLI boundary.
//!
//! Active commands are narrow wrappers around the current workspace crates.
//! They must not call archived 2026-05 writers.

use agentflow_execute::{BuildAgentCompletion, BuildAgentCompletionRequest};
use agentflow_input::issue::{
    AgentRole, InputIssue, InputIssueModel, InputIssueStatus, IssueCategory,
};
use agentflow_loop::{
    write_issue_merge_proof, DirectIssueLoop, IssueLoop, IssueLoopProjection, ProjectLoop,
};
use anyhow::{Context, Result};
use std::{
    fs,
    path::{Path, PathBuf},
    time::SystemTime,
};

const CLI_FRESHNESS_PATHS: [&str; 9] = [
    "Cargo.toml",
    "Cargo.lock",
    "crates/agentflow-cli/src",
    "crates/execute/src",
    "crates/input/src",
    "crates/state/src",
    "crates/panel/src",
    "crates/agent-manual/src",
    "crates/loop/src",
];

#[derive(Debug, Clone)]
pub(crate) struct BuildAgentStart {
    pub issue_id: String,
    pub run_id: String,
    pub stage: String,
    pub branch_name: Option<String>,
    pub project_id: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct BuildAgentMergeProof {
    pub issue_id: String,
    pub run_id: String,
    pub merged: bool,
    pub proof_path: PathBuf,
}

pub(crate) fn complete_build_agent_issue_from_request(
    root: &Path,
    request_path: &Path,
) -> Result<BuildAgentCompletion> {
    assert_current_cli_is_fresh(root)?;
    let raw = fs::read_to_string(request_path)
        .with_context(|| format!("read completion request {}", request_path.display()))?;
    let request: BuildAgentCompletionRequest = serde_json::from_str(&raw)
        .with_context(|| format!("parse completion request {}", request_path.display()))?;
    let completion = agentflow_execute::complete_build_agent_issue(root, request)?;
    agentflow_state::refresh_state(root)?;
    Ok(completion)
}

pub(crate) fn prepare_build_agent_review_from_request(
    root: &Path,
    request_path: &Path,
) -> Result<BuildAgentCompletion> {
    assert_current_cli_is_fresh(root)?;
    let raw = fs::read_to_string(request_path)
        .with_context(|| format!("read review preparation request {}", request_path.display()))?;
    let request: BuildAgentCompletionRequest = serde_json::from_str(&raw).with_context(|| {
        format!(
            "parse review preparation request {}",
            request_path.display()
        )
    })?;
    let prepared = agentflow_execute::prepare_build_agent_review(root, request)?;
    agentflow_state::refresh_state(root)?;
    Ok(prepared)
}

pub(crate) fn start_build_agent_issue(root: &Path, issue_id: &str) -> Result<BuildAgentStart> {
    assert_current_cli_is_fresh(root)?;
    let issue_id = issue_id.trim();
    if issue_id.is_empty() {
        anyhow::bail!("build agent start requires issueId");
    }
    let mut issue = agentflow_input::load_input_issue(root, issue_id)
        .with_context(|| format!("load input issue {issue_id}"))?;
    assert_build_agent_contract(&issue)?;
    if matches!(issue.status, InputIssueStatus::Backlog) {
        schedule_issue_for_runtime(root, &issue)?;
        issue = agentflow_input::load_input_issue(root, issue_id)
            .with_context(|| format!("reload input issue {issue_id} after scheduling"))?;
    }
    if !matches!(issue.status, InputIssueStatus::Todo) {
        anyhow::bail!(
            "build agent start requires todo issue after scheduling; {} is {}",
            issue.issue_id,
            issue.status.as_str()
        );
    }

    let projection = start_issue_runtime_preflight(root, &issue)?;
    agentflow_state::refresh_state(root)?;
    Ok(BuildAgentStart {
        issue_id: issue.issue_id,
        run_id: projection
            .run_id
            .clone()
            .ok_or_else(|| anyhow::anyhow!("runtime preflight did not produce runId"))?,
        stage: projection.stage.as_str().to_string(),
        branch_name: projection.branch_name,
        project_id: projection.project_id,
    })
}

pub(crate) fn write_build_agent_merge_proof(
    root: &Path,
    issue_id: &str,
    run_id: &str,
    provider: &str,
    merge_mode: &str,
    remote_url: Option<String>,
    merged: bool,
) -> Result<BuildAgentMergeProof> {
    assert_current_cli_is_fresh(root)?;
    let issue = agentflow_input::load_input_issue(root, issue_id)
        .with_context(|| format!("load input issue {issue_id}"))?;
    assert_build_agent_contract(&issue)?;
    let proof_path = write_issue_merge_proof(
        root,
        &issue.issue_id,
        issue.project_id.as_deref(),
        run_id,
        provider,
        merge_mode,
        remote_url,
        merged,
    )?;
    agentflow_state::refresh_state(root)?;
    Ok(BuildAgentMergeProof {
        issue_id: issue.issue_id,
        run_id: run_id.to_string(),
        merged,
        proof_path,
    })
}

fn assert_build_agent_contract(issue: &InputIssue) -> Result<()> {
    if !matches!(issue.issue_category, IssueCategory::Spec) {
        anyhow::bail!(
            "build agent start only supports spec issues; {} is {}",
            issue.issue_id,
            issue.issue_category.as_str()
        );
    }
    if !matches!(issue.required_agent_role, AgentRole::BuildAgent) {
        anyhow::bail!(
            "build agent start only supports build-agent issues; {} is {}",
            issue.issue_id,
            issue.required_agent_role.as_str()
        );
    }
    Ok(())
}

fn schedule_issue_for_runtime(root: &Path, issue: &InputIssue) -> Result<()> {
    match issue.issue_model {
        InputIssueModel::Direct => {
            DirectIssueLoop::schedule_ready_issues(root)?;
        }
        InputIssueModel::Project => {
            let project_id = issue.project_id.as_deref().ok_or_else(|| {
                anyhow::anyhow!("project issue {} is missing projectId", issue.issue_id)
            })?;
            ProjectLoop::new(project_id).run_preflight(root)?;
            ProjectLoop::new(project_id).schedule_ready_issues(root)?;
        }
    }
    Ok(())
}

fn start_issue_runtime_preflight(root: &Path, issue: &InputIssue) -> Result<IssueLoopProjection> {
    match issue.issue_model {
        InputIssueModel::Direct => DirectIssueLoop::start_runtime_preflight(root, &issue.issue_id),
        InputIssueModel::Project => {
            let project_id = issue.project_id.as_deref().ok_or_else(|| {
                anyhow::anyhow!("project issue {} is missing projectId", issue.issue_id)
            })?;
            IssueLoop::new(project_id, &issue.issue_id).start_runtime_preflight(root)
        }
    }
}

fn assert_current_cli_is_fresh(root: &Path) -> Result<()> {
    let current_exe = std::env::current_exe().context("locate current agentflow CLI binary")?;
    if !is_local_target_binary(root, &current_exe) {
        return Ok(());
    }
    let binary_modified = file_modified(&current_exe)?;
    if let Some((newest_path, newest_modified)) = newest_source_mtime(root)? {
        if binary_is_stale(binary_modified, newest_modified) {
            anyhow::bail!(
                "current AgentFlow CLI is stale: {} is older than {}. Run `{}` before `build-agent complete`.",
                current_exe.display(),
                newest_path.display(),
                rebuild_hint(&current_exe)
            );
        }
    }
    Ok(())
}

fn newest_source_mtime(root: &Path) -> Result<Option<(PathBuf, SystemTime)>> {
    let mut newest = None;
    for relative in CLI_FRESHNESS_PATHS {
        collect_newest_mtime(&root.join(relative), &mut newest)?;
    }
    Ok(newest)
}

fn collect_newest_mtime(path: &Path, newest: &mut Option<(PathBuf, SystemTime)>) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }
    let metadata = fs::metadata(path)?;
    if metadata.is_file() {
        let modified = metadata.modified()?;
        let replace = newest
            .as_ref()
            .map(|(_, current)| modified > *current)
            .unwrap_or(true);
        if replace {
            *newest = Some((path.to_path_buf(), modified));
        }
        return Ok(());
    }
    if metadata.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            collect_newest_mtime(&entry.path(), newest)?;
        }
    }
    Ok(())
}

fn file_modified(path: &Path) -> Result<SystemTime> {
    Ok(fs::metadata(path)?.modified()?)
}

fn is_local_target_binary(root: &Path, executable: &Path) -> bool {
    executable.starts_with(root.join("target"))
        && executable
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| name == "agentflow")
            .unwrap_or(false)
}

fn rebuild_hint(executable: &Path) -> &'static str {
    if executable
        .components()
        .any(|component| component.as_os_str() == "release")
    {
        "cargo build --release --bin agentflow"
    } else {
        "cargo build --bin agentflow"
    }
}

fn binary_is_stale(binary_modified: SystemTime, newest_source_modified: SystemTime) -> bool {
    newest_source_modified > binary_modified
}

#[cfg(test)]
mod tests {
    use super::{binary_is_stale, is_local_target_binary, rebuild_hint};
    use std::{
        path::Path,
        time::{Duration, UNIX_EPOCH},
    };

    #[test]
    fn detects_local_target_binaries_only() {
        assert!(is_local_target_binary(
            Path::new("/repo"),
            Path::new("/repo/target/debug/agentflow")
        ));
        assert!(is_local_target_binary(
            Path::new("/repo"),
            Path::new("/repo/target/release/agentflow")
        ));
        assert!(!is_local_target_binary(
            Path::new("/repo"),
            Path::new("/usr/local/bin/agentflow")
        ));
    }

    #[test]
    fn stale_check_requires_newer_sources() {
        let binary = UNIX_EPOCH + Duration::from_secs(10);
        let source = UNIX_EPOCH + Duration::from_secs(11);
        assert!(binary_is_stale(binary, source));
        assert!(!binary_is_stale(source, binary));
    }

    #[test]
    fn rebuild_hint_matches_binary_profile() {
        assert_eq!(
            rebuild_hint(Path::new("/repo/target/debug/agentflow")),
            "cargo build --bin agentflow"
        );
        assert_eq!(
            rebuild_hint(Path::new("/repo/target/release/agentflow")),
            "cargo build --release --bin agentflow"
        );
    }
}
