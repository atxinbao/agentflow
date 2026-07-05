use serde::{Deserialize, Serialize};
use std::path::Path;

pub const TEAM_DELIVERY_DECISION_HISTORY_VERSION: &str =
    "agentflow-team-delivery-decision-history.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamDeliveryDecisionHistoryView {
    pub version: String,
    pub status: String,
    pub project_id: String,
    pub title: String,
    #[serde(default)]
    pub entries: Vec<TeamHistoryEntry>,
    pub latest_decision: TeamHistorySummary,
    pub latest_delivery: TeamHistorySummary,
    pub feedback: TeamFeedbackHook,
    pub audit_sidecar: TeamAuditSidecar,
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
pub struct TeamHistoryEntry {
    pub entry_id: String,
    pub entry_kind: String,
    pub issue_id: Option<String>,
    pub status: String,
    pub outcome: Option<String>,
    pub summary: String,
    #[serde(default)]
    pub reasons: Vec<String>,
    #[serde(default)]
    pub remediations: Vec<String>,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
    #[serde(default)]
    pub delivery_refs: Vec<String>,
    pub actor_role: String,
    pub feedback_route: Option<String>,
    pub updated_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamHistorySummary {
    pub status: String,
    pub summary: String,
    pub source_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamFeedbackHook {
    pub status: String,
    pub route: String,
    pub summary: String,
    #[serde(default)]
    pub required_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamAuditSidecar {
    pub status: String,
    pub summary: String,
    pub blocking: bool,
    pub source_ref: Option<String>,
}

pub fn team_delivery_decision_history_view(
    project_root: impl AsRef<Path>,
    project_id: &str,
) -> TeamDeliveryDecisionHistoryView {
    let project_root = project_root.as_ref();
    let project_projection =
        match agentflow_projection::load_project_projection(project_root, project_id) {
            Ok(projection) => projection,
            Err(error) => {
                return TeamDeliveryDecisionHistoryView {
                    version: TEAM_DELIVERY_DECISION_HISTORY_VERSION.to_string(),
                    status: "invalid".to_string(),
                    project_id: project_id.to_string(),
                    title: project_id.to_string(),
                    entries: Vec::new(),
                    latest_decision: summary(
                        "deferred",
                        "Project projection is missing.",
                        Some(project_projection_ref(project_id)),
                    ),
                    latest_delivery: summary(
                        "deferred",
                        "Project projection is missing.",
                        Some(project_projection_ref(project_id)),
                    ),
                    feedback: feedback_hook(
                        "blocked",
                        "Project projection is missing, so feedback cannot be routed.",
                    ),
                    audit_sidecar: audit_sidecar(
                        "not-requested",
                        "Audit remains optional and separate from delivery history.",
                        false,
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
    let mut source_refs = vec![project_projection_ref(&project_projection.project_id)];
    let mut entries = Vec::new();
    let mut task_projection_missing = 0usize;

    for issue_id in &project_projection.issue_ids {
        match agentflow_projection::load_task_projection(project_root, issue_id) {
            Ok(task) => {
                source_refs.push(task_projection_ref(issue_id));
                entries.extend(entries_for_task(&task));
            }
            Err(error) => {
                task_projection_missing += 1;
                blockers.push(format!(
                    "task projection unavailable for {issue_id}: {error}"
                ));
            }
        }
    }

    if let Some(completion) = project_projection.completion.as_ref() {
        entries.push(project_completion_entry(&project_projection, completion));
    }

    entries.sort_by(|left, right| {
        right
            .updated_at
            .cmp(&left.updated_at)
            .then_with(|| left.entry_id.cmp(&right.entry_id))
    });

    let latest_decision = entries
        .iter()
        .find(|entry| entry.entry_kind == "decision")
        .map(|entry| {
            summary(
                &entry.status,
                entry.summary.clone(),
                entry.feedback_route.clone(),
            )
        })
        .unwrap_or_else(|| {
            summary(
                "deferred",
                "No decision history is available yet.",
                Some(project_projection.project_brain.decisions_path.clone()),
            )
        });

    let latest_delivery = entries
        .iter()
        .find(|entry| entry.entry_kind == "delivery")
        .map(|entry| {
            summary(
                &entry.status,
                entry.summary.clone(),
                entry.delivery_refs.first().cloned(),
            )
        })
        .unwrap_or_else(|| {
            let delivery = project_projection.delivery.as_ref();
            summary(
                delivery
                    .map(|delivery| delivery.status.as_str())
                    .unwrap_or("deferred"),
                delivery
                    .map(|delivery| delivery.summary_line.clone())
                    .filter(|line| !line.trim().is_empty())
                    .unwrap_or_else(|| "No delivery history is available yet.".to_string()),
                delivery.and_then(|delivery| delivery.public_record_path.clone()),
            )
        });

    let audit_sidecar = project_projection
        .audit
        .as_ref()
        .map(|audit| {
            audit_sidecar(
                &audit.status,
                non_empty_or(
                    audit.summary_line.clone(),
                    "Audit is optional and does not block default delivery history.",
                ),
                false,
                audit.report_path.clone(),
            )
        })
        .unwrap_or_else(|| {
            audit_sidecar(
                "not-requested",
                "Audit sidecar is not requested by default.",
                false,
                None,
            )
        });

    let status = if !project_projection.blockers.is_empty() {
        "invalid"
    } else if task_projection_missing > 0 {
        "deferred"
    } else {
        "ready"
    };

    TeamDeliveryDecisionHistoryView {
        version: TEAM_DELIVERY_DECISION_HISTORY_VERSION.to_string(),
        status: status.to_string(),
        project_id: project_projection.project_id.clone(),
        title: project_projection.title.clone(),
        entries,
        latest_decision,
        latest_delivery,
        feedback: feedback_hook(
            if status == "invalid" {
                "blocked"
            } else {
                "ready"
            },
            "Team feedback can enter the next Spec Loop only after confirmation.",
        ),
        audit_sidecar,
        source_projection_refs: source_refs,
        blockers,
        readonly: true,
        authority: false,
        projection_backed: true,
        updated_at: project_projection.updated_at,
    }
}

fn entries_for_task(task: &agentflow_projection::TaskProjection) -> Vec<TeamHistoryEntry> {
    let mut entries = Vec::new();
    if let Some(acceptance) = task.acceptance.as_ref() {
        entries.push(TeamHistoryEntry {
            entry_id: format!("decision:{}", task.issue_id),
            entry_kind: "decision".to_string(),
            issue_id: Some(task.issue_id.clone()),
            status: if acceptance.passed {
                "accepted".to_string()
            } else {
                "rejected".to_string()
            },
            outcome: Some(acceptance.outcome.clone()),
            summary: non_empty_or(
                acceptance.summary.clone(),
                "Acceptance decision recorded for this task.",
            ),
            reasons: if acceptance.failure_reasons.is_empty() {
                vec!["Decision is backed by acceptance gate evidence.".to_string()]
            } else {
                acceptance.failure_reasons.clone()
            },
            remediations: acceptance.next_steps.clone(),
            evidence_refs: vec![
                acceptance.traceability.acceptance_decision_path.clone(),
                acceptance.traceability.evidence_path.clone(),
                acceptance.traceability.validation_path.clone(),
                acceptance.traceability.closeout_proof_path.clone(),
            ],
            delivery_refs: acceptance.traceability.pr_url.iter().cloned().collect(),
            actor_role: "review-agent".to_string(),
            feedback_route: Some("feedback-loop/spec-evolution".to_string()),
            updated_at: acceptance.checked_at,
        });
    }

    if task.delivery.status != "missing" || !task.delivery.summary_line.trim().is_empty() {
        entries.push(TeamHistoryEntry {
            entry_id: format!("delivery:{}", task.issue_id),
            entry_kind: "delivery".to_string(),
            issue_id: Some(task.issue_id.clone()),
            status: task.delivery.status.clone(),
            outcome: Some(task.current_state.clone()),
            summary: non_empty_or(
                task.delivery.summary_line.clone(),
                "Delivery record is available for this task.",
            ),
            reasons: delivery_reasons(&task.delivery),
            remediations: task.delivery.missing_public_records.clone(),
            evidence_refs: optional_vec(task.delivery.evidence_path.clone()),
            delivery_refs: delivery_refs(task),
            actor_role: "build-agent".to_string(),
            feedback_route: Some("feedback-loop/spec-evolution".to_string()),
            updated_at: task.updated_at,
        });
    }

    if task.audit.status != "not-requested" {
        entries.push(TeamHistoryEntry {
            entry_id: format!("audit-sidecar:{}", task.issue_id),
            entry_kind: "audit-sidecar".to_string(),
            issue_id: Some(task.issue_id.clone()),
            status: task.audit.status.clone(),
            outcome: task.audit.latest_audit_id.clone(),
            summary: non_empty_or(
                task.audit.summary_line.clone(),
                "Audit sidecar is available for this task.",
            ),
            reasons: task.audit.findings.clone(),
            remediations: task.audit.repair_recommendations.clone(),
            evidence_refs: task.audit.evidence_gaps.clone(),
            delivery_refs: optional_vec(task.audit.report_path.clone()),
            actor_role: "audit-agent".to_string(),
            feedback_route: Some("audit-sidecar".to_string()),
            updated_at: task.audit.requested_at.unwrap_or(task.updated_at),
        });
    }

    entries
}

fn project_completion_entry(
    project: &agentflow_projection::ProjectProjection,
    completion: &agentflow_projection::ProjectCompletionProjection,
) -> TeamHistoryEntry {
    TeamHistoryEntry {
        entry_id: format!("decision:project:{}", project.project_id),
        entry_kind: "decision".to_string(),
        issue_id: None,
        status: completion.current_state.clone(),
        outcome: completion.latest_outcome.clone(),
        summary: completion.next_recommended_action_reason.clone(),
        reasons: completion.rationale.clone(),
        remediations: completion.open_questions.clone(),
        evidence_refs: vec![format!(
            ".agentflow/projections/completions/{}.json",
            project.project_id
        )],
        delivery_refs: optional_vec(
            project
                .delivery
                .as_ref()
                .and_then(|delivery| delivery.public_record_path.clone()),
        ),
        actor_role: "human-owner".to_string(),
        feedback_route: Some("feedback-loop/spec-evolution".to_string()),
        updated_at: completion.updated_at,
    }
}

fn delivery_reasons(delivery: &agentflow_projection::ProjectionDeliverySummary) -> Vec<String> {
    let mut reasons = Vec::new();
    if delivery.ready_count > 0 {
        reasons.push(format!(
            "{} public delivery records ready",
            delivery.ready_count
        ));
    }
    if delivery.missing_count > 0 {
        reasons.push(format!(
            "{} public delivery records missing",
            delivery.missing_count
        ));
    }
    if reasons.is_empty() {
        reasons.push("Delivery status comes from projection read model.".to_string());
    }
    reasons
}

fn delivery_refs(task: &agentflow_projection::TaskProjection) -> Vec<String> {
    let mut refs = Vec::new();
    refs.extend(optional_vec(task.delivery.public_record_path.clone()));
    refs.extend(task.delivery.public_record_targets.clone());
    refs.extend(optional_vec(task.public_delivery.pr_url.clone()));
    refs.extend(optional_vec(task.public_delivery.merge_commit.clone()));
    refs.extend(optional_vec(task.public_delivery.changelog_path.clone()));
    refs.extend(optional_vec(task.public_delivery.release_notes_url.clone()));
    refs.sort();
    refs.dedup();
    refs
}

fn summary(
    status: &str,
    summary: impl Into<String>,
    source_ref: Option<String>,
) -> TeamHistorySummary {
    TeamHistorySummary {
        status: status.to_string(),
        summary: summary.into(),
        source_ref,
    }
}

fn feedback_hook(status: &str, summary: &str) -> TeamFeedbackHook {
    TeamFeedbackHook {
        status: status.to_string(),
        route: "feedback-loop/spec-evolution".to_string(),
        summary: summary.to_string(),
        required_refs: vec![
            "decision history entry".to_string(),
            "delivery history entry".to_string(),
            "human confirmation".to_string(),
        ],
    }
}

fn audit_sidecar(
    status: &str,
    summary: impl Into<String>,
    blocking: bool,
    source_ref: Option<String>,
) -> TeamAuditSidecar {
    TeamAuditSidecar {
        status: status.to_string(),
        summary: summary.into(),
        blocking,
        source_ref,
    }
}

fn non_empty_or(value: String, fallback: &str) -> String {
    if value.trim().is_empty() {
        fallback.to_string()
    } else {
        value
    }
}

fn optional_vec(value: Option<String>) -> Vec<String> {
    value.into_iter().collect()
}

fn project_projection_ref(project_id: &str) -> String {
    format!(".agentflow/projections/projects/{project_id}.json")
}

fn task_projection_ref(issue_id: &str) -> String {
    format!(".agentflow/projections/tasks/{issue_id}.json")
}

#[cfg(test)]
mod tests {
    use super::*;
    use agentflow_projection::{
        ProjectBrainProjection, ProjectCompletionProjection, ProjectIssueLanes, ProjectProjection,
        ProjectionAcceptanceSummary, ProjectionAcceptanceTraceabilitySummary,
        ProjectionDeliverySummary, ProjectionPublicDelivery, TaskProjection,
    };

    #[test]
    fn history_view_exposes_decision_delivery_and_feedback_route() {
        let dir = tempfile::tempdir().unwrap();
        write_project(dir.path(), project_projection());
        write_task(dir.path(), task_projection());

        let view = team_delivery_decision_history_view(dir.path(), "project-history");

        assert_eq!(view.status, "ready");
        assert!(view.readonly);
        assert!(!view.authority);
        assert!(view.projection_backed);
        assert!(view
            .entries
            .iter()
            .any(|entry| entry.entry_kind == "decision" && entry.status == "accepted"));
        assert!(view
            .entries
            .iter()
            .any(|entry| entry.entry_kind == "delivery"
                && entry
                    .delivery_refs
                    .iter()
                    .any(|item| item == "CHANGELOG.md")));
        assert_eq!(view.feedback.route, "feedback-loop/spec-evolution");
        assert!(!view.audit_sidecar.blocking);
    }

    #[test]
    fn missing_task_projection_is_deferred_not_authority() {
        let dir = tempfile::tempdir().unwrap();
        write_project(dir.path(), project_projection());

        let view = team_delivery_decision_history_view(dir.path(), "project-history");

        assert_eq!(view.status, "deferred");
        assert!(view
            .blockers
            .iter()
            .any(|item| item.contains("AF-HIST-001")));
        assert!(view.projection_backed);
        assert!(!view.authority);
    }

    fn write_project(root: &Path, projection: ProjectProjection) {
        agentflow_projection::storage::write_project_projection(root, &projection)
            .expect("write project projection");
    }

    fn write_task(root: &Path, projection: TaskProjection) {
        agentflow_projection::storage::write_task_projection(root, &projection)
            .expect("write task projection");
    }

    fn project_projection() -> ProjectProjection {
        ProjectProjection {
            version: agentflow_projection::PROJECT_PROJECTION_VERSION.to_string(),
            project_id: "project-history".to_string(),
            title: "Team History".to_string(),
            objective: "Make decision and delivery history readable.".to_string(),
            status: "in-progress".to_string(),
            stage_key: "active".to_string(),
            stage_label: "Active".to_string(),
            stage_summary: "History is available.".to_string(),
            issue_ids: vec!["AF-HIST-001".to_string()],
            current_issue_id: Some("AF-HIST-001".to_string()),
            lanes: ProjectIssueLanes {
                current: vec!["AF-HIST-001".to_string()],
                past: Vec::new(),
                future: Vec::new(),
                blocked: Vec::new(),
            },
            next_action: "review-history".to_string(),
            next_action_label: "Review history".to_string(),
            next_action_reason: "History is team-readable.".to_string(),
            blockers: Vec::new(),
            completion_hint: "not complete".to_string(),
            completion: Some(ProjectCompletionProjection {
                current_state: "in-progress".to_string(),
                latest_outcome: Some("accepted".to_string()),
                next_recommended_action: "continue".to_string(),
                next_recommended_action_label: "Continue".to_string(),
                next_recommended_action_reason: "Decision accepted latest delivery.".to_string(),
                total_issue_count: 1,
                completed_issue_count: 1,
                canceled_issue_count: 0,
                remaining_issue_count: 0,
                blocked_issue_count: 0,
                task_evidence_ready_count: 1,
                task_evidence_missing_count: 0,
                delivery_status: "ready".to_string(),
                delivery_missing_count: 0,
                audit_required: false,
                audit_status: "not-requested".to_string(),
                audit_blocking_findings: 0,
                goal_recheck_status: "ready".to_string(),
                project_health_status: "ready".to_string(),
                release_readiness: "ready".to_string(),
                open_questions: Vec::new(),
                rationale: vec!["Acceptance gate passed.".to_string()],
                updated_at: 200,
            }),
            release: None,
            external_review: None,
            delivery: Some(ProjectionDeliverySummary {
                status: "ready".to_string(),
                summary_line: "Public delivery is ready.".to_string(),
                public_record_path: Some("CHANGELOG.md".to_string()),
                ready_count: 1,
                ..ProjectionDeliverySummary::default()
            }),
            audit: None,
            issue_count: 1,
            completed_issue_count: 1,
            project_brain: ProjectBrainProjection {
                project_path: "docs/project/README.md".to_string(),
                goal_path: "docs/project/goal.md".to_string(),
                plan_path: "docs/project/roadmap.md".to_string(),
                decisions_path: "docs/project/decisions.md".to_string(),
                health_path: "docs/project/health.md".to_string(),
                brain_status: "ready".to_string(),
                goal_status: "ready".to_string(),
                plan_status: "ready".to_string(),
                decision_status: "ready".to_string(),
                health_status: "ready".to_string(),
                missing_documents: Vec::new(),
                open_questions: Vec::new(),
                next_recommended_action: "continue".to_string(),
                next_recommended_action_label: "Continue".to_string(),
                next_recommended_action_reason: "Continue.".to_string(),
                readonly: true,
            },
            updated_at: 210,
        }
    }

    fn task_projection() -> TaskProjection {
        TaskProjection {
            version: agentflow_projection::TASK_PROJECTION_VERSION.to_string(),
            issue_id: "AF-HIST-001".to_string(),
            project_id: Some("project-history".to_string()),
            workflow_ref: "workflow://history".to_string(),
            current_state: "done".to_string(),
            display_status: "done".to_string(),
            current_transition: Some("task.completed".to_string()),
            latest_run_id: Some("run-001".to_string()),
            branch_name: Some("codex/history".to_string()),
            timeline: Vec::new(),
            public_delivery: ProjectionPublicDelivery {
                pr_url: Some("https://example.invalid/pull/1".to_string()),
                merge_commit: Some("abc123".to_string()),
                changelog_path: Some("CHANGELOG.md".to_string()),
                ..ProjectionPublicDelivery::default()
            },
            runtime: Default::default(),
            session: Default::default(),
            delivery: ProjectionDeliverySummary {
                status: "ready".to_string(),
                evidence_status: "ready".to_string(),
                evidence_path: Some(".agentflow/tasks/AF-HIST-001/evidence".to_string()),
                public_record_path: Some("CHANGELOG.md".to_string()),
                summary_line: "Task delivery accepted.".to_string(),
                ready_count: 1,
                ..ProjectionDeliverySummary::default()
            },
            audit: Default::default(),
            acceptance: Some(ProjectionAcceptanceSummary {
                outcome: "accepted".to_string(),
                passed: true,
                summary: "Acceptance gate passed.".to_string(),
                failure_reasons: Vec::new(),
                next_steps: Vec::new(),
                sub_gates: Vec::new(),
                traceability: ProjectionAcceptanceTraceabilitySummary {
                    issue_id: "AF-HIST-001".to_string(),
                    run_id: "run-001".to_string(),
                    acceptance_decision_path: ".agentflow/tasks/AF-HIST-001/acceptance.json"
                        .to_string(),
                    evidence_path: ".agentflow/tasks/AF-HIST-001/evidence".to_string(),
                    validation_path: ".agentflow/tasks/AF-HIST-001/validation.log".to_string(),
                    closeout_proof_path: ".agentflow/tasks/AF-HIST-001/closeout.json".to_string(),
                    session_id: Some("session-001".to_string()),
                    provider: Some("codex".to_string()),
                    pr_url: Some("https://example.invalid/pull/1".to_string()),
                    merge_commit_sha: Some("abc123".to_string()),
                },
                checked_at: 220,
            }),
            updated_at: 230,
        }
    }
}
