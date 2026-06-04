use serde::{Deserialize, Serialize};

use crate::issue::{InputPanelLink, InputSystemRecord};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InputProjectStatus {
    Planned,
    Active,
    Blocked,
    Done,
    Canceled,
}

impl Default for InputProjectStatus {
    fn default() -> Self {
        Self::Planned
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputProject {
    pub version: String,
    pub project_id: String,
    pub source_spec_id: String,
    pub title: String,
    pub summary: String,
    pub objective: String,
    pub scope: Vec<String>,
    pub non_goals: Vec<String>,
    pub success_criteria: Vec<String>,
    pub issue_ids: Vec<String>,
    pub status: InputProjectStatus,
    pub panel: InputPanelLink,
    pub system: InputSystemRecord,
}

impl Default for InputProject {
    fn default() -> Self {
        Self {
            version: "input-project.v1".to_string(),
            project_id: String::new(),
            source_spec_id: String::new(),
            title: String::new(),
            summary: String::new(),
            objective: String::new(),
            scope: Vec::new(),
            non_goals: Vec::new(),
            success_criteria: Vec::new(),
            issue_ids: Vec::new(),
            status: InputProjectStatus::default(),
            panel: InputPanelLink::default(),
            system: InputSystemRecord::default(),
        }
    }
}
