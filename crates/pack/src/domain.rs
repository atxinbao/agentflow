use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const PACK_DOMAIN_VERSION: &str = "agentflow-pack-domain.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackDomainDefinition {
    pub version: String,
    pub pack_id: String,
    pub domain_id: String,
    pub object_types: Vec<DomainObjectType>,
    pub link_types: Vec<DomainLinkType>,
    pub state_machines: Vec<DomainStateMachine>,
    pub action_semantics: Vec<DomainActionSemantic>,
    pub acceptance_semantics: Vec<DomainAcceptanceSemantic>,
    pub evidence_policy: DomainEvidencePolicy,
    #[serde(default)]
    pub audit_trigger_hints: Vec<DomainAuditTriggerHint>,
    pub migration_compatibility: DomainMigrationCompatibility,
    pub writes_events: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DomainObjectType {
    pub object_type_id: String,
    pub label: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DomainLinkType {
    pub link_type_id: String,
    pub from_object_type: String,
    pub to_object_type: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DomainStateMachine {
    pub object_type: String,
    pub states: Vec<String>,
    pub transitions: Vec<DomainStateTransition>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DomainStateTransition {
    pub from: String,
    pub to: String,
    pub action_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DomainActionSemantic {
    pub action_type: String,
    pub target_object_type: String,
    pub description: String,
    #[serde(default)]
    pub allowed_roles: Vec<String>,
    pub contract_ref: String,
    pub arbitration_ref: String,
    pub simulation_ref: String,
    #[serde(default)]
    pub required_evidence: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DomainAcceptanceSemantic {
    pub acceptance_id: String,
    pub object_type: String,
    pub description: String,
    #[serde(default)]
    pub required_evidence: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DomainEvidencePolicy {
    pub policy_id: String,
    #[serde(default)]
    pub required_evidence_kinds: Vec<String>,
    pub missing_evidence_behavior: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DomainAuditTriggerHint {
    pub hint_id: String,
    pub object_type: String,
    pub condition: String,
    pub sidecar_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DomainMigrationCompatibility {
    pub compatible_with_runtime: String,
    pub migration_policy_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackDomainValidationIssue {
    pub field: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackDomainValidationReport {
    pub version: String,
    pub pack_id: String,
    pub domain_id: String,
    pub valid: bool,
    pub writes_events: bool,
    pub issues: Vec<PackDomainValidationIssue>,
}

pub fn validate_domain_definition(domain: &PackDomainDefinition) -> PackDomainValidationReport {
    let mut issues = Vec::new();
    require_non_empty(&mut issues, "version", &domain.version);
    if domain.version != PACK_DOMAIN_VERSION {
        issues.push(issue("version", "must be agentflow-pack-domain.v1"));
    }
    require_non_empty(&mut issues, "packId", &domain.pack_id);
    require_non_empty(&mut issues, "domainId", &domain.domain_id);
    if domain.writes_events {
        issues.push(issue(
            "writesEvents",
            "domain pack definitions must not write runtime events",
        ));
    }
    if domain.object_types.is_empty() {
        issues.push(issue(
            "objectTypes",
            "must contain at least one object type",
        ));
    }
    if domain.action_semantics.is_empty() {
        issues.push(issue(
            "actionSemantics",
            "must contain executable action semantics",
        ));
    }

    let object_type_ids = domain
        .object_types
        .iter()
        .map(|object| object.object_type_id.as_str())
        .collect::<BTreeSet<_>>();
    for object in &domain.object_types {
        require_non_empty(
            &mut issues,
            "objectTypes.objectTypeId",
            &object.object_type_id,
        );
        require_non_empty(&mut issues, "objectTypes.label", &object.label);
    }
    for link in &domain.link_types {
        if !object_type_ids.contains(link.from_object_type.as_str()) {
            issues.push(issue(
                "linkTypes.fromObjectType",
                "must reference a declared object type",
            ));
        }
        if !object_type_ids.contains(link.to_object_type.as_str()) {
            issues.push(issue(
                "linkTypes.toObjectType",
                "must reference a declared object type",
            ));
        }
    }
    for machine in &domain.state_machines {
        if !object_type_ids.contains(machine.object_type.as_str()) {
            issues.push(issue(
                "stateMachines.objectType",
                "must reference a declared object type",
            ));
        }
        if machine.states.is_empty() {
            issues.push(issue("stateMachines.states", "must not be empty"));
        }
    }
    for action in &domain.action_semantics {
        if !object_type_ids.contains(action.target_object_type.as_str()) {
            issues.push(issue(
                "actionSemantics.targetObjectType",
                "must reference a declared object type",
            ));
        }
        require_non_empty(
            &mut issues,
            "actionSemantics.contractRef",
            &action.contract_ref,
        );
        require_non_empty(
            &mut issues,
            "actionSemantics.arbitrationRef",
            &action.arbitration_ref,
        );
        require_non_empty(
            &mut issues,
            "actionSemantics.simulationRef",
            &action.simulation_ref,
        );
    }
    for acceptance in &domain.acceptance_semantics {
        if !object_type_ids.contains(acceptance.object_type.as_str()) {
            issues.push(issue(
                "acceptanceSemantics.objectType",
                "must reference a declared object type",
            ));
        }
    }

    PackDomainValidationReport {
        version: "agentflow-pack-domain-validation.v1".to_string(),
        pack_id: domain.pack_id.clone(),
        domain_id: domain.domain_id.clone(),
        valid: issues.is_empty(),
        writes_events: domain.writes_events,
        issues,
    }
}

pub fn software_dev_domain_definition() -> PackDomainDefinition {
    PackDomainDefinition {
        version: PACK_DOMAIN_VERSION.to_string(),
        pack_id: "software-dev".to_string(),
        domain_id: "software-dev-domain".to_string(),
        object_types: objects(&[
            ("Requirement", "公开需求文档"),
            ("Spec", "已确认规格"),
            ("Issue", "可执行任务"),
            ("Run", "工作循环运行"),
            ("Acceptance", "验收结论"),
            ("Delivery", "交付记录"),
            ("PullRequest", "PR/MR 交付入口"),
            ("Release", "版本发布"),
            ("Evidence", "验证证据"),
            ("Audit", "独立审计"),
            ("Finding", "审计发现"),
            ("FollowUpProposal", "后续修复提案"),
        ]),
        link_types: vec![
            link("requirement-produces-spec", "Requirement", "Spec"),
            link("spec-produces-issue", "Spec", "Issue"),
            link("issue-produces-run", "Issue", "Run"),
            link("run-produces-acceptance", "Run", "Acceptance"),
            link("acceptance-produces-delivery", "Acceptance", "Delivery"),
            link("run-produces-evidence", "Run", "Evidence"),
            link("run-produces-pr", "Run", "PullRequest"),
            link("delivery-includes-pr", "Delivery", "PullRequest"),
            link("release-includes-delivery", "Release", "Delivery"),
            link("delivery-can-request-audit", "Delivery", "Audit"),
            link("audit-produces-finding", "Audit", "Finding"),
            link(
                "finding-produces-follow-up-proposal",
                "Finding",
                "FollowUpProposal",
            ),
        ],
        state_machines: vec![
            DomainStateMachine {
                object_type: "Issue".to_string(),
                states: vec![
                    "backlog".to_string(),
                    "todo".to_string(),
                    "in_progress".to_string(),
                    "in_review".to_string(),
                    "done".to_string(),
                    "blocked".to_string(),
                    "cancel".to_string(),
                ],
                transitions: vec![
                    transition("backlog", "todo", "activateIssue"),
                    transition("todo", "in_progress", "startRun"),
                    transition("in_progress", "in_review", "runValidation"),
                    transition("in_review", "done", "markIssueDone"),
                ],
            },
            DomainStateMachine {
                object_type: "Audit".to_string(),
                states: vec![
                    "requested".to_string(),
                    "in_progress".to_string(),
                    "reported".to_string(),
                    "closed".to_string(),
                ],
                transitions: vec![
                    transition("requested", "in_progress", "requestAudit"),
                    transition("in_progress", "reported", "createFinding"),
                    transition("reported", "closed", "linkFixIssue"),
                ],
            },
        ],
        action_semantics: vec![
            action("submitRequirement", "Requirement"),
            action("approveSpec", "Spec"),
            action("createIssue", "Issue"),
            action("activateIssue", "Issue"),
            action("startRun", "Issue"),
            action("runValidation", "Run"),
            action("prepareDelivery", "Run"),
            action("markIssueDone", "Issue"),
            action("requestAudit", "Issue"),
            action("createFinding", "Audit"),
            action("linkFixIssue", "Finding"),
        ],
        acceptance_semantics: vec![DomainAcceptanceSemantic {
            acceptance_id: "software-dev.issue.done".to_string(),
            object_type: "Issue".to_string(),
            description: "任务完成必须有本地验证证据、PR/MR 记录和 Done 写回。".to_string(),
            required_evidence: vec![
                "validation".to_string(),
                "pull-request".to_string(),
                "completion-commit".to_string(),
            ],
        }],
        evidence_policy: DomainEvidencePolicy {
            policy_id: "software-dev.evidence.required".to_string(),
            required_evidence_kinds: vec!["validation".to_string(), "delivery".to_string()],
            missing_evidence_behavior: "reject-completion".to_string(),
        },
        audit_trigger_hints: vec![DomainAuditTriggerHint {
            hint_id: "software-dev.audit.sidecar".to_string(),
            object_type: "Release".to_string(),
            condition: "explicit-human-or-release-rule".to_string(),
            sidecar_only: true,
        }],
        migration_compatibility: DomainMigrationCompatibility {
            compatible_with_runtime: ">=0.8.0".to_string(),
            migration_policy_ref: "pack.migration.preview-only".to_string(),
        },
        writes_events: false,
    }
}

pub fn ui_design_domain_definition() -> PackDomainDefinition {
    PackDomainDefinition {
        version: PACK_DOMAIN_VERSION.to_string(),
        pack_id: "ui-design".to_string(),
        domain_id: "ui-design-domain".to_string(),
        object_types: objects(&[
            ("ProductBrief", "产品简报"),
            ("Prd", "产品需求文档"),
            ("Direction", "设计方向"),
            ("Wireframe", "线框稿"),
            ("HiFi", "高保真设计"),
            ("DesignSystem", "设计系统"),
            ("Page", "页面"),
            ("Handoff", "设计交接"),
            ("Evidence", "设计验证证据"),
        ]),
        link_types: vec![
            link("brief-produces-prd", "ProductBrief", "Prd"),
            link("prd-produces-direction", "Prd", "Direction"),
            link("direction-produces-wireframe", "Direction", "Wireframe"),
            link("wireframe-produces-hifi", "Wireframe", "HiFi"),
            link("hifi-produces-handoff", "HiFi", "Handoff"),
        ],
        state_machines: vec![DomainStateMachine {
            object_type: "Handoff".to_string(),
            states: vec![
                "draft".to_string(),
                "review".to_string(),
                "accepted".to_string(),
                "blocked".to_string(),
            ],
            transitions: vec![
                transition("draft", "review", "design.request-review"),
                transition("review", "accepted", "design.accept-handoff"),
            ],
        }],
        action_semantics: vec![
            action("design.generate-wireframe", "Wireframe"),
            action("design.generate-hifi", "HiFi"),
            action("design.accept-handoff", "Handoff"),
        ],
        acceptance_semantics: vec![DomainAcceptanceSemantic {
            acceptance_id: "ui-design.handoff.accepted".to_string(),
            object_type: "Handoff".to_string(),
            description: "设计交付必须包含页面、设计系统引用和可追溯证据。".to_string(),
            required_evidence: vec!["design-preview".to_string(), "handoff".to_string()],
        }],
        evidence_policy: DomainEvidencePolicy {
            policy_id: "ui-design.evidence.required".to_string(),
            required_evidence_kinds: vec!["visual-preview".to_string(), "handoff".to_string()],
            missing_evidence_behavior: "reject-handoff".to_string(),
        },
        audit_trigger_hints: Vec::new(),
        migration_compatibility: DomainMigrationCompatibility {
            compatible_with_runtime: ">=0.8.0".to_string(),
            migration_policy_ref: "pack.migration.preview-only".to_string(),
        },
        writes_events: false,
    }
}

fn objects(values: &[(&str, &str)]) -> Vec<DomainObjectType> {
    values
        .iter()
        .map(|(object_type_id, description)| DomainObjectType {
            object_type_id: (*object_type_id).to_string(),
            label: (*object_type_id).to_string(),
            description: (*description).to_string(),
        })
        .collect()
}

fn link(link_type_id: &str, from_object_type: &str, to_object_type: &str) -> DomainLinkType {
    DomainLinkType {
        link_type_id: link_type_id.to_string(),
        from_object_type: from_object_type.to_string(),
        to_object_type: to_object_type.to_string(),
        description: format!("{from_object_type} -> {to_object_type}"),
    }
}

fn transition(from: &str, to: &str, action_type: &str) -> DomainStateTransition {
    DomainStateTransition {
        from: from.to_string(),
        to: to.to_string(),
        action_type: action_type.to_string(),
    }
}

fn action(action_type: &str, target_object_type: &str) -> DomainActionSemantic {
    DomainActionSemantic {
        action_type: action_type.to_string(),
        target_object_type: target_object_type.to_string(),
        description: format!("{action_type} on {target_object_type}"),
        allowed_roles: vec!["WorkAgent".to_string()],
        contract_ref: format!("action-contract:{action_type}"),
        arbitration_ref: format!("action-arbitration:{action_type}"),
        simulation_ref: format!("simulation:{action_type}"),
        required_evidence: vec!["validation".to_string()],
    }
}

fn require_non_empty(issues: &mut Vec<PackDomainValidationIssue>, field: &str, value: &str) {
    if value.trim().is_empty() {
        issues.push(issue(field, "must not be empty"));
    }
}

fn issue(field: &str, reason: &str) -> PackDomainValidationIssue {
    PackDomainValidationIssue {
        field: field.to_string(),
        reason: reason.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        software_dev_domain_definition, ui_design_domain_definition, validate_domain_definition,
    };

    #[test]
    fn domain_pack_cannot_write_events() {
        let mut domain = software_dev_domain_definition();
        domain.writes_events = true;

        let report = validate_domain_definition(&domain);

        assert!(!report.valid);
        assert!(report
            .issues
            .iter()
            .any(|issue| issue.field == "writesEvents"));
    }

    #[test]
    fn built_in_domains_express_software_and_design_differences() {
        let software = software_dev_domain_definition();
        let design = ui_design_domain_definition();

        assert!(validate_domain_definition(&software).valid);
        assert!(validate_domain_definition(&design).valid);
        assert!(software
            .object_types
            .iter()
            .any(|object| object.object_type_id == "Issue"));
        assert!(software
            .object_types
            .iter()
            .any(|object| object.object_type_id == "Acceptance"));
        assert!(software
            .object_types
            .iter()
            .any(|object| object.object_type_id == "Delivery"));
        assert!(software
            .link_types
            .iter()
            .any(|link| link.link_type_id == "finding-produces-follow-up-proposal"));
        assert!(!software
            .object_types
            .iter()
            .any(|object| object.object_type_id == "Wireframe"));
        assert!(design
            .object_types
            .iter()
            .any(|object| object.object_type_id == "Wireframe"));
        assert!(!design
            .object_types
            .iter()
            .any(|object| object.object_type_id == "Issue"));
    }

    #[test]
    fn action_semantics_are_readable_by_contract_arbitration_and_simulation() {
        let domain = software_dev_domain_definition();

        for action in domain.action_semantics {
            assert!(action.contract_ref.starts_with("action-contract:"));
            assert!(action.arbitration_ref.starts_with("action-arbitration:"));
            assert!(action.simulation_ref.starts_with("simulation:"));
        }
    }
}
