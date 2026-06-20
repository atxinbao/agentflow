pub mod acceptance;
pub mod assertions;
pub mod boundaries;
pub mod browser_preview;
pub mod fixture;

pub use acceptance::{
    create_high_risk_blocker_fixture, create_stale_lease_fixture,
    prepare_execution_completed_fixture, prepare_project_done_fixture,
    prepare_project_in_review_fixture, ExecutionCompletedFixture, ProjectLoopCloseoutFixture,
};
pub use fixture::{create_fixture_project, WorkflowFixture};

#[cfg(test)]
mod tests {
    use super::*;
    use agentflow_action_contract::{
        core_action_contract_bundle, validate_action_contract_bundle, ActionSourceSurface,
    };
    use agentflow_audit::AuditStatus;
    use agentflow_object_state::{core_object_state_bundle, validate_object_state_bundle};
    use agentflow_ontology::{
        core_ontology_bundle, core_ontology_registry, validate_ontology_bundle,
    };
    use agentflow_projection::{
        get_spec_loop_view, load_project_projection, load_task_projection, rebuild_projections,
    };
    use agentflow_role_policy::{core_role_policy_bundle, validate_role_policy_bundle};
    use agentflow_runtime_api::{
        execute_command_via_arbitration, get_project_home_view, get_runtime_health_view,
        get_task_workbench_view, project_confirm_goal, project_confirm_plan, project_intake,
        project_materialize, RuntimeCommandRequest, RuntimeCommandStatus,
    };
    use agentflow_state::{WorkflowAuditStatus, WorkflowStage};
    use agentflow_task_loop::TaskLoop;
    use serde_json::json;
    use tempfile::tempdir;

    #[test]
    fn full_workflow_keeps_execute_completed_without_auto_audit_request() -> anyhow::Result<()> {
        let ready = prepare_execution_completed_fixture()?;

        assert_eq!(
            ready.execution_state.current_stage,
            WorkflowStage::ExecuteCompleted
        );
        assert_eq!(
            ready.execution_state.audit_status,
            WorkflowAuditStatus::NotRequested
        );
        assert!(!ready
            .execution_state
            .next_actions
            .contains(&"request-human-audit".to_string()));
        let audit_index =
            agentflow_audit::load_audit_index(ready.fixture.root()).unwrap_or_default();
        assert_eq!(audit_index.audits.len(), 0);
        ready.fixture.assert_user_files_unchanged()?;
        boundaries::assert_write_boundary(ready.fixture.root())?;
        Ok(())
    }

    #[test]
    fn runtime_foundation_main_chain_closeout_is_verifiable() -> anyhow::Result<()> {
        let ontology_bundle = core_ontology_bundle();
        let ontology_report = validate_ontology_bundle(&ontology_bundle);
        assert!(ontology_report.valid, "{:?}", ontology_report.errors);

        let ontology_registry = core_ontology_registry();
        let action_bundle = core_action_contract_bundle();
        let action_report = validate_action_contract_bundle(&action_bundle, &ontology_registry);
        assert!(action_report.valid, "{:?}", action_report.errors);

        let action_registry =
            agentflow_action_contract::core_action_contract_registry(&ontology_registry);
        let role_bundle = core_role_policy_bundle();
        let role_report =
            validate_role_policy_bundle(&role_bundle, &ontology_registry, &action_registry);
        assert!(role_report.valid, "{:?}", role_report.errors);

        let state_bundle = core_object_state_bundle();
        let state_report =
            validate_object_state_bundle(&state_bundle, &ontology_registry, &action_registry);
        assert!(state_report.valid, "{state_report:?}");

        let runtime_dir = tempdir()?;
        let response = execute_command_via_arbitration(
            runtime_dir.path(),
            &RuntimeCommandRequest {
                command_id: "cmd-foundation-submit".to_string(),
                command_type: "submitRequirement".to_string(),
                source_surface: ActionSourceSurface::Desktop,
                actor_role: "spec-agent".to_string(),
                target_object_ref: None,
                input: json!({
                    "summary": "收口 Runtime Foundation closeout baseline",
                    "requestType": "feature"
                }),
                evidence_refs: Vec::new(),
                artifact_refs: Vec::new(),
                idempotency_key: "idem-foundation-submit".to_string(),
                created_at: "2026-06-20T00:00:00Z".to_string(),
            },
        )?;
        assert_ne!(response.status, RuntimeCommandStatus::InvalidCommand);
        assert_eq!(
            response
                .next_query_hint
                .as_ref()
                .map(|hint| hint.view.as_str()),
            Some("RequirementIntakeView")
        );
        assert_eq!(response.correlation_id, "cmd-foundation-submit");

        let fixture = prepare_project_in_review_fixture()?;
        let project_view = get_project_home_view(fixture.fixture.root(), &fixture.project_id)?;
        let task_view = get_task_workbench_view(fixture.fixture.root(), &fixture.current_issue_id)?;
        let health_view = get_runtime_health_view(fixture.fixture.root(), &fixture.project_id)?;

        assert_eq!(project_view.project_id, fixture.project_id);
        assert!(matches!(
            project_view.freshness.staleness.as_str(),
            "fresh" | "current"
        ));
        assert!(!project_view.recent_events.is_empty());
        assert_eq!(
            project_view.issue_groups.current,
            vec![fixture.current_issue_id.clone()]
        );
        assert_eq!(
            project_view.issue_groups.future,
            vec![fixture.next_issue_id.clone()]
        );

        assert_eq!(task_view.issue_id, fixture.current_issue_id);
        assert_eq!(task_view.issue_state, "in_review");
        assert!(matches!(
            task_view.freshness.staleness.as_str(),
            "fresh" | "current"
        ));
        assert!(task_view
            .timeline
            .iter()
            .any(|item| item.state == "in_review" && !item.events.is_empty()));

        assert_eq!(health_view.project_id, fixture.project_id);
        assert!(matches!(
            health_view.project_status.as_str(),
            "active" | "in_review"
        ));
        assert_eq!(
            health_view.current_issue_id.as_deref(),
            Some(fixture.current_issue_id.as_str())
        );
        assert_eq!(health_view.active_issue_count, 1);

        fixture.fixture.assert_user_files_unchanged()?;
        boundaries::assert_write_boundary(fixture.fixture.root())?;
        Ok(())
    }

    #[test]
    fn human_audit_request_updates_state() -> anyhow::Result<()> {
        let ready = prepare_execution_completed_fixture()?;
        let report = ready.request_human_audit()?;

        assert_eq!(
            report.request.reason,
            "Human requested end-to-end acceptance audit."
        );
        assert_eq!(report.request.scope.refs.len(), 4);
        assert!(!report.report_markdown.trim().is_empty());
        assert!(matches!(
            report.audit.status,
            AuditStatus::Passed
                | AuditStatus::PassedWithWarnings
                | AuditStatus::Failed
                | AuditStatus::Cancelled
        ));

        let loaded = agentflow_audit::load_audit_report(
            ready.fixture.root(),
            report.audit.audit_id.clone(),
        )?;
        assert_eq!(loaded.audit.audit_id, report.audit.audit_id);
        assert!(!loaded.findings.version.is_empty());
        assert!(!loaded.checklist_markdown.trim().is_empty());
        assert_eq!(loaded.evidence_map.audit_id, report.audit.audit_id);
        assert_eq!(loaded.traceability.audit_id, report.audit.audit_id);

        let state = agentflow_state::refresh_state(ready.fixture.root())?;
        assert_eq!(state.current_stage, WorkflowStage::AuditCompleted);
        assert_ne!(state.audit_status, WorkflowAuditStatus::NotRequested);
        ready.fixture.assert_user_files_unchanged()?;
        boundaries::assert_write_boundary(ready.fixture.root())?;
        Ok(())
    }

    #[test]
    fn write_boundary_keeps_user_source_unchanged() -> anyhow::Result<()> {
        let ready = prepare_execution_completed_fixture()?;
        ready.request_human_audit()?;

        ready.fixture.assert_user_files_unchanged()?;
        boundaries::assert_no_forbidden_side_effects(ready.fixture.root())?;
        boundaries::assert_write_boundary(ready.fixture.root())?;
        Ok(())
    }

    #[test]
    fn dependency_issue_creates_blocker() -> anyhow::Result<()> {
        let fixture = create_high_risk_blocker_fixture()?;
        let blockers = agentflow_state::load_blockers(fixture.root())?;

        assert!(blockers
            .blockers
            .iter()
            .any(|blocker| blocker.reason.contains("前置依赖")));
        fixture.assert_user_files_unchanged()?;
        Ok(())
    }

    #[test]
    fn retired_execute_lease_layer_keeps_locks_empty() -> anyhow::Result<()> {
        let fixture = create_stale_lease_fixture()?;
        let locks = agentflow_state::load_state_locks(fixture.root())?;

        assert!(locks.active.is_empty());
        assert!(locks.stale.is_empty());
        assert!(locks.cleanup_candidates.is_empty());
        fixture.assert_user_files_unchanged()?;
        Ok(())
    }

    #[test]
    fn browser_preview_smoke_contract_is_registered() -> anyhow::Result<()> {
        let contract = browser_preview::load_browser_preview_smoke_contract()?;

        assert!(contract.package_script.contains("preview:smoke"));
        assert!(contract
            .browser_preview_data
            .contains("createBrowserPreviewHumanAuditReport"));
        assert!(contract.desktop_app.contains("buildTaskDeliveryProjection"));
        assert!(contract
            .desktop_app
            .contains("createBrowserPreviewHumanAuditReport"));
        assert!(!contract.desktop_app.contains("request_human_audit"));
        Ok(())
    }

    #[test]
    fn project_loop_keeps_next_issue_pending_until_closeout_done() -> anyhow::Result<()> {
        let fixture = prepare_project_in_review_fixture()?;
        let task = load_task_projection(fixture.fixture.root(), &fixture.current_issue_id)?;
        let project = load_project_projection(fixture.fixture.root(), &fixture.project_id)?;

        assert_eq!(task.current_state, "in_review");
        let review_timeline = task
            .timeline
            .iter()
            .find(|item| item.state == "in_review")
            .expect("missing in_review timeline");
        assert!(review_timeline.summary.contains("PR/MR 合并"));
        assert!(review_timeline.summary.contains("Issue 关闭"));
        assert!(review_timeline.summary.contains("Done 写回"));
        assert_eq!(
            project.current_issue_id.as_deref(),
            Some(fixture.current_issue_id.as_str())
        );
        assert!(project.stage_summary.contains("PR/MR 合并"));
        assert!(project.stage_summary.contains("Issue 关闭"));
        assert!(project.stage_summary.contains("Done 写回"));
        assert!(TaskLoop::new(&fixture.project_id)
            .tick(fixture.fixture.root(), "codex")?
            .is_none());
        fixture.fixture.assert_user_files_unchanged()?;
        Ok(())
    }

    #[test]
    fn project_loop_advances_only_after_done_writeback() -> anyhow::Result<()> {
        let fixture = prepare_project_done_fixture()?;
        let task = load_task_projection(fixture.fixture.root(), &fixture.current_issue_id)?;

        assert_eq!(task.current_state, "done");
        let tick = TaskLoop::new(&fixture.project_id)
            .tick(fixture.fixture.root(), "codex")?
            .expect("expected next issue launch after done writeback");
        assert_eq!(tick.launch.issue_id, fixture.next_issue_id);

        rebuild_projections(fixture.fixture.root())?;
        let next_issue = load_task_projection(fixture.fixture.root(), &fixture.next_issue_id)?;
        assert_eq!(next_issue.current_state, "in_progress");
        assert_eq!(next_issue.latest_run_id.as_deref(), Some("run-001"));
        fixture.fixture.assert_user_files_unchanged()?;
        Ok(())
    }

    #[test]
    fn spec_loop_projection_proves_main_chain_to_runtime_action_proposal() -> anyhow::Result<()> {
        let fixture = create_fixture_project()?;
        let root = fixture.root();
        let define = agentflow_agent_manual::prepare_agent_working_manual(root)?;
        assert!(define.ready);
        let panel = agentflow_panel::prepare_project_panel(
            root,
            agentflow_panel::PanelPrepareMode::Blocking,
        )?;
        assert!(matches!(panel.status, agentflow_panel::PanelStatus::Ready));
        agentflow_spec::prepare_spec_workspace(root)?;
        agentflow_projection::prepare_projection_workspace(root)?;
        agentflow_event_store::prepare_event_store(root)?;
        agentflow_audit::prepare_audit_workspace(root)?;
        let state = agentflow_state::prepare_state_workspace(root)?;
        assert_eq!(state.status, agentflow_state::StateWorkspaceStatus::Ready);

        let requirement = root.join("docs/requirements/041-spec-loop-acceptance.md");
        std::fs::create_dir_all(requirement.parent().unwrap())?;
        std::fs::write(
            &requirement,
            "# Spec Loop Acceptance\n\n把原始需求走到 preview、confirmation、materialization，并接到 Runtime Action Proposal。\n",
        )?;

        let preview = project_intake(root, &requirement, Some("project-spec-loop"))?;
        assert_eq!(preview.requirement_id, "041-spec-loop-acceptance");

        let preview = project_confirm_goal(root, "041-spec-loop-acceptance", "goal-agent")?;
        assert_eq!(preview.current_state, "plan_draft");

        let preview = project_confirm_plan(root, "041-spec-loop-acceptance", "spec-agent")?;
        assert_eq!(preview.current_state, "confirmed");

        let materialized = project_materialize(root, "041-spec-loop-acceptance")?;
        let view = get_spec_loop_view(root, "041-spec-loop-acceptance")?;

        assert_eq!(view.requirement_id, "041-spec-loop-acceptance");
        assert_eq!(view.stages.len(), 8);
        assert_eq!(
            view.stages.last().map(|stage| stage.status.as_str()),
            Some("materialized")
        );
        assert_eq!(materialized.project.project_id, "project-spec-loop");
        assert_eq!(
            materialized.action_proposal_bridge.len(),
            view.runtime_action_proposals.len()
        );
        assert_eq!(
            materialized.accepted_count,
            materialized.action_proposal_bridge.len()
        );
        assert_eq!(materialized.rejected_count, 0);
        assert!(view.traceability.iter().any(|edge| {
            edge.from_ref == "docs/requirements/041-spec-loop-acceptance.md"
                && edge.to_ref.ends_with("/intake.json")
        }));
        assert!(view.traceability.iter().any(|edge| {
            edge.from_ref.ends_with("/materialization.json")
                && edge
                    .to_ref
                    .starts_with("runtime-action-proposal:createProject:")
        }));
        assert!(view.runtime_action_proposals.iter().any(|proposal| {
            proposal.action_type == "createProject" && proposal.target_object_type == "Spec"
        }));
        assert!(view.runtime_action_proposals.iter().any(|proposal| {
            proposal.action_type == "createIssue" && proposal.target_object_type == "Project"
        }));
        assert_eq!(view.freshness.staleness, "current");

        fixture.assert_user_files_unchanged()?;
        boundaries::assert_write_boundary(root)?;
        Ok(())
    }
}
