//! Tauri command registration modules.
//!
//! Command names stay stable while implementation modules are split by product
//! boundary. Desktop remains read-only unless a requirement explicitly changes
//! that boundary.

pub(crate) mod agent_manual;
pub(crate) mod agentflow_watcher;
pub(crate) mod execute;
pub(crate) mod input;
pub(crate) mod output;
pub(crate) mod panel;
pub(crate) mod project_files;
pub(crate) mod project_workspace;
pub(crate) mod state;
pub(crate) mod workflow_events;

#[cfg(test)]
mod tests {
    use super::{output, state};
    use agentflow_state::{WorkflowAuditStatus, WorkflowStage};

    #[test]
    fn desktop_commands_read_e2e_workflow_state_and_audit_outputs() {
        let ready = agentflow_workflow_acceptance::prepare_delivery_ready_fixture().unwrap();
        let project_root = ready.fixture.root().display().to_string();

        let status = state::load_state_status(project_root.clone()).unwrap();
        assert_eq!(status.current_stage, WorkflowStage::DeliveryReady);
        assert_eq!(status.audit_status, WorkflowAuditStatus::NotRequested);
        let gates = state::load_workflow_gates(project_root.clone()).unwrap();
        assert_eq!(gates.current_stage, WorkflowStage::DeliveryReady);
        assert_eq!(gates.audit_status, WorkflowAuditStatus::NotRequested);
        let actions = state::load_next_actions(project_root.clone()).unwrap();
        assert!(!actions
            .actions
            .iter()
            .any(|action| action.action == "request-human-audit" && action.allowed));
        let blockers = state::load_blockers(project_root.clone()).unwrap();
        assert!(blockers.blockers.is_empty());
        let output_status = output::load_output_status(project_root.clone()).unwrap();
        assert_eq!(output_status.summary.release_deliveries, 1);
        let output_index = output::load_output_index(project_root.clone()).unwrap();
        assert_eq!(output_index.release_deliveries.len(), 1);

        let report = output::request_human_audit(
            project_root.clone(),
            agentflow_workflow_acceptance::acceptance::human_audit_draft(&ready.run_id),
        )
        .unwrap();
        let audit_index = output::load_audit_index(project_root.clone()).unwrap();
        assert_eq!(audit_index.audits.len(), 1);
        let loaded_report =
            output::load_audit_report(project_root.clone(), report.audit.audit_id.clone()).unwrap();
        assert!(!loaded_report.report_markdown.trim().is_empty());

        let refreshed = state::refresh_state(project_root).unwrap();
        assert_eq!(refreshed.current_stage, WorkflowStage::AuditCompleted);
        assert_ne!(refreshed.audit_status, WorkflowAuditStatus::NotRequested);
        ready.fixture.assert_user_files_unchanged().unwrap();
    }
}
