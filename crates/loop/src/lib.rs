pub mod audit_gate;
pub mod direct_issue_loop;
pub mod error;
pub mod events;
pub mod issue_loop;
pub mod model;
pub mod project_executor;
pub mod project_loop;
pub mod storage;

pub use audit_gate::ProjectAuditGate;
pub use direct_issue_loop::{DirectIssueLoop, DirectIssueLoopSummary};
pub use issue_loop::{write_issue_merge_proof, IssueLoop};
pub use model::{
    AuditGateKind, AuditGateStatus, IssueLoopProjection, IssueLoopStage, LoopBlocker,
    ProjectLoopSnapshot, ProjectLoopStatus, LOOP_ISSUE_PROJECTION_VERSION,
    LOOP_PROJECT_SNAPSHOT_VERSION,
};
pub use project_executor::{ProjectExecutionLaunch, ProjectExecutionTick, ProjectExecutor};
pub use project_loop::ProjectLoop;

#[cfg(test)]
mod tests {
    use super::*;
    use agentflow_execute::{
        acquire_execute_lease, apply_execute_patch, create_execute_checkpoint, load_execute_run,
        prepare_release_delivery, run_execute_command, validate_execute_run, write_execute_plan,
        ExecuteCommandRequest, ExecutePlanDraft, ExecuteRunStatus,
    };
    use agentflow_input::{
        issue::{
            AgentRole, InputIssue, InputIssueModel, InputIssueStatus, InputRiskLevel, IssueCategory,
        },
        project::{InputProject, InputProjectStatus},
        spec_gate::{InputIssueGenerationMode, InputSpecApproval},
    };
    use std::{fs, path::Path, process::Command};
    use tempfile::tempdir;

    fn prepare_root(root: &Path) {
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("README.md"), "# fixture\n").unwrap();
        fs::write(
            root.join("src/lib.rs"),
            "pub fn value() -> u8 {\n    1\n}\n",
        )
        .unwrap();
        agentflow_agent_manual::prepare_agent_working_manual(root).unwrap();
        agentflow_panel::prepare_project_panel(root, agentflow_panel::PanelPrepareMode::Blocking)
            .unwrap();
        agentflow_input::prepare_input_workspace(root).unwrap();
        agentflow_execute::prepare_execute_workspace(root).unwrap();
    }

    fn init_clean_git_repo(root: &Path) {
        fs::write(root.join(".gitignore"), ".agentflow/\nAGENTS.md\n").unwrap();
        Command::new("git")
            .args(["init", "-b", "main"])
            .current_dir(root)
            .output()
            .unwrap();
        Command::new("git")
            .args(["add", ".gitignore", "README.md", "src/lib.rs"])
            .current_dir(root)
            .output()
            .unwrap();
        let output = Command::new("git")
            .args([
                "-c",
                "user.name=AgentFlow Test",
                "-c",
                "user.email=agentflow-test@example.com",
                "commit",
                "-m",
                "initial",
            ])
            .current_dir(root)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    fn write_approved_spec(root: &Path) {
        let spec_dir = root.join(".agentflow/input/specs/approved/spec-001");
        fs::create_dir_all(&spec_dir).unwrap();
        fs::write(spec_dir.join("product.md"), "# Product\n").unwrap();
        fs::write(spec_dir.join("tech.md"), "# Tech\n").unwrap();
        fs::write(spec_dir.join("spec.json"), "{}\n").unwrap();
        fs::write(
            spec_dir.join("approval.json"),
            serde_json::to_string_pretty(&InputSpecApproval {
                spec_id: "spec-001".to_string(),
                issue_generation_mode: InputIssueGenerationMode::Project,
                ..InputSpecApproval::default()
            })
            .unwrap(),
        )
        .unwrap();
    }

    fn write_project_issue(root: &Path, status: InputIssueStatus) {
        let project = InputProject {
            project_id: "proj-001".to_string(),
            source_spec_id: "spec-001".to_string(),
            title: "Project Loop fixture".to_string(),
            summary: "Schedule one issue.".to_string(),
            objective: "Schedule one issue.".to_string(),
            issue_ids: vec!["AF-001".to_string()],
            status: InputProjectStatus::Planned,
            ..InputProject::default()
        };
        let issue = InputIssue {
            issue_id: "AF-001".to_string(),
            issue_model: InputIssueModel::Project,
            issue_category: IssueCategory::Spec,
            required_agent_role: AgentRole::BuildAgent,
            source_spec_id: "spec-001".to_string(),
            project_id: Some("proj-001".to_string()),
            title: "Implement Project Loop fixture".to_string(),
            summary: "Update src/lib.rs.".to_string(),
            status,
            execution_risk: InputRiskLevel::Low,
            scope: vec!["src/lib.rs".to_string()],
            validation_hints: vec!["printf ok".to_string()],
            ..InputIssue::default()
        };
        fs::write(
            root.join(".agentflow/input/projects/proj-001.json"),
            serde_json::to_string_pretty(&project).unwrap(),
        )
        .unwrap();
        fs::write(
            root.join(".agentflow/input/issues/AF-001.json"),
            serde_json::to_string_pretty(&issue).unwrap(),
        )
        .unwrap();
    }

    fn write_project_issue_chain(root: &Path) {
        let project = InputProject {
            project_id: "proj-001".to_string(),
            source_spec_id: "spec-001".to_string(),
            title: "Project Loop chain fixture".to_string(),
            summary: "Schedule dependent issues.".to_string(),
            objective: "Schedule dependent issues.".to_string(),
            issue_ids: vec!["AF-001".to_string(), "AF-002".to_string()],
            status: InputProjectStatus::Planned,
            ..InputProject::default()
        };
        let mut first = InputIssue {
            issue_id: "AF-001".to_string(),
            issue_model: InputIssueModel::Project,
            issue_category: IssueCategory::Spec,
            required_agent_role: AgentRole::BuildAgent,
            source_spec_id: "spec-001".to_string(),
            project_id: Some("proj-001".to_string()),
            title: "First issue".to_string(),
            summary: "Run first.".to_string(),
            status: InputIssueStatus::Backlog,
            execution_risk: InputRiskLevel::Low,
            scope: vec!["src/lib.rs".to_string()],
            validation_hints: vec!["printf ok".to_string()],
            ..InputIssue::default()
        };
        first.normalize_execution_metadata();

        let mut second = InputIssue {
            issue_id: "AF-002".to_string(),
            issue_model: InputIssueModel::Project,
            issue_category: IssueCategory::Spec,
            required_agent_role: AgentRole::BuildAgent,
            source_spec_id: "spec-001".to_string(),
            project_id: Some("proj-001".to_string()),
            title: "Second issue".to_string(),
            summary: "Run after first.".to_string(),
            status: InputIssueStatus::Backlog,
            execution_risk: InputRiskLevel::Low,
            scope: vec!["src/lib.rs".to_string()],
            validation_hints: vec!["printf ok".to_string()],
            ..InputIssue::default()
        };
        second.relations.blocked_by = vec!["AF-001".to_string()];
        second.normalize_execution_metadata();

        fs::write(
            root.join(".agentflow/input/projects/proj-001.json"),
            serde_json::to_string_pretty(&project).unwrap(),
        )
        .unwrap();
        fs::write(
            root.join(".agentflow/input/issues/AF-001.json"),
            serde_json::to_string_pretty(&first).unwrap(),
        )
        .unwrap();
        fs::write(
            root.join(".agentflow/input/issues/AF-002.json"),
            serde_json::to_string_pretty(&second).unwrap(),
        )
        .unwrap();
    }

    fn write_project_issue_with_invalid_role(root: &Path, status: InputIssueStatus) {
        let project = InputProject {
            project_id: "proj-001".to_string(),
            source_spec_id: "spec-001".to_string(),
            title: "Project Loop invalid-role fixture".to_string(),
            summary: "Schedule one invalid issue.".to_string(),
            objective: "Schedule one invalid issue.".to_string(),
            issue_ids: vec!["AF-001".to_string()],
            status: InputProjectStatus::Planned,
            ..InputProject::default()
        };
        let mut issue = InputIssue {
            issue_id: "AF-001".to_string(),
            issue_model: InputIssueModel::Project,
            issue_category: IssueCategory::Spec,
            required_agent_role: AgentRole::AuditAgent,
            source_spec_id: "spec-001".to_string(),
            project_id: Some("proj-001".to_string()),
            title: "Invalid role issue".to_string(),
            summary: "Should stay blocked.".to_string(),
            status,
            execution_risk: InputRiskLevel::Low,
            scope: vec!["src/lib.rs".to_string()],
            validation_hints: vec!["printf ok".to_string()],
            ..InputIssue::default()
        };
        issue.normalize_execution_metadata();
        fs::write(
            root.join(".agentflow/input/projects/proj-001.json"),
            serde_json::to_string_pretty(&project).unwrap(),
        )
        .unwrap();
        fs::write(
            root.join(".agentflow/input/issues/AF-001.json"),
            serde_json::to_string_pretty(&issue).unwrap(),
        )
        .unwrap();
    }

    fn mark_project_preflight_ready(root: &Path) {
        let mut snapshot = ProjectLoop::new("proj-001").snapshot(100);
        snapshot.status = ProjectLoopStatus::Active;
        storage::write_project_loop_snapshot(root, &snapshot).unwrap();
    }

    fn schedule_issue(root: &Path) {
        write_approved_spec(root);
        write_project_issue(root, InputIssueStatus::Backlog);
        agentflow_input::prepare_input_workspace(root).unwrap();
        mark_project_preflight_ready(root);

        let snapshot = ProjectLoop::new("proj-001")
            .schedule_ready_issues(root)
            .unwrap();

        assert_eq!(snapshot.active_issue_ids, vec!["AF-001"]);
        let issue = agentflow_input::load_input_issue(root, "AF-001").unwrap();
        assert_eq!(issue.status, InputIssueStatus::Todo);
        assert!(root
            .join(".agentflow/panel/context-packs/AF-001.json")
            .is_file());
    }

    #[test]
    fn project_preflight_does_not_block_on_missing_git_remote() {
        let dir = tempdir().unwrap();
        prepare_root(dir.path());
        write_approved_spec(dir.path());
        write_project_issue(dir.path(), InputIssueStatus::Backlog);
        agentflow_input::prepare_input_workspace(dir.path()).unwrap();

        let snapshot = ProjectLoop::new("proj-001")
            .run_preflight(dir.path())
            .unwrap();

        assert_eq!(snapshot.status, ProjectLoopStatus::Active);
        assert!(snapshot.blockers.is_empty());
    }

    #[test]
    fn project_scheduler_moves_backlog_issue_to_todo() {
        let dir = tempdir().unwrap();
        prepare_root(dir.path());

        schedule_issue(dir.path());
    }

    #[test]
    fn project_scheduler_keeps_dependency_waiting_issue_in_backlog() {
        let dir = tempdir().unwrap();
        prepare_root(dir.path());
        write_approved_spec(dir.path());
        write_project_issue_chain(dir.path());
        agentflow_input::prepare_input_workspace(dir.path()).unwrap();
        mark_project_preflight_ready(dir.path());

        let snapshot = ProjectLoop::new("proj-001")
            .schedule_ready_issues(dir.path())
            .unwrap();

        assert_eq!(snapshot.active_issue_ids, vec!["AF-001"]);
        assert!(snapshot.blocked_issue_ids.is_empty());

        let first = agentflow_input::load_input_issue(dir.path(), "AF-001").unwrap();
        let second = agentflow_input::load_input_issue(dir.path(), "AF-002").unwrap();
        assert_eq!(first.status, InputIssueStatus::Todo);
        assert_eq!(second.status, InputIssueStatus::Backlog);

        let second_projection = storage::read_issue_loop_projection(dir.path(), "AF-002").unwrap();
        assert_eq!(second_projection.stage, IssueLoopStage::Backlog);
        assert_eq!(
            second_projection.display_status,
            Some(agentflow_input::issue::DisplayStatus::Backlog)
        );
        assert_eq!(second_projection.blockers.len(), 1);
        assert_eq!(second_projection.blockers[0].code, "dependency-not-done");
    }

    #[test]
    fn project_scheduler_does_not_rewrite_waiting_backlog_issue_or_active_project_when_status_unchanged(
    ) {
        let dir = tempdir().unwrap();
        prepare_root(dir.path());
        write_approved_spec(dir.path());
        write_project_issue_chain(dir.path());
        agentflow_input::prepare_input_workspace(dir.path()).unwrap();
        mark_project_preflight_ready(dir.path());

        ProjectLoop::new("proj-001")
            .schedule_ready_issues(dir.path())
            .unwrap();

        let waiting_before = agentflow_input::load_input_issue(dir.path(), "AF-002").unwrap();
        let project_before = agentflow_input::load_input_project(dir.path(), "proj-001").unwrap();

        ProjectLoop::new("proj-001")
            .schedule_ready_issues(dir.path())
            .unwrap();

        let waiting_after = agentflow_input::load_input_issue(dir.path(), "AF-002").unwrap();
        let project_after = agentflow_input::load_input_project(dir.path(), "proj-001").unwrap();

        assert_eq!(waiting_after.status, InputIssueStatus::Backlog);
        assert_eq!(
            waiting_after.system.revision,
            waiting_before.system.revision
        );
        assert_eq!(project_after.status, InputProjectStatus::Active);
        assert_eq!(
            project_after.system.revision,
            project_before.system.revision
        );
    }

    #[test]
    fn project_scheduler_preserves_blocker_reasons_for_true_blocked_issue() {
        let dir = tempdir().unwrap();
        prepare_root(dir.path());
        write_approved_spec(dir.path());
        write_project_issue_with_invalid_role(dir.path(), InputIssueStatus::Backlog);
        agentflow_input::prepare_input_workspace(dir.path()).unwrap();
        mark_project_preflight_ready(dir.path());

        ProjectLoop::new("proj-001")
            .schedule_ready_issues(dir.path())
            .unwrap();
        let first_projection = storage::read_issue_loop_projection(dir.path(), "AF-001").unwrap();
        assert_eq!(first_projection.stage, IssueLoopStage::Blocked);
        assert!(first_projection
            .blockers
            .iter()
            .any(|blocker| blocker.code == "build-agent-contract-invalid"));

        let snapshot = ProjectLoop::new("proj-001")
            .schedule_ready_issues(dir.path())
            .unwrap();
        assert_eq!(snapshot.blocked_issue_ids, vec!["AF-001"]);

        let second_projection = storage::read_issue_loop_projection(dir.path(), "AF-001").unwrap();
        assert_eq!(second_projection.stage, IssueLoopStage::Blocked);
        assert!(second_projection
            .blockers
            .iter()
            .any(|blocker| blocker.code == "build-agent-contract-invalid"));
    }

    #[test]
    fn project_scheduler_recovers_retryable_branch_block_to_todo() {
        let dir = tempdir().unwrap();
        prepare_root(dir.path());
        init_clean_git_repo(dir.path());
        write_approved_spec(dir.path());
        write_project_issue(dir.path(), InputIssueStatus::Blocked);
        let mut issue = agentflow_input::load_input_issue(dir.path(), "AF-001").unwrap();
        issue.latest_run_id = Some("run-001".to_string());
        fs::write(
            dir.path().join(".agentflow/input/issues/AF-001.json"),
            serde_json::to_string_pretty(&issue).unwrap(),
        )
        .unwrap();
        fs::create_dir_all(dir.path().join(".agentflow/execute/runs/run-001")).unwrap();
        fs::write(
            dir.path()
                .join(".agentflow/execute/runs/run-001/branch.json"),
            serde_json::to_string_pretty(&serde_json::json!({
                "version": "execute-branch-check.v1",
                "runId": "run-001",
                "issueId": "AF-001",
                "projectId": "proj-001",
                "baseBranch": "main",
                "issueBranch": "agentflow/proj-001/AF-001",
                "currentBranchBefore": "main",
                "currentBranchAfter": "main",
                "status": "blocked",
                "blockedReason": "current branch does not match issue branch and worktree has uncommitted changes"
            }))
            .unwrap(),
        )
        .unwrap();
        agentflow_input::prepare_input_workspace(dir.path()).unwrap();
        mark_project_preflight_ready(dir.path());

        let snapshot = ProjectLoop::new("proj-001")
            .schedule_ready_issues(dir.path())
            .unwrap();

        assert_eq!(snapshot.active_issue_ids, vec!["AF-001"]);
        assert!(snapshot.blocked_issue_ids.is_empty());
        let issue = agentflow_input::load_input_issue(dir.path(), "AF-001").unwrap();
        assert_eq!(issue.status, InputIssueStatus::Todo);
        let projection = storage::read_issue_loop_projection(dir.path(), "AF-001").unwrap();
        assert_eq!(projection.stage, IssueLoopStage::Todo);
    }

    #[test]
    fn project_scheduler_preserves_runtime_projection_fields_after_done() {
        let dir = tempdir().unwrap();
        prepare_root(dir.path());
        write_approved_spec(dir.path());
        write_project_issue(dir.path(), InputIssueStatus::Done);
        let mut issue = agentflow_input::load_input_issue(dir.path(), "AF-001").unwrap();
        issue.latest_run_id = Some("run-001".to_string());
        fs::write(
            dir.path().join(".agentflow/input/issues/AF-001.json"),
            serde_json::to_string_pretty(&issue).unwrap(),
        )
        .unwrap();
        fs::create_dir_all(dir.path().join(".agentflow/execute/runs/run-001")).unwrap();
        fs::write(
            dir.path()
                .join(".agentflow/execute/runs/run-001/branch.json"),
            serde_json::to_string_pretty(&serde_json::json!({
                "issueBranch": "agentflow/proj-001/AF-001"
            }))
            .unwrap(),
        )
        .unwrap();
        agentflow_input::prepare_input_workspace(dir.path()).unwrap();
        mark_project_preflight_ready(dir.path());

        let snapshot = ProjectLoop::new("proj-001")
            .schedule_ready_issues(dir.path())
            .unwrap();

        assert_eq!(snapshot.done_issue_ids, vec!["AF-001"]);
        let projection = storage::read_issue_loop_projection(dir.path(), "AF-001").unwrap();
        assert_eq!(projection.stage, IssueLoopStage::Done);
        assert_eq!(
            projection.display_status,
            Some(agentflow_input::issue::DisplayStatus::Done)
        );
        assert_eq!(projection.run_id.as_deref(), Some("run-001"));
        assert_eq!(
            projection.branch_name.as_deref(),
            Some("agentflow/proj-001/AF-001")
        );
        assert_eq!(projection.review_substate.as_deref(), Some("merged"));
    }

    #[test]
    fn direct_issue_loop_moves_backlog_issue_to_todo() {
        let dir = tempdir().unwrap();
        prepare_root(dir.path());
        write_approved_spec(dir.path());
        let mut issue = InputIssue {
            issue_id: "AF-DIRECT-001".to_string(),
            issue_model: InputIssueModel::Direct,
            issue_category: IssueCategory::Spec,
            required_agent_role: AgentRole::BuildAgent,
            source_spec_id: "spec-001".to_string(),
            project_id: None,
            title: "Direct Issue Loop fixture".to_string(),
            summary: "Schedule one direct issue.".to_string(),
            status: InputIssueStatus::Backlog,
            execution_risk: InputRiskLevel::Low,
            scope: vec!["src/lib.rs".to_string()],
            validation_hints: vec!["printf ok".to_string()],
            ..InputIssue::default()
        };
        issue.normalize_execution_metadata();
        fs::write(
            dir.path()
                .join(".agentflow/input/issues/AF-DIRECT-001.json"),
            serde_json::to_string_pretty(&issue).unwrap(),
        )
        .unwrap();
        agentflow_input::prepare_input_workspace(dir.path()).unwrap();

        let summary = DirectIssueLoop::schedule_ready_issues(dir.path()).unwrap();

        assert_eq!(summary.active_issue_ids, vec!["AF-DIRECT-001"]);
        let issue = agentflow_input::load_input_issue(dir.path(), "AF-DIRECT-001").unwrap();
        assert_eq!(issue.status, InputIssueStatus::Todo);
        assert!(dir
            .path()
            .join(".agentflow/panel/context-packs/AF-DIRECT-001.json")
            .is_file());
        let projection = storage::read_issue_loop_projection(dir.path(), "AF-DIRECT-001").unwrap();
        assert_eq!(projection.project_id, None);
        assert_eq!(projection.stage, IssueLoopStage::Todo);
        assert_eq!(
            projection.display_status,
            Some(agentflow_input::issue::DisplayStatus::Todo)
        );
    }

    #[test]
    fn direct_issue_loop_preserves_runtime_projection_fields_after_done() {
        let dir = tempdir().unwrap();
        prepare_root(dir.path());
        write_approved_spec(dir.path());
        let mut issue = InputIssue {
            issue_id: "AF-DIRECT-001".to_string(),
            issue_model: InputIssueModel::Direct,
            issue_category: IssueCategory::Spec,
            required_agent_role: AgentRole::BuildAgent,
            source_spec_id: "spec-001".to_string(),
            project_id: None,
            title: "Direct Issue Loop fixture".to_string(),
            summary: "Schedule one direct issue.".to_string(),
            status: InputIssueStatus::Done,
            latest_run_id: Some("run-001".to_string()),
            execution_risk: InputRiskLevel::Low,
            scope: vec!["src/lib.rs".to_string()],
            validation_hints: vec!["printf ok".to_string()],
            ..InputIssue::default()
        };
        issue.normalize_execution_metadata();
        fs::write(
            dir.path()
                .join(".agentflow/input/issues/AF-DIRECT-001.json"),
            serde_json::to_string_pretty(&issue).unwrap(),
        )
        .unwrap();
        fs::create_dir_all(dir.path().join(".agentflow/execute/runs/run-001")).unwrap();
        fs::write(
            dir.path()
                .join(".agentflow/execute/runs/run-001/branch.json"),
            serde_json::to_string_pretty(&serde_json::json!({
                "issueBranch": "agentflow/direct/AF-DIRECT-001"
            }))
            .unwrap(),
        )
        .unwrap();
        agentflow_input::prepare_input_workspace(dir.path()).unwrap();

        let summary = DirectIssueLoop::schedule_ready_issues(dir.path()).unwrap();

        assert_eq!(summary.done_issue_ids, vec!["AF-DIRECT-001"]);
        let projection = storage::read_issue_loop_projection(dir.path(), "AF-DIRECT-001").unwrap();
        assert_eq!(projection.stage, IssueLoopStage::Done);
        assert_eq!(
            projection.display_status,
            Some(agentflow_input::issue::DisplayStatus::Done)
        );
        assert_eq!(projection.run_id.as_deref(), Some("run-001"));
        assert_eq!(
            projection.branch_name.as_deref(),
            Some("agentflow/direct/AF-DIRECT-001")
        );
        assert_eq!(projection.review_substate.as_deref(), Some("merged"));
    }

    #[test]
    fn issue_loop_starts_todo_issue_and_writes_branch_projection() {
        let dir = tempdir().unwrap();
        prepare_root(dir.path());
        schedule_issue(dir.path());

        let projection = IssueLoop::new("proj-001", "AF-001")
            .start_runtime_preflight(dir.path())
            .unwrap();

        assert_eq!(projection.stage, IssueLoopStage::InProgress);
        assert_eq!(
            projection.display_status,
            Some(agentflow_input::issue::DisplayStatus::InProgress)
        );
        assert!(projection.run_id.is_some());
        let issue = agentflow_input::load_input_issue(dir.path(), "AF-001").unwrap();
        assert_eq!(issue.status, InputIssueStatus::InProgress);
        let run = load_execute_run(dir.path(), projection.run_id.clone().unwrap()).unwrap();
        assert_eq!(run.status, ExecuteRunStatus::Planned);
        assert!(dir
            .path()
            .join(format!(
                ".agentflow/execute/runs/{}/branch.json",
                projection.run_id.unwrap()
            ))
            .is_file());
    }

    #[test]
    fn project_executor_starts_first_todo_issue_runtime() {
        let dir = tempdir().unwrap();
        prepare_root(dir.path());
        write_approved_spec(dir.path());
        write_project_issue(dir.path(), InputIssueStatus::Backlog);
        agentflow_input::prepare_input_workspace(dir.path()).unwrap();

        let tick = ProjectExecutor::new("proj-001").tick(dir.path()).unwrap();

        assert_eq!(tick.snapshot.status, ProjectLoopStatus::Executing);
        assert_eq!(tick.snapshot.active_issue_ids, vec!["AF-001"]);
        let launch = tick.launch.expect("expected runtime launch");
        assert_eq!(launch.issue_id, "AF-001");
        assert_eq!(launch.stage, IssueLoopStage::InProgress);

        let issue = agentflow_input::load_input_issue(dir.path(), "AF-001").unwrap();
        assert_eq!(issue.status, InputIssueStatus::InProgress);
        assert_eq!(issue.latest_run_id.as_deref(), Some(launch.run_id.as_str()));
        let run = load_execute_run(dir.path(), launch.run_id.clone()).unwrap();
        assert_eq!(run.status, ExecuteRunStatus::Planned);
        let launch_request_path = dir.path().join(&launch.launch_request_path);
        assert!(launch_request_path.is_file());
        let launch_request: serde_json::Value =
            agentflow_execute::storage::read_json(&launch_request_path).unwrap();
        assert_eq!(
            launch_request
                .get("issueId")
                .and_then(serde_json::Value::as_str),
            Some("AF-001")
        );
        assert_eq!(
            launch_request
                .get("runId")
                .and_then(serde_json::Value::as_str),
            Some(launch.run_id.as_str())
        );
        let events = agentflow_workflow_events::load_events(dir.path()).unwrap();
        assert!(events.iter().any(|event| {
            event.event_type == agentflow_workflow_events::EVENT_TYPE_BUILD_AGENT_LAUNCH_REQUESTED
                && event.subject_id == "AF-001"
        }));
        let pending = agentflow_workflow_events::load_pending_events(
            dir.path(),
            agentflow_workflow_events::CONSUMER_BUILD_AGENT,
            &[agentflow_workflow_events::EVENT_TYPE_BUILD_AGENT_LAUNCH_REQUESTED],
        )
        .unwrap();
        assert_eq!(pending.len(), 1);
        let stored_snapshot =
            storage::read_project_loop_snapshot(dir.path(), "proj-001").expect("project snapshot");
        assert_eq!(stored_snapshot.status, ProjectLoopStatus::Executing);
        assert_eq!(stored_snapshot.active_issue_ids, vec!["AF-001"]);
        assert_eq!(stored_snapshot.runtime_issue_id.as_deref(), Some("AF-001"));
        assert_eq!(
            stored_snapshot.runtime_run_id.as_deref(),
            Some(launch.run_id.as_str())
        );
        assert_eq!(
            stored_snapshot.runtime_stage,
            Some(IssueLoopStage::InProgress)
        );
        assert_eq!(
            stored_snapshot.runtime_launch_request_path.as_deref(),
            Some(launch.launch_request_path.as_str())
        );
    }

    #[test]
    fn project_executor_starts_next_issue_after_previous_done() {
        let dir = tempdir().unwrap();
        prepare_root(dir.path());
        write_approved_spec(dir.path());
        write_project_issue_chain(dir.path());
        agentflow_input::prepare_input_workspace(dir.path()).unwrap();

        let first_tick = ProjectExecutor::new("proj-001").tick(dir.path()).unwrap();
        let first_launch = first_tick.launch.expect("expected first runtime launch");
        assert_eq!(first_launch.issue_id, "AF-001");

        agentflow_input::update_input_issue_status(dir.path(), "AF-001", InputIssueStatus::Done)
            .unwrap();
        let mut first_issue = agentflow_input::load_input_issue(dir.path(), "AF-001").unwrap();
        first_issue.latest_run_id = Some(first_launch.run_id.clone());
        fs::write(
            dir.path().join(".agentflow/input/issues/AF-001.json"),
            serde_json::to_string_pretty(&first_issue).unwrap(),
        )
        .unwrap();
        agentflow_input::prepare_input_workspace(dir.path()).unwrap();

        let second_tick = ProjectExecutor::new("proj-001").tick(dir.path()).unwrap();
        let second_launch = second_tick.launch.expect("expected second runtime launch");
        assert_eq!(second_launch.issue_id, "AF-002");
        assert_eq!(second_launch.stage, IssueLoopStage::InProgress);

        let second_issue = agentflow_input::load_input_issue(dir.path(), "AF-002").unwrap();
        assert_eq!(second_issue.status, InputIssueStatus::InProgress);
    }

    #[test]
    fn project_executor_keeps_snapshot_blocked_when_runtime_preflight_blocks() {
        let dir = tempdir().unwrap();
        prepare_root(dir.path());
        init_clean_git_repo(dir.path());
        write_approved_spec(dir.path());
        write_project_issue(dir.path(), InputIssueStatus::Backlog);
        fs::write(dir.path().join("README.md"), "# dirty\n").unwrap();
        agentflow_input::prepare_input_workspace(dir.path()).unwrap();

        let tick = ProjectExecutor::new("proj-001").tick(dir.path()).unwrap();

        assert!(tick.launch.is_none());
        assert_eq!(tick.snapshot.status, ProjectLoopStatus::Blocked);
        assert_eq!(tick.snapshot.blocked_issue_ids, vec!["AF-001"]);
        let issue = agentflow_input::load_input_issue(dir.path(), "AF-001").unwrap();
        assert_eq!(issue.status, InputIssueStatus::Blocked);
        assert!(issue.latest_run_id.is_some());
        let launch_request_glob = dir
            .path()
            .join(".agentflow/execute/runs")
            .join(issue.latest_run_id.unwrap())
            .join("launcher/build-agent-request.json");
        assert!(!launch_request_glob.is_file());
        let pending = agentflow_workflow_events::load_pending_events(
            dir.path(),
            agentflow_workflow_events::CONSUMER_BUILD_AGENT,
            &[agentflow_workflow_events::EVENT_TYPE_BUILD_AGENT_LAUNCH_REQUESTED],
        )
        .unwrap();
        assert!(pending.is_empty());
        let stored_snapshot =
            storage::read_project_loop_snapshot(dir.path(), "proj-001").expect("project snapshot");
        assert_eq!(stored_snapshot.status, ProjectLoopStatus::Blocked);
        assert_eq!(stored_snapshot.blocked_issue_ids, vec!["AF-001"]);
    }

    #[test]
    fn project_executor_restores_missing_launch_request_for_active_runtime() {
        let dir = tempdir().unwrap();
        prepare_root(dir.path());
        write_approved_spec(dir.path());
        write_project_issue(dir.path(), InputIssueStatus::Backlog);
        agentflow_input::prepare_input_workspace(dir.path()).unwrap();

        let first_tick = ProjectExecutor::new("proj-001").tick(dir.path()).unwrap();
        let launch = first_tick.launch.expect("expected runtime launch");
        let launch_request_path = dir.path().join(&launch.launch_request_path);
        std::fs::remove_file(&launch_request_path).unwrap();
        assert!(!launch_request_path.exists());

        let second_tick = ProjectExecutor::new("proj-001").tick(dir.path()).unwrap();
        assert!(second_tick.launch.is_none());
        assert!(launch_request_path.is_file());
        assert_eq!(
            second_tick.snapshot.runtime_issue_id.as_deref(),
            Some("AF-001")
        );
        assert_eq!(
            second_tick.snapshot.runtime_run_id.as_deref(),
            Some(launch.run_id.as_str())
        );
        assert_eq!(
            second_tick.snapshot.runtime_stage,
            Some(IssueLoopStage::InProgress)
        );
        assert_eq!(
            second_tick.snapshot.runtime_launch_request_path.as_deref(),
            Some(launch.launch_request_path.as_str())
        );
    }

    #[test]
    fn direct_issue_loop_starts_todo_issue_and_writes_branch_projection() {
        let dir = tempdir().unwrap();
        prepare_root(dir.path());
        write_approved_spec(dir.path());
        let mut issue = InputIssue {
            issue_id: "AF-DIRECT-START-001".to_string(),
            issue_model: InputIssueModel::Direct,
            issue_category: IssueCategory::Spec,
            required_agent_role: AgentRole::BuildAgent,
            source_spec_id: "spec-001".to_string(),
            project_id: None,
            title: "Direct runtime fixture".to_string(),
            summary: "Start runtime preflight.".to_string(),
            status: InputIssueStatus::Backlog,
            execution_risk: InputRiskLevel::Low,
            scope: vec!["src/lib.rs".to_string()],
            validation_hints: vec!["printf ok".to_string()],
            ..InputIssue::default()
        };
        issue.normalize_execution_metadata();
        fs::write(
            dir.path()
                .join(".agentflow/input/issues/AF-DIRECT-START-001.json"),
            serde_json::to_string_pretty(&issue).unwrap(),
        )
        .unwrap();
        agentflow_input::prepare_input_workspace(dir.path()).unwrap();
        DirectIssueLoop::schedule_ready_issues(dir.path()).unwrap();

        let projection =
            DirectIssueLoop::start_runtime_preflight(dir.path(), "AF-DIRECT-START-001").unwrap();

        assert_eq!(projection.stage, IssueLoopStage::InProgress);
        assert_eq!(
            projection.display_status,
            Some(agentflow_input::issue::DisplayStatus::InProgress)
        );
        assert!(projection.run_id.is_some());
        let issue = agentflow_input::load_input_issue(dir.path(), "AF-DIRECT-START-001").unwrap();
        assert_eq!(issue.status, InputIssueStatus::InProgress);
        let run = load_execute_run(dir.path(), projection.run_id.clone().unwrap()).unwrap();
        assert_eq!(run.status, ExecuteRunStatus::Planned);
        assert!(dir
            .path()
            .join(format!(
                ".agentflow/execute/runs/{}/branch.json",
                projection.run_id.unwrap()
            ))
            .is_file());
    }

    #[test]
    fn audit_gate_generates_delivery_and_project_final_reports_without_audit_issue() {
        let dir = tempdir().unwrap();
        prepare_root(dir.path());
        schedule_issue(dir.path());
        let loop_issue = IssueLoop::new("proj-001", "AF-001");
        let projection = loop_issue.start_runtime_preflight(dir.path()).unwrap();
        let run_id = projection.run_id.unwrap();
        acquire_execute_lease(dir.path(), run_id.clone()).unwrap();
        write_execute_plan(
            dir.path(),
            run_id.clone(),
            ExecutePlanDraft {
                steps: Vec::new(),
                allowed_write_paths: vec!["src/lib.rs".to_string()],
                allowed_commands: vec!["printf ok".to_string()],
            },
        )
        .unwrap();
        create_execute_checkpoint(dir.path(), run_id.clone()).unwrap();
        let patch = "diff --git a/src/lib.rs b/src/lib.rs\n--- a/src/lib.rs\n+++ b/src/lib.rs\n@@ -1,3 +1,3 @@\n pub fn value() -> u8 {\n-    1\n+    2\n }\n";
        apply_execute_patch(dir.path(), run_id.clone(), patch.to_string()).unwrap();
        run_execute_command(
            dir.path(),
            run_id.clone(),
            ExecuteCommandRequest {
                label: "printf ok".to_string(),
                program: "printf".to_string(),
                args: vec!["ok".to_string()],
                source: Some("test".to_string()),
            },
        )
        .unwrap();
        validate_execute_run(dir.path(), run_id.clone()).unwrap();
        prepare_release_delivery(dir.path(), run_id.clone()).unwrap();
        loop_issue
            .write_merge_proof(
                dir.path(),
                &run_id,
                "github",
                "auto-merge-if-eligible",
                Some("https://github.com/atxinbao/agentflow/pull/1".to_string()),
                true,
            )
            .unwrap();
        agentflow_input::update_input_issue_status(dir.path(), "AF-001", InputIssueStatus::Done)
            .unwrap();

        let delivery_gate =
            ProjectAuditGate::generate_delivery_audit(dir.path(), "proj-001", "AF-001", &run_id)
                .unwrap();
        assert_eq!(delivery_gate.status, AuditGateStatus::Passed);
        assert!(dir
            .path()
            .join(format!(
                ".agentflow/output/audit/delivery-{run_id}/audit-report.md"
            ))
            .is_file());
        assert!(!dir
            .path()
            .join(format!(".agentflow/input/issues/audit-{run_id}.json"))
            .is_file());

        let project_gate =
            ProjectAuditGate::generate_project_final_audit(dir.path(), "proj-001").unwrap();
        assert_eq!(project_gate.status, AuditGateStatus::Passed);
        assert!(dir
            .path()
            .join(".agentflow/output/audit/project-proj-001-final/audit-report.md")
            .is_file());
        let project = agentflow_input::load_input_project(dir.path(), "proj-001").unwrap();
        assert_eq!(project.status, InputProjectStatus::Done);
    }
}
