use crate::{
    lock::expected_skills_lock,
    model::{AgentEnvironmentState, AgentEnvironmentStatus, STATUS_VERSION},
    templates::{
        agent_md_template, agentflow_manual_template, skill_templates, AGENT_MANUAL_RELATIVE_PATH,
        SKILLS_LOCK_RELATIVE_PATH,
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
    let root = canonical_project_root(project_root.as_ref())?;
    let repaired_at = unix_timestamp_seconds();
    let mut repairs = Vec::new();

    if let Some(error) = external_symlink_error(&root, &root.join("AGENT.MD"))? {
        let status = blocked_status(&root, error, repaired_at);
        write_state_files(&root, &status, &status).ok();
        return Ok(status);
    }

    ensure_directory(&root.join(".agentflow/define/agent/state"), &mut repairs)?;
    ensure_directory(&root.join(".agentflow/define/agent/skills"), &mut repairs)?;
    ensure_directory(
        &root.join(".agentflow/output/backup/agent-md"),
        &mut repairs,
    )?;
    ensure_directory(&root.join(".agentflow/output/logs"), &mut repairs)?;

    write_agent_md(&root, &mut repairs, repaired_at)?;
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

    let lock = expected_skills_lock(repaired_at);
    write_file_if_changed(
        &root.join(SKILLS_LOCK_RELATIVE_PATH),
        &(serde_json::to_string_pretty(&lock)? + "\n"),
        "Rewrote skills-lock.json",
        &mut repairs,
    )?;

    let status = validate_agent_working_manual_with_context(&root, repairs, Some(repaired_at))?;
    write_state_files(&root, &status, &status)?;
    Ok(status)
}

fn ensure_directory(path: &Path, repairs: &mut Vec<String>) -> Result<()> {
    if path.exists() {
        return Ok(());
    }
    fs::create_dir_all(path)?;
    repairs.push(format!("Created directory {}", path.display()));
    Ok(())
}

fn write_agent_md(root: &Path, repairs: &mut Vec<String>, timestamp: u64) -> Result<()> {
    let path = root.join("AGENT.MD");
    let desired = agent_md_template();
    let current = fs::read_to_string(&path).ok();
    if current.as_deref() == Some(desired.as_str()) {
        return Ok(());
    }

    if let Some(content) = current {
        let backup_path = root
            .join(".agentflow/output/backup/agent-md")
            .join(format!("AGENT.MD.{timestamp}.bak.md"));
        fs::write(&backup_path, content)?;
        repairs.push(format!(
            "Backed up existing AGENT.MD to {}",
            backup_path
                .strip_prefix(root)
                .unwrap_or(&backup_path)
                .display()
        ));
    }

    fs::write(&path, desired)?;
    repairs.push("Rewrote AGENT.MD as AgentFlow managed entry.".to_string());
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
    AgentEnvironmentStatus {
        version: STATUS_VERSION.to_string(),
        project_root: root.display().to_string(),
        status: AgentEnvironmentState::Blocked,
        ready: false,
        checked_at,
        repaired_at: Some(checked_at),
        agent_md: crate::model::AgentMdStatus {
            exists: root.join("AGENT.MD").exists(),
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
    }
}
