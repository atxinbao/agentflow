pub mod checkpoint;
pub mod command;
pub mod completion;
pub mod delivery;
pub mod evidence;
pub mod lease;
pub mod manager;
pub mod model;
pub mod patch;
pub mod plan;
pub mod preflight;
pub mod result;
pub mod storage;
pub mod validation;

pub use agentflow_output::{
    load_output_evidence, load_release_delivery, OutputEvidence, OutputReleaseDelivery,
    OutputReleaseDeliveryArtifacts, OUTPUT_EVIDENCE_VERSION, OUTPUT_RELEASE_DELIVERY_VERSION,
};
pub use checkpoint::create_execute_checkpoint;
pub use command::run_execute_command;
pub use completion::complete_build_agent_issue;
pub use delivery::prepare_release_delivery;
pub use evidence::write_execute_evidence;
pub use lease::{acquire_execute_lease, release_execute_lease};
pub use manager::{
    cancel_execute_run, create_execute_run, load_execute_index, load_execute_manifest,
    load_execute_result, load_execute_run, load_execute_snapshot, load_execute_status,
    prepare_execute_workspace, validate_execute_workspace,
};
pub use model::*;
pub use patch::apply_execute_patch;
pub use plan::write_execute_plan;
pub use preflight::{confirm_high_risk_execute_run, execute_run_preflight};
pub use validation::{complete_execute_run, validate_execute_run};

#[cfg(test)]
mod tests {
    use super::*;
    use agentflow_input::{
        issue::{
            AgentRole, InputIssue, InputIssueModel, InputIssueStatus, InputRiskLevel, IssueCategory,
        },
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
        agentflow_input::prepare_input_workspace(root).unwrap();
        agentflow_panel::prepare_project_panel(root, agentflow_panel::PanelPrepareMode::Blocking)
            .unwrap();
    }

    fn write_approved_spec(root: &Path, spec_id: &str) {
        let spec_dir = root.join(".agentflow/input/specs/approved").join(spec_id);
        fs::create_dir_all(&spec_dir).unwrap();
        fs::write(spec_dir.join("product.md"), "# Product\n").unwrap();
        fs::write(spec_dir.join("tech.md"), "# Tech\n").unwrap();
        fs::write(spec_dir.join("spec.json"), "{}\n").unwrap();
        fs::write(
            spec_dir.join("approval.json"),
            serde_json::to_string_pretty(&InputSpecApproval {
                spec_id: spec_id.to_string(),
                issue_generation_mode: InputIssueGenerationMode::Direct,
                ..InputSpecApproval::default()
            })
            .unwrap(),
        )
        .unwrap();
    }

    fn write_issue(
        root: &Path,
        issue_id: &str,
        spec_id: &str,
        risk_level: InputRiskLevel,
    ) -> InputIssue {
        let issue = InputIssue {
            issue_id: issue_id.to_string(),
            issue_model: InputIssueModel::Direct,
            source_spec_id: spec_id.to_string(),
            project_id: None,
            title: format!("Execute {issue_id}"),
            summary: "Execute fixture issue".to_string(),
            status: InputIssueStatus::Todo,
            execution_risk: risk_level,
            scope: vec!["src/lib.rs".to_string()],
            validation_hints: vec!["printf ok".to_string()],
            ..InputIssue::default()
        };
        fs::write(
            root.join(".agentflow/input/issues")
                .join(format!("{issue_id}.json")),
            serde_json::to_string_pretty(&issue).unwrap(),
        )
        .unwrap();
        issue
    }

    fn ready_low_risk_run(root: &Path, issue_id: &str) -> ExecuteRun {
        prepare_root(root);
        write_approved_spec(root, "spec-001");
        write_issue(root, issue_id, "spec-001", InputRiskLevel::Low);
        let run = create_execute_run(root, issue_id.to_string()).unwrap();
        let preflight = execute_run_preflight(root, run.run_id.clone()).unwrap();
        assert_eq!(preflight.status, "ready");
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
        run
    }

    fn completed_low_risk_run(root: &Path, issue_id: &str) -> ExecuteRun {
        let run = ready_low_risk_run(root, issue_id);
        let patch = "diff --git a/src/lib.rs b/src/lib.rs\n--- a/src/lib.rs\n+++ b/src/lib.rs\n@@ -1,3 +1,3 @@\n pub fn value() -> u8 {\n-    1\n+    2\n }\n";
        apply_execute_patch(root, run.run_id.clone(), patch.to_string()).unwrap();
        run_execute_command(
            root,
            run.run_id.clone(),
            ExecuteCommandRequest {
                label: "printf ok".to_string(),
                program: "printf".to_string(),
                args: vec!["ok".to_string()],
                source: Some("runPlan.allowedCommands".to_string()),
            },
        )
        .unwrap();
        let result = validate_execute_run(root, run.run_id.clone()).unwrap();
        assert_eq!(result.status, ExecuteRunStatus::Completed);
        assert!(result.next.ready_for_delivery);
        run
    }

    fn init_clean_git_repo(root: &Path) {
        fs::write(root.join(".gitignore"), ".agentflow/\nAGENTS.md\n").unwrap();
        Command::new("git")
            .arg("init")
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

    #[test]
    fn build_agent_completion_creates_run_evidence_and_delivery() {
        let dir = tempdir().unwrap();
        prepare_root(dir.path());
        write_approved_spec(dir.path(), "spec-001");
        write_issue(dir.path(), "iss-001", "spec-001", InputRiskLevel::Low);
        let run = create_execute_run(dir.path(), "iss-001".to_string()).unwrap();
        let preflight = execute_run_preflight(dir.path(), run.run_id.clone()).unwrap();
        assert_eq!(preflight.status, "ready");
        storage::write_json(
            &storage::run_dir(dir.path(), &run.run_id).join("review/merge-proof.json"),
            &serde_json::json!({
                "version": "execute-merge-proof.v1",
                "runId": run.run_id,
                "issueId": "iss-001",
                "provider": "github",
                "mergeMode": "auto-merge-if-eligible",
                "merged": true,
                "remoteUrl": "https://github.com/atxinbao/agentflow/pull/1"
            }),
        )
        .unwrap();

        let completion = complete_build_agent_issue(
            dir.path(),
            BuildAgentCompletionRequest {
                issue_id: "iss-001".to_string(),
                run_id: Some(run.run_id.clone()),
                changed_files: vec![ExecuteChangedFile {
                    path: "src/lib.rs".to_string(),
                    change_type: "modified".to_string(),
                    insertions: 1,
                    deletions: 1,
                }],
                validation_commands: vec![BuildAgentValidationCommand {
                    label: "printf ok".to_string(),
                    program: "printf".to_string(),
                    args: vec!["ok".to_string()],
                    exit_code: Some(0),
                    stdout: Some("ok".to_string()),
                    stderr: None,
                    source: Some("test".to_string()),
                }],
            },
        )
        .unwrap();

        assert_eq!(completion.run.issue_id, "iss-001");
        assert_eq!(completion.run.status, ExecuteRunStatus::Completed);
        assert_eq!(completion.result.status, ExecuteRunStatus::Completed);
        assert!(completion.result.validation.passed);
        assert!(!completion.result.next.needs_audit);
        assert_eq!(completion.delivery.status, "drafted");
        let issue_after: InputIssue =
            crate::storage::read_json(&dir.path().join(".agentflow/input/issues/iss-001.json"))
                .unwrap();
        assert_eq!(issue_after.status, InputIssueStatus::Done);
        assert_eq!(
            issue_after.display_status,
            agentflow_input::issue::DisplayStatus::Done
        );

        let execute_index = load_execute_index(dir.path()).unwrap();
        assert_eq!(execute_index.runs.len(), 1);
        assert_eq!(execute_index.runs[0].status, ExecuteRunStatus::Completed);

        let output_index = agentflow_output::load_output_index(dir.path()).unwrap();
        assert_eq!(output_index.evidence.len(), 1);
        assert_eq!(output_index.release_deliveries.len(), 1);
        let pr_metadata: agentflow_output::OutputPrMetadata = crate::storage::read_json(
            &dir.path()
                .join(".agentflow/output/release")
                .join(&completion.run.run_id)
                .join("pr-metadata.json"),
        )
        .unwrap();
        assert_eq!(
            pr_metadata.branch_name.as_deref(),
            Some("agentflow/direct/iss-001")
        );
        assert_eq!(
            pr_metadata.remote_pr_url.as_deref(),
            Some("https://github.com/atxinbao/agentflow/pull/1")
        );
        assert_eq!(pr_metadata.status, "merged");
        assert_eq!(pr_metadata.provider.as_deref(), Some("github"));
        assert_eq!(
            pr_metadata.merge_mode.as_deref(),
            Some("auto-merge-if-eligible")
        );
        assert!(pr_metadata.created_remote_pr);
        assert!(pr_metadata.merged);
        let loop_projection: serde_json::Value = crate::storage::read_json(
            &dir.path()
                .join(".agentflow/state/loops/issues/iss-001.json"),
        )
        .unwrap();
        assert_eq!(loop_projection["stage"], "done");
        assert_eq!(loop_projection["runId"], completion.run.run_id);
        assert_eq!(loop_projection["branchName"], "agentflow/direct/iss-001");
    }

    #[test]
    fn missing_input_issue_cannot_create_run() {
        let dir = tempdir().unwrap();
        prepare_root(dir.path());
        let error = create_execute_run(dir.path(), "missing".to_string()).unwrap_err();
        assert!(error.to_string().contains("does not exist"));
    }

    #[test]
    fn backlog_issue_cannot_start_runtime_preflight() {
        let dir = tempdir().unwrap();
        prepare_root(dir.path());
        write_approved_spec(dir.path(), "spec-001");
        let mut issue = write_issue(dir.path(), "iss-001", "spec-001", InputRiskLevel::Low);
        issue.status = InputIssueStatus::Backlog;
        fs::write(
            dir.path().join(".agentflow/input/issues/iss-001.json"),
            serde_json::to_string_pretty(&issue).unwrap(),
        )
        .unwrap();

        let error = create_execute_run(dir.path(), "iss-001".to_string()).unwrap_err();

        assert!(error.to_string().contains("must be todo"));
    }

    #[test]
    fn audit_issue_cannot_create_build_agent_run() {
        let dir = tempdir().unwrap();
        prepare_root(dir.path());
        write_approved_spec(dir.path(), "release-v0.1.0");
        let issue = InputIssue {
            issue_id: "audit-release-v0.1.0".to_string(),
            issue_model: InputIssueModel::Direct,
            issue_category: IssueCategory::Audit,
            required_agent_role: AgentRole::AuditAgent,
            source_spec_id: "release-v0.1.0".to_string(),
            project_id: None,
            title: "Audit release".to_string(),
            summary: "Audit release delivery".to_string(),
            status: InputIssueStatus::Backlog,
            execution_risk: InputRiskLevel::High,
            validation_hints: vec!["audit output".to_string()],
            ..InputIssue::default()
        };
        fs::write(
            dir.path()
                .join(".agentflow/input/issues/audit-release-v0.1.0.json"),
            serde_json::to_string_pretty(&issue).unwrap(),
        )
        .unwrap();

        let error = create_execute_run(dir.path(), "audit-release-v0.1.0".to_string()).unwrap_err();

        assert!(error.to_string().contains("Agent role mismatch"));
    }

    #[test]
    fn issue_without_source_spec_id_blocks_preflight() {
        let dir = tempdir().unwrap();
        prepare_root(dir.path());
        write_issue(dir.path(), "iss-001", "", InputRiskLevel::Low);
        let run = create_execute_run(dir.path(), "iss-001".to_string()).unwrap();
        let preflight = execute_run_preflight(dir.path(), run.run_id).unwrap();
        assert_eq!(preflight.status, "blocked");
        assert!(preflight
            .checks
            .iter()
            .any(|check| check.name == "source-spec-id"
                && matches!(check.status, ExecuteCheckStatus::Blocked)));
    }

    #[test]
    fn missing_approved_spec_blocks_preflight() {
        let dir = tempdir().unwrap();
        prepare_root(dir.path());
        write_issue(dir.path(), "iss-001", "missing-spec", InputRiskLevel::Low);
        let run = create_execute_run(dir.path(), "iss-001".to_string()).unwrap();
        let preflight = execute_run_preflight(dir.path(), run.run_id).unwrap();
        let issue = agentflow_input::load_input_issue(dir.path(), "iss-001").unwrap();
        assert_eq!(preflight.status, "blocked");
        assert_eq!(issue.status, InputIssueStatus::Blocked);
        assert!(preflight
            .checks
            .iter()
            .any(|check| check.name == "approved-spec"
                && matches!(check.status, ExecuteCheckStatus::Blocked)));
    }

    #[test]
    fn preflight_generates_panel_context_pack_for_issue() {
        let dir = tempdir().unwrap();
        prepare_root(dir.path());
        write_approved_spec(dir.path(), "spec-001");
        write_issue(dir.path(), "iss-001", "spec-001", InputRiskLevel::Low);
        let run = create_execute_run(dir.path(), "iss-001".to_string()).unwrap();
        let issue_before_preflight =
            agentflow_input::load_input_issue(dir.path(), "iss-001").unwrap();
        assert_eq!(issue_before_preflight.status, InputIssueStatus::Todo);

        let preflight = execute_run_preflight(dir.path(), run.run_id.clone()).unwrap();
        let loaded = load_execute_run(dir.path(), run.run_id).unwrap();
        let issue_after_preflight =
            agentflow_input::load_input_issue(dir.path(), "iss-001").unwrap();

        assert_eq!(preflight.status, "ready");
        assert_eq!(issue_after_preflight.status, InputIssueStatus::InProgress);
        assert!(preflight.checks.iter().any(|check| {
            check.name == "context-pack" && matches!(check.status, ExecuteCheckStatus::Passed)
        }));
        assert!(dir
            .path()
            .join(".agentflow/panel/context-packs/iss-001.json")
            .is_file());
        assert_eq!(loaded.input.context_pack_id.as_deref(), Some("iss-001"));
        assert_eq!(
            loaded.input.context_pack_path.as_deref(),
            Some(".agentflow/panel/context-packs/iss-001.json")
        );
    }

    #[test]
    fn runtime_preflight_blocks_dirty_worktree_before_in_progress() {
        let dir = tempdir().unwrap();
        prepare_root(dir.path());
        init_clean_git_repo(dir.path());
        write_approved_spec(dir.path(), "spec-001");
        write_issue(dir.path(), "iss-001", "spec-001", InputRiskLevel::Low);
        let run = create_execute_run(dir.path(), "iss-001".to_string()).unwrap();
        fs::write(dir.path().join("README.md"), "# dirty\n").unwrap();

        let preflight = execute_run_preflight(dir.path(), run.run_id).unwrap();
        let issue = agentflow_input::load_input_issue(dir.path(), "iss-001").unwrap();

        assert_eq!(preflight.status, "blocked");
        assert_eq!(issue.status, InputIssueStatus::Blocked);
        assert!(preflight.checks.iter().any(|check| {
            check.name == "working-tree-clean"
                && matches!(check.status, ExecuteCheckStatus::Blocked)
        }));
    }

    #[test]
    fn low_and_medium_risk_do_not_require_confirmation() {
        let dir = tempdir().unwrap();
        prepare_root(dir.path());
        write_approved_spec(dir.path(), "spec-001");
        write_issue(dir.path(), "iss-low", "spec-001", InputRiskLevel::Low);
        write_issue(dir.path(), "iss-medium", "spec-001", InputRiskLevel::Medium);
        let low = create_execute_run(dir.path(), "iss-low".to_string()).unwrap();
        let medium = create_execute_run(dir.path(), "iss-medium".to_string()).unwrap();
        assert_eq!(
            execute_run_preflight(dir.path(), low.run_id)
                .unwrap()
                .status,
            "ready"
        );
        assert_eq!(
            execute_run_preflight(dir.path(), medium.run_id)
                .unwrap()
                .status,
            "ready"
        );
    }

    #[test]
    fn high_risk_requires_and_accepts_confirmation() {
        let dir = tempdir().unwrap();
        prepare_root(dir.path());
        write_approved_spec(dir.path(), "spec-001");
        write_issue(dir.path(), "iss-high", "spec-001", InputRiskLevel::High);
        let run = create_execute_run(dir.path(), "iss-high".to_string()).unwrap();
        let blocked = execute_run_preflight(dir.path(), run.run_id.clone()).unwrap();
        assert_eq!(blocked.status, "blocked");
        confirm_high_risk_execute_run(
            dir.path(),
            run.run_id.clone(),
            "I approve executing this high risk issue.".to_string(),
        )
        .unwrap();
        let ready = execute_run_preflight(dir.path(), run.run_id).unwrap();
        assert_eq!(ready.status, "ready");
    }

    #[test]
    fn same_issue_cannot_start_a_second_run_after_preflight() {
        let dir = tempdir().unwrap();
        let first = ready_low_risk_run(dir.path(), "iss-001");
        let error = create_execute_run(dir.path(), "iss-001".to_string()).unwrap_err();
        assert!(error.to_string().contains("must be todo"));
        release_execute_lease(dir.path(), first.run_id).unwrap();
    }

    #[test]
    fn released_lease_does_not_reset_issue_to_backlog() {
        let dir = tempdir().unwrap();
        let first = ready_low_risk_run(dir.path(), "iss-001");
        release_execute_lease(dir.path(), first.run_id).unwrap();
        let lease: ExecuteLease = storage::read_json(
            &dir.path()
                .join(".agentflow/execute/leases")
                .join("iss-001.json"),
        )
        .unwrap();
        assert_eq!(lease.status, ExecuteLeaseStatus::Released);

        let error = create_execute_run(dir.path(), "iss-001".to_string()).unwrap_err();
        assert!(error.to_string().contains("must be todo"));
    }

    #[test]
    fn corrupted_lease_blocks_preflight() {
        let dir = tempdir().unwrap();
        prepare_root(dir.path());
        write_approved_spec(dir.path(), "spec-001");
        write_issue(dir.path(), "iss-001", "spec-001", InputRiskLevel::Low);
        let run = create_execute_run(dir.path(), "iss-001".to_string()).unwrap();
        fs::write(
            dir.path()
                .join(".agentflow/execute/leases")
                .join("iss-001.json"),
            "{not valid json",
        )
        .unwrap();
        let preflight = execute_run_preflight(dir.path(), run.run_id).unwrap();

        assert!(preflight.checks.iter().any(|check| {
            check.name == "lease" && matches!(check.status, ExecuteCheckStatus::Blocked)
        }));
        assert!(preflight
            .blocked_reason
            .as_deref()
            .unwrap_or_default()
            .contains("Lease state unreadable"));
    }

    #[test]
    fn completed_run_releases_lease_and_writes_result_evidence() {
        let dir = tempdir().unwrap();
        let run = ready_low_risk_run(dir.path(), "iss-001");
        run_execute_command(
            dir.path(),
            run.run_id.clone(),
            ExecuteCommandRequest {
                label: "printf ok".to_string(),
                program: "printf".to_string(),
                args: vec!["ok".to_string()],
                source: Some("runPlan.allowedCommands".to_string()),
            },
        )
        .unwrap();
        let result = validate_execute_run(dir.path(), run.run_id.clone()).unwrap();
        assert_eq!(result.status, ExecuteRunStatus::Completed);
        assert!(result.next.ready_for_delivery);
        assert!(!result.next.needs_audit);
        assert!(dir
            .path()
            .join(".agentflow/execute/runs")
            .join(&run.run_id)
            .join("result.json")
            .is_file());
        assert!(dir
            .path()
            .join(".agentflow/output/evidence")
            .join(format!("{}.json", run.run_id))
            .is_file());
        let evidence = load_output_evidence(dir.path(), run.run_id.clone()).unwrap();
        assert_eq!(
            evidence.panel.context_pack_path.as_deref(),
            Some(".agentflow/panel/context-packs/iss-001.json")
        );
        let lease: ExecuteLease = storage::read_json(
            &dir.path()
                .join(".agentflow/execute/leases")
                .join("iss-001.json"),
        )
        .unwrap();
        assert_eq!(lease.status, ExecuteLeaseStatus::Released);
    }

    #[test]
    fn release_delivery_requires_completed_run() {
        let dir = tempdir().unwrap();
        let run = ready_low_risk_run(dir.path(), "iss-001");

        let error = prepare_release_delivery(dir.path(), run.run_id).unwrap_err();

        assert!(error.to_string().contains("completed execute run"));
    }

    #[test]
    fn release_delivery_requires_evidence() {
        let dir = tempdir().unwrap();
        let run = completed_low_risk_run(dir.path(), "iss-001");
        fs::remove_file(
            dir.path()
                .join(".agentflow/output/evidence")
                .join(format!("{}.json", run.run_id)),
        )
        .unwrap();

        let error = prepare_release_delivery(dir.path(), run.run_id).unwrap_err();

        assert!(error.to_string().contains(".agentflow/output/evidence"));
    }

    #[test]
    fn release_delivery_writes_build_agent_artifacts() {
        let dir = tempdir().unwrap();
        let run = completed_low_risk_run(dir.path(), "iss-001");

        let delivery = prepare_release_delivery(dir.path(), run.run_id.clone()).unwrap();
        let loaded = load_release_delivery(dir.path(), run.run_id.clone()).unwrap();

        assert_eq!(delivery.version, OUTPUT_RELEASE_DELIVERY_VERSION);
        assert_eq!(delivery.created_by, "Build Agent");
        assert_eq!(delivery.status, "drafted");
        assert_eq!(
            delivery.evidence_path,
            format!(".agentflow/output/evidence/{}.json", run.run_id)
        );
        assert_eq!(loaded.run_id, run.run_id);
        for artifact in [
            "delivery.json",
            "pr-draft.md",
            "pr-metadata.json",
            "review-checklist.md",
            "changelog.md",
            "release-note.md",
        ] {
            assert!(dir
                .path()
                .join(".agentflow/output/release")
                .join(&loaded.run_id)
                .join(artifact)
                .is_file());
        }
        assert_eq!(
            delivery.artifacts.pr_draft,
            format!(".agentflow/output/release/{}/pr-draft.md", loaded.run_id)
        );
    }

    #[test]
    fn failed_and_cancelled_runs_release_lease() {
        let failed_dir = tempdir().unwrap();
        let failed_run = ready_low_risk_run(failed_dir.path(), "iss-001");
        let failed_result =
            validate_execute_run(failed_dir.path(), failed_run.run_id.clone()).unwrap();
        assert_eq!(failed_result.status, ExecuteRunStatus::Failed);
        let failed_lease: ExecuteLease = storage::read_json(
            &failed_dir
                .path()
                .join(".agentflow/execute/leases/iss-001.json"),
        )
        .unwrap();
        assert_eq!(failed_lease.status, ExecuteLeaseStatus::Released);

        let cancelled_dir = tempdir().unwrap();
        let cancelled_run = ready_low_risk_run(cancelled_dir.path(), "iss-001");
        cancel_execute_run(cancelled_dir.path(), cancelled_run.run_id.clone()).unwrap();
        let cancelled_lease: ExecuteLease = storage::read_json(
            &cancelled_dir
                .path()
                .join(".agentflow/execute/leases/iss-001.json"),
        )
        .unwrap();
        assert_eq!(cancelled_lease.status, ExecuteLeaseStatus::Released);
    }

    #[test]
    fn patch_requires_checkpoint_and_stays_within_allowed_paths() {
        let dir = tempdir().unwrap();
        prepare_root(dir.path());
        write_approved_spec(dir.path(), "spec-001");
        write_issue(dir.path(), "iss-001", "spec-001", InputRiskLevel::Low);
        let run = create_execute_run(dir.path(), "iss-001".to_string()).unwrap();
        execute_run_preflight(dir.path(), run.run_id.clone()).unwrap();
        acquire_execute_lease(dir.path(), run.run_id.clone()).unwrap();
        write_execute_plan(
            dir.path(),
            run.run_id.clone(),
            ExecutePlanDraft {
                steps: Vec::new(),
                allowed_write_paths: vec!["src/lib.rs".to_string()],
                allowed_commands: vec!["printf ok".to_string()],
            },
        )
        .unwrap();
        let patch = "diff --git a/src/lib.rs b/src/lib.rs\n--- a/src/lib.rs\n+++ b/src/lib.rs\n@@ -1,3 +1,3 @@\n pub fn value() -> u8 {\n-    1\n+    2\n }\n";
        let missing_checkpoint =
            apply_execute_patch(dir.path(), run.run_id.clone(), patch.to_string()).unwrap_err();
        assert!(missing_checkpoint.to_string().contains("checkpoint"));
        create_execute_checkpoint(dir.path(), run.run_id.clone()).unwrap();
        let outcome =
            apply_execute_patch(dir.path(), run.run_id.clone(), patch.to_string()).unwrap();
        assert_eq!(outcome.changed_files.files[0].path, "src/lib.rs");
        assert_eq!(
            outcome.proposed_patch_path,
            format!(
                ".agentflow/execute/runs/{}/patches/proposed.patch",
                run.run_id
            )
        );
        assert_eq!(
            outcome.applied_patch_path,
            format!(
                ".agentflow/execute/runs/{}/patches/applied.patch",
                run.run_id
            )
        );
        assert_eq!(
            outcome.worktree_diff_path,
            format!(
                ".agentflow/execute/runs/{}/patches/worktree.diff",
                run.run_id
            )
        );
        assert!(fs::read_to_string(dir.path().join("src/lib.rs"))
            .unwrap()
            .contains("2"));
    }

    #[test]
    fn patch_blocks_unauthorized_path() {
        let dir = tempdir().unwrap();
        let run = ready_low_risk_run(dir.path(), "iss-001");
        let patch = "diff --git a/README.md b/README.md\n--- a/README.md\n+++ b/README.md\n@@ -1 +1 @@\n-# fixture\n+# changed\n";
        let error = apply_execute_patch(dir.path(), run.run_id, patch.to_string()).unwrap_err();
        assert!(error.to_string().contains("unauthorized path"));
    }

    #[test]
    fn command_records_stdout_stderr_exit_code_and_blocks_dangerous_commands() {
        let dir = tempdir().unwrap();
        let run = ready_low_risk_run(dir.path(), "iss-001");
        let record = run_execute_command(
            dir.path(),
            run.run_id.clone(),
            ExecuteCommandRequest {
                label: "printf ok".to_string(),
                program: "printf".to_string(),
                args: vec!["ok".to_string()],
                source: None,
            },
        )
        .unwrap();
        assert_eq!(record.exit_code, Some(0));
        assert!(dir.path().join(&record.stdout_path).is_file());
        let blocked = run_execute_command(
            dir.path(),
            run.run_id,
            ExecuteCommandRequest {
                label: "git push".to_string(),
                program: "git".to_string(),
                args: vec!["push".to_string()],
                source: None,
            },
        )
        .unwrap_err();
        assert!(blocked.to_string().contains("dangerous command"));
    }

    #[test]
    fn execute_only_updates_input_issue_status() {
        let dir = tempdir().unwrap();
        let run = ready_low_risk_run(dir.path(), "iss-001");
        let issue_path = dir.path().join(".agentflow/input/issues/iss-001.json");
        let spec_path = dir
            .path()
            .join(".agentflow/input/specs/approved/spec-001/approval.json");
        let issue_before: InputIssue = crate::storage::read_json(&issue_path).unwrap();
        let spec_before = fs::read_to_string(&spec_path).unwrap();
        run_execute_command(
            dir.path(),
            run.run_id.clone(),
            ExecuteCommandRequest {
                label: "printf ok".to_string(),
                program: "printf".to_string(),
                args: vec!["ok".to_string()],
                source: None,
            },
        )
        .unwrap();
        validate_execute_run(dir.path(), run.run_id).unwrap();
        let issue_after: InputIssue = crate::storage::read_json(&issue_path).unwrap();
        assert_eq!(issue_after.status, InputIssueStatus::InReview);
        assert_eq!(issue_after.issue_id, issue_before.issue_id);
        assert_eq!(issue_after.title, issue_before.title);
        assert_eq!(issue_after.source_spec_id, issue_before.source_spec_id);
        assert_eq!(spec_before, fs::read_to_string(&spec_path).unwrap());
    }
}
