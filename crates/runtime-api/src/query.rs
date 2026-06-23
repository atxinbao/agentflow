use std::path::Path;

use anyhow::Result;

pub const RUNTIME_QUERY_API_VERSION: &str = "agentflow-runtime-query-api.v1";

pub fn get_requirement_intake_view(
    project_root: impl AsRef<Path>,
    requirement_id: &str,
) -> Result<agentflow_projection::RequirementIntakeView> {
    agentflow_projection::get_requirement_intake_view(project_root, requirement_id)
}

pub fn get_spec_preview_view(
    project_root: impl AsRef<Path>,
    requirement_id: &str,
) -> Result<agentflow_projection::SpecPreviewView> {
    agentflow_projection::get_spec_preview_view(project_root, requirement_id)
}

pub fn get_spec_loop_view(
    project_root: impl AsRef<Path>,
    requirement_id: &str,
) -> Result<agentflow_projection::SpecLoopView> {
    agentflow_projection::get_spec_loop_view(project_root, requirement_id)
}

pub fn get_project_home_view(
    project_root: impl AsRef<Path>,
    project_id: &str,
) -> Result<agentflow_projection::ProjectHomeView> {
    agentflow_projection::get_project_home_view(project_root, project_id)
}

pub fn get_task_workbench_view(
    project_root: impl AsRef<Path>,
    issue_id: &str,
) -> Result<agentflow_projection::TaskWorkbenchView> {
    agentflow_projection::get_task_workbench_view(project_root, issue_id)
}

pub fn get_work_loop_run_view(
    project_root: impl AsRef<Path>,
    issue_id: &str,
    run_id: &str,
) -> Result<agentflow_projection::WorkLoopRunView> {
    agentflow_projection::get_work_loop_run_view(project_root, issue_id, run_id)
}

pub fn get_work_loop_session_view(
    project_root: impl AsRef<Path>,
    session_id: &str,
) -> Result<agentflow_projection::WorkLoopSessionView> {
    agentflow_projection::get_work_loop_session_view(project_root, session_id)
}

pub fn get_audit_surface_view(
    project_root: impl AsRef<Path>,
    audit_id: &str,
) -> Result<agentflow_projection::AuditSurfaceView> {
    agentflow_projection::get_audit_surface_view(project_root, audit_id)
}

pub fn get_delivery_package_view(
    project_root: impl AsRef<Path>,
    issue_id: &str,
) -> Result<agentflow_projection::DeliveryPackageView> {
    agentflow_projection::get_delivery_package_view(project_root, issue_id)
}

pub fn get_pack_industry_workbench_view(
    project_root: impl AsRef<Path>,
    pack_id: Option<&str>,
) -> Result<agentflow_projection::PackIndustryWorkbenchView> {
    agentflow_projection::get_pack_industry_workbench_view(project_root, pack_id)
}

pub fn get_runtime_health_view(
    project_root: impl AsRef<Path>,
    project_id: &str,
) -> Result<agentflow_projection::RuntimeHealthView> {
    agentflow_projection::get_runtime_health_view(project_root, project_id)
}

#[cfg(test)]
mod tests {
    use super::get_pack_industry_workbench_view;

    #[test]
    fn runtime_api_reads_pack_industry_workbench() {
        let dir = tempfile::tempdir().unwrap();

        let view = get_pack_industry_workbench_view(dir.path(), Some("software-dev")).unwrap();

        assert!(!view.authority);
        assert_eq!(view.active_pack_id.as_deref(), Some("software-dev"));
        assert!(view
            .domain_object_index
            .iter()
            .any(|object| object.object_type_id == "Issue"));
        assert!(view
            .connector_capability_index
            .iter()
            .any(|capability| capability.connector_id == "github"));
    }
}
