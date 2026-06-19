use agentflow_audit::{
    request_human_audit, AuditScope, AuditScopeRef, HumanAuditReport, HumanAuditRequestDraft,
};
use agentflow_release::{
    confirm_project_release, prepare_project_release, publish_project_release,
    record_project_release_tag, record_project_remote_release,
};
use agentflow_spec::{
    confirm_goal_draft_preview, confirm_plan_draft_preview, list_spec_issues,
    materialize_spec_from_requirement_preview, read_completion_decision_runtime,
    read_requirement_preview_runtime, record_completion_decision,
    requirement_preview_from_requirement, sync_completion_decision_runtimes,
    CompletionDecisionOutcome, CompletionDecisionRuntime, RequirementPreviewRuntime, SpecIssue,
    SpecProject,
};
use agentflow_task_artifacts::load_task_run;
use anyhow::{bail, Result};
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProjectMaterializationResult {
    pub project: SpecProject,
    pub issues: Vec<SpecIssue>,
}

pub(crate) fn project_intake(
    root: &Path,
    requirement_path: &Path,
    project_id: Option<&str>,
) -> Result<RequirementPreviewRuntime> {
    let preview = requirement_preview_from_requirement(root, requirement_path, project_id)?;
    let _ = agentflow_projection::rebuild_projections(root)?;
    Ok(preview)
}

pub(crate) fn project_preview_goal(
    root: &Path,
    requirement_id: &str,
) -> Result<RequirementPreviewRuntime> {
    read_requirement_preview_runtime(root, requirement_id)
}

pub(crate) fn project_confirm_goal(
    root: &Path,
    requirement_id: &str,
    actor: &str,
) -> Result<RequirementPreviewRuntime> {
    let preview = confirm_goal_draft_preview(root, requirement_id, actor)?;
    let _ = agentflow_projection::rebuild_projections(root)?;
    Ok(preview)
}

pub(crate) fn project_confirm_plan(
    root: &Path,
    requirement_id: &str,
    actor: &str,
) -> Result<RequirementPreviewRuntime> {
    let preview = confirm_plan_draft_preview(root, requirement_id, actor)?;
    let _ = agentflow_projection::rebuild_projections(root)?;
    Ok(preview)
}

pub(crate) fn project_materialize(
    root: &Path,
    requirement_id: &str,
) -> Result<ProjectMaterializationResult> {
    let (project, issues) = materialize_spec_from_requirement_preview(root, requirement_id)?;
    let _ = agentflow_projection::rebuild_projections(root)?;
    Ok(ProjectMaterializationResult { project, issues })
}

pub(crate) fn audit_request_human(
    root: &Path,
    run_id: &str,
    issue_id: Option<&str>,
    reason: &str,
    public_delivery_path: &str,
) -> Result<HumanAuditReport> {
    let issue_id = resolve_issue_id_for_run(root, run_id, issue_id)?;
    let draft = HumanAuditRequestDraft {
        reason: reason.trim().to_string(),
        scope: AuditScope {
            description: format!("Audit workflow delivery for {issue_id} / {run_id}."),
            refs: vec![
                AuditScopeRef {
                    kind: "issue".to_string(),
                    id: issue_id.clone(),
                    path: format!(".agentflow/spec/issues/{issue_id}.json"),
                },
                AuditScopeRef {
                    kind: "task-run".to_string(),
                    id: run_id.to_string(),
                    path: format!(".agentflow/tasks/{issue_id}/runs/{run_id}/run.json"),
                },
                AuditScopeRef {
                    kind: "evidence".to_string(),
                    id: run_id.to_string(),
                    path: format!(".agentflow/tasks/{issue_id}/evidence/evidence.json"),
                },
                AuditScopeRef {
                    kind: "public-delivery".to_string(),
                    id: public_delivery_path.trim().to_string(),
                    path: public_delivery_path.trim().to_string(),
                },
            ],
        },
    };
    request_human_audit(root, draft)
}

pub(crate) fn completion_inspect(
    root: &Path,
    project_id: &str,
) -> Result<CompletionDecisionRuntime> {
    let _ = sync_completion_decision_runtimes(root)?;
    read_completion_decision_runtime(root, project_id)
}

pub(crate) fn completion_decide(
    root: &Path,
    project_id: &str,
    outcome: &str,
    actor: &str,
    summary: &str,
    rationale: Vec<String>,
) -> Result<CompletionDecisionRuntime> {
    let outcome = parse_completion_outcome(outcome)?;
    let runtime = record_completion_decision(root, project_id, outcome, actor, summary, rationale)?;
    let _ = agentflow_projection::rebuild_projections(root)?;
    Ok(runtime)
}

pub(crate) fn release_prepare(
    root: &Path,
    project_id: &str,
) -> Result<agentflow_release::ProjectReleaseFacts> {
    let facts = prepare_project_release(root, project_id)?;
    let _ = agentflow_projection::rebuild_projections(root)?;
    Ok(facts)
}

pub(crate) fn release_confirm(
    root: &Path,
    project_id: &str,
) -> Result<agentflow_release::ProjectReleaseFacts> {
    let facts = confirm_project_release(root, project_id)?;
    let _ = agentflow_projection::rebuild_projections(root)?;
    Ok(facts)
}

pub(crate) fn release_publish(
    root: &Path,
    project_id: &str,
) -> Result<agentflow_release::ProjectReleaseFacts> {
    let facts = publish_project_release(root, project_id)?;
    let _ = agentflow_projection::rebuild_projections(root)?;
    Ok(facts)
}

pub(crate) fn release_record_tag(
    root: &Path,
    project_id: &str,
    tag_name: &str,
    tag_commit_sha: &str,
    actor: &str,
) -> Result<agentflow_release::ProjectReleaseFacts> {
    let facts = record_project_release_tag(root, project_id, tag_name, tag_commit_sha, actor)?;
    let _ = agentflow_projection::rebuild_projections(root)?;
    Ok(facts)
}

pub(crate) fn release_record_remote(
    root: &Path,
    project_id: &str,
    provider: &str,
    release_id: &str,
    release_url: &str,
    tag_name: &str,
    release_commit_sha: &str,
    artifact_manifest_path: &str,
    actor: &str,
) -> Result<agentflow_release::ProjectReleaseFacts> {
    let facts = record_project_remote_release(
        root,
        project_id,
        provider,
        release_id,
        release_url,
        tag_name,
        release_commit_sha,
        artifact_manifest_path,
        actor,
    )?;
    let _ = agentflow_projection::rebuild_projections(root)?;
    Ok(facts)
}

pub(crate) fn parse_completion_outcome(raw: &str) -> Result<CompletionDecisionOutcome> {
    match raw.trim() {
        "continue" => Ok(CompletionDecisionOutcome::Continue),
        "adjust" => Ok(CompletionDecisionOutcome::Adjust),
        "pause" => Ok(CompletionDecisionOutcome::Pause),
        "accept" => Ok(CompletionDecisionOutcome::Accept),
        "next-stage" => Ok(CompletionDecisionOutcome::NextStage),
        other => bail!("unsupported completion outcome: {other}"),
    }
}

fn resolve_issue_id_for_run(root: &Path, run_id: &str, issue_id: Option<&str>) -> Result<String> {
    if let Some(issue_id) = issue_id.map(str::trim).filter(|value| !value.is_empty()) {
        load_task_run(root, issue_id, run_id)?;
        return Ok(issue_id.to_string());
    }

    let issues = list_spec_issues(root)?;
    let mut matched = issues
        .into_iter()
        .filter_map(|issue| {
            load_task_run(root, &issue.issue_id, run_id)
                .ok()
                .map(|_| issue.issue_id)
        })
        .collect::<Vec<_>>();
    matched.sort();
    matched.dedup();
    match matched.len() {
        1 => Ok(matched.remove(0)),
        0 => bail!("no task run matched runId {run_id}"),
        _ => bail!("multiple task runs matched runId {run_id}; pass --issue-id explicitly"),
    }
}
