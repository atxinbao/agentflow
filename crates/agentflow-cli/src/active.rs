//! Active CLI boundary.
//!
//! Active commands are narrow wrappers around the current workspace crates.
//! They must not call archived 2026-05 writers.

use agentflow_execute::{BuildAgentCompletion, BuildAgentCompletionRequest};
use anyhow::{Context, Result};
use std::{fs, path::Path};

pub(crate) fn complete_build_agent_issue_from_request(
    root: &Path,
    request_path: &Path,
) -> Result<BuildAgentCompletion> {
    let raw = fs::read_to_string(request_path)
        .with_context(|| format!("read completion request {}", request_path.display()))?;
    let request: BuildAgentCompletionRequest = serde_json::from_str(&raw)
        .with_context(|| format!("parse completion request {}", request_path.display()))?;
    let completion = agentflow_execute::complete_build_agent_issue(root, request)?;
    agentflow_state::refresh_state(root)?;
    Ok(completion)
}
