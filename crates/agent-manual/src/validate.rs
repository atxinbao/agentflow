use crate::{
    git::is_git_tracked,
    hash::file_sha256_hex,
    layout::{detect_shadow_files, shadow_warnings, validate_workspace_layout},
    lock::{expected_skills_lock, read_skills_lock},
    model::{
        AgentEnvironmentState, AgentEnvironmentStatus, AgentMdStatus, ManualStatus, SkillStatus,
        SkillsLockStatus, AGENT_ENTRY_VERSION, STATUS_VERSION,
    },
    templates::{
        skill_templates, AGENT_ENTRY_RELATIVE_PATH, AGENT_MANUAL_RELATIVE_PATH,
        BOOTSTRAP_RELATIVE_PATH, LEGACY_AGENT_ENTRY_RELATIVE_PATH, SKILLS_LOCK_RELATIVE_PATH,
        VALIDATION_RELATIVE_PATH,
    },
};
use anyhow::{anyhow, Result};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

pub fn validate_agent_working_manual(
    project_root: impl AsRef<Path>,
) -> Result<AgentEnvironmentStatus> {
    validate_agent_working_manual_with_context(project_root.as_ref(), Vec::new(), None)
}

pub(crate) fn validate_agent_working_manual_with_context(
    project_root: &Path,
    repairs: Vec<String>,
    repaired_at: Option<u64>,
) -> Result<AgentEnvironmentStatus> {
    let root = canonical_project_root(project_root)?;
    let checked_at = unix_timestamp_seconds();
    let mut warnings = Vec::new();
    let mut errors = Vec::new();
    let mut blocked = false;

    let shadow_guard = detect_shadow_files(&root);
    warnings.extend(shadow_warnings(&shadow_guard));

    let agent_md_path = root.join(AGENT_ENTRY_RELATIVE_PATH);
    if let Some(message) = external_symlink_error(&root, &agent_md_path)? {
        errors.push(message);
        blocked = true;
    } else if let Some(message) = internal_symlink_warning(&root, &agent_md_path)? {
        warnings.push(message);
    }

    let tracked_by_git = is_git_tracked(&root, AGENT_ENTRY_RELATIVE_PATH);
    if tracked_by_git {
        warnings.push(
            "AGENTS.md is tracked by Git. AgentFlow rewrote it as the managed Agent entry. Review your Git diff before committing."
                .to_string(),
        );
    }

    let agent_md_exists = agent_md_path.exists();
    let agent_md_content = fs::read_to_string(&agent_md_path).ok();
    let managed = agent_md_content
        .as_deref()
        .map(|content| content.contains("<!-- AGENTFLOW:MANAGED version=agent-entry.v2 -->"))
        .unwrap_or(false);
    let agent_md_version = managed.then(|| AGENT_ENTRY_VERSION.to_string());
    if !agent_md_exists {
        errors.push("AGENTS.md is missing.".to_string());
    } else if !managed {
        errors.push("AGENTS.md is not managed by AgentFlow.".to_string());
    }

    let expected_lock = expected_skills_lock(checked_at);
    let agent_hash = file_sha256_hex(&agent_md_path);
    if agent_md_exists && agent_hash.as_deref() != Some(expected_lock.entry.hash.as_str()) {
        errors.push("AGENTS.md hash does not match AgentFlow managed template.".to_string());
    }

    let (workspace_manifest, layout) = validate_workspace_layout(&root)?;
    if !workspace_manifest.exists {
        errors.push(".agentflow/workspace-manifest.json is missing.".to_string());
    } else if !workspace_manifest.valid {
        errors.push(".agentflow/workspace-manifest.json is invalid.".to_string());
    }
    if !layout.ready {
        for path in &layout.missing_paths {
            errors.push(format!(
                "AgentFlow workspace layout path is missing: {path}"
            ));
        }
    }

    let manual_path = root.join(AGENT_MANUAL_RELATIVE_PATH);
    let manual_exists = manual_path.exists();
    let manual_hash = file_sha256_hex(&manual_path);
    if !manual_exists {
        errors.push("Agentflow.md is missing.".to_string());
    } else if manual_hash.as_deref() != Some(expected_lock.manual.hash.as_str()) {
        errors.push("Agentflow.md hash mismatch.".to_string());
    }

    let lock_path = root.join(SKILLS_LOCK_RELATIVE_PATH);
    let lock = read_skills_lock(&lock_path).ok();
    let lock_exists = lock_path.exists();
    let mut lock_valid = lock_exists && lock.is_some();
    if !lock_exists {
        errors.push("skills-lock.json is missing.".to_string());
        lock_valid = false;
    } else if lock.is_none() {
        errors.push("skills-lock.json format is invalid.".to_string());
        lock_valid = false;
    }

    if let Some(lock) = &lock {
        if lock.version != expected_lock.version || lock.managed_by != expected_lock.managed_by {
            errors.push("skills-lock.json header mismatch.".to_string());
            lock_valid = false;
        }
        if lock.entry.hash != expected_lock.entry.hash
            || lock.entry.version != expected_lock.entry.version
        {
            errors.push("skills-lock.json entry hash mismatch.".to_string());
            lock_valid = false;
        }
        if lock.manual.hash != expected_lock.manual.hash
            || lock.manual.version != expected_lock.manual.version
        {
            errors.push("skills-lock.json manual hash mismatch.".to_string());
            lock_valid = false;
        }
        if lock.skills.len() != expected_lock.skills.len() {
            errors.push("skills-lock.json skill count mismatch.".to_string());
            lock_valid = false;
        }
    }

    let bootstrap_state_path = root.join(BOOTSTRAP_RELATIVE_PATH);
    let validation_state_path = root.join(VALIDATION_RELATIVE_PATH);
    let bootstrap_state_exists = bootstrap_state_path.exists();
    let validation_state_exists = validation_state_path.exists();
    if repaired_at.is_none() {
        if !bootstrap_state_exists {
            errors.push(format!(
                "Agent Manual bootstrap state is missing: {}",
                BOOTSTRAP_RELATIVE_PATH
            ));
        }
        if !validation_state_exists {
            errors.push(format!(
                "Agent Manual validation state is missing: {}",
                VALIDATION_RELATIVE_PATH
            ));
        }
    }

    let skills = skill_templates()
        .into_iter()
        .map(|skill| {
            let path = root.join(skill.relative_path);
            let exists = path.exists();
            let expected_hash = expected_lock
                .skills
                .get(skill.name)
                .map(|item| item.hash.as_str());
            let actual_hash = file_sha256_hex(&path);
            let lock_hash = lock
                .as_ref()
                .and_then(|value| value.skills.get(skill.name))
                .map(|item| item.hash.as_str());
            let hash_matches =
                exists && actual_hash.as_deref() == expected_hash && lock_hash == expected_hash;
            if !exists {
                errors.push(format!("Skill {} is missing.", skill.name));
            } else if !hash_matches {
                errors.push(format!("Skill {} hash mismatch.", skill.name));
            }
            SkillStatus {
                name: skill.name.to_string(),
                path: skill.relative_path.to_string(),
                exists,
                hash_matches,
                version: "v1".to_string(),
            }
        })
        .collect::<Vec<_>>();

    let state = if blocked {
        AgentEnvironmentState::Blocked
    } else if !errors.is_empty() {
        if !agent_md_exists
            || !manual_exists
            || !lock_exists
            || !bootstrap_state_exists
            || !validation_state_exists
            || !workspace_manifest.exists
            || !layout.ready
        {
            AgentEnvironmentState::Missing
        } else {
            AgentEnvironmentState::Failed
        }
    } else if repaired_at.is_some() {
        AgentEnvironmentState::Repaired
    } else if !warnings.is_empty() {
        AgentEnvironmentState::Degraded
    } else {
        AgentEnvironmentState::Ready
    };
    let ready = matches!(
        state,
        AgentEnvironmentState::Ready
            | AgentEnvironmentState::Repaired
            | AgentEnvironmentState::Degraded
    );

    let status = AgentEnvironmentStatus {
        version: STATUS_VERSION.to_string(),
        project_root: root.display().to_string(),
        status: state,
        ready,
        checked_at,
        repaired_at,
        agent_md: AgentMdStatus {
            exists: agent_md_exists,
            managed,
            version: agent_md_version,
            hash: agent_hash,
            backed_up: repairs
                .iter()
                .any(|repair| repair.contains("Backed up existing AGENTS.md")),
            tracked_by_git,
        },
        manual: ManualStatus {
            exists: manual_exists,
            path: AGENT_MANUAL_RELATIVE_PATH.to_string(),
            hash: manual_hash,
        },
        skills_lock: SkillsLockStatus {
            exists: lock_exists,
            valid: lock_valid
                && errors
                    .iter()
                    .all(|error| !error.contains("skills-lock.json")),
            path: SKILLS_LOCK_RELATIVE_PATH.to_string(),
            skill_count: lock
                .as_ref()
                .map(|value| value.skills.len())
                .unwrap_or_else(|| expected_lock.skills.len()),
        },
        skills,
        repairs,
        warnings,
        errors,
        workspace_manifest,
        layout,
        legacy_agent_entry: crate::model::LegacyAgentEntryStatus {
            exists: root.join(LEGACY_AGENT_ENTRY_RELATIVE_PATH).exists(),
            path: LEGACY_AGENT_ENTRY_RELATIVE_PATH.to_string(),
            managed: fs::read_to_string(root.join(LEGACY_AGENT_ENTRY_RELATIVE_PATH))
                .map(|content| content.contains("AGENTFLOW:MANAGED"))
                .unwrap_or(false),
        },
        shadow_guard,
    };

    Ok(status)
}

pub(crate) fn write_state_files(
    root: &Path,
    bootstrap: &AgentEnvironmentStatus,
    validation: &AgentEnvironmentStatus,
) -> Result<()> {
    write_json(root.join(BOOTSTRAP_RELATIVE_PATH), bootstrap)?;
    write_json(root.join(VALIDATION_RELATIVE_PATH), validation)?;
    Ok(())
}

fn write_json(path: PathBuf, value: &AgentEnvironmentStatus) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_string_pretty(value)? + "\n")?;
    Ok(())
}

pub(crate) fn canonical_project_root(project_root: &Path) -> Result<PathBuf> {
    let root = project_root
        .canonicalize()
        .map_err(|error| anyhow!("canonicalize selected project root: {error}"))?;
    if !root.is_dir() {
        return Err(anyhow!("selected project root is not a directory"));
    }
    Ok(root)
}

pub(crate) fn unix_timestamp_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

pub(crate) fn external_symlink_error(root: &Path, path: &Path) -> Result<Option<String>> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(_) => return Ok(None),
    };
    if !metadata.file_type().is_symlink() {
        return Ok(None);
    }

    let target = fs::read_link(path)?;
    let resolved = if target.is_absolute() {
        target
    } else {
        path.parent().unwrap_or(root).join(target)
    };
    let canonical_target = resolved.canonicalize().unwrap_or(resolved);
    if canonical_target.starts_with(root) {
        return Ok(None);
    }

    Ok(Some(format!(
        "{} is a symlink outside project root: {}",
        path.file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("Agent entry"),
        canonical_target.display()
    )))
}

pub(crate) fn internal_symlink_warning(root: &Path, path: &Path) -> Result<Option<String>> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(_) => return Ok(None),
    };
    if !metadata.file_type().is_symlink() {
        return Ok(None);
    }

    let target = fs::read_link(path)?;
    let resolved = if target.is_absolute() {
        target
    } else {
        path.parent().unwrap_or(root).join(target)
    };
    let canonical_target = resolved.canonicalize().unwrap_or(resolved);
    if !canonical_target.starts_with(root) {
        return Ok(None);
    }

    Ok(Some(format!(
        "{} is a symlink inside project root: {}",
        path.file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("Agent entry"),
        canonical_target.display()
    )))
}
