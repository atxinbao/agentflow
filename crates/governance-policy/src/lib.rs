//! Runtime governance policy evaluation.
//!
//! This crate evaluates runtime admission before work enters provider execution.
//! It is read-only: it does not append events, write authority files, mutate
//! runtime state, launch providers, or change audit sidecar state.

use agentflow_capability_registry::{
    evaluate_command, CapabilityPolicy, CapabilityRegistry, CommandSurfaceDecision, WorkerHealth,
};
use agentflow_ontology::{core_skill_registry_contract, CoreSkillDefinition};
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
    pub generic_action: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skill_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_surface: Option<String>,
    #[serde(default)]
    pub tool_scopes: Vec<String>,
    #[serde(default)]
    pub connector_scopes: Vec<String>,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
    #[serde(default)]
    pub artifact_refs: Vec<String>,
    #[serde(default)]
    pub expected_outputs: Vec<String>,
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
pub struct GovernanceSkillRegistryDecision {
    pub decision: GovernanceDecision,
    pub skill_ref: Option<String>,
    pub skill_id: Option<String>,
    pub owner_role: Option<String>,
    pub generic_action: Option<String>,
    pub reason: String,
    #[serde(default)]
    pub missing_evidence: Vec<String>,
    #[serde(default)]
    pub forbidden_tool_scopes: Vec<String>,
    #[serde(default)]
    pub forbidden_connector_scopes: Vec<String>,
    #[serde(default)]
    pub expected_outputs: Vec<String>,
    #[serde(default)]
    pub allowed_surfaces: Vec<String>,
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
    pub skill_registry_policy: GovernanceSkillRegistryDecision,
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
    let skill_registry_policy = skill_registry_decision(&request);
    let capability_raw =
        evaluate_command(capability_registry, &request.worker_id, &request.command);
    let capability_policy = capability_decision(capability_registry, capability_raw);
    let audit_sidecar_policy = audit_sidecar_decision(request.audit_sidecar_mode);

    let decision = role_policy
        .decision
        .merge(skill_registry_policy.decision)
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
            stage: "core-skill-registry".to_string(),
            decision: skill_registry_policy.decision,
            reason: skill_registry_policy.reason.clone(),
            evidence: skill_registry_policy
                .skill_id
                .iter()
                .cloned()
                .collect::<Vec<_>>(),
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
        skill_registry_policy,
        capability_policy,
        audit_sidecar_policy,
        trace,
    }
}

fn skill_registry_decision(request: &GovernancePolicyRequest) -> GovernanceSkillRegistryDecision {
    let generic_action = request
        .generic_action
        .clone()
        .unwrap_or_else(|| runtime_action_to_generic_action(&request.action_type).to_string());
    let Some(actor_role) = normalize_role(&request.actor_role) else {
        return GovernanceSkillRegistryDecision {
            decision: GovernanceDecision::Rejected,
            skill_ref: request.skill_ref.clone(),
            skill_id: None,
            owner_role: None,
            generic_action: Some(generic_action),
            reason: "unknownActorRole".to_string(),
            missing_evidence: Vec::new(),
            forbidden_tool_scopes: Vec::new(),
            forbidden_connector_scopes: Vec::new(),
            expected_outputs: Vec::new(),
            allowed_surfaces: Vec::new(),
        };
    };
    let Some(skill_ref) = request
        .skill_ref
        .as_ref()
        .filter(|value| !value.trim().is_empty())
    else {
        return GovernanceSkillRegistryDecision {
            decision: GovernanceDecision::Rejected,
            skill_ref: request.skill_ref.clone(),
            skill_id: None,
            owner_role: Some(actor_role),
            generic_action: Some(generic_action),
            reason: "missingSkillRef".to_string(),
            missing_evidence: Vec::new(),
            forbidden_tool_scopes: Vec::new(),
            forbidden_connector_scopes: Vec::new(),
            expected_outputs: Vec::new(),
            allowed_surfaces: Vec::new(),
        };
    };
    let registry = core_skill_registry_contract();
    let parsed = parse_skill_ref(skill_ref);
    if parsed
        .role
        .as_deref()
        .is_some_and(|role| role != actor_role)
    {
        return GovernanceSkillRegistryDecision {
            decision: GovernanceDecision::Rejected,
            skill_ref: request.skill_ref.clone(),
            skill_id: parsed.skill_token,
            owner_role: Some(actor_role),
            generic_action: Some(generic_action),
            reason: "skillOwnerMismatch".to_string(),
            missing_evidence: Vec::new(),
            forbidden_tool_scopes: Vec::new(),
            forbidden_connector_scopes: Vec::new(),
            expected_outputs: Vec::new(),
            allowed_surfaces: Vec::new(),
        };
    }
    let Some(skill) = resolve_core_skill(&registry.skills, &actor_role, &generic_action, &parsed)
    else {
        return GovernanceSkillRegistryDecision {
            decision: GovernanceDecision::Rejected,
            skill_ref: request.skill_ref.clone(),
            skill_id: parsed.skill_token,
            owner_role: Some(actor_role),
            generic_action: Some(generic_action),
            reason: "missingCoreSkill".to_string(),
            missing_evidence: Vec::new(),
            forbidden_tool_scopes: Vec::new(),
            forbidden_connector_scopes: Vec::new(),
            expected_outputs: Vec::new(),
            allowed_surfaces: Vec::new(),
        };
    };
    if normalize_role(&skill.owner_role).as_deref() != Some(actor_role.as_str()) {
        return GovernanceSkillRegistryDecision {
            decision: GovernanceDecision::Rejected,
            skill_ref: request.skill_ref.clone(),
            skill_id: Some(skill.skill_id.clone()),
            owner_role: Some(skill.owner_role.clone()),
            generic_action: Some(generic_action),
            reason: "skillNotOwnedByActorRole".to_string(),
            missing_evidence: Vec::new(),
            forbidden_tool_scopes: Vec::new(),
            forbidden_connector_scopes: Vec::new(),
            expected_outputs: skill.expected_outputs.clone(),
            allowed_surfaces: allowed_surfaces(&actor_role),
        };
    }
    if !skill
        .allowed_actions
        .iter()
        .any(|value| value == &generic_action)
    {
        return GovernanceSkillRegistryDecision {
            decision: GovernanceDecision::Rejected,
            skill_ref: request.skill_ref.clone(),
            skill_id: Some(skill.skill_id.clone()),
            owner_role: Some(skill.owner_role.clone()),
            generic_action: Some(generic_action),
            reason: "actionNotAllowedBySkill".to_string(),
            missing_evidence: Vec::new(),
            forbidden_tool_scopes: Vec::new(),
            forbidden_connector_scopes: Vec::new(),
            expected_outputs: skill.expected_outputs.clone(),
            allowed_surfaces: allowed_surfaces(&actor_role),
        };
    }
    if let Some(object_type) = request.object_type.as_deref() {
        if !is_core_object_type(object_type) {
            return GovernanceSkillRegistryDecision {
                decision: GovernanceDecision::Rejected,
                skill_ref: request.skill_ref.clone(),
                skill_id: Some(skill.skill_id.clone()),
                owner_role: Some(skill.owner_role.clone()),
                generic_action: Some(generic_action),
                reason: "invalidObjectType".to_string(),
                missing_evidence: Vec::new(),
                forbidden_tool_scopes: Vec::new(),
                forbidden_connector_scopes: Vec::new(),
                expected_outputs: skill.expected_outputs.clone(),
                allowed_surfaces: allowed_surfaces(&actor_role),
            };
        }
    }
    let allowed_surfaces = allowed_surfaces(&actor_role);
    if let Some(surface) = request.source_surface.as_deref() {
        if !allowed_surfaces.iter().any(|allowed| allowed == surface) {
            return GovernanceSkillRegistryDecision {
                decision: GovernanceDecision::Rejected,
                skill_ref: request.skill_ref.clone(),
                skill_id: Some(skill.skill_id.clone()),
                owner_role: Some(skill.owner_role.clone()),
                generic_action: Some(generic_action),
                reason: "sourceSurfaceNotAllowed".to_string(),
                missing_evidence: Vec::new(),
                forbidden_tool_scopes: Vec::new(),
                forbidden_connector_scopes: Vec::new(),
                expected_outputs: skill.expected_outputs.clone(),
                allowed_surfaces,
            };
        }
    }
    let forbidden_tool_scopes = forbidden_scopes(&request.tool_scopes, &skill.allowed_tool_scopes);
    let forbidden_connector_scopes =
        forbidden_scopes(&request.connector_scopes, &skill.allowed_connector_scopes);
    if !forbidden_tool_scopes.is_empty() || !forbidden_connector_scopes.is_empty() {
        return GovernanceSkillRegistryDecision {
            decision: GovernanceDecision::Rejected,
            skill_ref: request.skill_ref.clone(),
            skill_id: Some(skill.skill_id.clone()),
            owner_role: Some(skill.owner_role.clone()),
            generic_action: Some(generic_action),
            reason: "scopeNotAllowedBySkill".to_string(),
            missing_evidence: Vec::new(),
            forbidden_tool_scopes,
            forbidden_connector_scopes,
            expected_outputs: skill.expected_outputs.clone(),
            allowed_surfaces,
        };
    }
    let missing_evidence = missing_required_evidence(request, skill);
    if !missing_evidence.is_empty() {
        return GovernanceSkillRegistryDecision {
            decision: GovernanceDecision::Deferred,
            skill_ref: request.skill_ref.clone(),
            skill_id: Some(skill.skill_id.clone()),
            owner_role: Some(skill.owner_role.clone()),
            generic_action: Some(generic_action),
            reason: "missingRequiredEvidence".to_string(),
            missing_evidence,
            forbidden_tool_scopes: Vec::new(),
            forbidden_connector_scopes: Vec::new(),
            expected_outputs: skill.expected_outputs.clone(),
            allowed_surfaces,
        };
    }

    GovernanceSkillRegistryDecision {
        decision: GovernanceDecision::Allowed,
        skill_ref: request.skill_ref.clone(),
        skill_id: Some(skill.skill_id.clone()),
        owner_role: Some(skill.owner_role.clone()),
        generic_action: Some(generic_action),
        reason: "coreSkillRegistryAllowed".to_string(),
        missing_evidence: Vec::new(),
        forbidden_tool_scopes: Vec::new(),
        forbidden_connector_scopes: Vec::new(),
        expected_outputs: skill.expected_outputs.clone(),
        allowed_surfaces,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedSkillRef {
    role: Option<String>,
    skill_token: Option<String>,
}

fn parse_skill_ref(value: &str) -> ParsedSkillRef {
    let parts = value
        .split(':')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    match parts.as_slice() {
        ["core", role, token, ..] => ParsedSkillRef {
            role: normalize_role(role),
            skill_token: Some((*token).to_string()),
        },
        ["core", token] => ParsedSkillRef {
            role: None,
            skill_token: Some((*token).to_string()),
        },
        [role, token] => ParsedSkillRef {
            role: normalize_role(role),
            skill_token: Some((*token).to_string()),
        },
        [token] => ParsedSkillRef {
            role: None,
            skill_token: Some((*token).to_string()),
        },
        _ => ParsedSkillRef {
            role: None,
            skill_token: None,
        },
    }
}

fn resolve_core_skill<'a>(
    skills: &'a [CoreSkillDefinition],
    actor_role: &str,
    generic_action: &str,
    parsed: &ParsedSkillRef,
) -> Option<&'a CoreSkillDefinition> {
    if let Some(token) = parsed.skill_token.as_deref() {
        if let Some(skill) = skills.iter().find(|skill| skill.skill_id == token) {
            return Some(skill);
        }
    }
    skills.iter().find(|skill| {
        normalize_role(&skill.owner_role).as_deref() == Some(actor_role)
            && skill
                .allowed_actions
                .iter()
                .any(|allowed| allowed == generic_action)
    })
}

fn normalize_role(value: &str) -> Option<String> {
    match value.trim() {
        "GoalAgent" | "goal-agent" => Some("goal-agent".to_string()),
        "SpecAgent" | "spec-agent" => Some("spec-agent".to_string()),
        "WorkAgent" | "work-agent" | "BuildAgent" | "build-agent" => Some("work-agent".to_string()),
        "AuditAgent" | "audit-agent" => Some("audit-agent".to_string()),
        "DeliveryAgent" | "delivery-agent" => Some("delivery-agent".to_string()),
        "HumanOwner" | "human-owner" => Some("human-owner".to_string()),
        _ => None,
    }
}

fn runtime_action_to_generic_action(action_type: &str) -> &'static str {
    match action_type {
        "submitRequirement"
        | "normalizeRequirement"
        | "classifyRequirement"
        | "draftSpec"
        | "approveSpec"
        | "createProject"
        | "createIssue" => "acceptObject",
        "activateIssue" | "claimIssue" | "startRun" => "startObject",
        "writePatch" => "attachArtifact",
        "runValidation" | "submitEvidence" => "attachEvidence",
        "prepareDelivery" | "submitArtifact" => "attachArtifact",
        "markIssueDone" | "recordDecision" => "completeObject",
        "requestAudit" => "submitForReview",
        "createFinding" => "blockObject",
        "linkFixIssue" => "supersedeObject",
        _ => "unlisted-action",
    }
}

fn is_core_object_type(value: &str) -> bool {
    matches!(
        value,
        "Requirement"
            | "Spec"
            | "Project"
            | "Issue"
            | "Run"
            | "Evidence"
            | "Artifact"
            | "Decision"
            | "Audit"
            | "Finding"
    )
}

fn allowed_surfaces(role: &str) -> Vec<String> {
    let values: &[&str] = match role {
        "work-agent" | "delivery-agent" | "audit-agent" => &["agent", "cli", "sdk", "system"],
        "human-owner" => &["conversation", "desktop", "cli", "sdk", "system"],
        _ => &["conversation", "desktop", "agent", "cli", "sdk", "system"],
    };
    values.iter().map(|value| (*value).to_string()).collect()
}

fn forbidden_scopes(requested: &[String], allowed: &[String]) -> Vec<String> {
    requested
        .iter()
        .filter(|scope| !allowed.iter().any(|allowed| allowed == *scope))
        .cloned()
        .collect()
}

fn missing_required_evidence(
    request: &GovernancePolicyRequest,
    skill: &CoreSkillDefinition,
) -> Vec<String> {
    let refs = request
        .evidence_refs
        .iter()
        .chain(request.artifact_refs.iter())
        .map(|value| value.to_ascii_lowercase())
        .collect::<Vec<_>>();
    skill
        .required_evidence
        .iter()
        .filter(|required| {
            let required = required.to_ascii_lowercase();
            !refs.iter().any(|value| value.contains(&required))
        })
        .cloned()
        .collect()
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
        let ontology = agentflow_ontology::software_dev_reference_ontology_registry();
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
            generic_action: Some("startObject".to_string()),
            object_type: Some("Issue".to_string()),
            skill_ref: Some("core:work-agent:work-execution-skill".to_string()),
            source_surface: Some("agent".to_string()),
            tool_scopes: Vec::new(),
            connector_scopes: Vec::new(),
            evidence_refs: vec!["EvidenceRef:issue-ready".to_string()],
            artifact_refs: vec!["ArtifactRef:context-pack".to_string()],
            expected_outputs: Vec::new(),
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
        assert_eq!(report.trace.len(), 4);
        assert_eq!(
            report.skill_registry_policy.skill_id.as_deref(),
            Some("work-execution-skill")
        );
    }

    #[test]
    fn runtime_action_mapping_keeps_done_separate_from_review() {
        assert_eq!(
            runtime_action_to_generic_action("markIssueDone"),
            "completeObject"
        );
        assert_eq!(
            runtime_action_to_generic_action("requestAudit"),
            "submitForReview"
        );
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
    fn governance_rejects_missing_skill_ref() {
        let mut request = request("local-shell-validator", "local.test");
        request.skill_ref = None;

        let report = evaluate_runtime_governance(&role_registry(), &ready_registry(), request);

        assert_eq!(report.decision, GovernanceDecision::Rejected);
        assert_eq!(report.skill_registry_policy.reason, "missingSkillRef");
        assert!(!report.runtime_admission);
    }

    #[test]
    fn governance_rejects_unauthorized_skill_owner() {
        let mut request = request("local-shell-validator", "local.test");
        request.skill_ref = Some("core:audit-agent:work-execution-skill".to_string());

        let report = evaluate_runtime_governance(&role_registry(), &ready_registry(), request);

        assert_eq!(report.decision, GovernanceDecision::Rejected);
        assert_eq!(report.skill_registry_policy.reason, "skillOwnerMismatch");
        assert!(!report.runtime_admission);
    }

    #[test]
    fn governance_defers_missing_skill_evidence() {
        let mut request = request("local-shell-validator", "local.test");
        request.evidence_refs = Vec::new();
        request.artifact_refs = Vec::new();

        let report = evaluate_runtime_governance(&role_registry(), &ready_registry(), request);

        assert_eq!(report.decision, GovernanceDecision::Deferred);
        assert_eq!(
            report.skill_registry_policy.reason,
            "missingRequiredEvidence"
        );
        assert!(report
            .skill_registry_policy
            .missing_evidence
            .contains(&"EvidenceRef".to_string()));
        assert!(!report.runtime_admission);
    }

    #[test]
    fn governance_rejects_forbidden_skill_surface() {
        let mut request = request("local-shell-validator", "local.test");
        request.source_surface = Some("desktop".to_string());

        let report = evaluate_runtime_governance(&role_registry(), &ready_registry(), request);

        assert_eq!(report.decision, GovernanceDecision::Rejected);
        assert_eq!(
            report.skill_registry_policy.reason,
            "sourceSurfaceNotAllowed"
        );
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
