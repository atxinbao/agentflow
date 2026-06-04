//! Output commands expose delivery/evidence/audit status to Desktop.
//!
//! Human-triggered audit is the only Desktop-authorized output write path here.
//! It writes only `.agentflow/output/audit/<audit-id>/**`.

#[tauri::command]
pub(crate) fn load_output_status(
    project_root: String,
) -> Result<agentflow_output::OutputStatusSnapshot, String> {
    agentflow_output::load_output_status(project_root).map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn load_output_manifest(
    project_root: String,
) -> Result<agentflow_output::OutputManifest, String> {
    agentflow_output::load_output_manifest(project_root).map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn load_output_index(
    project_root: String,
) -> Result<agentflow_output::OutputIndex, String> {
    agentflow_output::load_output_index(project_root).map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn load_output_snapshot(
    project_root: String,
) -> Result<agentflow_output::OutputSnapshot, String> {
    agentflow_output::load_output_snapshot(project_root).map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn validate_output(
    project_root: String,
) -> Result<agentflow_output::OutputSnapshot, String> {
    agentflow_output::validate_output(project_root).map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn request_human_audit(
    project_root: String,
    draft: agentflow_output::HumanAuditRequestDraft,
) -> Result<agentflow_output::HumanAuditReport, String> {
    agentflow_output::request_human_audit(project_root, draft).map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn load_audit_report(
    project_root: String,
    audit_id: String,
) -> Result<agentflow_output::HumanAuditReport, String> {
    agentflow_output::load_audit_report(project_root, audit_id).map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn load_audit_index(
    project_root: String,
) -> Result<agentflow_output::AuditIndex, String> {
    agentflow_output::load_audit_index(project_root).map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn load_audit_status(
    project_root: String,
) -> Result<agentflow_output::AuditManifest, String> {
    agentflow_output::load_audit_status(project_root).map_err(|error| error.to_string())
}
