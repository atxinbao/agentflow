//! Task and project projection commands for Desktop.
//!
//! These commands expose event-driven read models only. They rebuild and read
//! `.agentflow/projections/**` plus `.agentflow/indexes/**`; they do not execute
//! issues, create delivery artifacts, or mutate user source files.

use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SpecWorkbenchProjection {
    version: String,
    selected_requirement_id: Option<String>,
    requirements: Vec<agentflow_projection::RequirementPreviewIndexEntry>,
    intake: Option<agentflow_projection::RequirementIntakeView>,
    preview: Option<agentflow_projection::SpecPreviewView>,
    spec_loop: Option<agentflow_projection::SpecLoopView>,
    warnings: Vec<String>,
}

#[tauri::command]
pub(crate) fn rebuild_task_projections(
    project_root: String,
) -> Result<agentflow_projection::ProjectionSummary, String> {
    agentflow_projection::rebuild_projections(project_root).map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn load_projection_issue_status_index(
    project_root: String,
) -> Result<agentflow_projection::IssueStatusIndex, String> {
    agentflow_projection::load_issue_status_index(project_root).map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn load_task_projection(
    project_root: String,
    issue_id: String,
) -> Result<agentflow_projection::TaskProjection, String> {
    agentflow_projection::load_task_projection(project_root, &issue_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn load_project_projection(
    project_root: String,
    project_id: String,
) -> Result<agentflow_projection::ProjectProjection, String> {
    agentflow_projection::load_project_projection(project_root, &project_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn load_spec_workbench_projection(
    project_root: String,
    requirement_id: Option<String>,
) -> Result<SpecWorkbenchProjection, String> {
    let mut warnings = Vec::new();
    agentflow_projection::rebuild_projections(&project_root).map_err(|error| error.to_string())?;
    let index = match agentflow_projection::load_requirement_preview_index(&project_root) {
        Ok(index) => index,
        Err(error) => {
            warnings.push(format!("requirement-preview-index-missing: {error}"));
            return Ok(SpecWorkbenchProjection {
                version: "spec-workbench-projection.v1".to_string(),
                selected_requirement_id: None,
                requirements: Vec::new(),
                intake: None,
                preview: None,
                spec_loop: None,
                warnings,
            });
        }
    };
    let mut requirements = index.previews;
    requirements.sort_by(|left, right| right.updated_at.cmp(&left.updated_at));
    let selected_requirement_id = requirement_id
        .filter(|candidate| {
            requirements
                .iter()
                .any(|entry| entry.requirement_id == *candidate)
        })
        .or_else(|| {
            requirements
                .first()
                .map(|entry| entry.requirement_id.clone())
        });
    let (intake, preview, spec_loop) = match selected_requirement_id.as_deref() {
        Some(requirement_id) => (
            Some(
                agentflow_projection::get_requirement_intake_view(&project_root, requirement_id)
                    .map_err(|error| error.to_string())?,
            ),
            Some(
                agentflow_projection::get_spec_preview_view(&project_root, requirement_id)
                    .map_err(|error| error.to_string())?,
            ),
            Some(
                agentflow_projection::get_spec_loop_view(&project_root, requirement_id)
                    .map_err(|error| error.to_string())?,
            ),
        ),
        None => (None, None, None),
    };

    Ok(SpecWorkbenchProjection {
        version: "spec-workbench-projection.v1".to_string(),
        selected_requirement_id,
        requirements,
        intake,
        preview,
        spec_loop,
        warnings,
    })
}
