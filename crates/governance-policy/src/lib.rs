//! Runtime governance policy evaluation.
//!
//! This crate evaluates runtime admission before work enters provider execution.
//! It is read-only: it does not append events, write authority files, mutate
//! runtime state, launch providers, or change audit sidecar state.

use agentflow_capability_registry::{
    evaluate_command, CapabilityPolicy, CapabilityRegistry, CommandSurfaceDecision, WorkerHealth,
};
use agentflow_role_policy::{RoleCapabilityDecision, RolePolicyRegistry};
use serde::{Deserialize, Serialize};

pub const GOVERNANCE_POLICY_REPORT_VERSION: &str = "agentflow-governance-policy-report.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GovernanceDecision {
    Allowed,
    Rejected,
    Deferred,
}

impl GovernanceDecision {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Allowed => "allowed",
            Self::Rejected => "rejected",
            Self::Deferred => "deferred",
        }
    }

    fn merge(self, next: Self) -> Self {
        match (self, next) {
            (Self::Rejected, _) | (_, Self::Rejected) => Self::Rejected,
            (Self::Deferred, _) | (_, Self::Deferred) => Self::Deferred,
            _ => Self::Allowed,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AuditSidecarMode {
    Independent,
    NotRequested,
    BoundToMainChain,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GovernancePolicyRequest {
    pub actor_role: String,
    pub action_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object_type: Option<String>,
    pub worker_id: String,
    pub command: String,
    pub audit_sidecar_mode: AuditSidecarMode,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GovernanceTraceEntry {
    pub stage: String,
    pub decision: GovernanceDecision,
    pub reason: String,
    #[serde(default)]
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GovernanceRoleDecision {
    pub decision: GovernanceDecision,
    pub allowed: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime_role: Option<String>,
    pub reason: String,
    #[serde(default)]
    pub requires_handoff: bool,
    #[serde(default)]
    pub requires_human_approval: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GovernanceCapabilityDecision {
    pub decision: GovernanceDecision,
    pub worker_id: String,
    pub command: String,
    pub enabled: bool,
    pub health: WorkerHealth,
    #[serde(default)]
    pub required_capabilities: Vec<String>,
    #[serde(default)]
    pub missing_capabilities: Vec<String>,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GovernanceAuditSidecarDecision {
    pub decision: GovernanceDecision,
    pub mode: AuditSidecarMode,
    pub independent: bool,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GovernancePolicyReport {
    pub version: String,
    pub decision: GovernanceDecision,
    pub runtime_admission: bool,
    pub request: GovernancePolicyRequest,
    pub role_policy: GovernanceRoleDecision,
    pub capability_policy: GovernanceCapabilityDecision,
    pub audit_sidecar_policy: GovernanceAuditSidecarDecision,
    #[serde(default)]
    pub trace: Vec<GovernanceTraceEntry>,
}

pub fn evaluate_runtime_governance(
    role_registry: &RolePolicyRegistry,
    capability_registry: &CapabilityRegistry,
    request: GovernancePolicyRequest,
) -> GovernancePolicyReport {
    let role_raw = role_registry.can_role_propose_action(
        &request.actor_role,
        &request.action_type,
        request.object_type.as_deref(),
    );
    let role_policy = role_decision(role_raw);
    let capability_raw =
        evaluate_command(capability_registry, &request.worker_id, &request.command);
    let capability_policy = capability_decision(capability_registry, capability_raw);
    let audit_sidecar_policy = audit_sidecar_decision(request.audit_sidecar_mode);

    let decision = role_policy
        .decision
        .merge(capability_policy.decision)
        .merge(audit_sidecar_policy.decision);
    let runtime_admission = decision == GovernanceDecision::Allowed;
    let trace = vec![
        GovernanceTraceEntry {
            stage: "role-policy".to_string(),
            decision: role_policy.decision,
            reason: role_policy.reason.clone(),
            evidence: vec!["role-policy-bundle".to_string()],
        },
        GovernanceTraceEntry {
            stage: "capability-policy".to_string(),
            decision: capability_policy.decision,
            reason: capability_policy.reason.clone(),
            evidence: vec!["capability-registry".to_string()],
        },
        GovernanceTraceEntry {
            stage: "audit-sidecar-policy".to_string(),
            decision: audit_sidecar_policy.decision,
            reason: audit_sidecar_policy.reason.clone(),
            evidence: vec!["audit-sidecar-boundary".to_string()],
        },
    ];

    GovernancePolicyReport {
        version: GOVERNANCE_POLICY_REPORT_VERSION.to_string(),
        decision,
        runtime_admission,
        request,
        role_policy,
        capability_policy,
        audit_sidecar_policy,
        trace,
    }
}

fn role_decision(decision: RoleCapabilityDecision) -> GovernanceRoleDecision {
    let outcome = if decision.allowed {
        GovernanceDecision::Allowed
    } else if decision.reason == "missingHandoffRule" {
        GovernanceDecision::Deferred
    } else {
        GovernanceDecision::Rejected
    };
    GovernanceRoleDecision {
        decision: outcome,
        allowed: decision.allowed,
        runtime_role: decision.runtime_role.map(|role| role.as_str().to_string()),
        reason: decision.reason,
        requires_handoff: decision.requires_handoff,
        requires_human_approval: decision.requires_human_approval,
    }
}

fn capability_decision(
    registry: &CapabilityRegistry,
    decision: CommandSurfaceDecision,
) -> GovernanceCapabilityDecision {
    let reason = decision.disabled_reason.clone().unwrap_or_else(|| {
        if decision.enabled {
            "capability allowed".to_string()
        } else {
            "capability unavailable".to_string()
        }
    });
    let outcome = if decision.enabled {
        GovernanceDecision::Allowed
    } else if should_defer_capability(registry, &decision, &reason) {
        GovernanceDecision::Deferred
    } else {
        GovernanceDecision::Rejected
    };
    GovernanceCapabilityDecision {
        decision: outcome,
        worker_id: decision.worker_id,
        command: decision.command,
        enabled: decision.enabled,
        health: decision.health,
        required_capabilities: decision.required_capabilities,
        missing_capabilities: decision.missing_capabilities,
        reason,
    }
}

fn should_defer_capability(
    registry: &CapabilityRegistry,
    decision: &CommandSurfaceDecision,
    reason: &str,
) -> bool {
    if matches!(
        decision.health,
        WorkerHealth::Unknown | WorkerHealth::Unauthenticated | WorkerHealth::PermissionDenied
    ) {
        return true;
    }
    if reason.contains("has not been checked") || reason.contains("requires authentication") {
        return true;
    }
    registry
        .worker(&decision.worker_id)
        .and_then(|worker| worker.capability_for_command(&decision.command))
        .map(|capability| matches!(capability.policy, CapabilityPolicy::RequiresAuth))
        .unwrap_or(false)
}

fn audit_sidecar_decision(mode: AuditSidecarMode) -> GovernanceAuditSidecarDecision {
    match mode {
        AuditSidecarMode::Independent => GovernanceAuditSidecarDecision {
            decision: GovernanceDecision::Allowed,
            mode,
            independent: true,
            reason: "audit sidecar is independent from the main runtime chain".to_string(),
        },
        AuditSidecarMode::NotRequested => GovernanceAuditSidecarDecision {
            decision: GovernanceDecision::Allowed,
            mode,
            independent: true,
            reason: "audit sidecar is not requested for this admission".to_string(),
        },
        AuditSidecarMode::BoundToMainChain => GovernanceAuditSidecarDecision {
            decision: GovernanceDecision::Rejected,
            mode,
            independent: false,
            reason: "audit sidecar must not be rebound into the main runtime chain".to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agentflow_capability_registry::{
        CapabilityPolicy, WorkerBoundary, WorkerCapability, WorkerKind, WorkerRegistryEntry,
        CAPABILITY_REGISTRY_VERSION,
    };
    use agentflow_role_policy::core_role_policy_registry;

    fn role_registry() -> RolePolicyRegistry {
        let ontology = agentflow_ontology::core_ontology_registry();
        let actions = agentflow_action_contract::core_action_contract_registry(&ontology);
        core_role_policy_registry(&ontology, &actions)
    }

    fn ready_registry() -> CapabilityRegistry {
        CapabilityRegistry {
            version: CAPABILITY_REGISTRY_VERSION.to_string(),
            workers: vec![WorkerRegistryEntry {
                worker_id: "local-shell-validator".to_string(),
                title: "Local Shell Validator".to_string(),
                kind: WorkerKind::Validator,
                health: WorkerHealth::Ready,
                requires_auth: false,
                disabled_reason: None,
                provider_smoke: None,
                runtime_roles: Vec::new(),
                skill_packs: Vec::new(),
                tool_kinds: Vec::new(),
                capabilities: vec![WorkerCapability {
                    capability_id: "local.test".to_string(),
                    label: "Local Test".to_string(),
                    command: "local.test".to_string(),
                    required: true,
                    available: true,
                    requires_auth: false,
                    policy: CapabilityPolicy::Allowed,
                    disabled_reason: None,
                }],
                boundary: WorkerBoundary::runtime_worker(vec!["evidence".to_string()]),
            }],
        }
    }

    fn disabled_registry() -> CapabilityRegistry {
        CapabilityRegistry {
            version: CAPABILITY_REGISTRY_VERSION.to_string(),
            workers: vec![WorkerRegistryEntry {
                worker_id: "codex".to_string(),
                title: "Codex Provider".to_string(),
                kind: WorkerKind::AgentProvider,
                health: WorkerHealth::Failed,
                requires_auth: false,
                disabled_reason: Some("provider smoke failed".to_string()),
                provider_smoke: None,
                runtime_roles: Vec::new(),
                skill_packs: Vec::new(),
                tool_kinds: Vec::new(),
                capabilities: vec![WorkerCapability {
                    capability_id: "launch".to_string(),
                    label: "Launch".to_string(),
                    command: "launch".to_string(),
                    required: true,
                    available: false,
                    requires_auth: false,
                    policy: CapabilityPolicy::Disabled,
                    disabled_reason: Some("provider codex smoke gate failed".to_string()),
                }],
                boundary: WorkerBoundary::runtime_worker(vec!["launch-request".to_string()]),
            }],
        }
    }

    fn request(worker_id: &str, command: &str) -> GovernancePolicyRequest {
        GovernancePolicyRequest {
            actor_role: "work-agent".to_string(),
            action_type: "startRun".to_string(),
            object_type: Some("Issue".to_string()),
            worker_id: worker_id.to_string(),
            command: command.to_string(),
            audit_sidecar_mode: AuditSidecarMode::Independent,
        }
    }

    #[test]
    fn governance_allows_valid_role_and_capability() {
        let report = evaluate_runtime_governance(
            &role_registry(),
            &ready_registry(),
            request("local-shell-validator", "local.test"),
        );

        assert_eq!(report.decision, GovernanceDecision::Allowed);
        assert!(report.runtime_admission);
        assert_eq!(report.trace.len(), 3);
    }

    #[test]
    fn governance_rejects_forbidden_role_action() {
        let mut request = request("local-shell-validator", "local.test");
        request.actor_role = "audit-agent".to_string();
        request.action_type = "startRun".to_string();

        let report = evaluate_runtime_governance(&role_registry(), &ready_registry(), request);

        assert_eq!(report.decision, GovernanceDecision::Rejected);
        assert_eq!(report.role_policy.reason, "actionNotAllowedForRole");
        assert!(!report.runtime_admission);
    }

    #[test]
    fn governance_defers_unchecked_provider_capability() {
        let registry = agentflow_capability_registry::default_capability_registry();
        let report = evaluate_runtime_governance(
            &role_registry(),
            &registry,
            request("github", "repo.read"),
        );

        assert_eq!(report.decision, GovernanceDecision::Deferred);
        assert!(report.capability_policy.reason.contains("not been checked"));
        assert!(!report.runtime_admission);
    }

    #[test]
    fn governance_rejects_failed_provider_smoke() {
        let report = evaluate_runtime_governance(
            &role_registry(),
            &disabled_registry(),
            request("codex", "launch"),
        );

        assert_eq!(report.decision, GovernanceDecision::Rejected);
        assert!(report
            .capability_policy
            .reason
            .contains("smoke gate failed"));
        assert!(!report.runtime_admission);
    }

    #[test]
    fn governance_rejects_audit_sidecar_bound_to_main_chain() {
        let mut request = request("local-shell-validator", "local.test");
        request.audit_sidecar_mode = AuditSidecarMode::BoundToMainChain;

        let report = evaluate_runtime_governance(&role_registry(), &ready_registry(), request);

        assert_eq!(report.decision, GovernanceDecision::Rejected);
        assert!(!report.audit_sidecar_policy.independent);
        assert!(!report.runtime_admission);
    }
}
