//! Project Workspace Manager backend.
//!
//! This module prepares local `.agentflow/` workspace metadata and keeps source
//! projects protected from AgentFlow runtime files.

mod base_release;
mod commands;
mod dedupe;
mod git;
mod ignore;
mod model;
mod prepare;
mod remove;

pub(crate) use commands::{load_project_initialization_status, prepare_local_project_workspace};
pub(crate) use model::{ProjectInitializationSummary, ProjectWorkspaceSummary};
