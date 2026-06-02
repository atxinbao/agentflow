//! Goal Tree V1 command wrappers.
//!
//! Goal Tree 是新的本地目标树事实源，只写 `.agentflow/define/**`。
//! 这里不启动 Agent、不执行项目命令、不复用 legacy workflow。

#[tauri::command]
pub(crate) fn load_goal_tree_snapshot(
    project_root: String,
) -> Result<agentflow_goal_tree::GoalTreeSnapshot, String> {
    agentflow_goal_tree::load_goal_tree_snapshot(project_root).map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn create_goal_tree_goal(
    project_root: String,
    input: agentflow_goal_tree::CreateGoalInput,
) -> Result<agentflow_goal_tree::GoalRecord, String> {
    agentflow_goal_tree::create_goal(project_root, input).map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn update_goal_tree_goal(
    project_root: String,
    goal_id: String,
    patch: agentflow_goal_tree::UpdateGoalInput,
) -> Result<agentflow_goal_tree::GoalRecord, String> {
    agentflow_goal_tree::update_goal(project_root, &goal_id, patch)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn archive_goal_tree_goal(
    project_root: String,
    goal_id: String,
) -> Result<agentflow_goal_tree::GoalRecord, String> {
    agentflow_goal_tree::archive_goal(project_root, &goal_id).map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn create_goal_tree_milestone(
    project_root: String,
    goal_id: String,
    input: agentflow_goal_tree::CreateMilestoneInput,
) -> Result<agentflow_goal_tree::MilestoneRecord, String> {
    agentflow_goal_tree::create_milestone(project_root, &goal_id, input)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn update_goal_tree_milestone(
    project_root: String,
    milestone_id: String,
    patch: agentflow_goal_tree::UpdateMilestoneInput,
) -> Result<agentflow_goal_tree::MilestoneRecord, String> {
    agentflow_goal_tree::update_milestone(project_root, &milestone_id, patch)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn archive_goal_tree_milestone(
    project_root: String,
    milestone_id: String,
) -> Result<agentflow_goal_tree::MilestoneRecord, String> {
    agentflow_goal_tree::archive_milestone(project_root, &milestone_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn create_goal_tree_issue(
    project_root: String,
    milestone_id: String,
    input: agentflow_goal_tree::CreateIssueInput,
) -> Result<agentflow_goal_tree::IssueRecord, String> {
    agentflow_goal_tree::create_issue(project_root, &milestone_id, input)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn update_goal_tree_issue(
    project_root: String,
    issue_id: String,
    patch: agentflow_goal_tree::UpdateIssueInput,
) -> Result<agentflow_goal_tree::IssueRecord, String> {
    agentflow_goal_tree::update_issue(project_root, &issue_id, patch)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn archive_goal_tree_issue(
    project_root: String,
    issue_id: String,
) -> Result<agentflow_goal_tree::IssueRecord, String> {
    agentflow_goal_tree::archive_issue(project_root, &issue_id).map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn reorder_goal_tree(
    project_root: String,
    input: agentflow_goal_tree::ReorderGoalTreeInput,
) -> Result<agentflow_goal_tree::GoalTreeSnapshot, String> {
    agentflow_goal_tree::reorder_goal_tree(project_root, input).map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn validate_goal_tree(
    project_root: String,
) -> Result<agentflow_goal_tree::GoalTreeValidationSnapshot, String> {
    agentflow_goal_tree::validate_goal_tree(project_root).map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn prepare_goal_tree_issue_context(
    project_root: String,
    issue_id: String,
) -> Result<agentflow_goal_tree::GoalTreeIssueContextSnapshot, String> {
    let snapshot = agentflow_goal_tree::load_goal_tree_snapshot(&project_root)
        .map_err(|error| error.to_string())?;
    let Some(issue) = snapshot.issues.iter().find(|item| item.id == issue_id) else {
        return Err(format!("issue not found: {issue_id}"));
    };

    let pack = match agentflow_graph::build_context_pack(
        &project_root,
        "goal-tree-issue",
        Some(&issue.id),
        &issue.human.title,
        &issue.human.goal,
        &issue.human.acceptance_criteria,
    ) {
        Ok(pack) => pack,
        Err(error) => {
            return agentflow_goal_tree::empty_issue_context_snapshot(
                project_root,
                &issue_id,
                "graph-degraded",
                vec![format!(
                    "代码地图尚未准备好，Issue 上下文推荐可能不完整：{error}"
                )],
            )
            .map_err(|error| error.to_string());
        }
    };

    let context_pack_path = Some(format!(
        ".agentflow/output/graph/context-packs/{}.json",
        issue.id
    ));
    agentflow_goal_tree::record_issue_graph_context_path(
        &project_root,
        &issue.id,
        context_pack_path.clone(),
    )
    .map_err(|error| error.to_string())?;

    agentflow_goal_tree::context_snapshot_from_parts(
        project_root,
        &issue.id,
        "ready",
        context_pack_path,
        pack.recommended_files
            .into_iter()
            .map(|item| agentflow_goal_tree::GoalTreeRecommendedFile {
                path: item.path,
                reason: item.reason,
                score: item.score,
            })
            .collect(),
        pack.recommended_tests
            .into_iter()
            .map(|item| agentflow_goal_tree::GoalTreeRecommendedFile {
                path: item.path,
                reason: item.reason,
                score: item.score,
            })
            .collect(),
        Vec::new(),
    )
    .map_err(|error| error.to_string())
}
