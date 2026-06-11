use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InputIntakeStatus {
    NeedsClarification,
    AnswerOnly,
    BlockedByBoundary,
    ReadyForSpec,
}

impl Default for InputIntakeStatus {
    fn default() -> Self {
        Self::NeedsClarification
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputIntakeResult {
    pub version: String,
    pub intake_id: String,
    pub raw_input: String,
    pub request_type: String,
    pub agent_understanding: String,
    pub clarification_questions: Vec<String>,
    pub status: InputIntakeStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InputSpecStatus {
    Draft,
    Approved,
    Archived,
}

impl Default for InputSpecStatus {
    fn default() -> Self {
        Self::Draft
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputSpecDescriptor {
    pub version: String,
    pub spec_id: String,
    pub status: InputSpecStatus,
    pub product_path: String,
    pub tech_path: String,
    pub approval_path: Option<String>,
    pub spec_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InputIssueGenerationMode {
    Direct,
    Project,
}

impl Default for InputIssueGenerationMode {
    fn default() -> Self {
        Self::Project
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputSpecApproval {
    pub version: String,
    pub spec_id: String,
    pub approved_by: String,
    pub approved_at: u64,
    pub approved_product_hash: String,
    pub approved_tech_hash: String,
    pub issue_generation_mode: InputIssueGenerationMode,
    pub authorized_outputs: Vec<String>,
    pub not_authorized: Vec<String>,
}

impl Default for InputSpecApproval {
    fn default() -> Self {
        Self {
            version: "input-spec-approval.v1".to_string(),
            spec_id: String::new(),
            approved_by: "human".to_string(),
            approved_at: 0,
            approved_product_hash: String::new(),
            approved_tech_hash: String::new(),
            issue_generation_mode: InputIssueGenerationMode::default(),
            authorized_outputs: vec!["issues".to_string()],
            not_authorized: vec![
                "execute".to_string(),
                "sourceWrites".to_string(),
                "remotePr".to_string(),
                "merge".to_string(),
                "release".to_string(),
            ],
        }
    }
}
