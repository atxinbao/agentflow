pub mod acceptance;
pub mod assertions;
pub mod boundaries;
pub mod browser_preview;
pub mod fixture;

pub use acceptance::{
    create_high_risk_blocker_fixture, create_stale_lease_fixture, prepare_delivery_ready_fixture,
    DeliveryReadyFixture,
};
pub use fixture::{create_fixture_project, WorkflowFixture};

#[cfg(test)]
mod tests {
    use super::*;
    use agentflow_output::AuditStatus;
    use agentflow_state::{WorkflowAuditStatus, WorkflowStage};

    #[test]
    fn full_workflow_keeps_delivery_ready_without_auto_audit_request() -> anyhow::Result<()> {
        let ready = prepare_delivery_ready_fixture()?;

        assert_eq!(
            ready.delivery_state.current_stage,
            WorkflowStage::DeliveryReady
        );
        assert_eq!(
            ready.delivery_state.audit_status,
            WorkflowAuditStatus::NotRequested
        );
        assert!(!ready
            .delivery_state
            .next_actions
            .contains(&"request-human-audit".to_string()));
        let audit_index =
            agentflow_output::load_audit_index(ready.fixture.root()).unwrap_or_default();
        assert_eq!(audit_index.audits.len(), 0);
        ready.fixture.assert_user_files_unchanged()?;
        boundaries::assert_write_boundary(ready.fixture.root())?;
        Ok(())
    }

    #[test]
    fn human_audit_request_updates_state() -> anyhow::Result<()> {
        let ready = prepare_delivery_ready_fixture()?;
        let report = ready.request_human_audit()?;

        assert_eq!(
            report.request.reason,
            "Human requested end-to-end acceptance audit."
        );
        assert_eq!(report.request.scope.refs.len(), 5);
        assert!(!report.report_markdown.trim().is_empty());
        assert!(matches!(
            report.audit.status,
            AuditStatus::Passed
                | AuditStatus::PassedWithWarnings
                | AuditStatus::Failed
                | AuditStatus::Cancelled
        ));

        let loaded = agentflow_output::load_audit_report(
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
        let ready = prepare_delivery_ready_fixture()?;
        ready.request_human_audit()?;

        ready.fixture.assert_user_files_unchanged()?;
        boundaries::assert_no_forbidden_side_effects(ready.fixture.root())?;
        boundaries::assert_write_boundary(ready.fixture.root())?;
        Ok(())
    }

    #[test]
    fn high_risk_issue_creates_blocker() -> anyhow::Result<()> {
        let fixture = create_high_risk_blocker_fixture()?;
        let blockers = agentflow_state::load_blockers(fixture.root())?;

        assert!(blockers
            .blockers
            .iter()
            .any(|blocker| blocker.reason.contains("High risk issue requires")));
        fixture.assert_user_files_unchanged()?;
        Ok(())
    }

    #[test]
    fn stale_lease_is_reported() -> anyhow::Result<()> {
        let fixture = create_stale_lease_fixture()?;
        let locks = agentflow_state::load_state_locks(fixture.root())?;

        assert_eq!(locks.stale.len(), 1);
        assert_eq!(locks.stale[0].run_id.as_deref(), Some("run-stale"));
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
