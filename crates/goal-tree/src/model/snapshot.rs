use super::{GoalRecord, GoalTreeIndex, IssueRecord, MilestoneRecord};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GoalTreeSnapshot {
    pub version: String,
    pub project_root: String,
    pub index: GoalTreeIndex,
    pub goals: Vec<GoalRecord>,
    pub milestones: Vec<MilestoneRecord>,
    pub issues: Vec<IssueRecord>,
    pub validation: GoalTreeValidationSnapshot,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GoalTreeValidationSnapshot {
    pub version: String,
    pub project_root: String,
    pub valid: bool,
    pub errors: Vec<GoalTreeValidationIssue>,
    pub warnings: Vec<GoalTreeValidationIssue>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GoalTreeValidationIssue {
    pub code: String,
    pub message: String,
    pub object_type: String,
    pub object_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GoalTreeIssueContextSnapshot {
    pub version: String,
    pub project_root: String,
    pub issue_id: String,
    pub status: String,
    pub context_pack_path: Option<String>,
    pub recommended_files: Vec<GoalTreeRecommendedFile>,
    pub recommended_tests: Vec<GoalTreeRecommendedFile>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GoalTreeRecommendedFile {
    pub path: String,
    pub reason: String,
    pub score: f64,
}
