use std::collections::{BTreeMap, BTreeSet};

use agentflow_action_contract::ActionContractRegistry;
use agentflow_ontology::OntologyRegistry;

use crate::model::{
    AgentRolePolicy, AgentRolePolicyBundle, ProductAgentRole, RoleActionMatrixEntry,
    RoleEvidenceMatrixEntry, RoleObjectMatrixEntry, RuntimeAgentRole,
};
use crate::report::{RoleCapabilityDecision, RolePolicyValidationReport};
use crate::validation::validate_role_policy_bundle;

#[derive(Debug, Clone)]
pub struct RolePolicyRegistry {
    bundle: AgentRolePolicyBundle,
    roles: BTreeMap<RuntimeAgentRole, AgentRolePolicy>,
}

impl RolePolicyRegistry {
    pub fn load_bundle(
        bundle: AgentRolePolicyBundle,
        ontology_registry: &OntologyRegistry,
        action_registry: &ActionContractRegistry,
    ) -> Result<Self, RolePolicyValidationReport> {
        let report = validate_role_policy_bundle(&bundle, ontology_registry, action_registry);
        if !report.valid {
            return Err(report);
        }
        let roles = bundle
            .roles
            .iter()
            .cloned()
            .map(|policy| (policy.role_id, policy))
            .collect();
        Ok(Self { bundle, roles })
    }

    pub fn bundle(&self) -> &AgentRolePolicyBundle {
        &self.bundle
    }

    pub fn list_core_roles(&self) -> Vec<&AgentRolePolicy> {
        self.roles.values().collect()
    }

    pub fn get_role_policy(&self, role: RuntimeAgentRole) -> Option<&AgentRolePolicy> {
        self.roles.get(&role)
    }

    pub fn resolve_runtime_role(&self, role_or_alias: &str) -> Option<RuntimeAgentRole> {
        if let Some(role) = RuntimeAgentRole::parse_alias(role_or_alias) {
            return Some(role);
        }
        self.bundle
            .compatibility
            .aliases
            .iter()
            .find(|binding| binding.alias == role_or_alias)
            .map(|binding| binding.runtime_role)
    }

    pub fn runtime_role_for_product_role(
        &self,
        product_role: ProductAgentRole,
    ) -> Option<RuntimeAgentRole> {
        self.bundle
            .product_role_bindings
            .iter()
            .find(|binding| binding.product_role == product_role)
            .map(|binding| binding.runtime_role)
    }

    pub fn can_role_propose_action(
        &self,
        role_or_alias: &str,
        action_type: &str,
        object_type: Option<&str>,
    ) -> RoleCapabilityDecision {
        let object_type_owned = object_type.map(str::to_string);
        let Some(runtime_role) = self.resolve_runtime_role(role_or_alias) else {
            return RoleCapabilityDecision::denied(action_type, object_type_owned, "unknownRole");
        };
        let Some(policy) = self.get_role_policy(runtime_role) else {
            return RoleCapabilityDecision::denied(action_type, object_type_owned, "unknownRole");
        };
        if policy.status == crate::model::AgentRolePolicyStatus::Retired {
            return RoleCapabilityDecision::denied(action_type, object_type_owned, "roleRetired");
        }
        if policy.cannot_do.iter().any(|item| item == action_type) {
            return RoleCapabilityDecision {
                allowed: false,
                runtime_role: Some(runtime_role),
                action_type: action_type.to_string(),
                object_type: object_type_owned,
                reason: "actionExplicitlyForbidden".into(),
                requires_handoff: false,
                requires_human_approval: false,
            };
        }

        let Some(capability) = policy
            .action_capabilities
            .iter()
            .find(|capability| capability.action_type == action_type)
        else {
            return RoleCapabilityDecision {
                allowed: false,
                runtime_role: Some(runtime_role),
                action_type: action_type.to_string(),
                object_type: object_type_owned,
                reason: "actionNotAllowedForRole".into(),
                requires_handoff: false,
                requires_human_approval: false,
            };
        };

        if let Some(expected_object_type) = capability.object_type.as_deref() {
            if Some(expected_object_type) != object_type {
                return RoleCapabilityDecision {
                    allowed: false,
                    runtime_role: Some(runtime_role),
                    action_type: action_type.to_string(),
                    object_type: object_type_owned,
                    reason: "objectTypeMismatch".into(),
                    requires_handoff: capability.requires_handoff,
                    requires_human_approval: capability.requires_human_approval,
                };
            }
        }

        if let Some(expected_object_type) = object_type {
            let covered_scope = policy
                .object_scopes
                .iter()
                .any(|scope| scope.object_type == expected_object_type);
            if !covered_scope {
                return RoleCapabilityDecision {
                    allowed: false,
                    runtime_role: Some(runtime_role),
                    action_type: action_type.to_string(),
                    object_type: object_type_owned,
                    reason: "objectScopeMissing".into(),
                    requires_handoff: capability.requires_handoff,
                    requires_human_approval: capability.requires_human_approval,
                };
            }
        }

        if capability.requires_handoff {
            let Some(handoff_rule) = capability.handoff_rule.as_deref() else {
                return RoleCapabilityDecision {
                    allowed: false,
                    runtime_role: Some(runtime_role),
                    action_type: action_type.to_string(),
                    object_type: object_type_owned,
                    reason: "missingHandoffRule".into(),
                    requires_handoff: true,
                    requires_human_approval: capability.requires_human_approval,
                };
            };
            if !policy.handoff_rules.iter().any(|rule| rule == handoff_rule) {
                return RoleCapabilityDecision {
                    allowed: false,
                    runtime_role: Some(runtime_role),
                    action_type: action_type.to_string(),
                    object_type: object_type_owned,
                    reason: "missingHandoffRule".into(),
                    requires_handoff: true,
                    requires_human_approval: capability.requires_human_approval,
                };
            }
        }

        RoleCapabilityDecision {
            allowed: true,
            runtime_role: Some(runtime_role),
            action_type: action_type.to_string(),
            object_type: object_type_owned,
            reason: "allowed".into(),
            requires_handoff: capability.requires_handoff,
            requires_human_approval: capability.requires_human_approval,
        }
    }

    pub fn export_role_action_matrix(
        &self,
        action_registry: &ActionContractRegistry,
    ) -> Vec<RoleActionMatrixEntry> {
        let mut actions = action_registry
            .list_action_types()
            .into_iter()
            .map(|action| {
                let object_type = action
                    .target_object_type
                    .as_deref()
                    .or(action.creates_object_type.as_deref());
                (action.id.clone(), object_type.map(str::to_string))
            })
            .collect::<Vec<_>>();
        actions.sort_by(|left, right| left.0.cmp(&right.0));

        let mut roles = self.roles.keys().copied().collect::<Vec<_>>();
        roles.sort();

        let mut entries = Vec::new();
        for role in roles {
            for (action_type, object_type) in &actions {
                let decision = self.can_role_propose_action(
                    role.as_str(),
                    action_type,
                    object_type.as_deref(),
                );
                entries.push(RoleActionMatrixEntry {
                    runtime_role: role,
                    action_type: action_type.clone(),
                    allowed: decision.allowed,
                    reason: decision.reason,
                });
            }
        }
        entries
    }

    pub fn export_role_object_matrix(
        &self,
        ontology_registry: &OntologyRegistry,
    ) -> Vec<RoleObjectMatrixEntry> {
        let mut object_types = ontology_registry
            .list_object_types()
            .into_iter()
            .map(|item| item.id.clone())
            .collect::<Vec<_>>();
        object_types.sort();

        let mut roles = self.roles.keys().copied().collect::<Vec<_>>();
        roles.sort();

        let mut entries = Vec::new();
        for role in roles {
            let policy = self.roles.get(&role).expect("role must exist");
            for object_type in &object_types {
                let can_propose = policy.action_capabilities.iter().any(|capability| {
                    capability.object_type.as_deref() == Some(object_type.as_str())
                });
                entries.push(RoleObjectMatrixEntry {
                    runtime_role: role,
                    object_type: object_type.clone(),
                    can_read: policy.can_read.iter().any(|item| item == object_type),
                    can_write: policy.can_write.iter().any(|item| item == object_type),
                    can_propose,
                });
            }
        }
        entries
    }

    pub fn export_role_evidence_matrix(&self) -> Vec<RoleEvidenceMatrixEntry> {
        let mut evidence_types = BTreeSet::new();
        for policy in self.roles.values() {
            for evidence_type in &policy.must_produce {
                evidence_types.insert(evidence_type.clone());
            }
            for evidence_type in &policy.required_evidence {
                evidence_types.insert(evidence_type.clone());
            }
        }

        let mut roles = self.roles.keys().copied().collect::<Vec<_>>();
        roles.sort();

        let mut entries = Vec::new();
        for role in roles {
            let policy = self.roles.get(&role).expect("role must exist");
            for evidence_type in &evidence_types {
                let must_produce = policy.must_produce.iter().any(|item| item == evidence_type)
                    || policy
                        .required_evidence
                        .iter()
                        .any(|item| item == evidence_type);
                let can_read = policy
                    .can_read
                    .iter()
                    .any(|item| item == "Evidence" || item == "Artifact");
                let cannot_produce =
                    !must_produce && !policy.can_write.iter().any(|item| item == "Evidence");
                entries.push(RoleEvidenceMatrixEntry {
                    runtime_role: role,
                    evidence_type: evidence_type.clone(),
                    must_produce,
                    can_read,
                    cannot_produce,
                });
            }
        }
        entries
    }
}
