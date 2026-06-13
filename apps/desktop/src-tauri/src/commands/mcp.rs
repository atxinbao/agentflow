//! MCP / provider bridge read commands.
//!
//! Desktop keeps this boundary read-only. Session creation is driven by
//! workflow-event consumers, not by direct human UI mutation.

#[tauri::command]
pub(crate) fn load_mcp_session_snapshots(
    project_root: String,
) -> Result<Vec<agentflow_mcp::McpSessionSnapshot>, String> {
    agentflow_mcp::load_session_snapshots(project_root).map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn poll_mcp_session_snapshots(
    project_root: String,
) -> Result<Vec<agentflow_mcp::McpSessionSnapshot>, String> {
    agentflow_mcp::poll_session_snapshots(project_root).map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn load_mcp_session_log_chunk(
    project_root: String,
    session_id: String,
    cursor: Option<String>,
) -> Result<agentflow_mcp::McpLogChunk, String> {
    agentflow_mcp::load_session_log_chunk(project_root, &session_id, cursor.as_deref())
        .map_err(|error| error.to_string())
}
