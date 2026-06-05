use crate::{
    layout::prepare_workspace_layout,
    locale::expected_locale_state,
    model::{
        WorkspaceOwnershipAction, WorkspaceOwnershipMarker, WorkspaceOwnershipState,
        WorkspaceOwnershipStatus, WORKSPACE_LAYOUT_VERSION, WORKSPACE_MANAGED_BY,
        WORKSPACE_MANIFEST_VERSION, WORKSPACE_OWNERSHIP_VERSION,
    },
    style::expected_style_state,
    templates::{
        AGENT_MANUAL_RELATIVE_PATH, LEGACY_AGENT_ENTRY_RELATIVE_PATH, SKILLS_LOCK_RELATIVE_PATH,
        WORKSPACE_MANIFEST_RELATIVE_PATH,
    },
    validate::canonical_project_root,
};
use anyhow::{anyhow, Context, Result};
use serde_json::Value;
use std::{
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

pub fn check_agentflow_workspace_ownership(
    project_root: impl AsRef<Path>,
) -> Result<WorkspaceOwnershipStatus> {
    let root = canonical_project_root(project_root.as_ref())?;
    Ok(check_agentflow_workspace_ownership_at(&root))
}

pub fn assert_agentflow_workspace_owned_or_creatable(
    project_root: impl AsRef<Path>,
) -> Result<WorkspaceOwnershipStatus> {
    let status = check_agentflow_workspace_ownership(project_root)?;
    if status.ready_for_prepare {
        return Ok(status);
    }

    Err(anyhow!(
        "AgentFlow workspace ownership is not safe for prepare: {:?}: {}",
        status.status,
        status.errors.join("; ")
    ))
}

pub fn take_over_agentflow_workspace(
    project_root: impl AsRef<Path>,
) -> Result<WorkspaceOwnershipStatus> {
    let root = canonical_project_root(project_root.as_ref())?;
    let current = check_agentflow_workspace_ownership_at(&root);
    if current.status != WorkspaceOwnershipState::Foreign {
        return Err(anyhow!(
            "AgentFlow workspace takeover requires foreign ownership status, found {:?}",
            current.status
        ));
    }

    let agentflow_path = root.join(".agentflow");
    let backup_path = root.join(format!(
        ".agentflow.unmanaged.{}.bak",
        unix_timestamp_seconds()
    ));
    fs::rename(&agentflow_path, &backup_path).with_context(|| {
        format!(
            "rename foreign {} to {}",
            agentflow_path.display(),
            backup_path.display()
        )
    })?;

    let mut repairs = vec![format!(
        "Backed up foreign .agentflow to {}",
        backup_path.display()
    )];
    let warning = format!("Took over foreign .agentflow after explicit user confirmation.");
    let checked_at = unix_timestamp_seconds();
    let locale = expected_locale_state(&root, None, checked_at);
    let style = expected_style_state(checked_at);
    prepare_workspace_layout(&root, &[warning], &mut repairs, &locale, &style)?;

    Ok(check_agentflow_workspace_ownership_at(&root))
}

pub(crate) fn check_agentflow_workspace_ownership_at(root: &Path) -> WorkspaceOwnershipStatus {
    let agentflow_path = root.join(".agentflow");
    let mut warnings = Vec::new();
    let mut errors = Vec::new();
    let mut detected_files = Vec::new();
    let metadata = match fs::symlink_metadata(&agentflow_path) {
        Ok(metadata) => Some(metadata),
        Err(error) if error.kind() == ErrorKind::NotFound => None,
        Err(error) => {
            errors.push(format!("Cannot inspect .agentflow ownership: {}", error));
            return status(
                root,
                &agentflow_path,
                WorkspaceOwnershipState::Blocked,
                marker(false, false, None, None, false, false, false),
                detected_files,
                warnings,
                errors,
                WorkspaceOwnershipAction::Stop,
            );
        }
    };

    let Some(metadata) = metadata else {
        return status(
            root,
            &agentflow_path,
            WorkspaceOwnershipState::None,
            marker(false, false, None, None, false, false, false),
            detected_files,
            warnings,
            errors,
            WorkspaceOwnershipAction::Create,
        );
    };

    if metadata.file_type().is_symlink() {
        match resolve_symlink(root, &agentflow_path) {
            Ok(target) if target.starts_with(root) => {
                warnings.push(format!(
                    ".agentflow is a symlink inside project root: {}",
                    target.display()
                ));
            }
            Ok(target) => {
                errors.push(format!(
                    ".agentflow is a symlink outside project root: {}",
                    target.display()
                ));
                return status(
                    root,
                    &agentflow_path,
                    WorkspaceOwnershipState::Blocked,
                    marker(false, false, None, None, false, false, false),
                    detected_files,
                    warnings,
                    errors,
                    WorkspaceOwnershipAction::Stop,
                );
            }
            Err(error) => {
                errors.push(format!("Cannot resolve .agentflow symlink: {error}"));
                return status(
                    root,
                    &agentflow_path,
                    WorkspaceOwnershipState::Blocked,
                    marker(false, false, None, None, false, false, false),
                    detected_files,
                    warnings,
                    errors,
                    WorkspaceOwnershipAction::Stop,
                );
            }
        }
    } else if !metadata.is_dir() {
        errors.push(".agentflow exists but is not a directory.".to_string());
        return status(
            root,
            &agentflow_path,
            WorkspaceOwnershipState::Blocked,
            marker(false, false, None, None, false, false, false),
            detected_files,
            warnings,
            errors,
            WorkspaceOwnershipAction::Stop,
        );
    }

    if metadata.permissions().readonly() {
        errors.push(".agentflow exists but is read-only.".to_string());
        return status(
            root,
            &agentflow_path,
            WorkspaceOwnershipState::Blocked,
            marker(false, false, None, None, false, false, false),
            detected_files,
            warnings,
            errors,
            WorkspaceOwnershipAction::Stop,
        );
    }

    let manifest_path = root.join(WORKSPACE_MANIFEST_RELATIVE_PATH);
    let manifest = inspect_manifest(&manifest_path);
    let agent_manual_exists = root.join(AGENT_MANUAL_RELATIVE_PATH).is_file();
    let skills_lock_exists = root.join(SKILLS_LOCK_RELATIVE_PATH).is_file();
    let managed_entry_exists = contains_managed_marker(&root.join("AGENTS.md"))
        || contains_managed_marker(&root.join(LEGACY_AGENT_ENTRY_RELATIVE_PATH));

    if manifest.exists {
        detected_files.push(WORKSPACE_MANIFEST_RELATIVE_PATH.to_string());
    }
    if agent_manual_exists {
        detected_files.push(AGENT_MANUAL_RELATIVE_PATH.to_string());
    }
    if skills_lock_exists {
        detected_files.push(SKILLS_LOCK_RELATIVE_PATH.to_string());
    }
    if managed_entry_exists {
        detected_files.push("AGENTS.md or AGENT.MD managed marker".to_string());
    }

    let ownership_marker = marker(
        manifest.exists,
        manifest.managed_by_agentflow,
        manifest.version.clone(),
        manifest.layout_version.clone(),
        agent_manual_exists,
        skills_lock_exists,
        managed_entry_exists,
    );

    if manifest.parse_failed {
        if manifest.raw_mentions_agentflow
            || agent_manual_exists
            || skills_lock_exists
            || managed_entry_exists
        {
            warnings.push(
                ".agentflow/workspace-manifest.json is corrupted but AgentFlow markers were found."
                    .to_string(),
            );
            return status(
                root,
                &agentflow_path,
                WorkspaceOwnershipState::Corrupted,
                ownership_marker,
                detected_files,
                warnings,
                errors,
                WorkspaceOwnershipAction::ValidateRepair,
            );
        }

        errors.push(
            ".agentflow/workspace-manifest.json is corrupted and no AgentFlow marker was found."
                .to_string(),
        );
        return status(
            root,
            &agentflow_path,
            WorkspaceOwnershipState::Blocked,
            ownership_marker,
            detected_files,
            warnings,
            errors,
            WorkspaceOwnershipAction::Stop,
        );
    }

    if manifest.managed_by_agentflow
        && manifest.version.as_deref() == Some(WORKSPACE_MANIFEST_VERSION)
        && manifest.layout_version.as_deref() == Some(WORKSPACE_LAYOUT_VERSION)
    {
        return status(
            root,
            &agentflow_path,
            WorkspaceOwnershipState::ManagedCurrent,
            ownership_marker,
            detected_files,
            warnings,
            errors,
            WorkspaceOwnershipAction::ValidateRepair,
        );
    }

    if manifest.managed_by_agentflow
        || agent_manual_exists
        || skills_lock_exists
        || managed_entry_exists
        || legacy_layout_exists(root)
    {
        warnings.push("Legacy AgentFlow workspace markers were found.".to_string());
        return status(
            root,
            &agentflow_path,
            WorkspaceOwnershipState::ManagedLegacy,
            ownership_marker,
            detected_files,
            warnings,
            errors,
            WorkspaceOwnershipAction::MigrateRepair,
        );
    }

    errors.push(
        "Detected existing .agentflow directory, but no AgentFlow ownership marker was found."
            .to_string(),
    );
    status(
        root,
        &agentflow_path,
        WorkspaceOwnershipState::Foreign,
        ownership_marker,
        detected_files,
        warnings,
        errors,
        WorkspaceOwnershipAction::AskUserToTakeOver,
    )
}

#[derive(Debug)]
struct ManifestInspection {
    exists: bool,
    parse_failed: bool,
    raw_mentions_agentflow: bool,
    managed_by_agentflow: bool,
    version: Option<String>,
    layout_version: Option<String>,
}

fn inspect_manifest(path: &Path) -> ManifestInspection {
    let Ok(raw) = fs::read_to_string(path) else {
        return ManifestInspection {
            exists: path.exists(),
            parse_failed: false,
            raw_mentions_agentflow: false,
            managed_by_agentflow: false,
            version: None,
            layout_version: None,
        };
    };
    let raw_mentions_agentflow = raw.contains(WORKSPACE_MANAGED_BY) || raw.contains("AgentFlow");
    let Ok(value) = serde_json::from_str::<Value>(&raw) else {
        return ManifestInspection {
            exists: true,
            parse_failed: true,
            raw_mentions_agentflow,
            managed_by_agentflow: false,
            version: None,
            layout_version: None,
        };
    };

    ManifestInspection {
        exists: true,
        parse_failed: false,
        raw_mentions_agentflow,
        managed_by_agentflow: value.get("managedBy").and_then(Value::as_str)
            == Some(WORKSPACE_MANAGED_BY),
        version: value
            .get("version")
            .and_then(Value::as_str)
            .map(str::to_string),
        layout_version: value
            .get("layoutVersion")
            .and_then(Value::as_str)
            .map(str::to_string),
    }
}

fn contains_managed_marker(path: &Path) -> bool {
    fs::read_to_string(path)
        .map(|content| content.contains("AGENTFLOW:MANAGED"))
        .unwrap_or(false)
}

fn legacy_layout_exists(root: &Path) -> bool {
    [
        ".agentflow/define/goals",
        ".agentflow/output/graph",
        ".agentflow/graph",
    ]
    .iter()
    .any(|relative| root.join(relative).exists())
        || file_contains(&root.join(".agentflow/workspace.yaml"), "AgentFlow Desktop")
        || file_contains(
            &root.join(".agentflow/config.yaml"),
            "agentflowDir: .agentflow",
        )
}

fn file_contains(path: &Path, marker: &str) -> bool {
    fs::read_to_string(path)
        .map(|content| content.contains(marker))
        .unwrap_or(false)
}

fn resolve_symlink(root: &Path, path: &Path) -> Result<PathBuf> {
    let target = fs::read_link(path)?;
    let resolved = if target.is_absolute() {
        target
    } else {
        path.parent().unwrap_or(root).join(target)
    };
    Ok(resolved.canonicalize().unwrap_or(resolved))
}

fn marker(
    manifest_exists: bool,
    manifest_managed_by_agentflow: bool,
    manifest_version: Option<String>,
    layout_version: Option<String>,
    agent_manual_exists: bool,
    skills_lock_exists: bool,
    managed_entry_exists: bool,
) -> WorkspaceOwnershipMarker {
    WorkspaceOwnershipMarker {
        manifest_exists,
        manifest_managed_by_agentflow,
        manifest_version,
        layout_version,
        agent_manual_exists,
        skills_lock_exists,
        managed_entry_exists,
    }
}

fn status(
    root: &Path,
    agentflow_path: &Path,
    state: WorkspaceOwnershipState,
    marker: WorkspaceOwnershipMarker,
    detected_files: Vec<String>,
    warnings: Vec<String>,
    errors: Vec<String>,
    recommended_action: WorkspaceOwnershipAction,
) -> WorkspaceOwnershipStatus {
    let ready_for_prepare = matches!(
        state,
        WorkspaceOwnershipState::None
            | WorkspaceOwnershipState::ManagedCurrent
            | WorkspaceOwnershipState::ManagedLegacy
            | WorkspaceOwnershipState::Corrupted
    );
    WorkspaceOwnershipStatus {
        version: WORKSPACE_OWNERSHIP_VERSION.to_string(),
        project_root: root.display().to_string(),
        status: state,
        ready_for_prepare,
        agent_blocked: !ready_for_prepare,
        agentflow_path: agentflow_path.display().to_string(),
        marker,
        detected_files,
        warnings,
        errors,
        recommended_action,
    }
}

fn unix_timestamp_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}
