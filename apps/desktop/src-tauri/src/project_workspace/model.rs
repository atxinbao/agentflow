use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProjectInitializationContext {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) summary: String,
    pub(crate) committed_at: Option<String>,
    pub(crate) author: Option<String>,
    pub(crate) changed_files: Vec<String>,
    pub(crate) source_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProjectInitializationSummary {
    pub(crate) version: String,
    pub(crate) project_kind: String,
    pub(crate) initialized: bool,
    pub(crate) demo_data_created: bool,
    pub(crate) git_context_loaded: bool,
    pub(crate) recent_context_count: usize,
    pub(crate) demo_issue_count: usize,
    pub(crate) demo_delivery_count: usize,
    pub(crate) demo_audit_count: usize,
    pub(crate) message: String,
    pub(crate) paths: Vec<String>,
    pub(crate) warnings: Vec<String>,
    pub(crate) recent_context: Vec<ProjectInitializationContext>,
}

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
    pub(crate) agents_gitignore_path: Option<String>,
    pub(crate) protected_agents_gitignore: bool,
    pub(crate) agents_md_tracked_by_git: bool,
    pub(crate) agents_md_git_warning: Option<String>,
    pub(crate) ownership: agentflow_agent_manual::model::WorkspaceOwnershipStatus,
    pub(crate) agent_manual_status: agentflow_agent_manual::model::AgentEnvironmentStatus,
    pub(crate) state_status: Option<agentflow_state::StateStatusSnapshot>,
    pub(crate) initialization_status: Option<ProjectInitializationSummary>,
}
