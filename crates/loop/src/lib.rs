pub mod audit_gate;
pub mod direct_issue_loop;
pub mod error;
pub mod events;
pub mod issue_loop;
pub mod model;
pub mod project_loop;
pub mod storage;

pub use audit_gate::ProjectAuditGate;
pub use direct_issue_loop::{DirectIssueLoop, DirectIssueLoopSummary};
pub use issue_loop::IssueLoop;
pub use model::{
    AuditGateKind, AuditGateStatus, IssueLoopProjection, IssueLoopStage, LoopBlocker,
    ProjectLoopSnapshot, ProjectLoopStatus, LOOP_ISSUE_PROJECTION_VERSION,
    LOOP_PROJECT_SNAPSHOT_VERSION,
};
pub use project_loop::ProjectLoop;

#[cfg(test)]
mod tests {
    use super::*;
    use agentflow_execute::{
        acquire_execute_lease, apply_execute_patch, create_execute_checkpoint,
        prepare_release_delivery, run_execute_command, validate_execute_run, write_execute_plan,
        ExecuteCommandRequest, ExecutePlanDraft,
    };
    use agentflow_input::{
        issue::{
            AgentRole, InputIssue, InputIssueModel, InputIssueStatus, InputRiskLevel, IssueCategory,
        },
        project::{InputProject, InputProjectStatus},
        spec_gate::{InputIssueGenerationMode, InputSpecApproval},
    };
    use std::{fs, path::Path};
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
        assert!(projection.run_id.is_some());
        let issue = agentflow_input::load_input_issue(dir.path(), "AF-001").unwrap();
        assert_eq!(issue.status, InputIssueStatus::InProgress);
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
