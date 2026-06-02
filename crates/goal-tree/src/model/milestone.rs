use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MilestoneRecord {
    pub version: String,
    pub id: String,
    pub goal_id: String,
    pub project_root: String,
    pub status: String,
    pub human: MilestoneHumanContract,
    pub agent_draft: MilestoneAgentDraft,
    pub system: MilestoneSystemState,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MilestoneHumanContract {
    pub title: String,
    pub stage_goal: String,
    pub entry_criteria: Vec<String>,
    pub scope: Vec<String>,
    pub non_goals: Vec<String>,
    pub issue_order: Vec<String>,
    pub exit_criteria: Vec<String>,
    pub next_gate: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MilestoneAgentDraft {
    pub suggested_issues: Vec<String>,
    pub suggested_risks: Vec<String>,
    pub suggested_questions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MilestoneSystemState {
    pub created_at: u64,
    pub updated_at: u64,
    pub created_by: String,
    pub updated_by: String,
    pub path: String,
    pub revision: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CreateMilestoneInput {
    pub status: Option<String>,
    pub title: String,
    pub stage_goal: String,
    pub entry_criteria: Option<Vec<String>>,
    pub scope: Option<Vec<String>>,
    pub non_goals: Option<Vec<String>>,
    pub exit_criteria: Option<Vec<String>>,
    pub next_gate: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMilestoneInput {
    pub status: Option<String>,
    pub title: Option<String>,
    pub stage_goal: Option<String>,
    pub entry_criteria: Option<Vec<String>>,
    pub scope: Option<Vec<String>>,
    pub non_goals: Option<Vec<String>>,
    pub issue_order: Option<Vec<String>>,
    pub exit_criteria: Option<Vec<String>>,
    pub next_gate: Option<Vec<String>>,
    pub agent_draft: Option<MilestoneAgentDraft>,
}
