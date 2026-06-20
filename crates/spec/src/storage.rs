use crate::model::{
    CompletionDecisionFacts, CompletionDecisionOutcome, CompletionDecisionRecord,
    CompletionDecisionRuntime, CompletionDecisionState, GoalDraftPreview, GoalDraftStatus,
    IssueContractDraftPreview, MilestoneDraftPreview, PlanDraftPreview, PlanDraftStatus,
    PreviewConfirmationRecord, ProjectBrainDocumentSet, ProjectBrainDocumentStatus,
    ProjectBrainSnapshot, ProjectBrainStatus, RequirementClass, RequirementClassificationResult,
    RequirementDocument, RequirementExecutionPermission, RequirementFactImpact,
    RequirementIntakeResult, RequirementIntentType, RequirementPreviewLifecycle,
    RequirementPreviewRuntime, RequirementRiskLevel, RequirementTargetObject,
    SpecArtifactAuthority, SpecExpectedOutputs, SpecIssue, SpecIssueCategory, SpecIssueDraft,
    SpecIssueStatus, SpecLoopRequirementManifest, SpecLoopStageArtifact, SpecLoopStageFileRef,
    SpecLoopStageName, SpecLoopStageStatus, SpecPriority, SpecProject, SpecProjectDraft,
    SpecProjectStatus, SpecRequiredAgentRole, SpecSystemRecord, COMPLETION_DECISION_VERSION,
    PROJECT_BRAIN_DOCUMENT_SET_VERSION, PROJECT_BRAIN_SNAPSHOT_VERSION,
    REQUIREMENT_CLASSIFICATION_VERSION, REQUIREMENT_PREVIEW_VERSION, SPEC_INDEX_VERSION,
    SPEC_ISSUE_VERSION, SPEC_MANIFEST_VERSION, SPEC_PROJECT_VERSION,
    SPEC_REQUIREMENT_MANIFEST_VERSION, SPEC_STAGE_ARTIFACT_VERSION,
};
use agentflow_audit::load_project_audit_review_summary;
use agentflow_task_artifacts::load_task_evidence;
use agentflow_workflow_core::{
    canonicalize_project_root, join_relative_path, normalize_relative_path_string,
    normalize_relative_to_root, validate_safe_local_id, IssueId, ProjectId,
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
    ".agentflow/spec/completions",
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

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CloseoutProofSnapshot {
    #[serde(default)]
    merged: bool,
    #[serde(default)]
    issue_closed: bool,
    #[serde(default)]
    public_delivery_written: bool,
    #[serde(default)]
    pr_url: Option<String>,
    #[serde(default)]
    merge_commit_sha: Option<String>,
    #[serde(default)]
    changelog_path: Option<String>,
    #[serde(default)]
    release_notes_path: Option<String>,
}

#[derive(Debug, Clone, Default)]
struct CompletionDeliveryFacts {
    status: String,
    missing_count: usize,
}

pub fn prepare_spec_workspace(project_root: impl AsRef<Path>) -> Result<SpecWorkspaceSummary> {
    let root = canonicalize_project_root(project_root)?;
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
    let root = canonicalize_project_root(project_root)?;
    let requirement = read_requirement_document(&root, requirement_path)?;
    let issue_id = IssueId::parse(&draft.issue_id)?;
    let issue_path = normalize_relative_to_root(&root, spec_issue_path(&root, issue_id.as_str())?)?;
    let now = unix_timestamp_seconds();
    let title = draft.title.unwrap_or_else(|| requirement.title.clone());
    let summary = draft.summary.unwrap_or_else(|| requirement.summary.clone());
    let source_spec_id = draft
        .source_spec_id
        .unwrap_or_else(|| requirement.requirement_id.clone());

    Ok(SpecIssue {
        version: SPEC_ISSUE_VERSION.to_string(),
        issue_id: issue_id.as_str().to_string(),
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
    let root = canonicalize_project_root(project_root)?;
    prepare_spec_workspace(&root)?;
    validate_issue_contract(issue)?;
    let path = spec_issue_path(&root, &issue.issue_id)?;
    write_json(&path, issue)?;
    rebuild_spec_index(&root)?;
    Ok(path)
}

pub fn write_spec_project(
    project_root: impl AsRef<Path>,
    project: &SpecProject,
) -> Result<PathBuf> {
    let root = canonicalize_project_root(project_root)?;
    prepare_spec_workspace(&root)?;
    validate_project_contract(project)?;
    let path = spec_project_path(&root, &project.project_id)?;
    write_json(&path, project)?;
    rebuild_spec_index(&root)?;
    Ok(path)
}

pub fn read_spec_issue(project_root: impl AsRef<Path>, issue_id: &str) -> Result<SpecIssue> {
    let root = canonicalize_project_root(project_root)?;
    read_json(&spec_issue_path(&root, issue_id)?)
}

pub fn update_spec_issue_status(
    project_root: impl AsRef<Path>,
    issue_id: &str,
    status: SpecIssueStatus,
) -> Result<SpecIssue> {
    let root = canonicalize_project_root(project_root)?;
    let mut issue = read_spec_issue(&root, issue_id)?;
    if issue.status != status {
        issue.status = status;
        issue.system.updated_at = unix_timestamp_seconds();
        write_spec_issue(&root, &issue)?;
    }
    Ok(issue)
}

pub fn read_spec_project(project_root: impl AsRef<Path>, project_id: &str) -> Result<SpecProject> {
    let root = canonicalize_project_root(project_root)?;
    read_json(&spec_project_path(&root, project_id)?)
}

pub fn requirement_preview_from_requirement(
    project_root: impl AsRef<Path>,
    requirement_path: impl AsRef<Path>,
    project_id: Option<&str>,
) -> Result<RequirementPreviewRuntime> {
    let root = canonicalize_project_root(project_root)?;
    prepare_spec_workspace(&root)?;
    let requirement = read_requirement_document(&root, requirement_path)?;
    let existing = read_requirement_preview_runtime(&root, &requirement.requirement_id).ok();
    if existing
        .as_ref()
        .is_some_and(|preview| preview.lifecycle == RequirementPreviewLifecycle::Materialized)
    {
        anyhow::bail!(
            "requirement {} already materialized; reset is not allowed in preview runtime",
            requirement.requirement_id
        );
    }

    let now = unix_timestamp_seconds();
    let project_id = project_id
        .map(str::to_string)
        .or_else(|| existing.as_ref().map(|preview| preview.project_id.clone()))
        .unwrap_or_else(|| default_preview_project_id(&requirement.requirement_id));
    ProjectId::parse(&project_id)?;
    let revision = existing
        .as_ref()
        .map(|preview| preview.revision.saturating_add(1))
        .unwrap_or(1);
    let intake = build_requirement_intake(&requirement, &project_id);
    let goal_draft = build_goal_draft_preview(&requirement, &project_id, &intake, revision);

    let preview = RequirementPreviewRuntime {
        version: REQUIREMENT_PREVIEW_VERSION.to_string(),
        requirement_id: requirement.requirement_id.clone(),
        requirement_path: requirement.path.clone(),
        project_id: project_id.clone(),
        project_title: requirement.title.clone(),
        revision,
        lifecycle: RequirementPreviewLifecycle::Active,
        current_state: "goal_draft".to_string(),
        intake,
        goal_draft,
        plan_draft: None,
        confirmation_records: Vec::new(),
        materialized_project_id: None,
        materialized_issue_ids: Vec::new(),
        next_recommended_action: "confirm-goal-draft-preview".to_string(),
        next_recommended_action_label: "确认 Goal 草稿预览".to_string(),
        next_recommended_action_reason:
            "原始需求已经整理成 Goal 草稿，先确认目标是否成立，再进入 Plan 草稿。".to_string(),
        readonly: true,
        updated_at: now,
    };

    emit_project_preview_transition(
        &root,
        &project_id,
        "intake",
        "project.intake.accepted",
        "spec-agent",
        serde_json::json!({
            "requirementId": preview.requirement_id,
            "requirementPath": preview.requirement_path,
            "revision": preview.revision,
            "goalDraftId": preview.goal_draft.goal_draft_id,
        }),
        &["goal.contract.ready"],
        &["project.goal.capture"],
    )?;
    write_requirement_preview_runtime(&root, &preview)?;
    Ok(preview)
}

pub fn confirm_goal_draft_preview(
    project_root: impl AsRef<Path>,
    requirement_id: &str,
    actor: &str,
) -> Result<RequirementPreviewRuntime> {
    let root = canonicalize_project_root(project_root)?;
    let mut preview = read_requirement_preview_runtime(&root, requirement_id)?;
    ensure_active_preview(&preview)?;
    if preview.current_state != "goal_draft" {
        anyhow::bail!(
            "goal draft confirmation requires goal_draft state, found {}",
            preview.current_state
        );
    }

    preview.goal_draft.status = GoalDraftStatus::Confirmed;
    preview.plan_draft = Some(build_plan_draft_preview(
        &preview.goal_draft,
        preview.revision,
    ));
    preview
        .confirmation_records
        .push(PreviewConfirmationRecord {
            timestamp: unix_timestamp_seconds(),
            actor: actor.to_string(),
            target_type: "goal-draft".to_string(),
            target_id: preview.goal_draft.goal_draft_id.clone(),
            summary: "确认 Goal 草稿，允许进入 Plan Draft Preview。".to_string(),
            decision: "confirmed".to_string(),
            impact: "GOAL.md 成为已确认项目目标，下一步进入 Plan Draft Preview。".to_string(),
            next_action: "confirm-plan-draft-preview".to_string(),
        });
    write_confirmed_goal_document(&root, &preview)?;
    append_decision_entry(
        &root,
        &preview.project_id,
        preview
            .confirmation_records
            .last()
            .expect("goal confirmation record exists"),
    )?;
    emit_project_preview_transition(
        &root,
        &preview.project_id,
        "goal_draft",
        "goal.draft.confirmed",
        actor,
        serde_json::json!({
            "requirementId": preview.requirement_id,
            "goalDraftId": preview.goal_draft.goal_draft_id,
            "planDraftId": preview.plan_draft.as_ref().map(|draft| draft.plan_draft_id.clone()),
            "revision": preview.revision,
        }),
        &["plan.contract.ready"],
        &["project.plan.capture"],
    )?;
    preview.current_state = "plan_draft".to_string();
    preview.next_recommended_action = "confirm-plan-draft-preview".to_string();
    preview.next_recommended_action_label = "确认 Plan 草稿预览".to_string();
    preview.next_recommended_action_reason =
        "Goal 已确认，当前需要确认阶段计划和候选执行合同。".to_string();
    preview.updated_at = unix_timestamp_seconds();
    write_requirement_preview_runtime(&root, &preview)?;
    Ok(preview)
}

pub fn confirm_plan_draft_preview(
    project_root: impl AsRef<Path>,
    requirement_id: &str,
    actor: &str,
) -> Result<RequirementPreviewRuntime> {
    let root = canonicalize_project_root(project_root)?;
    let mut preview = read_requirement_preview_runtime(&root, requirement_id)?;
    ensure_active_preview(&preview)?;
    if preview.current_state != "plan_draft" {
        anyhow::bail!(
            "plan draft confirmation requires plan_draft state, found {}",
            preview.current_state
        );
    }
    let plan_draft_id = {
        let plan_draft = preview
            .plan_draft
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("plan draft is missing"))?;
        plan_draft.status = PlanDraftStatus::Confirmed;
        plan_draft.plan_draft_id.clone()
    };
    preview
        .confirmation_records
        .push(PreviewConfirmationRecord {
            timestamp: unix_timestamp_seconds(),
            actor: actor.to_string(),
            target_type: "plan-draft".to_string(),
            target_id: plan_draft_id.clone(),
            summary: "确认 Plan 草稿，允许物化 SpecProject / SpecIssue。".to_string(),
            decision: "confirmed".to_string(),
            impact: "PLAN.md 成为已确认项目计划，下一步可以正式物化任务合同。".to_string(),
            next_action: "materialize-spec-project-and-issues".to_string(),
        });
    let requirement_id = preview.requirement_id.clone();
    let revision = preview.revision;
    write_confirmed_plan_document(&root, &preview)?;
    append_decision_entry(
        &root,
        &preview.project_id,
        preview
            .confirmation_records
            .last()
            .expect("plan confirmation record exists"),
    )?;
    emit_project_preview_transition(
        &root,
        &preview.project_id,
        "plan_draft",
        "plan.draft.confirmed",
        actor,
        serde_json::json!({
            "requirementId": requirement_id,
            "planDraftId": plan_draft_id,
            "revision": revision,
        }),
        &["project.confirmed"],
        &["project.confirm.write"],
    )?;
    preview.current_state = "confirmed".to_string();
    preview.next_recommended_action = "materialize-spec-project-and-issues".to_string();
    preview.next_recommended_action_label = "物化 SpecProject / SpecIssue".to_string();
    preview.next_recommended_action_reason =
        "Goal 和 Plan 都已确认，可以把预览草稿物化成项目循环可读取的结构化合同。".to_string();
    preview.updated_at = unix_timestamp_seconds();
    write_requirement_preview_runtime(&root, &preview)?;
    Ok(preview)
}

pub fn cancel_requirement_preview(
    project_root: impl AsRef<Path>,
    requirement_id: &str,
    reason: &str,
) -> Result<RequirementPreviewRuntime> {
    let root = canonicalize_project_root(project_root)?;
    let mut preview = read_requirement_preview_runtime(&root, requirement_id)?;
    preview.lifecycle = RequirementPreviewLifecycle::Cancelled;
    preview.next_recommended_action = "start-new-requirement".to_string();
    preview.next_recommended_action_label = "开始新需求".to_string();
    preview.next_recommended_action_reason =
        format!("当前预览已取消：{reason}。旧预览不会继续进入项目循环。");
    preview.updated_at = unix_timestamp_seconds();
    write_requirement_preview_runtime(&root, &preview)?;
    Ok(preview)
}

pub fn materialize_spec_from_requirement_preview(
    project_root: impl AsRef<Path>,
    requirement_id: &str,
) -> Result<(SpecProject, Vec<SpecIssue>)> {
    let root = canonicalize_project_root(project_root)?;
    let mut preview = read_requirement_preview_runtime(&root, requirement_id)?;
    ensure_active_preview(&preview)?;
    if preview.current_state != "confirmed" {
        anyhow::bail!(
            "preview must be confirmed before materialization, found {}",
            preview.current_state
        );
    }
    let plan_draft = preview
        .plan_draft
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("plan draft is missing"))?;
    if plan_draft.status != PlanDraftStatus::Confirmed {
        anyhow::bail!("plan draft must be confirmed before materialization");
    }

    let issue_ids = plan_draft
        .issue_contract_drafts
        .iter()
        .map(|draft| draft.issue_draft_id.clone())
        .collect::<Vec<_>>();

    let mut project_draft = SpecProjectDraft::new(preview.project_id.clone());
    project_draft.title = Some(preview.project_title.clone());
    project_draft.summary = Some(preview.goal_draft.outcome.clone());
    project_draft.objective = Some(preview.goal_draft.outcome.clone());
    project_draft.issue_ids = issue_ids.clone();
    let project =
        project_from_requirement(&root, root.join(&preview.requirement_path), project_draft)?;
    write_spec_project(&root, &project)?;

    let issues = plan_draft
        .issue_contract_drafts
        .iter()
        .map(|draft| {
            let mut issue_draft = SpecIssueDraft::new(draft.issue_draft_id.clone());
            issue_draft.title = Some(draft.title.clone());
            issue_draft.summary = Some(draft.goal.clone());
            issue_draft.source_spec_id = Some(plan_draft.plan_draft_id.clone());
            issue_draft.project_id = Some(preview.project_id.clone());
            issue_draft.priority = draft.priority.clone();
            issue_draft.allowed_paths = draft.boundary.clone();
            issue_draft.validation_commands = draft.validation_commands.clone();
            issue_from_requirement(&root, root.join(&preview.requirement_path), issue_draft)
        })
        .collect::<Result<Vec<_>>>()?;
    for issue in &issues {
        write_spec_issue(&root, issue)?;
    }

    preview.lifecycle = RequirementPreviewLifecycle::Materialized;
    preview.materialized_project_id = Some(project.project_id.clone());
    preview.materialized_issue_ids = issue_ids;
    preview.next_recommended_action = "start-project-loop".to_string();
    preview.next_recommended_action_label = "进入项目循环".to_string();
    preview.next_recommended_action_reason =
        "SpecProject / SpecIssue 已生成，后续由项目循环继续调度。".to_string();
    preview.updated_at = unix_timestamp_seconds();
    write_requirement_preview_runtime(&root, &preview)?;
    Ok((project, issues))
}

pub fn write_requirement_preview_runtime(
    project_root: impl AsRef<Path>,
    preview: &RequirementPreviewRuntime,
) -> Result<PathBuf> {
    let root = canonicalize_project_root(project_root)?;
    prepare_spec_workspace(&root)?;
    let path = requirement_preview_runtime_path(&root, &preview.requirement_id)?;
    write_json(&path, preview)?;
    sync_requirement_preview_stage_contracts(&root, preview)?;
    let legacy_path = legacy_requirement_preview_path(&root, &preview.requirement_id)?;
    if legacy_path.is_file() {
        fs::remove_file(&legacy_path)
            .with_context(|| format!("remove {}", legacy_path.display()))?;
    }
    Ok(path)
}

pub fn read_requirement_preview_runtime(
    project_root: impl AsRef<Path>,
    requirement_id: &str,
) -> Result<RequirementPreviewRuntime> {
    let root = canonicalize_project_root(project_root)?;
    let runtime_path = requirement_preview_runtime_path(&root, requirement_id)?;
    if runtime_path.is_file() {
        return read_json(&runtime_path);
    }
    read_json(&legacy_requirement_preview_path(&root, requirement_id)?)
}

pub fn list_requirement_preview_runtimes(
    project_root: impl AsRef<Path>,
) -> Result<Vec<RequirementPreviewRuntime>> {
    let root = canonicalize_project_root(project_root)?;
    let requirements_root = root.join(".agentflow/spec/requirements");
    if !requirements_root.exists() {
        return Ok(Vec::new());
    }
    let mut previews: Vec<RequirementPreviewRuntime> = Vec::new();
    for entry in fs::read_dir(&requirements_root)
        .with_context(|| format!("read {}", requirements_root.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let runtime_path = path.join("runtime.json");
            if runtime_path.is_file() {
                previews.push(read_json(&runtime_path)?);
            }
            continue;
        }
        if path.extension().and_then(|value| value.to_str()) == Some("json") {
            previews.push(read_json(&path)?);
        }
    }
    previews.sort_by(|left, right| left.requirement_id.cmp(&right.requirement_id));
    Ok(previews)
}

pub fn write_completion_decision_runtime(
    project_root: impl AsRef<Path>,
    runtime: &CompletionDecisionRuntime,
) -> Result<PathBuf> {
    let root = canonicalize_project_root(project_root)?;
    prepare_spec_workspace(&root)?;
    let path = completion_decision_path(&root, &runtime.project_id)?;
    write_json(&path, runtime)?;
    Ok(path)
}

pub fn read_completion_decision_runtime(
    project_root: impl AsRef<Path>,
    project_id: &str,
) -> Result<CompletionDecisionRuntime> {
    let root = canonicalize_project_root(project_root)?;
    read_json(&completion_decision_path(&root, project_id)?)
}

pub fn list_completion_decision_runtimes(
    project_root: impl AsRef<Path>,
) -> Result<Vec<CompletionDecisionRuntime>> {
    let root = canonicalize_project_root(project_root)?;
    let mut runtimes: Vec<CompletionDecisionRuntime> =
        read_json_files(&root.join(".agentflow/spec/completions"))?;
    runtimes.sort_by(|left, right| left.project_id.cmp(&right.project_id));
    Ok(runtimes)
}

pub fn sync_completion_decision_runtimes(
    project_root: impl AsRef<Path>,
) -> Result<Vec<CompletionDecisionRuntime>> {
    let root = canonicalize_project_root(project_root)?;
    prepare_spec_workspace(&root)?;
    let projects = list_spec_projects(&root)?;
    let issues = list_spec_issues(&root)?;
    let mut runtimes = Vec::new();

    for project in projects {
        let project_issues = issues
            .iter()
            .filter(|issue| issue.project_id.as_deref() == Some(project.project_id.as_str()))
            .cloned()
            .collect::<Vec<_>>();
        if project_issues.is_empty() {
            continue;
        }

        let facts = build_completion_facts(&root, &project, &project_issues);
        let all_finished = facts.remaining_issue_count == 0;
        let existing = read_completion_decision_runtime(&root, &project.project_id).ok();

        if !all_finished && existing.is_none() {
            continue;
        }

        let runtime = sync_completion_runtime_for_project(&project, facts, existing, all_finished)?;
        write_completion_decision_runtime(&root, &runtime)?;
        runtimes.push(runtime);
    }

    runtimes.sort_by(|left, right| left.project_id.cmp(&right.project_id));
    Ok(runtimes)
}

pub fn record_completion_decision(
    project_root: impl AsRef<Path>,
    project_id: &str,
    outcome: CompletionDecisionOutcome,
    actor: &str,
    summary: &str,
    rationale: Vec<String>,
) -> Result<CompletionDecisionRuntime> {
    let root = canonicalize_project_root(project_root)?;
    let mut runtime = read_completion_decision_runtime(&root, project_id)?;
    if matches!(outcome, CompletionDecisionOutcome::Accept) {
        let blockers = completion_accept_blockers(&runtime.facts);
        if !blockers.is_empty() {
            anyhow::bail!(
                "completion accept blocked for {}: {}",
                project_id,
                blockers.join("；")
            );
        }
    }
    let previous_state = runtime.current_state.clone();
    runtime.history.push(CompletionDecisionRecord {
        actor: actor.to_string(),
        outcome: outcome.clone(),
        summary: summary.to_string(),
        rationale: rationale.clone(),
        decided_at: unix_timestamp_seconds(),
    });
    runtime.latest_outcome = Some(outcome.clone());
    runtime.current_state = completion_state_for_outcome(&outcome);
    runtime.rationale = rationale;
    runtime.open_questions = completion_open_questions_for_state(&runtime.current_state);
    let (action, label, reason) = completion_next_action_bundle(
        &runtime.current_state,
        runtime.latest_outcome.as_ref(),
        &runtime.facts,
    );
    runtime.next_recommended_action = action;
    runtime.next_recommended_action_label = label;
    runtime.next_recommended_action_reason = reason;
    runtime.updated_at = unix_timestamp_seconds();

    match outcome {
        CompletionDecisionOutcome::Accept => emit_completion_acceptance(&root, &runtime, actor)?,
        _ => emit_completion_recheck_event(&root, &runtime, actor, &previous_state, summary)?,
    }
    write_completion_decision_runtime(&root, &runtime)?;
    Ok(runtime)
}

pub fn read_project_brain_document_set(
    project_root: impl AsRef<Path>,
    project_id: &str,
) -> Result<ProjectBrainDocumentSet> {
    let root = canonicalize_project_root(project_root)?;
    let project_path = project_brain_root(project_id)?;
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
    let root = canonicalize_project_root(project_root)?;
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
    let root = canonicalize_project_root(project_root)?;
    let mut issues: Vec<SpecIssue> = read_json_files(&root.join(".agentflow/spec/issues"))?;
    issues.sort_by(|left, right| left.issue_id.cmp(&right.issue_id));
    Ok(issues)
}

pub fn list_spec_projects(project_root: impl AsRef<Path>) -> Result<Vec<SpecProject>> {
    let root = canonicalize_project_root(project_root)?;
    let mut projects: Vec<SpecProject> = read_json_files(&root.join(".agentflow/spec/projects"))?;
    projects.sort_by(|left, right| left.project_id.cmp(&right.project_id));
    Ok(projects)
}

pub fn project_from_requirement(
    project_root: impl AsRef<Path>,
    requirement_path: impl AsRef<Path>,
    draft: SpecProjectDraft,
) -> Result<SpecProject> {
    let root = canonicalize_project_root(project_root)?;
    let requirement = read_requirement_document(&root, requirement_path)?;
    let project_id = ProjectId::parse(&draft.project_id)?;
    let project_path =
        normalize_relative_to_root(&root, spec_project_path(&root, project_id.as_str())?)?;
    let now = unix_timestamp_seconds();
    let title = draft.title.unwrap_or_else(|| requirement.title.clone());
    let summary = draft.summary.unwrap_or_else(|| requirement.summary.clone());
    let objective = draft.objective.unwrap_or_else(|| summary.clone());

    Ok(SpecProject {
        version: SPEC_PROJECT_VERSION.to_string(),
        project_id: project_id.as_str().to_string(),
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
    let relative = normalize_relative_to_root(root, requirement_path)?;
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
        raw_text: raw,
    })
}

fn build_requirement_intake(
    requirement: &RequirementDocument,
    project_id: &str,
) -> RequirementIntakeResult {
    let raw_text = requirement.raw_text.trim().to_string();
    let intent = detect_requirement_intent(requirement);
    let detected_scope = vec![requirement.summary.clone()];
    let detected_deliverables = default_deliverables(&intent);
    let detected_constraints = vec!["确认前不生成执行合同。".to_string()];
    let clarification_questions = if requirement.summary.chars().count() < 16 {
        vec!["这个需求最终要交付什么？".to_string()]
    } else {
        Vec::new()
    };
    let missing_information = if clarification_questions.is_empty() {
        Vec::new()
    } else {
        vec!["最终交付物仍不够明确。".to_string()]
    };
    let confidence = if clarification_questions.is_empty() {
        82
    } else {
        64
    };
    let referenced_files = extract_referenced_files(&raw_text);
    let referenced_urls = extract_referenced_urls(&raw_text);
    let referenced_versions = extract_referenced_versions(&raw_text);
    let referenced_releases = extract_referenced_releases(&raw_text);
    let referenced_branches = extract_referenced_branches(&raw_text);
    let referenced_issues = extract_referenced_issues(&raw_text);
    let referenced_pull_requests = extract_referenced_pull_requests(&raw_text);
    let explicit_actions = extract_explicit_actions(&raw_text);
    let input_sources = detect_input_sources(
        &raw_text,
        &referenced_files,
        &referenced_urls,
        &referenced_issues,
        &referenced_pull_requests,
        &referenced_releases,
    );
    RequirementIntakeResult {
        requirement_id: requirement.requirement_id.clone(),
        project_id: project_id.to_string(),
        raw_text,
        agent_locale: detect_agent_locale(&requirement.raw_text),
        referenced_files,
        referenced_urls,
        referenced_versions,
        referenced_releases,
        referenced_branches,
        referenced_issues,
        referenced_pull_requests,
        explicit_actions,
        input_sources,
        detected_intent: intent,
        detected_scope,
        detected_non_goals: Vec::new(),
        detected_deliverables,
        detected_constraints,
        missing_information,
        clarification_questions,
        confidence,
        next_action: "confirm-goal-draft-preview".to_string(),
    }
}

fn detect_agent_locale(raw: &str) -> String {
    if raw.chars().any(is_cjk_character) {
        "zh-CN".to_string()
    } else {
        "en-US".to_string()
    }
}

fn is_cjk_character(ch: char) -> bool {
    matches!(
        ch as u32,
        0x3400..=0x4DBF | 0x4E00..=0x9FFF | 0xF900..=0xFAFF
    )
}

fn extract_referenced_files(raw: &str) -> Vec<String> {
    let mut values = extract_markdown_link_targets(raw)
        .into_iter()
        .filter(|target| looks_like_file_reference(target))
        .collect::<Vec<_>>();
    values.extend(
        tokenize_requirement_text(raw)
            .into_iter()
            .filter(|token| looks_like_file_reference(token)),
    );
    dedupe_preserve_order(values)
}

fn extract_referenced_urls(raw: &str) -> Vec<String> {
    let mut values = extract_markdown_link_targets(raw)
        .into_iter()
        .filter(|target| target.starts_with("http://") || target.starts_with("https://"))
        .collect::<Vec<_>>();
    values.extend(
        tokenize_requirement_text(raw)
            .into_iter()
            .filter(|token| token.starts_with("http://") || token.starts_with("https://")),
    );
    dedupe_preserve_order(values)
}

fn extract_referenced_versions(raw: &str) -> Vec<String> {
    dedupe_preserve_order(
        tokenize_requirement_text(raw)
            .into_iter()
            .filter(|token| is_version_token(token))
            .collect(),
    )
}

fn extract_referenced_releases(raw: &str) -> Vec<String> {
    let lowered = raw.to_ascii_lowercase();
    let mut values = Vec::new();
    if lowered.contains("release") {
        values.push("release".to_string());
    }
    if lowered.contains("发布") {
        values.push("发布".to_string());
    }
    for token in tokenize_requirement_text(raw) {
        let lowered = token.to_ascii_lowercase();
        if lowered.contains("release") || lowered.contains("tag") {
            values.push(token);
        }
    }
    dedupe_preserve_order(values)
}

fn extract_referenced_branches(raw: &str) -> Vec<String> {
    let mut values = Vec::new();
    for token in tokenize_requirement_text(raw) {
        if token == "main" || token == "master" || token.starts_with("origin/") {
            values.push(token);
            continue;
        }
        if token.contains('/') && !token.starts_with("http") && !looks_like_file_reference(&token) {
            values.push(token);
        }
    }
    dedupe_preserve_order(values)
}

fn extract_referenced_issues(raw: &str) -> Vec<String> {
    dedupe_preserve_order(
        tokenize_requirement_text(raw)
            .into_iter()
            .filter_map(|token| normalize_hash_reference(&token))
            .collect(),
    )
}

fn extract_referenced_pull_requests(raw: &str) -> Vec<String> {
    let tokens = tokenize_requirement_text(raw);
    let mut values = Vec::new();
    for window in tokens.windows(2) {
        let current = window[0].to_ascii_lowercase();
        if matches!(current.as_str(), "pr" | "pull-request" | "mr") {
            if let Some(reference) = normalize_hash_reference(&window[1]) {
                values.push(reference);
            }
        }
    }
    dedupe_preserve_order(values)
}

fn extract_explicit_actions(raw: &str) -> Vec<String> {
    let keywords = [
        "审计",
        "修复",
        "设计",
        "规划",
        "执行",
        "确认",
        "取消",
        "发布",
        "研究",
        "理解",
        "audit",
        "fix",
        "design",
        "plan",
        "execute",
        "confirm",
        "cancel",
        "release",
        "research",
        "understand",
    ];
    let lowered = raw.to_ascii_lowercase();
    let mut values = Vec::new();
    for keyword in keywords {
        if keyword.is_ascii() {
            if lowered.contains(keyword) {
                values.push(keyword.to_string());
            }
        } else if raw.contains(keyword) {
            values.push(keyword.to_string());
        }
    }
    dedupe_preserve_order(values)
}

fn detect_input_sources(
    raw: &str,
    referenced_files: &[String],
    referenced_urls: &[String],
    referenced_issues: &[String],
    referenced_pull_requests: &[String],
    referenced_releases: &[String],
) -> Vec<String> {
    let mut sources = vec!["requirement-document".to_string()];
    if !referenced_files.is_empty() {
        sources.push("file-reference".to_string());
    }
    if !referenced_urls.is_empty() {
        sources.push("url-reference".to_string());
    }
    if !referenced_issues.is_empty() {
        sources.push("issue-reference".to_string());
    }
    if !referenced_pull_requests.is_empty() {
        sources.push("pull-request-reference".to_string());
    }
    if !referenced_releases.is_empty()
        || raw.to_ascii_lowercase().contains("release")
        || raw.contains("发布")
    {
        sources.push("release-reference".to_string());
    }
    dedupe_preserve_order(sources)
}

fn build_requirement_classification(
    intake: &RequirementIntakeResult,
) -> RequirementClassificationResult {
    let raw = intake.raw_text.as_str();
    let lowered = raw.to_ascii_lowercase();
    let tokens = tokenize_requirement_text(raw);

    let has_question = raw.contains('？')
        || raw.contains('?')
        || ["为什么", "怎么", "如何", "是否", "what", "why", "how"]
            .iter()
            .any(|keyword| lowered.contains(&keyword.to_ascii_lowercase()));
    let has_research = [
        "研究",
        "调研",
        "分析",
        "理解",
        "research",
        "investigate",
        "understand",
    ]
    .iter()
    .any(|keyword| lowered.contains(&keyword.to_ascii_lowercase()));
    let has_feature = [
        "新增",
        "实现",
        "添加",
        "支持",
        "功能",
        "开发",
        "接入",
        "feature",
        "implement",
    ]
    .iter()
    .any(|keyword| lowered.contains(&keyword.to_ascii_lowercase()));
    let has_bug = ["修复", "bug", "错误", "异常", "故障", "fix", "regression"]
        .iter()
        .any(|keyword| lowered.contains(&keyword.to_ascii_lowercase()));
    let has_audit = ["审计", "audit", "review", "验收"]
        .iter()
        .any(|keyword| lowered.contains(&keyword.to_ascii_lowercase()));
    let has_design = ["设计", "figma", "ui", "ux", "原型", "交互", "design"]
        .iter()
        .any(|keyword| lowered.contains(&keyword.to_ascii_lowercase()));
    let has_release = raw.contains("发布")
        || lowered.contains("release notes")
        || lowered.contains("changelog")
        || tokens.iter().any(|token| {
            let lowered = token.to_ascii_lowercase();
            lowered == "release" || lowered == "tag" || is_version_token(token)
        });
    let has_maintenance = [
        "维护",
        "upgrade",
        "升级",
        "依赖",
        "dependency",
        "迁移",
        "重构",
    ]
    .iter()
    .any(|keyword| lowered.contains(&keyword.to_ascii_lowercase()));
    let has_cleanup = [
        "清理", "cleanup", "删除", "移除", "去掉", "retire", "remove", "delete",
    ]
    .iter()
    .any(|keyword| lowered.contains(&keyword.to_ascii_lowercase()));
    let has_executable_hint = ["issue", "任务", "执行", "编码", "代码", "build agent"]
        .iter()
        .any(|keyword| lowered.contains(&keyword.to_ascii_lowercase()))
        || !intake.referenced_files.is_empty();
    let design_only = has_design
        && !has_feature
        && !has_bug
        && !has_audit
        && !has_release
        && !has_executable_hint;
    let executable_issue =
        (has_feature || has_bug || has_maintenance || has_cleanup || has_executable_hint)
            && !has_audit
            && !design_only;

    let mut basic_types = Vec::new();
    let mut reasons = Vec::new();

    if has_question {
        push_unique(&mut basic_types, RequirementClass::Question);
        reasons.push("需求包含疑问句或提问关键词。".to_string());
    }
    if has_research {
        push_unique(&mut basic_types, RequirementClass::Research);
        reasons.push("需求包含研究、理解或调研语义。".to_string());
    }
    if has_feature {
        push_unique(&mut basic_types, RequirementClass::Feature);
        reasons.push("需求包含新增、实现或支持类语义。".to_string());
    }
    if has_bug {
        push_unique(&mut basic_types, RequirementClass::Bug);
        reasons.push("需求包含修复或缺陷处理语义。".to_string());
    }
    if has_audit {
        push_unique(&mut basic_types, RequirementClass::Audit);
        reasons.push("需求明确提到了审计或验收。".to_string());
    }
    if design_only {
        push_unique(&mut basic_types, RequirementClass::DesignOnly);
        reasons.push("需求以设计/UI 为主，没有明确代码执行语义。".to_string());
    }
    if executable_issue {
        push_unique(&mut basic_types, RequirementClass::ExecutableIssue);
        reasons.push("需求带有任务/Issue/代码落地语义，可继续进入 Spec Loop。".to_string());
    }
    if has_release {
        push_unique(&mut basic_types, RequirementClass::Release);
        reasons.push("需求涉及 release、tag 或 changelog。".to_string());
    }
    if has_maintenance {
        push_unique(&mut basic_types, RequirementClass::Maintenance);
        reasons.push("需求涉及升级、维护或迁移。".to_string());
    }
    if has_cleanup {
        push_unique(&mut basic_types, RequirementClass::Cleanup);
        reasons.push("需求包含清理、删除或收口动作。".to_string());
    }

    let mut ambiguous = false;
    if basic_types.is_empty() {
        basic_types.push(RequirementClass::Research);
        reasons.push("未命中明确分类关键词，先按 research 保守处理。".to_string());
        ambiguous = true;
    }

    let conflicting = (has_audit && executable_issue)
        || (has_question && executable_issue)
        || (has_question && has_release);
    if conflicting {
        reasons.push("需求同时包含互相竞争的执行语义，需要后续边界检查收口。".to_string());
    }
    if basic_types.len() > 3 {
        ambiguous = true;
        reasons.push("需求同时命中多个类型标签，后续 route 需要更保守处理。".to_string());
    }

    let primary_type = pick_primary_requirement_class(&basic_types);
    let execution_permission = if conflicting {
        RequirementExecutionPermission::PreviewOnly
    } else if basic_types.contains(&RequirementClass::Audit) {
        RequirementExecutionPermission::AuditLoop
    } else if basic_types.contains(&RequirementClass::Release) {
        RequirementExecutionPermission::ReleaseCloseout
    } else if basic_types.len() == 1 && basic_types[0] == RequirementClass::Question {
        RequirementExecutionPermission::AnswerOnly
    } else if basic_types.iter().all(|class| {
        matches!(
            class,
            RequirementClass::Question | RequirementClass::Research | RequirementClass::DesignOnly
        )
    }) {
        RequirementExecutionPermission::PreviewOnly
    } else {
        RequirementExecutionPermission::SpecLoop
    };

    let fact_impacts = match execution_permission {
        RequirementExecutionPermission::AnswerOnly => vec![RequirementFactImpact::ReadOnly],
        RequirementExecutionPermission::PreviewOnly => {
            vec![RequirementFactImpact::RequirementPreview]
        }
        RequirementExecutionPermission::SpecLoop => vec![
            RequirementFactImpact::RequirementPreview,
            RequirementFactImpact::SpecAuthority,
            RequirementFactImpact::RuntimeProposal,
        ],
        RequirementExecutionPermission::AuditLoop => vec![
            RequirementFactImpact::RequirementPreview,
            RequirementFactImpact::AuditSurface,
        ],
        RequirementExecutionPermission::ReleaseCloseout => vec![
            RequirementFactImpact::RequirementPreview,
            RequirementFactImpact::ReleaseSurface,
        ],
    };

    let risk_level = if conflicting || has_audit || has_release {
        RequirementRiskLevel::High
    } else if executable_issue || has_feature || has_bug || has_maintenance {
        RequirementRiskLevel::Medium
    } else {
        RequirementRiskLevel::Low
    };

    let mut target_objects = vec![RequirementTargetObject::Requirement];
    if basic_types.contains(&RequirementClass::ExecutableIssue)
        || basic_types.contains(&RequirementClass::Feature)
        || basic_types.contains(&RequirementClass::Bug)
        || basic_types.contains(&RequirementClass::Maintenance)
        || basic_types.contains(&RequirementClass::Cleanup)
    {
        push_unique(&mut target_objects, RequirementTargetObject::SpecProject);
        push_unique(&mut target_objects, RequirementTargetObject::SpecIssue);
        push_unique(&mut target_objects, RequirementTargetObject::Code);
    }
    if basic_types.contains(&RequirementClass::DesignOnly) {
        push_unique(&mut target_objects, RequirementTargetObject::Design);
    }
    if basic_types.contains(&RequirementClass::Audit) {
        push_unique(&mut target_objects, RequirementTargetObject::Audit);
    }
    if basic_types.contains(&RequirementClass::Release) {
        push_unique(&mut target_objects, RequirementTargetObject::Release);
    }
    if basic_types.contains(&RequirementClass::Question)
        || basic_types.contains(&RequirementClass::Research)
    {
        push_unique(&mut target_objects, RequirementTargetObject::Documentation);
    }

    reasons.push(format!(
        "执行权限判定为 {}。",
        execution_permission.as_str()
    ));
    let confirmation_required = !matches!(
        execution_permission,
        RequirementExecutionPermission::AnswerOnly
    );

    RequirementClassificationResult {
        version: REQUIREMENT_CLASSIFICATION_VERSION.to_string(),
        primary_type,
        basic_types,
        intent_type: intake.detected_intent.clone(),
        execution_permission,
        fact_impacts,
        risk_level,
        target_objects,
        confirmation_required,
        ambiguous,
        conflicting,
        reasons,
    }
}

fn pick_primary_requirement_class(classes: &[RequirementClass]) -> RequirementClass {
    for candidate in [
        RequirementClass::Audit,
        RequirementClass::Release,
        RequirementClass::Bug,
        RequirementClass::Feature,
        RequirementClass::ExecutableIssue,
        RequirementClass::DesignOnly,
        RequirementClass::Maintenance,
        RequirementClass::Cleanup,
        RequirementClass::Research,
        RequirementClass::Question,
    ] {
        if classes.contains(&candidate) {
            return candidate;
        }
    }
    RequirementClass::Research
}

fn extract_markdown_link_targets(raw: &str) -> Vec<String> {
    let bytes = raw.as_bytes();
    let mut values = Vec::new();
    let mut index = 0usize;
    while index + 1 < bytes.len() {
        if bytes[index] == b']' && bytes[index + 1] == b'(' {
            let start = index + 2;
            if let Some(end_offset) = raw[start..].find(')') {
                let target = raw[start..start + end_offset].trim();
                if !target.is_empty() {
                    values.push(target.to_string());
                }
                index = start + end_offset + 1;
                continue;
            }
        }
        index += 1;
    }
    values
}

fn tokenize_requirement_text(raw: &str) -> Vec<String> {
    raw.split(|ch: char| ch.is_whitespace() || matches!(ch, '，' | '。' | ',' | ';' | '；' | '、'))
        .map(trim_token)
        .filter(|token| !token.is_empty())
        .collect()
}

fn trim_token(raw: &str) -> String {
    raw.trim_matches(|ch: char| {
        matches!(
            ch,
            '(' | ')' | '[' | ']' | '{' | '}' | '<' | '>' | ',' | ';' | ':' | '"' | '\'' | '`'
        )
    })
    .trim_end_matches('.')
    .trim_end_matches('。')
    .trim_end_matches('，')
    .to_string()
}

fn looks_like_file_reference(token: &str) -> bool {
    let lowered = token.to_ascii_lowercase();
    let has_known_extension = [
        ".md", ".json", ".yaml", ".yml", ".txt", ".png", ".svg", ".html", ".rs", ".tsx", ".ts",
    ]
    .iter()
    .any(|suffix| lowered.ends_with(suffix));
    has_known_extension
        && (token.contains('/')
            || token.starts_with("docs/")
            || token.starts_with(".agentflow/")
            || token.starts_with("apps/")
            || token.starts_with("crates/"))
}

fn is_version_token(token: &str) -> bool {
    let Some(rest) = token.strip_prefix('v') else {
        return false;
    };
    let mut has_digit = false;
    for ch in rest.chars() {
        if ch.is_ascii_digit() {
            has_digit = true;
            continue;
        }
        if ch == '.' {
            continue;
        }
        return false;
    }
    has_digit
}

fn normalize_hash_reference(token: &str) -> Option<String> {
    let stripped = token.strip_prefix('#')?;
    if stripped.chars().all(|ch| ch.is_ascii_digit()) && !stripped.is_empty() {
        Some(format!("#{stripped}"))
    } else {
        None
    }
}

fn dedupe_preserve_order(values: Vec<String>) -> Vec<String> {
    let mut unique = Vec::new();
    for value in values {
        if !unique.contains(&value) {
            unique.push(value);
        }
    }
    unique
}

fn push_unique<T: PartialEq>(values: &mut Vec<T>, value: T) {
    if !values.contains(&value) {
        values.push(value);
    }
}

fn build_goal_draft_preview(
    requirement: &RequirementDocument,
    project_id: &str,
    intake: &RequirementIntakeResult,
    revision: u32,
) -> GoalDraftPreview {
    GoalDraftPreview {
        goal_draft_id: format!("goal-{}-r{}", requirement.requirement_id, revision),
        project_id: project_id.to_string(),
        source_requirement_id: requirement.requirement_id.clone(),
        title: requirement.title.clone(),
        intent_type: intake.detected_intent.clone(),
        outcome: requirement.summary.clone(),
        target_user: "当前项目使用者".to_string(),
        expected_deliverables: intake.detected_deliverables.clone(),
        scope: intake.detected_scope.clone(),
        non_goals: intake.detected_non_goals.clone(),
        success_criteria: vec!["目标、范围和约束都能被用户确认。".to_string()],
        constraints: intake.detected_constraints.clone(),
        open_questions: intake.clarification_questions.clone(),
        risk_hints: vec!["需求仍可能在计划确认前发生调整。".to_string()],
        confidence: intake.confidence,
        status: if intake.clarification_questions.is_empty() {
            GoalDraftStatus::ReadyForReview
        } else {
            GoalDraftStatus::NeedsClarification
        },
    }
}

fn build_plan_draft_preview(goal: &GoalDraftPreview, revision: u32) -> PlanDraftPreview {
    let stage_plan = vec![
        "Goal confirmation".to_string(),
        "Plan confirmation".to_string(),
        "Task materialization".to_string(),
        "Work / Delivery handoff".to_string(),
    ];
    let issue_prefix = default_issue_prefix(&goal.project_id);
    let issue_contract_drafts = vec![
        IssueContractDraftPreview {
            issue_draft_id: format!("{issue_prefix}-001"),
            title: format!("{}任务拆解与结构化合同", goal.title),
            goal: "把已确认的 Goal / Plan 转成结构化任务合同。".to_string(),
            scope: goal.scope.clone(),
            non_goals: goal.non_goals.clone(),
            dependencies: Vec::new(),
            acceptance_criteria: vec!["任务合同结构完整且可投影。".to_string()],
            validation_commands: vec!["cargo test --workspace".to_string()],
            evidence_requirements: vec!["本地验证结果".to_string()],
            boundary: vec![
                "crates/spec/**".to_string(),
                "crates/projection/**".to_string(),
            ],
            priority: SpecPriority::P1,
            suggested_agent_role: SpecRequiredAgentRole::BuildAgent,
        },
        IssueContractDraftPreview {
            issue_draft_id: format!("{issue_prefix}-002"),
            title: format!("{}交付与验证收口", goal.title),
            goal: "把物化后的项目循环入口与交付边界整理清楚。".to_string(),
            scope: vec!["任务投影".to_string(), "公开交付记录".to_string()],
            non_goals: vec!["不直接执行 Work Loop。".to_string()],
            dependencies: vec![format!("{issue_prefix}-001")],
            acceptance_criteria: vec!["验证和交付边界明确。".to_string()],
            validation_commands: vec!["npm --prefix apps/desktop run build".to_string()],
            evidence_requirements: vec!["构建结果".to_string()],
            boundary: vec!["apps/desktop/src/**".to_string()],
            priority: SpecPriority::P1,
            suggested_agent_role: SpecRequiredAgentRole::BuildAgent,
        },
    ];
    PlanDraftPreview {
        plan_draft_id: format!("plan-{}-r{}", goal.source_requirement_id, revision),
        project_id: goal.project_id.clone(),
        source_goal_id: goal.goal_draft_id.clone(),
        plan_type: goal.intent_type.clone(),
        stage_plan,
        milestone_drafts: vec![
            MilestoneDraftPreview {
                milestone_id: format!("milestone-{}-01", goal.project_id),
                title: "确认 Goal / Plan".to_string(),
                goal: "把目标和计划变成可确认事实。".to_string(),
                depends_on: Vec::new(),
                expected_outputs: vec!["GOAL.md".to_string(), "PLAN.md".to_string()],
                validation_need: "确认文档结构完整。".to_string(),
                evidence_need: "Decision entry".to_string(),
            },
            MilestoneDraftPreview {
                milestone_id: format!("milestone-{}-02", goal.project_id),
                title: "生成任务合同".to_string(),
                goal: "输出 SpecProject / SpecIssue 物化结果。".to_string(),
                depends_on: vec![format!("milestone-{}-01", goal.project_id)],
                expected_outputs: vec!["SpecProject".to_string(), "SpecIssue".to_string()],
                validation_need: "确认任务合同可读取。".to_string(),
                evidence_need: "materialization record".to_string(),
            },
        ],
        issue_contract_drafts,
        validation_strategy: vec!["先确认 Goal，再确认 Plan。".to_string()],
        evidence_strategy: vec!["记录 Goal confirmation 和 Plan confirmation。".to_string()],
        human_confirmation_points: vec![
            "scope change".to_string(),
            "high-risk issue".to_string(),
            "plan structure change".to_string(),
        ],
        risk_list: vec!["未确认前不得直接生成执行合同。".to_string()],
        blockers: Vec::new(),
        next_recommended_action: "confirm-plan-draft-preview".to_string(),
        status: PlanDraftStatus::ReadyForReview,
    }
}

fn write_confirmed_goal_document(root: &Path, preview: &RequirementPreviewRuntime) -> Result<()> {
    let path = root
        .join(project_brain_root(&preview.project_id)?)
        .join("GOAL.md");
    let content = format!(
        "# {}\n\n## Outcome\n{}\n\n## Expected Deliverables\n{}\n\n## Scope\n{}\n\n## Non-goals\n{}\n\n## Success Criteria\n{}\n",
        preview.goal_draft.title,
        preview.goal_draft.outcome,
        markdown_list(&preview.goal_draft.expected_deliverables),
        markdown_list(&preview.goal_draft.scope),
        markdown_list(&preview.goal_draft.non_goals),
        markdown_list(&preview.goal_draft.success_criteria),
    );
    write_text(&path, &content)
}

fn write_confirmed_plan_document(root: &Path, preview: &RequirementPreviewRuntime) -> Result<()> {
    let Some(plan) = preview.plan_draft.as_ref() else {
        anyhow::bail!("plan draft is missing");
    };
    let path = root
        .join(project_brain_root(&preview.project_id)?)
        .join("PLAN.md");
    let content = format!(
        "# {}\n\n## Stage Plan\n{}\n\n## Milestones\n{}\n\n## Human Confirmation Points\n{}\n",
        preview.project_title,
        markdown_list(&plan.stage_plan),
        markdown_list(
            &plan
                .milestone_drafts
                .iter()
                .map(|draft| format!("{}：{}", draft.title, draft.goal))
                .collect::<Vec<_>>()
        ),
        markdown_list(&plan.human_confirmation_points),
    );
    write_text(&path, &content)
}

fn append_decision_entry(
    root: &Path,
    project_id: &str,
    record: &PreviewConfirmationRecord,
) -> Result<()> {
    let path = root
        .join(project_brain_root(project_id)?)
        .join("DECISIONS.md");
    let entry = format!(
        "## {}\n\n- actor: {}\n- target: {} / {}\n- decision: {}\n- impact: {}\n- nextAction: {}\n\n{}\n\n",
        record.timestamp,
        record.actor,
        record.target_type,
        record.target_id,
        record.decision,
        record.impact,
        record.next_action,
        record.summary,
    );
    let next_content = if path.exists() {
        format!("{}{}", fs::read_to_string(&path)?, entry)
    } else {
        format!("# Decisions\n\n{entry}")
    };
    write_text(&path, &next_content)
}

fn emit_project_preview_transition(
    root: &Path,
    project_id: &str,
    current_state: &str,
    event_type: &str,
    actor_role: &str,
    payload: serde_json::Value,
    passed_guards: &[&str],
    completed_actions: &[&str],
) -> Result<()> {
    let workflow = agentflow_workflow_core::canonical_workflow(
        agentflow_workflow_core::WorkflowFlowType::Project,
    );
    let context = agentflow_workflow_runtime::RuntimeContext {
        aggregate_type: "project".to_string(),
        aggregate_id: project_id.to_string(),
        issue_id: None,
        project_id: Some(project_id.to_string()),
        run_id: None,
        actor: agentflow_event_store::EventActor {
            role: actor_role.to_string(),
            kind: "runtime".to_string(),
        },
        correlation_id: Some(format!("corr-project-{project_id}")),
        causation_id: None,
        artifact_refs: Vec::new(),
        payload,
    };
    let guards =
        agentflow_workflow_runtime::StaticGuardRegistry::all_pass(passed_guards.iter().copied());
    let actions = agentflow_workflow_runtime::StaticActionRegistry::all_complete(
        completed_actions.iter().copied(),
    );
    let result = agentflow_workflow_runtime::apply_workflow_event(
        root,
        &workflow,
        current_state,
        event_type,
        context,
        &guards,
        &actions,
    )?;
    if !result.applied {
        anyhow::bail!(
            "project preview transition {} from {} was blocked: {}",
            event_type,
            current_state,
            result
                .blocked_reason
                .unwrap_or_else(|| "unknown reason".to_string())
        );
    }
    Ok(())
}

fn sync_requirement_preview_stage_contracts(
    root: &Path,
    preview: &RequirementPreviewRuntime,
) -> Result<()> {
    let requirement_dir = requirement_preview_dir(root, &preview.requirement_id)?;
    ensure_directory(&requirement_dir)?;
    let runtime_path = requirement_preview_runtime_path(root, &preview.requirement_id)?;
    let runtime_ref = normalize_relative_to_root(root, &runtime_path)?;
    let stage_artifacts = build_requirement_stage_artifacts(root, preview)?;
    let mut stage_files = Vec::new();

    for artifact in stage_artifacts {
        let path = requirement_stage_artifact_path(root, &preview.requirement_id, &artifact.stage)?;
        write_json(&path, &artifact)?;
        stage_files.push(SpecLoopStageFileRef {
            stage: artifact.stage,
            path: normalize_relative_to_root(root, &path)?,
            status: artifact.status,
            authority: artifact.authority,
        });
    }

    let manifest = SpecLoopRequirementManifest {
        version: SPEC_REQUIREMENT_MANIFEST_VERSION.to_string(),
        requirement_id: preview.requirement_id.clone(),
        project_id: preview.project_id.clone(),
        root_path: normalize_relative_to_root(root, &requirement_dir)?,
        runtime_path: runtime_ref,
        stage_files,
        updated_at: preview.updated_at,
    };
    write_json(
        &requirement_manifest_path(root, &preview.requirement_id)?,
        &manifest,
    )
}

fn build_requirement_stage_artifacts(
    root: &Path,
    preview: &RequirementPreviewRuntime,
) -> Result<Vec<SpecLoopStageArtifact>> {
    SpecLoopStageName::all()
        .iter()
        .cloned()
        .map(|stage| build_requirement_stage_artifact(root, preview, stage))
        .collect()
}

fn build_requirement_stage_artifact(
    root: &Path,
    preview: &RequirementPreviewRuntime,
    stage: SpecLoopStageName,
) -> Result<SpecLoopStageArtifact> {
    let requirement_id = preview.requirement_id.as_str();
    let stage_path = requirement_stage_artifact_path(root, requirement_id, &stage)?;
    let stage_ref = normalize_relative_to_root(root, &stage_path)?;
    let runtime_path = requirement_preview_runtime_path(root, requirement_id)?;
    let runtime_ref = normalize_relative_to_root(root, &runtime_path)?;
    let confirmation_ref =
        requirement_stage_artifact_path(root, requirement_id, &SpecLoopStageName::Confirmation)
            .and_then(|path| normalize_relative_to_root(root, path))?;
    let preview_ref =
        requirement_stage_artifact_path(root, requirement_id, &SpecLoopStageName::Preview)
            .and_then(|path| normalize_relative_to_root(root, path))?;
    let intake_ref =
        requirement_stage_artifact_path(root, requirement_id, &SpecLoopStageName::Intake)
            .and_then(|path| normalize_relative_to_root(root, path))?;
    let classification_ref =
        requirement_stage_artifact_path(root, requirement_id, &SpecLoopStageName::Classification)
            .and_then(|path| normalize_relative_to_root(root, path))?;
    let context_ref =
        requirement_stage_artifact_path(root, requirement_id, &SpecLoopStageName::Context)
            .and_then(|path| normalize_relative_to_root(root, path))?;
    let boundary_ref =
        requirement_stage_artifact_path(root, requirement_id, &SpecLoopStageName::Boundary)
            .and_then(|path| normalize_relative_to_root(root, path))?;
    let route_ref =
        requirement_stage_artifact_path(root, requirement_id, &SpecLoopStageName::Route)
            .and_then(|path| normalize_relative_to_root(root, path))?;
    let classification = build_requirement_classification(&preview.intake);
    let classification_summary = format!(
        "分类结果：{}，执行权限 {}。",
        classification.primary_type.as_str(),
        classification.execution_permission.as_str()
    );

    let (status, authority, input_refs, output_refs, evidence_refs, payload, summary) = match stage {
        SpecLoopStageName::Intake => (
            SpecLoopStageStatus::Ready,
            SpecArtifactAuthority::Derived,
            vec![preview.requirement_path.clone()],
            vec![stage_ref.clone(), runtime_ref.clone()],
            vec![preview.requirement_path.clone()],
            None,
            "原始需求已经清洗成 Normalized Requirement，并作为后续阶段的共同输入。".to_string(),
        ),
        SpecLoopStageName::Classification => (
            SpecLoopStageStatus::Ready,
            SpecArtifactAuthority::Derived,
            vec![intake_ref.clone()],
            vec![stage_ref.clone()],
            Vec::new(),
            Some(serde_json::to_value(&classification)?),
            classification_summary,
        ),
        SpecLoopStageName::Context => (
            SpecLoopStageStatus::Declared,
            SpecArtifactAuthority::Derived,
            vec![classification_ref.clone()],
            vec![stage_ref.clone()],
            Vec::new(),
            None,
            "上下文解析阶段文件合同已建立，后续由 AF-SPEC-003 写入上下文结果。".to_string(),
        ),
        SpecLoopStageName::Boundary => (
            SpecLoopStageStatus::Declared,
            SpecArtifactAuthority::Derived,
            vec![context_ref.clone()],
            vec![stage_ref.clone()],
            Vec::new(),
            None,
            "边界判断阶段文件合同已建立，后续由 AF-SPEC-004 写入边界结果。".to_string(),
        ),
        SpecLoopStageName::Route => (
            SpecLoopStageStatus::Declared,
            SpecArtifactAuthority::Derived,
            vec![boundary_ref.clone()],
            vec![stage_ref.clone()],
            Vec::new(),
            None,
            "路由决策阶段文件合同已建立，后续由 AF-SPEC-005 写入 route 决策。".to_string(),
        ),
        SpecLoopStageName::Preview => (
            preview_stage_status(preview),
            SpecArtifactAuthority::Derived,
            vec![route_ref.clone(), runtime_ref.clone()],
            vec![stage_ref.clone(), runtime_ref.clone()],
            Vec::new(),
            None,
            "预览阶段保存当前可读预览产物；它可追踪，但不是 authority。".to_string(),
        ),
        SpecLoopStageName::Confirmation => (
            confirmation_stage_status(preview),
            SpecArtifactAuthority::Derived,
            vec![preview_ref.clone()],
            vec![stage_ref.clone()],
            confirmation_evidence_refs(preview)?,
            None,
            "确认阶段绑定到具体 preview artifact；没有确认记录就不能进入正式物化。".to_string(),
        ),
        SpecLoopStageName::Materialization => (
            materialization_stage_status(preview),
            SpecArtifactAuthority::Authority,
            vec![confirmation_ref],
            materialization_output_refs(root, preview)?,
            materialization_evidence_refs(preview)?,
            None,
            "物化阶段只负责把 preview / confirmation artifact 转成正式 requirement、spec project 和 spec issues。".to_string(),
        ),
    };

    Ok(SpecLoopStageArtifact {
        version: SPEC_STAGE_ARTIFACT_VERSION.to_string(),
        requirement_id: preview.requirement_id.clone(),
        project_id: preview.project_id.clone(),
        stage,
        status,
        authority,
        current_state: Some(preview.current_state.clone()),
        input_refs,
        output_refs,
        evidence_refs,
        payload,
        summary,
        updated_at: preview.updated_at,
    })
}

fn preview_stage_status(preview: &RequirementPreviewRuntime) -> SpecLoopStageStatus {
    match preview.lifecycle {
        RequirementPreviewLifecycle::Cancelled => SpecLoopStageStatus::Cancelled,
        RequirementPreviewLifecycle::Materialized => SpecLoopStageStatus::Ready,
        RequirementPreviewLifecycle::Active => SpecLoopStageStatus::Ready,
    }
}

fn confirmation_stage_status(preview: &RequirementPreviewRuntime) -> SpecLoopStageStatus {
    match preview.lifecycle {
        RequirementPreviewLifecycle::Cancelled => SpecLoopStageStatus::Cancelled,
        RequirementPreviewLifecycle::Materialized => SpecLoopStageStatus::Confirmed,
        RequirementPreviewLifecycle::Active => {
            if preview.current_state == "confirmed" {
                SpecLoopStageStatus::Confirmed
            } else {
                SpecLoopStageStatus::Declared
            }
        }
    }
}

fn materialization_stage_status(preview: &RequirementPreviewRuntime) -> SpecLoopStageStatus {
    match preview.lifecycle {
        RequirementPreviewLifecycle::Cancelled => SpecLoopStageStatus::Cancelled,
        RequirementPreviewLifecycle::Materialized => SpecLoopStageStatus::Materialized,
        RequirementPreviewLifecycle::Active => SpecLoopStageStatus::Declared,
    }
}

fn confirmation_evidence_refs(preview: &RequirementPreviewRuntime) -> Result<Vec<String>> {
    if preview.confirmation_records.is_empty() {
        return Ok(Vec::new());
    }
    let mut refs = vec![format!(
        "{}/DECISIONS.md",
        project_brain_root(&preview.project_id)?
    )];
    if preview.goal_draft.status == GoalDraftStatus::Confirmed {
        refs.push(format!(
            "{}/GOAL.md",
            project_brain_root(&preview.project_id)?
        ));
    }
    if preview
        .plan_draft
        .as_ref()
        .is_some_and(|draft| draft.status == PlanDraftStatus::Confirmed)
    {
        refs.push(format!(
            "{}/PLAN.md",
            project_brain_root(&preview.project_id)?
        ));
    }
    Ok(refs)
}

fn materialization_output_refs(
    root: &Path,
    preview: &RequirementPreviewRuntime,
) -> Result<Vec<String>> {
    let mut refs = vec![
        requirement_preview_runtime_path(root, &preview.requirement_id)
            .and_then(|path| normalize_relative_to_root(root, path))?,
    ];
    if let Some(project_id) = preview.materialized_project_id.as_deref() {
        refs.push(
            spec_project_path(root, project_id)
                .and_then(|path| normalize_relative_to_root(root, path))?,
        );
    }
    for issue_id in &preview.materialized_issue_ids {
        refs.push(
            spec_issue_path(root, issue_id)
                .and_then(|path| normalize_relative_to_root(root, path))?,
        );
    }
    Ok(refs)
}

fn materialization_evidence_refs(preview: &RequirementPreviewRuntime) -> Result<Vec<String>> {
    let mut refs = confirmation_evidence_refs(preview)?;
    refs.push(preview.requirement_path.clone());
    Ok(refs)
}

fn ensure_active_preview(preview: &RequirementPreviewRuntime) -> Result<()> {
    if preview.lifecycle != RequirementPreviewLifecycle::Active {
        anyhow::bail!(
            "requirement preview {} is not active: {}",
            preview.requirement_id,
            preview.lifecycle.as_str()
        );
    }
    Ok(())
}

fn spec_issue_path(root: &Path, issue_id: &str) -> Result<PathBuf> {
    let issue_id = IssueId::parse(issue_id)?;
    join_relative_path(
        root,
        PathBuf::from(".agentflow")
            .join("spec")
            .join("issues")
            .join(format!("{}.json", issue_id.as_str())),
    )
}

fn spec_project_path(root: &Path, project_id: &str) -> Result<PathBuf> {
    let project_id = ProjectId::parse(project_id)?;
    join_relative_path(
        root,
        PathBuf::from(".agentflow")
            .join("spec")
            .join("projects")
            .join(format!("{}.json", project_id.as_str())),
    )
}

fn requirement_preview_dir(root: &Path, requirement_id: &str) -> Result<PathBuf> {
    validate_safe_local_id("requirementId", requirement_id)?;
    join_relative_path(
        root,
        PathBuf::from(".agentflow")
            .join("spec")
            .join("requirements")
            .join(requirement_id),
    )
}

fn requirement_preview_runtime_path(root: &Path, requirement_id: &str) -> Result<PathBuf> {
    Ok(requirement_preview_dir(root, requirement_id)?.join("runtime.json"))
}

fn requirement_manifest_path(root: &Path, requirement_id: &str) -> Result<PathBuf> {
    Ok(requirement_preview_dir(root, requirement_id)?.join("manifest.json"))
}

fn requirement_stage_artifact_path(
    root: &Path,
    requirement_id: &str,
    stage: &SpecLoopStageName,
) -> Result<PathBuf> {
    Ok(requirement_preview_dir(root, requirement_id)?.join(stage.file_name()))
}

fn legacy_requirement_preview_path(root: &Path, requirement_id: &str) -> Result<PathBuf> {
    validate_safe_local_id("requirementId", requirement_id)?;
    join_relative_path(
        root,
        PathBuf::from(".agentflow")
            .join("spec")
            .join("requirements")
            .join(format!("{requirement_id}.json")),
    )
}

fn completion_decision_path(root: &Path, project_id: &str) -> Result<PathBuf> {
    let project_id = ProjectId::parse(project_id)?;
    join_relative_path(
        root,
        PathBuf::from(".agentflow")
            .join("spec")
            .join("completions")
            .join(format!("{}.json", project_id.as_str())),
    )
}

fn default_preview_project_id(requirement_id: &str) -> String {
    let slug = requirement_id
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>();
    format!("project-{}", slug.trim_matches('-'))
}

fn default_issue_prefix(project_id: &str) -> String {
    project_id
        .chars()
        .map(|ch| match ch {
            'a'..='z' => ch.to_ascii_uppercase(),
            'A'..='Z' | '0'..='9' | '-' => ch,
            _ => '-',
        })
        .collect()
}

fn build_completion_facts(
    root: &Path,
    project: &SpecProject,
    issues: &[SpecIssue],
) -> CompletionDecisionFacts {
    let total_issue_count = issues.len();
    let completed_issue_count = issues
        .iter()
        .filter(|issue| issue.status == SpecIssueStatus::Done)
        .count();
    let canceled_issue_count = issues
        .iter()
        .filter(|issue| issue.status == SpecIssueStatus::Cancel)
        .count();
    let blocked_issue_count = issues
        .iter()
        .filter(|issue| issue.status == SpecIssueStatus::Blocked)
        .count();
    let remaining_issue_count =
        total_issue_count.saturating_sub(completed_issue_count + canceled_issue_count);
    let task_evidence_ready_count = issues
        .iter()
        .filter(|issue| issue.status == SpecIssueStatus::Done)
        .filter(|issue| {
            load_task_evidence(root, &issue.issue_id)
                .map(|evidence| matches!(evidence.status.as_str(), "ready" | "passed"))
                .unwrap_or(false)
        })
        .count();
    let task_evidence_missing_count =
        completed_issue_count.saturating_sub(task_evidence_ready_count);
    let delivery = build_completion_delivery_facts(root, issues);
    let audit = load_project_audit_review_summary(root, &project.project_id, &project.issue_ids)
        .ok()
        .flatten();
    let audit_required = audit
        .as_ref()
        .is_some_and(|summary| summary.total_count > 0);
    let audit_status = audit
        .as_ref()
        .and_then(|summary| summary.latest_status.clone())
        .unwrap_or_else(|| "not-requested".to_string());
    let audit_blocking_findings = if audit_status == "failed" {
        audit
            .as_ref()
            .map(|summary| summary.findings_count)
            .unwrap_or(0)
    } else {
        0
    };
    let brain_snapshot =
        read_project_brain_snapshot(root, &project.project_id, &project.title).ok();
    let project_health_status = brain_snapshot
        .as_ref()
        .map(|snapshot| snapshot.health_status.as_str().to_string())
        .unwrap_or_else(|| "missing".to_string());
    let goal_recheck_status = build_goal_recheck_status(
        remaining_issue_count,
        brain_snapshot
            .as_ref()
            .map(|snapshot| &snapshot.health_status),
    );
    CompletionDecisionFacts {
        total_issue_count,
        completed_issue_count,
        canceled_issue_count,
        remaining_issue_count,
        blocked_issue_count,
        task_evidence_ready_count,
        task_evidence_missing_count,
        delivery_status: delivery.status.clone(),
        delivery_missing_count: delivery.missing_count,
        audit_required,
        audit_status: audit_status.clone(),
        audit_blocking_findings,
        goal_recheck_status: goal_recheck_status.clone(),
        project_health_status: project_health_status.clone(),
        release_readiness: build_release_readiness(
            remaining_issue_count,
            task_evidence_missing_count,
            delivery.missing_count,
            audit_required,
            &audit_status,
            &goal_recheck_status,
        ),
    }
}

fn build_completion_delivery_facts(root: &Path, issues: &[SpecIssue]) -> CompletionDeliveryFacts {
    let mut ready_count = 0usize;
    let mut published_count = 0usize;
    let mut missing_count = 0usize;

    for issue in issues
        .iter()
        .filter(|issue| issue.status == SpecIssueStatus::Done)
    {
        let evidence = load_task_evidence(root, &issue.issue_id).ok();
        let Some(evidence) = evidence else {
            missing_count += 1;
            continue;
        };
        let closeout = load_closeout_proof_for_issue(root, &issue.issue_id, &evidence.run_id).ok();
        let Some(closeout) = closeout else {
            missing_count += 1;
            continue;
        };
        let has_review_proof = closeout.merged
            && closeout.issue_closed
            && (has_value(closeout.pr_url.as_deref())
                || has_value(closeout.merge_commit_sha.as_deref()));
        if !has_review_proof {
            missing_count += 1;
            continue;
        }
        let has_public_record = closeout.public_delivery_written
            || has_value(closeout.changelog_path.as_deref())
            || has_value(closeout.release_notes_path.as_deref());
        if has_public_record {
            published_count += 1;
        } else {
            ready_count += 1;
        }
    }

    let status = if missing_count == 0 && published_count > 0 && ready_count == 0 {
        "published".to_string()
    } else if published_count > 0 || ready_count > 0 {
        "ready".to_string()
    } else {
        "missing".to_string()
    };

    CompletionDeliveryFacts {
        status,
        missing_count,
    }
}

fn load_closeout_proof_for_issue(
    root: &Path,
    issue_id: &str,
    run_id: &str,
) -> Result<CloseoutProofSnapshot> {
    validate_safe_local_id("runId", run_id)?;
    let path = root
        .join(".agentflow")
        .join("tasks")
        .join(IssueId::parse(issue_id)?.as_str())
        .join("runs")
        .join(run_id)
        .join("review")
        .join("closeout-proof.json");
    read_json(&path)
}

fn has_value(value: Option<&str>) -> bool {
    value.is_some_and(|value| !value.trim().is_empty())
}

fn build_goal_recheck_status(
    remaining_issue_count: usize,
    health_status: Option<&ProjectBrainDocumentStatus>,
) -> String {
    if remaining_issue_count > 0 {
        return "not-ready".to_string();
    }
    match health_status.unwrap_or(&ProjectBrainDocumentStatus::Missing) {
        ProjectBrainDocumentStatus::Blocked => "blocked".to_string(),
        ProjectBrainDocumentStatus::Draft
        | ProjectBrainDocumentStatus::NeedsConfirmation
        | ProjectBrainDocumentStatus::Stale => "needs-recheck".to_string(),
        ProjectBrainDocumentStatus::Confirmed | ProjectBrainDocumentStatus::Missing => {
            "ready".to_string()
        }
    }
}

fn build_release_readiness(
    remaining_issue_count: usize,
    task_evidence_missing_count: usize,
    delivery_missing_count: usize,
    audit_required: bool,
    audit_status: &str,
    goal_recheck_status: &str,
) -> String {
    if remaining_issue_count > 0 {
        return "blocked-remaining-issues".to_string();
    }
    if task_evidence_missing_count > 0 {
        return "blocked-missing-evidence".to_string();
    }
    if delivery_missing_count > 0 {
        return "blocked-missing-delivery".to_string();
    }
    if audit_required && !audit_status_allows_accept(audit_status) {
        return "blocked-audit".to_string();
    }
    if goal_recheck_status != "ready" {
        return "blocked-goal-recheck".to_string();
    }
    "ready".to_string()
}

fn audit_status_allows_accept(status: &str) -> bool {
    matches!(status, "passed" | "passed-with-warnings" | "not-requested")
}

fn completion_accept_blockers(facts: &CompletionDecisionFacts) -> Vec<String> {
    let mut blockers = Vec::new();
    if facts.remaining_issue_count > 0 {
        blockers.push(format!(
            "当前还有 {} 条任务未完成。",
            facts.remaining_issue_count
        ));
    }
    if facts.task_evidence_missing_count > 0 {
        blockers.push(format!(
            "还有 {} 条任务缺少验证证据。",
            facts.task_evidence_missing_count
        ));
    }
    if facts.delivery_missing_count > 0 {
        blockers.push(format!(
            "还有 {} 条任务缺少交付收口事实。",
            facts.delivery_missing_count
        ));
    }
    if facts.audit_required && !audit_status_allows_accept(&facts.audit_status) {
        blockers.push(format!("审计当前处于 {}，还不能完成。", facts.audit_status));
    }
    if facts.goal_recheck_status != "ready" {
        blockers.push(format!(
            "Goal Recheck 当前状态是 {}，还不能接受项目完成。",
            facts.goal_recheck_status
        ));
    }
    blockers
}

fn sync_completion_runtime_for_project(
    project: &SpecProject,
    facts: CompletionDecisionFacts,
    existing: Option<CompletionDecisionRuntime>,
    all_finished: bool,
) -> Result<CompletionDecisionRuntime> {
    let now = unix_timestamp_seconds();
    let accept_blockers = completion_accept_blockers(&facts);
    let mut runtime = existing.unwrap_or_else(|| CompletionDecisionRuntime {
        version: COMPLETION_DECISION_VERSION.to_string(),
        project_id: project.project_id.clone(),
        project_title: project.title.clone(),
        source_requirement_id: project.source_requirement_id.clone(),
        current_state: CompletionDecisionState::GoalRecheck,
        latest_outcome: None,
        facts: facts.clone(),
        open_questions: completion_open_questions_for_state(&CompletionDecisionState::GoalRecheck),
        rationale: vec![
            "当前项目所有任务已经完成，但项目是否真正结束必须回到 Goal Agent 再判断。".to_string(),
        ],
        history: Vec::new(),
        next_recommended_action: "enter-completion-decision".to_string(),
        next_recommended_action_label: "进入完成判断".to_string(),
        next_recommended_action_reason:
            "先回到 Goal Recheck，再决定继续、调整、暂停、接受或进入下一阶段。".to_string(),
        readonly: false,
        updated_at: now,
    });

    runtime.project_title = project.title.clone();
    runtime.source_requirement_id = project.source_requirement_id.clone();
    runtime.facts = facts;

    if all_finished {
        if runtime.latest_outcome.is_none() {
            runtime.current_state = CompletionDecisionState::GoalRecheck;
            runtime.open_questions =
                completion_open_questions_for_state(&CompletionDecisionState::GoalRecheck);
            runtime.rationale = if accept_blockers.is_empty() {
                vec![
                    "任务执行已经收口，证据、交付、审计和目标满足度都已齐备。".to_string(),
                    "Project 完成必须由 Goal Agent 显式给出 completion decision。".to_string(),
                ]
            } else {
                let mut rationale = vec![
                    "任务执行已经收口，但完成门还没有全部满足。".to_string(),
                    "先补齐完成前置条件，再进入 Accept。".to_string(),
                ];
                rationale.extend(accept_blockers.clone());
                rationale
            };
            let (action, label, reason) = completion_next_action_bundle(
                &runtime.current_state,
                runtime.latest_outcome.as_ref(),
                &runtime.facts,
            );
            runtime.next_recommended_action = action;
            runtime.next_recommended_action_label = label;
            runtime.next_recommended_action_reason = reason;
        }
    } else {
        runtime.current_state = CompletionDecisionState::Continue;
        runtime.latest_outcome = runtime
            .latest_outcome
            .take()
            .filter(|outcome| *outcome != CompletionDecisionOutcome::Accept);
        runtime.open_questions = vec!["当前还有未完成任务，先继续推进项目执行。".to_string()];
        runtime.rationale = vec![format!(
            "当前还有 {} 条任务未完成，Completion Decision 暂时不能收口项目。",
            runtime.facts.remaining_issue_count
        )];
        let (action, label, reason) = completion_next_action_bundle(
            &runtime.current_state,
            runtime.latest_outcome.as_ref(),
            &runtime.facts,
        );
        runtime.next_recommended_action = action;
        runtime.next_recommended_action_label = label;
        runtime.next_recommended_action_reason = reason;
    }

    runtime.updated_at = now;
    Ok(runtime)
}

fn completion_state_for_outcome(outcome: &CompletionDecisionOutcome) -> CompletionDecisionState {
    match outcome {
        CompletionDecisionOutcome::Continue => CompletionDecisionState::Continue,
        CompletionDecisionOutcome::Adjust => CompletionDecisionState::Adjust,
        CompletionDecisionOutcome::Pause => CompletionDecisionState::Pause,
        CompletionDecisionOutcome::Accept => CompletionDecisionState::Accepted,
        CompletionDecisionOutcome::NextStage => CompletionDecisionState::NextStage,
    }
}

fn completion_open_questions_for_state(state: &CompletionDecisionState) -> Vec<String> {
    match state {
        CompletionDecisionState::GoalRecheck => vec![
            "当前交付是否真正满足 GOAL.md 和 PLAN.md？".to_string(),
            "项目应该接受、继续、调整、暂停，还是进入下一阶段？".to_string(),
        ],
        CompletionDecisionState::Continue => Vec::new(),
        CompletionDecisionState::Adjust => vec!["下一轮需要调整哪些目标、范围或计划？".to_string()],
        CompletionDecisionState::Pause => vec!["项目暂停后，恢复条件是什么？".to_string()],
        CompletionDecisionState::Accepted => Vec::new(),
        CompletionDecisionState::NextStage => {
            vec!["下一阶段要生成新的 Goal / Plan 还是延续当前上下文？".to_string()]
        }
    }
}

fn completion_next_action_bundle(
    state: &CompletionDecisionState,
    outcome: Option<&CompletionDecisionOutcome>,
    facts: &CompletionDecisionFacts,
) -> (String, String, String) {
    match state {
        CompletionDecisionState::GoalRecheck => {
            if completion_accept_blockers(facts).is_empty() {
                (
                    "enter-completion-decision".to_string(),
                    "进入完成判断".to_string(),
                    "当前任务已经收口，下一步由 Goal Agent 明确给出项目完成决策。".to_string(),
                )
            } else {
                (
                    "resolve-completion-blockers".to_string(),
                    "补齐完成前置条件".to_string(),
                    "当前还有证据、交付、审计或目标满足度缺口，先补齐再做 completion accept。"
                        .to_string(),
                )
            }
        }
        CompletionDecisionState::Continue => (
            "start-project-loop".to_string(),
            "继续项目循环".to_string(),
            format!(
                "当前还有 {} 条任务未完成，先继续推进任务循环。",
                facts.remaining_issue_count
            ),
        ),
        CompletionDecisionState::Adjust => (
            "run-goal-recheck".to_string(),
            "重新检查目标与计划".to_string(),
            "Goal Agent 要求先调整 Goal / Plan，再继续项目循环。".to_string(),
        ),
        CompletionDecisionState::Pause => (
            "pause-project".to_string(),
            "暂停项目".to_string(),
            "Goal Agent 已暂停项目，等待后续人工决定。".to_string(),
        ),
        CompletionDecisionState::Accepted => (
            "project-accepted".to_string(),
            "项目已接受".to_string(),
            "Goal Agent 已接受当前交付，项目可以视为完成。".to_string(),
        ),
        CompletionDecisionState::NextStage => (
            "start-next-stage".to_string(),
            "进入下一阶段".to_string(),
            match outcome {
                Some(CompletionDecisionOutcome::NextStage) => {
                    "当前目标已完成，下一步进入下一阶段 Goal / Plan。".to_string()
                }
                _ => "Goal Agent 建议进入下一阶段。".to_string(),
            },
        ),
    }
}

fn emit_completion_acceptance(
    root: &Path,
    runtime: &CompletionDecisionRuntime,
    actor: &str,
) -> Result<()> {
    let workflow = agentflow_workflow_core::canonical_workflow(
        agentflow_workflow_core::WorkflowFlowType::Project,
    );
    let mut context = agentflow_workflow_runtime::RuntimeContext::project(
        runtime.project_id.clone(),
        agentflow_event_store::EventActor {
            role: actor.to_string(),
            kind: "agent".to_string(),
        },
    );
    context.correlation_id = Some(format!("completion-{}", runtime.project_id));
    context.payload = serde_json::json!({
        "completionDecisionRef": format!(".agentflow/spec/completions/{}.json", runtime.project_id),
        "outcome": "accept",
    });
    let guards = agentflow_workflow_runtime::StaticGuardRegistry::all_pass(Vec::<String>::new());
    let actions =
        agentflow_workflow_runtime::StaticActionRegistry::all_complete(["project.accept.write"]);
    let result = agentflow_workflow_runtime::apply_workflow_event(
        root,
        &workflow,
        "goal_recheck",
        "project.accepted",
        context,
        &guards,
        &actions,
    )?;
    if !result.applied {
        anyhow::bail!(
            "project acceptance transition for {} was blocked: {}",
            runtime.project_id,
            result
                .blocked_reason
                .unwrap_or_else(|| "unknown reason".to_string())
        );
    }
    Ok(())
}

fn emit_completion_recheck_event(
    root: &Path,
    runtime: &CompletionDecisionRuntime,
    actor: &str,
    previous_state: &CompletionDecisionState,
    summary: &str,
) -> Result<()> {
    let event_type = match runtime.latest_outcome.as_ref() {
        Some(CompletionDecisionOutcome::Continue) => "project.goal_recheck.continue",
        Some(CompletionDecisionOutcome::Adjust) => "project.goal_recheck.adjust",
        Some(CompletionDecisionOutcome::Pause) => "project.goal_recheck.pause",
        Some(CompletionDecisionOutcome::NextStage) => "project.goal_recheck.next_stage",
        Some(CompletionDecisionOutcome::Accept) | None => return Ok(()),
    };

    agentflow_event_store::append_task_event(
        root,
        agentflow_event_store::TaskEventDraft {
            flow_type: agentflow_workflow_core::WorkflowFlowType::Project,
            aggregate_type: "project".to_string(),
            aggregate_id: runtime.project_id.clone(),
            project_id: Some(runtime.project_id.clone()),
            issue_id: None,
            run_id: None,
            event_type: event_type.to_string(),
            authority_role: Some(agentflow_workflow_core::WorkflowAgentRole::GoalAgent),
            actor: agentflow_event_store::EventActor {
                role: actor.to_string(),
                kind: "agent".to_string(),
            },
            state: Some(agentflow_event_store::EventStateTransition {
                from_state: previous_state.as_str().to_string(),
                to_state: runtime.current_state.as_str().to_string(),
            }),
            correlation_id: Some(format!("completion-{}", runtime.project_id)),
            causation_id: None,
            payload: serde_json::json!({
                "completionDecisionRef": format!(".agentflow/spec/completions/{}.json", runtime.project_id),
                "outcome": runtime.latest_outcome.as_ref().map(|outcome| outcome.as_str()),
                "summary": summary,
                "nextAction": runtime.next_recommended_action.as_str(),
                "nextActionLabel": runtime.next_recommended_action_label.as_str(),
                "nextActionReason": runtime.next_recommended_action_reason.as_str(),
                "facts": &runtime.facts,
                "openQuestions": &runtime.open_questions,
                "rationale": &runtime.rationale,
            }),
            artifact_refs: vec![format!(
                ".agentflow/spec/completions/{}.json",
                runtime.project_id
            )],
            idempotency_key: None,
        },
    )?;
    Ok(())
}

fn detect_requirement_intent(requirement: &RequirementDocument) -> RequirementIntentType {
    let raw = format!("{} {}", requirement.title, requirement.summary).to_ascii_lowercase();
    if raw.contains("audit") || raw.contains("审计") {
        RequirementIntentType::Audit
    } else if raw.contains("repair")
        || raw.contains("bug")
        || raw.contains("修复")
        || raw.contains("fix")
    {
        RequirementIntentType::Repair
    } else if raw.contains("design") || raw.contains("设计") {
        RequirementIntentType::Design
    } else if raw.contains("understand") || raw.contains("理解") {
        RequirementIntentType::Understanding
    } else if raw.contains("technical")
        || raw.contains("runtime")
        || raw.contains("workflow")
        || raw.contains("技术")
    {
        RequirementIntentType::Technical
    } else if raw.contains("product") || raw.contains("产品") {
        RequirementIntentType::Product
    } else {
        RequirementIntentType::Mixed
    }
}

fn default_deliverables(intent: &RequirementIntentType) -> Vec<String> {
    match intent {
        RequirementIntentType::Audit => vec!["审计结论".to_string(), "审计建议".to_string()],
        RequirementIntentType::Design => vec!["设计方案".to_string(), "交互说明".to_string()],
        RequirementIntentType::Understanding => {
            vec!["项目说明".to_string(), "结构化结论".to_string()]
        }
        _ => vec!["项目计划".to_string(), "结构化任务合同".to_string()],
    }
}

fn markdown_list(items: &[String]) -> String {
    if items.is_empty() {
        "- 无\n".to_string()
    } else {
        items
            .iter()
            .map(|item| format!("- {item}\n"))
            .collect::<String>()
    }
}

fn write_text(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        ensure_directory(parent)?;
    }
    fs::write(path, content).with_context(|| format!("write {}", path.display()))
}

fn validate_issue_contract(issue: &SpecIssue) -> Result<()> {
    let issue_id = IssueId::parse(&issue.issue_id)?;
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
    let task_run_dir = normalize_relative_path_string(&issue.expected_outputs.task_run_dir)?;
    if !task_run_dir.starts_with(&format!(".agentflow/tasks/{}/runs/", issue_id.as_str())) {
        anyhow::bail!("issue {} has invalid taskRunDir", issue.issue_id);
    }
    let evidence_path = normalize_relative_path_string(&issue.expected_outputs.evidence_path)?;
    if !evidence_path.starts_with(&format!(".agentflow/tasks/{}/evidence/", issue_id.as_str())) {
        anyhow::bail!("issue {} has invalid evidencePath", issue.issue_id);
    }
    let system_path = normalize_relative_path_string(&issue.system.path)?;
    if system_path != format!(".agentflow/spec/issues/{}.json", issue_id.as_str()) {
        anyhow::bail!("issue {} has invalid system path", issue.issue_id);
    }
    Ok(())
}

fn validate_project_contract(project: &SpecProject) -> Result<()> {
    let project_id = ProjectId::parse(&project.project_id)?;
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
    let system_path = normalize_relative_path_string(&project.system.path)?;
    if system_path != format!(".agentflow/spec/projects/{}.json", project_id.as_str()) {
        anyhow::bail!("project {} has invalid system path", project.project_id);
    }
    Ok(())
}

#[derive(Debug, Clone, Default)]
struct DocumentProbe {
    exists: bool,
    updated_at: Option<u64>,
    raw: Option<String>,
}

fn project_brain_root(project_id: &str) -> Result<String> {
    let project_id = ProjectId::parse(project_id)?;
    Ok(format!("docs/projects/{}", project_id.as_str()))
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
    use crate::model::{
        RequirementClass, RequirementClassificationResult, RequirementExecutionPermission,
        RequirementRiskLevel, SpecArtifactAuthority, SpecLoopRequirementManifest,
        SpecLoopStageArtifact, SpecLoopStageName, SpecLoopStageStatus, SpecPriority,
        DEFAULT_WORKFLOW_REF,
    };
    use agentflow_event_store::load_task_events;
    use serde_json::Value;
    use tempfile::tempdir;

    fn write_requirement(root: &Path) -> PathBuf {
        write_requirement_with_text(
            root,
            "999-task-workflow-test",
            "# 任务工作流测试\n\n把任务运行状态改成事件驱动。\n",
        )
    }

    fn write_requirement_with_text(root: &Path, file_stem: &str, content: &str) -> PathBuf {
        let path = root.join(format!("docs/requirements/{file_stem}.md"));
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, content).unwrap();
        path
    }

    fn write_completion_ready_artifacts(
        root: &Path,
        issue_id: &str,
        run_id: &str,
        public_delivery_written: bool,
    ) {
        let issue_root = root.join(".agentflow/tasks").join(issue_id);
        let evidence_dir = issue_root.join("evidence");
        let review_dir = issue_root.join("runs").join(run_id).join("review");
        fs::create_dir_all(&evidence_dir).unwrap();
        fs::create_dir_all(&review_dir).unwrap();
        write_json(
            &evidence_dir.join("evidence.json"),
            &serde_json::json!({
                "version": "task-evidence.v1",
                "issueId": issue_id,
                "runId": run_id,
                "status": "ready",
                "summary": "验证通过",
                "runPath": format!(".agentflow/tasks/{issue_id}/runs/{run_id}/run.json"),
                "commandPaths": [],
                "validationPath": format!(".agentflow/tasks/{issue_id}/runs/{run_id}/validation.json"),
                "createdAt": 1
            }),
        )
        .unwrap();
        write_json(
            &review_dir.join("closeout-proof.json"),
            &serde_json::json!({
                "merged": true,
                "issueClosed": true,
                "publicDeliveryWritten": public_delivery_written,
                "prUrl": "https://github.com/acme/repo/pull/1",
                "mergeCommitSha": "merge-001",
                "changelogPath": if public_delivery_written { Some("CHANGELOG.md") } else { None::<&str> },
                "releaseNotesPath": if public_delivery_written { Some("docs/release-notes/test.md") } else { None::<&str> }
            }),
        )
        .unwrap();
    }

    fn write_project_health(root: &Path, project_id: &str, marker: &str) {
        let path = root
            .join("docs/projects")
            .join(project_id)
            .join("PROJECT_HEALTH.md");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, format!("# PROJECT_HEALTH\n\n{marker}\n")).unwrap();
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

    #[test]
    fn rejects_path_like_project_ids() {
        let dir = tempdir().unwrap();
        let requirement = write_requirement(dir.path());
        let err = project_from_requirement(
            dir.path(),
            &requirement,
            SpecProjectDraft::new("../project-bad"),
        )
        .unwrap_err()
        .to_string();
        assert!(err.contains("safe local id"));
    }

    #[test]
    fn rejects_issue_contract_with_escape_output_paths() {
        let dir = tempdir().unwrap();
        let requirement = write_requirement(dir.path());
        let mut issue =
            issue_from_requirement(dir.path(), &requirement, SpecIssueDraft::new("AF-SPEC-099"))
                .unwrap();
        issue.expected_outputs.task_run_dir =
            ".agentflow/tasks/AF-SPEC-099/runs/../../escape".to_string();
        let err = write_spec_issue(dir.path(), &issue)
            .unwrap_err()
            .to_string();
        assert!(err.contains("parent traversal"));
    }

    #[test]
    fn requirement_preview_runtime_starts_from_goal_preview_and_writes_project_event() {
        let dir = tempdir().unwrap();
        let requirement = write_requirement(dir.path());

        let preview =
            requirement_preview_from_requirement(dir.path(), &requirement, Some("project-preview"))
                .unwrap();

        assert_eq!(preview.current_state, "goal_draft");
        assert_eq!(preview.lifecycle, RequirementPreviewLifecycle::Active);
        assert_eq!(
            preview.goal_draft.status,
            GoalDraftStatus::NeedsClarification
        );
        assert!(!preview.goal_draft.open_questions.is_empty());
        assert_eq!(
            preview.next_recommended_action,
            "confirm-goal-draft-preview"
        );

        let events = load_task_events(dir.path()).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "project.intake.accepted");
        assert_eq!(events[0].project_id.as_deref(), Some("project-preview"));
    }

    #[test]
    fn requirement_preview_runtime_writes_stage_contracts_and_normalized_intake() {
        let dir = tempdir().unwrap();
        let requirement = write_requirement_with_text(
            dir.path(),
            "340-spec-loop-filesystem-contract",
            "# 发布前需求整理\n\n请阅读 docs/requirements/source.md 与 crates/spec/src/lib.rs，参考 https://example.com/spec 。\n在 main 分支上，处理 issue #34 和 PR #12，针对 v0.5.0 release 做规划、确认和发布。\n",
        );

        let preview =
            requirement_preview_from_requirement(dir.path(), &requirement, Some("project-preview"))
                .unwrap();

        assert_eq!(preview.intake.agent_locale, "zh-CN");
        assert_eq!(
            preview.intake.raw_text,
            fs::read_to_string(&requirement).unwrap().trim()
        );
        assert!(preview
            .intake
            .referenced_files
            .contains(&"docs/requirements/source.md".to_string()));
        assert!(
            preview
                .intake
                .referenced_files
                .iter()
                .any(|value| value.contains("crates/spec/src/lib.rs")),
            "{:?}",
            preview.intake.referenced_files
        );
        assert_eq!(
            preview.intake.referenced_urls,
            vec!["https://example.com/spec".to_string()]
        );
        assert!(preview
            .intake
            .referenced_versions
            .contains(&"v0.5.0".to_string()));
        assert!(preview
            .intake
            .referenced_releases
            .contains(&"release".to_string()));
        assert!(preview
            .intake
            .referenced_branches
            .contains(&"main".to_string()));
        assert!(preview
            .intake
            .referenced_issues
            .contains(&"#34".to_string()));
        assert!(preview
            .intake
            .referenced_pull_requests
            .contains(&"#12".to_string()));
        assert!(preview
            .intake
            .explicit_actions
            .contains(&"规划".to_string()));
        assert!(preview
            .intake
            .explicit_actions
            .contains(&"确认".to_string()));
        assert!(preview
            .intake
            .explicit_actions
            .contains(&"发布".to_string()));
        assert!(preview
            .intake
            .input_sources
            .contains(&"requirement-document".to_string()));
        assert!(preview
            .intake
            .input_sources
            .contains(&"file-reference".to_string()));
        assert!(preview
            .intake
            .input_sources
            .contains(&"url-reference".to_string()));
        assert!(preview
            .intake
            .input_sources
            .contains(&"issue-reference".to_string()));
        assert!(preview
            .intake
            .input_sources
            .contains(&"pull-request-reference".to_string()));
        assert!(preview
            .intake
            .input_sources
            .contains(&"release-reference".to_string()));

        let requirement_dir = dir
            .path()
            .join(".agentflow/spec/requirements/340-spec-loop-filesystem-contract");
        assert!(requirement_dir.join("runtime.json").is_file());
        assert!(requirement_dir.join("manifest.json").is_file());
        for stage in SpecLoopStageName::all() {
            assert!(requirement_dir.join(stage.file_name()).is_file());
        }

        let manifest: SpecLoopRequirementManifest =
            read_json(&requirement_dir.join("manifest.json")).unwrap();
        assert_eq!(manifest.version, "agentflow-spec-requirement-manifest.v1");
        assert_eq!(manifest.requirement_id, "340-spec-loop-filesystem-contract");
        assert_eq!(manifest.stage_files.len(), 8);

        let intake_artifact: SpecLoopStageArtifact =
            read_json(&requirement_dir.join("intake.json")).unwrap();
        assert_eq!(intake_artifact.stage, SpecLoopStageName::Intake);
        assert_eq!(intake_artifact.status, SpecLoopStageStatus::Ready);
        assert_eq!(intake_artifact.authority, SpecArtifactAuthority::Derived);
        assert!(intake_artifact
            .input_refs
            .contains(&"docs/requirements/340-spec-loop-filesystem-contract.md".to_string()));

        let classification_artifact: SpecLoopStageArtifact =
            read_json(&requirement_dir.join("classification.json")).unwrap();
        assert_eq!(
            classification_artifact.stage,
            SpecLoopStageName::Classification
        );
        assert_eq!(classification_artifact.status, SpecLoopStageStatus::Ready);
        let classification: RequirementClassificationResult =
            serde_json::from_value(classification_artifact.payload.unwrap()).unwrap();
        assert_eq!(classification.primary_type, RequirementClass::Release);
        assert_eq!(
            classification.execution_permission,
            RequirementExecutionPermission::ReleaseCloseout
        );
        assert_eq!(classification.risk_level, RequirementRiskLevel::High);
    }

    #[test]
    fn classification_marks_question_as_answer_only() {
        let dir = tempdir().unwrap();
        let requirement = write_requirement_with_text(
            dir.path(),
            "question-only",
            "# 只是提问\n\n为什么当前 release-gate 会失败？\n",
        );

        requirement_preview_from_requirement(dir.path(), &requirement, Some("project-preview"))
            .unwrap();

        let classification_artifact: SpecLoopStageArtifact = read_json(
            &dir.path()
                .join(".agentflow/spec/requirements/question-only/classification.json"),
        )
        .unwrap();
        let classification: RequirementClassificationResult =
            serde_json::from_value(classification_artifact.payload.unwrap()).unwrap();

        assert_eq!(classification.primary_type, RequirementClass::Question);
        assert_eq!(
            classification.execution_permission,
            RequirementExecutionPermission::AnswerOnly
        );
        assert!(!classification.confirmation_required);
        assert!(!classification.conflicting);
    }

    #[test]
    fn classification_keeps_audit_out_of_build_loop() {
        let dir = tempdir().unwrap();
        let requirement = write_requirement_with_text(
            dir.path(),
            "audit-only",
            "# 发布审计\n\n请对 v0.5.0 release 做审计并输出 findings。\n",
        );

        requirement_preview_from_requirement(dir.path(), &requirement, Some("project-preview"))
            .unwrap();

        let classification_artifact: SpecLoopStageArtifact = read_json(
            &dir.path()
                .join(".agentflow/spec/requirements/audit-only/classification.json"),
        )
        .unwrap();
        let classification: RequirementClassificationResult =
            serde_json::from_value(classification_artifact.payload.unwrap()).unwrap();

        assert!(classification
            .basic_types
            .contains(&RequirementClass::Audit));
        assert_eq!(
            classification.execution_permission,
            RequirementExecutionPermission::AuditLoop
        );
        assert!(classification
            .reasons
            .iter()
            .any(|reason| reason.contains("审计")));
    }

    #[test]
    fn classification_keeps_design_only_non_executable() {
        let dir = tempdir().unwrap();
        let requirement = write_requirement_with_text(
            dir.path(),
            "design-only",
            "# 设计稿调整\n\n请基于 Figma 调整页面 UI 视觉和交互，只输出设计说明。\n",
        );

        requirement_preview_from_requirement(dir.path(), &requirement, Some("project-preview"))
            .unwrap();

        let classification_artifact: SpecLoopStageArtifact = read_json(
            &dir.path()
                .join(".agentflow/spec/requirements/design-only/classification.json"),
        )
        .unwrap();
        let classification: RequirementClassificationResult =
            serde_json::from_value(classification_artifact.payload.unwrap()).unwrap();

        assert!(classification
            .basic_types
            .contains(&RequirementClass::DesignOnly));
        assert_eq!(
            classification.execution_permission,
            RequirementExecutionPermission::PreviewOnly
        );
        assert!(!classification
            .basic_types
            .contains(&RequirementClass::ExecutableIssue));
    }

    #[test]
    fn confirming_goal_and_plan_writes_project_brain_documents() {
        let dir = tempdir().unwrap();
        let requirement = write_requirement(dir.path());

        requirement_preview_from_requirement(dir.path(), &requirement, Some("project-preview"))
            .unwrap();
        let goal_confirmed =
            confirm_goal_draft_preview(dir.path(), "999-task-workflow-test", "goal-agent").unwrap();
        assert_eq!(goal_confirmed.current_state, "plan_draft");
        assert_eq!(goal_confirmed.goal_draft.status, GoalDraftStatus::Confirmed);
        assert!(goal_confirmed.plan_draft.is_some());
        assert!(dir
            .path()
            .join("docs/projects/project-preview/GOAL.md")
            .is_file());

        let plan_confirmed =
            confirm_plan_draft_preview(dir.path(), "999-task-workflow-test", "spec-agent").unwrap();
        assert_eq!(plan_confirmed.current_state, "confirmed");
        assert_eq!(
            plan_confirmed.plan_draft.as_ref().unwrap().status,
            PlanDraftStatus::Confirmed
        );
        assert!(dir
            .path()
            .join("docs/projects/project-preview/PLAN.md")
            .is_file());
        let decisions = fs::read_to_string(
            dir.path()
                .join("docs/projects/project-preview/DECISIONS.md"),
        )
        .unwrap();
        assert!(decisions.contains("goal-draft"));
        assert!(decisions.contains("plan-draft"));
    }

    #[test]
    fn confirmation_and_materialization_stage_files_follow_preview_state() {
        let dir = tempdir().unwrap();
        let requirement = write_requirement(dir.path());

        requirement_preview_from_requirement(dir.path(), &requirement, Some("project-preview"))
            .unwrap();
        let requirement_dir = dir
            .path()
            .join(".agentflow/spec/requirements/999-task-workflow-test");

        let confirmation_before: SpecLoopStageArtifact =
            read_json(&requirement_dir.join("confirmation.json")).unwrap();
        let materialization_before: SpecLoopStageArtifact =
            read_json(&requirement_dir.join("materialization.json")).unwrap();
        assert_eq!(confirmation_before.status, SpecLoopStageStatus::Declared);
        assert_eq!(materialization_before.status, SpecLoopStageStatus::Declared);

        confirm_goal_draft_preview(dir.path(), "999-task-workflow-test", "goal-agent").unwrap();
        confirm_plan_draft_preview(dir.path(), "999-task-workflow-test", "spec-agent").unwrap();

        let confirmation_after: SpecLoopStageArtifact =
            read_json(&requirement_dir.join("confirmation.json")).unwrap();
        assert_eq!(confirmation_after.status, SpecLoopStageStatus::Confirmed);
        assert!(!confirmation_after.evidence_refs.is_empty());

        materialize_spec_from_requirement_preview(dir.path(), "999-task-workflow-test").unwrap();

        let materialization_after: SpecLoopStageArtifact =
            read_json(&requirement_dir.join("materialization.json")).unwrap();
        assert_eq!(
            materialization_after.status,
            SpecLoopStageStatus::Materialized
        );
        assert_eq!(
            materialization_after.authority,
            SpecArtifactAuthority::Authority
        );
        assert!(materialization_after
            .output_refs
            .iter()
            .any(|path| path == ".agentflow/spec/projects/project-preview.json"));
        assert!(materialization_after
            .output_refs
            .iter()
            .any(|path| path.starts_with(".agentflow/spec/issues/")));
    }

    #[test]
    fn confirmed_preview_materializes_spec_project_and_issues() {
        let dir = tempdir().unwrap();
        let requirement = write_requirement(dir.path());

        requirement_preview_from_requirement(dir.path(), &requirement, Some("project-preview"))
            .unwrap();
        confirm_goal_draft_preview(dir.path(), "999-task-workflow-test", "goal-agent").unwrap();
        confirm_plan_draft_preview(dir.path(), "999-task-workflow-test", "spec-agent").unwrap();

        let (project, issues) =
            materialize_spec_from_requirement_preview(dir.path(), "999-task-workflow-test")
                .unwrap();

        assert_eq!(project.project_id, "project-preview");
        assert_eq!(issues.len(), 2);
        assert!(dir
            .path()
            .join(".agentflow/spec/projects/project-preview.json")
            .is_file());
        assert!(dir
            .path()
            .join(format!(
                ".agentflow/spec/issues/{}.json",
                issues[0].issue_id
            ))
            .is_file());
        let preview =
            read_requirement_preview_runtime(dir.path(), "999-task-workflow-test").unwrap();
        assert_eq!(preview.lifecycle, RequirementPreviewLifecycle::Materialized);
        assert_eq!(preview.next_recommended_action, "start-project-loop");
        assert_eq!(
            preview.materialized_project_id.as_deref(),
            Some("project-preview")
        );
        assert_eq!(preview.materialized_issue_ids.len(), 2);
    }

    #[test]
    fn sync_completion_runtime_enters_goal_recheck_after_all_tasks_done() {
        let dir = tempdir().unwrap();
        let requirement = write_requirement(dir.path());
        let mut issue_draft = SpecIssueDraft::new("AF-COMP-001");
        issue_draft.project_id = Some("project-completion".to_string());
        let mut issue = issue_from_requirement(dir.path(), &requirement, issue_draft).unwrap();
        issue.status = SpecIssueStatus::Done;
        write_spec_issue(dir.path(), &issue).unwrap();

        let mut project_draft = SpecProjectDraft::new("project-completion");
        project_draft.issue_ids = vec!["AF-COMP-001".to_string()];
        let project = project_from_requirement(dir.path(), &requirement, project_draft).unwrap();
        write_spec_project(dir.path(), &project).unwrap();

        let runtimes = sync_completion_decision_runtimes(dir.path()).unwrap();
        assert_eq!(runtimes.len(), 1);
        assert_eq!(runtimes[0].project_id, "project-completion");
        assert_eq!(
            runtimes[0].current_state,
            CompletionDecisionState::GoalRecheck
        );
        assert_eq!(runtimes[0].latest_outcome, None);
        assert_eq!(runtimes[0].facts.completed_issue_count, 1);
        assert_eq!(
            runtimes[0].next_recommended_action,
            "resolve-completion-blockers"
        );
        assert_eq!(runtimes[0].facts.task_evidence_missing_count, 1);
    }

    #[test]
    fn recording_accept_completion_decision_marks_runtime_accepted() {
        let dir = tempdir().unwrap();
        let requirement = write_requirement(dir.path());
        let mut issue_draft = SpecIssueDraft::new("AF-COMP-002");
        issue_draft.project_id = Some("project-completion-accepted".to_string());
        let mut issue = issue_from_requirement(dir.path(), &requirement, issue_draft).unwrap();
        issue.status = SpecIssueStatus::Done;
        write_spec_issue(dir.path(), &issue).unwrap();

        let mut project_draft = SpecProjectDraft::new("project-completion-accepted");
        project_draft.issue_ids = vec!["AF-COMP-002".to_string()];
        let project = project_from_requirement(dir.path(), &requirement, project_draft).unwrap();
        write_spec_project(dir.path(), &project).unwrap();
        write_completion_ready_artifacts(dir.path(), "AF-COMP-002", "run-001", false);
        sync_completion_decision_runtimes(dir.path()).unwrap();

        let runtime = record_completion_decision(
            dir.path(),
            "project-completion-accepted",
            CompletionDecisionOutcome::Accept,
            "goal-agent",
            "当前交付满足目标。",
            vec!["任务与交付都已经满足当前项目目标。".to_string()],
        )
        .unwrap();

        assert_eq!(runtime.current_state, CompletionDecisionState::Accepted);
        assert_eq!(
            runtime.latest_outcome,
            Some(CompletionDecisionOutcome::Accept)
        );
        assert_eq!(runtime.next_recommended_action, "project-accepted");

        let events = load_task_events(dir.path()).unwrap();
        assert!(events
            .iter()
            .any(|event| event.event_type == "project.accepted"));
    }

    #[test]
    fn accept_completion_rejects_missing_evidence() {
        let dir = tempdir().unwrap();
        let requirement = write_requirement(dir.path());
        let mut issue_draft = SpecIssueDraft::new("AF-COMP-004");
        issue_draft.project_id = Some("project-completion-no-evidence".to_string());
        let mut issue = issue_from_requirement(dir.path(), &requirement, issue_draft).unwrap();
        issue.status = SpecIssueStatus::Done;
        write_spec_issue(dir.path(), &issue).unwrap();

        let mut project_draft = SpecProjectDraft::new("project-completion-no-evidence");
        project_draft.issue_ids = vec!["AF-COMP-004".to_string()];
        let project = project_from_requirement(dir.path(), &requirement, project_draft).unwrap();
        write_spec_project(dir.path(), &project).unwrap();
        sync_completion_decision_runtimes(dir.path()).unwrap();

        let err = record_completion_decision(
            dir.path(),
            "project-completion-no-evidence",
            CompletionDecisionOutcome::Accept,
            "goal-agent",
            "尝试完成。",
            vec!["证据缺失时不允许 Accept。".to_string()],
        )
        .unwrap_err();
        assert!(err.to_string().contains("缺少验证证据"));
    }

    #[test]
    fn accept_completion_rejects_failed_audit() {
        let dir = tempdir().unwrap();
        let requirement = write_requirement(dir.path());
        let mut issue_draft = SpecIssueDraft::new("AF-COMP-005");
        issue_draft.project_id = Some("project-completion-failed-audit".to_string());
        let mut issue = issue_from_requirement(dir.path(), &requirement, issue_draft).unwrap();
        issue.status = SpecIssueStatus::Done;
        write_spec_issue(dir.path(), &issue).unwrap();

        let mut project_draft = SpecProjectDraft::new("project-completion-failed-audit");
        project_draft.issue_ids = vec!["AF-COMP-005".to_string()];
        let project = project_from_requirement(dir.path(), &requirement, project_draft).unwrap();
        write_spec_project(dir.path(), &project).unwrap();
        write_completion_ready_artifacts(dir.path(), "AF-COMP-005", "run-001", false);
        fs::create_dir_all(dir.path().join(".agentflow/audit/audit-001")).unwrap();
        write_json(
            &dir.path().join(".agentflow/audit/index.json"),
            &serde_json::json!({
                "version": "audit-index.v1",
                "updatedAt": 1,
                "audits": [{
                    "auditId": "audit-001",
                    "status": "failed",
                    "trigger": "human-via-agent",
                    "requestedBy": "audit-agent",
                    "requestedAt": 1,
                    "sourceIssueId": "AF-COMP-005",
                    "reportPath": ".agentflow/audit/audit-001/audit-report.md",
                    "auditPath": ".agentflow/audit/audit-001/audit.json"
                }]
            }),
        )
        .unwrap();
        write_json(
            &dir.path()
                .join(".agentflow/audit/audit-001/audit-report.json"),
            &serde_json::json!({
                "version": "audit-result-summary.v1",
                "auditId": "audit-001",
                "status": "failed",
                "requestedAt": 1,
                "sourceIssueId": "AF-COMP-005",
                "reportPath": ".agentflow/audit/audit-001/audit-report.md",
                "summaryLine": "审计失败。",
                "findingsCount": 2,
                "findings": ["high：交付不完整"],
                "evidenceGaps": ["缺少公开交付记录"],
                "repairRecommendations": ["补齐交付后重新审计"]
            }),
        )
        .unwrap();
        sync_completion_decision_runtimes(dir.path()).unwrap();

        let err = record_completion_decision(
            dir.path(),
            "project-completion-failed-audit",
            CompletionDecisionOutcome::Accept,
            "goal-agent",
            "尝试完成。",
            vec!["审计失败时不允许 Accept。".to_string()],
        )
        .unwrap_err();
        assert!(err.to_string().contains("审计当前处于 failed"));
    }

    #[test]
    fn accept_completion_rejects_goal_recheck_gap() {
        let dir = tempdir().unwrap();
        let requirement = write_requirement(dir.path());
        let mut issue_draft = SpecIssueDraft::new("AF-COMP-006");
        issue_draft.project_id = Some("project-completion-goal-gap".to_string());
        let mut issue = issue_from_requirement(dir.path(), &requirement, issue_draft).unwrap();
        issue.status = SpecIssueStatus::Done;
        write_spec_issue(dir.path(), &issue).unwrap();

        let mut project_draft = SpecProjectDraft::new("project-completion-goal-gap");
        project_draft.issue_ids = vec!["AF-COMP-006".to_string()];
        let project = project_from_requirement(dir.path(), &requirement, project_draft).unwrap();
        write_spec_project(dir.path(), &project).unwrap();
        write_completion_ready_artifacts(dir.path(), "AF-COMP-006", "run-001", false);
        write_project_health(dir.path(), "project-completion-goal-gap", "draft");
        sync_completion_decision_runtimes(dir.path()).unwrap();

        let err = record_completion_decision(
            dir.path(),
            "project-completion-goal-gap",
            CompletionDecisionOutcome::Accept,
            "goal-agent",
            "尝试完成。",
            vec!["Goal Recheck 未通过时不允许 Accept。".to_string()],
        )
        .unwrap_err();
        assert!(err
            .to_string()
            .contains("Goal Recheck 当前状态是 needs-recheck"));
    }

    #[test]
    fn recording_continue_completion_decision_emits_goal_recheck_event() {
        let dir = tempdir().unwrap();
        let requirement = write_requirement(dir.path());
        let mut issue_draft = SpecIssueDraft::new("AF-COMP-003");
        issue_draft.project_id = Some("project-completion-continue".to_string());
        let mut issue = issue_from_requirement(dir.path(), &requirement, issue_draft).unwrap();
        issue.status = SpecIssueStatus::Done;
        write_spec_issue(dir.path(), &issue).unwrap();

        let mut project_draft = SpecProjectDraft::new("project-completion-continue");
        project_draft.issue_ids = vec!["AF-COMP-003".to_string()];
        let project = project_from_requirement(dir.path(), &requirement, project_draft).unwrap();
        write_spec_project(dir.path(), &project).unwrap();
        sync_completion_decision_runtimes(dir.path()).unwrap();

        let runtime = record_completion_decision(
            dir.path(),
            "project-completion-continue",
            CompletionDecisionOutcome::Continue,
            "goal-agent",
            "当前交付需要继续推进。",
            vec!["还有后续任务需要继续处理。".to_string()],
        )
        .unwrap();

        assert_eq!(runtime.current_state, CompletionDecisionState::Continue);
        assert_eq!(
            runtime.latest_outcome,
            Some(CompletionDecisionOutcome::Continue)
        );
        assert_eq!(runtime.next_recommended_action, "start-project-loop");

        let events = load_task_events(dir.path()).unwrap();
        let event = events
            .iter()
            .find(|event| event.event_type == "project.goal_recheck.continue")
            .expect("missing continue goal recheck event");
        assert_eq!(
            event.project_id.as_deref(),
            Some("project-completion-continue")
        );
        assert_eq!(
            event.state.as_ref().map(|state| state.from_state.as_str()),
            Some("goal-recheck")
        );
        assert_eq!(
            event.state.as_ref().map(|state| state.to_state.as_str()),
            Some("continue")
        );
        assert_eq!(
            event.payload["completionDecisionRef"].as_str(),
            Some(".agentflow/spec/completions/project-completion-continue.json")
        );
    }
}
