use crate::{
    model::{
        GoalRecord, GoalTreeIndex, GoalTreeValidationIssue, GoalTreeValidationSnapshot,
        IssueRecord, MilestoneRecord,
    },
    storage::{load_goals, load_index, load_issues, load_milestones, paths_for},
};
use anyhow::Result;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

pub fn validate_goal_tree(project_root: impl AsRef<Path>) -> Result<GoalTreeValidationSnapshot> {
    let paths = paths_for(project_root)?;
    let index = load_index(&paths)?;
    let goals = load_goals(&paths)?;
    let milestones = load_milestones(&paths)?;
    let issues = load_issues(&paths)?;
    Ok(validate_loaded_goal_tree(
        &paths.root,
        &index,
        &goals,
        &milestones,
        &issues,
    ))
}

pub(crate) fn validate_loaded_goal_tree(
    project_root: &Path,
    index: &GoalTreeIndex,
    goals: &[GoalRecord],
    milestones: &[MilestoneRecord],
    issues: &[IssueRecord],
) -> GoalTreeValidationSnapshot {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let goal_ids = goals
        .iter()
        .map(|item| item.id.clone())
        .collect::<BTreeSet<_>>();
    let milestone_ids = milestones
        .iter()
        .map(|item| item.id.clone())
        .collect::<BTreeSet<_>>();
    let issue_ids = issues
        .iter()
        .map(|item| item.id.clone())
        .collect::<BTreeSet<_>>();
    let goals_by_id = goals
        .iter()
        .map(|goal| (goal.id.clone(), goal))
        .collect::<BTreeMap<_, _>>();
    let issues_by_id = issues
        .iter()
        .map(|issue| (issue.id.clone(), issue))
        .collect::<BTreeMap<_, _>>();

    if let Some(active_goal_id) = &index.active_goal_id {
        match goals_by_id.get(active_goal_id) {
            Some(goal) if goal.status == "archived" => errors.push(issue(
                "archived_active_goal",
                "archived Goal 不能作为 activeGoalId。",
                "goal",
                Some(active_goal_id),
            )),
            Some(_) => {}
            None => errors.push(issue(
                "missing_active_goal",
                "activeGoalId 引用的 Goal 不存在。",
                "goal",
                Some(active_goal_id),
            )),
        }
    }

    for goal_id in &index.goal_order {
        if !goal_ids.contains(goal_id) {
            errors.push(issue(
                "missing_goal_order_ref",
                "goalOrder 引用的 Goal 不存在。",
                "goal",
                Some(goal_id),
            ));
        }
    }

    for milestone in milestones {
        if !goal_ids.contains(&milestone.goal_id) {
            errors.push(issue(
                "missing_milestone_goal",
                "Milestone.goalId 引用的 Goal 不存在。",
                "milestone",
                Some(&milestone.id),
            ));
        }
    }

    for issue_record in issues {
        if !goal_ids.contains(&issue_record.goal_id) {
            errors.push(issue(
                "missing_issue_goal",
                "Issue.goalId 引用的 Goal 不存在。",
                "issue",
                Some(&issue_record.id),
            ));
        }
        if !milestone_ids.contains(&issue_record.milestone_id) {
            errors.push(issue(
                "missing_issue_milestone",
                "Issue.milestoneId 引用的 Milestone 不存在。",
                "issue",
                Some(&issue_record.id),
            ));
        }
        for dependency_id in &issue_record.human.dependencies {
            match issues_by_id.get(dependency_id) {
                Some(dependency) if dependency.status == "archived" => errors.push(issue(
                    "archived_dependency",
                    "Issue 不能引用 archived 对象作为 active dependency。",
                    "issue",
                    Some(&issue_record.id),
                )),
                Some(_) => {}
                None => errors.push(issue(
                    "missing_issue_dependency",
                    "Issue.dependencies 引用的 Issue 不存在。",
                    "issue",
                    Some(&issue_record.id),
                )),
            }
        }
        if issue_record.status == "ready" {
            if issue_record.human.title.trim().is_empty() {
                errors.push(issue(
                    "ready_issue_missing_title",
                    "ready Issue 必须填写 title。",
                    "issue",
                    Some(&issue_record.id),
                ));
            }
            if issue_record.human.goal.trim().is_empty() {
                errors.push(issue(
                    "ready_issue_missing_goal",
                    "ready Issue 必须填写 goal。",
                    "issue",
                    Some(&issue_record.id),
                ));
            }
            if issue_record.human.acceptance_criteria.is_empty() {
                errors.push(issue(
                    "ready_issue_missing_acceptance",
                    "ready Issue 必须至少有一条 acceptanceCriteria。",
                    "issue",
                    Some(&issue_record.id),
                ));
            }
            if issue_record.human.boundary.is_empty() {
                errors.push(issue(
                    "ready_issue_missing_boundary",
                    "ready Issue 必须至少有一条 boundary。",
                    "issue",
                    Some(&issue_record.id),
                ));
            }
        }
        if issue_record.human.validation_commands.is_empty() {
            warnings.push(issue(
                "missing_validation_commands",
                "Issue 缺少 validationCommands；V1 不阻塞，但执行前需要补齐。",
                "issue",
                Some(&issue_record.id),
            ));
        }
        if issue_record.human.evidence_requirements.is_empty() {
            warnings.push(issue(
                "missing_evidence_requirements",
                "Issue 缺少 evidenceRequirements；V1 不阻塞，但完成前需要补齐。",
                "issue",
                Some(&issue_record.id),
            ));
        }
        if issue_record.system.graph_context_pack_path.is_none() {
            warnings.push(issue(
                "missing_graph_context_pack",
                "代码地图尚未准备好，Issue 上下文推荐可能不完整。",
                "issue",
                Some(&issue_record.id),
            ));
        }
    }

    for (goal_id, milestone_order) in &index.milestone_order_by_goal {
        if !goal_ids.contains(goal_id) {
            errors.push(issue(
                "missing_milestone_order_goal",
                "milestoneOrderByGoal 的 Goal 不存在。",
                "goal",
                Some(goal_id),
            ));
        }
        for milestone_id in milestone_order {
            if !milestone_ids.contains(milestone_id) {
                errors.push(issue(
                    "missing_milestone_order_ref",
                    "milestoneOrderByGoal 引用的 Milestone 不存在。",
                    "milestone",
                    Some(milestone_id),
                ));
            }
        }
    }

    for (milestone_id, issue_order) in &index.issue_order_by_milestone {
        if !milestone_ids.contains(milestone_id) {
            errors.push(issue(
                "missing_issue_order_milestone",
                "issueOrderByMilestone 的 Milestone 不存在。",
                "milestone",
                Some(milestone_id),
            ));
        }
        for issue_id in issue_order {
            if !issue_ids.contains(issue_id) {
                errors.push(issue(
                    "missing_issue_order_ref",
                    "issueOrderByMilestone 引用的 Issue 不存在。",
                    "issue",
                    Some(issue_id),
                ));
            }
        }
    }

    for goal in goals {
        if goal.status == "completed"
            && milestones
                .iter()
                .any(|milestone| milestone.goal_id == goal.id && milestone.status == "active")
        {
            errors.push(issue(
                "completed_goal_has_active_milestone",
                "completed Goal 不能包含 active Milestone。",
                "goal",
                Some(&goal.id),
            ));
        }
    }

    for milestone in milestones {
        if milestone.status == "completed"
            && issues.iter().any(|item| {
                item.milestone_id == milestone.id
                    && (item.status == "ready" || item.status == "blocked")
            })
        {
            errors.push(issue(
                "completed_milestone_has_open_issue",
                "completed Milestone 不能包含 ready / blocked Issue。",
                "milestone",
                Some(&milestone.id),
            ));
        }
    }

    if let Some(cycle) = first_dependency_cycle(issues) {
        errors.push(issue(
            "cyclic_issue_dependency",
            &format!("Issue dependencies 出现循环依赖：{}。", cycle.join(" -> ")),
            "issue",
            cycle.first(),
        ));
    }

    GoalTreeValidationSnapshot {
        version: "goal-tree-validation.v1".to_string(),
        project_root: project_root.display().to_string(),
        valid: errors.is_empty(),
        errors,
        warnings,
    }
}

fn first_dependency_cycle(issues: &[IssueRecord]) -> Option<Vec<String>> {
    let graph = issues
        .iter()
        .map(|item| (item.id.clone(), item.human.dependencies.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut visiting = BTreeSet::new();
    let mut visited = BTreeSet::new();
    let mut stack = Vec::<String>::new();
    for issue_id in graph.keys() {
        if let Some(cycle) = visit(issue_id, &graph, &mut visiting, &mut visited, &mut stack) {
            return Some(cycle);
        }
    }
    None
}

fn visit(
    issue_id: &str,
    graph: &BTreeMap<String, Vec<String>>,
    visiting: &mut BTreeSet<String>,
    visited: &mut BTreeSet<String>,
    stack: &mut Vec<String>,
) -> Option<Vec<String>> {
    if visited.contains(issue_id) {
        return None;
    }
    if visiting.contains(issue_id) {
        let start = stack.iter().position(|item| item == issue_id).unwrap_or(0);
        let mut cycle = stack[start..].to_vec();
        cycle.push(issue_id.to_string());
        return Some(cycle);
    }
    visiting.insert(issue_id.to_string());
    stack.push(issue_id.to_string());
    for dependency in graph.get(issue_id).into_iter().flatten() {
        if graph.contains_key(dependency) {
            if let Some(cycle) = visit(dependency, graph, visiting, visited, stack) {
                return Some(cycle);
            }
        }
    }
    stack.pop();
    visiting.remove(issue_id);
    visited.insert(issue_id.to_string());
    None
}

fn issue(
    code: &str,
    message: &str,
    object_type: &str,
    object_id: Option<&String>,
) -> GoalTreeValidationIssue {
    GoalTreeValidationIssue {
        code: code.to_string(),
        message: message.to_string(),
        object_type: object_type.to_string(),
        object_id: object_id.cloned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        create_goal, create_issue, create_milestone, update_issue, CreateGoalInput,
        CreateIssueInput, CreateMilestoneInput, UpdateIssueInput,
    };
    use tempfile::tempdir;

    #[test]
    fn detects_cyclic_issue_dependencies() {
        let dir = tempdir().unwrap();
        let goal = create_goal(
            dir.path(),
            CreateGoalInput {
                title: "Goal".to_string(),
                objective: "Objective".to_string(),
                ..CreateGoalInput::default()
            },
        )
        .unwrap();
        let milestone = create_milestone(
            dir.path(),
            &goal.id,
            CreateMilestoneInput {
                title: "Milestone".to_string(),
                stage_goal: "Stage".to_string(),
                ..CreateMilestoneInput::default()
            },
        )
        .unwrap();
        let issue_a = create_issue(
            dir.path(),
            &milestone.id,
            CreateIssueInput {
                title: "A".to_string(),
                goal: "A".to_string(),
                dependencies: Some(vec![]),
                acceptance_criteria: Some(vec!["done".to_string()]),
                boundary: Some(vec!["bounded".to_string()]),
                ..CreateIssueInput::default()
            },
        )
        .unwrap();
        let issue_b = create_issue(
            dir.path(),
            &milestone.id,
            CreateIssueInput {
                title: "B".to_string(),
                goal: "B".to_string(),
                dependencies: Some(vec![issue_a.id.clone()]),
                acceptance_criteria: Some(vec!["done".to_string()]),
                boundary: Some(vec!["bounded".to_string()]),
                ..CreateIssueInput::default()
            },
        )
        .unwrap();
        update_issue(
            dir.path(),
            &issue_a.id,
            UpdateIssueInput {
                dependencies: Some(vec![issue_b.id.clone()]),
                ..UpdateIssueInput::default()
            },
        )
        .unwrap();

        let validation = validate_goal_tree(dir.path()).unwrap();

        assert!(!validation.valid);
        assert!(validation
            .errors
            .iter()
            .any(|item| item.code == "cyclic_issue_dependency"));
    }
}
