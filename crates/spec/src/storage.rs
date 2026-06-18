use crate::model::{
    ProjectBrainDocumentSet, ProjectBrainDocumentStatus, ProjectBrainSnapshot, ProjectBrainStatus,
    RequirementDocument, SpecExpectedOutputs, SpecIssue, SpecIssueCategory, SpecIssueDraft,
    SpecIssueStatus, SpecProject, SpecProjectDraft, SpecProjectStatus, SpecRequiredAgentRole,
    SpecSystemRecord, PROJECT_BRAIN_DOCUMENT_SET_VERSION, PROJECT_BRAIN_SNAPSHOT_VERSION,
    SPEC_INDEX_VERSION, SPEC_ISSUE_VERSION, SPEC_MANIFEST_VERSION, SPEC_PROJECT_VERSION,
};
use anyhow::{Context, Result};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

pub const SPEC_DIRECTORIES: &[&str] = &[
    ".agentflow/spec",
    ".agentflow/spec/projects",
    ".agentflow/spec/issues",
    ".agentflow/spec/archive",
    ".agentflow/spec/requirements",
];

pub const SPEC_REQUIRED_FILES: &[&str] = &[
    ".agentflow/spec/manifest.json",
    ".agentflow/spec/index.json",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpecWorkspaceSummary {
    pub project_root: String,
    pub manifest_path: String,
    pub index_path: String,
    pub created_directories: Vec<String>,
    pub created_files: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SpecManifest {
    version: String,
    project_root: String,
    generated_by: String,
    updated_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SpecIndex {
    version: String,
    updated_at: u64,
    projects: Vec<SpecProjectIndexEntry>,
    issues: Vec<SpecIssueIndexEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SpecProjectIndexEntry {
    project_id: String,
    path: String,
    title: String,
    status: SpecProjectStatus,
    issue_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SpecIssueIndexEntry {
    issue_id: String,
    project_id: Option<String>,
    path: String,
    title: String,
    status: SpecIssueStatus,
    workflow_ref: String,
}

pub fn prepare_spec_workspace(project_root: impl AsRef<Path>) -> Result<SpecWorkspaceSummary> {
    let root = canonical_project_root(project_root)?;
    let mut created_directories = Vec::new();
    let mut created_files = Vec::new();

    for relative in SPEC_DIRECTORIES {
        let path = root.join(relative);
        if !path.exists() {
            ensure_directory(&path)?;
            created_directories.push(relative.to_string());
        }
    }

    let manifest_path = root.join(".agentflow/spec/manifest.json");
    if !manifest_path.exists() {
        write_json(
            &manifest_path,
            &SpecManifest {
                version: SPEC_MANIFEST_VERSION.to_string(),
                project_root: root.display().to_string(),
                generated_by: "agentflow-spec".to_string(),
                updated_at: unix_timestamp_seconds(),
            },
        )?;
        created_files.push(".agentflow/spec/manifest.json".to_string());
    }

    let index_path = root.join(".agentflow/spec/index.json");
    if !index_path.exists() {
        write_json(&index_path, &empty_index())?;
        created_files.push(".agentflow/spec/index.json".to_string());
    }

    Ok(SpecWorkspaceSummary {
        project_root: root.display().to_string(),
        manifest_path: ".agentflow/spec/manifest.json".to_string(),
        index_path: ".agentflow/spec/index.json".to_string(),
        created_directories,
        created_files,
    })
}

pub fn issue_from_requirement(
    project_root: impl AsRef<Path>,
    requirement_path: impl AsRef<Path>,
    draft: SpecIssueDraft,
) -> Result<SpecIssue> {
    let root = canonical_project_root(project_root)?;
    let requirement = read_requirement_document(&root, requirement_path)?;
    validate_issue_id(&draft.issue_id)?;
    let issue_path = format!(".agentflow/spec/issues/{}.json", draft.issue_id);
    let now = unix_timestamp_seconds();
    let title = draft.title.unwrap_or_else(|| requirement.title.clone());
    let summary = draft.summary.unwrap_or_else(|| requirement.summary.clone());
    let source_spec_id = draft
        .source_spec_id
        .unwrap_or_else(|| requirement.requirement_id.clone());

    Ok(SpecIssue {
        version: SPEC_ISSUE_VERSION.to_string(),
        issue_id: draft.issue_id.clone(),
        issue_category: SpecIssueCategory::Spec,
        required_agent_role: SpecRequiredAgentRole::BuildAgent,
        status: SpecIssueStatus::Backlog,
        workflow_ref: draft.workflow_ref,
        source_requirement_id: requirement.requirement_id,
        source_requirement_path: requirement.path.clone(),
        source_spec_id,
        project_id: draft.project_id,
        title,
        summary,
        priority: draft.priority,
        blocked_by: draft.blocked_by,
        allowed_paths: draft.allowed_paths,
        forbidden_paths: draft.forbidden_paths,
        validation_commands: draft.validation_commands,
        expected_outputs: SpecExpectedOutputs::for_issue(&draft.issue_id),
        system: SpecSystemRecord {
            created_by: "spec-agent".to_string(),
            created_at: now,
            updated_at: now,
            path: issue_path,
            public_requirement_path: requirement.path,
        },
    })
}

pub fn write_spec_issue(project_root: impl AsRef<Path>, issue: &SpecIssue) -> Result<PathBuf> {
    let root = canonical_project_root(project_root)?;
    prepare_spec_workspace(&root)?;
    validate_issue_contract(issue)?;
    let path = root.join(format!(".agentflow/spec/issues/{}.json", issue.issue_id));
    write_json(&path, issue)?;
    rebuild_spec_index(&root)?;
    Ok(path)
}

pub fn write_spec_project(
    project_root: impl AsRef<Path>,
    project: &SpecProject,
) -> Result<PathBuf> {
    let root = canonical_project_root(project_root)?;
    prepare_spec_workspace(&root)?;
    validate_project_contract(project)?;
    let path = root.join(format!(
        ".agentflow/spec/projects/{}.json",
        project.project_id
    ));
    write_json(&path, project)?;
    rebuild_spec_index(&root)?;
    Ok(path)
}

pub fn read_spec_issue(project_root: impl AsRef<Path>, issue_id: &str) -> Result<SpecIssue> {
    let root = canonical_project_root(project_root)?;
    read_json(&root.join(format!(".agentflow/spec/issues/{issue_id}.json")))
}

pub fn read_spec_project(project_root: impl AsRef<Path>, project_id: &str) -> Result<SpecProject> {
    let root = canonical_project_root(project_root)?;
    read_json(&root.join(format!(".agentflow/spec/projects/{project_id}.json")))
}

pub fn read_project_brain_document_set(
    project_root: impl AsRef<Path>,
    project_id: &str,
) -> Result<ProjectBrainDocumentSet> {
    let root = canonical_project_root(project_root)?;
    let project_path = project_brain_root(project_id);
    let goal_path = format!("{project_path}/GOAL.md");
    let plan_path = format!("{project_path}/PLAN.md");
    let decisions_path = format!("{project_path}/DECISIONS.md");
    let health_path = format!("{project_path}/PROJECT_HEALTH.md");

    let goal = read_document_probe(&root, &goal_path)?;
    let plan = read_document_probe(&root, &plan_path)?;
    let decisions = read_document_probe(&root, &decisions_path)?;
    let health = read_document_probe(&root, &health_path)?;
    let mut missing_documents = Vec::new();
    if !goal.exists {
        missing_documents.push("GOAL.md".to_string());
    }
    if !plan.exists {
        missing_documents.push("PLAN.md".to_string());
    }
    if !decisions.exists {
        missing_documents.push("DECISIONS.md".to_string());
    }

    Ok(ProjectBrainDocumentSet {
        version: PROJECT_BRAIN_DOCUMENT_SET_VERSION.to_string(),
        project_id: project_id.to_string(),
        root_path: project_path,
        goal_path,
        plan_path,
        decisions_path,
        health_path,
        goal_exists: goal.exists,
        plan_exists: plan.exists,
        decisions_exists: decisions.exists,
        health_exists: health.exists,
        goal_updated_at: goal.updated_at,
        plan_updated_at: plan.updated_at,
        decisions_updated_at: decisions.updated_at,
        health_updated_at: health.updated_at,
        missing_documents,
        readonly: true,
    })
}

pub fn read_project_brain_snapshot(
    project_root: impl AsRef<Path>,
    project_id: &str,
    project_title: &str,
) -> Result<ProjectBrainSnapshot> {
    let root = canonical_project_root(project_root)?;
    let document_set = read_project_brain_document_set(&root, project_id)?;
    let goal_probe = read_document_probe(&root, &document_set.goal_path)?;
    let plan_probe = read_document_probe(&root, &document_set.plan_path)?;
    let decisions_probe = read_document_probe(&root, &document_set.decisions_path)?;
    let health_probe = read_document_probe(&root, &document_set.health_path)?;

    let goal_status = classify_project_brain_document(&goal_probe);
    let plan_status = classify_project_brain_document(&plan_probe);
    let decision_status = classify_project_brain_document(&decisions_probe);
    let health_status = classify_optional_project_health_document(&health_probe);
    let missing_documents = document_set.missing_documents.clone();

    let brain_status = if health_status == ProjectBrainDocumentStatus::Blocked {
        ProjectBrainStatus::Blocked
    } else if missing_documents.len() == 3 {
        ProjectBrainStatus::NotInitialized
    } else if !goal_probe.exists {
        ProjectBrainStatus::NeedsGoal
    } else if !plan_probe.exists {
        ProjectBrainStatus::NeedsPlan
    } else if !decisions_probe.exists
        || matches!(
            goal_status,
            ProjectBrainDocumentStatus::Draft | ProjectBrainDocumentStatus::NeedsConfirmation
        )
        || matches!(
            plan_status,
            ProjectBrainDocumentStatus::Draft | ProjectBrainDocumentStatus::NeedsConfirmation
        )
        || matches!(
            decision_status,
            ProjectBrainDocumentStatus::Draft | ProjectBrainDocumentStatus::NeedsConfirmation
        )
    {
        ProjectBrainStatus::NeedsConfirmation
    } else if matches!(
        goal_status,
        ProjectBrainDocumentStatus::Blocked | ProjectBrainDocumentStatus::Stale
    ) || matches!(
        plan_status,
        ProjectBrainDocumentStatus::Blocked | ProjectBrainDocumentStatus::Stale
    ) || matches!(
        decision_status,
        ProjectBrainDocumentStatus::Blocked | ProjectBrainDocumentStatus::Stale
    ) {
        ProjectBrainStatus::Blocked
    } else if matches!(
        health_status,
        ProjectBrainDocumentStatus::Draft
            | ProjectBrainDocumentStatus::NeedsConfirmation
            | ProjectBrainDocumentStatus::Stale
    ) {
        ProjectBrainStatus::NeedsRecheck
    } else {
        ProjectBrainStatus::ReadyForProjectLoop
    };

    let mut open_questions = Vec::new();
    if !goal_probe.exists {
        open_questions.push("项目目标文档缺失，需要先确认 GOAL.md。".to_string());
    }
    if !plan_probe.exists {
        open_questions.push("项目计划文档缺失，需要先确认 PLAN.md。".to_string());
    }
    if !decisions_probe.exists {
        open_questions.push("项目决策记录缺失，需要补齐 DECISIONS.md。".to_string());
    }
    if matches!(goal_status, ProjectBrainDocumentStatus::NeedsConfirmation) {
        open_questions.push("GOAL.md 仍处于待确认状态。".to_string());
    }
    if matches!(plan_status, ProjectBrainDocumentStatus::NeedsConfirmation) {
        open_questions.push("PLAN.md 仍处于待确认状态。".to_string());
    }
    if matches!(
        health_status,
        ProjectBrainDocumentStatus::Draft | ProjectBrainDocumentStatus::NeedsConfirmation
    ) {
        open_questions.push("PROJECT_HEALTH.md 提示需要重新检查项目状态。".to_string());
    }

    let next_recommended_action = project_brain_next_action(&brain_status).to_string();
    let next_recommended_action_label = project_brain_next_action_label(&brain_status).to_string();
    let next_recommended_action_reason =
        project_brain_next_action_reason(&brain_status, &missing_documents, &health_status)
            .to_string();

    Ok(ProjectBrainSnapshot {
        version: PROJECT_BRAIN_SNAPSHOT_VERSION.to_string(),
        project_id: project_id.to_string(),
        project_title: project_title.to_string(),
        project_path: document_set.root_path,
        goal_document: document_set.goal_path,
        plan_document: document_set.plan_path,
        decisions_document: document_set.decisions_path,
        health_document: document_set.health_path,
        goal_status,
        plan_status,
        decision_status,
        health_status,
        brain_status,
        missing_documents,
        open_questions,
        next_recommended_action,
        next_recommended_action_label,
        next_recommended_action_reason,
        readonly: true,
    })
}

fn project_brain_next_action(status: &ProjectBrainStatus) -> &'static str {
    match status {
        ProjectBrainStatus::NotInitialized | ProjectBrainStatus::NeedsGoal => {
            "create-goal-draft-preview"
        }
        ProjectBrainStatus::NeedsPlan => "create-plan-draft-preview",
        ProjectBrainStatus::NeedsConfirmation => "confirm-project-brain",
        ProjectBrainStatus::ReadyForProjectLoop => "start-project-loop",
        ProjectBrainStatus::NeedsRecheck => "run-goal-recheck",
        ProjectBrainStatus::Blocked => "resolve-project-brain-blocker",
    }
}

fn project_brain_next_action_label(status: &ProjectBrainStatus) -> &'static str {
    match status {
        ProjectBrainStatus::NotInitialized | ProjectBrainStatus::NeedsGoal => "生成 Goal 草稿预览",
        ProjectBrainStatus::NeedsPlan => "生成 Plan 草稿预览",
        ProjectBrainStatus::NeedsConfirmation => "确认 Project Brain",
        ProjectBrainStatus::ReadyForProjectLoop => "进入项目循环",
        ProjectBrainStatus::NeedsRecheck => "重新检查项目目标",
        ProjectBrainStatus::Blocked => "处理 Project Brain 阻断",
    }
}

fn project_brain_next_action_reason(
    status: &ProjectBrainStatus,
    missing_documents: &[String],
    health_status: &ProjectBrainDocumentStatus,
) -> &'static str {
    match status {
        ProjectBrainStatus::NotInitialized | ProjectBrainStatus::NeedsGoal => {
            "项目还没有确认目标，先把 Goal 变成可确认文档。"
        }
        ProjectBrainStatus::NeedsPlan => "目标已经存在，但当前还缺计划文档，不能直接进入项目循环。",
        ProjectBrainStatus::NeedsConfirmation => {
            "Goal / Plan / Decisions 还没有全部确认，先把 Project Brain 定稿。"
        }
        ProjectBrainStatus::ReadyForProjectLoop => {
            "Goal / Plan / Decisions 已就绪，可以把项目正式交给 Spec / Project Loop 继续拆任务。"
        }
        ProjectBrainStatus::NeedsRecheck => {
            if *health_status == ProjectBrainDocumentStatus::Missing {
                "项目主文档已就绪；如果目标或计划发生漂移，再补一轮 Goal Recheck。"
            } else {
                "Project Health 提示需要重新检查目标、计划或当前决策。"
            }
        }
        ProjectBrainStatus::Blocked => {
            if missing_documents.is_empty() {
                "Project Brain 文档里存在阻断标记，先解除阻断再继续。"
            } else {
                "Project Brain 仍有缺失或阻断文档，先补齐再继续。"
            }
        }
    }
}

pub fn list_spec_issues(project_root: impl AsRef<Path>) -> Result<Vec<SpecIssue>> {
    let root = canonical_project_root(project_root)?;
    let mut issues: Vec<SpecIssue> = read_json_files(&root.join(".agentflow/spec/issues"))?;
    issues.sort_by(|left, right| left.issue_id.cmp(&right.issue_id));
    Ok(issues)
}

pub fn list_spec_projects(project_root: impl AsRef<Path>) -> Result<Vec<SpecProject>> {
    let root = canonical_project_root(project_root)?;
    let mut projects: Vec<SpecProject> = read_json_files(&root.join(".agentflow/spec/projects"))?;
    projects.sort_by(|left, right| left.project_id.cmp(&right.project_id));
    Ok(projects)
}

pub fn project_from_requirement(
    project_root: impl AsRef<Path>,
    requirement_path: impl AsRef<Path>,
    draft: SpecProjectDraft,
) -> Result<SpecProject> {
    let root = canonical_project_root(project_root)?;
    let requirement = read_requirement_document(&root, requirement_path)?;
    validate_project_id(&draft.project_id)?;
    let project_path = format!(".agentflow/spec/projects/{}.json", draft.project_id);
    let now = unix_timestamp_seconds();
    let title = draft.title.unwrap_or_else(|| requirement.title.clone());
    let summary = draft.summary.unwrap_or_else(|| requirement.summary.clone());
    let objective = draft.objective.unwrap_or_else(|| summary.clone());

    Ok(SpecProject {
        version: SPEC_PROJECT_VERSION.to_string(),
        project_id: draft.project_id,
        source_requirement_id: requirement.requirement_id,
        source_requirement_path: requirement.path.clone(),
        title,
        summary,
        objective,
        issue_ids: draft.issue_ids,
        status: SpecProjectStatus::Planned,
        system: SpecSystemRecord {
            created_by: "spec-agent".to_string(),
            created_at: now,
            updated_at: now,
            path: project_path,
            public_requirement_path: requirement.path,
        },
    })
}

fn rebuild_spec_index(root: &Path) -> Result<()> {
    let projects: Vec<SpecProject> = read_json_files(&root.join(".agentflow/spec/projects"))?;
    let issues: Vec<SpecIssue> = read_json_files(&root.join(".agentflow/spec/issues"))?;

    let mut project_entries = projects
        .into_iter()
        .map(|project| SpecProjectIndexEntry {
            project_id: project.project_id,
            path: project.system.path,
            title: project.title,
            status: project.status,
            issue_count: project.issue_ids.len(),
        })
        .collect::<Vec<_>>();
    project_entries.sort_by(|left, right| left.project_id.cmp(&right.project_id));

    let mut issue_entries = issues
        .into_iter()
        .map(|issue| SpecIssueIndexEntry {
            issue_id: issue.issue_id,
            project_id: issue.project_id,
            path: issue.system.path,
            title: issue.title,
            status: issue.status,
            workflow_ref: issue.workflow_ref,
        })
        .collect::<Vec<_>>();
    issue_entries.sort_by(|left, right| left.issue_id.cmp(&right.issue_id));

    write_json(
        &root.join(".agentflow/spec/index.json"),
        &SpecIndex {
            version: SPEC_INDEX_VERSION.to_string(),
            updated_at: unix_timestamp_seconds(),
            projects: project_entries,
            issues: issue_entries,
        },
    )
}

fn read_requirement_document(
    root: &Path,
    requirement_path: impl AsRef<Path>,
) -> Result<RequirementDocument> {
    let relative = normalize_relative_path(root, requirement_path)?;
    if !relative.starts_with("docs/requirements/") {
        anyhow::bail!("requirement document must live under docs/requirements, found {relative}");
    }
    if !relative.ends_with(".md") {
        anyhow::bail!("requirement document must be markdown, found {relative}");
    }
    let path = root.join(&relative);
    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let requirement_id = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("requirement")
        .to_string();
    let title = extract_title(&raw).unwrap_or_else(|| requirement_id.clone());
    let summary = extract_summary(&raw).unwrap_or_else(|| title.clone());
    Ok(RequirementDocument {
        requirement_id,
        path: relative,
        title,
        summary,
    })
}

fn validate_issue_contract(issue: &SpecIssue) -> Result<()> {
    validate_issue_id(&issue.issue_id)?;
    if issue.workflow_ref.trim().is_empty() {
        anyhow::bail!("issue {} missing workflowRef", issue.issue_id);
    }
    if issue.source_requirement_path.trim().is_empty()
        || !issue
            .source_requirement_path
            .starts_with("docs/requirements/")
    {
        anyhow::bail!(
            "issue {} must reference docs/requirements source",
            issue.issue_id
        );
    }
    if issue.expected_outputs.task_run_dir.trim().is_empty()
        || !issue
            .expected_outputs
            .task_run_dir
            .starts_with(&format!(".agentflow/tasks/{}/runs/", issue.issue_id))
    {
        anyhow::bail!("issue {} has invalid taskRunDir", issue.issue_id);
    }
    if issue.expected_outputs.evidence_path.trim().is_empty()
        || !issue
            .expected_outputs
            .evidence_path
            .starts_with(&format!(".agentflow/tasks/{}/evidence/", issue.issue_id))
    {
        anyhow::bail!("issue {} has invalid evidencePath", issue.issue_id);
    }
    Ok(())
}

fn validate_project_contract(project: &SpecProject) -> Result<()> {
    validate_project_id(&project.project_id)?;
    if project.source_requirement_path.trim().is_empty()
        || !project
            .source_requirement_path
            .starts_with("docs/requirements/")
    {
        anyhow::bail!(
            "project {} must reference docs/requirements source",
            project.project_id
        );
    }
    Ok(())
}

fn validate_issue_id(issue_id: &str) -> Result<()> {
    if issue_id.trim().is_empty() {
        anyhow::bail!("issueId is required");
    }
    let Some((prefix, number)) = issue_id.rsplit_once('-') else {
        anyhow::bail!("issueId must use <prefix>-<number>, found {issue_id}");
    };
    if prefix.trim().is_empty() || number.trim().is_empty() {
        anyhow::bail!("issueId must use <prefix>-<number>, found {issue_id}");
    }
    if !number.chars().all(|ch| ch.is_ascii_digit()) {
        anyhow::bail!("issueId numeric suffix must be digits, found {issue_id}");
    }
    Ok(())
}

fn validate_project_id(project_id: &str) -> Result<()> {
    if project_id.trim().is_empty() {
        anyhow::bail!("projectId is required");
    }
    Ok(())
}

fn normalize_relative_path(root: &Path, path: impl AsRef<Path>) -> Result<String> {
    let path = path.as_ref();
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    };
    let absolute = if absolute.exists() {
        absolute
            .canonicalize()
            .with_context(|| format!("canonicalize {}", absolute.display()))?
    } else {
        absolute
    };
    let relative = absolute
        .strip_prefix(root)
        .with_context(|| format!("{} is outside {}", absolute.display(), root.display()))?;
    Ok(relative
        .components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/"))
}

#[derive(Debug, Clone, Default)]
struct DocumentProbe {
    exists: bool,
    updated_at: Option<u64>,
    raw: Option<String>,
}

fn project_brain_root(project_id: &str) -> String {
    format!("docs/projects/{project_id}")
}

fn read_document_probe(root: &Path, relative_path: &str) -> Result<DocumentProbe> {
    let path = root.join(relative_path);
    if !path.exists() {
        return Ok(DocumentProbe::default());
    }
    let metadata = fs::metadata(&path).with_context(|| format!("metadata {}", path.display()))?;
    let updated_at = metadata
        .modified()
        .ok()
        .and_then(|timestamp| timestamp.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs());
    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    Ok(DocumentProbe {
        exists: true,
        updated_at,
        raw: Some(raw),
    })
}

fn classify_project_brain_document(probe: &DocumentProbe) -> ProjectBrainDocumentStatus {
    if !probe.exists {
        return ProjectBrainDocumentStatus::Missing;
    }
    let raw = probe
        .raw
        .as_deref()
        .unwrap_or_default()
        .to_ascii_lowercase();
    if raw.contains("needs-confirmation") || raw.contains("待确认") {
        return ProjectBrainDocumentStatus::NeedsConfirmation;
    }
    if raw.contains("blocked") || raw.contains("阻断") {
        return ProjectBrainDocumentStatus::Blocked;
    }
    if raw.contains("draft") || raw.contains("草稿") {
        return ProjectBrainDocumentStatus::Draft;
    }
    ProjectBrainDocumentStatus::Confirmed
}

fn classify_optional_project_health_document(probe: &DocumentProbe) -> ProjectBrainDocumentStatus {
    if !probe.exists {
        return ProjectBrainDocumentStatus::Missing;
    }
    classify_project_brain_document(probe)
}

fn extract_title(raw: &str) -> Option<String> {
    raw.lines()
        .find_map(|line| line.trim().strip_prefix("# ").map(str::trim))
        .filter(|line| !line.is_empty())
        .map(str::to_string)
}

fn extract_summary(raw: &str) -> Option<String> {
    raw.lines()
        .map(str::trim)
        .find(|line| {
            !line.is_empty()
                && !line.starts_with('#')
                && !line.starts_with('>')
                && !line.starts_with("```")
        })
        .map(str::to_string)
}

fn empty_index() -> SpecIndex {
    SpecIndex {
        version: SPEC_INDEX_VERSION.to_string(),
        updated_at: unix_timestamp_seconds(),
        projects: Vec::new(),
        issues: Vec::new(),
    }
}

fn canonical_project_root(project_root: impl AsRef<Path>) -> Result<PathBuf> {
    let root = project_root.as_ref();
    if root.exists() {
        return root
            .canonicalize()
            .with_context(|| format!("canonicalize {}", root.display()));
    }
    Ok(root.to_path_buf())
}

fn ensure_directory(path: &Path) -> Result<()> {
    if path.exists() {
        if path.is_dir() {
            return Ok(());
        }
        anyhow::bail!("{} exists but is not a directory", path.display());
    }
    fs::create_dir_all(path).with_context(|| format!("create {}", path.display()))
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        ensure_directory(parent)?;
    }
    let content = serde_json::to_string_pretty(value)? + "\n";
    fs::write(path, content).with_context(|| format!("write {}", path.display()))
}

fn read_json<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

fn read_json_files<T: DeserializeOwned>(directory: &Path) -> Result<Vec<T>> {
    if !directory.exists() {
        return Ok(Vec::new());
    }
    let mut entries = fs::read_dir(directory)
        .with_context(|| format!("read directory {}", directory.display()))?
        .collect::<Result<Vec<_>, _>>()
        .with_context(|| format!("collect directory {}", directory.display()))?;
    entries.sort_by_key(|entry| entry.path());
    entries
        .into_iter()
        .filter(|entry| entry.path().extension().and_then(|value| value.to_str()) == Some("json"))
        .map(|entry| read_json::<T>(&entry.path()))
        .collect()
}

fn unix_timestamp_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{SpecPriority, DEFAULT_WORKFLOW_REF};
    use serde_json::Value;
    use tempfile::tempdir;

    fn write_requirement(root: &Path) -> PathBuf {
        let path = root.join("docs/requirements/999-task-workflow-test.md");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "# 任务工作流测试\n\n把任务运行状态改成事件驱动。\n").unwrap();
        path
    }

    #[test]
    fn prepare_creates_spec_workspace_layout() {
        let dir = tempdir().unwrap();
        let summary = prepare_spec_workspace(dir.path()).unwrap();

        for relative in SPEC_DIRECTORIES {
            assert!(dir.path().join(relative).is_dir(), "{relative}");
        }
        for relative in SPEC_REQUIRED_FILES {
            assert!(dir.path().join(relative).is_file(), "{relative}");
        }
        assert_eq!(summary.manifest_path, ".agentflow/spec/manifest.json");
        assert_eq!(summary.index_path, ".agentflow/spec/index.json");
    }

    #[test]
    fn issue_from_requirement_writes_backlog_contract_and_index() {
        let dir = tempdir().unwrap();
        let requirement = write_requirement(dir.path());
        let mut draft = SpecIssueDraft::new("AF-SPEC-001");
        draft.project_id = Some("project-spec-test".to_string());
        draft.priority = SpecPriority::P1;
        draft.allowed_paths = vec!["apps/desktop/src/**".to_string(), "docs/**".to_string()];
        draft.validation_commands = vec!["git diff --check".to_string()];

        let issue = issue_from_requirement(dir.path(), &requirement, draft).unwrap();
        let path = write_spec_issue(dir.path(), &issue).unwrap();
        let stored = read_spec_issue(dir.path(), "AF-SPEC-001").unwrap();

        assert_eq!(
            path,
            dir.path()
                .canonicalize()
                .unwrap()
                .join(".agentflow/spec/issues/AF-SPEC-001.json")
        );
        assert_eq!(stored.status, SpecIssueStatus::Backlog);
        assert_eq!(stored.workflow_ref, DEFAULT_WORKFLOW_REF);
        assert_eq!(
            stored.source_requirement_path,
            "docs/requirements/999-task-workflow-test.md"
        );
        assert_eq!(
            stored.expected_outputs.task_run_dir,
            ".agentflow/tasks/AF-SPEC-001/runs/<run-id>"
        );
        assert_eq!(
            stored.expected_outputs.evidence_path,
            ".agentflow/tasks/AF-SPEC-001/evidence/evidence.json"
        );

        let index: Value = read_json(&dir.path().join(".agentflow/spec/index.json")).unwrap();
        assert_eq!(index["issues"][0]["issueId"], "AF-SPEC-001");
    }

    #[test]
    fn project_from_requirement_writes_project_contract_and_index() {
        let dir = tempdir().unwrap();
        let requirement = write_requirement(dir.path());
        let mut draft = SpecProjectDraft::new("project-spec-test");
        draft.issue_ids = vec!["AF-SPEC-001".to_string()];

        let project = project_from_requirement(dir.path(), &requirement, draft).unwrap();
        write_spec_project(dir.path(), &project).unwrap();
        let stored = read_spec_project(dir.path(), "project-spec-test").unwrap();

        assert_eq!(stored.status, SpecProjectStatus::Planned);
        assert_eq!(stored.issue_ids, vec!["AF-SPEC-001"]);
        let index: Value = read_json(&dir.path().join(".agentflow/spec/index.json")).unwrap();
        assert_eq!(index["projects"][0]["projectId"], "project-spec-test");
    }

    #[test]
    fn project_brain_snapshot_reports_missing_documents_until_confirmed() {
        let dir = tempdir().unwrap();
        let requirement = write_requirement(dir.path());
        let draft = SpecProjectDraft::new("project-spec-test");
        let project = project_from_requirement(dir.path(), &requirement, draft).unwrap();
        write_spec_project(dir.path(), &project).unwrap();

        let missing =
            read_project_brain_snapshot(dir.path(), "project-spec-test", &project.title).unwrap();
        assert_eq!(missing.brain_status, ProjectBrainStatus::NotInitialized);
        assert_eq!(
            missing.missing_documents,
            vec!["GOAL.md", "PLAN.md", "DECISIONS.md"]
        );
        assert_eq!(
            missing.health_document,
            "docs/projects/project-spec-test/PROJECT_HEALTH.md"
        );
        assert_eq!(missing.health_status, ProjectBrainDocumentStatus::Missing);
        assert_eq!(missing.next_recommended_action_label, "生成 Goal 草稿预览");
        assert_eq!(missing.next_recommended_action, "create-goal-draft-preview");

        let project_docs = dir.path().join("docs/projects/project-spec-test");
        fs::create_dir_all(&project_docs).unwrap();
        fs::write(project_docs.join("GOAL.md"), "# Goal\n\n已确认目标。\n").unwrap();
        fs::write(project_docs.join("PLAN.md"), "# Plan\n\n已确认计划。\n").unwrap();
        fs::write(
            project_docs.join("DECISIONS.md"),
            "# Decisions\n\n## Decision Log\n\n### 2026-06-18 - Goal confirmation\n",
        )
        .unwrap();

        let ready =
            read_project_brain_snapshot(dir.path(), "project-spec-test", &project.title).unwrap();
        assert_eq!(ready.brain_status, ProjectBrainStatus::ReadyForProjectLoop);
        assert!(ready.missing_documents.is_empty());
        assert_eq!(ready.goal_status, ProjectBrainDocumentStatus::Confirmed);
        assert_eq!(ready.health_status, ProjectBrainDocumentStatus::Missing);
        assert_eq!(ready.next_recommended_action, "start-project-loop");
        assert_eq!(ready.next_recommended_action_label, "进入项目循环");
        assert!(ready
            .next_recommended_action_reason
            .contains("Goal / Plan / Decisions 已就绪"));
    }

    #[test]
    fn project_brain_health_document_can_request_recheck_without_blocking_missing_core_docs() {
        let dir = tempdir().unwrap();
        let requirement = write_requirement(dir.path());
        let draft = SpecProjectDraft::new("project-health-test");
        let project = project_from_requirement(dir.path(), &requirement, draft).unwrap();
        write_spec_project(dir.path(), &project).unwrap();

        let project_docs = dir.path().join("docs/projects/project-health-test");
        fs::create_dir_all(&project_docs).unwrap();
        fs::write(project_docs.join("GOAL.md"), "# Goal\n\n已确认目标。\n").unwrap();
        fs::write(project_docs.join("PLAN.md"), "# Plan\n\n已确认计划。\n").unwrap();
        fs::write(
            project_docs.join("DECISIONS.md"),
            "# Decisions\n\n已确认。\n",
        )
        .unwrap();
        fs::write(
            project_docs.join("PROJECT_HEALTH.md"),
            "# Project Health\n\n待确认：需要重新检查项目状态。\n",
        )
        .unwrap();

        let snapshot =
            read_project_brain_snapshot(dir.path(), "project-health-test", &project.title).unwrap();
        assert_eq!(
            snapshot.health_status,
            ProjectBrainDocumentStatus::NeedsConfirmation
        );
        assert_eq!(snapshot.brain_status, ProjectBrainStatus::NeedsRecheck);
        assert_eq!(snapshot.next_recommended_action, "run-goal-recheck");
        assert_eq!(snapshot.next_recommended_action_label, "重新检查项目目标");
    }

    #[test]
    fn requirement_must_live_under_docs_requirements() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("README.md");
        fs::write(&path, "# Bad\n\nBad input.\n").unwrap();

        let err = issue_from_requirement(dir.path(), &path, SpecIssueDraft::new("AF-SPEC-001"))
            .unwrap_err()
            .to_string();
        assert!(err.contains("docs/requirements"));
    }

    #[test]
    fn issue_id_must_have_numeric_suffix() {
        let dir = tempdir().unwrap();
        let requirement = write_requirement(dir.path());
        let err =
            issue_from_requirement(dir.path(), &requirement, SpecIssueDraft::new("AF-SPEC-A"))
                .unwrap_err()
                .to_string();
        assert!(err.contains("numeric suffix"));
    }
}
