use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct IssueRecord {
    pub version: String,
    pub id: String,
    pub goal_id: String,
    pub milestone_id: String,
    pub project_root: String,
    pub status: String,
    /// Human confirmed contract. The field name is kept for schema v1
    /// compatibility; it does not mean Desktop human UI can edit it.
    pub human: IssueHumanContract,
    pub agent_draft: IssueAgentDraft,
    pub system: IssueSystemState,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct IssueHumanContract {
    pub title: String,
    pub goal: String,
    pub scope: Vec<String>,
    pub non_goals: Vec<String>,
    pub dependencies: Vec<String>,
    pub acceptance_criteria: Vec<String>,
    pub validation_commands: Vec<String>,
    pub evidence_requirements: Vec<String>,
    pub boundary: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct IssueAgentDraft {
    pub suggested_files: Vec<String>,
    pub suggested_symbols: Vec<String>,
    pub suggested_tests: Vec<String>,
    pub suggested_implementation_plan: Vec<String>,
    pub suggested_risks: Vec<String>,
    pub questions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct IssueSystemState {
    pub created_at: u64,
    pub updated_at: u64,
    pub created_by: String,
    pub updated_by: String,
    pub path: String,
    pub revision: u64,
    pub panel_context_pack_path: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CreateIssueInput {
    pub status: Option<String>,
    pub title: String,
    pub goal: String,
    pub scope: Option<Vec<String>>,
    pub non_goals: Option<Vec<String>>,
    pub dependencies: Option<Vec<String>>,
    pub acceptance_criteria: Option<Vec<String>>,
    pub validation_commands: Option<Vec<String>>,
    pub evidence_requirements: Option<Vec<String>>,
    pub boundary: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateIssueInput {
    pub status: Option<String>,
    pub title: Option<String>,
    pub goal: Option<String>,
    pub scope: Option<Vec<String>>,
    pub non_goals: Option<Vec<String>>,
    pub dependencies: Option<Vec<String>>,
    pub acceptance_criteria: Option<Vec<String>>,
    pub validation_commands: Option<Vec<String>>,
    pub evidence_requirements: Option<Vec<String>>,
    pub boundary: Option<Vec<String>>,
    pub agent_draft: Option<IssueAgentDraft>,
}
