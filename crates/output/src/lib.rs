pub mod audit;
pub mod manager;
pub mod model;
pub mod storage;
pub mod validate;

pub use audit::{
    ensure_release_auto_audits, load_audit_index, load_audit_manifest, load_audit_report,
    load_audit_status, prepare_audit_workspace, request_human_audit,
};
pub use manager::{
    load_output_index, load_output_manifest, load_output_snapshot, load_output_status,
    prepare_output_workspace, validate_output,
};
pub use model::*;
pub use validate::{
    load_output_evidence, load_release_delivery, validate_output_evidence,
    validate_release_delivery,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{ensure_directory, read_json, write_json};
    use serde_json::json;
    use std::fs;
    use tempfile::tempdir;

    fn valid_evidence(run_id: &str) -> OutputEvidence {
        valid_evidence_with_risk(run_id, "medium")
    }

    fn valid_evidence_with_risk(run_id: &str, risk_level: &str) -> OutputEvidence {
        OutputEvidence {
            version: OUTPUT_EVIDENCE_VERSION.to_string(),
            run_id: run_id.to_string(),
            issue_id: "iss-001".to_string(),
            source_spec_id: "spec-001".to_string(),
            risk_level: risk_level.to_string(),
            completed_at: 1,
            summary: "Evidence fixture".to_string(),
            input: OutputEvidenceInput {
                issue_path: ".agentflow/input/issues/iss-001.json".to_string(),
                spec_path: ".agentflow/input/specs/approved/spec-001".to_string(),
            },
            panel: OutputEvidencePanel {
                snapshot_id: Some("panel-snapshot-001".to_string()),
                context_pack_id: Some("ctx-001".to_string()),
            },
            execute: OutputEvidenceExecuteArtifacts {
                run: format!(".agentflow/execute/runs/{run_id}/run.json"),
                preflight: format!(".agentflow/execute/runs/{run_id}/preflight.json"),
                plan: format!(".agentflow/execute/runs/{run_id}/plan.json"),
                result: format!(".agentflow/execute/runs/{run_id}/result.json"),
                checkpoint: Some(format!(
                    ".agentflow/execute/runs/{run_id}/checkpoints/chk-001.json"
                )),
                diff: Some(format!(
                    ".agentflow/execute/runs/{run_id}/patches/worktree.diff"
                )),
                changed_files: Some(format!(
                    ".agentflow/execute/runs/{run_id}/patches/changed-files.json"
                )),
                diff_summary: Some(format!(
                    ".agentflow/execute/runs/{run_id}/review/diff-summary.json"
                )),
            },
            commands: vec![OutputCommandEvidence {
                command_id: "cmd-001".to_string(),
                label: "cargo test".to_string(),
                exit_code: Some(0),
                record_path: format!(".agentflow/execute/runs/{run_id}/commands/cmd-001.json"),
                stdout_path: Some(format!(
                    ".agentflow/execute/runs/{run_id}/commands/cmd-001.stdout.txt"
                )),
                stderr_path: Some(format!(
                    ".agentflow/execute/runs/{run_id}/commands/cmd-001.stderr.txt"
                )),
            }],
            validation: OutputValidationSummary {
                passed: true,
                failed_commands: Vec::new(),
                skipped: Vec::new(),
            },
            manual_proof: OutputManualProof::default(),
        }
    }

    fn write_execute_artifacts(root: &std::path::Path, run_id: &str) {
        let run_dir = root.join(".agentflow/execute/runs").join(run_id);
        ensure_directory(&run_dir.join("commands")).unwrap();
        ensure_directory(&run_dir.join("checkpoints")).unwrap();
        ensure_directory(&run_dir.join("patches")).unwrap();
        ensure_directory(&run_dir.join("review")).unwrap();
        fs::write(run_dir.join("run.json"), "{}\n").unwrap();
        fs::write(run_dir.join("preflight.json"), "{}\n").unwrap();
        fs::write(
            run_dir.join("plan.json"),
            serde_json::to_string_pretty(&json!({
                "version": "execute-plan.v1",
                "runId": run_id,
                "issueId": "iss-001",
                "steps": [],
                "allowedWritePaths": ["src"],
                "allowedCommands": ["cargo test"]
            }))
            .unwrap(),
        )
        .unwrap();
        fs::write(
            run_dir.join("result.json"),
            serde_json::to_string_pretty(&json!({
                "validation": { "passed": true }
            }))
            .unwrap(),
        )
        .unwrap();
        fs::write(run_dir.join("checkpoints/chk-001.json"), "{}\n").unwrap();
        fs::write(run_dir.join("patches/worktree.diff"), "diff\n").unwrap();
        fs::write(
            run_dir.join("patches/changed-files.json"),
            serde_json::to_string_pretty(&json!({
                "version": "execute-changed-files.v1",
                "runId": run_id,
                "files": [
                    {
                        "path": "src/lib.rs",
                        "changeType": "modified",
                        "insertions": 1,
                        "deletions": 0
                    }
                ]
            }))
            .unwrap(),
        )
        .unwrap();
        fs::write(run_dir.join("review/diff-summary.json"), "{}\n").unwrap();
        fs::write(run_dir.join("commands/cmd-001.json"), "{}\n").unwrap();
        fs::write(run_dir.join("commands/cmd-001.stdout.txt"), "ok\n").unwrap();
        fs::write(run_dir.join("commands/cmd-001.stderr.txt"), "").unwrap();
    }

    fn write_evidence(root: &std::path::Path, run_id: &str) {
        write_execute_artifacts(root, run_id);
        write_json(
            &root
                .join(".agentflow/output/evidence")
                .join(format!("{run_id}.json")),
            &valid_evidence(run_id),
        )
        .unwrap();
    }

    fn write_evidence_with_risk(root: &std::path::Path, run_id: &str, risk_level: &str) {
        write_execute_artifacts(root, run_id);
        write_json(
            &root
                .join(".agentflow/output/evidence")
                .join(format!("{run_id}.json")),
            &valid_evidence_with_risk(run_id, risk_level),
        )
        .unwrap();
    }

    fn write_release_delivery(root: &std::path::Path, run_id: &str) {
        write_release_delivery_with_risk(root, run_id, "medium");
    }

    fn write_release_delivery_with_risk(root: &std::path::Path, run_id: &str, risk_level: &str) {
        let dir = root.join(".agentflow/output/release").join(run_id);
        ensure_directory(&dir).unwrap();
        let relative_dir = format!(".agentflow/output/release/{run_id}");
        let delivery = OutputReleaseDelivery {
            version: OUTPUT_RELEASE_DELIVERY_VERSION.to_string(),
            run_id: run_id.to_string(),
            issue_id: "iss-001".to_string(),
            source_spec_id: "spec-001".to_string(),
            risk_level: risk_level.to_string(),
            status: "drafted".to_string(),
            created_by: "Build Agent".to_string(),
            created_at: 1,
            evidence_path: format!(".agentflow/output/evidence/{run_id}.json"),
            execute_result_path: format!(".agentflow/execute/runs/{run_id}/result.json"),
            diff_summary_path: Some(format!(
                ".agentflow/execute/runs/{run_id}/review/diff-summary.json"
            )),
            artifacts: OutputReleaseDeliveryArtifacts {
                pr_draft: format!("{relative_dir}/pr-draft.md"),
                pr_metadata: format!("{relative_dir}/pr-metadata.json"),
                review_checklist: format!("{relative_dir}/review-checklist.md"),
                changelog: format!("{relative_dir}/changelog.md"),
                release_note: format!("{relative_dir}/release-note.md"),
            },
        };
        write_json(&dir.join("delivery.json"), &delivery).unwrap();
        write_json(
            &dir.join("pr-metadata.json"),
            &OutputPrMetadata {
                version: OUTPUT_PR_METADATA_VERSION.to_string(),
                run_id: run_id.to_string(),
                issue_id: "iss-001".to_string(),
                source_spec_id: "spec-001".to_string(),
                title: "Implement fixture issue".to_string(),
                branch_name: None,
                remote_pr_url: None,
                status: "draft-only".to_string(),
                created_remote_pr: false,
            },
        )
        .unwrap();
        for file in [
            "pr-draft.md",
            "review-checklist.md",
            "changelog.md",
            "release-note.md",
        ] {
            fs::write(dir.join(file), "# fixture\n").unwrap();
        }
    }

    fn audit_request(run_id: &str) -> HumanAuditRequestDraft {
        HumanAuditRequestDraft {
            reason: "Human requested audit before accepting delivery.".to_string(),
            scope: AuditScope {
                description: "Review Build Agent delivery evidence chain.".to_string(),
                refs: vec![
                    AuditScopeRef {
                        kind: "spec".to_string(),
                        id: "spec-001".to_string(),
                        path: ".agentflow/input/specs/approved/spec-001/".to_string(),
                    },
                    AuditScopeRef {
                        kind: "issue".to_string(),
                        id: "iss-001".to_string(),
                        path: ".agentflow/input/issues/iss-001.json".to_string(),
                    },
                    AuditScopeRef {
                        kind: "execute-run".to_string(),
                        id: run_id.to_string(),
                        path: format!(".agentflow/execute/runs/{run_id}/"),
                    },
                    AuditScopeRef {
                        kind: "evidence".to_string(),
                        id: run_id.to_string(),
                        path: format!(".agentflow/output/evidence/{run_id}.json"),
                    },
                    AuditScopeRef {
                        kind: "release-delivery".to_string(),
                        id: run_id.to_string(),
                        path: format!(".agentflow/output/release/{run_id}/delivery.json"),
                    },
                ],
            },
        }
    }

    #[test]
    fn prepare_output_workspace_creates_manifest_index_and_required_directories() {
        let dir = tempdir().unwrap();
        let snapshot = prepare_output_workspace(dir.path()).unwrap();

        assert!(snapshot.ready);
        assert!(dir.path().join(".agentflow/output/manifest.json").is_file());
        assert!(dir.path().join(".agentflow/output/index.json").is_file());
        for path in OUTPUT_DIRECTORIES {
            assert!(dir.path().join(path).is_dir(), "{path}");
        }
    }

    #[test]
    fn output_status_is_ready_after_prepare() {
        let dir = tempdir().unwrap();
        prepare_output_workspace(dir.path()).unwrap();
        let status = load_output_status(dir.path()).unwrap();

        assert!(status.ready);
        assert_eq!(status.status, OutputWorkspaceStatus::Ready);
    }

    #[test]
    fn evidence_validation_passes_with_valid_references() {
        let dir = tempdir().unwrap();
        prepare_output_workspace(dir.path()).unwrap();
        write_evidence(dir.path(), "run-001");

        assert!(
            validate_output_evidence(dir.path(), "run-001")
                .unwrap()
                .valid
        );
    }

    #[test]
    fn evidence_validation_fails_when_referenced_run_is_missing() {
        let dir = tempdir().unwrap();
        prepare_output_workspace(dir.path()).unwrap();
        write_json(
            &dir.path().join(".agentflow/output/evidence/run-001.json"),
            &valid_evidence("run-001"),
        )
        .unwrap();

        let validation = validate_output_evidence(dir.path(), "run-001").unwrap();

        assert!(!validation.valid);
        assert!(validation
            .errors
            .iter()
            .any(|error| error.contains("run.json")));
    }

    #[test]
    fn release_delivery_validation_passes_with_required_files() {
        let dir = tempdir().unwrap();
        prepare_output_workspace(dir.path()).unwrap();
        write_evidence(dir.path(), "run-001");
        write_release_delivery(dir.path(), "run-001");

        assert!(
            validate_release_delivery(dir.path(), "run-001")
                .unwrap()
                .valid
        );
    }

    #[test]
    fn release_delivery_validation_fails_when_pr_draft_is_missing() {
        let dir = tempdir().unwrap();
        prepare_output_workspace(dir.path()).unwrap();
        write_evidence(dir.path(), "run-001");
        write_release_delivery(dir.path(), "run-001");
        fs::remove_file(
            dir.path()
                .join(".agentflow/output/release/run-001/pr-draft.md"),
        )
        .unwrap();

        let validation = validate_release_delivery(dir.path(), "run-001").unwrap();

        assert!(!validation.valid);
        assert!(validation
            .errors
            .iter()
            .any(|error| error.contains("pr-draft.md")));
    }

    #[test]
    fn delivery_must_reference_existing_evidence() {
        let dir = tempdir().unwrap();
        prepare_output_workspace(dir.path()).unwrap();
        write_release_delivery(dir.path(), "run-001");

        let validation = validate_release_delivery(dir.path(), "run-001").unwrap();

        assert!(!validation.valid);
        assert!(validation
            .errors
            .iter()
            .any(|error| error.contains("evidence")));
    }

    #[test]
    fn prepare_audit_space_only_creates_manifest_and_index() {
        let dir = tempdir().unwrap();
        prepare_output_workspace(dir.path()).unwrap();

        assert!(dir
            .path()
            .join(".agentflow/output/audit/manifest.json")
            .is_file());
        assert!(dir
            .path()
            .join(".agentflow/output/audit/index.json")
            .is_file());
        assert!(!dir
            .path()
            .join(".agentflow/output/audit/audit-001")
            .exists());
    }

    #[test]
    fn request_human_audit_writes_complete_report_package() {
        let dir = tempdir().unwrap();
        prepare_output_workspace(dir.path()).unwrap();
        write_evidence(dir.path(), "run-001");
        write_release_delivery(dir.path(), "run-001");

        let report = request_human_audit(dir.path(), audit_request("run-001")).unwrap();

        assert_eq!(report.audit.audit_id, "audit-001");
        assert_eq!(report.audit.status, AuditStatus::Passed);
        for file in [
            "audit-request.json",
            "audit.json",
            "audit-report.md",
            "findings.json",
            "checklist.md",
            "evidence-map.json",
            "traceability.json",
        ] {
            assert!(
                dir.path()
                    .join(".agentflow/output/audit/audit-001")
                    .join(file)
                    .is_file(),
                "{file}"
            );
        }
        let index = load_audit_index(dir.path()).unwrap();
        assert_eq!(index.audits.len(), 1);
    }

    #[test]
    fn prepare_output_workspace_creates_release_auto_audit_request() {
        let dir = tempdir().unwrap();
        prepare_output_workspace(dir.path()).unwrap();
        write_evidence(dir.path(), "run-001");
        write_release_delivery(dir.path(), "run-001");

        prepare_output_workspace(dir.path()).unwrap();

        let index = load_audit_index(dir.path()).unwrap();
        assert_eq!(index.audits.len(), 1);
        let audit = &index.audits[0];
        assert_eq!(audit.status, AuditStatus::Requested);
        assert_eq!(audit.trigger, AuditTrigger::ReleaseAuto);
        assert_eq!(audit.source_run_id.as_deref(), Some("run-001"));
        assert!(dir
            .path()
            .join(".agentflow/output/audit/audit-001/audit-request.json")
            .is_file());
        let audit_issue_path = dir
            .path()
            .join(".agentflow/input/issues/audit-run-001.json");
        assert!(audit_issue_path.is_file());
        let audit_issue: agentflow_input::issue::InputIssue = read_json(&audit_issue_path).unwrap();
        assert_eq!(audit_issue.issue_category.as_str(), "audit");
        assert_eq!(audit_issue.required_agent_role.as_str(), "audit-agent");
        assert!(!dir
            .path()
            .join(".agentflow/output/audit/audit-001/audit.json")
            .exists());
    }

    #[test]
    fn release_auto_audit_request_is_idempotent() {
        let dir = tempdir().unwrap();
        prepare_output_workspace(dir.path()).unwrap();
        write_evidence(dir.path(), "run-001");
        write_release_delivery(dir.path(), "run-001");

        prepare_output_workspace(dir.path()).unwrap();
        prepare_output_workspace(dir.path()).unwrap();

        let index = load_audit_index(dir.path()).unwrap();
        assert_eq!(index.audits.len(), 1);
        assert_eq!(index.audits[0].trigger, AuditTrigger::ReleaseAuto);
    }

    #[test]
    fn prepare_output_workspace_backfills_issue_for_existing_release_auto_request() {
        let dir = tempdir().unwrap();
        let audit_dir = dir
            .path()
            .join(".agentflow/output/audit/audit-release-v0.1.0");
        ensure_directory(&audit_dir).unwrap();
        write_json(
            &audit_dir.join("audit-request.json"),
            &AuditRequest {
                version: AUDIT_REQUEST_VERSION.to_string(),
                audit_id: "audit-release-v0.1.0".to_string(),
                trigger: AuditTrigger::ReleaseAuto,
                requested_by: "agentflow-release-auto".to_string(),
                requested_at: 1,
                reason: "Release 已生成，AgentFlow 规则要求进行审计。".to_string(),
                source: Some(AuditRequestSource {
                    kind: "release-delivery".to_string(),
                    delivery_id: Some("release-v0.1.0".to_string()),
                    run_id: Some("release-v0.1.0".to_string()),
                    issue_id: Some("AF-DOGFOOD-001".to_string()),
                    spec_id: Some("dogfood-cutover-v1".to_string()),
                }),
                scope: AuditScope {
                    description: "审计 release delivery。".to_string(),
                    refs: vec![
                        AuditScopeRef {
                            kind: "spec".to_string(),
                            id: "dogfood-cutover-v1".to_string(),
                            path: ".agentflow/input/specs/approved/dogfood-cutover-v1/spec.json"
                                .to_string(),
                        },
                        AuditScopeRef {
                            kind: "release-delivery".to_string(),
                            id: "release-v0.1.0".to_string(),
                            path: ".agentflow/output/release/release-v0.1.0/delivery.json"
                                .to_string(),
                        },
                    ],
                },
            },
        )
        .unwrap();

        prepare_output_workspace(dir.path()).unwrap();

        let audit_issue_path = dir
            .path()
            .join(".agentflow/input/issues/audit-release-v0.1.0.json");
        assert!(audit_issue_path.is_file());
        let audit_issue: agentflow_input::issue::InputIssue = read_json(&audit_issue_path).unwrap();
        assert_eq!(audit_issue.source_spec_id, "dogfood-cutover-v1");
        assert_eq!(audit_issue.issue_category.as_str(), "audit");
        assert_eq!(audit_issue.required_agent_role.as_str(), "audit-agent");
        assert_eq!(audit_issue.display_status.as_str(), "ready");
    }

    #[test]
    fn audit_checks_fail_when_checkpoint_is_missing() {
        let dir = tempdir().unwrap();
        prepare_output_workspace(dir.path()).unwrap();
        write_evidence(dir.path(), "run-001");
        write_release_delivery(dir.path(), "run-001");
        fs::remove_file(
            dir.path()
                .join(".agentflow/execute/runs/run-001/checkpoints/chk-001.json"),
        )
        .unwrap();

        let report = request_human_audit(dir.path(), audit_request("run-001")).unwrap();

        assert_eq!(report.audit.status, AuditStatus::Failed);
        assert_eq!(
            report.audit.checks.checkpoint_exists,
            AuditCheckStatus::Failed
        );
    }

    #[test]
    fn audit_checks_fail_when_changed_files_are_missing() {
        let dir = tempdir().unwrap();
        prepare_output_workspace(dir.path()).unwrap();
        write_evidence(dir.path(), "run-001");
        write_release_delivery(dir.path(), "run-001");
        fs::remove_file(
            dir.path()
                .join(".agentflow/execute/runs/run-001/patches/changed-files.json"),
        )
        .unwrap();

        let report = request_human_audit(dir.path(), audit_request("run-001")).unwrap();

        assert_eq!(
            report.audit.checks.changed_files_recorded,
            AuditCheckStatus::Failed
        );
    }

    #[test]
    fn audit_checks_fail_when_changed_file_is_outside_allowed_write_paths() {
        let dir = tempdir().unwrap();
        prepare_output_workspace(dir.path()).unwrap();
        write_evidence(dir.path(), "run-001");
        write_release_delivery(dir.path(), "run-001");
        fs::write(
            dir.path()
                .join(".agentflow/execute/runs/run-001/patches/changed-files.json"),
            serde_json::to_string_pretty(&json!({
                "version": "execute-changed-files.v1",
                "runId": "run-001",
                "files": [
                    {
                        "path": "README.md",
                        "changeType": "modified",
                        "insertions": 1,
                        "deletions": 0
                    }
                ]
            }))
            .unwrap(),
        )
        .unwrap();

        let report = request_human_audit(dir.path(), audit_request("run-001")).unwrap();

        assert_eq!(
            report.audit.checks.allowed_write_paths_only,
            AuditCheckStatus::Failed
        );
    }

    #[test]
    fn audit_checks_fail_when_command_record_is_incomplete() {
        let dir = tempdir().unwrap();
        prepare_output_workspace(dir.path()).unwrap();
        write_evidence(dir.path(), "run-001");
        write_release_delivery(dir.path(), "run-001");
        fs::remove_file(
            dir.path()
                .join(".agentflow/execute/runs/run-001/commands/cmd-001.stdout.txt"),
        )
        .unwrap();

        let report = request_human_audit(dir.path(), audit_request("run-001")).unwrap();

        assert_eq!(
            report.audit.checks.commands_recorded,
            AuditCheckStatus::Failed
        );
    }

    #[test]
    fn high_risk_issue_without_confirmation_fails_audit() {
        let dir = tempdir().unwrap();
        prepare_output_workspace(dir.path()).unwrap();
        write_evidence_with_risk(dir.path(), "run-001", "high");
        write_release_delivery_with_risk(dir.path(), "run-001", "high");

        let report = request_human_audit(dir.path(), audit_request("run-001")).unwrap();

        assert_eq!(
            report.audit.checks.high_risk_confirmed_if_needed,
            AuditCheckStatus::Failed
        );
    }

    #[test]
    fn missing_evidence_fails_audit() {
        let dir = tempdir().unwrap();
        prepare_output_workspace(dir.path()).unwrap();
        write_execute_artifacts(dir.path(), "run-001");
        write_release_delivery(dir.path(), "run-001");

        let report = request_human_audit(dir.path(), audit_request("run-001")).unwrap();

        assert_eq!(
            report.audit.checks.evidence_complete,
            AuditCheckStatus::Failed
        );
    }

    #[test]
    fn missing_release_delivery_fails_audit() {
        let dir = tempdir().unwrap();
        prepare_output_workspace(dir.path()).unwrap();
        write_evidence(dir.path(), "run-001");

        let report = request_human_audit(dir.path(), audit_request("run-001")).unwrap();

        assert_eq!(
            report.audit.checks.release_delivery_complete,
            AuditCheckStatus::Failed
        );
    }

    #[test]
    fn audit_does_not_modify_input_execute_evidence_or_release() {
        let dir = tempdir().unwrap();
        prepare_output_workspace(dir.path()).unwrap();
        write_evidence(dir.path(), "run-001");
        write_release_delivery(dir.path(), "run-001");
        let evidence_path = dir.path().join(".agentflow/output/evidence/run-001.json");
        let delivery_path = dir
            .path()
            .join(".agentflow/output/release/run-001/delivery.json");
        let plan_path = dir.path().join(".agentflow/execute/runs/run-001/plan.json");
        let evidence_before = fs::read_to_string(&evidence_path).unwrap();
        let delivery_before = fs::read_to_string(&delivery_path).unwrap();
        let plan_before = fs::read_to_string(&plan_path).unwrap();

        request_human_audit(dir.path(), audit_request("run-001")).unwrap();

        assert_eq!(fs::read_to_string(evidence_path).unwrap(), evidence_before);
        assert_eq!(fs::read_to_string(delivery_path).unwrap(), delivery_before);
        assert_eq!(fs::read_to_string(plan_path).unwrap(), plan_before);
    }

    #[test]
    fn output_prepare_does_not_write_input_panel_or_execute() {
        let dir = tempdir().unwrap();
        prepare_output_workspace(dir.path()).unwrap();

        assert!(!dir.path().join(".agentflow/input").exists());
        assert!(!dir.path().join(".agentflow/panel").exists());
        assert!(!dir.path().join(".agentflow/execute").exists());
    }
}
