use crate::{
    locale::expected_locale_state,
    model::{AgentEnvironmentState, AgentEnvironmentStatus},
    ownership::check_agentflow_workspace_ownership_at,
    repair::repair_agent_working_manual_with_locale,
    templates::{BOOTSTRAP_RELATIVE_PATH, VALIDATION_RELATIVE_PATH},
    validate::{
        canonical_project_root, validate_agent_working_manual,
        validate_agent_working_manual_with_context,
    },
};
use anyhow::{anyhow, Result};
use std::{fs, path::Path};

pub fn prepare_agent_working_manual(
    project_root: impl AsRef<Path>,
) -> Result<AgentEnvironmentStatus> {
    prepare_agent_working_manual_with_locale(project_root, None)
}

pub fn prepare_agent_working_manual_with_locale(
    project_root: impl AsRef<Path>,
    app_locale: Option<String>,
) -> Result<AgentEnvironmentStatus> {
    let status = validate_agent_working_manual_with_context(
        project_root.as_ref(),
        Vec::new(),
        None,
        app_locale.as_deref(),
    )?;
    if matches!(
        status.status,
        AgentEnvironmentState::Ready
            | AgentEnvironmentState::Repaired
            | AgentEnvironmentState::Degraded
    ) {
        return Ok(status);
    }
    repair_agent_working_manual_with_locale(project_root, app_locale)
}

pub fn load_agent_environment_status(
    project_root: impl AsRef<Path>,
) -> Result<AgentEnvironmentStatus> {
    let root = canonical_project_root(project_root.as_ref())?;
    let ownership = check_agentflow_workspace_ownership_at(&root);
    if ownership.agent_blocked {
        return validate_agent_working_manual(&root);
    }

    let bootstrap_path = root.join(BOOTSTRAP_RELATIVE_PATH);
    let validation_path = root.join(VALIDATION_RELATIVE_PATH);
    if bootstrap_path.exists() && validation_path.exists() {
        let raw = fs::read_to_string(&validation_path)?;
        if let Ok(status) = serde_json::from_str::<AgentEnvironmentStatus>(&raw) {
            let expected_locale = expected_locale_state(&root, None, status.checked_at);
            if status.locale.agent_locale == expected_locale.agent_locale {
                return Ok(status);
            }
        }
    }
    validate_agent_working_manual(root)
}

pub fn assert_agent_environment_ready(
    project_root: impl AsRef<Path>,
) -> Result<AgentEnvironmentStatus> {
    let status = prepare_agent_working_manual(project_root)?;
    if status.ready {
        return Ok(status);
    }
    Err(anyhow!(
        "Agent working environment is not ready: {:?}: {}",
        status.status,
        status.errors.join("; ")
    ))
}
