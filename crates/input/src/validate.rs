use crate::{
    issue::{DisplayStatus, InputIssue, InputIssueModel},
    manager::{load_input_facts, load_summary, missing_input_paths, status},
    model::{InputManifest, InputSnapshot, InputWorkspaceStatus, INPUT_SNAPSHOT_VERSION},
    project::InputProject,
    relations::InputIssueRelationsFile,
    storage::canonical_project_root,
};
use anyhow::Result;
use std::{collections::BTreeSet, path::Path};

pub fn validate_input_snapshot(project_root: impl AsRef<Path>) -> Result<InputSnapshot> {
    let root = canonical_project_root(project_root)?;
    build_input_snapshot(&root)
}

pub(crate) fn build_input_snapshot(root: &Path) -> Result<InputSnapshot> {
    let missing_paths = missing_input_paths(root);
    let manifest_exists = root.join(".agentflow/input/manifest.json").exists();
    let index_exists = root.join(".agentflow/input/index.json").exists();
    let summary = load_summary(root).unwrap_or_default();
    let mut warnings = Vec::new();
    let mut errors = Vec::new();

    if !missing_paths.is_empty() {
        errors.extend(
            missing_paths
                .iter()
                .map(|path| format!("Input workspace path is missing: {path}")),
        );
    }

    let ownership = agentflow_agent_manual::check_agentflow_workspace_ownership(root)?;
    if ownership.agent_blocked {
        errors.push(format!(
            "Input workspace ownership blocked: {:?}: {}",
            ownership.status,
            ownership.errors.join("; ")
        ));
    } else if !ownership.warnings.is_empty() {
        warnings.extend(ownership.warnings);
    }

    let facts = load_input_facts(root);
    let (manifest, index, intake, specs, projects, mut issues, relations) = match facts {
        Ok(value) => value,
        Err(error) => {
            errors.push(error.to_string());
            let manifest = InputManifest {
                version: crate::model::INPUT_MANIFEST_VERSION.to_string(),
                project_root: root.display().to_string(),
                status: InputWorkspaceStatus::Missing,
                paths: crate::model::input_paths(),
                legacy_paths: crate::model::legacy_paths(),
                summary: summary.clone(),
            };
            (
                manifest,
                crate::model::InputIndex::default(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                InputIssueRelationsFile::default(),
            )
        }
    };

    for issue in &mut issues {
        issue.normalize_execution_metadata();
    }

    validate_manifest(&manifest, &mut errors);
    validate_spec_gate(root, &specs, &mut errors);
    validate_issue_graph(&projects, &issues, &relations, &mut errors);
    normalize_display_statuses(&mut issues);

    let ready = errors.is_empty() && missing_paths.is_empty();
    let status = status(
        root,
        ready,
        manifest_exists,
        index_exists,
        summary,
        missing_paths,
        warnings,
        errors,
    );

    Ok(InputSnapshot {
        version: INPUT_SNAPSHOT_VERSION.to_string(),
        project_root: root.display().to_string(),
        ready,
        status,
        manifest,
        index,
        intake,
        specs,
        projects,
        issues,
        relations,
    })
}

fn normalize_display_statuses(issues: &mut [InputIssue]) {
    for issue in issues {
        issue.display_status = DisplayStatus::from_input_status(&issue.status);
    }
}

fn validate_manifest(manifest: &InputManifest, errors: &mut Vec<String>) {
    if manifest.version != crate::model::INPUT_MANIFEST_VERSION {
        errors.push(format!(
            "input manifest version mismatch: {}",
            manifest.version
        ));
    }
    if !manifest.paths.contains_key("root") || !manifest.paths.contains_key("issues") {
        errors.push("input manifest canonical paths are incomplete".to_string());
    }
    if manifest.legacy_paths.get("legacySpec").map(String::as_str) != Some(".agentflow/spec") {
        errors.push("input manifest legacySpec marker is missing".to_string());
    }
    if manifest
        .legacy_paths
        .get("legacyGoalTree")
        .map(String::as_str)
        != Some(".agentflow/goal-tree")
    {
        errors.push("input manifest legacyGoalTree marker is missing".to_string());
    }
}

fn validate_spec_gate(
    root: &Path,
    specs: &[crate::spec_gate::InputSpecDescriptor],
    errors: &mut Vec<String>,
) {
    for spec in specs {
        let product = root.join(&spec.product_path);
        let tech = root.join(&spec.tech_path);
        let spec_json = root.join(&spec.spec_path);
        if !product.is_file() {
            errors.push(format!("Spec {} is missing product.md", spec.spec_id));
        }
        if !tech.is_file() {
            errors.push(format!("Spec {} is missing tech.md", spec.spec_id));
        }
        if !spec_json.is_file() {
            errors.push(format!("Spec {} is missing spec.json", spec.spec_id));
        }
        if matches!(spec.status, crate::spec_gate::InputSpecStatus::Approved) {
            if let Some(approval) = &spec.approval_path {
                if !root.join(approval).is_file() {
                    errors.push(format!(
                        "Approved spec {} is missing approval.json",
                        spec.spec_id
                    ));
                }
            } else {
                errors.push(format!(
                    "Approved spec {} is missing approval path",
                    spec.spec_id
                ));
            }
        }
    }
}

fn validate_issue_graph(
    projects: &[InputProject],
    issues: &[InputIssue],
    relations: &InputIssueRelationsFile,
    errors: &mut Vec<String>,
) {
    let project_ids = projects
        .iter()
        .map(|project| project.project_id.as_str())
        .collect::<BTreeSet<_>>();
    let issue_ids = issues
        .iter()
        .map(|issue| issue.issue_id.as_str())
        .collect::<BTreeSet<_>>();

    for issue in issues {
        errors.extend(issue.target_metadata_errors());
        match issue.issue_model {
            InputIssueModel::Direct => {
                if issue.project_id.is_some() {
                    errors.push(format!(
                        "direct issue {} must use projectId = null",
                        issue.issue_id
                    ));
                }
            }
            InputIssueModel::Project => match issue.project_id.as_deref() {
                Some(project_id) if project_ids.contains(project_id) => {}
                Some(project_id) => errors.push(format!(
                    "issue {} references missing project {}",
                    issue.issue_id, project_id
                )),
                None => errors.push(format!(
                    "project issue {} must include projectId",
                    issue.issue_id
                )),
            },
        }
    }

    for project in projects {
        for issue_id in &project.issue_ids {
            if !issue_ids.contains(issue_id.as_str()) {
                errors.push(format!(
                    "project {} references missing issue {}",
                    project.project_id, issue_id
                ));
            }
        }
    }

    for relation in &relations.relations {
        if !issue_ids.contains(relation.from_issue_id.as_str()) {
            errors.push(format!(
                "issue relation references missing fromIssueId {}",
                relation.from_issue_id
            ));
        }
        if !issue_ids.contains(relation.to_issue_id.as_str()) {
            errors.push(format!(
                "issue relation references missing toIssueId {}",
                relation.to_issue_id
            ));
        }
    }
}
