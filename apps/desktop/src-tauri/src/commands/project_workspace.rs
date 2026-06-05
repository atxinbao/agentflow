//! Project Workspace Manager command wrappers.
//!
//! Tauri command names stay stable while the implementation is isolated under
//! `crate::project_workspace`.

#[tauri::command]
pub(crate) fn prepare_local_project_workspace(
    project_root: String,
    app_locale: Option<String>,
) -> Result<crate::project_workspace::ProjectWorkspaceSummary, String> {
    crate::project_workspace::prepare_local_project_workspace(project_root, app_locale)
}
