use crate::{
    model::{ExecuteResult, OutputEvidence, OUTPUT_EVIDENCE_VERSION},
    storage::{canonical_project_root, read_run, run_dir, write_json},
};
use anyhow::Result;
use std::{collections::BTreeMap, path::Path};

pub fn write_execute_evidence(
    project_root: impl AsRef<Path>,
    run_id: String,
    result: ExecuteResult,
) -> Result<OutputEvidence> {
    let root = canonical_project_root(project_root)?;
    let run = read_run(&root, &run_id)?;
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
        changed_files: result.changed_files.clone(),
        commands: result.commands.clone(),
        validation_passed: result.validation.passed,
        artifacts: BTreeMap::from([
            (
                "run".to_string(),
                format!(".agentflow/execute/runs/{}/run.json", run.run_id),
            ),
            (
                "preflight".to_string(),
                format!(".agentflow/execute/runs/{}/preflight.json", run.run_id),
            ),
            (
                "result".to_string(),
                format!(".agentflow/execute/runs/{}/result.json", run.run_id),
            ),
            (
                "diff".to_string(),
                format!(
                    ".agentflow/execute/runs/{}/patches/worktree.diff",
                    run.run_id
                ),
            ),
        ]),
    };
    write_json(
        &root
            .join(".agentflow/output/evidence")
            .join(format!("{}.json", run.run_id)),
        &evidence,
    )?;
    write_json(&run_dir(&root, &run_id).join("result.json"), &result)?;
    Ok(evidence)
}
