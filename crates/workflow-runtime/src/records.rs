use agentflow_action_arbitration::{DefinitionVersions, ObjectLockPlan};
use agentflow_action_contract::{ActionRef, ActionSourceSurface};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const RUNTIME_COMMAND_FACT_VERSION: &str = "agentflow-runtime-command-fact.v1";
pub const RUNTIME_PROPOSAL_FACT_VERSION: &str = "agentflow-runtime-proposal-fact.v1";
pub const RUNTIME_DECISION_FACT_VERSION: &str = "agentflow-runtime-decision-fact.v1";
pub const RUNTIME_ACCEPTED_ACTION_FACT_VERSION: &str = "agentflow-runtime-accepted-action-fact.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeCommandValidationFact {
    pub valid: bool,
    #[serde(default)]
    pub normalized_action_type: Option<String>,
    #[serde(default)]
    pub errors: Vec<String>,
    #[serde(default)]
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeQueryHintFact {
    pub view: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_id: Option<String>,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeCommandFact {
    pub version: String,
    pub command_id: String,
    pub command_type: String,
    pub source_surface: ActionSourceSurface,
    pub actor_role: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_object_ref: Option<ActionRef>,
    pub input: Value,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
    #[serde(default)]
    pub artifact_refs: Vec<String>,
    pub idempotency_key: String,
    pub created_at: String,
    pub recorded_at: u64,
    pub validation: RuntimeCommandValidationFact,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeProposalFact {
    pub version: String,
    pub command_id: String,
    pub proposal_id: String,
    pub action_type: String,
    pub actor_role: String,
    pub source_surface: ActionSourceSurface,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_object_ref: Option<ActionRef>,
    pub input: Value,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
    #[serde(default)]
    pub artifact_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(default)]
    pub expected_effects: Vec<String>,
    pub ontology_version: String,
    pub contract_version: String,
    pub created_at: String,
    pub recorded_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeDecisionFact {
    pub version: String,
    pub command_id: String,
    pub proposal_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decision_id: Option<String>,
    pub status: String,
    pub decision: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blocking_proposal_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accepted_action_id: Option<String>,
    #[serde(default)]
    pub rejected_reasons: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub human_decision_request: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_query_hint: Option<RuntimeQueryHintFact>,
    pub correlation_id: String,
    #[serde(default)]
    pub would_emit_events: Vec<String>,
    pub recorded_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeAcceptedActionFact {
    pub version: String,
    pub command_id: String,
    pub proposal_id: String,
    pub accepted_action_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issue_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    pub action_type: String,
    pub actor_role: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_object_ref: Option<ActionRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_state: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to_state: Option<String>,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
    #[serde(default)]
    pub artifact_refs: Vec<String>,
    #[serde(default)]
    pub expected_events: Vec<String>,
    pub lock_plan: ObjectLockPlan,
    pub definition_versions: DefinitionVersions,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_type: Option<String>,
    pub recorded_at: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeCommandFactBundle {
    pub command: RuntimeCommandFact,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proposal: Option<RuntimeProposalFact>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decision: Option<RuntimeDecisionFact>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accepted_action: Option<RuntimeAcceptedActionFact>,
}
