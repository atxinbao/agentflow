use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InputIssueRelationKind {
    BlockedBy,
    Blocks,
    Related,
    DuplicateOf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputIssueRelation {
    pub from_issue_id: String,
    pub to_issue_id: String,
    #[serde(rename = "type")]
    pub relation_type: InputIssueRelationKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputIssueRelationsFile {
    pub version: String,
    pub relations: Vec<InputIssueRelation>,
}

impl Default for InputIssueRelationsFile {
    fn default() -> Self {
        Self {
            version: "input-issue-relations.v1".to_string(),
            relations: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputDependencyGraph {
    pub version: String,
    pub nodes: Vec<String>,
    pub edges: Vec<InputIssueRelation>,
}

impl Default for InputDependencyGraph {
    fn default() -> Self {
        Self {
            version: "input-dependency-graph.v1".to_string(),
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }
}
