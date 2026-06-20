use serde::{Deserialize, Serialize};

use crate::errors::RuntimeCommandError;
use crate::mapping::RuntimeQueryHint;

pub const RUNTIME_COMMAND_API_VERSION: &str = "agentflow-runtime-command-api.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RuntimeCommandStatus {
    Accepted,
    Rejected,
    HumanDecisionRequired,
    InvalidCommand,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RuntimeCommandDecision {
    Accepted,
    Rejected,
    HumanDecisionRequired,
    InvalidCommand,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeCommandValidationReport {
    pub command_id: String,
    pub command_type: String,
    pub valid: bool,
    #[serde(default)]
    pub errors: Vec<RuntimeCommandError>,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub normalized_action_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeHumanDecisionRequest {
    pub question: String,
    #[serde(default)]
    pub allowed_responses: Vec<String>,
    pub required_evidence_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeCommandResponse {
    pub version: String,
    pub command_id: String,
    pub proposal_id: String,
    pub status: RuntimeCommandStatus,
    pub decision: RuntimeCommandDecision,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accepted_action_id: Option<String>,
    #[serde(default)]
    pub rejected_reasons: Vec<RuntimeCommandError>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub human_decision_request: Option<RuntimeHumanDecisionRequest>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_query_hint: Option<RuntimeQueryHint>,
    pub correlation_id: String,
}
