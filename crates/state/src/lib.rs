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
    use agentflow_execute::{
        acquire_execute_lease, create_execute_checkpoint, create_execute_run,
        execute_run_preflight, release_execute_lease, run_execute_command, validate_execute_run,
        write_execute_plan, ExecuteCommandRequest, ExecuteLease, ExecuteLeaseStatus,
        ExecutePlanDraft, ExecuteRun,
    };
    use agentflow_input::{
        issue::{
            AgentClaim, AgentRole, DisplayStatus, InputIssue, InputIssueModel, InputIssueStatus,
            InputRiskLevel, IssueCategory, AGENT_CLAIM_VERSION,
        },
        spec_gate::{InputIssueGenerationMode, InputSpecApproval},
    };
    use agentflow_output::{
        request_human_audit, AuditScope, AuditScopeRef, HumanAuditRequestDraft,
        OutputReleaseDelivery, OutputReleaseDeliveryArtifacts, OUTPUT_RELEASE_DELIVERY_VERSION,
    };
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
        agentflow_input::prepare_input_workspace(root).unwrap();
        agentflow_execute::prepare_execute_workspace(root).unwrap();
    }

    fn write_spec(root: &Path) {
        let spec_dir = root.join(".agentflow/input/specs/approved/spec-001");
        fs::create_dir_all(&spec_dir).unwrap();
        fs::write(spec_dir.join("product.md"), "# Product\n").unwrap();
        fs::write(spec_dir.join("tech.md"), "# Tech\n").unwrap();
        fs::write(spec_dir.join("spec.json"), "{}\n").unwrap();
        fs::write(
            spec_dir.join("approval.json"),
            serde_json::to_string_pretty(&InputSpecApproval {
                spec_id: "spec-001".to_string(),
                issue_generation_mode: InputIssueGenerationMode::Direct,
                ..InputSpecApproval::default()
            })
            .unwrap(),
        )
        .unwrap();
    }

    fn write_issue(root: &Path, risk_level: InputRiskLevel) {
        let issue = InputIssue {
            issue_id: "iss-001".to_string(),
            issue_model: InputIssueModel::Direct,
            source_spec_id: "spec-001".to_string(),
            project_id: None,
            title: "Fixture issue".to_string(),
            summary: "Fixture issue".to_string(),
            status: InputIssueStatus::Todo,
            execution_risk: risk_level,
            validation_hints: vec!["printf ok".to_string()],
            scope: vec!["src/lib.rs".to_string()],
            ..InputIssue::default()
        };
        fs::write(
            root.join(".agentflow/input/issues/iss-001.json"),
            serde_json::to_string_pretty(&issue).unwrap(),
        )
        .unwrap();
    }

    fn write_plan_and_checkpoint(root: &Path, run: &ExecuteRun) {
        acquire_execute_lease(root, run.run_id.clone()).unwrap();
        write_execute_plan(
            root,
            run.run_id.clone(),
            ExecutePlanDraft {
                steps: Vec::new(),
                allowed_write_paths: vec!["src/lib.rs".to_string()],
                allowed_commands: vec!["printf ok".to_string()],
            },
        )
        .unwrap();
        create_execute_checkpoint(root, run.run_id.clone()).unwrap();
    }

    fn complete_run(root: &Path) -> ExecuteRun {
        write_spec(root);
        write_issue(root, InputRiskLevel::Low);
        let run = create_execute_run(root, "iss-001".to_string()).unwrap();
        let preflight = execute_run_preflight(root, run.run_id.clone()).unwrap();
        assert_eq!(preflight.status, "ready");
        write_plan_and_checkpoint(root, &run);
        run_execute_command(
            root,
            run.run_id.clone(),
            ExecuteCommandRequest {
                label: "printf ok".to_string(),
                program: "printf".to_string(),
                args: vec!["ok".to_string()],
                source: Some("test".to_string()),
            },
        )
        .unwrap();
        validate_execute_run(root, run.run_id.clone()).unwrap();
        agentflow_execute::load_execute_run(root, run.run_id).unwrap()
    }

    fn write_evidence_and_delivery(root: &Path, run: &ExecuteRun) {
        let run_id = &run.run_id;
        assert!(root
            .join(format!(
                ".agentflow/tasks/{}/evidence/evidence.json",
                run.issue_id
            ))
            .is_file());
        let release_dir = root.join(".agentflow/output/release").join(run_id);
        fs::create_dir_all(&release_dir).unwrap();
        let delivery = OutputReleaseDelivery {
            version: OUTPUT_RELEASE_DELIVERY_VERSION.to_string(),
            run_id: run_id.clone(),
            issue_id: run.issue_id.clone(),
            source_spec_id: run.source_spec_id.clone(),
            risk_level: run.risk_level.clone(),
            status: "drafted".to_string(),
            created_by: "Build Agent".to_string(),
            created_at: 1,
            evidence_path: format!(".agentflow/tasks/{}/evidence/evidence.json", run.issue_id),
            execute_result_path: format!(".agentflow/execute/runs/{run_id}/result.json"),
            diff_summary_path: Some(format!(
                ".agentflow/execute/runs/{run_id}/review/diff-summary.json"
            )),
            artifacts: OutputReleaseDeliveryArtifacts {
                pr_draft: format!(".agentflow/output/release/{run_id}/pr-draft.md"),
                pr_metadata: format!(".agentflow/output/release/{run_id}/pr-metadata.json"),
                review_checklist: format!(".agentflow/output/release/{run_id}/review-checklist.md"),
                changelog: format!(".agentflow/output/release/{run_id}/changelog.md"),
                release_note: format!(".agentflow/output/release/{run_id}/release-note.md"),
            },
        };
        fs::write(
            release_dir.join("delivery.json"),
            serde_json::to_string_pretty(&delivery).unwrap(),
        )
        .unwrap();
        for file in [
            "pr-draft.md",
            "review-checklist.md",
            "changelog.md",
            "release-note.md",
        ] {
            fs::write(release_dir.join(file), "# fixture\n").unwrap();
        }
        fs::write(
            release_dir.join("pr-metadata.json"),
            serde_json::to_string_pretty(&json!({
                "version": "output-pr-metadata.v1",
                "runId": run_id,
                "issueId": run.issue_id,
                "sourceSpecId": run.source_spec_id,
                "title": "Fixture PR",
                "branchName": null,
                "remotePrUrl": null,
                "status": "draft-only",
                "createdRemotePr": false
            }))
            .unwrap(),
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
    fn state_prepare_does_not_write_other_layers() {
        let dir = tempdir().unwrap();
        prepare_state_workspace(dir.path()).unwrap();
        assert!(!dir.path().join(".agentflow/input/manifest.json").exists());
        assert!(!dir.path().join(".agentflow/panel/manifest.json").exists());
        assert!(!dir.path().join(".agentflow/execute/manifest.json").exists());
        assert!(!dir.path().join(".agentflow/output/manifest.json").exists());
    }

    #[test]
    fn health_aggregation_reads_layer_statuses() {
        let dir = tempdir().unwrap();
        prepare_layers(dir.path());
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
            status.health.get("tasks").map(String::as_str),
            Some("ready")
        );
        assert_eq!(
            status.health.get("events").map(String::as_str),
            Some("ready")
        );
        assert_eq!(status.health.get("audit").map(String::as_str), Some("idle"));
    }

    #[test]
    fn workflow_gate_returns_workspace_ready_when_core_layers_are_ready() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("README.md"), "# fixture\n").unwrap();
        agentflow_agent_manual::prepare_agent_working_manual(dir.path()).unwrap();
        let status = prepare_state_workspace(dir.path()).unwrap();
        assert_eq!(status.current_stage, WorkflowStage::WorkspaceReady);
    }

    #[test]
    fn workflow_gate_keeps_delivery_ready_without_auto_audit_after_delivery() {
        let dir = tempdir().unwrap();
        prepare_layers(dir.path());
        let run = complete_run(dir.path());
        write_evidence_and_delivery(dir.path(), &run);
        agentflow_output::prepare_output_workspace(dir.path()).unwrap();
        let status = refresh_state(dir.path()).unwrap();
        assert_eq!(status.current_stage, WorkflowStage::DeliveryReady);
        assert_eq!(status.audit_status, WorkflowAuditStatus::NotRequested);
        assert!(!status
            .next_actions
            .contains(&"request-human-audit".to_string()));
        agentflow_output::prepare_audit_workspace(dir.path()).unwrap();
        let audit_index = agentflow_output::load_audit_index(dir.path()).unwrap();
        assert_eq!(audit_index.audits.len(), 0);
    }

    #[test]
    fn workflow_gate_does_not_block_when_delivery_has_no_audit_request() {
        let dir = tempdir().unwrap();
        prepare_layers(dir.path());
        let run = complete_run(dir.path());
        write_evidence_and_delivery(dir.path(), &run);
        agentflow_output::prepare_output_workspace(dir.path()).unwrap();

        let status = refresh_state(dir.path()).unwrap();

        assert_eq!(status.current_stage, WorkflowStage::DeliveryReady);
        assert_eq!(status.audit_status, WorkflowAuditStatus::NotRequested);
        assert!(status.blockers.is_empty());
    }

    #[test]
    fn high_risk_blocked_preflight_appears_in_blockers() {
        let dir = tempdir().unwrap();
        prepare_layers(dir.path());
        write_spec(dir.path());
        write_issue(dir.path(), InputRiskLevel::High);
        let run = create_execute_run(dir.path(), "iss-001".to_string()).unwrap();
        execute_run_preflight(dir.path(), run.run_id).unwrap();
        let status = refresh_state(dir.path()).unwrap();
        assert!(status
            .blockers
            .iter()
            .any(|blocker| blocker.reason.contains("High risk issue requires")));
    }

    #[test]
    fn stale_blocked_preflight_is_ignored_after_newer_completed_run_for_same_issue() {
        let dir = tempdir().unwrap();
        prepare_layers(dir.path());
        write_spec(dir.path());
        write_issue(dir.path(), InputRiskLevel::High);
        let blocked_run = create_execute_run(dir.path(), "iss-001".to_string()).unwrap();
        let blocked_preflight = execute_run_preflight(dir.path(), blocked_run.run_id).unwrap();
        assert_eq!(blocked_preflight.status, "blocked");

        let completed_run = complete_run(dir.path());
        assert_eq!(
            completed_run.status,
            agentflow_execute::ExecuteRunStatus::Completed
        );

        let status = refresh_state(dir.path()).unwrap();

        assert!(!status.blockers.iter().any(|blocker| {
            blocker.action == "execute-issue"
                && blocker.source_path.as_deref()
                    == Some(".agentflow/execute/runs/run-001/preflight.json")
        }));
        assert!(status.blockers.is_empty());
    }

    #[test]
    fn runtime_preflight_refresh_aligns_issue_and_run_indexes() {
        let dir = tempdir().unwrap();
        prepare_layers(dir.path());
        write_spec(dir.path());
        write_issue(dir.path(), InputRiskLevel::Low);
        let run = create_execute_run(dir.path(), "iss-001".to_string()).unwrap();

        let preflight = execute_run_preflight(dir.path(), run.run_id.clone()).unwrap();
        assert_eq!(preflight.status, "ready");

        refresh_state(dir.path()).unwrap();

        let issue_index: IssueStatusIndex = crate::storage::read_json(
            &dir.path()
                .join(".agentflow/state/indexes/issue-status.json"),
        )
        .unwrap();
        let issue = issue_index
            .issues
            .iter()
            .find(|item| item.issue_id == "iss-001")
            .unwrap();
        assert_eq!(issue.display_status, DisplayStatus::InProgress);
        assert_eq!(issue.execute_status.as_deref(), Some("planned"));
        assert_eq!(issue.latest_run_id.as_deref(), Some(run.run_id.as_str()));

        let run_index: RunStatusIndex =
            crate::storage::read_json(&dir.path().join(".agentflow/state/indexes/run-status.json"))
                .unwrap();
        let run_entry = run_index
            .runs
            .iter()
            .find(|item| item.run_id == run.run_id)
            .unwrap();
        assert_eq!(run_entry.execute_status, "planned");
    }

    #[test]
    fn missing_issue_target_metadata_blocks_handoff_copy() {
        let dir = tempdir().unwrap();
        prepare_layers(dir.path());
        let issue = InputIssue {
            issue_id: "iss-missing-target".to_string(),
            issue_model: InputIssueModel::Direct,
            source_spec_id: String::new(),
            project_id: None,
            title: "Missing target".to_string(),
            summary: "Missing target metadata".to_string(),
            status: InputIssueStatus::Todo,
            execution_risk: InputRiskLevel::Low,
            ..InputIssue::default()
        };
        fs::write(
            dir.path()
                .join(".agentflow/input/issues/iss-missing-target.json"),
            serde_json::to_string_pretty(&issue).unwrap(),
        )
        .unwrap();

        let status = refresh_state(dir.path()).unwrap();

        assert!(status.blockers.iter().any(|blocker| {
            blocker.action == "copy-handoff" && blocker.reason == "任务缺少执行目标，不能生成任务包"
        }));
    }

    #[test]
    fn active_released_and_stale_locks_are_classified() {
        let dir = tempdir().unwrap();
        prepare_layers(dir.path());
        write_spec(dir.path());
        write_issue(dir.path(), InputRiskLevel::Low);
        let active_run = create_execute_run(dir.path(), "iss-001".to_string()).unwrap();
        execute_run_preflight(dir.path(), active_run.run_id.clone()).unwrap();
        acquire_execute_lease(dir.path(), active_run.run_id.clone()).unwrap();
        refresh_state(dir.path()).unwrap();
        let active: StateLockSnapshot =
            crate::storage::read_json(&dir.path().join(".agentflow/state/locks/active.json"))
                .unwrap();
        assert_eq!(active.active.len(), 1);

        release_execute_lease(dir.path(), active_run.run_id.clone()).unwrap();
        refresh_state(dir.path()).unwrap();
        let active_after_release: StateLockSnapshot =
            crate::storage::read_json(&dir.path().join(".agentflow/state/locks/active.json"))
                .unwrap();
        assert!(active_after_release.active.is_empty());

        let stale_lease = ExecuteLease {
            version: "execute-lease.v1".to_string(),
            issue_id: "iss-stale".to_string(),
            run_id: "run-stale".to_string(),
            status: ExecuteLeaseStatus::Active,
            created_at: 1,
            released_at: None,
            expires_at: None,
            locked_files: Vec::new(),
        };
        fs::write(
            dir.path().join(".agentflow/execute/leases/iss-stale.json"),
            serde_json::to_string_pretty(&stale_lease).unwrap(),
        )
        .unwrap();
        refresh_state(dir.path()).unwrap();
        let stale: StateLockSnapshot =
            crate::storage::read_json(&dir.path().join(".agentflow/state/locks/stale.json"))
                .unwrap();
        assert_eq!(stale.stale.len(), 1);
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
                last_action: Some("execute.preflight.blocked".to_string()),
            },
        )
        .unwrap();
        assert!(session.waiting_for_human);
        assert!(dir
            .path()
            .join(".agentflow/state/sessions/session-001.json")
            .is_file());
    }

    #[test]
    fn role_mismatch_writes_blocker_and_timeline_event() {
        let dir = tempdir().unwrap();
        prepare_layers(dir.path());
        write_spec(dir.path());
        write_issue(dir.path(), InputRiskLevel::Low);
        let run = create_execute_run(dir.path(), "iss-001".to_string()).unwrap();
        fs::write(
            dir.path().join(format!(
                ".agentflow/execute/runs/{}/agent-claim.json",
                run.run_id
            )),
            serde_json::to_string_pretty(&AgentClaim {
                version: AGENT_CLAIM_VERSION.to_string(),
                issue_id: "iss-001".to_string(),
                issue_category: IssueCategory::Spec,
                claimed_agent_role: AgentRole::AuditAgent,
                handoff_id: "handoff-iss-001".to_string(),
                created_by: "audit-agent".to_string(),
            })
            .unwrap(),
        )
        .unwrap();

        let status = refresh_state(dir.path()).unwrap();

        assert!(status
            .blockers
            .iter()
            .any(|blocker| blocker.action == "agent-role-mismatch"));
        assert!(load_state_timeline(dir.path())
            .unwrap()
            .iter()
            .any(|event| event.event == "agent.role_mismatch"));
    }

    #[test]
    fn issue_status_index_aggregates_issue_run_output_and_audit_status() {
        let dir = tempdir().unwrap();
        prepare_layers(dir.path());
        let run = complete_run(dir.path());
        write_evidence_and_delivery(dir.path(), &run);
        agentflow_output::prepare_output_workspace(dir.path()).unwrap();
        request_human_audit(
            dir.path(),
            HumanAuditRequestDraft {
                reason: "Human requested audit.".to_string(),
                scope: AuditScope {
                    description: "Audit delivery chain.".to_string(),
                    refs: vec![
                        AuditScopeRef {
                            kind: "spec".to_string(),
                            id: "spec-001".to_string(),
                            path: ".agentflow/input/specs/approved/spec-001".to_string(),
                        },
                        AuditScopeRef {
                            kind: "issue".to_string(),
                            id: "iss-001".to_string(),
                            path: ".agentflow/input/issues/iss-001.json".to_string(),
                        },
                        AuditScopeRef {
                            kind: "execute-run".to_string(),
                            id: run.run_id.clone(),
                            path: format!(".agentflow/execute/runs/{}", run.run_id),
                        },
                    ],
                },
            },
        )
        .unwrap();
        let status = refresh_state(dir.path()).unwrap();
        assert_eq!(status.audit_status, WorkflowAuditStatus::Failed);
        let issue_index: IssueStatusIndex = crate::storage::read_json(
            &dir.path()
                .join(".agentflow/state/indexes/issue-status.json"),
        )
        .unwrap();
        let issue = issue_index
            .issues
            .iter()
            .find(|item| item.issue_id == "iss-001")
            .unwrap();
        assert_eq!(issue.latest_run_id.as_deref(), Some(run.run_id.as_str()));
        assert_eq!(issue.display_status, DisplayStatus::InReview);
        assert_eq!(issue.delivery_status, "drafted");
        assert_eq!(issue.audit_status, WorkflowAuditStatus::Failed);
    }
}
