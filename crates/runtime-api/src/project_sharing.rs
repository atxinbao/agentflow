use serde::{Deserialize, Serialize};
use std::path::Path;

pub const PROJECT_SHARING_READ_MODEL_VERSION: &str = "agentflow-project-sharing-read-model.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSharingReadModel {
    pub version: String,
    pub status: String,
    pub project_id: String,
    pub title: String,
    pub product: ProjectSharingField,
    pub goal: ProjectSharingField,
    pub roadmap: ProjectSharingField,
    pub tasks: ProjectSharingTaskSummary,
    pub latest_decision: ProjectSharingField,
    pub latest_delivery: ProjectSharingField,
    pub feedback: ProjectSharingField,
    #[serde(default)]
    pub source_projection_refs: Vec<String>,
    #[serde(default)]
    pub blockers: Vec<String>,
    pub readonly: bool,
    pub authority: bool,
    pub projection_backed: bool,
    pub updated_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSharingField {
    pub label: String,
    pub status: String,
    pub summary: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSharingTaskSummary {
    pub status: String,
    pub total: usize,
    pub completed: usize,
    pub current: usize,
    pub future: usize,
    pub blocked: usize,
    pub summary: String,
    pub source_ref: String,
}

pub fn project_sharing_read_model(
    project_root: impl AsRef<Path>,
    project_id: &str,
) -> ProjectSharingReadModel {
    let project_root = project_root.as_ref();
    let workspace_projection = super::load_product_workspace_projection(project_root);
    let project_projection =
        match agentflow_projection::load_project_projection(project_root, project_id) {
            Ok(projection) => projection,
            Err(error) => {
                return ProjectSharingReadModel {
                    version: PROJECT_SHARING_READ_MODEL_VERSION.to_string(),
                    status: "invalid".to_string(),
                    project_id: project_id.to_string(),
                    title: project_id.to_string(),
                    product: field(
                        "Product",
                        "deferred",
                        "Project projection is missing.",
                        None,
                    ),
                    goal: field("Goal", "deferred", "Project projection is missing.", None),
                    roadmap: field(
                        "Roadmap",
                        "deferred",
                        "Project projection is missing.",
                        None,
                    ),
                    tasks: ProjectSharingTaskSummary {
                        status: "deferred".to_string(),
                        total: 0,
                        completed: 0,
                        current: 0,
                        future: 0,
                        blocked: 0,
                        summary: "Project projection is missing.".to_string(),
                        source_ref: project_projection_ref(project_id),
                    },
                    latest_decision: field(
                        "Latest decision",
                        "deferred",
                        "Project projection is missing.",
                        None,
                    ),
                    latest_delivery: field(
                        "Latest delivery",
                        "deferred",
                        "Project projection is missing.",
                        None,
                    ),
                    feedback: field(
                        "Feedback",
                        "deferred",
                        "Project projection is missing.",
                        None,
                    ),
                    source_projection_refs: vec![project_projection_ref(project_id)],
                    blockers: vec![format!("project projection unavailable: {error}")],
                    readonly: true,
                    authority: false,
                    projection_backed: false,
                    updated_at: 0,
                };
            }
        };

    let mut blockers = project_projection
        .blockers
        .iter()
        .map(|blocker| format!("{}: {}", blocker.issue_id, blocker.reason))
        .collect::<Vec<_>>();
    if !workspace_projection.blockers.is_empty() {
        blockers.extend(
            workspace_projection
                .blockers
                .iter()
                .map(|blocker| format!("workspace projection: {blocker}")),
        );
    }

    let status = if !project_projection.blockers.is_empty() {
        "invalid"
    } else if workspace_projection.blockers.is_empty() {
        "ready"
    } else {
        "deferred"
    };
    let product = workspace_projection
        .active_product
        .as_ref()
        .map(|product| {
            field(
                "Product",
                "ready",
                format!("{} ({})", product.name, product.product_id),
                Some(".agentflow/projections/workspace-state.json".to_string()),
            )
        })
        .unwrap_or_else(|| {
            field(
                "Product",
                "deferred",
                "Active product projection is not available.",
                Some(".agentflow/projections/workspace-state.json".to_string()),
            )
        });

    let completion = project_projection.completion.as_ref();
    let release = project_projection.release.as_ref();
    let external_review = project_projection.external_review.as_ref();
    let delivery = project_projection.delivery.as_ref();
    let decision_summary = completion
        .and_then(|completion| completion.latest_outcome.clone())
        .unwrap_or_else(|| project_projection.stage_summary.clone());
    let delivery_summary = delivery
        .map(|delivery| {
            if delivery.summary_line.is_empty() {
                format!(
                    "{} / ready {} / missing {}",
                    delivery.status, delivery.ready_count, delivery.missing_count
                )
            } else {
                delivery.summary_line.clone()
            }
        })
        .or_else(|| {
            release.map(|release| {
                if release.summary_line.is_empty() {
                    release.delivery_status.clone()
                } else {
                    release.summary_line.clone()
                }
            })
        })
        .unwrap_or_else(|| "No delivery projection yet.".to_string());

    ProjectSharingReadModel {
        version: PROJECT_SHARING_READ_MODEL_VERSION.to_string(),
        status: status.to_string(),
        project_id: project_projection.project_id.clone(),
        title: project_projection.title.clone(),
        product,
        goal: field(
            "Goal",
            &project_projection.project_brain.goal_status,
            project_projection.objective.clone(),
            Some(project_projection.project_brain.goal_path.clone()),
        ),
        roadmap: field(
            "Roadmap",
            &project_projection.project_brain.plan_status,
            project_projection
                .next_action_reason
                .clone()
                .if_empty(project_projection.stage_summary.clone()),
            Some(project_projection.project_brain.plan_path.clone()),
        ),
        tasks: ProjectSharingTaskSummary {
            status: project_projection.status.clone(),
            total: project_projection.issue_count,
            completed: project_projection.completed_issue_count,
            current: project_projection.lanes.current.len(),
            future: project_projection.lanes.future.len(),
            blocked: project_projection.lanes.blocked.len(),
            summary: format!(
                "{} / {} completed",
                project_projection.completed_issue_count, project_projection.issue_count
            ),
            source_ref: project_projection_ref(&project_projection.project_id),
        },
        latest_decision: field(
            "Latest decision",
            completion
                .map(|completion| completion.current_state.as_str())
                .unwrap_or("deferred"),
            decision_summary,
            Some(project_projection.project_brain.decisions_path.clone()),
        ),
        latest_delivery: field(
            "Latest delivery",
            delivery
                .map(|delivery| delivery.status.as_str())
                .or_else(|| release.map(|release| release.delivery_status.as_str()))
                .unwrap_or("deferred"),
            delivery_summary,
            delivery
                .and_then(|delivery| delivery.public_record_path.clone())
                .or_else(|| release.map(|release| release.release_notes_path.clone())),
        ),
        feedback: field(
            "Feedback",
            external_review
                .map(|review| review.review_status.as_str())
                .unwrap_or("deferred"),
            external_review
                .map(|review| review.summary_line.clone())
                .unwrap_or_else(|| "No team feedback projection yet.".to_string()),
            external_review.map(|review| review.handoff_path.clone()),
        ),
        source_projection_refs: vec![
            project_projection_ref(&project_projection.project_id),
            ".agentflow/projections/workspace-state.json".to_string(),
        ],
        blockers,
        readonly: true,
        authority: false,
        projection_backed: true,
        updated_at: project_projection.updated_at,
    }
}

fn field(
    label: &str,
    status: &str,
    summary: impl Into<String>,
    source_ref: Option<String>,
) -> ProjectSharingField {
    ProjectSharingField {
        label: label.to_string(),
        status: status.to_string(),
        summary: summary.into(),
        source_ref,
    }
}

fn project_projection_ref(project_id: &str) -> String {
    format!(".agentflow/projections/projects/{project_id}.json")
}

trait IfEmpty {
    fn if_empty(self, fallback: String) -> String;
}

impl IfEmpty for String {
    fn if_empty(self, fallback: String) -> String {
        if self.trim().is_empty() {
            fallback
        } else {
            self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agentflow_projection::{
        ProjectBrainProjection, ProjectIssueLanes, ProjectProjection, ProjectionDeliverySummary,
    };

    #[test]
    fn sharing_view_is_projection_backed_and_readonly() {
        let dir = tempfile::tempdir().unwrap();
        write_projection(
            dir.path(),
            ProjectProjection {
                version: agentflow_projection::PROJECT_PROJECTION_VERSION.to_string(),
                project_id: "project-sharing".to_string(),
                title: "Project Sharing".to_string(),
                objective: "Make project state shareable without authority reads.".to_string(),
                status: "in-progress".to_string(),
                stage_key: "active".to_string(),
                stage_label: "Active".to_string(),
                stage_summary: "Project is moving.".to_string(),
                issue_ids: vec!["AF-SHARE-001".to_string(), "AF-SHARE-002".to_string()],
                current_issue_id: Some("AF-SHARE-001".to_string()),
                lanes: ProjectIssueLanes {
                    current: vec!["AF-SHARE-001".to_string()],
                    past: vec!["AF-SHARE-000".to_string()],
                    future: vec!["AF-SHARE-002".to_string()],
                    blocked: Vec::new(),
                },
                next_action: "continue".to_string(),
                next_action_label: "Continue".to_string(),
                next_action_reason: "Continue current issue.".to_string(),
                blockers: Vec::new(),
                completion_hint: "not complete".to_string(),
                completion: None,
                release: None,
                external_review: None,
                delivery: Some(ProjectionDeliverySummary {
                    status: "ready".to_string(),
                    summary_line: "Latest delivery is ready.".to_string(),
                    public_record_path: Some("CHANGELOG.md".to_string()),
                    ..ProjectionDeliverySummary::default()
                }),
                audit: None,
                issue_count: 2,
                completed_issue_count: 1,
                project_brain: brain(),
                updated_at: 123,
            },
        );

        let view = project_sharing_read_model(dir.path(), "project-sharing");

        assert_eq!(view.status, "deferred");
        assert!(view.readonly);
        assert!(!view.authority);
        assert!(view.projection_backed);
        assert_eq!(view.tasks.total, 2);
        assert_eq!(view.tasks.completed, 1);
        assert_eq!(view.goal.status, "ready");
        assert_eq!(view.latest_delivery.status, "ready");
        assert!(view
            .source_projection_refs
            .iter()
            .any(|path| path.ends_with(".agentflow/projections/projects/project-sharing.json")));
    }

    #[test]
    fn missing_project_projection_is_explicitly_invalid() {
        let dir = tempfile::tempdir().unwrap();

        let view = project_sharing_read_model(dir.path(), "missing-project");

        assert_eq!(view.status, "invalid");
        assert!(!view.projection_backed);
        assert!(!view.blockers.is_empty());
        assert_eq!(view.tasks.status, "deferred");
    }

    fn write_projection(root: &Path, projection: ProjectProjection) {
        agentflow_projection::storage::write_project_projection(root, &projection)
            .expect("write project projection");
    }

    fn brain() -> ProjectBrainProjection {
        ProjectBrainProjection {
            project_path: "docs/project/README.md".to_string(),
            goal_path: "docs/project/goal.md".to_string(),
            plan_path: "docs/project/roadmap.md".to_string(),
            decisions_path: "docs/project/decisions.md".to_string(),
            health_path: "docs/project/health.md".to_string(),
            brain_status: "ready".to_string(),
            goal_status: "ready".to_string(),
            plan_status: "ready".to_string(),
            decision_status: "deferred".to_string(),
            health_status: "ready".to_string(),
            missing_documents: Vec::new(),
            open_questions: Vec::new(),
            next_recommended_action: "continue".to_string(),
            next_recommended_action_label: "Continue".to_string(),
            next_recommended_action_reason: "Continue current issue.".to_string(),
            readonly: true,
        }
    }
}
