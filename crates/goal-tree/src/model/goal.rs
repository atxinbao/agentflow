use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GoalRecord {
    pub version: String,
    pub id: String,
    pub project_root: String,
    pub status: String,
    pub human: GoalHumanContract,
    pub agent_draft: GoalAgentDraft,
    pub system: GoalSystemState,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GoalHumanContract {
    pub title: String,
    pub objective: String,
    pub scope: Vec<String>,
    pub non_goals: Vec<String>,
    pub success_criteria: Vec<String>,
    pub milestone_order: Vec<String>,
    pub validation_gate: Vec<String>,
    pub closure_gate: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GoalAgentDraft {
    pub suggested_milestones: Vec<String>,
    pub suggested_risks: Vec<String>,
    pub suggested_questions: Vec<String>,
    pub suggested_issue_breakdown: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GoalSystemState {
    pub created_at: u64,
    pub updated_at: u64,
    pub created_by: String,
    pub updated_by: String,
    pub path: String,
    pub revision: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CreateGoalInput {
    pub status: Option<String>,
    pub title: String,
    pub objective: String,
    pub scope: Option<Vec<String>>,
    pub non_goals: Option<Vec<String>>,
    pub success_criteria: Option<Vec<String>>,
    pub validation_gate: Option<Vec<String>>,
    pub closure_gate: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateGoalInput {
    pub status: Option<String>,
    pub title: Option<String>,
    pub objective: Option<String>,
    pub scope: Option<Vec<String>>,
    pub non_goals: Option<Vec<String>>,
    pub success_criteria: Option<Vec<String>>,
    pub milestone_order: Option<Vec<String>>,
    pub validation_gate: Option<Vec<String>>,
    pub closure_gate: Option<Vec<String>>,
    pub agent_draft: Option<GoalAgentDraft>,
}
