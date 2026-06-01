use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProjectWorkspaceSummary {
    pub(crate) version: String,
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) root: String,
    pub(crate) agentflow_path: String,
    pub(crate) workspace_path: String,
    pub(crate) config_path: String,
    pub(crate) created_agentflow: bool,
    pub(crate) created_paths: Vec<String>,
    pub(crate) reused_paths: Vec<String>,
    pub(crate) git_exclude_path: Option<String>,
    pub(crate) protected_git_exclude: bool,
}
