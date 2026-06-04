use crate::{
    model::{
        ExecuteChangedFiles, ExecuteDiffSummary, ExecuteResult, ExecuteResultNext,
        ExecuteReviewState, ExecuteRunStatus, ExecuteValidationResult, EXECUTE_RESULT_VERSION,
    },
    storage::{load_command_records, read_json, run_dir, write_json},
};
use anyhow::Result;
use std::path::Path;

pub(crate) fn build_execute_result(
    root: &Path,
    run_id: &str,
    passed: bool,
) -> Result<ExecuteResult> {
    let run = crate::storage::read_run(root, run_id)?;
    let changed = read_changed_files(root, run_id).unwrap_or_else(|_| ExecuteChangedFiles {
        version: "execute-changed-files.v1".to_string(),
        run_id: run_id.to_string(),
        files: Vec::new(),
    });
    let commands = load_command_records(root, run_id)?;
    let command_ids = commands
        .iter()
        .map(|record| record.command_id.clone())
        .collect::<Vec<_>>();
    let changed_paths = changed
        .files
        .iter()
        .map(|file| file.path.clone())
        .collect::<Vec<_>>();
    let evidence_path = format!(".agentflow/output/evidence/{run_id}.json");

    let result = ExecuteResult {
        version: EXECUTE_RESULT_VERSION.to_string(),
        run_id: run_id.to_string(),
        issue_id: run.issue_id,
        status: if passed {
            ExecuteRunStatus::Completed
        } else {
            ExecuteRunStatus::Failed
        },
        risk_level: run.risk_level,
        changed_files: changed_paths,
        commands: command_ids,
        validation: ExecuteValidationResult {
            passed,
            evidence_path,
        },
        next: ExecuteResultNext {
            ready_for_delivery: passed,
            needs_audit: true,
        },
    };
    write_review_artifacts(root, run_id, &changed, &result)?;
    Ok(result)
}

fn read_changed_files(root: &Path, run_id: &str) -> Result<ExecuteChangedFiles> {
    read_json(&run_dir(root, run_id).join("patches/changed-files.json"))
}

fn write_review_artifacts(
    root: &Path,
    run_id: &str,
    changed: &ExecuteChangedFiles,
    result: &ExecuteResult,
) -> Result<()> {
    let insertions = changed.files.iter().map(|file| file.insertions).sum();
    let deletions = changed.files.iter().map(|file| file.deletions).sum();
    let diff_summary = ExecuteDiffSummary {
        version: "execute-diff-summary.v1".to_string(),
        run_id: run_id.to_string(),
        changed_files: changed.files.len(),
        insertions,
        deletions,
        risk_level: result.risk_level.clone(),
        notes: vec!["Patch only touched allowed write paths.".to_string()],
    };
    let review_state = ExecuteReviewState {
        version: "execute-review-state.v1".to_string(),
        run_id: run_id.to_string(),
        status: "pending-review".to_string(),
        hunk_review_enabled: false,
        notes: vec![
            "V1 stores reviewable diff artifacts but does not implement hunk accept/reject UI."
                .to_string(),
        ],
    };
    let review_dir = run_dir(root, run_id).join("review");
    write_json(&review_dir.join("diff-summary.json"), &diff_summary)?;
    write_json(&review_dir.join("review-state.json"), &review_state)?;
    Ok(())
}
