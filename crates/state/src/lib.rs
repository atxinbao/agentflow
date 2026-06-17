pub mod model;

mod events;
mod gates;
mod health;
mod indexes;
mod locks;
mod manager;
mod readiness;
mod sessions;
mod storage;

pub use events::{append_state_event, load_state_timeline};
pub use gates::{load_blockers, load_next_actions, load_workflow_gates};
pub use locks::load_state_locks;
pub use manager::{
    load_issue_status_index, load_state_index, load_state_manifest, load_state_status,
    prepare_state_workspace, refresh_state,
};
pub use model::*;
pub use sessions::{load_state_session, update_state_session};

pub fn state_paths() -> std::collections::BTreeMap<String, String> {
    std::collections::BTreeMap::from([
        ("health".to_string(), ".agentflow/state/health".to_string()),
        ("gates".to_string(), ".agentflow/state/gates".to_string()),
        (
            "sessions".to_string(),
            ".agentflow/state/sessions".to_string(),
        ),
        ("locks".to_string(), ".agentflow/state/locks".to_string()),
        ("events".to_string(), ".agentflow/state/events".to_string()),
        (
            "indexes".to_string(),
            ".agentflow/state/indexes".to_string(),
        ),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use agentflow_audit::{request_human_audit, AuditScope, AuditScopeRef, HumanAuditRequestDraft};
    use agentflow_event_store::{append_task_event_once, EventActor, TaskEventDraft};
    use agentflow_spec::{SpecIssueDraft, SpecIssueStatus};
    use serde_json::json;
    use std::{fs, path::Path};
    use tempfile::tempdir;

    fn prepare_layers(root: &Path) {
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("README.md"), "# fixture\n").unwrap();
        fs::write(root.join("src/lib.rs"), "pub fn value() -> u8 { 1 }\n").unwrap();
        agentflow_agent_manual::prepare_agent_working_manual(root).unwrap();
        agentflow_panel::prepare_project_panel(root, agentflow_panel::PanelPrepareMode::Blocking)
            .unwrap();
        agentflow_spec::prepare_spec_workspace(root).unwrap();
        agentflow_projection::prepare_projection_workspace(root).unwrap();
        agentflow_audit::prepare_audit_workspace(root).unwrap();
    }

    fn write_issue(root: &Path, status: SpecIssueStatus) -> agentflow_spec::SpecIssue {
        let requirement = root.join("docs/requirements/spec-001.md");
        fs::create_dir_all(requirement.parent().unwrap()).unwrap();
        fs::write(&requirement, "# Fixture\n\nFixture requirement.\n").unwrap();
        let mut draft = SpecIssueDraft::new("iss-001");
        draft.title = Some("Fixture issue".to_string());
        draft.validation_commands = vec!["printf ok".to_string()];
        let mut issue = agentflow_spec::issue_from_requirement(root, &requirement, draft).unwrap();
        issue.status = status;
        agentflow_spec::write_spec_issue(root, &issue).unwrap();
        issue
    }

    fn append_issue_event(root: &Path, event_type: &str, state: Option<&str>) {
        append_task_event_once(
            root,
            TaskEventDraft {
                flow_type: agentflow_workflow_core::WorkflowFlowType::Work,
                aggregate_type: "issue".to_string(),
                aggregate_id: "iss-001".to_string(),
                project_id: None,
                issue_id: Some("iss-001".to_string()),
                run_id: Some("run-001".to_string()),
                event_type: event_type.to_string(),
                authority_role: Some(agentflow_workflow_core::WorkflowAgentRole::WorkAgent),
                actor: EventActor {
                    role: "test".to_string(),
                    kind: "system".to_string(),
                },
                state: state.map(|to_state| agentflow_event_store::EventStateTransition {
                    from_state: "todo".to_string(),
                    to_state: to_state.to_string(),
                }),
                correlation_id: Some("corr-iss-001".to_string()),
                causation_id: None,
                payload: json!({"runId": "run-001"}),
                artifact_refs: Vec::new(),
                idempotency_key: Some(format!("{event_type}:iss-001")),
            },
        )
        .unwrap();
    }

    #[test]
    fn prepare_state_workspace_creates_manifest_index_and_required_directories() {
        let dir = tempdir().unwrap();
        prepare_state_workspace(dir.path()).unwrap();
        assert!(dir.path().join(".agentflow/state/manifest.json").is_file());
        assert!(dir.path().join(".agentflow/state/index.json").is_file());
        assert!(dir.path().join(".agentflow/state/health").is_dir());
        assert!(dir.path().join(".agentflow/state/gates").is_dir());
        assert!(dir.path().join(".agentflow/state/sessions").is_dir());
        assert!(dir.path().join(".agentflow/state/locks").is_dir());
        assert!(dir
            .path()
            .join(".agentflow/state/events/timeline.jsonl")
            .is_file());
        assert!(dir.path().join(".agentflow/state/indexes").is_dir());
    }

    #[test]
    fn state_prepare_does_not_write_retired_layers() {
        let dir = tempdir().unwrap();
        prepare_state_workspace(dir.path()).unwrap();
        assert!(!dir.path().join(".agentflow/input/manifest.json").exists());
        assert!(!dir.path().join(".agentflow/execute/manifest.json").exists());
        assert!(!dir.path().join(".agentflow/output/manifest.json").exists());
    }

    #[test]
    fn health_aggregation_reads_new_layer_statuses() {
        let dir = tempdir().unwrap();
        prepare_layers(dir.path());
        agentflow_event_store::prepare_event_store(dir.path()).unwrap();
        let status = prepare_state_workspace(dir.path()).unwrap();

        assert_eq!(
            status.health.get("define").map(String::as_str),
            Some("ready")
        );
        assert_eq!(
            status.health.get("panel").map(String::as_str),
            Some("ready")
        );
        assert_eq!(status.health.get("spec").map(String::as_str), Some("ready"));
        assert_eq!(
            status.health.get("projection").map(String::as_str),
            Some("ready")
        );
        assert_eq!(
            status.health.get("events").map(String::as_str),
            Some("ready")
        );
        assert_eq!(status.health.get("audit").map(String::as_str), Some("idle"));
    }

    #[test]
    fn workflow_gate_uses_projection_task_state() {
        let dir = tempdir().unwrap();
        prepare_layers(dir.path());
        write_issue(dir.path(), SpecIssueStatus::Todo);
        append_issue_event(dir.path(), "agent.launch.requested", Some("in_progress"));
        agentflow_projection::rebuild_projections(dir.path()).unwrap();

        let status = refresh_state(dir.path()).unwrap();

        assert_eq!(status.current_stage, WorkflowStage::ExecuteRunning);
        assert_eq!(status.active_issue_id.as_deref(), Some("iss-001"));
        assert_eq!(status.active_run_id.as_deref(), Some("run-001"));
    }

    #[test]
    fn issue_status_index_reads_projection_and_task_evidence() {
        let dir = tempdir().unwrap();
        prepare_layers(dir.path());
        write_issue(dir.path(), SpecIssueStatus::Todo);
        append_issue_event(dir.path(), "agent.launch.requested", Some("in_progress"));
        let run = agentflow_task_artifacts::create_task_run(
            dir.path(),
            "iss-001",
            "run-001",
            "build-agent.issue-loop@v1",
            None,
        )
        .unwrap();
        agentflow_task_artifacts::write_task_command_record(
            dir.path(),
            "iss-001",
            &run.run_id,
            agentflow_task_artifacts::TaskCommandInput {
                label: "printf ok".to_string(),
                program: "printf".to_string(),
                args: vec!["ok".to_string()],
                exit_code: Some(0),
                stdout: "ok".to_string(),
                stderr: String::new(),
            },
        )
        .unwrap();
        agentflow_task_artifacts::write_task_validation(dir.path(), "iss-001", &run.run_id)
            .unwrap();
        agentflow_task_artifacts::write_task_evidence(
            dir.path(),
            "iss-001",
            &run.run_id,
            "Fixture evidence.",
        )
        .unwrap();
        append_issue_event(dir.path(), "issue.review.requested", Some("in_review"));
        agentflow_projection::rebuild_projections(dir.path()).unwrap();

        refresh_state(dir.path()).unwrap();
        let issue_index = load_issue_status_index(dir.path()).unwrap();
        let issue = issue_index
            .issues
            .iter()
            .find(|item| item.issue_id == "iss-001")
            .unwrap();
        assert_eq!(issue.display_status, "in_review");
        assert_eq!(issue.latest_run_id.as_deref(), Some("run-001"));
        assert_eq!(issue.evidence_status, "ready");
    }

    #[test]
    fn human_audit_request_updates_state() {
        let dir = tempdir().unwrap();
        prepare_layers(dir.path());
        write_issue(dir.path(), SpecIssueStatus::Done);
        request_human_audit(
            dir.path(),
            HumanAuditRequestDraft {
                reason: "Human requested audit.".to_string(),
                scope: AuditScope {
                    description: "Audit delivery chain.".to_string(),
                    refs: vec![
                        AuditScopeRef {
                            kind: "issue".to_string(),
                            id: "iss-001".to_string(),
                            path: ".agentflow/spec/issues/iss-001.json".to_string(),
                        },
                        AuditScopeRef {
                            kind: "evidence".to_string(),
                            id: "run-001".to_string(),
                            path: ".agentflow/tasks/iss-001/evidence/evidence.json".to_string(),
                        },
                    ],
                },
            },
        )
        .unwrap();

        let state = refresh_state(dir.path()).unwrap();
        assert_eq!(state.current_stage, WorkflowStage::AuditCompleted);
        assert_ne!(state.audit_status, WorkflowAuditStatus::NotRequested);
    }

    #[test]
    fn append_event_and_update_session_write_state_only_records() {
        let dir = tempdir().unwrap();
        prepare_state_workspace(dir.path()).unwrap();
        append_state_event(
            dir.path(),
            StateTimelineEventDraft {
                event: "workspace.prepared".to_string(),
                details: Default::default(),
            },
        )
        .unwrap();
        assert_eq!(load_state_timeline(dir.path()).unwrap().len(), 1);
        let session = update_state_session(
            dir.path(),
            StateSessionUpdate {
                session_id: "session-001".to_string(),
                active_role: Some("Build Agent".to_string()),
                active_issue_id: Some("iss-001".to_string()),
                active_run_id: Some("run-001".to_string()),
                status: Some("waiting-human-confirmation".to_string()),
                waiting_for_human: Some(true),
                last_action: Some("task.preflight.blocked".to_string()),
            },
        )
        .unwrap();
        assert!(session.waiting_for_human);
        assert!(dir
            .path()
            .join(".agentflow/state/sessions/session-001.json")
            .is_file());
    }
}
