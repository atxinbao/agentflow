use agentflow_execute::{ExecuteRunIndexEntry, ExecuteRunStatus};
use agentflow_input::{
    issue::{DisplayStatus, InputIssue, InputIssueStatus},
    relations::InputIssueRelationKind,
    InputSnapshot,
};
use agentflow_output::OutputSnapshot;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
    process::{Command, Stdio},
};

#[derive(Debug, Clone)]
pub(crate) struct IssueReadinessBlocker {
    pub issue_id: String,
    pub action: String,
    pub reason: String,
    pub source_path: Option<String>,
}

pub(crate) fn issue_readiness_blockers(
    root: &Path,
    input: Option<&InputSnapshot>,
    execute: Option<&agentflow_execute::ExecuteSnapshot>,
    output: Option<&OutputSnapshot>,
) -> Vec<IssueReadinessBlocker> {
    let Some(input) = input else {
        return Vec::new();
    };

    let latest_runs = latest_runs_by_issue(execute);
    let blocked_by = blocked_by_map(input);
    let mut blockers = dependency_blockers(input, output, &latest_runs, &blocked_by);
    blockers.extend(git_provider_blockers(root, input, output, &latest_runs));
    blockers
}

pub(crate) fn issue_has_readiness_blocker(
    issue: &InputIssue,
    blockers: &[crate::model::WorkflowBlockedAction],
) -> bool {
    let issue_path = issue.issue_path.trim();
    if issue_path.is_empty() {
        return false;
    }
    blockers.iter().any(|blocker| {
        matches!(
            blocker.action.as_str(),
            "dependency-ready" | "git-provider-ready"
        ) && blocker.source_path.as_deref() == Some(issue_path)
    })
}

fn dependency_blockers(
    input: &InputSnapshot,
    output: Option<&OutputSnapshot>,
    latest_runs: &BTreeMap<String, ExecuteRunIndexEntry>,
    blocked_by: &BTreeMap<String, BTreeSet<String>>,
) -> Vec<IssueReadinessBlocker> {
    let issues_by_id = input
        .issues
        .iter()
        .map(|issue| (issue.issue_id.as_str(), issue))
        .collect::<BTreeMap<_, _>>();
    let mut blockers = Vec::new();

    for issue in &input.issues {
        if issue_terminal(issue, latest_runs.get(&issue.issue_id), output) {
            continue;
        }
        let Some(dependencies) = blocked_by.get(&issue.issue_id) else {
            continue;
        };

        for dependency_id in dependencies {
            let Some(dependency) = issues_by_id.get(dependency_id.as_str()) else {
                blockers.push(IssueReadinessBlocker {
                    issue_id: issue.issue_id.clone(),
                    action: "dependency-ready".to_string(),
                    reason: format!(
                        "任务 {} 的前置依赖 {} 不存在，不能开始执行。",
                        issue.issue_id, dependency_id
                    ),
                    source_path: issue_source_path(issue),
                });
                continue;
            };

            let dependency_status =
                issue_display_status(dependency, latest_runs.get(dependency_id), output);
            if dependency_status != DisplayStatus::Done {
                blockers.push(IssueReadinessBlocker {
                    issue_id: issue.issue_id.clone(),
                    action: "dependency-ready".to_string(),
                    reason: format!(
                        "任务 {} 的前置依赖 {} 还没有完成，不能开始执行。",
                        issue.issue_id, dependency_id
                    ),
                    source_path: issue_source_path(issue),
                });
            }
        }
    }

    blockers
}

fn git_provider_blockers(
    root: &Path,
    input: &InputSnapshot,
    output: Option<&OutputSnapshot>,
    latest_runs: &BTreeMap<String, ExecuteRunIndexEntry>,
) -> Vec<IssueReadinessBlocker> {
    let gated_issues = input
        .issues
        .iter()
        .filter(|issue| {
            issue_requires_git_provider_gate(issue)
                && !issue_terminal(issue, latest_runs.get(&issue.issue_id), output)
        })
        .collect::<Vec<_>>();
    if gated_issues.is_empty() {
        return Vec::new();
    }

    let reasons = git_provider_gate_reasons(root);
    if reasons.is_empty() {
        return Vec::new();
    }

    gated_issues
        .into_iter()
        .flat_map(|issue| {
            reasons.iter().map(move |reason| IssueReadinessBlocker {
                issue_id: issue.issue_id.clone(),
                action: "git-provider-ready".to_string(),
                reason: format!("任务 {} 的 Git 自动化预检失败：{}", issue.issue_id, reason),
                source_path: issue_source_path(issue),
            })
        })
        .collect()
}

fn issue_requires_git_provider_gate(issue: &InputIssue) -> bool {
    if matches!(
        issue.status,
        InputIssueStatus::Done | InputIssueStatus::Canceled
    ) {
        return false;
    }
    issue.execution_pipeline.as_ref().is_some_and(|pipeline| {
        pipeline
            .stages
            .iter()
            .any(|stage| stage.stage_id == "git-provider-preflight" && stage.required)
    })
}

fn git_provider_gate_reasons(root: &Path) -> Vec<String> {
    let mut reasons = Vec::new();
    if command_stdout(root, "git", &["rev-parse", "--is-inside-work-tree"])
        .map(|value| value.trim() == "true")
        != Ok(true)
    {
        return vec!["当前项目不是 Git 仓库。".to_string()];
    }

    let remote_url = command_stdout(root, "git", &["remote", "get-url", "origin"])
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| first_remote_url(root));
    let Some(remote_url) = remote_url else {
        reasons.push("没有可用的 Git remote。".to_string());
        return reasons;
    };
    let Some(provider) = detect_git_provider(&remote_url) else {
        reasons.push(format!("远端 provider 不支持：{remote_url}"));
        return reasons;
    };

    let branch = command_stdout(root, "git", &["branch", "--show-current"])
        .unwrap_or_default()
        .trim()
        .to_string();
    if branch.is_empty() {
        reasons.push("当前不在可提交分支上。".to_string());
    }

    if let Ok(status) = command_stdout(root, "git", &["status", "--short"]) {
        if !status.trim().is_empty() {
            reasons.push("工作区不干净，请先提交或清理本地改动。".to_string());
        }
    } else {
        reasons.push("无法读取 Git 工作区状态。".to_string());
    }

    match provider {
        "github" => {
            if !command_success(root, "gh", &["--version"]) {
                reasons.push("当前远端是 GitHub，但 gh CLI 不可用。".to_string());
            } else if !command_success(root, "gh", &["auth", "status"]) {
                reasons.push("当前远端是 GitHub，但 gh 未完成认证。".to_string());
            }
        }
        "gitlab" => {
            if !command_success(root, "glab", &["--version"]) {
                reasons.push("当前远端是 GitLab，但 glab CLI 不可用。".to_string());
            } else if !command_success(root, "glab", &["auth", "status"]) {
                reasons.push("当前远端是 GitLab，但 glab 未完成认证。".to_string());
            }
        }
        _ => {}
    }

    reasons
}

fn first_remote_url(root: &Path) -> Option<String> {
    let remotes = command_stdout(root, "git", &["remote"]).ok()?;
    remotes
        .lines()
        .map(str::trim)
        .find(|remote| !remote.is_empty())
        .and_then(|remote| command_stdout(root, "git", &["remote", "get-url", remote]).ok())
        .filter(|value| !value.trim().is_empty())
}

fn detect_git_provider(remote_url: &str) -> Option<&'static str> {
    let remote = remote_url.to_ascii_lowercase();
    if remote.contains("github.com") {
        Some("github")
    } else if remote.contains("gitlab.com") {
        Some("gitlab")
    } else {
        None
    }
}

fn command_success(root: &Path, program: &str, args: &[&str]) -> bool {
    Command::new(program)
        .args(args)
        .current_dir(root)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn command_stdout(root: &Path, program: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(program)
        .args(args)
        .current_dir(root)
        .stdin(Stdio::null())
        .output()
        .map_err(|error| error.to_string())?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

fn blocked_by_map(input: &InputSnapshot) -> BTreeMap<String, BTreeSet<String>> {
    let mut map = input
        .issues
        .iter()
        .map(|issue| (issue.issue_id.clone(), BTreeSet::new()))
        .collect::<BTreeMap<_, _>>();

    for issue in &input.issues {
        for dependency_id in &issue.relations.blocked_by {
            map.entry(issue.issue_id.clone())
                .or_default()
                .insert(dependency_id.clone());
        }
        for blocked_issue_id in &issue.relations.blocks {
            map.entry(blocked_issue_id.clone())
                .or_default()
                .insert(issue.issue_id.clone());
        }
    }

    for relation in &input.relations.relations {
        match relation.relation_type {
            InputIssueRelationKind::BlockedBy => {
                map.entry(relation.from_issue_id.clone())
                    .or_default()
                    .insert(relation.to_issue_id.clone());
            }
            InputIssueRelationKind::Blocks => {
                map.entry(relation.to_issue_id.clone())
                    .or_default()
                    .insert(relation.from_issue_id.clone());
            }
            InputIssueRelationKind::Related | InputIssueRelationKind::DuplicateOf => {}
        }
    }

    map
}

fn latest_runs_by_issue(
    execute: Option<&agentflow_execute::ExecuteSnapshot>,
) -> BTreeMap<String, ExecuteRunIndexEntry> {
    let mut latest_runs = BTreeMap::<String, ExecuteRunIndexEntry>::new();
    let Some(execute) = execute else {
        return latest_runs;
    };
    for run in &execute.index.runs {
        let replace = match latest_runs.get(&run.issue_id) {
            Some(existing) => {
                (run.updated_at, run.run_id.as_str())
                    > (existing.updated_at, existing.run_id.as_str())
            }
            None => true,
        };
        if replace {
            latest_runs.insert(run.issue_id.clone(), run.clone());
        }
    }
    latest_runs
}

pub(crate) fn issue_display_status(
    issue: &InputIssue,
    latest_run: Option<&ExecuteRunIndexEntry>,
    output: Option<&OutputSnapshot>,
) -> DisplayStatus {
    if matches!(issue.status, InputIssueStatus::Canceled) {
        return DisplayStatus::Cancel;
    }
    if let Some(run) = latest_run {
        return match run.status {
            ExecuteRunStatus::Cancelled => DisplayStatus::Cancel,
            ExecuteRunStatus::Completed => DisplayStatus::Done,
            ExecuteRunStatus::Failed => DisplayStatus::Review,
            ExecuteRunStatus::Blocked => DisplayStatus::Blocked,
            ExecuteRunStatus::Queued
            | ExecuteRunStatus::Preflight
            | ExecuteRunStatus::Planned
            | ExecuteRunStatus::Checkpointed
            | ExecuteRunStatus::Patching
            | ExecuteRunStatus::Running
            | ExecuteRunStatus::Validating => DisplayStatus::InProgress,
        };
    }
    if output_has_issue_delivery(
        output,
        &issue.issue_id,
        latest_run.map(|run| run.run_id.as_str()),
    ) {
        return DisplayStatus::Done;
    }
    DisplayStatus::from_input_status(&issue.status)
}

fn issue_terminal(
    issue: &InputIssue,
    latest_run: Option<&ExecuteRunIndexEntry>,
    output: Option<&OutputSnapshot>,
) -> bool {
    matches!(
        issue_display_status(issue, latest_run, output),
        DisplayStatus::Done | DisplayStatus::Cancel
    )
}

fn output_has_issue_delivery(
    output: Option<&OutputSnapshot>,
    issue_id: &str,
    latest_run_id: Option<&str>,
) -> bool {
    let Some(snapshot) = output else {
        return false;
    };
    snapshot.index.release_deliveries.iter().any(|entry| {
        entry.issue_id == issue_id || latest_run_id.is_some_and(|run_id| entry.run_id == run_id)
    }) || snapshot.index.evidence.iter().any(|entry| {
        entry.issue_id == issue_id || latest_run_id.is_some_and(|run_id| entry.run_id == run_id)
    })
}

fn issue_source_path(issue: &InputIssue) -> Option<String> {
    (!issue.issue_path.trim().is_empty()).then(|| issue.issue_path.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use agentflow_input::{
        issue::{
            default_build_agent_execution_pipeline, AgentRole, InputIssueModel, InputRiskLevel,
            IssueCategory,
        },
        model::{InputIndex, InputManifest, InputStatusSnapshot, InputWorkspaceStatus},
        relations::InputIssueRelationsFile,
    };

    fn snapshot(issues: Vec<InputIssue>, relations: InputIssueRelationsFile) -> InputSnapshot {
        InputSnapshot {
            version: "input-snapshot.v1".to_string(),
            project_root: "/tmp/agentflow-test".to_string(),
            ready: true,
            status: InputStatusSnapshot {
                version: "input-status.v1".to_string(),
                project_root: "/tmp/agentflow-test".to_string(),
                status: InputWorkspaceStatus::Ready,
                ready: true,
                manifest_exists: true,
                index_exists: true,
                summary: Default::default(),
                missing_paths: Vec::new(),
                warnings: Vec::new(),
                errors: Vec::new(),
            },
            manifest: InputManifest::new("/tmp/agentflow-test", Default::default()),
            index: InputIndex::default(),
            intake: Vec::new(),
            specs: Vec::new(),
            projects: Vec::new(),
            issues,
            relations,
        }
    }

    fn issue(issue_id: &str, status: InputIssueStatus) -> InputIssue {
        InputIssue {
            issue_id: issue_id.to_string(),
            issue_model: InputIssueModel::Direct,
            issue_category: IssueCategory::Spec,
            required_agent_role: AgentRole::BuildAgent,
            source_spec_id: "spec-001".to_string(),
            issue_path: format!(".agentflow/input/issues/{issue_id}.json"),
            title: issue_id.to_string(),
            summary: issue_id.to_string(),
            status,
            execution_risk: InputRiskLevel::Low,
            ..InputIssue::default()
        }
    }

    #[test]
    fn dependency_gate_blocks_until_blocked_by_issue_is_done() {
        let mut blocked = issue("AF-002", InputIssueStatus::ReadyForExecute);
        blocked.relations.blocked_by = vec!["AF-001".to_string()];
        let input = snapshot(
            vec![issue("AF-001", InputIssueStatus::ReadyForExecute), blocked],
            InputIssueRelationsFile::default(),
        );

        let blockers = issue_readiness_blockers(Path::new("/tmp"), Some(&input), None, None);

        assert!(blockers.iter().any(|blocker| {
            blocker.issue_id == "AF-002" && blocker.action == "dependency-ready"
        }));
    }

    #[test]
    fn dependency_gate_allows_when_blocked_by_issue_is_done() {
        let mut blocked = issue("AF-002", InputIssueStatus::ReadyForExecute);
        blocked.relations.blocked_by = vec!["AF-001".to_string()];
        let input = snapshot(
            vec![issue("AF-001", InputIssueStatus::Done), blocked],
            InputIssueRelationsFile::default(),
        );

        let blockers = issue_readiness_blockers(Path::new("/tmp"), Some(&input), None, None);

        assert!(blockers.is_empty());
    }

    #[test]
    fn git_provider_gate_only_runs_for_pipeline_issues() {
        let mut gated = issue("AF-001", InputIssueStatus::ReadyForExecute);
        gated.execution_pipeline = Some(default_build_agent_execution_pipeline());
        let plain = issue("AF-002", InputIssueStatus::ReadyForExecute);
        let input = snapshot(vec![gated, plain], InputIssueRelationsFile::default());

        let blockers =
            issue_readiness_blockers(Path::new("/tmp/not-a-git-repo"), Some(&input), None, None);

        assert!(blockers.iter().any(|blocker| {
            blocker.issue_id == "AF-001" && blocker.action == "git-provider-ready"
        }));
        assert!(!blockers.iter().any(|blocker| blocker.issue_id == "AF-002"));
    }

    #[test]
    fn git_provider_detection_treats_github_and_gitlab_as_alternatives() {
        assert_eq!(
            detect_git_provider("git@github.com:owner/repo.git"),
            Some("github")
        );
        assert_eq!(
            detect_git_provider("https://gitlab.com/owner/repo.git"),
            Some("gitlab")
        );
    }
}
