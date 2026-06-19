use serde::{Deserialize, Serialize};

pub const OBJECT_STATE_BUNDLE_VERSION: &str = "agentflow-object-state-bundle.v1";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ObjectStateMachineStatus {
    Draft,
    Active,
    Deprecated,
    Retired,
}

impl Default for ObjectStateMachineStatus {
    fn default() -> Self {
        Self::Draft
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectStateMachineBundle {
    pub version: String,
    pub registry_id: String,
    pub namespace: String,
    pub definition_version: String,
    pub status: ObjectStateMachineStatus,
    #[serde(default)]
    pub state_machines: Vec<ObjectStateMachine>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectStateMachine {
    pub state_machine_id: String,
    pub object_type: String,
    pub namespace: String,
    pub version: String,
    pub status: ObjectStateMachineStatus,
    pub initial_state: String,
    #[serde(default)]
    pub states: Vec<ObjectStateDefinition>,
    #[serde(default)]
    pub terminal_states: Vec<String>,
    #[serde(default)]
    pub transitions: Vec<StateTransitionDefinition>,
    #[serde(default)]
    pub invariants: Vec<String>,
    pub projection_hints: StateProjectionHints,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectStateDefinition {
    pub state_id: String,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub is_terminal: bool,
    #[serde(default)]
    pub is_error_state: bool,
    #[serde(default)]
    pub legacy_status_aliases: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StateTransitionDefinition {
    pub transition_id: String,
    #[serde(rename = "from", default)]
    pub from_states: Vec<String>,
    pub to_state: String,
    pub action_type: String,
    #[serde(default)]
    pub compatibility_action_aliases: Vec<String>,
    #[serde(default)]
    pub guards: Vec<String>,
    #[serde(default)]
    pub required_evidence: Vec<String>,
    #[serde(default)]
    pub emitted_events: Vec<String>,
    #[serde(default)]
    pub link_effects: Vec<String>,
    #[serde(default)]
    pub role_hints: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StateProjectionHints {
    #[serde(default)]
    pub timeline_order: Vec<String>,
    #[serde(default)]
    pub preferred_current_states: Vec<String>,
}
