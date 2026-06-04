//! Goal Tree manager APIs.
//!
//! Read APIs may be used by the Desktop human UI. Write APIs in this module
//! mutate `.agentflow/define/**` and are agent-only / system-only / internal
//! test helpers. They must not be exposed through the human Desktop UI.

use crate::{
    ids::next_id,
    integrity::validate_loaded_goal_tree,
    model::{
        CreateGoalInput, CreateIssueInput, CreateMilestoneInput, GoalAgentDraft, GoalHumanContract,
        GoalRecord, GoalSystemState, GoalTreeIssueContextSnapshot, GoalTreeRecommendedFile,
        GoalTreeSnapshot, IssueAgentDraft, IssueHumanContract, IssueRecord, IssueSystemState,
        MilestoneAgentDraft, MilestoneHumanContract, MilestoneRecord, MilestoneSystemState,
        ReorderGoalTreeInput, UpdateGoalInput, UpdateIssueInput, UpdateMilestoneInput,
    },
    storage::{
        bump_record_revision, default_index, ensure_goal_tree_dirs, load_goals, load_index,
        load_issues, load_milestones, paths_for, read_goal, read_issue, read_milestone,
        relative_record_path, save_goal, save_index, save_issue, save_milestone,
        unix_timestamp_seconds,
    },
};
use anyhow::{Context, Result};
use std::path::Path;

pub fn load_goal_tree_snapshot(project_root: impl AsRef<Path>) -> Result<GoalTreeSnapshot> {
    let paths = paths_for(project_root)?;
    ensure_goal_tree_dirs(&paths)?;
    let mut index = load_index(&paths)?;
    index.project_root = paths.root.display().to_string();
    let goals = load_goals(&paths)?;
    let milestones = load_milestones(&paths)?;
    let issues = load_issues(&paths)?;
    let validation = validate_loaded_goal_tree(&paths.root, &index, &goals, &milestones, &issues);
    Ok(GoalTreeSnapshot {
        version: "goal-tree-snapshot.v1".to_string(),
        project_root: paths.root.display().to_string(),
        index,
        goals,
        milestones,
        issues,
        validation,
    })
}

/// Agent-only / system-only write API.
///
/// Mutates `.agentflow/define/**`; Desktop human UI must not call this function.
pub fn create_goal(project_root: impl AsRef<Path>, input: CreateGoalInput) -> Result<GoalRecord> {
    let paths = paths_for(project_root)?;
    ensure_goal_tree_dirs(&paths)?;
    let id = next_id(&paths.goals, "goal")?;
    let now = unix_timestamp_seconds();
    let status = input.status.unwrap_or_else(|| "draft".to_string());
    let record_path = paths.goals.join(format!("{id}.json"));
    let goal = GoalRecord {
        version: "goal.v1".to_string(),
        id: id.clone(),
        project_root: paths.root.display().to_string(),
        status,
        human: GoalHumanContract {
            title: input.title,
            objective: input.objective,
            scope: input.scope.unwrap_or_default(),
            non_goals: input.non_goals.unwrap_or_default(),
            success_criteria: input.success_criteria.unwrap_or_default(),
            milestone_order: Vec::new(),
            validation_gate: input.validation_gate.unwrap_or_default(),
            closure_gate: input.closure_gate.unwrap_or_default(),
        },
        agent_draft: GoalAgentDraft::default(),
        system: GoalSystemState {
            created_at: now,
            updated_at: now,
            created_by: "agent-system".to_string(),
            updated_by: "agent-system".to_string(),
            path: relative_record_path(&record_path, &paths.root),
            revision: 1,
        },
    };
    save_goal(&paths, &goal)?;

    let mut index = load_index(&paths).unwrap_or_else(|_| default_index(&paths));
    if !index.goal_order.contains(&id) {
        index.goal_order.push(id.clone());
    }
    if index.active_goal_id.is_none() {
        index.active_goal_id = Some(id);
    }
    index.updated_at = unix_timestamp_seconds();
    save_index(&paths, &index)?;

    Ok(goal)
}

/// Agent-only / system-only write API.
///
/// Mutates `.agentflow/define/**`; Desktop human UI must not call this function.
pub fn update_goal(
    project_root: impl AsRef<Path>,
    goal_id: &str,
    patch: UpdateGoalInput,
) -> Result<GoalRecord> {
    let paths = paths_for(project_root)?;
    ensure_goal_tree_dirs(&paths)?;
    let mut goal = read_goal(&paths, goal_id)?;
    if let Some(status) = patch.status {
        goal.status = status;
    }
    if let Some(title) = patch.title {
        goal.human.title = title;
    }
    if let Some(objective) = patch.objective {
        goal.human.objective = objective;
    }
    if let Some(scope) = patch.scope {
        goal.human.scope = scope;
    }
    if let Some(non_goals) = patch.non_goals {
        goal.human.non_goals = non_goals;
    }
    if let Some(success_criteria) = patch.success_criteria {
        goal.human.success_criteria = success_criteria;
    }
    if let Some(milestone_order) = patch.milestone_order {
        goal.human.milestone_order = milestone_order;
    }
    if let Some(validation_gate) = patch.validation_gate {
        goal.human.validation_gate = validation_gate;
    }
    if let Some(closure_gate) = patch.closure_gate {
        goal.human.closure_gate = closure_gate;
    }
    if let Some(agent_draft) = patch.agent_draft {
        goal.agent_draft = agent_draft;
    }
    let (updated_at, revision, updated_by) =
        bump_record_revision(goal.system.created_at, goal.system.revision);
    goal.system.updated_at = updated_at;
    goal.system.revision = revision;
    goal.system.updated_by = updated_by;
    save_goal(&paths, &goal)?;
    Ok(goal)
}

/// Agent-only / system-only write API.
///
/// Mutates `.agentflow/define/**`; Desktop human UI must not call this function.
pub fn archive_goal(project_root: impl AsRef<Path>, goal_id: &str) -> Result<GoalRecord> {
    let goal = update_goal(
        project_root.as_ref(),
        goal_id,
        UpdateGoalInput {
            status: Some("archived".to_string()),
            ..UpdateGoalInput::default()
        },
    )?;
    let paths = paths_for(project_root)?;
    let mut index = load_index(&paths)?;
    if index.active_goal_id.as_deref() == Some(goal_id) {
        index.active_goal_id = None;
    }
    index.updated_at = unix_timestamp_seconds();
    save_index(&paths, &index)?;
    Ok(goal)
}

/// Agent-only / system-only write API.
///
/// Mutates `.agentflow/define/**`; Desktop human UI must not call this function.
pub fn create_milestone(
    project_root: impl AsRef<Path>,
    goal_id: &str,
    input: CreateMilestoneInput,
) -> Result<MilestoneRecord> {
    let paths = paths_for(project_root)?;
    ensure_goal_tree_dirs(&paths)?;
    let goal = read_goal(&paths, goal_id).with_context(|| format!("goal {goal_id} is required"))?;
    let id = next_id(&paths.milestones, "ms")?;
    let now = unix_timestamp_seconds();
    let record_path = paths.milestones.join(format!("{id}.json"));
    let milestone = MilestoneRecord {
        version: "milestone.v1".to_string(),
        id: id.clone(),
        goal_id: goal.id.clone(),
        project_root: paths.root.display().to_string(),
        status: input.status.unwrap_or_else(|| "planned".to_string()),
        human: MilestoneHumanContract {
            title: input.title,
            stage_goal: input.stage_goal,
            entry_criteria: input.entry_criteria.unwrap_or_default(),
            scope: input.scope.unwrap_or_default(),
            non_goals: input.non_goals.unwrap_or_default(),
            issue_order: Vec::new(),
            exit_criteria: input.exit_criteria.unwrap_or_default(),
            next_gate: input.next_gate.unwrap_or_default(),
        },
        agent_draft: MilestoneAgentDraft::default(),
        system: MilestoneSystemState {
            created_at: now,
            updated_at: now,
            created_by: "agent-system".to_string(),
            updated_by: "agent-system".to_string(),
            path: relative_record_path(&record_path, &paths.root),
            revision: 1,
        },
    };
    save_milestone(&paths, &milestone)?;
    let mut index = load_index(&paths).unwrap_or_else(|_| default_index(&paths));
    index
        .milestone_order_by_goal
        .entry(goal.id.clone())
        .or_default()
        .push(id.clone());
    index.updated_at = unix_timestamp_seconds();
    save_index(&paths, &index)?;

    let mut goal = goal;
    if !goal.human.milestone_order.contains(&id) {
        goal.human.milestone_order.push(id);
        let (updated_at, revision, updated_by) =
            bump_record_revision(goal.system.created_at, goal.system.revision);
        goal.system.updated_at = updated_at;
        goal.system.revision = revision;
        goal.system.updated_by = updated_by;
        save_goal(&paths, &goal)?;
    }
    Ok(milestone)
}

/// Agent-only / system-only write API.
///
/// Mutates `.agentflow/define/**`; Desktop human UI must not call this function.
pub fn update_milestone(
    project_root: impl AsRef<Path>,
    milestone_id: &str,
    patch: UpdateMilestoneInput,
) -> Result<MilestoneRecord> {
    let paths = paths_for(project_root)?;
    ensure_goal_tree_dirs(&paths)?;
    let mut milestone = read_milestone(&paths, milestone_id)?;
    if let Some(status) = patch.status {
        milestone.status = status;
    }
    if let Some(title) = patch.title {
        milestone.human.title = title;
    }
    if let Some(stage_goal) = patch.stage_goal {
        milestone.human.stage_goal = stage_goal;
    }
    if let Some(entry_criteria) = patch.entry_criteria {
        milestone.human.entry_criteria = entry_criteria;
    }
    if let Some(scope) = patch.scope {
        milestone.human.scope = scope;
    }
    if let Some(non_goals) = patch.non_goals {
        milestone.human.non_goals = non_goals;
    }
    if let Some(issue_order) = patch.issue_order {
        milestone.human.issue_order = issue_order;
    }
    if let Some(exit_criteria) = patch.exit_criteria {
        milestone.human.exit_criteria = exit_criteria;
    }
    if let Some(next_gate) = patch.next_gate {
        milestone.human.next_gate = next_gate;
    }
    if let Some(agent_draft) = patch.agent_draft {
        milestone.agent_draft = agent_draft;
    }
    let (updated_at, revision, updated_by) =
        bump_record_revision(milestone.system.created_at, milestone.system.revision);
    milestone.system.updated_at = updated_at;
    milestone.system.revision = revision;
    milestone.system.updated_by = updated_by;
    save_milestone(&paths, &milestone)?;
    Ok(milestone)
}

/// Agent-only / system-only write API.
///
/// Mutates `.agentflow/define/**`; Desktop human UI must not call this function.
pub fn archive_milestone(
    project_root: impl AsRef<Path>,
    milestone_id: &str,
) -> Result<MilestoneRecord> {
    update_milestone(
        project_root,
        milestone_id,
        UpdateMilestoneInput {
            status: Some("archived".to_string()),
            ..UpdateMilestoneInput::default()
        },
    )
}

/// Agent-only / system-only write API.
///
/// Mutates `.agentflow/define/**`; Desktop human UI must not call this function.
pub fn create_issue(
    project_root: impl AsRef<Path>,
    milestone_id: &str,
    input: CreateIssueInput,
) -> Result<IssueRecord> {
    let paths = paths_for(project_root)?;
    ensure_goal_tree_dirs(&paths)?;
    let milestone = read_milestone(&paths, milestone_id)
        .with_context(|| format!("milestone {milestone_id} is required"))?;
    let id = next_id(&paths.issues, "iss")?;
    let now = unix_timestamp_seconds();
    let record_path = paths.issues.join(format!("{id}.json"));
    let issue = IssueRecord {
        version: "issue.v1".to_string(),
        id: id.clone(),
        goal_id: milestone.goal_id.clone(),
        milestone_id: milestone.id.clone(),
        project_root: paths.root.display().to_string(),
        status: input.status.unwrap_or_else(|| "draft".to_string()),
        human: IssueHumanContract {
            title: input.title,
            goal: input.goal,
            scope: input.scope.unwrap_or_default(),
            non_goals: input.non_goals.unwrap_or_default(),
            dependencies: input.dependencies.unwrap_or_default(),
            acceptance_criteria: input.acceptance_criteria.unwrap_or_default(),
            validation_commands: input.validation_commands.unwrap_or_default(),
            evidence_requirements: input.evidence_requirements.unwrap_or_default(),
            boundary: input.boundary.unwrap_or_default(),
        },
        agent_draft: IssueAgentDraft::default(),
        system: IssueSystemState {
            created_at: now,
            updated_at: now,
            created_by: "agent-system".to_string(),
            updated_by: "agent-system".to_string(),
            path: relative_record_path(&record_path, &paths.root),
            revision: 1,
            panel_context_pack_path: None,
        },
    };
    save_issue(&paths, &issue)?;
    let mut index = load_index(&paths).unwrap_or_else(|_| default_index(&paths));
    index
        .issue_order_by_milestone
        .entry(milestone.id.clone())
        .or_default()
        .push(id.clone());
    index.updated_at = unix_timestamp_seconds();
    save_index(&paths, &index)?;

    let mut milestone = milestone;
    if !milestone.human.issue_order.contains(&id) {
        milestone.human.issue_order.push(id);
        let (updated_at, revision, updated_by) =
            bump_record_revision(milestone.system.created_at, milestone.system.revision);
        milestone.system.updated_at = updated_at;
        milestone.system.revision = revision;
        milestone.system.updated_by = updated_by;
        save_milestone(&paths, &milestone)?;
    }
    Ok(issue)
}

/// Agent-only / system-only write API.
///
/// Mutates `.agentflow/define/**`; Desktop human UI must not call this function.
pub fn update_issue(
    project_root: impl AsRef<Path>,
    issue_id: &str,
    patch: UpdateIssueInput,
) -> Result<IssueRecord> {
    let paths = paths_for(project_root)?;
    ensure_goal_tree_dirs(&paths)?;
    let mut issue = read_issue(&paths, issue_id)?;
    if let Some(status) = patch.status {
        issue.status = status;
    }
    if let Some(title) = patch.title {
        issue.human.title = title;
    }
    if let Some(goal) = patch.goal {
        issue.human.goal = goal;
    }
    if let Some(scope) = patch.scope {
        issue.human.scope = scope;
    }
    if let Some(non_goals) = patch.non_goals {
        issue.human.non_goals = non_goals;
    }
    if let Some(dependencies) = patch.dependencies {
        issue.human.dependencies = dependencies;
    }
    if let Some(acceptance_criteria) = patch.acceptance_criteria {
        issue.human.acceptance_criteria = acceptance_criteria;
    }
    if let Some(validation_commands) = patch.validation_commands {
        issue.human.validation_commands = validation_commands;
    }
    if let Some(evidence_requirements) = patch.evidence_requirements {
        issue.human.evidence_requirements = evidence_requirements;
    }
    if let Some(boundary) = patch.boundary {
        issue.human.boundary = boundary;
    }
    if let Some(agent_draft) = patch.agent_draft {
        issue.agent_draft = agent_draft;
    }
    let (updated_at, revision, updated_by) =
        bump_record_revision(issue.system.created_at, issue.system.revision);
    issue.system.updated_at = updated_at;
    issue.system.revision = revision;
    issue.system.updated_by = updated_by;
    save_issue(&paths, &issue)?;
    Ok(issue)
}

/// Agent-only / system-only write API.
///
/// Mutates `.agentflow/define/**`; Desktop human UI must not call this function.
pub fn archive_issue(project_root: impl AsRef<Path>, issue_id: &str) -> Result<IssueRecord> {
    update_issue(
        project_root,
        issue_id,
        UpdateIssueInput {
            status: Some("archived".to_string()),
            ..UpdateIssueInput::default()
        },
    )
}

/// Agent-only / system-only write API.
///
/// Mutates `.agentflow/define/**`; Desktop human UI must not call this function.
pub fn reorder_goal_tree(
    project_root: impl AsRef<Path>,
    input: ReorderGoalTreeInput,
) -> Result<GoalTreeSnapshot> {
    let paths = paths_for(project_root)?;
    ensure_goal_tree_dirs(&paths)?;
    let mut index = load_index(&paths).unwrap_or_else(|_| default_index(&paths));
    if let Some(active_goal_id) = input.active_goal_id {
        index.active_goal_id = if active_goal_id.trim().is_empty() {
            None
        } else {
            Some(active_goal_id)
        };
    }
    if let Some(goal_order) = input.goal_order {
        index.goal_order = goal_order;
    }
    if let Some(milestone_order_by_goal) = input.milestone_order_by_goal {
        index.milestone_order_by_goal = milestone_order_by_goal;
    }
    if let Some(issue_order_by_milestone) = input.issue_order_by_milestone {
        index.issue_order_by_milestone = issue_order_by_milestone;
    }
    index.updated_at = unix_timestamp_seconds();
    save_index(&paths, &index)?;
    load_goal_tree_snapshot(paths.root)
}

/// Agent-only / system-only write API.
///
/// Records an existing Agent-prepared Panel Context Pack path. Desktop human UI
/// must not call this function or generate context packs.
pub fn record_issue_panel_context_path(
    project_root: impl AsRef<Path>,
    issue_id: &str,
    context_pack_path: Option<String>,
) -> Result<IssueRecord> {
    let paths = paths_for(project_root)?;
    let mut issue = read_issue(&paths, issue_id)?;
    issue.system.panel_context_pack_path = context_pack_path;
    let (updated_at, revision, updated_by) =
        bump_record_revision(issue.system.created_at, issue.system.revision);
    issue.system.updated_at = updated_at;
    issue.system.revision = revision;
    issue.system.updated_by = updated_by;
    save_issue(&paths, &issue)?;
    Ok(issue)
}

pub fn empty_issue_context_snapshot(
    project_root: impl AsRef<Path>,
    issue_id: &str,
    status: &str,
    warnings: Vec<String>,
) -> Result<GoalTreeIssueContextSnapshot> {
    let paths = paths_for(project_root)?;
    Ok(GoalTreeIssueContextSnapshot {
        version: "goal-tree-issue-context.v1".to_string(),
        project_root: paths.root.display().to_string(),
        issue_id: issue_id.to_string(),
        status: status.to_string(),
        context_pack_path: None,
        recommended_files: Vec::new(),
        recommended_tests: Vec::new(),
        warnings,
    })
}

pub fn context_snapshot_from_parts(
    project_root: impl AsRef<Path>,
    issue_id: &str,
    status: &str,
    context_pack_path: Option<String>,
    recommended_files: Vec<GoalTreeRecommendedFile>,
    recommended_tests: Vec<GoalTreeRecommendedFile>,
    warnings: Vec<String>,
) -> Result<GoalTreeIssueContextSnapshot> {
    let paths = paths_for(project_root)?;
    Ok(GoalTreeIssueContextSnapshot {
        version: "goal-tree-issue-context.v1".to_string(),
        project_root: paths.root.display().to_string(),
        issue_id: issue_id.to_string(),
        status: status.to_string(),
        context_pack_path,
        recommended_files,
        recommended_tests,
        warnings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn creates_goal_tree_records_under_define_paths() {
        let dir = tempdir().unwrap();
        let goal = create_goal(
            dir.path(),
            CreateGoalInput {
                title: "Goal Tree".to_string(),
                objective: "管理本地目标树".to_string(),
                ..CreateGoalInput::default()
            },
        )
        .unwrap();
        let milestone = create_milestone(
            dir.path(),
            &goal.id,
            CreateMilestoneInput {
                title: "Storage".to_string(),
                stage_goal: "写入 JSON 事实源".to_string(),
                ..CreateMilestoneInput::default()
            },
        )
        .unwrap();
        let issue = create_issue(
            dir.path(),
            &milestone.id,
            CreateIssueInput {
                title: "Create storage".to_string(),
                goal: "创建 Goal Tree 存储".to_string(),
                acceptance_criteria: Some(vec!["可以读取 snapshot".to_string()]),
                boundary: Some(vec!["不写用户源码".to_string()]),
                ..CreateIssueInput::default()
            },
        )
        .unwrap();

        assert!(dir
            .path()
            .join(".agentflow/define/goal-tree.json")
            .is_file());
        assert!(dir
            .path()
            .join(format!(".agentflow/define/goals/{}.json", goal.id))
            .is_file());
        assert!(dir
            .path()
            .join(format!(
                ".agentflow/define/milestones/{}.json",
                milestone.id
            ))
            .is_file());
        assert!(dir
            .path()
            .join(format!(".agentflow/define/issues/{}.json", issue.id))
            .is_file());
        assert!(!dir.path().join(".agentflow/issues").exists());
    }

    #[test]
    fn load_snapshot_includes_validation_warnings_without_panel_context() {
        let dir = tempdir().unwrap();
        let goal = create_goal(
            dir.path(),
            CreateGoalInput {
                title: "Goal Tree".to_string(),
                objective: "管理本地目标树".to_string(),
                ..CreateGoalInput::default()
            },
        )
        .unwrap();
        let milestone = create_milestone(
            dir.path(),
            &goal.id,
            CreateMilestoneInput {
                title: "Storage".to_string(),
                stage_goal: "写入 JSON 事实源".to_string(),
                ..CreateMilestoneInput::default()
            },
        )
        .unwrap();
        let issue = create_issue(
            dir.path(),
            &milestone.id,
            CreateIssueInput {
                status: Some("ready".to_string()),
                title: "Ready issue".to_string(),
                goal: "创建 ready issue".to_string(),
                acceptance_criteria: Some(vec!["验收可读".to_string()]),
                boundary: Some(vec!["不写用户源码".to_string()]),
                ..CreateIssueInput::default()
            },
        )
        .unwrap();

        let snapshot = load_goal_tree_snapshot(dir.path()).unwrap();

        assert_eq!(snapshot.goals.len(), 1);
        assert_eq!(snapshot.milestones.len(), 1);
        assert_eq!(snapshot.issues.len(), 1);
        assert!(snapshot.validation.valid);
        assert!(snapshot
            .validation
            .warnings
            .iter()
            .any(|warning| warning.object_id.as_deref() == Some(&issue.id)));
    }
}
