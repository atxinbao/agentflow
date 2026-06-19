use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const ACTION_CONTRACT_BUNDLE_VERSION: &str = "agentflow-action-contract-bundle.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ActionDefinitionStatus {
    Draft,
    Active,
    Deprecated,
    Retired,
}

impl Default for ActionDefinitionStatus {
    fn default() -> Self {
        Self::Draft
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ActionCategory {
    Intake,
    Spec,
    Planning,
    Execution,
    Evidence,
    Delivery,
    Audit,
    Finding,
    Decision,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ActionTargetMode {
    ExistingObject,
    CreateObject,
    LinkObjects,
    RecordDecision,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ActionFieldValueType {
    String,
    Number,
    Boolean,
    ObjectRef,
    ObjectRefList,
    EvidenceRef,
    EvidenceRefList,
    ArtifactRef,
    ArtifactRefList,
    Timestamp,
    Enum,
    StructuredObject,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ActionPreconditionKind {
    TargetExists,
    TargetStateIs,
    LinkExists,
    LinkAbsent,
    DependencySatisfied,
    EvidenceExists,
    HumanDecisionExists,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ActionEffectKind {
    CreateObject,
    ChangeState,
    AttachEvidence,
    AttachArtifact,
    CreateLink,
    RecordDecision,
    EmitEvent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AcceptedRefKind {
    EvidenceRef,
    ArtifactRef,
    DecisionRef,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ActionSourceSurface {
    Conversation,
    Desktop,
    Cli,
    Sdk,
    Agent,
    System,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionFieldDefinition {
    pub name: String,
    pub value_type: ActionFieldValueType,
    #[serde(default)]
    pub required: bool,
    pub description: String,
    #[serde(default)]
    pub enum_values: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object_type_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub link_type_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionInputSchema {
    #[serde(default)]
    pub fields: Vec<ActionFieldDefinition>,
    #[serde(default)]
    pub required_fields: Vec<String>,
    #[serde(default)]
    pub allow_additional_fields: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionPrecondition {
    pub id: String,
    pub kind: ActionPreconditionKind,
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expression: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required_state: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required_link: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required_evidence_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionEffect {
    pub id: String,
    pub kind: ActionEffectKind,
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub link_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state_transition_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequiredEvidenceDefinition {
    pub evidence_type: String,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub min_count: usize,
    pub accepted_ref_kind: AcceptedRefKind,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionExpectedEvent {
    pub event_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object_type: Option<String>,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub payload_fields: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionApprovalHint {
    #[serde(default)]
    pub human_approval_required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionSimulationHint {
    #[serde(default)]
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionIdempotencyPolicy {
    #[serde(default)]
    pub required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionTypeDefinition {
    pub id: String,
    pub namespace: String,
    pub version: String,
    pub status: ActionDefinitionStatus,
    pub name: String,
    pub description: String,
    pub category: ActionCategory,
    pub target_mode: ActionTargetMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_object_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub creates_object_type: Option<String>,
    pub contract_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionContract {
    pub id: String,
    pub action_type: String,
    pub namespace: String,
    pub version: String,
    pub status: ActionDefinitionStatus,
    pub target_mode: ActionTargetMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_object_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub creates_object_type: Option<String>,
    pub input_schema: ActionInputSchema,
    #[serde(default)]
    pub preconditions: Vec<ActionPrecondition>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state_transition_ref: Option<String>,
    #[serde(default)]
    pub effects: Vec<ActionEffect>,
    #[serde(default)]
    pub required_evidence: Vec<RequiredEvidenceDefinition>,
    #[serde(default)]
    pub expected_events: Vec<ActionExpectedEvent>,
    #[serde(default)]
    pub expected_links: Vec<String>,
    pub idempotency: ActionIdempotencyPolicy,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conflict_scope_hint: Option<String>,
    pub approval_hint: ActionApprovalHint,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rollback_hint: Option<String>,
    pub simulation_hint: ActionSimulationHint,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionRef {
    pub object_type: String,
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionProposal {
    pub proposal_id: String,
    pub idempotency_key: String,
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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionContractBundle {
    pub version: String,
    pub registry_id: String,
    pub namespace: String,
    pub definition_version: String,
    pub status: ActionDefinitionStatus,
    #[serde(default)]
    pub action_types: Vec<ActionTypeDefinition>,
    #[serde(default)]
    pub contracts: Vec<ActionContract>,
}
