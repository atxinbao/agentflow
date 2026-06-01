use super::{model::ProjectWorkspaceSummary, prepare::prepare_local_project_workspace_at};

pub(crate) fn prepare_local_project_workspace(
    project_root: String,
) -> Result<ProjectWorkspaceSummary, String> {
    prepare_local_project_workspace_at(&project_root)
}
