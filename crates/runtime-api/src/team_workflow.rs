use serde::{Deserialize, Serialize};

pub const TEAM_WORKFLOW_BOUNDARY_CONTRACT_VERSION: &str = "agentflow-team-workflow-boundary.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamWorkflowBoundaryContract {
    pub version: String,
    pub release: String,
    pub scope: String,
    pub authority_boundary: String,
    pub included_capabilities: Vec<TeamWorkflowCapability>,
    pub excluded_capabilities: Vec<String>,
    pub role_boundaries: Vec<TeamWorkflowRoleBoundary>,
    pub permission_views: Vec<TeamWorkflowPermissionView>,
    pub handoff_boundaries: Vec<TeamWorkflowHandoffBoundary>,
    pub feedback_boundaries: Vec<TeamWorkflowFeedbackBoundary>,
    pub delivery_history_boundaries: Vec<TeamWorkflowDeliveryHistoryBoundary>,
    pub reference_app_consumption: Vec<String>,
    pub core_neutrality_rules: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamWorkflowCapability {
    pub id: String,
    pub label: String,
    pub owner_plane: String,
    pub read_model: String,
    pub write_boundary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamWorkflowRoleBoundary {
    pub role: String,
    pub responsibilities: Vec<String>,
    pub cannot_do: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamWorkflowPermissionView {
    pub id: String,
    pub label: String,
    pub source: String,
    pub display_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamWorkflowHandoffBoundary {
    pub id: String,
    pub from_role: String,
    pub to_role: String,
    pub required_refs: Vec<String>,
    pub product_neutral: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamWorkflowFeedbackBoundary {
    pub id: String,
    pub source: String,
    pub target_loop: String,
    pub authority_rule: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamWorkflowDeliveryHistoryBoundary {
    pub id: String,
    pub source: String,
    pub visible_to: Vec<String>,
    pub write_boundary: String,
}

pub fn team_workflow_boundary_contract() -> TeamWorkflowBoundaryContract {
    TeamWorkflowBoundaryContract {
        version: TEAM_WORKFLOW_BOUNDARY_CONTRACT_VERSION.to_string(),
        release: "v1.2.1".to_string(),
        scope: "local-lightweight-team-workflow".to_string(),
        authority_boundary:
            "Team workflow reads Product / Projection facts and writes through Runtime commands; it does not create Core or cloud authority."
                .to_string(),
        included_capabilities: vec![
            capability(
                "project-sharing",
                "项目共享视图",
                "projection",
                "project-sharing-read-model",
                "Runtime command / local workspace fact",
            ),
            capability(
                "role-permission-handoff",
                "角色、权限和交接视图",
                "projection",
                "role-permission-handoff-view",
                "Runtime command / local workspace fact",
            ),
            capability(
                "team-feedback",
                "团队反馈进入 Feedback Loop",
                "runtime",
                "feedback-loop-read-model",
                "Runtime command",
            ),
            capability(
                "delivery-decision-history",
                "团队可读交付和决策历史",
                "projection",
                "delivery-decision-history-read-model",
                "Task / decision / delivery facts",
            ),
        ],
        excluded_capabilities: vec![
            "cloud multi-tenant workspace".to_string(),
            "payment or entitlement management".to_string(),
            "public commercial launch".to_string(),
            "external account administration".to_string(),
            "industry-specific Core authority".to_string(),
        ],
        role_boundaries: vec![
            role(
                "human-owner",
                vec!["确认共享边界", "确认交付是否可接受", "决定是否进入下一轮"],
                vec!["绕过 Runtime 直接写入任务事实"],
            ),
            role(
                "spec-agent",
                vec!["整理团队输入", "生成 preview", "等待确认后 materialize"],
                vec!["直接执行工作任务", "代替 Human Owner 接受交付"],
            ),
            role(
                "build-agent",
                vec!["消费已确认任务", "生成证据", "写回交付事实"],
                vec!["修改产品层共享策略", "跳过验收门"],
            ),
            role(
                "audit-agent",
                vec!["读取交付和决策历史", "生成审计报告"],
                vec!["自动修改执行结果", "替代交付事实源"],
            ),
        ],
        permission_views: vec![
            permission(
                "local-workspace-access",
                "本地工作区访问",
                "workspace projection",
                true,
            ),
            permission(
                "handoff-allowed",
                "允许交接",
                "runtime role contract",
                true,
            ),
            permission(
                "delivery-visible",
                "交付可见",
                "delivery / decision projection",
                true,
            ),
        ],
        handoff_boundaries: vec![
            handoff(
                "spec-to-build",
                "spec-agent",
                "build-agent",
                vec!["confirmed requirement", "spec issue", "context refs"],
            ),
            handoff(
                "build-to-audit",
                "build-agent",
                "audit-agent",
                vec!["delivery record", "decision record", "evidence refs"],
            ),
            handoff(
                "audit-to-human",
                "audit-agent",
                "human-owner",
                vec!["audit report", "risk notes", "recommended decision"],
            ),
        ],
        feedback_boundaries: vec![TeamWorkflowFeedbackBoundary {
            id: "team-feedback-to-spec-loop".to_string(),
            source: "human-owner / team reviewer".to_string(),
            target_loop: "feedback-loop".to_string(),
            authority_rule: "Feedback is input, not authority, until preview and confirmation materialize it."
                .to_string(),
        }],
        delivery_history_boundaries: vec![TeamWorkflowDeliveryHistoryBoundary {
            id: "team-readable-delivery-history".to_string(),
            source: "task evidence + acceptance decision + delivery record".to_string(),
            visible_to: vec!["human-owner".to_string(), "team reviewer".to_string()],
            write_boundary: "Runtime writes facts; Desktop renders readonly history.".to_string(),
        }],
        reference_app_consumption: vec![
            "Software Dev Reference App may render these views as examples.".to_string(),
            "Reference App behavior must not become Core authority.".to_string(),
        ],
        core_neutrality_rules: vec![
            "The contract is Product / Projection level.".to_string(),
            "Core does not store users, billing, external accounts, or tenant policy.".to_string(),
            "Industry products consume the contract through Runtime API read models.".to_string(),
        ],
    }
}

fn capability(
    id: &str,
    label: &str,
    owner_plane: &str,
    read_model: &str,
    write_boundary: &str,
) -> TeamWorkflowCapability {
    TeamWorkflowCapability {
        id: id.to_string(),
        label: label.to_string(),
        owner_plane: owner_plane.to_string(),
        read_model: read_model.to_string(),
        write_boundary: write_boundary.to_string(),
    }
}

fn role(role: &str, responsibilities: Vec<&str>, cannot_do: Vec<&str>) -> TeamWorkflowRoleBoundary {
    TeamWorkflowRoleBoundary {
        role: role.to_string(),
        responsibilities: responsibilities.into_iter().map(str::to_string).collect(),
        cannot_do: cannot_do.into_iter().map(str::to_string).collect(),
    }
}

fn permission(
    id: &str,
    label: &str,
    source: &str,
    display_only: bool,
) -> TeamWorkflowPermissionView {
    TeamWorkflowPermissionView {
        id: id.to_string(),
        label: label.to_string(),
        source: source.to_string(),
        display_only,
    }
}

fn handoff(
    id: &str,
    from_role: &str,
    to_role: &str,
    required_refs: Vec<&str>,
) -> TeamWorkflowHandoffBoundary {
    TeamWorkflowHandoffBoundary {
        id: id.to_string(),
        from_role: from_role.to_string(),
        to_role: to_role.to_string(),
        required_refs: required_refs.into_iter().map(str::to_string).collect(),
        product_neutral: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contract_defines_local_team_boundary_without_cloud_scope() {
        let contract = team_workflow_boundary_contract();

        assert_eq!(contract.version, TEAM_WORKFLOW_BOUNDARY_CONTRACT_VERSION);
        assert_eq!(contract.scope, "local-lightweight-team-workflow");
        assert!(contract
            .included_capabilities
            .iter()
            .any(|capability| capability.id == "project-sharing"));
        assert!(contract
            .excluded_capabilities
            .iter()
            .any(|capability| capability.contains("cloud multi-tenant")));
        assert!(contract
            .core_neutrality_rules
            .iter()
            .any(|rule| rule.contains("Product / Projection")));
    }

    #[test]
    fn contract_exposes_product_neutral_handoff_and_display_only_permissions() {
        let contract = team_workflow_boundary_contract();

        assert!(contract
            .handoff_boundaries
            .iter()
            .all(|handoff| handoff.product_neutral));
        assert!(contract
            .permission_views
            .iter()
            .all(|permission| permission.display_only));
        assert!(contract
            .role_boundaries
            .iter()
            .any(|role| role.role == "build-agent"
                && role.cannot_do.iter().any(|item| item.contains("验收门"))));
    }
}
