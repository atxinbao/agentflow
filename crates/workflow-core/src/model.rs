use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const AGENTFLOW_WORKFLOW_API_VERSION: &str = "agentflow.dev/v1";
pub const TASK_WORKFLOW_KIND: &str = "TaskWorkflow";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowDefinition {
    pub api_version: String,
    pub kind: String,
    pub metadata: WorkflowMetadata,
    pub spec: WorkflowSpec,
}

impl WorkflowDefinition {
    pub fn workflow_ref(&self) -> String {
        format!("{}@{}", self.metadata.name, self.metadata.version)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowMetadata {
    pub name: String,
    pub version: String,
    pub title: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowSpec {
    pub initial_state: String,
    pub terminal_states: Vec<String>,
    pub states: BTreeMap<String, StateDefinition>,
    pub transitions: Vec<TransitionDefinition>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StateDefinition {
    pub label: String,
    pub phase: WorkflowStatePhase,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WorkflowStatePhase {
    Future,
    Current,
    Past,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransitionDefinition {
    pub id: String,
    #[serde(rename = "from", deserialize_with = "deserialize_state_refs")]
    pub from_states: Vec<String>,
    pub to: String,
    pub on: String,
    #[serde(default)]
    pub guards: Vec<GuardDefinition>,
    #[serde(default)]
    pub actions: Vec<ActionDefinition>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GuardDefinition {
    Reference(String),
    Detailed {
        name: String,
        #[serde(default)]
        description: Option<String>,
    },
}

impl GuardDefinition {
    pub fn name(&self) -> &str {
        match self {
            Self::Reference(name) => name,
            Self::Detailed { name, .. } => name,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ActionDefinition {
    Reference(String),
    Detailed {
        name: String,
        #[serde(default)]
        description: Option<String>,
    },
}

impl ActionDefinition {
    pub fn name(&self) -> &str {
        match self {
            Self::Reference(name) => name,
            Self::Detailed { name, .. } => name,
        }
    }
}

fn deserialize_state_refs<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StateRefs {
        One(String),
        Many(Vec<String>),
    }

    match StateRefs::deserialize(deserializer)? {
        StateRefs::One(value) => Ok(vec![value]),
        StateRefs::Many(values) => Ok(values),
    }
}
