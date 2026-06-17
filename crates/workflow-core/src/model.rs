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
    #[serde(default)]
    pub role: Option<WorkflowAgentRole>,
    #[serde(default)]
    pub skill_pack: Option<WorkflowSkillPack>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WorkflowStatePhase {
    Future,
    Current,
    Past,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WorkflowAgentRole {
    GoalAgent,
    SpecAgent,
    WorkAgent,
    AuditAgent,
    DeliveryAgent,
    Specialist,
    System,
}

impl WorkflowAgentRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::GoalAgent => "goal-agent",
            Self::SpecAgent => "spec-agent",
            Self::WorkAgent => "work-agent",
            Self::AuditAgent => "audit-agent",
            Self::DeliveryAgent => "delivery-agent",
            Self::Specialist => "specialist",
            Self::System => "system",
        }
    }

    pub fn parse_alias(value: &str) -> Option<Self> {
        match value.trim() {
            "goal-agent" => Some(Self::GoalAgent),
            "spec-agent" => Some(Self::SpecAgent),
            "work-agent" | "build-agent" => Some(Self::WorkAgent),
            "audit-agent" => Some(Self::AuditAgent),
            "delivery-agent" => Some(Self::DeliveryAgent),
            "specialist" => Some(Self::Specialist),
            "system" => Some(Self::System),
            _ => None,
        }
    }

    pub fn default_skill_pack(&self) -> Option<WorkflowSkillPack> {
        match self {
            Self::GoalAgent => Some(WorkflowSkillPack::BrainSkills),
            Self::SpecAgent => Some(WorkflowSkillPack::ContractSkills),
            Self::WorkAgent => Some(WorkflowSkillPack::ExecutionSkills),
            Self::AuditAgent => Some(WorkflowSkillPack::JudgmentSkills),
            Self::DeliveryAgent => Some(WorkflowSkillPack::DeliverySkills),
            Self::Specialist => Some(WorkflowSkillPack::SpecialistSkills),
            Self::System => None,
        }
    }

    pub fn provider_role_alias(&self) -> Option<&'static str> {
        match self {
            Self::WorkAgent => Some("build-agent"),
            Self::AuditAgent => Some("audit-agent"),
            Self::SpecAgent => Some("spec-agent"),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WorkflowSkillPack {
    BrainSkills,
    ContractSkills,
    ExecutionSkills,
    JudgmentSkills,
    DeliverySkills,
    SpecialistSkills,
}

impl WorkflowSkillPack {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::BrainSkills => "brain-skills",
            Self::ContractSkills => "contract-skills",
            Self::ExecutionSkills => "execution-skills",
            Self::JudgmentSkills => "judgment-skills",
            Self::DeliverySkills => "delivery-skills",
            Self::SpecialistSkills => "specialist-skills",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WorkflowHandoffMode {
    OwnershipTransfer,
    BoundedCapabilityCall,
}

impl WorkflowHandoffMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OwnershipTransfer => "ownership-transfer",
            Self::BoundedCapabilityCall => "bounded-capability-call",
        }
    }
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
    #[serde(default)]
    pub handoff: Option<HandoffDefinition>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HandoffDefinition {
    pub from_role: WorkflowAgentRole,
    pub to_role: WorkflowAgentRole,
    pub mode: WorkflowHandoffMode,
    pub payload_ref: String,
    #[serde(default)]
    pub expected_state: Option<String>,
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
