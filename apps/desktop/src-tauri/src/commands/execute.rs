//! Execute commands are agent-only write APIs plus read APIs.
//!
//! Desktop human UI must keep these as read-only display unless a requirement
//! explicitly adds a human confirmation or cancel action.

#[tauri::command]
pub(crate) fn load_execute_status(
    project_root: String,
) -> Result<agentflow_execute::ExecuteStatusSnapshot, String> {
    agentflow_execute::load_execute_status(project_root).map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn load_execute_manifest(
    project_root: String,
) -> Result<agentflow_execute::ExecuteManifest, String> {
    agentflow_execute::load_execute_manifest(project_root).map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn load_execute_index(
    project_root: String,
) -> Result<agentflow_execute::ExecuteIndex, String> {
    agentflow_execute::load_execute_index(project_root).map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn load_execute_snapshot(
    project_root: String,
) -> Result<agentflow_execute::ExecuteSnapshot, String> {
    agentflow_execute::load_execute_snapshot(project_root).map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn create_execute_run(
    project_root: String,
    issue_id: String,
) -> Result<agentflow_execute::ExecuteRun, String> {
    agentflow_execute::create_execute_run(project_root, issue_id).map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn load_execute_run(
    project_root: String,
    run_id: String,
) -> Result<agentflow_execute::ExecuteRun, String> {
    agentflow_execute::load_execute_run(project_root, run_id).map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn execute_run_preflight(
    project_root: String,
    run_id: String,
) -> Result<agentflow_execute::ExecutePreflight, String> {
    agentflow_execute::execute_run_preflight(project_root, run_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn confirm_high_risk_execute_run(
    project_root: String,
    run_id: String,
    confirmation_text: String,
) -> Result<agentflow_execute::ExecuteHumanConfirmation, String> {
    agentflow_execute::confirm_high_risk_execute_run(project_root, run_id, confirmation_text)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn acquire_execute_lease(
    project_root: String,
    run_id: String,
) -> Result<agentflow_execute::ExecuteLease, String> {
    agentflow_execute::acquire_execute_lease(project_root, run_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn release_execute_lease(
    project_root: String,
    run_id: String,
) -> Result<agentflow_execute::ExecuteLease, String> {
    agentflow_execute::release_execute_lease(project_root, run_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn write_execute_plan(
    project_root: String,
    run_id: String,
    draft: agentflow_execute::ExecutePlanDraft,
) -> Result<agentflow_execute::ExecutePlan, String> {
    agentflow_execute::write_execute_plan(project_root, run_id, draft)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn create_execute_checkpoint(
    project_root: String,
    run_id: String,
) -> Result<agentflow_execute::ExecuteCheckpoint, String> {
    agentflow_execute::create_execute_checkpoint(project_root, run_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn apply_execute_patch(
    project_root: String,
    run_id: String,
    proposed_patch: String,
) -> Result<agentflow_execute::ExecutePatchOutcome, String> {
    agentflow_execute::apply_execute_patch(project_root, run_id, proposed_patch)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn run_execute_command(
    project_root: String,
    run_id: String,
    request: agentflow_execute::ExecuteCommandRequest,
) -> Result<agentflow_execute::ExecuteCommandRecord, String> {
    agentflow_execute::run_execute_command(project_root, run_id, request)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn load_execute_result(
    project_root: String,
    run_id: String,
) -> Result<agentflow_execute::ExecuteResult, String> {
    agentflow_execute::load_execute_result(project_root, run_id).map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn validate_execute_run(
    project_root: String,
    run_id: String,
) -> Result<agentflow_execute::ExecuteResult, String> {
    agentflow_execute::validate_execute_run(project_root, run_id).map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn complete_execute_run(
    project_root: String,
    run_id: String,
) -> Result<agentflow_execute::ExecuteResult, String> {
    agentflow_execute::complete_execute_run(project_root, run_id).map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn complete_build_agent_issue(
    project_root: String,
    request: agentflow_execute::BuildAgentCompletionRequest,
) -> Result<agentflow_execute::BuildAgentCompletion, String> {
    let completion = agentflow_execute::complete_build_agent_issue(&project_root, request)
        .map_err(|error| error.to_string())?;
    agentflow_state::refresh_state(&project_root).map_err(|error| error.to_string())?;
    Ok(completion)
}

#[tauri::command]
pub(crate) fn cancel_execute_run(
    project_root: String,
    run_id: String,
) -> Result<agentflow_execute::ExecuteRun, String> {
    agentflow_execute::cancel_execute_run(project_root, run_id).map_err(|error| error.to_string())
}
