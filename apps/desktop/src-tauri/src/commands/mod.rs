//! Tauri command registration modules.
//!
//! Command names stay stable while implementation modules are split by product
//! boundary. Desktop remains read-only unless a requirement explicitly changes
//! that boundary.

pub(crate) mod agent_manual;
pub(crate) mod agentflow_watcher;
pub(crate) mod mcp;
pub(crate) mod output;
pub(crate) mod panel;
pub(crate) mod project_files;
pub(crate) mod project_loop;
pub(crate) mod project_workspace;
pub(crate) mod projection;
pub(crate) mod spec;
pub(crate) mod state;
pub(crate) mod workflow_events;

#[cfg(test)]
mod tests {
    use super::state;
    use agentflow_state::{WorkflowAuditStatus, WorkflowStage};

    #[test]
    fn desktop_commands_read_e2e_workflow_state() {
        let ready = agentflow_workflow_acceptance::prepare_delivery_ready_fixture().unwrap();
        let project_root = ready.fixture.root().display().to_string();
        agentflow_spec::prepare_spec_workspace(&project_root).unwrap();
        agentflow_projection::prepare_projection_workspace(&project_root).unwrap();

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
        let refreshed = state::refresh_state(project_root).unwrap();
        assert_eq!(refreshed.current_stage, WorkflowStage::DeliveryReady);
        assert_eq!(refreshed.audit_status, WorkflowAuditStatus::NotRequested);
        ready.fixture.assert_user_files_unchanged().unwrap();
    }
}
