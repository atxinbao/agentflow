use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GoalTreeIndex {
    pub version: String,
    pub project_root: String,
    pub active_goal_id: Option<String>,
    pub goal_order: Vec<String>,
    pub milestone_order_by_goal: BTreeMap<String, Vec<String>>,
    pub issue_order_by_milestone: BTreeMap<String, Vec<String>>,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ReorderGoalTreeInput {
    pub active_goal_id: Option<String>,
    pub goal_order: Option<Vec<String>>,
    pub milestone_order_by_goal: Option<BTreeMap<String, Vec<String>>>,
    pub issue_order_by_milestone: Option<BTreeMap<String, Vec<String>>>,
}
