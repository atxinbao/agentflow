use crate::{
    issue::{AgentRole, DisplayStatus, InputIssue, InputIssueStatus, IssueCategory},
    model::{
        InputIndex, InputIndexEntry, InputManifest, InputSnapshot, InputStatusSnapshot,
        InputSummary, InputWorkspaceStatus, INPUT_STATUS_VERSION,
    },
    project::{InputProject, InputProjectStatus},
    relations::{
        InputDependencyGraph, InputIssueRelation, InputIssueRelationKind, InputIssueRelationsFile,
    },
    spec_gate::{InputIntakeResult, InputSpecDescriptor, InputSpecStatus},
    storage::{
        canonical_project_root, ensure_directory, read_json, read_json_files,
        unix_timestamp_seconds, write_json, write_json_if_changed, write_json_if_missing,
        INPUT_DIRECTORIES, INPUT_REQUIRED_FILES,
    },
    validate::build_input_snapshot,
    views::InputView,
};
use agentflow_workflow_events::{
    append_event_once, prepare_events_workspace, IssueReadyPayload, WorkflowEventDraft,
    EVENT_TYPE_INPUT_ISSUE_READY,
};
use anyhow::{Context, Result};
use serde_json::Value;
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
    normalize_issue_metadata_files(&root)?;

    let summary = load_summary(&root)?;
    let manifest = InputManifest::new(root.display().to_string(), summary);
    write_json_if_changed(&root.join(".agentflow/input/manifest.json"), &manifest)?;
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
    repair_derived_input_files(&root)?;
    write_json_if_changed(
        &root.join(".agentflow/input/views/active.json"),
        &InputView::active(),
    )?;
    write_json_if_changed(
        &root.join(".agentflow/input/views/blocked.json"),
        &InputView::blocked(),
    )?;
    write_json_if_changed(
        &root.join(".agentflow/input/views/by-spec.json"),
        &InputView::by_spec(),
    )?;
    write_json_if_changed(
        &root.join(".agentflow/input/views/by-project.json"),
        &InputView::by_project(),
    )?;

    let snapshot = build_input_snapshot(&root)?;
    rebuild_index(&root, &snapshot)?;
    let snapshot = build_input_snapshot(&root)?;
    publish_ready_issue_events(&root, &snapshot)?;
    Ok(snapshot)
}

fn repair_derived_input_files(root: &Path) -> Result<()> {
    repair_input_index_file(root)?;
    repair_issue_relations_file(root)?;
    repair_dependency_graph_file(root)?;
    Ok(())
}

fn repair_input_index_file(root: &Path) -> Result<()> {
    let path = root.join(".agentflow/input/index.json");
    if read_json::<InputIndex>(&path).is_ok() {
        return Ok(());
    }

    write_json(
        &path,
        &InputIndex {
            updated_at: unix_timestamp_seconds(),
            ..InputIndex::default()
        },
    )
}

fn repair_issue_relations_file(root: &Path) -> Result<()> {
    let path = root.join(".agentflow/input/relations/issue-relations.json");
    if read_json::<InputIssueRelationsFile>(&path).is_ok() {
        return Ok(());
    }

    let repaired = read_json_value(&path)
        .ok()
        .and_then(|value| {
            value
                .get("relations")
                .and_then(Value::as_array)
                .map(|relations| {
                    relations
                        .iter()
                        .filter_map(normalize_relation_value)
                        .collect::<Vec<_>>()
                })
        })
        .map(|relations| InputIssueRelationsFile {
            relations,
            ..InputIssueRelationsFile::default()
        })
        .unwrap_or_default();

    write_json(&path, &repaired)
}

fn repair_dependency_graph_file(root: &Path) -> Result<()> {
    let path = root.join(".agentflow/input/relations/dependency-graph.json");
    if read_json::<InputDependencyGraph>(&path).is_ok() {
        return Ok(());
    }

    let repaired = read_json_value(&path)
        .ok()
        .map(|value| {
            let nodes = value
                .get("nodes")
                .and_then(Value::as_array)
                .map(|nodes| {
                    nodes
                        .iter()
                        .filter_map(Value::as_str)
                        .map(str::to_string)
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            let edges = value
                .get("edges")
                .and_then(Value::as_array)
                .map(|edges| {
                    edges
                        .iter()
                        .filter_map(normalize_relation_value)
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            InputDependencyGraph {
                nodes,
                edges,
                ..InputDependencyGraph::default()
            }
        })
        .unwrap_or_default();

    write_json(&path, &repaired)
}

fn normalize_relation_value(value: &Value) -> Option<InputIssueRelation> {
    let from_issue_id = value
        .get("fromIssueId")
        .or_else(|| value.get("from"))
        .and_then(Value::as_str)?
        .to_string();
    let to_issue_id = value
        .get("toIssueId")
        .or_else(|| value.get("to"))
        .and_then(Value::as_str)?
        .to_string();
    let relation_type =
        serde_json::from_value::<InputIssueRelationKind>(value.get("type")?.clone()).ok()?;

    Some(InputIssueRelation {
        from_issue_id,
        to_issue_id,
        relation_type,
    })
}

fn read_json_value(path: &Path) -> Result<Value> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

pub(crate) fn normalize_issue_metadata_files(root: &Path) -> Result<()> {
    let issue_dir = root.join(".agentflow/input/issues");
    if !issue_dir.exists() {
        return Ok(());
    }

    let mut entries = fs::read_dir(&issue_dir)
        .with_context(|| format!("read directory {}", issue_dir.display()))?
        .collect::<Result<Vec<_>, _>>()
        .with_context(|| format!("collect directory {}", issue_dir.display()))?;
    entries.sort_by_key(|entry| entry.path());

    for entry in entries {
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let mut raw_issue = read_json_value(&path)?;
        let legacy_status = migrate_issue_status_fields(&mut raw_issue);
        let legacy_priority = raw_issue
            .get("priority")
            .and_then(Value::as_str)
            .is_some_and(|value| matches!(value, "low" | "normal" | "high"));
        let legacy_risk_field = raw_issue.get("riskLevel").is_some();
        let mut issue: InputIssue = serde_json::from_value(raw_issue.clone())
            .with_context(|| format!("parse {}", path.display()))?;
        let before = serde_json::to_value(&issue)?;
        issue.normalize_execution_metadata();
        if legacy_status
            || legacy_priority
            || legacy_risk_field
            || serde_json::to_value(&issue)? != before
        {
            write_json(&path, &issue)?;
        }
    }

    Ok(())
}

fn migrate_issue_status_fields(issue: &mut Value) -> bool {
    let mut changed = false;
    changed |= migrate_issue_status_field(issue, "status");
    changed |= migrate_issue_status_field(issue, "displayStatus");
    changed
}

fn migrate_issue_status_field(issue: &mut Value, key: &str) -> bool {
    let Some(value) = issue.get_mut(key) else {
        return false;
    };
    let Some(status) = value.as_str() else {
        return false;
    };
    let Some(canonical) = canonical_issue_status(status) else {
        return false;
    };
    if canonical == status {
        return false;
    }
    *value = Value::String(canonical.to_string());
    true
}

fn canonical_issue_status(status: &str) -> Option<&'static str> {
    match status {
        "backlog" => Some("backlog"),
        "todo" => Some("todo"),
        "in_progress" => Some("in_progress"),
        "in_review" => Some("in_review"),
        "done" => Some("done"),
        "blocked" => Some("blocked"),
        "cancel" => Some("cancel"),
        _ => None,
    }
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

pub fn load_input_issue(project_root: impl AsRef<Path>, issue_id: &str) -> Result<InputIssue> {
    let root = canonical_project_root(project_root)?;
    let issue_path = root
        .join(".agentflow/input/issues")
        .join(format!("{issue_id}.json"));
    let mut issue: InputIssue = read_json(&issue_path)?;
    issue.normalize_execution_metadata();
    if issue.issue_id != issue_id {
        anyhow::bail!(
            "input issue id mismatch: requested {issue_id}, found {}",
            issue.issue_id
        );
    }
    Ok(issue)
}

pub fn load_input_project(
    project_root: impl AsRef<Path>,
    project_id: &str,
) -> Result<InputProject> {
    let root = canonical_project_root(project_root)?;
    let project_path = root
        .join(".agentflow/input/projects")
        .join(format!("{project_id}.json"));
    let project: InputProject = read_json(&project_path)?;
    if project.project_id != project_id {
        anyhow::bail!(
            "input project id mismatch: requested {project_id}, found {}",
            project.project_id
        );
    }
    Ok(project)
}

pub fn update_input_issue_status(
    project_root: impl AsRef<Path>,
    issue_id: &str,
    status: InputIssueStatus,
) -> Result<InputIssue> {
    let root = canonical_project_root(project_root)?;
    let issue_path = root
        .join(".agentflow/input/issues")
        .join(format!("{issue_id}.json"));
    let mut issue: InputIssue = read_json(&issue_path)?;
    if issue.issue_id != issue_id {
        anyhow::bail!(
            "input issue id mismatch: requested {issue_id}, found {}",
            issue.issue_id
        );
    }
    let next_display_status = DisplayStatus::from_input_status(&status);
    if issue.status == status && issue.display_status == next_display_status {
        return Ok(issue);
    }
    issue.status = status;
    issue.display_status = next_display_status;
    issue.system.updated_at = unix_timestamp_seconds();
    issue.system.revision = issue.system.revision.saturating_add(1);
    write_json_if_changed(&issue_path, &issue)?;
    Ok(issue)
}

pub fn update_input_issue_latest_run(
    project_root: impl AsRef<Path>,
    issue_id: &str,
    latest_run_id: Option<String>,
) -> Result<InputIssue> {
    let root = canonical_project_root(project_root)?;
    let issue_path = root
        .join(".agentflow/input/issues")
        .join(format!("{issue_id}.json"));
    let mut issue: InputIssue = read_json(&issue_path)?;
    if issue.issue_id != issue_id {
        anyhow::bail!(
            "input issue id mismatch: requested {issue_id}, found {}",
            issue.issue_id
        );
    }
    if issue.latest_run_id == latest_run_id {
        return Ok(issue);
    }
    issue.latest_run_id = latest_run_id;
    issue.system.updated_at = unix_timestamp_seconds();
    issue.system.revision = issue.system.revision.saturating_add(1);
    write_json_if_changed(&issue_path, &issue)?;
    Ok(issue)
}

pub fn update_input_issue_branch_name(
    project_root: impl AsRef<Path>,
    issue_id: &str,
    branch_name: Option<String>,
) -> Result<InputIssue> {
    let root = canonical_project_root(project_root)?;
    let issue_path = root
        .join(".agentflow/input/issues")
        .join(format!("{issue_id}.json"));
    let mut issue: InputIssue = read_json(&issue_path)?;
    if issue.issue_id != issue_id {
        anyhow::bail!(
            "input issue id mismatch: requested {issue_id}, found {}",
            issue.issue_id
        );
    }
    if issue.branch_name == branch_name {
        return Ok(issue);
    }
    issue.branch_name = branch_name;
    issue.system.updated_at = unix_timestamp_seconds();
    issue.system.revision = issue.system.revision.saturating_add(1);
    write_json_if_changed(&issue_path, &issue)?;
    Ok(issue)
}

pub fn update_input_project_status(
    project_root: impl AsRef<Path>,
    project_id: &str,
    status: InputProjectStatus,
) -> Result<InputProject> {
    let root = canonical_project_root(project_root)?;
    let project_path = root
        .join(".agentflow/input/projects")
        .join(format!("{project_id}.json"));
    let mut project: InputProject = read_json(&project_path)?;
    if project.project_id != project_id {
        anyhow::bail!(
            "input project id mismatch: requested {project_id}, found {}",
            project.project_id
        );
    }
    if project.status == status {
        return Ok(project);
    }
    project.status = status;
    project.system.updated_at = unix_timestamp_seconds();
    project.system.revision = project.system.revision.saturating_add(1);
    write_json_if_changed(&project_path, &project)?;
    Ok(project)
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
            .filter(|issue| issue.execution_risk.requires_human_confirmation())
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
    let mut index = InputIndex {
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
                status: issue.status.as_str().to_string(),
                display_status: Some(issue.display_status.clone()),
            })
            .collect(),
    };
    let path = root.join(".agentflow/input/index.json");
    if let Ok(existing) = read_json::<InputIndex>(&path) {
        let mut existing_without_timestamp = existing.clone();
        existing_without_timestamp.updated_at = 0;
        let mut next_without_timestamp = index.clone();
        next_without_timestamp.updated_at = 0;
        if serde_json::to_value(&existing_without_timestamp)?
            == serde_json::to_value(&next_without_timestamp)?
        {
            return Ok(());
        }
    }
    index.updated_at = unix_timestamp_seconds();
    write_json_if_changed(&path, &index)?;
    Ok(())
}

fn publish_ready_issue_events(root: &Path, snapshot: &InputSnapshot) -> Result<()> {
    prepare_events_workspace(root)?;
    for issue in snapshot
        .issues
        .iter()
        .filter(|issue| issue_ready_for_event(issue))
    {
        let payload = IssueReadyPayload {
            issue_id: issue.issue_id.clone(),
            issue_path: issue.issue_path.clone(),
            issue_category: issue.issue_category.as_str().to_string(),
            required_agent_role: issue.required_agent_role.as_str().to_string(),
            display_status: issue.display_status.as_str().to_string(),
            title: issue.title.clone(),
            objective: if issue.summary.trim().is_empty() {
                issue.scope.join("\n")
            } else {
                issue.summary.clone()
            },
            acceptance_criteria: issue.acceptance_criteria.clone(),
            context_pack_path: Some(issue.context_pack_path.clone()),
        };
        append_event_once(
            root,
            WorkflowEventDraft {
                event_type: EVENT_TYPE_INPUT_ISSUE_READY.to_string(),
                source: "input".to_string(),
                subject_id: issue.issue_id.clone(),
                subject_path: Some(issue.issue_path.clone()),
                dedupe_key: format!(
                    "input.issue.ready:{}:{}",
                    issue.issue_id, issue.system.revision
                ),
                payload: serde_json::to_value(payload)?,
            },
        )?;
    }
    Ok(())
}

fn issue_ready_for_event(issue: &InputIssue) -> bool {
    matches!(issue.issue_category, IssueCategory::Spec)
        && matches!(issue.required_agent_role, AgentRole::BuildAgent)
        && matches!(
            issue.status,
            InputIssueStatus::Backlog | InputIssueStatus::Todo
        )
        && matches!(
            issue.display_status,
            DisplayStatus::Backlog | DisplayStatus::Todo
        )
        && !issue.context_pack_path.trim().is_empty()
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
