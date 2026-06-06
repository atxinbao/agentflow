//! Output commands expose delivery/evidence/audit status to Desktop.
//!
//! Desktop普通界面只读展示审计状态和报告。human-via-agent 写入能力
//! 保留在底层库和本地测试里，不注册为普通 Tauri 命令。

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

#[cfg(test)]
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
