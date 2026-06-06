use super::{
    base_release::load_project_initialization_status as load_project_initialization_status_at,
    git::canonical_project_root,
    model::{ProjectInitializationSummary, ProjectWorkspaceSummary},
    prepare::prepare_local_project_workspace_at,
};

pub(crate) fn prepare_local_project_workspace(
    project_root: String,
    app_locale: Option<String>,
) -> Result<ProjectWorkspaceSummary, String> {
    prepare_local_project_workspace_at(&project_root, app_locale)
}

pub(crate) fn load_project_initialization_status(
    project_root: String,
) -> Result<ProjectInitializationSummary, String> {
    let root = canonical_project_root(&project_root)?;
    load_project_initialization_status_at(&root)
}
