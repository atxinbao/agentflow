//! Project / Completion / Release runtime commands for Desktop.
//!
//! These commands expose the formal runtime entry points behind Project Brain,
//! Completion Decision, and Release. They share the same crate-level runtime
//! APIs as the CLI and rebuild projections after every mutation so Desktop can
//! refresh from the authoritative read model.

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProjectMaterializationResult {
    project: agentflow_spec::SpecProject,
    issues: Vec<agentflow_spec::SpecIssue>,
}

#[tauri::command]
pub(crate) fn project_intake(
    project_root: String,
    requirement_path: String,
    project_id: Option<String>,
) -> Result<agentflow_spec::RequirementPreviewRuntime, String> {
    let preview = agentflow_spec::requirement_preview_from_requirement(
        &project_root,
        &requirement_path,
        project_id.as_deref(),
    )
    .map_err(|error| error.to_string())?;
    agentflow_projection::rebuild_projections(&project_root).map_err(|error| error.to_string())?;
    Ok(preview)
}

#[tauri::command]
pub(crate) fn project_preview_goal(
    project_root: String,
    requirement_id: String,
) -> Result<agentflow_spec::RequirementPreviewRuntime, String> {
    agentflow_spec::read_requirement_preview_runtime(&project_root, &requirement_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn project_confirm_goal(
    project_root: String,
    requirement_id: String,
    actor: Option<String>,
) -> Result<agentflow_spec::RequirementPreviewRuntime, String> {
    let preview = agentflow_spec::confirm_goal_draft_preview(
        &project_root,
        &requirement_id,
        actor.as_deref().unwrap_or("goal-agent"),
    )
    .map_err(|error| error.to_string())?;
    agentflow_projection::rebuild_projections(&project_root).map_err(|error| error.to_string())?;
    Ok(preview)
}

#[tauri::command]
pub(crate) fn project_confirm_plan(
    project_root: String,
    requirement_id: String,
    actor: Option<String>,
) -> Result<agentflow_spec::RequirementPreviewRuntime, String> {
    let preview = agentflow_spec::confirm_plan_draft_preview(
        &project_root,
        &requirement_id,
        actor.as_deref().unwrap_or("goal-agent"),
    )
    .map_err(|error| error.to_string())?;
    agentflow_projection::rebuild_projections(&project_root).map_err(|error| error.to_string())?;
    Ok(preview)
}

#[tauri::command]
pub(crate) fn project_materialize(
    project_root: String,
    requirement_id: String,
) -> Result<ProjectMaterializationResult, String> {
    let (project, issues) =
        agentflow_spec::materialize_spec_from_requirement_preview(&project_root, &requirement_id)
            .map_err(|error| error.to_string())?;
    agentflow_projection::rebuild_projections(&project_root).map_err(|error| error.to_string())?;
    Ok(ProjectMaterializationResult { project, issues })
}

#[tauri::command]
pub(crate) fn completion_inspect(
    project_root: String,
    project_id: String,
) -> Result<agentflow_spec::CompletionDecisionRuntime, String> {
    agentflow_spec::sync_completion_decision_runtimes(&project_root)
        .map_err(|error| error.to_string())?;
    agentflow_spec::read_completion_decision_runtime(&project_root, &project_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn completion_decide(
    project_root: String,
    project_id: String,
    outcome: String,
    actor: Option<String>,
    summary: String,
    rationale: Vec<String>,
) -> Result<agentflow_spec::CompletionDecisionRuntime, String> {
    let parsed = parse_completion_outcome(&outcome).map_err(|error| error.to_string())?;
    let runtime = agentflow_spec::record_completion_decision(
        &project_root,
        &project_id,
        parsed,
        actor.as_deref().unwrap_or("goal-agent"),
        &summary,
        rationale,
    )
    .map_err(|error| error.to_string())?;
    agentflow_projection::rebuild_projections(&project_root).map_err(|error| error.to_string())?;
    Ok(runtime)
}

#[tauri::command]
pub(crate) fn release_prepare(
    project_root: String,
    project_id: String,
) -> Result<agentflow_release::ProjectReleaseFacts, String> {
    let facts = agentflow_release::prepare_project_release(&project_root, &project_id)
        .map_err(|error| error.to_string())?;
    agentflow_projection::rebuild_projections(&project_root).map_err(|error| error.to_string())?;
    Ok(facts)
}

#[tauri::command]
pub(crate) fn release_confirm(
    project_root: String,
    project_id: String,
) -> Result<agentflow_release::ProjectReleaseFacts, String> {
    let facts = agentflow_release::confirm_project_release(&project_root, &project_id)
        .map_err(|error| error.to_string())?;
    agentflow_projection::rebuild_projections(&project_root).map_err(|error| error.to_string())?;
    Ok(facts)
}

#[tauri::command]
pub(crate) fn release_publish(
    project_root: String,
    project_id: String,
) -> Result<agentflow_release::ProjectReleaseFacts, String> {
    let facts = agentflow_release::publish_project_release(&project_root, &project_id)
        .map_err(|error| error.to_string())?;
    agentflow_projection::rebuild_projections(&project_root).map_err(|error| error.to_string())?;
    Ok(facts)
}

fn parse_completion_outcome(
    raw: &str,
) -> anyhow::Result<agentflow_spec::CompletionDecisionOutcome> {
    match raw.trim() {
        "continue" => Ok(agentflow_spec::CompletionDecisionOutcome::Continue),
        "adjust" => Ok(agentflow_spec::CompletionDecisionOutcome::Adjust),
        "pause" => Ok(agentflow_spec::CompletionDecisionOutcome::Pause),
        "accept" => Ok(agentflow_spec::CompletionDecisionOutcome::Accept),
        "next-stage" => Ok(agentflow_spec::CompletionDecisionOutcome::NextStage),
        other => anyhow::bail!("unsupported completion outcome: {other}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn write_requirement(root: &std::path::Path) -> String {
        let path = root.join("docs/requirements/058e-runtime-entry-test.md");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(
            &path,
            "# Runtime Entry Test\n\n验证 formal project/completion/release 入口。\n",
        )
        .unwrap();
        "docs/requirements/058e-runtime-entry-test.md".to_string()
    }

    fn write_public_delivery_projection(root: &std::path::Path, issue_id: &str, project_id: &str) {
        let path = root.join(".agentflow/projections/tasks");
        fs::create_dir_all(&path).unwrap();
        fs::write(
            path.join(format!("{issue_id}.json")),
            serde_json::to_string_pretty(&serde_json::json!({
                "issueId": issue_id,
                "projectId": project_id,
                "currentState": "done",
                "publicDelivery": {
                    "prUrl": "https://github.com/acme/repo/pull/58",
                    "mergeCommit": "merge-058e",
                    "changelogPath": "CHANGELOG.md",
                    "releaseNotesUrl": format!("docs/release-notes/{project_id}.md"),
                },
                "delivery": {
                    "status": "published",
                    "evidenceStatus": "ready",
                    "evidencePath": format!(".agentflow/tasks/{issue_id}/evidence/evidence.json"),
                    "prUrl": "https://github.com/acme/repo/pull/58",
                    "mergeCommit": "merge-058e",
                    "publicRecordPath": "CHANGELOG.md",
                },
                "updatedAt": 58
            }))
            .unwrap(),
        )
        .unwrap();
    }

    fn write_project_release_projection(root: &std::path::Path, project_id: &str) {
        let path = root.join(".agentflow/projections/projects");
        fs::create_dir_all(&path).unwrap();
        fs::write(
            path.join(format!("{project_id}.json")),
            serde_json::to_string_pretty(&serde_json::json!({
                "projectId": project_id,
                "title": "project-runtime-entry",
                "completion": {
                    "currentState": "accepted",
                    "latestOutcome": "accept"
                },
                "delivery": {
                    "status": "published",
                    "missingCount": 0,
                    "summaryLine": "公开交付已统一写入 PR/MR body、CHANGELOG.md。"
                }
            }))
            .unwrap(),
        )
        .unwrap();
    }

    #[test]
    fn project_runtime_commands_materialize_and_drive_completion_release() {
        let dir = tempdir().unwrap();
        let requirement_path = write_requirement(dir.path());

        let intake = project_intake(
            dir.path().display().to_string(),
            requirement_path.clone(),
            Some("project-runtime-entry".to_string()),
        )
        .unwrap();
        assert_eq!(intake.current_state, "goal_draft");

        let goal = project_confirm_goal(
            dir.path().display().to_string(),
            intake.requirement_id.clone(),
            None,
        )
        .unwrap();
        assert_eq!(goal.current_state, "plan_draft");

        let plan = project_confirm_plan(
            dir.path().display().to_string(),
            intake.requirement_id.clone(),
            None,
        )
        .unwrap();
        assert_eq!(plan.current_state, "confirmed");

        let materialized = project_materialize(
            dir.path().display().to_string(),
            intake.requirement_id.clone(),
        )
        .unwrap();
        assert_eq!(materialized.project.project_id, "project-runtime-entry");
        assert!(!materialized.issues.is_empty());

        let project_id = materialized.project.project_id.clone();
        for materialized_issue in &materialized.issues {
            let mut issue =
                agentflow_spec::read_spec_issue(dir.path(), &materialized_issue.issue_id).unwrap();
            issue.status = agentflow_spec::SpecIssueStatus::Done;
            agentflow_spec::write_spec_issue(dir.path(), &issue).unwrap();
        }
        agentflow_projection::rebuild_projections(dir.path()).unwrap();

        let completion =
            completion_inspect(dir.path().display().to_string(), project_id.clone()).unwrap();
        assert_eq!(completion.current_state.as_str(), "goal-recheck");

        let accepted = completion_decide(
            dir.path().display().to_string(),
            project_id.clone(),
            "accept".to_string(),
            None,
            "接受当前项目交付".to_string(),
            vec!["任务执行和公开交付都已完整。".to_string()],
        )
        .unwrap();
        assert_eq!(accepted.current_state.as_str(), "accepted");
        for materialized_issue in &materialized.issues {
            write_public_delivery_projection(dir.path(), &materialized_issue.issue_id, &project_id);
        }
        write_project_release_projection(dir.path(), &project_id);

        let prepared =
            release_prepare(dir.path().display().to_string(), project_id.clone()).unwrap();
        assert_eq!(prepared.current_state, "ready");

        let confirmed =
            release_confirm(dir.path().display().to_string(), project_id.clone()).unwrap();
        assert_eq!(confirmed.current_state, "in_progress");

        let published =
            release_publish(dir.path().display().to_string(), project_id.clone()).unwrap();
        assert_eq!(published.current_state, "published");
    }
}
