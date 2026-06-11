use crate::{
    model::{AuditGateStatus, IssueLoopStage, LoopBlocker, ProjectLoopSnapshot, ProjectLoopStatus},
    storage::{
        read_project_loop_snapshot, write_issue_loop_projection, write_project_loop_snapshot,
    },
    IssueLoop,
};
use agentflow_input::{
    issue::{AgentRole, InputIssue, InputIssueModel, InputIssueStatus, IssueCategory},
    project::{InputProject, InputProjectStatus},
};
use agentflow_mcp::{
    check_github_provider, check_gitlab_provider,
    storage::{write_provider_status, write_registry_for_statuses},
    McpProviderStatus, McpProviderStatusCode,
};
use anyhow::{Context, Result};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectLoop {
    project_id: String,
}

impl ProjectLoop {
    pub fn new(project_id: impl Into<String>) -> Self {
        Self {
            project_id: project_id.into(),
        }
    }

    pub fn project_id(&self) -> &str {
        &self.project_id
    }

    pub fn snapshot(&self, updated_at: u64) -> ProjectLoopSnapshot {
        ProjectLoopSnapshot::new(self.project_id.clone(), updated_at)
    }

    pub fn run_preflight(&self, project_root: impl AsRef<Path>) -> Result<ProjectLoopSnapshot> {
        let root = canonical_project_root(project_root)?;
        let snapshot = agentflow_input::prepare_input_workspace(&root)?;
        let project = find_project(&snapshot.projects, &self.project_id)?;
        let mut loop_snapshot = self.snapshot(now());
        loop_snapshot.status = ProjectLoopStatus::Active;

        let mut blockers = Vec::new();
        if project.issue_ids.is_empty() {
            blockers.push(blocker(
                "project-empty",
                "Project has no input issues to schedule.",
                Some(project.system.path.clone()),
            ));
        }

        let provider_status = check_git_provider(&root, &mut blockers)?;
        if let Some(status) = provider_status {
            write_provider_status(&root, &status)?;
            write_registry_for_statuses(&root, &[status.clone()])?;
            if status.status != McpProviderStatusCode::Ready {
                blockers.push(blocker(
                    "git-provider-not-ready",
                    format!(
                        "{} provider is not ready: {}",
                        status.provider,
                        status.errors.join("; ")
                    ),
                    Some(format!(
                        ".agentflow/state/mcp/providers/{}.json",
                        status.provider
                    )),
                ));
            }
        }

        loop_snapshot.blockers = blockers;
        if !loop_snapshot.blockers.is_empty() {
            loop_snapshot.status = ProjectLoopStatus::PreflightBlocked;
        }
        write_project_loop_snapshot(&root, &loop_snapshot)?;
        Ok(loop_snapshot)
    }

    pub fn schedule_ready_issues(
        &self,
        project_root: impl AsRef<Path>,
    ) -> Result<ProjectLoopSnapshot> {
        let root = canonical_project_root(project_root)?;
        let snapshot = agentflow_input::prepare_input_workspace(&root)?;
        let project = find_project(&snapshot.projects, &self.project_id)?.clone();
        let issues_by_id = snapshot
            .issues
            .iter()
            .map(|issue| (issue.issue_id.as_str(), issue))
            .collect::<BTreeMap<_, _>>();
        let done_issues = snapshot
            .issues
            .iter()
            .filter(|issue| matches!(issue.status, InputIssueStatus::Done))
            .map(|issue| issue.issue_id.as_str())
            .collect::<BTreeSet<_>>();
        let mut loop_snapshot = self.snapshot(now());
        loop_snapshot.status = ProjectLoopStatus::Scheduling;

        let project_preflight_ready = read_project_loop_snapshot(&root, &project.project_id)
            .map(|snapshot| {
                snapshot.blockers.is_empty()
                    && !matches!(snapshot.status, ProjectLoopStatus::PreflightBlocked)
            })
            .unwrap_or(false);

        for issue_id in &project.issue_ids {
            let Some(issue) = issues_by_id.get(issue_id.as_str()) else {
                loop_snapshot.blockers.push(blocker(
                    "missing-project-issue",
                    format!("Project references missing issue {issue_id}."),
                    Some(project.system.path.clone()),
                ));
                continue;
            };

            match issue.status {
                InputIssueStatus::Done => {
                    loop_snapshot.done_issue_ids.push(issue.issue_id.clone());
                    write_issue_projection(
                        &root,
                        &project.project_id,
                        issue,
                        IssueLoopStage::Done,
                        Vec::new(),
                    )?;
                }
                InputIssueStatus::Blocked => {
                    loop_snapshot.blocked_issue_ids.push(issue.issue_id.clone());
                    write_issue_projection(
                        &root,
                        &project.project_id,
                        issue,
                        IssueLoopStage::Blocked,
                        Vec::new(),
                    )?;
                }
                InputIssueStatus::Todo
                | InputIssueStatus::InProgress
                | InputIssueStatus::InReview => {
                    loop_snapshot.active_issue_ids.push(issue.issue_id.clone());
                    write_issue_projection(
                        &root,
                        &project.project_id,
                        issue,
                        issue_stage(&issue.status),
                        Vec::new(),
                    )?;
                }
                InputIssueStatus::Cancel => {
                    write_issue_projection(
                        &root,
                        &project.project_id,
                        issue,
                        IssueLoopStage::Cancel,
                        Vec::new(),
                    )?;
                }
                InputIssueStatus::Backlog => {
                    let blockers = schedule_blockers(
                        &root,
                        &project,
                        issue,
                        &done_issues,
                        project_preflight_ready,
                    );
                    if blockers.is_empty() {
                        ensure_context_pack(&root, issue)?;
                        agentflow_input::update_input_issue_status(
                            &root,
                            &issue.issue_id,
                            InputIssueStatus::Todo,
                        )?;
                        loop_snapshot.active_issue_ids.push(issue.issue_id.clone());
                        write_issue_projection(
                            &root,
                            &project.project_id,
                            issue,
                            IssueLoopStage::Todo,
                            Vec::new(),
                        )?;
                    } else {
                        agentflow_input::update_input_issue_status(
                            &root,
                            &issue.issue_id,
                            InputIssueStatus::Blocked,
                        )?;
                        loop_snapshot.blocked_issue_ids.push(issue.issue_id.clone());
                        write_issue_projection(
                            &root,
                            &project.project_id,
                            issue,
                            IssueLoopStage::Blocked,
                            blockers.clone(),
                        )?;
                        loop_snapshot.blockers.extend(blockers);
                    }
                }
            }
        }

        if project.issue_ids.iter().all(|issue_id| {
            issues_by_id
                .get(issue_id.as_str())
                .is_some_and(|issue| matches!(issue.status, InputIssueStatus::Done))
        }) {
            loop_snapshot.status = ProjectLoopStatus::Done;
            loop_snapshot.audit_status = AuditGateStatus::Passed;
            agentflow_input::update_input_project_status(
                &root,
                &project.project_id,
                InputProjectStatus::Done,
            )?;
        } else if !loop_snapshot.active_issue_ids.is_empty() {
            loop_snapshot.status = ProjectLoopStatus::Executing;
            agentflow_input::update_input_project_status(
                &root,
                &project.project_id,
                InputProjectStatus::Active,
            )?;
        } else if !loop_snapshot.blocked_issue_ids.is_empty() {
            loop_snapshot.status = ProjectLoopStatus::Blocked;
            agentflow_input::update_input_project_status(
                &root,
                &project.project_id,
                InputProjectStatus::Blocked,
            )?;
        }

        write_project_loop_snapshot(&root, &loop_snapshot)?;
        agentflow_input::prepare_input_workspace(&root)?;
        Ok(loop_snapshot)
    }
}

fn schedule_blockers(
    root: &Path,
    project: &InputProject,
    issue: &InputIssue,
    done_issues: &BTreeSet<&str>,
    project_preflight_ready: bool,
) -> Vec<LoopBlocker> {
    let mut blockers = Vec::new();
    if !project_preflight_ready {
        blockers.push(blocker(
            "project-preflight-not-ready",
            "Project preflight must be ready before scheduling backlog issues.",
            Some(format!(
                ".agentflow/state/loops/projects/{}.json",
                project.project_id
            )),
        ));
    }
    if !matches!(issue.issue_model, InputIssueModel::Project)
        || issue.project_id.as_deref() != Some(project.project_id.as_str())
        || !project
            .issue_ids
            .iter()
            .any(|candidate| candidate == &issue.issue_id)
    {
        blockers.push(blocker(
            "project-boundary-invalid",
            "Issue must belong to the current Project before scheduling.",
            Some(issue.issue_path.clone()),
        ));
    }
    if !matches!(issue.issue_category, IssueCategory::Spec)
        || !matches!(issue.required_agent_role, AgentRole::BuildAgent)
    {
        blockers.push(blocker(
            "build-agent-contract-invalid",
            "Project scheduler only schedules spec issues assigned to Build Agent.",
            Some(issue.issue_path.clone()),
        ));
    }
    for error in issue.target_metadata_errors() {
        blockers.push(blocker(
            "issue-contract-incomplete",
            error,
            Some(issue.issue_path.clone()),
        ));
    }
    for dependency in &issue.relations.blocked_by {
        if !done_issues.contains(dependency.as_str()) {
            blockers.push(blocker(
                "dependency-not-done",
                format!("Dependency issue {dependency} is not done."),
                Some(issue.issue_path.clone()),
            ));
        }
    }
    if !context_pack_can_be_generated(root, issue) {
        blockers.push(blocker(
            "context-pack-not-ready",
            "Panel Context Pack cannot be generated for this issue.",
            Some(issue.context_pack_path.clone()),
        ));
    }
    blockers
}

fn context_pack_can_be_generated(root: &Path, issue: &InputIssue) -> bool {
    ensure_context_pack(root, issue).is_ok()
}

fn ensure_context_pack(root: &Path, issue: &InputIssue) -> Result<()> {
    let objective = if issue.summary.trim().is_empty() {
        issue.scope.join("\n")
    } else {
        issue.summary.clone()
    };
    let snapshot = agentflow_panel::panel_preflight(
        root,
        "issue",
        Some(&issue.issue_id),
        &issue.title,
        &objective,
        &issue.acceptance_criteria,
    )?;
    if !snapshot.ready {
        anyhow::bail!(snapshot.reason);
    }
    if !root.join(&issue.context_pack_path).is_file() {
        anyhow::bail!(
            "context pack was not written to {}",
            issue.context_pack_path
        );
    }
    Ok(())
}

fn write_issue_projection(
    root: &Path,
    project_id: &str,
    issue: &InputIssue,
    stage: IssueLoopStage,
    blockers: Vec<LoopBlocker>,
) -> Result<()> {
    let mut projection = IssueLoop::new(project_id, &issue.issue_id).projection(now());
    projection.stage = stage;
    projection.blockers = blockers;
    write_issue_loop_projection(root, &projection)?;
    Ok(())
}

fn issue_stage(status: &InputIssueStatus) -> IssueLoopStage {
    match status {
        InputIssueStatus::Backlog => IssueLoopStage::Backlog,
        InputIssueStatus::Todo => IssueLoopStage::Todo,
        InputIssueStatus::InProgress => IssueLoopStage::InProgress,
        InputIssueStatus::InReview => IssueLoopStage::InReview,
        InputIssueStatus::Done => IssueLoopStage::Done,
        InputIssueStatus::Blocked => IssueLoopStage::Blocked,
        InputIssueStatus::Cancel => IssueLoopStage::Cancel,
    }
}

fn check_git_provider(
    root: &Path,
    blockers: &mut Vec<LoopBlocker>,
) -> Result<Option<McpProviderStatus>> {
    let remote = git_remote_origin(root);
    let Some(remote) = remote else {
        blockers.push(blocker(
            "git-remote-missing",
            "Project Git remote origin is missing.",
            None,
        ));
        return Ok(None);
    };
    if remote.contains("github.com") {
        return Ok(Some(check_github_provider(root)));
    }
    if remote.contains("gitlab.com") {
        return Ok(Some(check_gitlab_provider(root)));
    }
    blockers.push(blocker(
        "git-provider-unsupported",
        format!("Unsupported Git provider remote: {remote}"),
        None,
    ));
    Ok(None)
}

fn git_remote_origin(root: &Path) -> Option<String> {
    let output = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .current_dir(root)
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|remote| !remote.is_empty())
}

fn find_project<'a>(projects: &'a [InputProject], project_id: &str) -> Result<&'a InputProject> {
    projects
        .iter()
        .find(|project| project.project_id == project_id)
        .with_context(|| format!("input project {project_id} does not exist"))
}

fn blocker(
    code: impl Into<String>,
    reason: impl Into<String>,
    source_path: Option<String>,
) -> LoopBlocker {
    LoopBlocker {
        code: code.into(),
        reason: reason.into(),
        source_path,
    }
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn canonical_project_root(project_root: impl AsRef<Path>) -> Result<PathBuf> {
    project_root
        .as_ref()
        .canonicalize()
        .with_context(|| format!("canonicalize {}", project_root.as_ref().display()))
}
