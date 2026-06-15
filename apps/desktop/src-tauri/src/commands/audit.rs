//! Audit commands expose audit status and reports to Desktop.
//!
//! Desktop普通界面只读展示审计状态和报告。human-via-agent 写入能力
//! 保留在底层库和本地测试里，不注册为普通 Tauri 命令。

#[tauri::command]
pub(crate) fn load_audit_report(
    project_root: String,
    audit_id: String,
) -> Result<agentflow_audit::HumanAuditReport, String> {
    agentflow_audit::load_audit_report(project_root, audit_id).map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn load_audit_index(
    project_root: String,
) -> Result<agentflow_audit::AuditIndex, String> {
    agentflow_audit::load_audit_index(project_root).map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn load_audit_status(
    project_root: String,
) -> Result<agentflow_audit::AuditManifest, String> {
    agentflow_audit::load_audit_status(project_root).map_err(|error| error.to_string())
}
