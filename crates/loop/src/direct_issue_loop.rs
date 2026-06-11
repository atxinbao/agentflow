use crate::{
    model::{IssueLoopProjection, IssueLoopStage, LoopBlocker},
    storage::write_issue_loop_projection,
};
use agentflow_input::issue::{
    AgentRole, InputIssue, InputIssueModel, InputIssueStatus, IssueCategory,
};
use anyhow::{Context, Result};
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DirectIssueLoopSummary {
    pub active_issue_ids: Vec<String>,
    pub blocked_issue_ids: Vec<String>,
    pub done_issue_ids: Vec<String>,
    pub blockers: Vec<LoopBlocker>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DirectIssueLoop;

impl DirectIssueLoop {
    pub fn schedule_ready_issues(project_root: impl AsRef<Path>) -> Result<DirectIssueLoopSummary> {
        let root = canonical_project_root(project_root)?;
        let snapshot = agentflow_input::prepare_input_workspace(&root)?;
        let done_issues = snapshot
            .issues
            .iter()
            .filter(|issue| matches!(issue.status, InputIssueStatus::Done))
            .map(|issue| issue.issue_id.as_str())
            .collect::<BTreeSet<_>>();
        let mut summary = DirectIssueLoopSummary::default();

        for issue in snapshot
            .issues
            .iter()
            .filter(|issue| matches!(issue.issue_model, InputIssueModel::Direct))
        {
            match issue.status {
                InputIssueStatus::Done => {
                    summary.done_issue_ids.push(issue.issue_id.clone());
                    write_direct_issue_projection(&root, issue, IssueLoopStage::Done, Vec::new())?;
                }
                InputIssueStatus::Blocked => {
                    summary.blocked_issue_ids.push(issue.issue_id.clone());
                    write_direct_issue_projection(
                        &root,
                        issue,
                        IssueLoopStage::Blocked,
                        Vec::new(),
                    )?;
                }
                InputIssueStatus::Todo
                | InputIssueStatus::InProgress
                | InputIssueStatus::InReview => {
                    summary.active_issue_ids.push(issue.issue_id.clone());
                    write_direct_issue_projection(
                        &root,
                        issue,
                        issue_stage(&issue.status),
                        Vec::new(),
                    )?;
                }
                InputIssueStatus::Cancel => {
                    write_direct_issue_projection(
                        &root,
                        issue,
                        IssueLoopStage::Cancel,
                        Vec::new(),
                    )?;
                }
                InputIssueStatus::Backlog => {
                    let blockers = direct_schedule_blockers(&root, issue, &done_issues);
                    if blockers.is_empty() {
                        ensure_context_pack(&root, issue)?;
                        agentflow_input::update_input_issue_status(
                            &root,
                            &issue.issue_id,
                            InputIssueStatus::Todo,
                        )?;
                        summary.active_issue_ids.push(issue.issue_id.clone());
                        write_direct_issue_projection(
                            &root,
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
                        summary.blocked_issue_ids.push(issue.issue_id.clone());
                        write_direct_issue_projection(
                            &root,
                            issue,
                            IssueLoopStage::Blocked,
                            blockers.clone(),
                        )?;
                        summary.blockers.extend(blockers);
                    }
                }
            }
        }

        agentflow_input::prepare_input_workspace(&root)?;
        Ok(summary)
    }
}

fn direct_schedule_blockers(
    root: &Path,
    issue: &InputIssue,
    done_issues: &BTreeSet<&str>,
) -> Vec<LoopBlocker> {
    let mut blockers = Vec::new();
    if issue.project_id.is_some() {
        blockers.push(blocker(
            "direct-boundary-invalid",
            "Direct issue must use projectId = null before scheduling.",
            Some(issue.issue_path.clone()),
        ));
    }
    if !matches!(issue.issue_category, IssueCategory::Spec)
        || !matches!(issue.required_agent_role, AgentRole::BuildAgent)
    {
        blockers.push(blocker(
            "build-agent-contract-invalid",
            "Direct Issue Loop only schedules spec issues assigned to Build Agent.",
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

fn write_direct_issue_projection(
    root: &Path,
    issue: &InputIssue,
    stage: IssueLoopStage,
    blockers: Vec<LoopBlocker>,
) -> Result<()> {
    let mut projection = IssueLoopProjection::new(None, &issue.issue_id, now());
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

fn canonical_project_root(project_root: impl AsRef<Path>) -> Result<PathBuf> {
    project_root
        .as_ref()
        .canonicalize()
        .with_context(|| format!("canonicalize {}", project_root.as_ref().display()))
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}
