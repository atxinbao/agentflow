use crate::{
    layout::{
        detect_shadow_files, prepare_workspace_layout, shadow_warnings, validate_workspace_layout,
    },
    locale::expected_locale_state,
    lock::expected_skills_lock,
    model::{
        AgentEnvironmentState, AgentEnvironmentStatus, RootAgentEntryShadowGuardStatus,
        STATUS_VERSION,
    },
    ownership::check_agentflow_workspace_ownership_at,
    style::expected_style_state,
    templates::{
        agent_entry_template, agentflow_manual_template, skill_templates,
        AGENT_ENTRY_RELATIVE_PATH, AGENT_MANUAL_RELATIVE_PATH, SKILLS_LOCK_RELATIVE_PATH,
    },
    validate::{
        canonical_project_root, external_symlink_error, unix_timestamp_seconds,
        validate_agent_working_manual_with_context, write_state_files,
    },
};
use anyhow::Result;
use std::{fs, path::Path};

pub fn repair_agent_working_manual(
    project_root: impl AsRef<Path>,
) -> Result<AgentEnvironmentStatus> {
    repair_agent_working_manual_with_locale(project_root, None)
}

pub fn repair_agent_working_manual_with_locale(
    project_root: impl AsRef<Path>,
    app_locale: Option<String>,
) -> Result<AgentEnvironmentStatus> {
    let root = canonical_project_root(project_root.as_ref())?;
    let repaired_at = unix_timestamp_seconds();
    let mut repairs = Vec::new();
    let ownership = check_agentflow_workspace_ownership_at(&root);
    if ownership.agent_blocked {
        return Ok(ownership_blocked_status(&root, ownership, repaired_at));
    }

    if let Some(error) = external_symlink_error(&root, &root.join(AGENT_ENTRY_RELATIVE_PATH))? {
        let status = blocked_status(&root, error, repaired_at);
        write_state_files(&root, &status, &status).ok();
        return Ok(status);
    }

    let shadow_guard = detect_shadow_files(&root);
    let warnings = shadow_warnings(&shadow_guard);
    let locale = expected_locale_state(&root, app_locale.as_deref(), repaired_at);
    let style = expected_style_state(repaired_at);
    prepare_workspace_layout(&root, &warnings, &mut repairs, &locale, &style)?;

    write_agent_entry(&root, &mut repairs)?;
    write_file_if_changed(
        &root.join(AGENT_MANUAL_RELATIVE_PATH),
        &agentflow_manual_template(),
        "Rewrote Agentflow.md",
        &mut repairs,
    )?;

    for skill in skill_templates() {
        write_file_if_changed(
            &root.join(skill.relative_path),
            skill.content,
            &format!("Rewrote skill {}", skill.name),
            &mut repairs,
        )?;
    }

    let lock = expected_skills_lock(repaired_at, &locale);
    write_file_if_changed(
        &root.join(SKILLS_LOCK_RELATIVE_PATH),
        &(serde_json::to_string_pretty(&lock)? + "\n"),
        "Rewrote skills-lock.json",
        &mut repairs,
    )?;
    crate::validate::write_policy_state_files(&root, &locale, &style)?;

    let status = validate_agent_working_manual_with_context(
        &root,
        repairs,
        Some(repaired_at),
        app_locale.as_deref(),
    )?;
    write_state_files(&root, &status, &status)?;
    Ok(status)
}

fn write_agent_entry(root: &Path, repairs: &mut Vec<String>) -> Result<()> {
    let path = root.join(AGENT_ENTRY_RELATIVE_PATH);
    if path.exists() {
        repairs.push("Kept existing local AGENTS.md unchanged.".to_string());
        return Ok(());
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, agent_entry_template())?;
    repairs.push("Created local AGENTS.md from AgentFlow default entry.".to_string());
    Ok(())
}

fn write_file_if_changed(
    path: &Path,
    content: &str,
    repair_message: &str,
    repairs: &mut Vec<String>,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    if fs::read_to_string(path).ok().as_deref() == Some(content) {
        return Ok(());
    }
    fs::write(path, content)?;
    repairs.push(repair_message.to_string());
    Ok(())
}

fn blocked_status(root: &Path, error: String, checked_at: u64) -> AgentEnvironmentStatus {
    let (workspace_manifest, layout) = validate_workspace_layout(root).unwrap_or_else(|_| {
        (
            crate::model::WorkspaceManifestStatus {
                exists: false,
                path: ".agentflow/workspace-manifest.json".to_string(),
                valid: false,
                layout_version: None,
            },
            crate::model::WorkspaceLayoutStatus {
                version: crate::model::WORKSPACE_LAYOUT_VERSION.to_string(),
                ready: false,
                created_paths: Vec::new(),
                reused_paths: Vec::new(),
                missing_paths: Vec::new(),
            },
        )
    });
    AgentEnvironmentStatus {
        version: STATUS_VERSION.to_string(),
        project_root: root.display().to_string(),
        status: AgentEnvironmentState::Blocked,
        ready: false,
        checked_at,
        repaired_at: Some(checked_at),
        agent_md: crate::model::AgentMdStatus {
            exists: root.join(AGENT_ENTRY_RELATIVE_PATH).exists(),
            managed: false,
            version: None,
            hash: None,
            backed_up: false,
            tracked_by_git: false,
        },
        manual: crate::model::ManualStatus {
            exists: false,
            path: AGENT_MANUAL_RELATIVE_PATH.to_string(),
            hash: None,
        },
        skills_lock: crate::model::SkillsLockStatus {
            exists: false,
            valid: false,
            path: SKILLS_LOCK_RELATIVE_PATH.to_string(),
            skill_count: 0,
        },
        skills: Vec::new(),
        repairs: Vec::new(),
        warnings: Vec::new(),
        errors: vec![error],
        workspace_manifest,
        ownership: check_agentflow_workspace_ownership_at(root),
        layout,
        locale: crate::locale::expected_locale_state(root, None, checked_at),
        style: crate::style::expected_style_state(checked_at),
        shadow_guard: RootAgentEntryShadowGuardStatus {
            checked: Vec::new(),
            detected: Vec::new(),
        },
    }
}

fn ownership_blocked_status(
    root: &Path,
    ownership: crate::model::WorkspaceOwnershipStatus,
    checked_at: u64,
) -> AgentEnvironmentStatus {
    AgentEnvironmentStatus {
        version: STATUS_VERSION.to_string(),
        project_root: root.display().to_string(),
        status: AgentEnvironmentState::Blocked,
        ready: false,
        checked_at,
        repaired_at: Some(checked_at),
        agent_md: crate::model::AgentMdStatus {
            exists: root.join(AGENT_ENTRY_RELATIVE_PATH).exists(),
            managed: false,
            version: None,
            hash: None,
            backed_up: false,
            tracked_by_git: false,
        },
        manual: crate::model::ManualStatus {
            exists: root.join(AGENT_MANUAL_RELATIVE_PATH).exists(),
            path: AGENT_MANUAL_RELATIVE_PATH.to_string(),
            hash: None,
        },
        skills_lock: crate::model::SkillsLockStatus {
            exists: root.join(SKILLS_LOCK_RELATIVE_PATH).exists(),
            valid: false,
            path: SKILLS_LOCK_RELATIVE_PATH.to_string(),
            skill_count: 0,
        },
        skills: Vec::new(),
        repairs: Vec::new(),
        warnings: ownership.warnings.clone(),
        errors: ownership.errors.clone(),
        workspace_manifest: crate::model::WorkspaceManifestStatus {
            exists: root.join(".agentflow/workspace-manifest.json").exists(),
            path: ".agentflow/workspace-manifest.json".to_string(),
            valid: false,
            layout_version: ownership.marker.layout_version.clone(),
        },
        ownership,
        layout: crate::model::WorkspaceLayoutStatus {
            version: crate::model::WORKSPACE_LAYOUT_VERSION.to_string(),
            ready: false,
            created_paths: Vec::new(),
            reused_paths: Vec::new(),
            missing_paths: Vec::new(),
        },
        locale: crate::locale::expected_locale_state(root, None, checked_at),
        style: crate::style::expected_style_state(checked_at),
        shadow_guard: RootAgentEntryShadowGuardStatus {
            checked: Vec::new(),
            detected: Vec::new(),
        },
    }
}
