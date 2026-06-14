use crate::{
    manager::{assert_build_agent_run, sync_issue_loop_projection, update_input_issue_status},
    model::{ExecuteResult, ExecuteRunStatus},
    storage::{
        canonical_project_root, ensure_directory, read_json, read_run, run_dir,
        unix_timestamp_seconds, write_json,
    },
};
use agentflow_input::issue::InputIssueStatus;
use agentflow_output::{
    OutputPrMetadata, OutputReleaseDelivery, OutputReleaseDeliveryArtifacts,
    OUTPUT_PR_METADATA_VERSION, OUTPUT_RELEASE_DELIVERY_VERSION,
};
use agentflow_task_artifacts::TaskEvidence;
use anyhow::{Context, Result};
use std::{fs, path::Path};

#[derive(Debug, Default)]
struct ReleasePrMetadataSource {
    branch_name: Option<String>,
    remote_pr_url: Option<String>,
    provider: Option<String>,
    merge_mode: Option<String>,
    merged: bool,
    checked_at: Option<u64>,
}

pub fn prepare_release_delivery(
    project_root: impl AsRef<Path>,
    run_id: String,
) -> Result<OutputReleaseDelivery> {
    let root = canonical_project_root(project_root)?;
    let run = read_run(&root, &run_id)?;
    assert_build_agent_run(&root, &run)?;
    if !matches!(run.status, ExecuteRunStatus::Completed) {
        anyhow::bail!("release delivery requires a completed execute run");
    }

    let run_directory = run_dir(&root, &run_id);
    let result_path = run_directory.join("result.json");
    let result: ExecuteResult = read_json(&result_path)?;
    if !matches!(result.status, ExecuteRunStatus::Completed) {
        anyhow::bail!("release delivery requires a completed execute result");
    }

    let evidence_relative_path =
        format!(".agentflow/tasks/{}/evidence/evidence.json", run.issue_id);
    let evidence_path = root.join(&evidence_relative_path);
    let _evidence: TaskEvidence = read_json(&evidence_path)?;

    let changed_files_path = run_directory.join("patches/changed-files.json");
    if !changed_files_path.is_file() {
        anyhow::bail!("release delivery requires changed-files summary");
    }

    let release_relative_dir = format!(".agentflow/output/release/{run_id}");
    let release_dir = root.join(&release_relative_dir);
    ensure_directory(&release_dir)?;
    let execute_result_relative_path = format!(".agentflow/execute/runs/{run_id}/result.json");
    let diff_summary_relative_path =
        format!(".agentflow/execute/runs/{run_id}/review/diff-summary.json");
    let diff_summary_path = root.join(&diff_summary_relative_path);
    let pr_source = release_pr_metadata_source(&root, &run_id);
    let pr_status = if pr_source.merged {
        "merged"
    } else if pr_source.remote_pr_url.is_some() {
        "opened"
    } else {
        "draft-only"
    };

    let artifacts = OutputReleaseDeliveryArtifacts {
        pr_draft: format!("{release_relative_dir}/pr-draft.md"),
        pr_metadata: format!("{release_relative_dir}/pr-metadata.json"),
        review_checklist: format!("{release_relative_dir}/review-checklist.md"),
        changelog: format!("{release_relative_dir}/changelog.md"),
        release_note: format!("{release_relative_dir}/release-note.md"),
    };

    fs::write(
        release_dir.join("pr-draft.md"),
        pr_draft_content(&run_id, &run.issue_id, &result),
    )
    .with_context(|| format!("write {}/pr-draft.md", release_dir.display()))?;
    write_json(
        &release_dir.join("pr-metadata.json"),
        &OutputPrMetadata {
            version: OUTPUT_PR_METADATA_VERSION.to_string(),
            run_id: run_id.clone(),
            issue_id: run.issue_id.clone(),
            source_spec_id: run.source_spec_id.clone(),
            title: format!("Implement {}", run.issue_id),
            branch_name: pr_source.branch_name,
            remote_pr_url: pr_source.remote_pr_url.clone(),
            status: pr_status.to_string(),
            created_remote_pr: pr_source.remote_pr_url.is_some(),
            provider: pr_source.provider,
            merge_mode: pr_source.merge_mode,
            merged: pr_source.merged,
            checked_at: pr_source.checked_at,
        },
    )?;
    fs::write(
        release_dir.join("review-checklist.md"),
        review_checklist_content(),
    )
    .with_context(|| format!("write {}/review-checklist.md", release_dir.display()))?;
    fs::write(
        release_dir.join("changelog.md"),
        changelog_content(&run.issue_id),
    )
    .with_context(|| format!("write {}/changelog.md", release_dir.display()))?;
    fs::write(
        release_dir.join("release-note.md"),
        release_note_content(&run.issue_id),
    )
    .with_context(|| format!("write {}/release-note.md", release_dir.display()))?;

    let delivery = OutputReleaseDelivery {
        version: OUTPUT_RELEASE_DELIVERY_VERSION.to_string(),
        run_id: run_id.clone(),
        issue_id: run.issue_id.clone(),
        source_spec_id: run.source_spec_id.clone(),
        risk_level: run.risk_level.clone(),
        status: "drafted".to_string(),
        created_by: "Build Agent".to_string(),
        created_at: unix_timestamp_seconds(),
        evidence_path: evidence_relative_path,
        execute_result_path: execute_result_relative_path,
        diff_summary_path: diff_summary_path
            .is_file()
            .then_some(diff_summary_relative_path),
        artifacts,
    };
    write_json(&release_dir.join("delivery.json"), &delivery)?;
    agentflow_output::prepare_output_workspace(&root)?;
    update_input_issue_status(&root, &delivery.issue_id, InputIssueStatus::InReview)?;
    sync_issue_loop_projection(
        &root,
        &run,
        InputIssueStatus::InReview,
        Some("delivery-prepared".to_string()),
        Vec::new(),
    )?;
    Ok(delivery)
}

fn release_pr_metadata_source(root: &Path, run_id: &str) -> ReleasePrMetadataSource {
    let run_directory = run_dir(root, run_id);
    let branch_name = read_json::<serde_json::Value>(&run_directory.join("branch.json"))
        .ok()
        .and_then(|value| {
            value
                .get("issueBranch")
                .and_then(serde_json::Value::as_str)
                .map(str::to_string)
        });
    let proof = read_json::<serde_json::Value>(&run_directory.join("review/merge-proof.json")).ok();
    ReleasePrMetadataSource {
        branch_name,
        remote_pr_url: proof
            .as_ref()
            .and_then(|value| value.get("remoteUrl"))
            .and_then(serde_json::Value::as_str)
            .map(str::to_string),
        provider: proof
            .as_ref()
            .and_then(|value| value.get("provider"))
            .and_then(serde_json::Value::as_str)
            .map(str::to_string),
        merge_mode: proof
            .as_ref()
            .and_then(|value| value.get("mergeMode"))
            .and_then(serde_json::Value::as_str)
            .map(str::to_string),
        merged: proof
            .as_ref()
            .and_then(|value| value.get("merged"))
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false),
        checked_at: proof
            .as_ref()
            .and_then(|value| value.get("checkedAt"))
            .and_then(serde_json::Value::as_u64),
    }
}

pub fn load_release_delivery(
    project_root: impl AsRef<Path>,
    run_id: String,
) -> Result<OutputReleaseDelivery> {
    agentflow_output::load_release_delivery(project_root, run_id)
}

fn pr_draft_content(run_id: &str, issue_id: &str, result: &ExecuteResult) -> String {
    format!(
        "# PR Draft\n\nRun: `{run_id}`\nIssue: `{issue_id}`\n\n## Summary\n- Build Agent prepared delivery artifacts from execute result and evidence.\n\n## Changed Files\n{}\n\n## Validation\n- passed: {}\n",
        result
            .changed_files
            .iter()
            .map(|path| format!("- `{path}`"))
            .collect::<Vec<_>>()
            .join("\n"),
        result.validation.passed
    )
}

fn review_checklist_content() -> &'static str {
    "# Review Checklist\n\n- [ ] Execute result exists\n- [ ] Evidence exists\n- [ ] Changed files summary exists\n- [ ] Validation result recorded\n- [ ] No merge or deploy performed\n"
}

fn changelog_content(issue_id: &str) -> String {
    format!("# Changelog Entry\n\n- Prepared delivery artifacts for `{issue_id}`.\n")
}

fn release_note_content(issue_id: &str) -> String {
    format!("# Release Note\n\nDelivery prepared for `{issue_id}`. No production release was performed.\n")
}
