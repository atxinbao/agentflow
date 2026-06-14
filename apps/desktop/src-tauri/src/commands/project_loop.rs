use agentflow_projection::{load_project_projection, load_task_projection};
use agentflow_spec::{list_spec_issues, list_spec_projects, SpecIssue};
use agentflow_task_loop::TaskLoop;
use serde::Serialize;
use std::path::Path;
use tauri::{AppHandle, Emitter};

const PROJECT_LOOP_RUN_VERSION: &str = "desktop-project-loop-run.v2";
const AGENTFLOW_PROJECT_LOOP_TICKED_EVENT: &str = "agentflow-project-loop-ticked";
const DEFAULT_AGENT_PROVIDER: &str = "codex";

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProjectLoopRunSummary {
    version: String,
    project_root: String,
    project_count: usize,
    direct_issue_count: usize,
    runtime_launch_count: usize,
    active_issue_count: usize,
    blocked_issue_count: usize,
    done_issue_count: usize,
    projects: Vec<ProjectLoopProjectSummary>,
    errors: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProjectLoopProjectSummary {
    project_id: String,
    status: String,
    active_issue_ids: Vec<String>,
    blocked_issue_ids: Vec<String>,
    done_issue_ids: Vec<String>,
    runtime_issue_id: Option<String>,
    runtime_run_id: Option<String>,
    runtime_stage: Option<String>,
    runtime_launch_request_path: Option<String>,
    blockers: Vec<LoopBlocker>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LoopBlocker {
    code: String,
    reason: String,
    issue_id: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectLoopTickedEvent {
    version: String,
    project_root: String,
    project_count: usize,
    direct_issue_count: usize,
    runtime_launch_count: usize,
    active_issue_count: usize,
    blocked_issue_count: usize,
    done_issue_count: usize,
    errors: Vec<String>,
}

#[tauri::command]
pub(crate) fn run_project_loop(
    project_root: String,
    app: AppHandle,
) -> Result<ProjectLoopRunSummary, String> {
    run_project_loop_for_app(project_root, &app)
}

pub(crate) fn run_project_loop_for_app(
    project_root: impl AsRef<Path>,
    app: &AppHandle,
) -> Result<ProjectLoopRunSummary, String> {
    let summary = run_project_loop_inner(project_root).map_err(|error| error.to_string())?;
    let payload = ProjectLoopTickedEvent {
        version: summary.version.clone(),
        project_root: summary.project_root.clone(),
        project_count: summary.project_count,
        direct_issue_count: summary.direct_issue_count,
        runtime_launch_count: summary.runtime_launch_count,
        active_issue_count: summary.active_issue_count,
        blocked_issue_count: summary.blocked_issue_count,
        done_issue_count: summary.done_issue_count,
        errors: summary.errors.clone(),
    };
    let _ = app.emit(AGENTFLOW_PROJECT_LOOP_TICKED_EVENT, payload);
    Ok(summary)
}

pub(crate) fn run_project_loop_inner(
    project_root: impl AsRef<Path>,
) -> anyhow::Result<ProjectLoopRunSummary> {
    let root = project_root.as_ref().canonicalize()?;
    agentflow_spec::prepare_spec_workspace(&root)?;

    let projects = list_spec_projects(&root)?;
    let issues = list_spec_issues(&root)?;
    let mut ticks = Vec::new();
    let mut errors = Vec::new();

    for project in &projects {
        match TaskLoop::new(project.project_id.clone()).tick(&root, DEFAULT_AGENT_PROVIDER) {
            Ok(Some(tick)) => ticks.push(tick),
            Ok(None) => {}
            Err(error) => errors.push(format!("{}: {error}", project.project_id)),
        }
    }

    let _ = agentflow_projection::rebuild_projections(&root)?;

    let mut summary = ProjectLoopRunSummary {
        version: PROJECT_LOOP_RUN_VERSION.to_string(),
        project_root: root.display().to_string(),
        project_count: projects.len(),
        direct_issue_count: issues
            .iter()
            .filter(|issue| issue.project_id.is_none())
            .count(),
        runtime_launch_count: ticks.len(),
        active_issue_count: 0,
        blocked_issue_count: 0,
        done_issue_count: 0,
        projects: Vec::new(),
        errors,
    };

    for issue in issues.iter().filter(|issue| issue.project_id.is_none()) {
        match projected_issue_state(&root, issue) {
            "blocked" => summary.blocked_issue_count += 1,
            "done" | "cancel" => summary.done_issue_count += 1,
            "todo" | "in_progress" | "in_review" => summary.active_issue_count += 1,
            _ => {}
        }
    }

    for project in projects {
        let tick = ticks
            .iter()
            .find(|tick| tick.launch.project_id.as_deref() == Some(project.project_id.as_str()));
        let mut active_issue_ids = Vec::new();
        let mut blocked_issue_ids = Vec::new();
        let mut done_issue_ids = Vec::new();

        for issue_id in &project.issue_ids {
            let issue = issues.iter().find(|issue| &issue.issue_id == issue_id);
            let state = issue
                .map(|issue| projected_issue_state(&root, issue))
                .unwrap_or("backlog");
            match state {
                "blocked" => blocked_issue_ids.push(issue_id.clone()),
                "done" | "cancel" => done_issue_ids.push(issue_id.clone()),
                "todo" | "in_progress" | "in_review" => active_issue_ids.push(issue_id.clone()),
                _ => {}
            }
        }

        summary.active_issue_count += active_issue_ids.len();
        summary.blocked_issue_count += blocked_issue_ids.len();
        summary.done_issue_count += done_issue_ids.len();

        let projection = load_project_projection(&root, &project.project_id).ok();
        let runtime_issue_id = tick.map(|tick| tick.launch.issue_id.clone()).or_else(|| {
            projection
                .as_ref()
                .and_then(|value| value.current_issue_id.clone())
        });
        let runtime_stage = runtime_issue_id
            .as_ref()
            .and_then(|issue_id| load_task_projection(&root, issue_id).ok())
            .map(|projection| projection.current_state);

        summary.projects.push(ProjectLoopProjectSummary {
            project_id: project.project_id.clone(),
            status: projection
                .as_ref()
                .map(|value| value.status.clone())
                .unwrap_or_else(|| "planned".to_string()),
            active_issue_ids,
            blocked_issue_ids,
            done_issue_ids,
            runtime_issue_id,
            runtime_run_id: tick.map(|tick| tick.launch.run_id.clone()),
            runtime_stage,
            runtime_launch_request_path: tick.map(|tick| tick.launch.launch_request_path.clone()),
            blockers: Vec::new(),
        });
    }

    Ok(summary)
}

fn projected_issue_state<'a>(root: &Path, issue: &'a SpecIssue) -> &'a str {
    if let Ok(projection) = load_task_projection(root, &issue.issue_id) {
        match projection.current_state.as_str() {
            "backlog" => "backlog",
            "todo" => "todo",
            "in_progress" => "in_progress",
            "in_review" => "in_review",
            "done" => "done",
            "blocked" => "blocked",
            "cancel" => "cancel",
            _ => issue.status.as_str(),
        }
    } else {
        issue.status.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agentflow_spec::{
        issue_from_requirement, project_from_requirement, read_spec_issue, write_spec_issue,
        write_spec_project, SpecIssueDraft, SpecIssueStatus, SpecProjectDraft,
    };
    use std::fs;
    use tempfile::tempdir;

    fn write_requirement(root: &Path) -> std::path::PathBuf {
        let path = root.join("docs/requirements/034-desktop-loop-test.md");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(
            &path,
            "# Desktop Project Loop\n\n验证桌面按钮走 task-loop。\n",
        )
        .unwrap();
        path
    }

    fn write_project_with_backlog_issue(root: &Path) {
        let requirement = write_requirement(root);
        let mut issue = SpecIssueDraft::new("AF-LOOP-001");
        issue.project_id = Some("project-loop".to_string());
        issue.allowed_paths = vec!["apps/desktop/src/**".to_string()];
        issue.validation_commands = vec!["npm --prefix apps/desktop run build".to_string()];
        let issue = issue_from_requirement(root, &requirement, issue).unwrap();
        write_spec_issue(root, &issue).unwrap();

        let mut project = SpecProjectDraft::new("project-loop");
        project.issue_ids = vec!["AF-LOOP-001".to_string()];
        let project = project_from_requirement(root, &requirement, project).unwrap();
        write_spec_project(root, &project).unwrap();
    }

    fn write_direct_backlog_issue(root: &Path) {
        let requirement = write_requirement(root);
        let mut issue = SpecIssueDraft::new("AF-DIRECT-001");
        issue.allowed_paths = vec!["apps/desktop/src/**".to_string()];
        let issue = issue_from_requirement(root, &requirement, issue).unwrap();
        write_spec_issue(root, &issue).unwrap();
    }

    #[test]
    fn run_project_loop_launches_spec_project_issue_from_desktop_entrypoint() {
        let dir = tempdir().unwrap();
        write_project_with_backlog_issue(dir.path());

        let summary = run_project_loop_inner(dir.path()).unwrap();

        assert_eq!(summary.project_count, 1);
        assert_eq!(summary.direct_issue_count, 0);
        assert_eq!(summary.runtime_launch_count, 1);
        assert_eq!(summary.active_issue_count, 1);
        assert_eq!(summary.blocked_issue_count, 0);
        assert!(summary.errors.is_empty());
        assert_eq!(summary.projects[0].project_id, "project-loop");
        assert_eq!(summary.projects[0].status, "active");
        assert_eq!(summary.projects[0].active_issue_ids, vec!["AF-LOOP-001"]);
        assert_eq!(
            summary.projects[0].runtime_issue_id.as_deref(),
            Some("AF-LOOP-001")
        );
        assert_eq!(
            summary.projects[0].runtime_stage.as_deref(),
            Some("in_progress")
        );
        assert_eq!(
            summary.projects[0].runtime_launch_request_path.as_deref(),
            Some(".agentflow/tasks/AF-LOOP-001/runs/run-001/launch/agent-request.json")
        );
        assert!(dir
            .path()
            .join(".agentflow/tasks/AF-LOOP-001/runs/run-001/launch/agent-request.json")
            .is_file());
        assert!(!dir.path().join(".agentflow/execute").exists());

        let issue = read_spec_issue(dir.path(), "AF-LOOP-001").unwrap();
        assert_eq!(issue.status, SpecIssueStatus::Backlog);
        let projection = load_task_projection(dir.path(), "AF-LOOP-001").unwrap();
        assert_eq!(projection.current_state, "in_progress");
        assert_eq!(projection.latest_run_id.as_deref(), Some("run-001"));
    }

    #[test]
    fn run_project_loop_keeps_direct_issues_out_of_project_scheduler() {
        let dir = tempdir().unwrap();
        write_direct_backlog_issue(dir.path());

        let summary = run_project_loop_inner(dir.path()).unwrap();

        assert_eq!(summary.project_count, 0);
        assert_eq!(summary.direct_issue_count, 1);
        assert_eq!(summary.runtime_launch_count, 0);
        assert_eq!(summary.active_issue_count, 0);
        assert_eq!(summary.blocked_issue_count, 0);
        assert_eq!(summary.done_issue_count, 0);
        assert!(summary.errors.is_empty());

        let projection = load_task_projection(dir.path(), "AF-DIRECT-001").unwrap();
        assert_eq!(projection.current_state, "backlog");
    }
}
