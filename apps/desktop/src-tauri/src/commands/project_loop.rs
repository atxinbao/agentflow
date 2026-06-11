use agentflow_loop::{DirectIssueLoopSummary, LoopBlocker, ProjectLoopSnapshot};
use serde::Serialize;
use std::path::Path;
use tauri::{AppHandle, Emitter};

const PROJECT_LOOP_RUN_VERSION: &str = "desktop-project-loop-run.v1";
const AGENTFLOW_PROJECT_LOOP_TICKED_EVENT: &str = "agentflow-project-loop-ticked";

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProjectLoopRunSummary {
    version: String,
    project_root: String,
    project_count: usize,
    direct_issue_count: usize,
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
    blockers: Vec<LoopBlocker>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectLoopTickedEvent {
    version: String,
    project_root: String,
    project_count: usize,
    direct_issue_count: usize,
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
    let snapshot = agentflow_input::prepare_input_workspace(&root)?;
    let mut summary = ProjectLoopRunSummary {
        version: PROJECT_LOOP_RUN_VERSION.to_string(),
        project_root: root.display().to_string(),
        project_count: snapshot.projects.len(),
        direct_issue_count: 0,
        active_issue_count: 0,
        blocked_issue_count: 0,
        done_issue_count: 0,
        projects: Vec::new(),
        errors: Vec::new(),
    };

    match agentflow_loop::DirectIssueLoop::schedule_ready_issues(&root) {
        Ok(direct_summary) => push_direct_summary(&mut summary, direct_summary),
        Err(error) => summary.errors.push(format!("direct issues: {error}")),
    }

    for project in snapshot.projects {
        let project_id = project.project_id;
        let project_loop = agentflow_loop::ProjectLoop::new(project_id.clone());
        match project_loop
            .run_preflight(&root)
            .and_then(|_| project_loop.schedule_ready_issues(&root))
        {
            Ok(loop_snapshot) => push_project_summary(&mut summary, loop_snapshot),
            Err(error) => summary.errors.push(format!("{project_id}: {error}")),
        }
    }

    let _ = agentflow_state::refresh_state(&root);
    Ok(summary)
}

fn push_direct_summary(summary: &mut ProjectLoopRunSummary, snapshot: DirectIssueLoopSummary) {
    summary.direct_issue_count += snapshot.active_issue_ids.len()
        + snapshot.blocked_issue_ids.len()
        + snapshot.done_issue_ids.len();
    summary.active_issue_count += snapshot.active_issue_ids.len();
    summary.blocked_issue_count += snapshot.blocked_issue_ids.len();
    summary.done_issue_count += snapshot.done_issue_ids.len();
    if !snapshot.blockers.is_empty() {
        summary.errors.extend(
            snapshot
                .blockers
                .into_iter()
                .map(|blocker| format!("{}: {}", blocker.code, blocker.reason)),
        );
    }
}

fn push_project_summary(summary: &mut ProjectLoopRunSummary, snapshot: ProjectLoopSnapshot) {
    summary.active_issue_count += snapshot.active_issue_ids.len();
    summary.blocked_issue_count += snapshot.blocked_issue_ids.len();
    summary.done_issue_count += snapshot.done_issue_ids.len();
    summary.projects.push(ProjectLoopProjectSummary {
        project_id: snapshot.project_id,
        status: snapshot.status.as_str().to_string(),
        active_issue_ids: snapshot.active_issue_ids,
        blocked_issue_ids: snapshot.blocked_issue_ids,
        done_issue_ids: snapshot.done_issue_ids,
        blockers: snapshot.blockers,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use agentflow_input::{
        issue::{
            AgentRole, DisplayStatus, InputIssue, InputIssueKind, InputIssueModel,
            InputIssueStatus, InputPriority, InputRiskLevel, InputSystemRecord, IssueCategory,
        },
        project::{InputProject, InputProjectStatus},
        spec_gate::{InputIssueGenerationMode, InputSpecApproval},
    };
    use agentflow_panel::PanelPrepareMode;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn run_project_loop_schedules_backlog_issue_from_desktop_entrypoint() {
        let dir = tempdir().unwrap();
        prepare_fixture_project(dir.path());
        write_approved_spec(dir.path());
        write_project_with_backlog_issue(dir.path());
        agentflow_input::prepare_input_workspace(dir.path()).unwrap();

        let summary = run_project_loop_inner(dir.path()).unwrap();

        assert_eq!(summary.project_count, 1);
        assert_eq!(summary.direct_issue_count, 0);
        assert_eq!(summary.active_issue_count, 1);
        assert_eq!(summary.blocked_issue_count, 0);
        assert!(summary.errors.is_empty());
        assert_eq!(summary.projects[0].status, "executing");
        assert_eq!(summary.projects[0].active_issue_ids, vec!["AF-LOOP-001"]);

        let issue = agentflow_input::load_input_issue(dir.path(), "AF-LOOP-001").unwrap();
        assert_eq!(issue.status, InputIssueStatus::Todo);
        assert_eq!(issue.display_status, DisplayStatus::Todo);
        assert!(dir
            .path()
            .join(".agentflow/panel/context-packs/AF-LOOP-001.json")
            .is_file());
        assert!(dir
            .path()
            .join(".agentflow/state/loops/projects/proj-loop.json")
            .is_file());
        assert!(dir
            .path()
            .join(".agentflow/state/loops/issues/AF-LOOP-001.json")
            .is_file());
    }

    #[test]
    fn run_project_loop_schedules_direct_backlog_issue_from_desktop_entrypoint() {
        let dir = tempdir().unwrap();
        prepare_fixture_project(dir.path());
        write_approved_spec_with_mode(dir.path(), InputIssueGenerationMode::Direct);
        write_direct_backlog_issue(dir.path());
        agentflow_input::prepare_input_workspace(dir.path()).unwrap();

        let summary = run_project_loop_inner(dir.path()).unwrap();

        assert_eq!(summary.project_count, 0);
        assert_eq!(summary.direct_issue_count, 1);
        assert_eq!(summary.active_issue_count, 1);
        assert_eq!(summary.blocked_issue_count, 0);
        assert!(summary.errors.is_empty());

        let issue = agentflow_input::load_input_issue(dir.path(), "AF-DIRECT-001").unwrap();
        assert_eq!(issue.status, InputIssueStatus::Todo);
        assert_eq!(issue.display_status, DisplayStatus::Todo);
        assert!(dir
            .path()
            .join(".agentflow/panel/context-packs/AF-DIRECT-001.json")
            .is_file());
        assert!(dir
            .path()
            .join(".agentflow/state/loops/issues/AF-DIRECT-001.json")
            .is_file());
    }

    fn prepare_fixture_project(root: &Path) {
        fs::create_dir_all(root.join("apps/desktop/src")).unwrap();
        fs::write(root.join("README.md"), "# AgentFlow fixture\n").unwrap();
        fs::write(
            root.join("apps/desktop/src/App.tsx"),
            "export function App() { return null; }\n",
        )
        .unwrap();
        agentflow_agent_manual::prepare_agent_working_manual(root).unwrap();
        agentflow_panel::prepare_project_panel(root, PanelPrepareMode::Blocking).unwrap();
        agentflow_input::prepare_input_workspace(root).unwrap();
    }

    fn write_approved_spec(root: &Path) {
        write_approved_spec_with_mode(root, InputIssueGenerationMode::Project);
    }

    fn write_approved_spec_with_mode(root: &Path, issue_generation_mode: InputIssueGenerationMode) {
        let spec_dir = root.join(".agentflow/input/specs/approved/spec-loop");
        fs::create_dir_all(&spec_dir).unwrap();
        fs::write(spec_dir.join("product.md"), "# Product\n").unwrap();
        fs::write(spec_dir.join("tech.md"), "# Tech\n").unwrap();
        fs::write(spec_dir.join("spec.json"), "{}\n").unwrap();
        fs::write(
            spec_dir.join("approval.json"),
            serde_json::to_string_pretty(&InputSpecApproval {
                spec_id: "spec-loop".to_string(),
                issue_generation_mode,
                ..InputSpecApproval::default()
            })
            .unwrap(),
        )
        .unwrap();
    }

    fn write_direct_backlog_issue(root: &Path) {
        let mut issue = InputIssue {
            issue_id: "AF-DIRECT-001".to_string(),
            issue_model: InputIssueModel::Direct,
            issue_category: IssueCategory::Spec,
            required_agent_role: AgentRole::BuildAgent,
            source_spec_id: "spec-loop".to_string(),
            project_id: None,
            title: "验证 Direct Issue Loop 调度".to_string(),
            summary: "把 direct backlog 任务推进到 todo，并生成上下文包。".to_string(),
            kind: InputIssueKind::Validation,
            priority: InputPriority::P2,
            status: InputIssueStatus::Backlog,
            display_status: DisplayStatus::Backlog,
            execution_risk: InputRiskLevel::Low,
            scope: vec!["apps/desktop/src/**".to_string()],
            acceptance_criteria: vec!["任务状态变成 todo".to_string()],
            validation_hints: vec!["npm --prefix apps/desktop run build".to_string()],
            system: InputSystemRecord {
                created_by: "test".to_string(),
                created_at: 1,
                updated_at: 1,
                path: ".agentflow/input/issues/AF-DIRECT-001.json".to_string(),
                revision: 1,
            },
            ..InputIssue::default()
        };
        issue.normalize_execution_metadata();

        fs::write(
            root.join(".agentflow/input/issues/AF-DIRECT-001.json"),
            serde_json::to_string_pretty(&issue).unwrap(),
        )
        .unwrap();
    }

    fn write_project_with_backlog_issue(root: &Path) {
        let project = InputProject {
            project_id: "proj-loop".to_string(),
            source_spec_id: "spec-loop".to_string(),
            title: "Loop fixture project".to_string(),
            summary: "验证 Project Loop 可以调度任务。".to_string(),
            objective: "验证 Project Loop 可以调度任务。".to_string(),
            issue_ids: vec!["AF-LOOP-001".to_string()],
            status: InputProjectStatus::Planned,
            system: InputSystemRecord {
                created_by: "test".to_string(),
                created_at: 1,
                updated_at: 1,
                path: ".agentflow/input/projects/proj-loop.json".to_string(),
                revision: 1,
            },
            ..InputProject::default()
        };
        let mut issue = InputIssue {
            issue_id: "AF-LOOP-001".to_string(),
            issue_model: InputIssueModel::Project,
            issue_category: IssueCategory::Spec,
            required_agent_role: AgentRole::BuildAgent,
            source_spec_id: "spec-loop".to_string(),
            project_id: Some("proj-loop".to_string()),
            title: "验证 Project Loop 调度".to_string(),
            summary: "把 backlog 任务推进到 todo，并生成上下文包。".to_string(),
            kind: InputIssueKind::Validation,
            priority: InputPriority::P2,
            status: InputIssueStatus::Backlog,
            display_status: DisplayStatus::Backlog,
            execution_risk: InputRiskLevel::Low,
            scope: vec!["apps/desktop/src/**".to_string()],
            acceptance_criteria: vec!["任务状态变成 todo".to_string()],
            validation_hints: vec!["npm --prefix apps/desktop run build".to_string()],
            system: InputSystemRecord {
                created_by: "test".to_string(),
                created_at: 1,
                updated_at: 1,
                path: ".agentflow/input/issues/AF-LOOP-001.json".to_string(),
                revision: 1,
            },
            ..InputIssue::default()
        };
        issue.normalize_execution_metadata();

        fs::write(
            root.join(".agentflow/input/projects/proj-loop.json"),
            serde_json::to_string_pretty(&project).unwrap(),
        )
        .unwrap();
        fs::write(
            root.join(".agentflow/input/issues/AF-LOOP-001.json"),
            serde_json::to_string_pretty(&issue).unwrap(),
        )
        .unwrap();
    }
}
