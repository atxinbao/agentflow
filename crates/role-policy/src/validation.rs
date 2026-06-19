use std::collections::BTreeSet;

use agentflow_action_contract::ActionContractRegistry;
use agentflow_ontology::OntologyRegistry;

use crate::model::{AgentRolePolicyBundle, ProductAgentRole, RuntimeAgentRole};
use crate::report::RolePolicyValidationReport;

pub fn validate_role_policy_bundle(
    bundle: &AgentRolePolicyBundle,
    ontology_registry: &OntologyRegistry,
    action_registry: &ActionContractRegistry,
) -> RolePolicyValidationReport {
    let mut report = RolePolicyValidationReport::success();

    if bundle.bundle_id.trim().is_empty() {
        report.push_error(
            "bundle-id-missing",
            "bundleId must not be empty",
            Some("bundleId".into()),
        );
    }
    if bundle.namespace.trim().is_empty() {
        report.push_error(
            "namespace-missing",
            "namespace must not be empty",
            Some("namespace".into()),
        );
    }
    if bundle.definition_version.trim().is_empty() {
        report.push_error(
            "version-missing",
            "definitionVersion must not be empty",
            Some("definitionVersion".into()),
        );
    }

    let mut product_role_seen = BTreeSet::new();
    let mut runtime_role_seen = BTreeSet::new();
    for (index, binding) in bundle.product_role_bindings.iter().enumerate() {
        if !product_role_seen.insert(binding.product_role) {
            report.push_error(
                "duplicate-product-role-binding",
                format!(
                    "product role `{}` is bound more than once",
                    binding.product_role.as_str()
                ),
                Some(format!("productRoleBindings[{index}].productRole")),
            );
        }
        if !binding_exists_for_role(bundle, binding.runtime_role) {
            report.push_error(
                "unknown-runtime-role",
                format!(
                    "product role `{}` points to missing runtime role `{}`",
                    binding.product_role.as_str(),
                    binding.runtime_role.as_str()
                ),
                Some(format!("productRoleBindings[{index}].runtimeRole")),
            );
        }
    }

    let known_action_types = action_registry
        .list_action_types()
        .into_iter()
        .map(|item| item.id.as_str())
        .collect::<BTreeSet<_>>();

    let known_object_types = ontology_registry
        .list_object_types()
        .into_iter()
        .map(|item| item.id.as_str())
        .collect::<BTreeSet<_>>();

    let handoff_ids = bundle
        .handoff_rules
        .iter()
        .map(|rule| rule.handoff_id.as_str())
        .collect::<BTreeSet<_>>();

    for (index, role) in bundle.roles.iter().enumerate() {
        if !runtime_role_seen.insert(role.role_id) {
            report.push_error(
                "duplicate-runtime-role",
                format!(
                    "runtime role `{}` is defined more than once",
                    role.role_id.as_str()
                ),
                Some(format!("roles[{index}].roleId")),
            );
        }
        if role.name.trim().is_empty() {
            report.push_error(
                "role-name-missing",
                format!("role `{}` must have non-empty name", role.role_id.as_str()),
                Some(format!("roles[{index}].name")),
            );
        }

        for object_type in &role.can_read {
            if !known_object_types.contains(object_type.as_str()) {
                report.push_error(
                    "unknown-object-type",
                    format!(
                        "role `{}` canRead unknown object type `{object_type}`",
                        role.role_id.as_str()
                    ),
                    Some(format!("roles[{index}].canRead")),
                );
            }
        }
        for object_type in &role.can_write {
            if !known_object_types.contains(object_type.as_str()) {
                report.push_error(
                    "unknown-object-type",
                    format!(
                        "role `{}` canWrite unknown object type `{object_type}`",
                        role.role_id.as_str()
                    ),
                    Some(format!("roles[{index}].canWrite")),
                );
            }
        }
        for (scope_index, scope) in role.object_scopes.iter().enumerate() {
            if !known_object_types.contains(scope.object_type.as_str()) {
                report.push_error(
                    "unknown-object-type",
                    format!(
                        "role `{}` object scope `{}` references unknown object type `{}`",
                        role.role_id.as_str(),
                        scope.scope_id,
                        scope.object_type
                    ),
                    Some(format!(
                        "roles[{index}].objectScopes[{scope_index}].objectType"
                    )),
                );
            }
        }
        for action_type in &role.cannot_do {
            if !known_action_types.contains(action_type.as_str()) {
                report.push_error(
                    "unknown-action-type",
                    format!(
                        "role `{}` cannotDo unknown action type `{action_type}`",
                        role.role_id.as_str()
                    ),
                    Some(format!("roles[{index}].cannotDo")),
                );
            }
        }
        for (capability_index, capability) in role.action_capabilities.iter().enumerate() {
            if !known_action_types.contains(capability.action_type.as_str()) {
                report.push_error(
                    "unknown-action-type",
                    format!(
                        "role `{}` references unknown action type `{}`",
                        role.role_id.as_str(),
                        capability.action_type
                    ),
                    Some(format!(
                        "roles[{index}].actionCapabilities[{capability_index}].actionType"
                    )),
                );
            }
            if let Some(object_type) = capability.object_type.as_deref() {
                if !known_object_types.contains(object_type) {
                    report.push_error(
                        "unknown-object-type",
                        format!(
                            "role `{}` capability `{}` references unknown object type `{object_type}`",
                            role.role_id.as_str(),
                            capability.action_type
                        ),
                        Some(format!(
                            "roles[{index}].actionCapabilities[{capability_index}].objectType"
                        )),
                    );
                }
            }
            if capability.requires_handoff {
                let Some(handoff_rule) = capability.handoff_rule.as_deref() else {
                    report.push_error(
                        "missing-handoff-rule",
                        format!(
                            "role `{}` action `{}` requires handoff but handoffRule is missing",
                            role.role_id.as_str(),
                            capability.action_type
                        ),
                        Some(format!(
                            "roles[{index}].actionCapabilities[{capability_index}].handoffRule"
                        )),
                    );
                    continue;
                };
                if !handoff_ids.contains(handoff_rule)
                    || !role.handoff_rules.iter().any(|rule| rule == handoff_rule)
                {
                    report.push_error(
                        "missing-handoff-rule",
                        format!(
                            "role `{}` action `{}` requires unknown or unbound handoff rule `{handoff_rule}`",
                            role.role_id.as_str(),
                            capability.action_type
                        ),
                        Some(format!(
                            "roles[{index}].actionCapabilities[{capability_index}].handoffRule"
                        )),
                    );
                }
            }
        }
    }

    for (index, alias) in bundle.compatibility.aliases.iter().enumerate() {
        if alias.alias.trim().is_empty() {
            report.push_error(
                "alias-missing",
                "compatibility alias must not be empty",
                Some(format!("compatibility.aliases[{index}].alias")),
            );
        }
        if !binding_exists_for_role(bundle, alias.runtime_role) {
            report.push_error(
                "unknown-runtime-role",
                format!(
                    "compatibility alias `{}` points to missing runtime role `{}`",
                    alias.alias,
                    alias.runtime_role.as_str()
                ),
                Some(format!("compatibility.aliases[{index}].runtimeRole")),
            );
        }
    }

    if !bundle.compatibility.aliases.iter().any(|alias| {
        alias.runtime_role == RuntimeAgentRole::WorkAgent && alias.alias == "build-agent"
    }) {
        report.push_error(
            "missing-build-agent-alias",
            "compatibility aliases must preserve build-agent -> work-agent",
            Some("compatibility.aliases".into()),
        );
    }

    ensure_product_role_binding(bundle, ProductAgentRole::GoalAgent, &mut report);
    ensure_product_role_binding(bundle, ProductAgentRole::SpecAgent, &mut report);
    ensure_product_role_binding(bundle, ProductAgentRole::WorkAgent, &mut report);
    ensure_product_role_binding(bundle, ProductAgentRole::AuditAgent, &mut report);
    ensure_product_role_binding(bundle, ProductAgentRole::DeliveryAgent, &mut report);

    report
}

fn binding_exists_for_role(bundle: &AgentRolePolicyBundle, runtime_role: RuntimeAgentRole) -> bool {
    bundle.roles.iter().any(|role| role.role_id == runtime_role)
}

fn ensure_product_role_binding(
    bundle: &AgentRolePolicyBundle,
    product_role: ProductAgentRole,
    report: &mut RolePolicyValidationReport,
) {
    if !bundle
        .product_role_bindings
        .iter()
        .any(|binding| binding.product_role == product_role)
    {
        report.push_error(
            "missing-product-role-binding",
            format!(
                "missing binding for product role `{}`",
                product_role.as_str()
            ),
            Some("productRoleBindings".into()),
        );
    }
}

#[cfg(test)]
mod tests {
    use agentflow_action_contract::core_action_contract_registry;
    use agentflow_ontology::core_ontology_registry;

    use crate::core::core_role_policy_bundle;
    use crate::model::{RoleActionCapability, RuntimeAgentRole};
    use crate::registry::RolePolicyRegistry;

    use super::validate_role_policy_bundle;

    fn registries() -> (
        agentflow_ontology::OntologyRegistry,
        agentflow_action_contract::ActionContractRegistry,
    ) {
        let ontology = core_ontology_registry();
        let action_registry = core_action_contract_registry(&ontology);
        (ontology, action_registry)
    }

    fn registry() -> RolePolicyRegistry {
        let (ontology, action_registry) = registries();
        RolePolicyRegistry::load_bundle(core_role_policy_bundle(), &ontology, &action_registry)
            .unwrap()
    }

    #[test]
    fn core_role_policy_bundle_validates() {
        let (ontology, action_registry) = registries();
        let report =
            validate_role_policy_bundle(&core_role_policy_bundle(), &ontology, &action_registry);
        assert!(report.valid, "{:?}", report.errors);
    }

    #[test]
    fn unknown_role_fails() {
        let registry = registry();
        let decision = registry.can_role_propose_action("ghost-agent", "startRun", Some("Issue"));
        assert!(!decision.allowed);
        assert_eq!(decision.reason, "unknownRole");
    }

    #[test]
    fn role_references_unknown_action_type_fails() {
        let (ontology, action_registry) = registries();
        let mut bundle = core_role_policy_bundle();
        bundle.roles[0]
            .action_capabilities
            .push(RoleActionCapability {
                action_type: "doesNotExist".into(),
                mode: crate::model::RoleCapabilityMode::Execute,
                object_type: Some("Issue".into()),
                scope_kind: Some(crate::model::ObjectScopeKind::AssignedIssue),
                requires_handoff: false,
                handoff_rule: None,
                requires_human_approval: false,
                required_evidence: vec![],
            });
        let report = validate_role_policy_bundle(&bundle, &ontology, &action_registry);
        assert!(report
            .errors
            .iter()
            .any(|error| error.code == "unknown-action-type"));
    }

    #[test]
    fn role_references_unknown_object_type_fails() {
        let (ontology, action_registry) = registries();
        let mut bundle = core_role_policy_bundle();
        bundle.roles[0].can_read.push("Ghost".into());
        let report = validate_role_policy_bundle(&bundle, &ontology, &action_registry);
        assert!(report
            .errors
            .iter()
            .any(|error| error.code == "unknown-object-type"));
    }

    #[test]
    fn build_agent_cannot_draft_spec() {
        let registry = registry();
        let decision =
            registry.can_role_propose_action("build-agent", "draftSpec", Some("Requirement"));
        assert!(!decision.allowed);
        assert_eq!(decision.reason, "actionExplicitlyForbidden");
    }

    #[test]
    fn build_agent_cannot_approve_spec() {
        let registry = registry();
        let decision = registry.can_role_propose_action("build-agent", "approveSpec", Some("Spec"));
        assert!(!decision.allowed);
        assert_eq!(decision.reason, "actionExplicitlyForbidden");
    }

    #[test]
    fn build_agent_cannot_request_audit() {
        let registry = registry();
        let decision =
            registry.can_role_propose_action("build-agent", "requestAudit", Some("Issue"));
        assert!(!decision.allowed);
        assert_eq!(decision.reason, "actionExplicitlyForbidden");
    }

    #[test]
    fn build_agent_cannot_create_finding() {
        let registry = registry();
        let decision =
            registry.can_role_propose_action("BuildAgent", "createFinding", Some("Audit"));
        assert!(!decision.allowed);
        assert_eq!(decision.reason, "actionExplicitlyForbidden");
    }

    #[test]
    fn audit_agent_cannot_mark_issue_done() {
        let registry = registry();
        let decision =
            registry.can_role_propose_action("audit-agent", "markIssueDone", Some("Issue"));
        assert!(!decision.allowed);
        assert_eq!(decision.reason, "actionExplicitlyForbidden");
    }

    #[test]
    fn audit_agent_cannot_rewrite_build_evidence() {
        let registry = registry();
        let policy = registry
            .get_role_policy(RuntimeAgentRole::AuditAgent)
            .unwrap();
        assert!(!policy.can_write.iter().any(|item| item == "Issue"));
        assert!(!policy.can_write.iter().any(|item| item == "Run"));
        assert!(policy.can_read.iter().any(|item| item == "Evidence"));
    }

    #[test]
    fn human_owner_can_approve_spec() {
        let registry = registry();
        let decision = registry.can_role_propose_action("human-owner", "approveSpec", Some("Spec"));
        assert!(decision.allowed);
        assert!(decision.requires_human_approval);
    }

    #[test]
    fn human_owner_can_request_audit() {
        let registry = registry();
        let decision =
            registry.can_role_propose_action("HumanOwner", "requestAudit", Some("Issue"));
        assert!(decision.allowed);
        assert!(decision.requires_human_approval);
    }

    #[test]
    fn prompt_cannot_override_cannot_do() {
        let registry = registry();
        let decision =
            registry.can_role_propose_action("BuildAgent", "createFinding", Some("Audit"));
        assert!(!decision.allowed);
        assert_eq!(decision.reason, "actionExplicitlyForbidden");
    }

    #[test]
    fn missing_handoff_rule_fails_when_required() {
        let (ontology, action_registry) = registries();
        let mut bundle = core_role_policy_bundle();
        let work_agent = bundle
            .roles
            .iter_mut()
            .find(|role| role.role_id == RuntimeAgentRole::WorkAgent)
            .unwrap();
        work_agent.action_capabilities[0].requires_handoff = true;
        work_agent.action_capabilities[0].handoff_rule = Some("missing-rule".into());
        let report = validate_role_policy_bundle(&bundle, &ontology, &action_registry);
        assert!(report
            .errors
            .iter()
            .any(|error| error.code == "missing-handoff-rule"));
    }

    #[test]
    fn role_action_matrix_exports_deterministically() {
        let (ontology, action_registry) = registries();
        let registry =
            RolePolicyRegistry::load_bundle(core_role_policy_bundle(), &ontology, &action_registry)
                .unwrap();
        let left = registry.export_role_action_matrix(&action_registry);
        let right = registry.export_role_action_matrix(&action_registry);
        assert_eq!(left, right);
        assert!(!left.is_empty());
        assert_eq!(left[0].runtime_role, RuntimeAgentRole::GoalAgent);
    }
}
