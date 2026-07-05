use serde::{Deserialize, Serialize};
use std::path::Path;

pub const ROLE_PERMISSION_HANDOFF_VIEW_VERSION: &str = "agentflow-role-permission-handoff-view.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RolePermissionHandoffView {
    pub version: String,
    pub status: String,
    pub project_id: String,
    pub current_owner_role: Option<String>,
    pub current_owner_reason: String,
    #[serde(default)]
    pub roles: Vec<RolePermissionViewRole>,
    #[serde(default)]
    pub handoffs: Vec<RolePermissionHandoffState>,
    #[serde(default)]
    pub negative_fixtures: Vec<RolePermissionNegativeFixture>,
    #[serde(default)]
    pub source_projection_refs: Vec<String>,
    #[serde(default)]
    pub blockers: Vec<String>,
    pub readonly: bool,
    pub authority: bool,
    pub projection_backed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RolePermissionViewRole {
    pub role_id: String,
    pub label: String,
    #[serde(default)]
    pub can_read: Vec<String>,
    #[serde(default)]
    pub can_act: Vec<String>,
    pub current_owner: bool,
    pub owner_reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RolePermissionHandoffState {
    pub handoff_id: String,
    pub from_role: String,
    pub to_role: String,
    pub status: String,
    pub reason: String,
    #[serde(default)]
    pub required_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RolePermissionNegativeFixture {
    pub fixture_id: String,
    pub rejected_input: String,
    pub expected_status: String,
    pub reason: String,
}

pub fn role_permission_handoff_view(
    project_root: impl AsRef<Path>,
    project_id: &str,
) -> RolePermissionHandoffView {
    let project_root = project_root.as_ref();
    let project_projection =
        match agentflow_projection::load_project_projection(project_root, project_id) {
            Ok(projection) => projection,
            Err(error) => {
                return RolePermissionHandoffView {
                    version: ROLE_PERMISSION_HANDOFF_VIEW_VERSION.to_string(),
                    status: "invalid".to_string(),
                    project_id: project_id.to_string(),
                    current_owner_role: None,
                    current_owner_reason: "Project projection is missing.".to_string(),
                    roles: base_roles(None),
                    handoffs: base_handoffs("blocked", "Project projection is missing."),
                    negative_fixtures: negative_fixtures(),
                    source_projection_refs: vec![project_projection_ref(project_id)],
                    blockers: vec![format!("project projection unavailable: {error}")],
                    readonly: true,
                    authority: false,
                    projection_backed: false,
                };
            }
        };

    let mut blockers = project_projection
        .blockers
        .iter()
        .map(|blocker| format!("{}: {}", blocker.issue_id, blocker.reason))
        .collect::<Vec<_>>();
    let missing_owner = project_projection.current_issue_id.is_none()
        && project_projection.completed_issue_count < project_projection.issue_count;
    if missing_owner {
        blockers.push("project has remaining issues but no current owner issue".to_string());
    }

    let current_owner_role = if missing_owner {
        None
    } else if project_projection.issue_count > 0
        && project_projection.completed_issue_count == project_projection.issue_count
    {
        Some("human-owner".to_string())
    } else if project_projection.current_issue_id.is_some() {
        Some("build-agent".to_string())
    } else {
        Some("spec-agent".to_string())
    };
    let status = if blockers.is_empty() {
        "ready"
    } else {
        "invalid"
    };
    let owner_reason = owner_reason(&project_projection, current_owner_role.as_deref());

    RolePermissionHandoffView {
        version: ROLE_PERMISSION_HANDOFF_VIEW_VERSION.to_string(),
        status: status.to_string(),
        project_id: project_projection.project_id.clone(),
        current_owner_role: current_owner_role.clone(),
        current_owner_reason: owner_reason,
        roles: base_roles(current_owner_role.as_deref()),
        handoffs: handoffs_for_project(&project_projection, status),
        negative_fixtures: negative_fixtures(),
        source_projection_refs: vec![project_projection_ref(&project_projection.project_id)],
        blockers,
        readonly: true,
        authority: false,
        projection_backed: true,
    }
}

fn base_roles(current_owner_role: Option<&str>) -> Vec<RolePermissionViewRole> {
    [
        role(
            "spec-agent",
            "Spec Agent",
            vec![
                "project sharing",
                "confirmed requirements",
                "feedback input",
            ],
            vec!["preview requirement", "materialize spec"],
        ),
        role(
            "build-agent",
            "Build Agent",
            vec!["project sharing", "spec issue", "handoff", "task evidence"],
            vec!["start work loop", "write evidence", "submit delivery"],
        ),
        role(
            "audit-agent",
            "Audit Agent",
            vec!["project sharing", "delivery history", "decision history"],
            vec!["inspect delivery", "write audit report"],
        ),
        role(
            "review-agent",
            "Review Agent",
            vec!["project sharing", "handoff state", "delivery history"],
            vec!["record review feedback"],
        ),
        role(
            "human-owner",
            "Human Owner",
            vec!["project sharing", "all readonly team views"],
            vec!["accept delivery", "request changes", "confirm next loop"],
        ),
        role(
            "viewer",
            "Viewer",
            vec!["project sharing", "delivery history"],
            Vec::new(),
        ),
    ]
    .into_iter()
    .map(|mut role| {
        role.current_owner = current_owner_role == Some(role.role_id.as_str());
        role.owner_reason = if role.current_owner {
            "This role currently owns the next move.".to_string()
        } else {
            "Readonly or waiting for handoff.".to_string()
        };
        role
    })
    .collect()
}

fn role(
    role_id: &str,
    label: &str,
    can_read: Vec<&str>,
    can_act: Vec<&str>,
) -> RolePermissionViewRole {
    RolePermissionViewRole {
        role_id: role_id.to_string(),
        label: label.to_string(),
        can_read: can_read.into_iter().map(str::to_string).collect(),
        can_act: can_act.into_iter().map(str::to_string).collect(),
        current_owner: false,
        owner_reason: String::new(),
    }
}

fn handoffs_for_project(
    project: &agentflow_projection::ProjectProjection,
    status: &str,
) -> Vec<RolePermissionHandoffState> {
    if status == "invalid" {
        return base_handoffs(
            "blocked",
            "Project handoff cannot proceed until blockers clear.",
        );
    }
    let spec_to_build = if project.current_issue_id.is_some() {
        "ready"
    } else {
        "pending"
    };
    let build_to_audit = project
        .delivery
        .as_ref()
        .map(|delivery| delivery.status.as_str())
        .unwrap_or("pending");
    let audit_to_owner = project
        .audit
        .as_ref()
        .map(|audit| audit.status.as_str())
        .unwrap_or("pending");

    vec![
        handoff(
            "spec-to-build",
            "spec-agent",
            "build-agent",
            spec_to_build,
            "Confirmed task enters the work loop.",
            vec!["spec issue", "context refs"],
        ),
        handoff(
            "build-to-audit",
            "build-agent",
            "audit-agent",
            normalize_handoff_status(build_to_audit),
            "Delivery evidence decides whether audit can inspect.",
            vec!["delivery record", "evidence refs"],
        ),
        handoff(
            "audit-to-owner",
            "audit-agent",
            "human-owner",
            normalize_handoff_status(audit_to_owner),
            "Audit report returns to the human owner for decision.",
            vec!["audit report", "risk notes"],
        ),
        handoff(
            "owner-to-spec-feedback",
            "human-owner",
            "spec-agent",
            "pending",
            "Human feedback becomes new Spec input only after confirmation.",
            vec!["feedback note", "confirmation"],
        ),
    ]
}

fn base_handoffs(status: &str, reason: &str) -> Vec<RolePermissionHandoffState> {
    vec![
        handoff(
            "spec-to-build",
            "spec-agent",
            "build-agent",
            status,
            reason,
            vec!["spec issue", "context refs"],
        ),
        handoff(
            "build-to-audit",
            "build-agent",
            "audit-agent",
            status,
            reason,
            vec!["delivery record", "evidence refs"],
        ),
        handoff(
            "audit-to-owner",
            "audit-agent",
            "human-owner",
            status,
            reason,
            vec!["audit report", "risk notes"],
        ),
    ]
}

fn handoff(
    handoff_id: &str,
    from_role: &str,
    to_role: &str,
    status: &str,
    reason: &str,
    required_refs: Vec<&str>,
) -> RolePermissionHandoffState {
    RolePermissionHandoffState {
        handoff_id: handoff_id.to_string(),
        from_role: from_role.to_string(),
        to_role: to_role.to_string(),
        status: status.to_string(),
        reason: reason.to_string(),
        required_refs: required_refs.into_iter().map(str::to_string).collect(),
    }
}

fn normalize_handoff_status(status: &str) -> &str {
    match status {
        "ready" | "passed" | "accepted" | "published" => "ready",
        "blocked" | "failed" | "invalid" => "blocked",
        _ => "pending",
    }
}

fn owner_reason(
    project: &agentflow_projection::ProjectProjection,
    owner_role: Option<&str>,
) -> String {
    match owner_role {
        Some("human-owner") => {
            "All project issues are complete; owner decides the next loop.".to_string()
        }
        Some("build-agent") => format!(
            "Current issue {} is in the work loop.",
            project.current_issue_id.as_deref().unwrap_or("unknown")
        ),
        Some("spec-agent") => {
            "Project has no active issue yet; Spec prepares the next task.".to_string()
        }
        _ => "No current owner could be derived from projection.".to_string(),
    }
}

fn negative_fixtures() -> Vec<RolePermissionNegativeFixture> {
    vec![
        RolePermissionNegativeFixture {
            fixture_id: "invalid-role".to_string(),
            rejected_input: "unknown-agent".to_string(),
            expected_status: "invalid".to_string(),
            reason: "Role is not in the runtime role boundary.".to_string(),
        },
        RolePermissionNegativeFixture {
            fixture_id: "missing-owner".to_string(),
            rejected_input: "project has remaining issues but no current owner".to_string(),
            expected_status: "invalid".to_string(),
            reason: "A team-facing handoff view must expose who owns the next move.".to_string(),
        },
    ]
}

fn project_projection_ref(project_id: &str) -> String {
    format!(".agentflow/projections/projects/{project_id}.json")
}

#[cfg(test)]
mod tests {
    use super::*;
    use agentflow_projection::{
        ProjectBrainProjection, ProjectIssueLanes, ProjectProjection, ProjectionAuditSummary,
        ProjectionDeliverySummary,
    };

    #[test]
    fn view_exposes_current_owner_permissions_and_handoff_state() {
        let dir = tempfile::tempdir().unwrap();
        write_projection(dir.path(), base_project(Some("AF-ROLE-001")));

        let view = role_permission_handoff_view(dir.path(), "project-role");

        assert_eq!(view.status, "ready");
        assert!(view.readonly);
        assert!(!view.authority);
        assert_eq!(view.current_owner_role.as_deref(), Some("build-agent"));
        assert!(view
            .roles
            .iter()
            .any(|role| role.role_id == "viewer" && role.can_act.is_empty()));
        assert!(view
            .handoffs
            .iter()
            .any(|handoff| handoff.handoff_id == "spec-to-build" && handoff.status == "ready"));
    }

    #[test]
    fn missing_owner_is_invalid_and_negative_fixture_is_declared() {
        let dir = tempfile::tempdir().unwrap();
        write_projection(dir.path(), base_project(None));

        let view = role_permission_handoff_view(dir.path(), "project-role");

        assert_eq!(view.status, "invalid");
        assert!(view.current_owner_role.is_none());
        assert!(view
            .negative_fixtures
            .iter()
            .any(|fixture| fixture.fixture_id == "missing-owner"));
        assert!(!view.blockers.is_empty());
    }

    fn write_projection(root: &Path, projection: ProjectProjection) {
        agentflow_projection::storage::write_project_projection(root, &projection)
            .expect("write project projection");
    }

    fn base_project(current_issue_id: Option<&str>) -> ProjectProjection {
        ProjectProjection {
            version: agentflow_projection::PROJECT_PROJECTION_VERSION.to_string(),
            project_id: "project-role".to_string(),
            title: "Role Permission Project".to_string(),
            objective: "Expose role permission handoff view.".to_string(),
            status: "in-progress".to_string(),
            stage_key: "active".to_string(),
            stage_label: "Active".to_string(),
            stage_summary: "Current handoff is active.".to_string(),
            issue_ids: vec!["AF-ROLE-001".to_string(), "AF-ROLE-002".to_string()],
            current_issue_id: current_issue_id.map(str::to_string),
            lanes: ProjectIssueLanes {
                current: current_issue_id.into_iter().map(str::to_string).collect(),
                past: Vec::new(),
                future: vec!["AF-ROLE-002".to_string()],
                blocked: Vec::new(),
            },
            next_action: "continue".to_string(),
            next_action_label: "Continue".to_string(),
            next_action_reason: "Continue active handoff.".to_string(),
            blockers: Vec::new(),
            completion_hint: "not complete".to_string(),
            completion: None,
            release: None,
            external_review: None,
            delivery: Some(ProjectionDeliverySummary {
                status: "ready".to_string(),
                ..ProjectionDeliverySummary::default()
            }),
            audit: Some(ProjectionAuditSummary {
                status: "pending".to_string(),
                ..ProjectionAuditSummary::default()
            }),
            issue_count: 2,
            completed_issue_count: 0,
            project_brain: ProjectBrainProjection {
                project_path: "docs/project/README.md".to_string(),
                goal_path: "docs/project/goal.md".to_string(),
                plan_path: "docs/project/roadmap.md".to_string(),
                decisions_path: "docs/project/decisions.md".to_string(),
                health_path: "docs/project/health.md".to_string(),
                brain_status: "ready".to_string(),
                goal_status: "ready".to_string(),
                plan_status: "ready".to_string(),
                decision_status: "pending".to_string(),
                health_status: "ready".to_string(),
                missing_documents: Vec::new(),
                open_questions: Vec::new(),
                next_recommended_action: "continue".to_string(),
                next_recommended_action_label: "Continue".to_string(),
                next_recommended_action_reason: "Continue active handoff.".to_string(),
                readonly: true,
            },
            updated_at: 123,
        }
    }
}
