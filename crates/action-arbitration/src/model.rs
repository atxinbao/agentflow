use std::collections::BTreeMap;

use agentflow_action_contract::{ActionContractRegistry, ActionProposal, ActionRef};
use agentflow_object_state::ObjectStateMachineRegistry;
use agentflow_ontology::OntologyRegistry;
use agentflow_role_policy::RolePolicyRegistry;
use serde::{Deserialize, Serialize};

pub const ACTION_ARBITRATION_VERSION: &str = "agentflow-action-arbitration.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DefinitionVersions {
    pub ontology_version: String,
    pub contract_version: String,
    pub role_policy_version: String,
    pub object_state_version: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArbitrationRequest {
    pub request_id: String,
    pub proposal: ActionProposal,
    pub definition_versions: DefinitionVersions,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ObjectRefKey {
    pub object_type: String,
    pub object_id: String,
}

impl ObjectRefKey {
    pub fn new(object_type: impl Into<String>, object_id: impl Into<String>) -> Self {
        Self {
            object_type: object_type.into(),
            object_id: object_id.into(),
        }
    }

    pub fn from_action_ref(target: &ActionRef) -> Self {
        Self::new(target.object_type.clone(), target.id.clone())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StateFact {
    pub object_type: String,
    pub object_id: String,
    pub state_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceFact {
    pub evidence_ref: String,
    pub evidence_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DependencyFact {
    pub dependency_key: String,
    pub satisfied: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ObjectLockKind {
    Write,
    RunExecution,
    AuditReview,
    DecisionPending,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectLock {
    pub lock_id: String,
    pub object_type: String,
    pub object_id: String,
    pub lock_kind: ObjectLockKind,
    pub owner_proposal_id: String,
    pub owner_role: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectLockPlan {
    #[serde(default)]
    pub acquire: Vec<ObjectLock>,
    #[serde(default)]
    pub release: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum HumanDecisionResponseKind {
    ApprovalRequired,
    ConfirmationRequired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HumanDecisionRequest {
    pub decision_kind: HumanDecisionResponseKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_object_ref: Option<ActionRef>,
    pub question: String,
    #[serde(default)]
    pub allowed_responses: Vec<String>,
    pub required_evidence_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcceptedAction {
    pub accepted_action_id: String,
    pub proposal_id: String,
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ArbitrationDecisionStatus {
    Accepted,
    Rejected,
    HumanDecisionRequired,
    Queued,
    ConflictDetected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArbitrationDecision {
    pub decision_id: String,
    pub request_id: String,
    pub proposal_id: String,
    pub status: ArbitrationDecisionStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accepted_action: Option<AcceptedAction>,
    #[serde(default)]
    pub rejected_reasons: Vec<crate::reasons::RejectionReason>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required_human_decision: Option<HumanDecisionRequest>,
    pub lock_plan: ObjectLockPlan,
    #[serde(default)]
    pub would_emit_events: Vec<String>,
    pub created_at: String,
}

impl ArbitrationDecision {
    pub fn rejected(
        request: &ArbitrationRequest,
        reasons: Vec<crate::reasons::RejectionReason>,
    ) -> Self {
        Self {
            decision_id: format!("decision-{}", request.request_id),
            request_id: request.request_id.clone(),
            proposal_id: request.proposal.proposal_id.clone(),
            status: ArbitrationDecisionStatus::Rejected,
            accepted_action: None,
            rejected_reasons: reasons,
            required_human_decision: None,
            lock_plan: ObjectLockPlan::default(),
            would_emit_events: Vec::new(),
            created_at: request.requested_at.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ArbitrationContext {
    pub ontology_registry: OntologyRegistry,
    pub action_contract_registry: ActionContractRegistry,
    pub role_policy_registry: RolePolicyRegistry,
    pub state_machine_registry: ObjectStateMachineRegistry,
    pub current_object_states: BTreeMap<ObjectRefKey, String>,
    pub evidence_index: BTreeMap<String, EvidenceFact>,
    pub dependency_index: BTreeMap<String, DependencyFact>,
    pub object_locks: Vec<ObjectLock>,
}

impl ArbitrationContext {
    pub fn new(
        ontology_registry: OntologyRegistry,
        action_contract_registry: ActionContractRegistry,
        role_policy_registry: RolePolicyRegistry,
        state_machine_registry: ObjectStateMachineRegistry,
    ) -> Self {
        Self {
            ontology_registry,
            action_contract_registry,
            role_policy_registry,
            state_machine_registry,
            current_object_states: BTreeMap::new(),
            evidence_index: BTreeMap::new(),
            dependency_index: BTreeMap::new(),
            object_locks: Vec::new(),
        }
    }

    pub fn insert_state(&mut self, fact: StateFact) {
        self.current_object_states.insert(
            ObjectRefKey::new(fact.object_type, fact.object_id),
            fact.state_id,
        );
    }

    pub fn insert_evidence(&mut self, fact: EvidenceFact) {
        self.evidence_index.insert(fact.evidence_ref.clone(), fact);
    }

    pub fn insert_dependency(&mut self, fact: DependencyFact) {
        self.dependency_index
            .insert(fact.dependency_key.clone(), fact);
    }

    pub fn push_lock(&mut self, lock: ObjectLock) {
        self.object_locks.push(lock);
    }

    pub fn current_state_for(&self, target: &ActionRef) -> Option<&str> {
        let key = ObjectRefKey::from_action_ref(target);
        self.current_object_states.get(&key).map(String::as_str)
    }

    pub fn evidence_by_ref(&self, evidence_ref: &str) -> Option<&EvidenceFact> {
        self.evidence_index.get(evidence_ref)
    }

    pub fn evidence_count_by_type(&self, evidence_refs: &[String], evidence_type: &str) -> usize {
        evidence_refs
            .iter()
            .filter_map(|evidence_ref| self.evidence_index.get(evidence_ref))
            .filter(|fact| fact.evidence_type == evidence_type)
            .count()
    }

    pub fn dependency_satisfied(&self, dependency_key: &str) -> bool {
        self.dependency_index
            .get(dependency_key)
            .map(|fact| fact.satisfied)
            .unwrap_or(false)
    }
}
