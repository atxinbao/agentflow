//! Goal Tree V1 command wrappers.
//!
//! Goal Tree 是 Agent 使用的本地目标树事实源。Desktop 人类界面只读取
//! `.agentflow/define/**`，不创建、不编辑、不归档、不排序、不准备 Context Pack。
//! 写入能力保留在 `agentflow-goal-tree` crate 的 agent-only/internal API 中，
//! 不通过 Desktop Tauri command surface 暴露。

#[tauri::command]
pub(crate) fn load_goal_tree_snapshot(
    project_root: String,
) -> Result<agentflow_goal_tree::GoalTreeSnapshot, String> {
    agentflow_goal_tree::load_goal_tree_snapshot(project_root).map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn validate_goal_tree(
    project_root: String,
) -> Result<agentflow_goal_tree::GoalTreeValidationSnapshot, String> {
    agentflow_goal_tree::validate_goal_tree(project_root).map_err(|error| error.to_string())
}
