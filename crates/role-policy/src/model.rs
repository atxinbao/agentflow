use serde::{Deserialize, Serialize};

pub const ROLE_POLICY_BUNDLE_VERSION: &str = "agentflow-role-policy-bundle.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProductAgentRole {
    GoalAgent,
    SpecAgent,
    WorkAgent,
    AuditAgent,
    DeliveryAgent,
}

impl ProductAgentRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::GoalAgent => "goal-agent",
            Self::SpecAgent => "spec-agent",
            Self::WorkAgent => "work-agent",
            Self::AuditAgent => "audit-agent",
            Self::DeliveryAgent => "delivery-agent",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuntimeAgentRole {
    GoalAgent,
    SpecAgent,
    WorkAgent,
    AuditAgent,
    DeliveryAgent,
    ReviewAgent,
    CoordinatorAgent,
    HumanOwner,
}

impl RuntimeAgentRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::GoalAgent => "goal-agent",
            Self::SpecAgent => "spec-agent",
            Self::WorkAgent => "work-agent",
            Self::AuditAgent => "audit-agent",
            Self::DeliveryAgent => "delivery-agent",
            Self::ReviewAgent => "review-agent",
            Self::CoordinatorAgent => "coordinator-agent",
            Self::HumanOwner => "human-owner",
        }
    }

    pub fn parse_alias(value: &str) -> Option<Self> {
        match value.trim() {
            "GoalAgent" | "goal-agent" => Some(Self::GoalAgent),
            "SpecAgent" | "spec-agent" => Some(Self::SpecAgent),
            "WorkAgent" | "work-agent" | "BuildAgent" | "build-agent" => Some(Self::WorkAgent),
            "AuditAgent" | "audit-agent" => Some(Self::AuditAgent),
            "DeliveryAgent" | "delivery-agent" => Some(Self::DeliveryAgent),
            "ReviewAgent" | "review-agent" => Some(Self::ReviewAgent),
            "CoordinatorAgent" | "coordinator-agent" => Some(Self::CoordinatorAgent),
            "HumanOwner" | "human-owner" => Some(Self::HumanOwner),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AgentRolePolicyStatus {
    Active,
    Deprecated,
    Retired,
}

impl Default for AgentRolePolicyStatus {
    fn default() -> Self {
        Self::Active
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RoleCapabilityMode {
    Read,
    Propose,
    Execute,
    Review,
    Decide,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ObjectScopeKind {
    AssignedIssue,
    CurrentRun,
    ReferencedEvidence,
    OwnedFinding,
    ApprovedSpec,
    ProjectWideRead,
    HumanDecisionTarget,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ToolKind {
    ReadDocs,
    InspectContext,
    Filesystem,
    LocalBuild,
    LocalTest,
    BrowserSmoke,
    ReadEvidence,
    InspectDiff,
    GenerateReport,
    InspectState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ApprovalGate {
    ContractRequired,
    HumanApprovalRequired,
    IndependentAuditRequired,
    ExplicitAuditRequestRequired,
    HandoffRequired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductRoleBinding {
    pub product_role: ProductAgentRole,
    pub runtime_role: RuntimeAgentRole,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleAliasBinding {
    pub alias: String,
    pub runtime_role: RuntimeAgentRole,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RolePolicyCompatibility {
    #[serde(default)]
    pub aliases: Vec<RoleAliasBinding>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectScope {
    pub scope_id: String,
    pub object_type: String,
    pub scope_kind: ObjectScopeKind,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleToolScope {
    #[serde(default)]
    pub allowed_tool_kinds: Vec<ToolKind>,
    #[serde(default)]
    pub forbidden_tool_kinds: Vec<ToolKind>,
    #[serde(default)]
    pub requires_evidence_capture: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HandoffRule {
    pub handoff_id: String,
    pub from_role: RuntimeAgentRole,
    pub to_role: RuntimeAgentRole,
    pub target_object_type: String,
    #[serde(default)]
    pub allowed_actions: Vec<String>,
    #[serde(default)]
    pub required_inputs: Vec<String>,
    #[serde(default)]
    pub expected_outputs: Vec<String>,
    #[serde(default)]
    pub boundary_notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleActionCapability {
    pub action_type: String,
    pub mode: RoleCapabilityMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope_kind: Option<ObjectScopeKind>,
    #[serde(default)]
    pub requires_handoff: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub handoff_rule: Option<String>,
    #[serde(default)]
    pub requires_human_approval: bool,
    #[serde(default)]
    pub required_evidence: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentRolePolicy {
    pub role_id: RuntimeAgentRole,
    pub name: String,
    pub description: String,
    pub status: AgentRolePolicyStatus,
    #[serde(default)]
    pub can_read: Vec<String>,
    #[serde(default)]
    pub can_write: Vec<String>,
    #[serde(default)]
    pub action_capabilities: Vec<RoleActionCapability>,
    #[serde(default)]
    pub must_produce: Vec<String>,
    #[serde(default)]
    pub cannot_do: Vec<String>,
    #[serde(default)]
    pub object_scopes: Vec<ObjectScope>,
    pub tool_scope: RoleToolScope,
    #[serde(default)]
    pub handoff_rules: Vec<String>,
    #[serde(default)]
    pub approval_gates: Vec<ApprovalGate>,
    #[serde(default)]
    pub required_evidence: Vec<String>,
    #[serde(default)]
    pub boundary_notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentRolePolicyBundle {
    pub version: String,
    pub bundle_id: String,
    pub namespace: String,
    pub definition_version: String,
    pub status: AgentRolePolicyStatus,
    #[serde(default)]
    pub product_role_bindings: Vec<ProductRoleBinding>,
    #[serde(default)]
    pub roles: Vec<AgentRolePolicy>,
    #[serde(default)]
    pub handoff_rules: Vec<HandoffRule>,
    pub compatibility: RolePolicyCompatibility,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleActionMatrixEntry {
    pub runtime_role: RuntimeAgentRole,
    pub action_type: String,
    pub allowed: bool,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleObjectMatrixEntry {
    pub runtime_role: RuntimeAgentRole,
    pub object_type: String,
    pub can_read: bool,
    pub can_write: bool,
    pub can_propose: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleEvidenceMatrixEntry {
    pub runtime_role: RuntimeAgentRole,
    pub evidence_type: String,
    pub must_produce: bool,
    pub can_read: bool,
    pub cannot_produce: bool,
}
