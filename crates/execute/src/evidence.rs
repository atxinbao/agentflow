use crate::{
    manager::assert_build_agent_run,
    model::ExecuteResult,
    storage::{canonical_project_root, load_command_records, read_run, run_dir, write_json},
};
use agentflow_output::{
    OutputCommandEvidence, OutputEvidence, OutputEvidenceExecuteArtifacts, OutputEvidenceInput,
    OutputEvidencePanel, OutputManualProof, OutputValidationSummary, OUTPUT_EVIDENCE_VERSION,
};
use anyhow::Result;
use std::fs;
use std::path::Path;

pub fn write_execute_evidence(
    project_root: impl AsRef<Path>,
    run_id: String,
    result: ExecuteResult,
) -> Result<OutputEvidence> {
    let root = canonical_project_root(project_root)?;
    let run = read_run(&root, &run_id)?;
    assert_build_agent_run(&root, &run)?;
    let run_directory = run_dir(&root, &run_id);
    let command_records = load_command_records(&root, &run_id)?;
    let checkpoint = latest_json_artifact(&run_directory.join("checkpoints")).map(|path| {
        format!(
            ".agentflow/execute/runs/{}/checkpoints/{}",
            run.run_id,
            path.file_name()
                .and_then(|value| value.to_str())
                .unwrap_or_default()
        )
    });
    let diff = artifact_if_exists(
        &run_directory.join("patches/worktree.diff"),
        &format!(
            ".agentflow/execute/runs/{}/patches/worktree.diff",
            run.run_id
        ),
    );
    let changed_files = artifact_if_exists(
        &run_directory.join("patches/changed-files.json"),
        &format!(
            ".agentflow/execute/runs/{}/patches/changed-files.json",
            run.run_id
        ),
    );
    let diff_summary = artifact_if_exists(
        &run_directory.join("review/diff-summary.json"),
        &format!(
            ".agentflow/execute/runs/{}/review/diff-summary.json",
            run.run_id
        ),
    );
    let commands = command_records
        .iter()
        .map(|record| OutputCommandEvidence {
            command_id: record.command_id.clone(),
            label: record.label.clone(),
            exit_code: record.exit_code,
            record_path: format!(
                ".agentflow/execute/runs/{}/commands/{}.json",
                run.run_id, record.command_id
            ),
            stdout_path: Some(record.stdout_path.clone()),
            stderr_path: Some(record.stderr_path.clone()),
        })
        .collect::<Vec<_>>();
    let failed_commands = command_records
        .iter()
        .filter(|record| record.exit_code != Some(0))
        .map(|record| record.command_id.clone())
        .collect::<Vec<_>>();
    let evidence = OutputEvidence {
        version: OUTPUT_EVIDENCE_VERSION.to_string(),
        run_id: run.run_id.clone(),
        issue_id: run.issue_id.clone(),
        source_spec_id: run.source_spec_id.clone(),
        risk_level: run.risk_level.clone(),
        completed_at: crate::storage::unix_timestamp_seconds(),
        summary: format!(
            "Execute run {} finished with status {:?}.",
            run.run_id, result.status
        ),
        input: OutputEvidenceInput {
            issue_path: run.input.issue_path.clone(),
            spec_path: run.input.spec_path.clone(),
        },
        panel: OutputEvidencePanel {
            snapshot_id: run.input.panel_snapshot_id.clone(),
            context_pack_id: run.input.context_pack_id.clone(),
            context_pack_path: run.input.context_pack_path.clone(),
        },
        execute: OutputEvidenceExecuteArtifacts {
            run: format!(".agentflow/execute/runs/{}/run.json", run.run_id),
            preflight: format!(".agentflow/execute/runs/{}/preflight.json", run.run_id),
            plan: format!(".agentflow/execute/runs/{}/plan.json", run.run_id),
            result: format!(".agentflow/execute/runs/{}/result.json", run.run_id),
            checkpoint,
            diff,
            changed_files,
            diff_summary,
        },
        commands,
        validation: OutputValidationSummary {
            passed: result.validation.passed,
            failed_commands,
            skipped: Vec::new(),
        },
        manual_proof: OutputManualProof::default(),
    };
    write_json(
        &root
            .join(".agentflow/output/evidence")
            .join(format!("{}.json", run.run_id)),
        &evidence,
    )?;
    agentflow_output::prepare_output_workspace(&root)?;
    Ok(evidence)
}

fn artifact_if_exists(path: &Path, relative_path: &str) -> Option<String> {
    path.is_file().then(|| relative_path.to_string())
}

fn latest_json_artifact(directory: &Path) -> Option<std::path::PathBuf> {
    let mut paths = fs::read_dir(directory)
        .ok()?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .collect::<Vec<_>>();
    paths.sort();
    paths
        .into_iter()
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("json"))
        .last()
}
