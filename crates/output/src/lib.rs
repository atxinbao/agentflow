pub mod audit;
pub mod manager;
pub mod model;
pub mod storage;
pub mod validate;

pub use audit::{create_audit_skeleton, load_audit_output};
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
    use crate::storage::{ensure_directory, write_json};
    use serde_json::json;
    use std::fs;
    use tempfile::tempdir;

    fn valid_evidence(run_id: &str) -> OutputEvidence {
        OutputEvidence {
            version: OUTPUT_EVIDENCE_VERSION.to_string(),
            run_id: run_id.to_string(),
            issue_id: "iss-001".to_string(),
            source_spec_id: "spec-001".to_string(),
            risk_level: "medium".to_string(),
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
        fs::write(run_dir.join("plan.json"), "{}\n").unwrap();
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
        fs::write(run_dir.join("patches/changed-files.json"), "{}\n").unwrap();
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

    fn write_release_delivery(root: &std::path::Path, run_id: &str) {
        let dir = root.join(".agentflow/output/release").join(run_id);
        ensure_directory(&dir).unwrap();
        let relative_dir = format!(".agentflow/output/release/{run_id}");
        let delivery = OutputReleaseDelivery {
            version: OUTPUT_RELEASE_DELIVERY_VERSION.to_string(),
            run_id: run_id.to_string(),
            issue_id: "iss-001".to_string(),
            source_spec_id: "spec-001".to_string(),
            risk_level: "medium".to_string(),
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
    fn output_audit_skeleton_can_be_created_for_run() {
        let dir = tempdir().unwrap();
        prepare_output_workspace(dir.path()).unwrap();
        let audit = create_audit_skeleton(dir.path(), "run-001".to_string()).unwrap();

        assert_eq!(audit.status, "pending");
        assert_eq!(audit.created_by, "Audit Agent");
        assert!(dir
            .path()
            .join(".agentflow/output/audit/run-001/audit.json")
            .is_file());
        assert!(dir
            .path()
            .join(".agentflow/output/audit/run-001/audit-report.md")
            .is_file());
    }

    #[test]
    fn audit_skeleton_does_not_execute_audit() {
        let dir = tempdir().unwrap();
        prepare_output_workspace(dir.path()).unwrap();
        let audit = create_audit_skeleton(dir.path(), "run-001".to_string()).unwrap();

        assert_eq!(audit.status, "pending");
        assert!(audit.findings.is_empty());
        assert!(audit.checks.spec_aligned.is_none());
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
