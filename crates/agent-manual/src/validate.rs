use crate::{
    git::is_git_tracked,
    hash::file_sha256_hex,
    layout::{detect_shadow_files, shadow_warnings, validate_workspace_layout},
    locale::{expected_locale_state, read_locale_state},
    lock::{expected_skills_lock, read_skills_lock},
    model::{
        AgentEnvironmentState, AgentEnvironmentStatus, AgentMdStatus, AgentStyleState,
        ManualStatus, SkillStatus, SkillsLockStatus, AGENT_ENTRY_VERSION, LOCALE_VERSION,
        MANUAL_LANGUAGE, PLAIN_WORK_STYLE_ID, STATUS_VERSION, STYLE_VERSION,
    },
    ownership::check_agentflow_workspace_ownership_at,
    style::expected_style_state,
    templates::{
        skill_templates, AGENT_ENTRY_RELATIVE_PATH, AGENT_MANUAL_RELATIVE_PATH,
        BOOTSTRAP_RELATIVE_PATH, LOCALE_RELATIVE_PATH, SKILLS_LOCK_RELATIVE_PATH,
        STYLE_RELATIVE_PATH, VALIDATION_RELATIVE_PATH,
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
    validate_agent_working_manual_with_context(project_root.as_ref(), Vec::new(), None, None)
}

pub(crate) fn validate_agent_working_manual_with_context(
    project_root: &Path,
    repairs: Vec<String>,
    repaired_at: Option<u64>,
    app_locale: Option<&str>,
) -> Result<AgentEnvironmentStatus> {
    let root = canonical_project_root(project_root)?;
    let checked_at = unix_timestamp_seconds();
    let mut warnings = Vec::new();
    let mut errors = Vec::new();
    let mut blocked = false;

    let ownership = check_agentflow_workspace_ownership_at(&root);
    warnings.extend(ownership.warnings.clone());
    if ownership.agent_blocked {
        blocked = true;
        errors.extend(ownership.errors.clone());
    }

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
            "AGENTS.md is tracked by Git. Keep it local with git rm --cached AGENTS.md and leave the file ignored."
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
        warnings.push(
            "AGENTS.md is local and not AgentFlow-managed; existing content is preserved."
                .to_string(),
        );
    }

    let expected_locale = expected_locale_state(&root, app_locale, checked_at);
    let expected_style = expected_style_state(checked_at);

    let locale_path = root.join(LOCALE_RELATIVE_PATH);
    let locale_state = read_locale_state(&root);
    if !locale_path.exists() {
        errors.push(format!(
            "Agent locale state is missing: {LOCALE_RELATIVE_PATH}"
        ));
    }
    if let Some(locale) = &locale_state {
        if locale.version != LOCALE_VERSION {
            errors.push("locale.json version mismatch.".to_string());
        }
        if locale.manual_language != MANUAL_LANGUAGE {
            errors.push("locale.json manualLanguage must be en.".to_string());
        }
        if locale.agent_locale.trim().is_empty() {
            errors.push("locale.json agentLocale is missing.".to_string());
        }
        if locale.agent_locale != expected_locale.agent_locale {
            errors.push("locale.json agentLocale does not match detected locale.".to_string());
        }
    } else if locale_path.exists() {
        errors.push("locale.json format is invalid.".to_string());
    }

    let style_path = root.join(STYLE_RELATIVE_PATH);
    let style_state = read_style_state(&style_path);
    if !style_path.exists() {
        errors.push(format!(
            "Agent style state is missing: {STYLE_RELATIVE_PATH}"
        ));
    }
    if let Some(style) = &style_state {
        if style.version != STYLE_VERSION {
            errors.push("style.json version mismatch.".to_string());
        }
        if style.style_id != PLAIN_WORK_STYLE_ID {
            errors.push("style.json styleId must be plain-work-style.".to_string());
        }
        if style.manual_language != MANUAL_LANGUAGE {
            errors.push("style.json manualLanguage must be en.".to_string());
        }
        if !style.applies_to_code_comments {
            errors.push("style.json appliesToCodeComments must be true.".to_string());
        }
    } else if style_path.exists() {
        errors.push("style.json format is invalid.".to_string());
    }

    let locale_status = locale_state.unwrap_or_else(|| expected_locale.clone());
    let style_status = style_state.unwrap_or_else(|| expected_style.clone());
    let expected_lock = expected_skills_lock(checked_at, &expected_locale);
    let agent_hash = file_sha256_hex(&agent_md_path);
    if agent_md_exists
        && managed
        && agent_hash.as_deref() != Some(expected_lock.entry.hash.as_str())
    {
        warnings.push(
            "AGENTS.md differs from AgentFlow default entry; existing local content is preserved."
                .to_string(),
        );
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
        if lock.manual_language.as_deref() != Some(MANUAL_LANGUAGE) {
            errors.push(
                "skills-lock.json manualLanguage metadata is missing or invalid.".to_string(),
            );
            lock_valid = false;
        }
        if lock.agent_locale.as_deref() != Some(expected_locale.agent_locale.as_str()) {
            errors.push("skills-lock.json agentLocale metadata mismatch.".to_string());
            lock_valid = false;
        }
        match &lock.style_policy {
            Some(style_policy)
                if style_policy.style_id == PLAIN_WORK_STYLE_ID
                    && style_policy.version == "v1"
                    && style_policy.path
                        == ".agentflow/define/agent/skills/plain-work-style/SKILL.md"
                    && style_policy.applies_to_code_comments => {}
            _ => {
                errors.push(
                    "skills-lock.json stylePolicy metadata is missing or invalid.".to_string(),
                );
                lock_valid = false;
            }
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
            || !locale_path.exists()
            || !style_path.exists()
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
            backed_up: false,
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
        ownership,
        layout,
        locale: locale_status,
        style: style_status,
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

pub(crate) fn write_policy_state_files(
    root: &Path,
    locale: &crate::model::AgentLocaleState,
    style: &AgentStyleState,
) -> Result<()> {
    write_policy_json(root.join(LOCALE_RELATIVE_PATH), locale)?;
    write_policy_json(root.join(STYLE_RELATIVE_PATH), style)?;
    Ok(())
}

fn write_json(path: PathBuf, value: &AgentEnvironmentStatus) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_string_pretty(value)? + "\n")?;
    Ok(())
}

fn write_policy_json<T: serde::Serialize>(path: PathBuf, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_string_pretty(value)? + "\n")?;
    Ok(())
}

fn read_style_state(path: &Path) -> Option<AgentStyleState> {
    let raw = fs::read_to_string(path).ok()?;
    serde_json::from_str(&raw).ok()
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
