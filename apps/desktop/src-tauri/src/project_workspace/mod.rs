//! Project Workspace Manager backend.
//!
//! This module prepares local `.agentflow/` workspace metadata and keeps source
//! projects protected from AgentFlow runtime files.

mod commands;
mod dedupe;
mod git;
mod ignore;
mod model;
mod prepare;
mod remove;

pub(crate) use commands::prepare_local_project_workspace;
pub(crate) use model::ProjectWorkspaceSummary;
