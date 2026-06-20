use agentflow_action_contract::ActionContractRegistry;
use agentflow_ontology::OntologyRegistry;

use crate::model::{
    AgentRolePolicy, AgentRolePolicyBundle, AgentRolePolicyStatus, ApprovalGate, HandoffRule,
    ObjectScope, ObjectScopeKind, ProductAgentRole, ProductRoleBinding, RoleActionCapability,
    RoleCapabilityMode, RolePolicyCompatibility, RoleToolScope, RuntimeAgentRole, ToolKind,
    ROLE_POLICY_BUNDLE_VERSION,
};
use crate::registry::RolePolicyRegistry;

pub const CORE_ROLE_POLICY_ID: &str = "agentflow.roles.core";
pub const CORE_ROLE_POLICY_NAMESPACE: &str = "agentflow.roles.core";
pub const CORE_ROLE_POLICY_VERSION: &str = "v1-draft";
pub const CORE_ROLE_POLICY_REF: &str = "agentflow.roles.core@v1-draft";

pub fn core_role_policy_bundle() -> AgentRolePolicyBundle {
    AgentRolePolicyBundle {
        version: ROLE_POLICY_BUNDLE_VERSION.into(),
        bundle_id: CORE_ROLE_POLICY_ID.into(),
        namespace: CORE_ROLE_POLICY_NAMESPACE.into(),
        definition_version: CORE_ROLE_POLICY_VERSION.into(),
        status: AgentRolePolicyStatus::Active,
        product_role_bindings: vec![
            product_binding(ProductAgentRole::GoalAgent, RuntimeAgentRole::GoalAgent, "Product Goal Agent maps directly to runtime GoalAgent."),
            product_binding(ProductAgentRole::SpecAgent, RuntimeAgentRole::SpecAgent, "Product Spec Agent maps directly to runtime SpecAgent."),
            product_binding(ProductAgentRole::WorkAgent, RuntimeAgentRole::WorkAgent, "Product Work Agent maps directly to runtime WorkAgent."),
            product_binding(ProductAgentRole::AuditAgent, RuntimeAgentRole::AuditAgent, "Product Audit Agent maps directly to runtime AuditAgent."),
            product_binding(ProductAgentRole::DeliveryAgent, RuntimeAgentRole::DeliveryAgent, "Product Delivery Agent maps directly to runtime DeliveryAgent."),
        ],
        roles: vec![
            goal_agent_policy(),
            spec_agent_policy(),
            work_agent_policy(),
            audit_agent_policy(),
            delivery_agent_policy(),
            review_agent_policy(),
            coordinator_agent_policy(),
            human_owner_policy(),
        ],
        handoff_rules: vec![
            handoff_rule(
                "spec-to-work-approved-spec",
                RuntimeAgentRole::SpecAgent,
                RuntimeAgentRole::WorkAgent,
                "Issue",
                &["createIssue", "activateIssue"],
                &["approvedSpecRef", "issueContractRef"],
                &["issueContract", "workflowRef"],
                &["Spec Agent hands approved execution contract to WorkAgent."],
            ),
            handoff_rule(
                "work-to-delivery-closeout",
                RuntimeAgentRole::WorkAgent,
                RuntimeAgentRole::DeliveryAgent,
                "Issue",
                &["markIssueDone"],
                &["verificationLogRef", "artifactSummaryRef"],
                &["publicDeliveryRecord"],
                &["WorkAgent hands verified completion package to DeliveryAgent."],
            ),
            handoff_rule(
                "delivery-to-audit-explicit-request",
                RuntimeAgentRole::DeliveryAgent,
                RuntimeAgentRole::AuditAgent,
                "Audit",
                &["requestAudit"],
                &["auditRequestRef", "deliverySummaryRef"],
                &["auditReport"],
                &["DeliveryAgent can request independent audit without letting WorkAgent create audit facts."],
            ),
            handoff_rule(
                "review-to-human-decision",
                RuntimeAgentRole::ReviewAgent,
                RuntimeAgentRole::HumanOwner,
                "Decision",
                &["recordDecision"],
                &["reviewDecisionRef"],
                &["humanDecisionRecord"],
                &["ReviewAgent escalates unresolved decisions to HumanOwner."],
            ),
        ],
        compatibility: RolePolicyCompatibility {
            aliases: vec![
                alias("BuildAgent", RuntimeAgentRole::WorkAgent, "Legacy runtime alias."),
                alias("build-agent", RuntimeAgentRole::WorkAgent, "Provider-facing execution alias."),
                alias("WorkAgent", RuntimeAgentRole::WorkAgent, "CamelCase compatibility alias."),
                alias("SpecAgent", RuntimeAgentRole::SpecAgent, "CamelCase compatibility alias."),
                alias("AuditAgent", RuntimeAgentRole::AuditAgent, "CamelCase compatibility alias."),
                alias("DeliveryAgent", RuntimeAgentRole::DeliveryAgent, "CamelCase compatibility alias."),
                alias("ReviewAgent", RuntimeAgentRole::ReviewAgent, "CamelCase compatibility alias."),
                alias("CoordinatorAgent", RuntimeAgentRole::CoordinatorAgent, "CamelCase compatibility alias."),
                alias("HumanOwner", RuntimeAgentRole::HumanOwner, "CamelCase compatibility alias."),
                alias("GoalAgent", RuntimeAgentRole::GoalAgent, "CamelCase compatibility alias."),
            ],
        },
    }
}

pub fn core_role_policy_registry(
    ontology_registry: &OntologyRegistry,
    action_registry: &ActionContractRegistry,
) -> RolePolicyRegistry {
    RolePolicyRegistry::load_bundle(
        core_role_policy_bundle(),
        ontology_registry,
        action_registry,
    )
    .expect("built-in core role policies must validate")
}

fn goal_agent_policy() -> AgentRolePolicy {
    AgentRolePolicy {
        role_id: RuntimeAgentRole::GoalAgent,
        name: "Goal Agent".into(),
        description: "Owns goal direction, planning intent, and project-level decision framing."
            .into(),
        status: AgentRolePolicyStatus::Active,
        can_read: vec!["Requirement", "Spec", "Project", "Issue", "Decision"]
            .into_iter()
            .map(str::to_string)
            .collect(),
        can_write: vec!["Decision"].into_iter().map(str::to_string).collect(),
        action_capabilities: vec![capability(
            "recordDecision",
            RoleCapabilityMode::Decide,
            Some("Decision"),
            Some(ObjectScopeKind::HumanDecisionTarget),
            false,
            None,
            true,
            &["goalDecision"],
        )],
        must_produce: vec!["goalDecision", "projectDirection"]
            .into_iter()
            .map(str::to_string)
            .collect(),
        cannot_do: vec![
            "startRun",
            "submitEvidence",
            "markIssueDone",
            "createFinding",
        ]
        .into_iter()
        .map(str::to_string)
        .collect(),
        object_scopes: vec![
            scope(
                "goal-project-read",
                "Project",
                ObjectScopeKind::ProjectWideRead,
                "Read active project state.",
            ),
            scope(
                "goal-decision-target",
                "Decision",
                ObjectScopeKind::HumanDecisionTarget,
                "Record project-level decisions.",
            ),
        ],
        tool_scope: tool_scope(
            &[
                ToolKind::ReadDocs,
                ToolKind::InspectContext,
                ToolKind::InspectState,
                ToolKind::GenerateReport,
            ],
            &[ToolKind::LocalBuild, ToolKind::LocalTest],
            false,
        ),
        handoff_rules: vec![],
        approval_gates: vec![
            ApprovalGate::ContractRequired,
            ApprovalGate::HumanApprovalRequired,
        ],
        required_evidence: vec!["goalDecision"]
            .into_iter()
            .map(str::to_string)
            .collect(),
        boundary_notes: vec!["Goal Agent does not execute code or produce delivery facts.".into()],
    }
}

fn spec_agent_policy() -> AgentRolePolicy {
    AgentRolePolicy {
        role_id: RuntimeAgentRole::SpecAgent,
        name: "Spec Agent".into(),
        description: "Converts requirements into structured draft spec and project contracts."
            .into(),
        status: AgentRolePolicyStatus::Active,
        can_read: vec!["Requirement", "Spec", "Project", "Issue", "Decision"]
            .into_iter()
            .map(str::to_string)
            .collect(),
        can_write: vec!["Spec", "Project", "Issue"]
            .into_iter()
            .map(str::to_string)
            .collect(),
        action_capabilities: vec![
            capability(
                "submitRequirement",
                RoleCapabilityMode::Propose,
                Some("Requirement"),
                Some(ObjectScopeKind::ProjectWideRead),
                false,
                None,
                false,
                &[],
            ),
            capability(
                "normalizeRequirement",
                RoleCapabilityMode::Propose,
                Some("Requirement"),
                Some(ObjectScopeKind::ProjectWideRead),
                false,
                None,
                false,
                &[],
            ),
            capability(
                "classifyRequirement",
                RoleCapabilityMode::Propose,
                Some("Requirement"),
                Some(ObjectScopeKind::ProjectWideRead),
                false,
                None,
                false,
                &["requirementClassification"],
            ),
            capability(
                "draftSpec",
                RoleCapabilityMode::Propose,
                Some("Requirement"),
                Some(ObjectScopeKind::ProjectWideRead),
                false,
                None,
                false,
                &["specDraftPreview"],
            ),
            capability(
                "createProject",
                RoleCapabilityMode::Propose,
                Some("Spec"),
                Some(ObjectScopeKind::ApprovedSpec),
                false,
                None,
                true,
                &["approvedSpecRef"],
            ),
            capability(
                "createIssue",
                RoleCapabilityMode::Propose,
                Some("Project"),
                Some(ObjectScopeKind::ProjectWideRead),
                true,
                Some("spec-to-work-approved-spec"),
                true,
                &["issueContract"],
            ),
        ],
        must_produce: vec![
            "requirementClassification",
            "specDraftPreview",
            "boundaryNotes",
            "issueContract",
        ]
        .into_iter()
        .map(str::to_string)
        .collect(),
        cannot_do: vec![
            "startRun",
            "submitEvidence",
            "markIssueDone",
            "requestAudit",
            "createFinding",
        ]
        .into_iter()
        .map(str::to_string)
        .collect(),
        object_scopes: vec![
            scope(
                "spec-approved-spec",
                "Spec",
                ObjectScopeKind::ApprovedSpec,
                "Operate on approved spec targets.",
            ),
            scope(
                "spec-project-read",
                "Project",
                ObjectScopeKind::ProjectWideRead,
                "Read project planning aggregate.",
            ),
            scope(
                "spec-requirement-read",
                "Requirement",
                ObjectScopeKind::ProjectWideRead,
                "Read requirement intake facts.",
            ),
        ],
        tool_scope: tool_scope(
            &[
                ToolKind::ReadDocs,
                ToolKind::InspectContext,
                ToolKind::InspectState,
                ToolKind::GenerateReport,
            ],
            &[
                ToolKind::LocalBuild,
                ToolKind::LocalTest,
                ToolKind::BrowserSmoke,
            ],
            false,
        ),
        handoff_rules: vec!["spec-to-work-approved-spec".into()],
        approval_gates: vec![
            ApprovalGate::ContractRequired,
            ApprovalGate::HumanApprovalRequired,
        ],
        required_evidence: vec!["specDraftPreview", "boundaryNotes"]
            .into_iter()
            .map(str::to_string)
            .collect(),
        boundary_notes: vec![
            "Spec Agent can generate preview and contract materials but cannot start execution."
                .into(),
        ],
    }
}

fn work_agent_policy() -> AgentRolePolicy {
    AgentRolePolicy {
        role_id: RuntimeAgentRole::WorkAgent,
        name: "Work Agent".into(),
        description: "Executes approved issue contracts inside allowed execution boundaries.".into(),
        status: AgentRolePolicyStatus::Active,
        can_read: vec!["Issue", "Run", "Evidence", "Artifact", "Spec", "Decision"]
            .into_iter()
            .map(str::to_string)
            .collect(),
        can_write: vec!["Run", "Evidence", "Artifact", "Issue"]
            .into_iter()
            .map(str::to_string)
            .collect(),
        action_capabilities: vec![
            capability("activateIssue", RoleCapabilityMode::Execute, Some("Issue"), Some(ObjectScopeKind::AssignedIssue), false, None, false, &[]),
            capability("claimIssue", RoleCapabilityMode::Execute, Some("Issue"), Some(ObjectScopeKind::AssignedIssue), false, None, false, &[]),
            capability("startRun", RoleCapabilityMode::Execute, Some("Issue"), Some(ObjectScopeKind::AssignedIssue), false, None, false, &["runLog"]),
            capability("writePatch", RoleCapabilityMode::Execute, Some("Run"), Some(ObjectScopeKind::CurrentRun), false, None, false, &["artifactSummary"]),
            capability("runValidation", RoleCapabilityMode::Execute, Some("Run"), Some(ObjectScopeKind::CurrentRun), false, None, false, &["verificationLog"]),
            capability("prepareDelivery", RoleCapabilityMode::Execute, Some("Run"), Some(ObjectScopeKind::CurrentRun), false, None, false, &["artifactSummary"]),
            capability("submitEvidence", RoleCapabilityMode::Execute, Some("Run"), Some(ObjectScopeKind::CurrentRun), false, None, false, &["verificationLog"]),
            capability("submitArtifact", RoleCapabilityMode::Execute, Some("Run"), Some(ObjectScopeKind::CurrentRun), false, None, false, &["artifactSummary"]),
            capability("markIssueDone", RoleCapabilityMode::Execute, Some("Issue"), Some(ObjectScopeKind::AssignedIssue), true, Some("work-to-delivery-closeout"), false, &["verificationLog", "artifactSummary", "implementationSummary"]),
        ],
        must_produce: vec!["runLog", "implementationSummary", "verificationLog", "artifactSummary"]
            .into_iter()
            .map(str::to_string)
            .collect(),
        cannot_do: vec!["draftSpec", "approveSpec", "requestAudit", "createFinding", "recordDecision"]
            .into_iter()
            .map(str::to_string)
            .collect(),
        object_scopes: vec![
            scope("work-assigned-issue", "Issue", ObjectScopeKind::AssignedIssue, "Operate only on the assigned issue."),
            scope("work-current-run", "Run", ObjectScopeKind::CurrentRun, "Write only inside the current execution run."),
        ],
        tool_scope: tool_scope(
            &[ToolKind::InspectContext, ToolKind::Filesystem, ToolKind::LocalBuild, ToolKind::LocalTest, ToolKind::BrowserSmoke, ToolKind::InspectDiff, ToolKind::InspectState],
            &[ToolKind::GenerateReport],
            true,
        ),
        handoff_rules: vec!["work-to-delivery-closeout".into()],
        approval_gates: vec![ApprovalGate::ContractRequired, ApprovalGate::HandoffRequired],
        required_evidence: vec!["verificationLog", "artifactSummary", "implementationSummary"]
            .into_iter()
            .map(str::to_string)
            .collect(),
        boundary_notes: vec![
            "BuildAgent is a compatibility alias only; WorkAgent is the runtime authority role.".into(),
            "WorkAgent does not create Audit facts and cannot accept delivery on behalf of HumanOwner.".into(),
        ],
    }
}

fn audit_agent_policy() -> AgentRolePolicy {
    AgentRolePolicy {
        role_id: RuntimeAgentRole::AuditAgent,
        name: "Audit Agent".into(),
        description:
            "Inspects evidence independently and records findings without rewriting build facts."
                .into(),
        status: AgentRolePolicyStatus::Active,
        can_read: vec!["Issue", "Run", "Evidence", "Artifact", "Audit", "Finding"]
            .into_iter()
            .map(str::to_string)
            .collect(),
        can_write: vec!["Audit", "Finding", "Evidence", "Artifact"]
            .into_iter()
            .map(str::to_string)
            .collect(),
        action_capabilities: vec![
            capability(
                "createFinding",
                RoleCapabilityMode::Review,
                Some("Audit"),
                Some(ObjectScopeKind::OwnedFinding),
                false,
                None,
                false,
                &["auditReport"],
            ),
            capability(
                "linkFixIssue",
                RoleCapabilityMode::Review,
                Some("Finding"),
                Some(ObjectScopeKind::OwnedFinding),
                false,
                None,
                false,
                &["findingRecord"],
            ),
            capability(
                "submitEvidence",
                RoleCapabilityMode::Review,
                Some("Run"),
                Some(ObjectScopeKind::ReferencedEvidence),
                false,
                None,
                false,
                &["evidenceMap"],
            ),
            capability(
                "submitArtifact",
                RoleCapabilityMode::Review,
                Some("Run"),
                Some(ObjectScopeKind::ReferencedEvidence),
                false,
                None,
                false,
                &["auditReport"],
            ),
        ],
        must_produce: vec!["auditReport", "evidenceMap", "findingRecord"]
            .into_iter()
            .map(str::to_string)
            .collect(),
        cannot_do: vec!["markIssueDone", "approveSpec", "requestAudit", "draftSpec"]
            .into_iter()
            .map(str::to_string)
            .collect(),
        object_scopes: vec![
            scope(
                "audit-owned-finding",
                "Finding",
                ObjectScopeKind::OwnedFinding,
                "Create and maintain findings owned by the audit flow.",
            ),
            scope(
                "audit-referenced-evidence",
                "Run",
                ObjectScopeKind::ReferencedEvidence,
                "Read referenced run evidence without mutating build state.",
            ),
            scope(
                "audit-audit-object",
                "Audit",
                ObjectScopeKind::ProjectWideRead,
                "Operate on independent audit objects.",
            ),
        ],
        tool_scope: tool_scope(
            &[
                ToolKind::InspectContext,
                ToolKind::ReadEvidence,
                ToolKind::InspectState,
                ToolKind::GenerateReport,
                ToolKind::InspectDiff,
            ],
            &[
                ToolKind::LocalBuild,
                ToolKind::LocalTest,
                ToolKind::Filesystem,
            ],
            true,
        ),
        handoff_rules: vec![],
        approval_gates: vec![
            ApprovalGate::ContractRequired,
            ApprovalGate::IndependentAuditRequired,
        ],
        required_evidence: vec!["auditReport", "evidenceMap"]
            .into_iter()
            .map(str::to_string)
            .collect(),
        boundary_notes: vec![
            "AuditAgent can read build evidence but cannot rewrite build delivery facts.".into(),
            "AuditAgent does not mark issues done.".into(),
        ],
    }
}

fn delivery_agent_policy() -> AgentRolePolicy {
    AgentRolePolicy {
        role_id: RuntimeAgentRole::DeliveryAgent,
        name: "Delivery Agent".into(),
        description: "Packages public delivery records and releases without overriding execution or audit authority.".into(),
        status: AgentRolePolicyStatus::Active,
        can_read: vec!["Issue", "Run", "Evidence", "Artifact", "Project", "Decision"]
            .into_iter()
            .map(str::to_string)
            .collect(),
        can_write: vec!["Artifact", "Project"].into_iter().map(str::to_string).collect(),
        action_capabilities: vec![capability(
            "submitArtifact",
            RoleCapabilityMode::Execute,
            Some("Run"),
            Some(ObjectScopeKind::CurrentRun),
            false,
            None,
            false,
            &["publicDeliveryRecord"],
        )],
        must_produce: vec!["publicDeliveryRecord", "releaseNotes"]
            .into_iter()
            .map(str::to_string)
            .collect(),
        cannot_do: vec!["draftSpec", "approveSpec", "createFinding", "markIssueDone"]
            .into_iter()
            .map(str::to_string)
            .collect(),
        object_scopes: vec![
            scope("delivery-current-run", "Run", ObjectScopeKind::CurrentRun, "Read the completed run for delivery packaging."),
            scope("delivery-project-read", "Project", ObjectScopeKind::ProjectWideRead, "Read project release state."),
        ],
        tool_scope: tool_scope(
            &[ToolKind::InspectContext, ToolKind::ReadEvidence, ToolKind::GenerateReport, ToolKind::InspectState],
            &[ToolKind::LocalBuild, ToolKind::LocalTest],
            true,
        ),
        handoff_rules: vec!["delivery-to-audit-explicit-request".into()],
        approval_gates: vec![ApprovalGate::ContractRequired, ApprovalGate::ExplicitAuditRequestRequired],
        required_evidence: vec!["publicDeliveryRecord"]
            .into_iter()
            .map(str::to_string)
            .collect(),
        boundary_notes: vec![
            "DeliveryAgent prepares public delivery material but does not perform audit decisions.".into(),
            "Audit remains an independent follow-up workflow.".into(),
        ],
    }
}

fn review_agent_policy() -> AgentRolePolicy {
    AgentRolePolicy {
        role_id: RuntimeAgentRole::ReviewAgent,
        name: "Review Agent".into(),
        description: "Validates evidence sufficiency and emits review decisions or missing-evidence guidance.".into(),
        status: AgentRolePolicyStatus::Active,
        can_read: vec!["Issue", "Run", "Evidence", "Artifact", "Decision"]
            .into_iter()
            .map(str::to_string)
            .collect(),
        can_write: vec!["Decision", "Evidence"].into_iter().map(str::to_string).collect(),
        action_capabilities: vec![
            capability("submitEvidence", RoleCapabilityMode::Review, Some("Run"), Some(ObjectScopeKind::ReferencedEvidence), false, None, false, &["reviewEvidenceMap"]),
            capability("recordDecision", RoleCapabilityMode::Review, Some("Decision"), Some(ObjectScopeKind::HumanDecisionTarget), true, Some("review-to-human-decision"), false, &["reviewDecision"]),
        ],
        must_produce: vec!["reviewDecision", "missingEvidenceList"]
            .into_iter()
            .map(str::to_string)
            .collect(),
        cannot_do: vec!["draftSpec", "requestAudit", "createFinding", "startRun"]
            .into_iter()
            .map(str::to_string)
            .collect(),
        object_scopes: vec![
            scope("review-referenced-evidence", "Run", ObjectScopeKind::ReferencedEvidence, "Inspect run evidence for contract completeness."),
            scope("review-decision-target", "Decision", ObjectScopeKind::HumanDecisionTarget, "Escalate unresolved review outcomes to human decision."),
        ],
        tool_scope: tool_scope(
            &[ToolKind::ReadEvidence, ToolKind::InspectState, ToolKind::GenerateReport],
            &[ToolKind::LocalBuild, ToolKind::Filesystem],
            true,
        ),
        handoff_rules: vec!["review-to-human-decision".into()],
        approval_gates: vec![ApprovalGate::ContractRequired],
        required_evidence: vec!["reviewDecision"]
            .into_iter()
            .map(str::to_string)
            .collect(),
        boundary_notes: vec!["ReviewAgent validates evidence; it does not implement code.".into()],
    }
}

fn coordinator_agent_policy() -> AgentRolePolicy {
    AgentRolePolicy {
        role_id: RuntimeAgentRole::CoordinatorAgent,
        name: "Coordinator Agent".into(),
        description: "Explains queueing, dependency, and fix routing decisions without bypassing arbitration.".into(),
        status: AgentRolePolicyStatus::Active,
        can_read: vec!["Project", "Issue", "Finding", "Decision"]
            .into_iter()
            .map(str::to_string)
            .collect(),
        can_write: vec!["Decision"].into_iter().map(str::to_string).collect(),
        action_capabilities: vec![
            capability("recordDecision", RoleCapabilityMode::Decide, Some("Decision"), Some(ObjectScopeKind::HumanDecisionTarget), false, None, true, &["coordinationDecision"]),
            capability("linkFixIssue", RoleCapabilityMode::Propose, Some("Finding"), Some(ObjectScopeKind::OwnedFinding), false, None, true, &["fixRoutingNote"]),
        ],
        must_produce: vec!["coordinationDecision", "fixRoutingNote"]
            .into_iter()
            .map(str::to_string)
            .collect(),
        cannot_do: vec!["startRun", "markIssueDone", "approveSpec", "requestAudit"]
            .into_iter()
            .map(str::to_string)
            .collect(),
        object_scopes: vec![
            scope("coordinator-project-read", "Project", ObjectScopeKind::ProjectWideRead, "Read project dependencies and queue state."),
            scope("coordinator-finding-owned", "Finding", ObjectScopeKind::OwnedFinding, "Route finding-driven fix work."),
            scope("coordinator-decision-target", "Decision", ObjectScopeKind::HumanDecisionTarget, "Record coordination decisions."),
        ],
        tool_scope: tool_scope(
            &[ToolKind::InspectContext, ToolKind::InspectState, ToolKind::GenerateReport],
            &[ToolKind::LocalBuild, ToolKind::LocalTest, ToolKind::Filesystem],
            false,
        ),
        handoff_rules: vec![],
        approval_gates: vec![ApprovalGate::ContractRequired, ApprovalGate::HumanApprovalRequired],
        required_evidence: vec!["coordinationDecision"]
            .into_iter()
            .map(str::to_string)
            .collect(),
        boundary_notes: vec![
            "CoordinatorAgent cannot bypass arbitration or override human approval.".into(),
            "CoordinatorAgent does not write accepted facts directly.".into(),
        ],
    }
}

fn human_owner_policy() -> AgentRolePolicy {
    AgentRolePolicy {
        role_id: RuntimeAgentRole::HumanOwner,
        name: "Human Owner".into(),
        description: "Supplies explicit human confirmation, rejection, audit request, and reopening decisions through proposals.".into(),
        status: AgentRolePolicyStatus::Active,
        can_read: vec!["Requirement", "Spec", "Issue", "Audit", "Finding", "Decision"]
            .into_iter()
            .map(str::to_string)
            .collect(),
        can_write: vec!["Decision", "Audit", "Finding"]
            .into_iter()
            .map(str::to_string)
            .collect(),
        action_capabilities: vec![
            capability("approveSpec", RoleCapabilityMode::Decide, Some("Spec"), Some(ObjectScopeKind::ApprovedSpec), false, None, true, &["humanConfirmation"]),
            capability("recordDecision", RoleCapabilityMode::Decide, Some("Decision"), Some(ObjectScopeKind::HumanDecisionTarget), false, None, true, &["humanDecisionRecord"]),
            capability("requestAudit", RoleCapabilityMode::Decide, Some("Issue"), Some(ObjectScopeKind::AssignedIssue), true, Some("delivery-to-audit-explicit-request"), true, &["humanConfirmation"]),
            capability("linkFixIssue", RoleCapabilityMode::Decide, Some("Finding"), Some(ObjectScopeKind::OwnedFinding), false, None, true, &["humanDecisionRecord"]),
        ],
        must_produce: vec!["humanConfirmation", "humanDecisionRecord"]
            .into_iter()
            .map(str::to_string)
            .collect(),
        cannot_do: vec!["startRun", "submitEvidence", "submitArtifact", "createFinding"]
            .into_iter()
            .map(str::to_string)
            .collect(),
        object_scopes: vec![
            scope("human-approved-spec", "Spec", ObjectScopeKind::ApprovedSpec, "Approve or reject specification targets."),
            scope("human-decision-target", "Decision", ObjectScopeKind::HumanDecisionTarget, "Create explicit human decisions."),
            scope("human-audit-target", "Issue", ObjectScopeKind::AssignedIssue, "Request audit on an issue."),
            scope("human-finding-target", "Finding", ObjectScopeKind::OwnedFinding, "Route fix issues from findings."),
        ],
        tool_scope: tool_scope(
            &[ToolKind::ReadDocs, ToolKind::InspectContext, ToolKind::InspectState],
            &[ToolKind::LocalBuild, ToolKind::LocalTest, ToolKind::Filesystem],
            false,
        ),
        handoff_rules: vec!["delivery-to-audit-explicit-request".into()],
        approval_gates: vec![ApprovalGate::ContractRequired, ApprovalGate::HumanApprovalRequired],
        required_evidence: vec!["humanConfirmation", "humanDecisionRecord"]
            .into_iter()
            .map(str::to_string)
            .collect(),
        boundary_notes: vec![
            "HumanOwner still submits Action Proposal instead of writing state directly.".into(),
            "HumanOwner decisions become facts only after Runtime accepts them.".into(),
        ],
    }
}

fn product_binding(
    product_role: ProductAgentRole,
    runtime_role: RuntimeAgentRole,
    description: &str,
) -> ProductRoleBinding {
    ProductRoleBinding {
        product_role,
        runtime_role,
        description: description.into(),
    }
}

fn alias(
    alias: &str,
    runtime_role: RuntimeAgentRole,
    reason: &str,
) -> crate::model::RoleAliasBinding {
    crate::model::RoleAliasBinding {
        alias: alias.into(),
        runtime_role,
        reason: reason.into(),
    }
}

fn capability(
    action_type: &str,
    mode: RoleCapabilityMode,
    object_type: Option<&str>,
    scope_kind: Option<ObjectScopeKind>,
    requires_handoff: bool,
    handoff_rule: Option<&str>,
    requires_human_approval: bool,
    required_evidence: &[&str],
) -> RoleActionCapability {
    RoleActionCapability {
        action_type: action_type.into(),
        mode,
        object_type: object_type.map(str::to_string),
        scope_kind,
        requires_handoff,
        handoff_rule: handoff_rule.map(str::to_string),
        requires_human_approval,
        required_evidence: required_evidence
            .iter()
            .map(|item| (*item).to_string())
            .collect(),
    }
}

fn scope(
    scope_id: &str,
    object_type: &str,
    scope_kind: ObjectScopeKind,
    description: &str,
) -> ObjectScope {
    ObjectScope {
        scope_id: scope_id.into(),
        object_type: object_type.into(),
        scope_kind,
        description: description.into(),
    }
}

fn tool_scope(
    allowed: &[ToolKind],
    forbidden: &[ToolKind],
    requires_evidence_capture: bool,
) -> RoleToolScope {
    RoleToolScope {
        allowed_tool_kinds: allowed.to_vec(),
        forbidden_tool_kinds: forbidden.to_vec(),
        requires_evidence_capture,
    }
}

fn handoff_rule(
    handoff_id: &str,
    from_role: RuntimeAgentRole,
    to_role: RuntimeAgentRole,
    target_object_type: &str,
    allowed_actions: &[&str],
    required_inputs: &[&str],
    expected_outputs: &[&str],
    boundary_notes: &[&str],
) -> HandoffRule {
    HandoffRule {
        handoff_id: handoff_id.into(),
        from_role,
        to_role,
        target_object_type: target_object_type.into(),
        allowed_actions: allowed_actions
            .iter()
            .map(|item| (*item).to_string())
            .collect(),
        required_inputs: required_inputs
            .iter()
            .map(|item| (*item).to_string())
            .collect(),
        expected_outputs: expected_outputs
            .iter()
            .map(|item| (*item).to_string())
            .collect(),
        boundary_notes: boundary_notes
            .iter()
            .map(|item| (*item).to_string())
            .collect(),
    }
}
