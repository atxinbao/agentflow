use crate::{
    issue::{DisplayStatus, InputIssue},
    model::{
        InputIndex, InputIndexEntry, InputManifest, InputSnapshot, InputStatusSnapshot,
        InputSummary, InputWorkspaceStatus, INPUT_STATUS_VERSION,
    },
    project::InputProject,
    relations::{InputDependencyGraph, InputIssueRelationsFile},
    spec_gate::{InputIntakeResult, InputSpecDescriptor, InputSpecStatus},
    storage::{
        canonical_project_root, ensure_directory, read_json, read_json_files,
        unix_timestamp_seconds, write_json, write_json_if_missing, INPUT_DIRECTORIES,
        INPUT_REQUIRED_FILES,
    },
    validate::build_input_snapshot,
    views::InputView,
};
use anyhow::{Context, Result};
use std::{fs, path::Path};

pub fn prepare_input_workspace(project_root: impl AsRef<Path>) -> Result<InputSnapshot> {
    let root = canonical_project_root(project_root)?;
    let ownership = agentflow_agent_manual::assert_agentflow_workspace_owned_or_creatable(&root)?;
    if matches!(
        ownership.status,
        agentflow_agent_manual::model::WorkspaceOwnershipState::None
    ) {
        agentflow_agent_manual::prepare_agent_working_manual(&root)?;
    }

    for relative_path in INPUT_DIRECTORIES {
        ensure_directory(&root.join(relative_path))?;
    }

    let summary = load_summary(&root)?;
    let manifest = InputManifest::new(root.display().to_string(), summary);
    write_json(&root.join(".agentflow/input/manifest.json"), &manifest)?;
    write_json_if_missing(
        &root.join(".agentflow/input/index.json"),
        &InputIndex {
            updated_at: unix_timestamp_seconds(),
            ..InputIndex::default()
        },
    )?;
    write_json_if_missing(
        &root.join(".agentflow/input/relations/issue-relations.json"),
        &InputIssueRelationsFile::default(),
    )?;
    write_json_if_missing(
        &root.join(".agentflow/input/relations/dependency-graph.json"),
        &InputDependencyGraph::default(),
    )?;
    write_json_if_missing(
        &root.join(".agentflow/input/views/active.json"),
        &InputView::active(),
    )?;
    write_json_if_missing(
        &root.join(".agentflow/input/views/blocked.json"),
        &InputView::blocked(),
    )?;
    write_json_if_missing(
        &root.join(".agentflow/input/views/by-spec.json"),
        &InputView::by_spec(),
    )?;
    write_json_if_missing(
        &root.join(".agentflow/input/views/by-project.json"),
        &InputView::by_project(),
    )?;

    let snapshot = build_input_snapshot(&root)?;
    rebuild_index(&root, &snapshot)?;
    build_input_snapshot(&root)
}

pub fn validate_input_workspace(project_root: impl AsRef<Path>) -> Result<InputSnapshot> {
    let root = canonical_project_root(project_root)?;
    build_input_snapshot(&root)
}

pub fn load_input_status(project_root: impl AsRef<Path>) -> Result<InputStatusSnapshot> {
    Ok(validate_input_workspace(project_root)?.status)
}

pub fn load_input_snapshot(project_root: impl AsRef<Path>) -> Result<InputSnapshot> {
    validate_input_workspace(project_root)
}

pub fn load_input_manifest(project_root: impl AsRef<Path>) -> Result<InputManifest> {
    let root = canonical_project_root(project_root)?;
    read_json(&root.join(".agentflow/input/manifest.json"))
}

pub fn load_input_index(project_root: impl AsRef<Path>) -> Result<InputIndex> {
    let root = canonical_project_root(project_root)?;
    read_json(&root.join(".agentflow/input/index.json"))
}

pub(crate) fn load_summary(root: &Path) -> Result<InputSummary> {
    let intake = read_json_files::<InputIntakeResult>(&root.join(".agentflow/input/intake"))?;
    let projects = read_json_files::<InputProject>(&root.join(".agentflow/input/projects"))?;
    let issues = read_json_files::<InputIssue>(&root.join(".agentflow/input/issues"))?;
    let draft_specs = count_spec_directories(&root.join(".agentflow/input/specs/drafts"))?;
    let approved_specs = count_spec_directories(&root.join(".agentflow/input/specs/approved"))?;
    Ok(InputSummary {
        intake: intake.len(),
        draft_specs,
        approved_specs,
        projects: projects.len(),
        issues: issues.len(),
        blocked_issues: issues
            .iter()
            .filter(|issue| matches!(issue.status, crate::issue::InputIssueStatus::Blocked))
            .count(),
        high_risk_issues: issues
            .iter()
            .filter(|issue| issue.risk_level.requires_human_confirmation())
            .count(),
    })
}

pub(crate) fn missing_input_paths(root: &Path) -> Vec<String> {
    INPUT_DIRECTORIES
        .iter()
        .copied()
        .chain(INPUT_REQUIRED_FILES.iter().copied())
        .filter(|relative_path| !root.join(relative_path).exists())
        .map(str::to_string)
        .collect()
}

pub(crate) fn load_input_facts(
    root: &Path,
) -> Result<(
    InputManifest,
    InputIndex,
    Vec<InputIntakeResult>,
    Vec<InputSpecDescriptor>,
    Vec<InputProject>,
    Vec<InputIssue>,
    InputIssueRelationsFile,
)> {
    let manifest = read_json(&root.join(".agentflow/input/manifest.json")).with_context(|| {
        format!(
            "load {}",
            root.join(".agentflow/input/manifest.json").display()
        )
    })?;
    let index = read_json(&root.join(".agentflow/input/index.json"))?;
    let intake = read_json_files(&root.join(".agentflow/input/intake"))?;
    let specs = load_specs(root)?;
    let projects = read_json_files(&root.join(".agentflow/input/projects"))?;
    let issues = read_json_files(&root.join(".agentflow/input/issues"))?;
    let relations = read_json(&root.join(".agentflow/input/relations/issue-relations.json"))?;
    Ok((manifest, index, intake, specs, projects, issues, relations))
}

pub(crate) fn status(
    root: &Path,
    ready: bool,
    manifest_exists: bool,
    index_exists: bool,
    summary: InputSummary,
    missing_paths: Vec<String>,
    warnings: Vec<String>,
    errors: Vec<String>,
) -> InputStatusSnapshot {
    let status = if errors.iter().any(|error| error.contains("ownership")) {
        InputWorkspaceStatus::Blocked
    } else if !errors.is_empty() {
        InputWorkspaceStatus::Failed
    } else if !missing_paths.is_empty() {
        InputWorkspaceStatus::Missing
    } else if !warnings.is_empty() {
        InputWorkspaceStatus::Degraded
    } else {
        InputWorkspaceStatus::Ready
    };
    InputStatusSnapshot {
        version: INPUT_STATUS_VERSION.to_string(),
        project_root: root.display().to_string(),
        status,
        ready,
        manifest_exists,
        index_exists,
        summary,
        missing_paths,
        warnings,
        errors,
    }
}

fn rebuild_index(root: &Path, snapshot: &InputSnapshot) -> Result<()> {
    let index = InputIndex {
        version: crate::model::INPUT_INDEX_VERSION.to_string(),
        updated_at: unix_timestamp_seconds(),
        specs: snapshot
            .specs
            .iter()
            .map(|spec| InputIndexEntry {
                id: spec.spec_id.clone(),
                title: spec.spec_id.clone(),
                path: spec.spec_path.clone(),
                status: format!("{:?}", spec.status).to_lowercase(),
                display_status: None,
            })
            .collect(),
        projects: snapshot
            .projects
            .iter()
            .map(|project| InputIndexEntry {
                id: project.project_id.clone(),
                title: project.title.clone(),
                path: project.system.path.clone(),
                status: format!("{:?}", project.status).to_lowercase(),
                display_status: None,
            })
            .collect(),
        issues: snapshot
            .issues
            .iter()
            .map(|issue| InputIndexEntry {
                id: issue.issue_id.clone(),
                title: issue.title.clone(),
                path: issue.system.path.clone(),
                status: format!("{:?}", issue.status).to_lowercase(),
                display_status: Some(DisplayStatus::from_input_status(&issue.status)),
            })
            .collect(),
    };
    write_json(&root.join(".agentflow/input/index.json"), &index)
}

fn load_specs(root: &Path) -> Result<Vec<InputSpecDescriptor>> {
    let mut specs = Vec::new();
    specs.extend(load_spec_descriptors(
        root,
        ".agentflow/input/specs/drafts",
        InputSpecStatus::Draft,
    )?);
    specs.extend(load_spec_descriptors(
        root,
        ".agentflow/input/specs/approved",
        InputSpecStatus::Approved,
    )?);
    Ok(specs)
}

fn load_spec_descriptors(
    root: &Path,
    relative_root: &str,
    status: InputSpecStatus,
) -> Result<Vec<InputSpecDescriptor>> {
    let directory = root.join(relative_root);
    if !directory.exists() {
        return Ok(Vec::new());
    }

    let mut entries = fs::read_dir(&directory)
        .with_context(|| format!("read {}", directory.display()))?
        .collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.path());

    Ok(entries
        .into_iter()
        .filter(|entry| entry.path().is_dir())
        .map(|entry| {
            let spec_id = entry.file_name().to_string_lossy().to_string();
            let base = format!("{relative_root}/{spec_id}");
            InputSpecDescriptor {
                version: "input-spec.v1".to_string(),
                spec_id,
                status: status.clone(),
                product_path: format!("{base}/product.md"),
                tech_path: format!("{base}/tech.md"),
                approval_path: matches!(status, InputSpecStatus::Approved)
                    .then(|| format!("{base}/approval.json")),
                spec_path: format!("{base}/spec.json"),
            }
        })
        .collect())
}

fn count_spec_directories(path: &Path) -> Result<usize> {
    if !path.exists() {
        return Ok(0);
    }
    Ok(fs::read_dir(path)
        .with_context(|| format!("read {}", path.display()))?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_dir())
        .count())
}
