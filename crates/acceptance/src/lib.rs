pub mod acceptance;
pub mod assertions;
pub mod boundaries;
pub mod browser_preview;
pub mod fixture;

pub use acceptance::{
    create_high_risk_blocker_fixture, create_stale_lease_fixture,
    prepare_execution_completed_fixture, ExecutionCompletedFixture,
};
pub use fixture::{create_fixture_project, WorkflowFixture};

#[cfg(test)]
mod tests {
    use super::*;
    use agentflow_audit::AuditStatus;
    use agentflow_state::{WorkflowAuditStatus, WorkflowStage};

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
}
