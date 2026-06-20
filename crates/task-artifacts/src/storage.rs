use crate::model::{
    TaskChangedFile, TaskChangedFilesRecord, TaskCommandInput, TaskCommandRecord, TaskEvidence,
    TaskRun, TaskRunCheckpoint, TaskRunStatus, TaskValidationRecord, WorkLoopArtifactClass,
    WorkLoopArtifactContract, WorkLoopFilesystemContract, WorkLoopRoleAlias, WorkLoopStage,
    WorkLoopStageContract, TASK_CHANGED_FILES_VERSION, TASK_COMMAND_VERSION, TASK_EVIDENCE_VERSION,
    TASK_RUN_CHECKPOINT_VERSION, TASK_RUN_VERSION, TASK_VALIDATION_VERSION,
    WORK_LOOP_FILESYSTEM_CONTRACT_VERSION,
};
use agentflow_event_store::TaskReplayCursor;
use agentflow_workflow_core::{
    canonicalize_project_root, join_relative_path, normalize_relative_path_string,
    validate_safe_local_id, IssueId, RunId, WorkflowFlowType,
};
use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

pub fn prepare_task_artifact_workspace(
    project_root: impl AsRef<Path>,
    issue_id: &str,
) -> Result<()> {
    let root = canonicalize_project_root(project_root)?;
    ensure_directory(&task_issue_runs_dir(&root, issue_id)?)?;
    ensure_directory(&task_evidence_dir_under_root(&root, issue_id)?)?;
    Ok(())
}

pub fn task_run_dir(
    project_root: impl AsRef<Path>,
    issue_id: &str,
    run_id: &str,
) -> Result<PathBuf> {
    let root = canonicalize_project_root(project_root)?;
    task_run_dir_under_root(&root, issue_id, run_id)
}

pub fn task_evidence_dir(project_root: impl AsRef<Path>, issue_id: &str) -> Result<PathBuf> {
    let root = canonicalize_project_root(project_root)?;
    task_evidence_dir_under_root(&root, issue_id)
}

pub fn task_changed_files_path(
    project_root: impl AsRef<Path>,
    issue_id: &str,
    run_id: &str,
) -> Result<PathBuf> {
    Ok(task_run_dir(project_root, issue_id, run_id)?.join("changed-files.json"))
}

pub fn task_work_loop_contract_path(
    project_root: impl AsRef<Path>,
    issue_id: &str,
) -> Result<PathBuf> {
    let root = canonicalize_project_root(project_root)?;
    Ok(task_issue_dir(&root, issue_id)?.join("work-loop-contract.json"))
}

pub fn task_launch_request_path(issue_id: &str, run_id: &str) -> Result<String> {
    normalize_relative_path_string(
        PathBuf::from(".agentflow")
            .join("tasks")
            .join(IssueId::parse(issue_id)?.as_str())
            .join("runs")
            .join(RunId::parse(run_id)?.as_str())
            .join("launch")
            .join("agent-request.json"),
    )
}

pub fn task_work_action_proposals_path(
    project_root: impl AsRef<Path>,
    issue_id: &str,
    run_id: &str,
) -> Result<PathBuf> {
    Ok(task_run_dir(project_root, issue_id, run_id)?
        .join("launch")
        .join("work-action-proposals.json"))
}

pub fn write_work_loop_filesystem_contract(
    project_root: impl AsRef<Path>,
    issue_id: &str,
    workflow_ref: &str,
) -> Result<WorkLoopFilesystemContract> {
    let root = canonicalize_project_root(project_root)?;
    let issue_id = IssueId::parse(issue_id)?;
    validate_required("workflowRef", workflow_ref)?;
    let contract_path = task_work_loop_contract_path(&root, issue_id.as_str())?;
    let contract = build_work_loop_filesystem_contract(
        issue_id.as_str(),
        workflow_ref,
        normalize_relative_to_project(&root, &contract_path)?,
    )?;
    write_json(&contract_path, &contract)?;
    Ok(contract)
}

pub fn load_work_loop_filesystem_contract(
    project_root: impl AsRef<Path>,
    issue_id: &str,
) -> Result<WorkLoopFilesystemContract> {
    let root = canonicalize_project_root(project_root)?;
    read_json(task_work_loop_contract_path(&root, issue_id)?)
}

pub fn create_task_run(
    project_root: impl AsRef<Path>,
    issue_id: &str,
    run_id: &str,
    workflow_ref: &str,
    branch_name: Option<String>,
) -> Result<TaskRun> {
    let root = canonicalize_project_root(project_root)?;
    let issue_id = IssueId::parse(issue_id)?;
    let run_id = RunId::parse(run_id)?;
    validate_required("workflowRef", workflow_ref)?;
    prepare_task_artifact_workspace(&root, issue_id.as_str())?;
    let _ = write_work_loop_filesystem_contract(&root, issue_id.as_str(), workflow_ref)?;
    let now = unix_timestamp_seconds();
    let run = TaskRun {
        version: TASK_RUN_VERSION.to_string(),
        issue_id: issue_id.as_str().to_string(),
        run_id: run_id.as_str().to_string(),
        workflow_ref: workflow_ref.to_string(),
        status: TaskRunStatus::Queued,
        base_commit: git_head_commit(&root).ok(),
        branch_name,
        created_at: now,
        updated_at: now,
    };
    let run_directory = task_run_dir_under_root(&root, issue_id.as_str(), run_id.as_str())?;
    ensure_directory(&run_directory)?;
    write_json(&run_directory.join("run.json"), &run)?;
    Ok(run)
}

pub fn load_task_run(
    project_root: impl AsRef<Path>,
    issue_id: &str,
    run_id: &str,
) -> Result<TaskRun> {
    let root = canonicalize_project_root(project_root)?;
    read_json(&task_run_dir_under_root(&root, issue_id, run_id)?.join("run.json"))
}

pub fn update_task_run_status(
    project_root: impl AsRef<Path>,
    issue_id: &str,
    run_id: &str,
    status: TaskRunStatus,
) -> Result<TaskRun> {
    let root = canonicalize_project_root(project_root)?;
    let path = task_run_dir_under_root(&root, issue_id, run_id)?.join("run.json");
    let mut run: TaskRun = read_json(&path)?;
    run.status = status;
    run.updated_at = unix_timestamp_seconds();
    write_json(&path, &run)?;
    Ok(run)
}

pub fn write_task_command_record(
    project_root: impl AsRef<Path>,
    issue_id: &str,
    run_id: &str,
    input: TaskCommandInput,
) -> Result<TaskCommandRecord> {
    let root = canonicalize_project_root(project_root)?;
    let run = load_task_run(&root, issue_id, run_id)?;
    validate_required("label", &input.label)?;
    validate_required("program", &input.program)?;
    let command_dir = task_run_dir_under_root(&root, issue_id, run_id)?.join("commands");
    ensure_directory(&command_dir)?;
    let command_id = next_named_id(&command_dir, "cmd-")?;
    let stdout_path = command_dir.join(format!("{command_id}.stdout.txt"));
    let stderr_path = command_dir.join(format!("{command_id}.stderr.txt"));
    fs::write(&stdout_path, input.stdout)
        .with_context(|| format!("write {}", stdout_path.display()))?;
    fs::write(&stderr_path, input.stderr)
        .with_context(|| format!("write {}", stderr_path.display()))?;
    let record = TaskCommandRecord {
        version: TASK_COMMAND_VERSION.to_string(),
        issue_id: run.issue_id,
        run_id: run.run_id,
        command_id: command_id.clone(),
        label: input.label,
        program: input.program,
        args: input.args,
        exit_code: input.exit_code,
        stdout_path: relative_command_path(issue_id, run_id, &command_id, "stdout.txt")?,
        stderr_path: relative_command_path(issue_id, run_id, &command_id, "stderr.txt")?,
        recorded_at: unix_timestamp_seconds(),
    };
    write_json(&command_dir.join(format!("{command_id}.json")), &record)?;
    Ok(record)
}

pub fn write_task_changed_files(
    project_root: impl AsRef<Path>,
    issue_id: &str,
    run_id: &str,
    files: Vec<TaskChangedFile>,
    base_commit: Option<String>,
    head_commit: Option<String>,
    tree_sha: Option<String>,
    working_tree_hash: impl Into<String>,
    patch_sha256: impl Into<String>,
    file_content_sha256: impl Into<String>,
) -> Result<TaskChangedFilesRecord> {
    let root = canonicalize_project_root(project_root)?;
    let issue_id = IssueId::parse(issue_id)?;
    let run_id = RunId::parse(run_id)?;
    let changed_file_hash = sha256_hex(&serde_json::to_vec(&serde_json::json!({
        "files": &files,
        "baseCommit": &base_commit,
        "headCommit": &head_commit,
    }))?);
    let record = TaskChangedFilesRecord {
        version: TASK_CHANGED_FILES_VERSION.to_string(),
        issue_id: issue_id.as_str().to_string(),
        run_id: run_id.as_str().to_string(),
        files,
        base_commit,
        head_commit,
        tree_sha,
        working_tree_hash: working_tree_hash.into(),
        patch_sha256: patch_sha256.into(),
        file_content_sha256: file_content_sha256.into(),
        changed_file_hash,
        collected_at: unix_timestamp_seconds(),
    };
    write_json(
        &task_changed_files_path(&root, issue_id.as_str(), run_id.as_str())?,
        &record,
    )?;
    Ok(record)
}

pub fn write_task_validation(
    project_root: impl AsRef<Path>,
    issue_id: &str,
    run_id: &str,
) -> Result<TaskValidationRecord> {
    write_task_validation_with_assessment(project_root, issue_id, run_id, Vec::new())
}

pub fn write_task_validation_with_assessment(
    project_root: impl AsRef<Path>,
    issue_id: &str,
    run_id: &str,
    boundary_failures: Vec<String>,
) -> Result<TaskValidationRecord> {
    let root = canonicalize_project_root(project_root)?;
    let command_records = load_command_records(&root, issue_id, run_id)?;
    if command_records.is_empty() {
        anyhow::bail!("task validation requires at least one command record");
    }
    let failed_command_ids = command_records
        .iter()
        .filter(|record| record.exit_code != Some(0))
        .map(|record| record.command_id.clone())
        .collect::<Vec<_>>();
    let changed_files = load_task_changed_files(&root, issue_id, run_id).ok();
    let validation_command_hash = sha256_hex(&serde_json::to_vec(
        &command_records
            .iter()
            .map(|record| {
                serde_json::json!({
                    "label": record.label,
                    "program": record.program,
                    "args": record.args,
                })
            })
            .collect::<Vec<_>>(),
    )?);
    let validation_output_hash = sha256_hex(&serde_json::to_vec(
        &command_records
            .iter()
            .map(|record| {
                let stdout = fs::read_to_string(
                    task_run_dir_under_root(&root, issue_id, run_id)?
                        .join("commands")
                        .join(format!("{}.stdout.txt", record.command_id)),
                )?;
                let stderr = fs::read_to_string(
                    task_run_dir_under_root(&root, issue_id, run_id)?
                        .join("commands")
                        .join(format!("{}.stderr.txt", record.command_id)),
                )?;
                Ok::<_, anyhow::Error>(serde_json::json!({
                    "commandId": record.command_id,
                    "exitCode": record.exit_code,
                    "stdout": stdout,
                    "stderr": stderr,
                }))
            })
            .collect::<Result<Vec<_>>>()?,
    )?);
    let passed = failed_command_ids.is_empty() && boundary_failures.is_empty();
    let changed_files_path = changed_files
        .as_ref()
        .map(|_| {
            normalize_relative_path_string(
                PathBuf::from(".agentflow")
                    .join("tasks")
                    .join(IssueId::parse(issue_id)?.as_str())
                    .join("runs")
                    .join(RunId::parse(run_id)?.as_str())
                    .join("changed-files.json"),
            )
        })
        .transpose()?;
    let changed_file_hash = changed_files
        .as_ref()
        .map(|record| record.changed_file_hash.clone());
    let patch_sha256 = changed_files
        .as_ref()
        .map(|record| record.patch_sha256.clone());
    let file_content_sha256 = changed_files
        .as_ref()
        .map(|record| record.file_content_sha256.clone());
    let base_commit = changed_files
        .as_ref()
        .and_then(|record| record.base_commit.clone());
    let head_commit = changed_files
        .as_ref()
        .and_then(|record| record.head_commit.clone());
    let tree_sha = changed_files
        .as_ref()
        .and_then(|record| record.tree_sha.clone());
    let working_tree_hash = changed_files
        .as_ref()
        .map(|record| record.working_tree_hash.clone());
    let validation_result_hash = sha256_hex(&serde_json::to_vec(&serde_json::json!({
        "passed": passed,
        "validationCommandHash": &validation_command_hash,
        "validationOutputHash": &validation_output_hash,
        "commandIds": command_records
            .iter()
            .map(|record| record.command_id.as_str())
            .collect::<Vec<_>>(),
        "failedCommandIds": &failed_command_ids,
        "boundaryFailures": &boundary_failures,
        "changedFileHash": &changed_file_hash,
        "patchSha256": &patch_sha256,
        "fileContentSha256": &file_content_sha256,
        "baseCommit": &base_commit,
        "headCommit": &head_commit,
        "treeSha": &tree_sha,
        "workingTreeHash": &working_tree_hash,
    }))?);
    let validation = TaskValidationRecord {
        version: TASK_VALIDATION_VERSION.to_string(),
        issue_id: issue_id.to_string(),
        run_id: run_id.to_string(),
        passed,
        command_ids: command_records
            .iter()
            .map(|record| record.command_id.clone())
            .collect(),
        failed_command_ids,
        boundary_failures,
        changed_files_path,
        validation_command_hash: Some(validation_command_hash.clone()),
        validation_output_hash: Some(validation_output_hash.clone()),
        patch_sha256,
        file_content_sha256,
        tree_sha,
        command_hash: Some(validation_command_hash),
        changed_file_hash,
        validation_result_hash: Some(validation_result_hash),
        base_commit,
        head_commit,
        working_tree_hash,
        checked_at: unix_timestamp_seconds(),
    };
    write_json(
        &task_run_dir_under_root(&root, issue_id, run_id)?.join("validation.json"),
        &validation,
    )?;
    Ok(validation)
}

pub fn write_task_evidence(
    project_root: impl AsRef<Path>,
    issue_id: &str,
    run_id: &str,
    summary: impl Into<String>,
) -> Result<TaskEvidence> {
    let root = canonicalize_project_root(project_root)?;
    let validation_path = task_run_dir_under_root(&root, issue_id, run_id)?.join("validation.json");
    let validation: TaskValidationRecord = read_json(&validation_path)?;
    let command_records = load_command_records(&root, issue_id, run_id)?;
    let generated_at = unix_timestamp_seconds();
    let evidence = TaskEvidence {
        version: TASK_EVIDENCE_VERSION.to_string(),
        issue_id: issue_id.to_string(),
        run_id: run_id.to_string(),
        status: if validation.passed {
            "passed"
        } else {
            "failed"
        }
        .to_string(),
        summary: summary.into(),
        run_path: normalize_relative_path_string(
            PathBuf::from(".agentflow")
                .join("tasks")
                .join(IssueId::parse(issue_id)?.as_str())
                .join("runs")
                .join(RunId::parse(run_id)?.as_str())
                .join("run.json"),
        )?,
        command_paths: command_records
            .iter()
            .map(|record| {
                normalize_relative_path_string(
                    PathBuf::from(".agentflow")
                        .join("tasks")
                        .join(IssueId::parse(issue_id)?.as_str())
                        .join("runs")
                        .join(RunId::parse(run_id)?.as_str())
                        .join("commands")
                        .join(format!("{}.json", record.command_id)),
                )
            })
            .collect::<Result<Vec<_>>>()?,
        validation_path: normalize_relative_path_string(
            PathBuf::from(".agentflow")
                .join("tasks")
                .join(IssueId::parse(issue_id)?.as_str())
                .join("runs")
                .join(RunId::parse(run_id)?.as_str())
                .join("validation.json"),
        )?,
        changed_files_path: validation.changed_files_path.clone(),
        validation_command_hash: validation.validation_command_hash.clone(),
        validation_output_hash: validation.validation_output_hash.clone(),
        patch_sha256: validation.patch_sha256.clone(),
        file_content_sha256: validation.file_content_sha256.clone(),
        tree_sha: validation.tree_sha.clone(),
        command_hash: validation.command_hash.clone(),
        changed_file_hash: validation.changed_file_hash.clone(),
        validation_result_hash: validation.validation_result_hash.clone(),
        base_commit: validation.base_commit.clone(),
        head_commit: validation.head_commit.clone(),
        working_tree_hash: validation.working_tree_hash.clone(),
        generated_at: Some(generated_at),
        created_at: generated_at,
    };
    write_json(
        &task_evidence_dir_under_root(&root, issue_id)?.join("evidence.json"),
        &evidence,
    )?;
    Ok(evidence)
}

pub fn write_task_run_checkpoint(
    project_root: impl AsRef<Path>,
    issue_id: &str,
    run_id: &str,
    flow_type: WorkflowFlowType,
    state: &str,
    event_id: &str,
    correlation_id: Option<String>,
    summary: impl Into<String>,
) -> Result<TaskRunCheckpoint> {
    let root = canonicalize_project_root(project_root)?;
    validate_required("state", state)?;
    validate_required("eventId", event_id)?;
    let checkpoint_dir = task_run_dir_under_root(&root, issue_id, run_id)?.join("checkpoints");
    ensure_directory(&checkpoint_dir)?;
    let checkpoint_id = next_named_id(&checkpoint_dir, "checkpoint-")?;
    let checkpoint = TaskRunCheckpoint {
        version: TASK_RUN_CHECKPOINT_VERSION.to_string(),
        issue_id: issue_id.to_string(),
        run_id: run_id.to_string(),
        checkpoint_id: checkpoint_id.clone(),
        flow_type,
        state: state.to_string(),
        event_id: event_id.to_string(),
        correlation_id,
        summary: summary.into(),
        created_at: unix_timestamp_seconds(),
    };
    write_json(
        &checkpoint_dir.join(format!("{checkpoint_id}.json")),
        &checkpoint,
    )?;
    Ok(checkpoint)
}

pub fn load_task_run_checkpoints(
    project_root: impl AsRef<Path>,
    issue_id: &str,
    run_id: &str,
) -> Result<Vec<TaskRunCheckpoint>> {
    let root = canonicalize_project_root(project_root)?;
    let checkpoint_dir = task_run_dir_under_root(&root, issue_id, run_id)?.join("checkpoints");
    if !checkpoint_dir.exists() {
        return Ok(Vec::new());
    }
    let mut paths = fs::read_dir(&checkpoint_dir)
        .with_context(|| format!("read {}", checkpoint_dir.display()))?
        .collect::<Result<Vec<_>, _>>()
        .with_context(|| format!("collect {}", checkpoint_dir.display()))?
        .into_iter()
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("json"))
        .collect::<Vec<_>>();
    paths.sort();
    paths.into_iter().map(read_json).collect()
}

pub fn latest_task_run_checkpoint(
    project_root: impl AsRef<Path>,
    issue_id: &str,
    run_id: &str,
) -> Result<Option<TaskRunCheckpoint>> {
    Ok(load_task_run_checkpoints(project_root, issue_id, run_id)?
        .into_iter()
        .last())
}

pub fn checkpoint_replay_cursor(checkpoint: &TaskRunCheckpoint) -> TaskReplayCursor {
    TaskReplayCursor {
        flow_type: checkpoint.flow_type,
        aggregate_type: "issue".to_string(),
        aggregate_id: checkpoint.issue_id.clone(),
        project_id: None,
        issue_id: Some(checkpoint.issue_id.clone()),
        run_id: Some(checkpoint.run_id.clone()),
        after_event_id: checkpoint.event_id.clone(),
    }
}

pub fn load_task_evidence(project_root: impl AsRef<Path>, issue_id: &str) -> Result<TaskEvidence> {
    let root = canonicalize_project_root(project_root)?;
    read_json(&task_evidence_dir_under_root(&root, issue_id)?.join("evidence.json"))
}

pub fn load_task_changed_files(
    project_root: impl AsRef<Path>,
    issue_id: &str,
    run_id: &str,
) -> Result<TaskChangedFilesRecord> {
    let root = canonicalize_project_root(project_root)?;
    read_json(&task_changed_files_path(&root, issue_id, run_id)?)
}

fn load_command_records(
    root: &Path,
    issue_id: &str,
    run_id: &str,
) -> Result<Vec<TaskCommandRecord>> {
    let command_dir = task_run_dir_under_root(root, issue_id, run_id)?.join("commands");
    if !command_dir.exists() {
        return Ok(Vec::new());
    }
    let mut paths = fs::read_dir(&command_dir)
        .with_context(|| format!("read {}", command_dir.display()))?
        .collect::<Result<Vec<_>, _>>()
        .with_context(|| format!("collect {}", command_dir.display()))?
        .into_iter()
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("json"))
        .collect::<Vec<_>>();
    paths.sort();
    paths.into_iter().map(read_json).collect()
}

fn build_work_loop_filesystem_contract(
    issue_id: &str,
    workflow_ref: &str,
    contract_path: String,
) -> Result<WorkLoopFilesystemContract> {
    let run_id = "<run-id>";
    let command_id = "<command-id>";
    let proposal_id = "<proposal-id>";
    let accepted_action_id = "<accepted-action-id>";
    let checkpoint_id = "<checkpoint-id>";
    let artifacts = vec![
        WorkLoopArtifactContract {
            key: "spec_issue_authority".to_string(),
            stage: WorkLoopStage::Command,
            class: WorkLoopArtifactClass::Authority,
            location_ref: normalize_relative_path_string(
                PathBuf::from(".agentflow")
                    .join("spec")
                    .join("issues")
                    .join(format!("{issue_id}.json")),
            )?,
            description: "Spec issue 是 Work Loop 的唯一任务权威。".to_string(),
            traces_to: vec!["issue".to_string()],
        },
        WorkLoopArtifactContract {
            key: "work_command".to_string(),
            stage: WorkLoopStage::Command,
            class: WorkLoopArtifactClass::Authority,
            location_ref: normalize_relative_path_string(
                PathBuf::from(".agentflow")
                    .join("runtime")
                    .join("commands")
                    .join(format!("{command_id}.json")),
            )?,
            description: "Work Command 是由 spec issue 派生出的 runtime 执行入口。".to_string(),
            traces_to: vec!["issue".to_string(), "command".to_string()],
        },
        WorkLoopArtifactContract {
            key: "action_proposal".to_string(),
            stage: WorkLoopStage::Proposal,
            class: WorkLoopArtifactClass::Authority,
            location_ref: normalize_relative_path_string(
                PathBuf::from(".agentflow")
                    .join("runtime")
                    .join("proposals")
                    .join(format!("{proposal_id}.json")),
            )?,
            description: "关键执行动作必须先写成 Action Proposal。".to_string(),
            traces_to: vec!["issue".to_string(), "proposal".to_string()],
        },
        WorkLoopArtifactContract {
            key: "proposal_decision".to_string(),
            stage: WorkLoopStage::Proposal,
            class: WorkLoopArtifactClass::DerivedArtifact,
            location_ref: normalize_relative_path_string(
                PathBuf::from(".agentflow")
                    .join("runtime")
                    .join("decisions")
                    .join(format!("{proposal_id}.json")),
            )?,
            description: "Proposal decision 记录 proposal 是否被 runtime 接受。".to_string(),
            traces_to: vec!["issue".to_string(), "proposal".to_string()],
        },
        WorkLoopArtifactContract {
            key: "accepted_action".to_string(),
            stage: WorkLoopStage::Proposal,
            class: WorkLoopArtifactClass::DerivedArtifact,
            location_ref: normalize_relative_path_string(
                PathBuf::from(".agentflow")
                    .join("runtime")
                    .join("actions")
                    .join(format!("{accepted_action_id}.json")),
            )?,
            description: "接受后的 action 作为真正进入执行面的 runtime 事实。".to_string(),
            traces_to: vec![
                "issue".to_string(),
                "proposal".to_string(),
                "accepted_action".to_string(),
            ],
        },
        WorkLoopArtifactContract {
            key: "preflight_report".to_string(),
            stage: WorkLoopStage::Preflight,
            class: WorkLoopArtifactClass::DerivedArtifact,
            location_ref: normalize_relative_path_string(
                PathBuf::from(".agentflow")
                    .join("tasks")
                    .join(issue_id)
                    .join("runs")
                    .join(run_id)
                    .join("preflight")
                    .join("preflight.json"),
            )?,
            description: "Preflight report 记录依赖、上下文、工作区与合同检查结果。".to_string(),
            traces_to: vec!["issue".to_string(), "run".to_string()],
        },
        WorkLoopArtifactContract {
            key: "handoff_request".to_string(),
            stage: WorkLoopStage::Handoff,
            class: WorkLoopArtifactClass::TransportSnapshot,
            location_ref: normalize_relative_path_string(
                PathBuf::from(".agentflow")
                    .join("tasks")
                    .join(issue_id)
                    .join("runs")
                    .join(run_id)
                    .join("launch")
                    .join("agent-request.json"),
            )?,
            description: "Handoff 只是 transport snapshot，不会替代 spec issue authority。"
                .to_string(),
            traces_to: vec!["issue".to_string(), "run".to_string()],
        },
        WorkLoopArtifactContract {
            key: "run_record".to_string(),
            stage: WorkLoopStage::Session,
            class: WorkLoopArtifactClass::DerivedArtifact,
            location_ref: normalize_relative_path_string(
                PathBuf::from(".agentflow")
                    .join("tasks")
                    .join(issue_id)
                    .join("runs")
                    .join(run_id)
                    .join("run.json"),
            )?,
            description: "run.json 记录本次 Work Session 的主体状态。".to_string(),
            traces_to: vec!["issue".to_string(), "run".to_string()],
        },
        WorkLoopArtifactContract {
            key: "command_records".to_string(),
            stage: WorkLoopStage::Session,
            class: WorkLoopArtifactClass::DerivedArtifact,
            location_ref: normalize_relative_path_string(
                PathBuf::from(".agentflow")
                    .join("tasks")
                    .join(issue_id)
                    .join("runs")
                    .join(run_id)
                    .join("commands")
                    .join(format!("{command_id}.json")),
            )?,
            description: "命令记录、stdout、stderr 都落在 runs/<run-id>/commands/**。".to_string(),
            traces_to: vec![
                "issue".to_string(),
                "run".to_string(),
                "command".to_string(),
            ],
        },
        WorkLoopArtifactContract {
            key: "run_checkpoint".to_string(),
            stage: WorkLoopStage::Session,
            class: WorkLoopArtifactClass::DerivedArtifact,
            location_ref: normalize_relative_path_string(
                PathBuf::from(".agentflow")
                    .join("tasks")
                    .join(issue_id)
                    .join("runs")
                    .join(run_id)
                    .join("checkpoints")
                    .join(format!("{checkpoint_id}.json")),
            )?,
            description: "checkpoint 用于恢复、重放和 durable session。".to_string(),
            traces_to: vec!["issue".to_string(), "run".to_string()],
        },
        WorkLoopArtifactContract {
            key: "validation_record".to_string(),
            stage: WorkLoopStage::Evidence,
            class: WorkLoopArtifactClass::DerivedArtifact,
            location_ref: normalize_relative_path_string(
                PathBuf::from(".agentflow")
                    .join("tasks")
                    .join(issue_id)
                    .join("runs")
                    .join(run_id)
                    .join("validation.json"),
            )?,
            description: "validation.json 记录本地验证命令与结果。".to_string(),
            traces_to: vec![
                "issue".to_string(),
                "run".to_string(),
                "command".to_string(),
            ],
        },
        WorkLoopArtifactContract {
            key: "changed_files".to_string(),
            stage: WorkLoopStage::Evidence,
            class: WorkLoopArtifactClass::DerivedArtifact,
            location_ref: normalize_relative_path_string(
                PathBuf::from(".agentflow")
                    .join("tasks")
                    .join(issue_id)
                    .join("runs")
                    .join(run_id)
                    .join("changed-files.json"),
            )?,
            description: "变更文件摘要用于验证和写回追溯。".to_string(),
            traces_to: vec!["issue".to_string(), "run".to_string()],
        },
        WorkLoopArtifactContract {
            key: "task_evidence".to_string(),
            stage: WorkLoopStage::Evidence,
            class: WorkLoopArtifactClass::Authority,
            location_ref: normalize_relative_path_string(
                PathBuf::from(".agentflow")
                    .join("tasks")
                    .join(issue_id)
                    .join("evidence")
                    .join("evidence.json"),
            )?,
            description: "本地 evidence 是任务验证事实的稳定出口。".to_string(),
            traces_to: vec![
                "issue".to_string(),
                "run".to_string(),
                "proposal".to_string(),
                "command".to_string(),
            ],
        },
        WorkLoopArtifactContract {
            key: "task_event_stream".to_string(),
            stage: WorkLoopStage::Session,
            class: WorkLoopArtifactClass::Authority,
            location_ref: ".agentflow/events/task-events.jsonl".to_string(),
            description: "Work Loop 事件统一进入 task-events.jsonl。".to_string(),
            traces_to: vec!["issue".to_string(), "run".to_string()],
        },
        WorkLoopArtifactContract {
            key: "task_projection".to_string(),
            stage: WorkLoopStage::Delivery,
            class: WorkLoopArtifactClass::ReadModel,
            location_ref: normalize_relative_path_string(
                PathBuf::from(".agentflow")
                    .join("projections")
                    .join("tasks")
                    .join(format!("{issue_id}.json")),
            )?,
            description: "任务投影是 Desktop 和查询面的只读视图。".to_string(),
            traces_to: vec!["issue".to_string()],
        },
        WorkLoopArtifactContract {
            key: "public_pr_record".to_string(),
            stage: WorkLoopStage::Delivery,
            class: WorkLoopArtifactClass::PublicRecord,
            location_ref: "public://pr-or-mr-body".to_string(),
            description: "PR/MR body 是公开交付记录，不写回 .agentflow/tasks/**。".to_string(),
            traces_to: vec!["issue".to_string(), "run".to_string()],
        },
        WorkLoopArtifactContract {
            key: "public_changelog_record".to_string(),
            stage: WorkLoopStage::Delivery,
            class: WorkLoopArtifactClass::PublicRecord,
            location_ref: "CHANGELOG.md".to_string(),
            description: "CHANGELOG 是版本级公开交付事实。".to_string(),
            traces_to: vec!["issue".to_string()],
        },
        WorkLoopArtifactContract {
            key: "public_release_notes".to_string(),
            stage: WorkLoopStage::Delivery,
            class: WorkLoopArtifactClass::PublicRecord,
            location_ref: "public://release-notes".to_string(),
            description: "Release notes 是版本发布后的外部交付记录。".to_string(),
            traces_to: vec!["issue".to_string()],
        },
    ];
    let stages = vec![
        WorkLoopStageContract {
            stage: WorkLoopStage::Command,
            issue_statuses: vec!["todo".to_string()],
            inputs: vec!["spec_issue_authority".to_string()],
            outputs: vec!["work_command".to_string()],
            evidence: Vec::new(),
        },
        WorkLoopStageContract {
            stage: WorkLoopStage::Proposal,
            issue_statuses: vec!["todo".to_string(), "in_progress".to_string()],
            inputs: vec!["work_command".to_string()],
            outputs: vec![
                "action_proposal".to_string(),
                "proposal_decision".to_string(),
                "accepted_action".to_string(),
            ],
            evidence: vec!["proposal_decision".to_string()],
        },
        WorkLoopStageContract {
            stage: WorkLoopStage::Preflight,
            issue_statuses: vec!["todo".to_string()],
            inputs: vec![
                "spec_issue_authority".to_string(),
                "accepted_action".to_string(),
            ],
            outputs: vec!["preflight_report".to_string()],
            evidence: vec!["preflight_report".to_string()],
        },
        WorkLoopStageContract {
            stage: WorkLoopStage::Session,
            issue_statuses: vec!["in_progress".to_string()],
            inputs: vec![
                "accepted_action".to_string(),
                "preflight_report".to_string(),
                "handoff_request".to_string(),
            ],
            outputs: vec![
                "run_record".to_string(),
                "command_records".to_string(),
                "run_checkpoint".to_string(),
                "task_event_stream".to_string(),
            ],
            evidence: vec![
                "run_checkpoint".to_string(),
                "task_event_stream".to_string(),
            ],
        },
        WorkLoopStageContract {
            stage: WorkLoopStage::Evidence,
            issue_statuses: vec!["in_review".to_string(), "done".to_string()],
            inputs: vec![
                "run_record".to_string(),
                "command_records".to_string(),
                "changed_files".to_string(),
            ],
            outputs: vec!["validation_record".to_string(), "task_evidence".to_string()],
            evidence: vec!["validation_record".to_string(), "task_evidence".to_string()],
        },
        WorkLoopStageContract {
            stage: WorkLoopStage::Handoff,
            issue_statuses: vec!["todo".to_string(), "in_progress".to_string()],
            inputs: vec![
                "spec_issue_authority".to_string(),
                "work_command".to_string(),
            ],
            outputs: vec!["handoff_request".to_string()],
            evidence: vec!["handoff_request".to_string()],
        },
        WorkLoopStageContract {
            stage: WorkLoopStage::Delivery,
            issue_statuses: vec!["in_review".to_string(), "done".to_string()],
            inputs: vec!["task_evidence".to_string(), "task_projection".to_string()],
            outputs: vec![
                "public_pr_record".to_string(),
                "public_changelog_record".to_string(),
                "public_release_notes".to_string(),
            ],
            evidence: vec!["task_projection".to_string()],
        },
    ];

    Ok(WorkLoopFilesystemContract {
        version: WORK_LOOP_FILESYSTEM_CONTRACT_VERSION.to_string(),
        issue_id: issue_id.to_string(),
        workflow_ref: workflow_ref.to_string(),
        contract_path,
        role_aliases: vec![WorkLoopRoleAlias {
            canonical_role: "work-agent".to_string(),
            accepted_aliases: vec!["build-agent".to_string()],
            description: "Build Agent 是 Work Agent 在当前 Runtime 和任务包中的兼容别名。"
                .to_string(),
        }],
        stages,
        artifacts,
        generated_at: unix_timestamp_seconds(),
    })
}

fn task_issue_dir(root: &Path, issue_id: &str) -> Result<PathBuf> {
    let issue_id = IssueId::parse(issue_id)?;
    join_relative_path(
        root,
        PathBuf::from(".agentflow")
            .join("tasks")
            .join(issue_id.as_str()),
    )
}

fn task_issue_runs_dir(root: &Path, issue_id: &str) -> Result<PathBuf> {
    Ok(task_issue_dir(root, issue_id)?.join("runs"))
}

fn task_run_dir_under_root(root: &Path, issue_id: &str, run_id: &str) -> Result<PathBuf> {
    let run_id = RunId::parse(run_id)?;
    Ok(task_issue_runs_dir(root, issue_id)?.join(run_id.as_str()))
}

fn task_evidence_dir_under_root(root: &Path, issue_id: &str) -> Result<PathBuf> {
    Ok(task_issue_dir(root, issue_id)?.join("evidence"))
}

fn relative_command_path(
    issue_id: &str,
    run_id: &str,
    command_id: &str,
    suffix: &str,
) -> Result<String> {
    validate_safe_local_id("commandId", command_id)?;
    normalize_relative_path_string(
        PathBuf::from(".agentflow")
            .join("tasks")
            .join(IssueId::parse(issue_id)?.as_str())
            .join("runs")
            .join(RunId::parse(run_id)?.as_str())
            .join("commands")
            .join(format!("{command_id}.{suffix}")),
    )
}

fn normalize_relative_to_project(root: &Path, path: &Path) -> Result<String> {
    let relative = path
        .strip_prefix(root)
        .with_context(|| format!("{} is outside {}", path.display(), root.display()))?;
    normalize_relative_path_string(relative)
}

fn next_named_id(directory: &Path, prefix: &str) -> Result<String> {
    ensure_directory(directory)?;
    let count = fs::read_dir(directory)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .path()
                .file_name()
                .and_then(|value| value.to_str())
                .is_some_and(|name| name.starts_with(prefix) && name.ends_with(".json"))
        })
        .count();
    Ok(format!("{prefix}{:03}", count + 1))
}

fn validate_required(field: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        anyhow::bail!("{field} is required");
    }
    Ok(())
}

fn ensure_directory(path: &Path) -> Result<()> {
    fs::create_dir_all(path).with_context(|| format!("create {}", path.display()))
}

fn read_json<T: serde::de::DeserializeOwned>(path: impl AsRef<Path>) -> Result<T> {
    let path = path.as_ref();
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

fn write_json<T: serde::Serialize>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        ensure_directory(parent)?;
    }
    fs::write(path, serde_json::to_string_pretty(value)? + "\n")
        .with_context(|| format!("write {}", path.display()))
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn unix_timestamp_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

fn git_head_commit(root: &Path) -> Result<String> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(root)
        .output()
        .with_context(|| format!("run git rev-parse HEAD in {}", root.display()))?;
    if !output.status.success() {
        anyhow::bail!(
            "git rev-parse HEAD failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .trim_end()
        .to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use agentflow_workflow_core::WorkflowFlowType;
    use tempfile::tempdir;

    fn command(exit_code: i32) -> TaskCommandInput {
        TaskCommandInput {
            label: "build".to_string(),
            program: "npm".to_string(),
            args: vec!["run".to_string(), "build".to_string()],
            exit_code: Some(exit_code),
            stdout: "ok".to_string(),
            stderr: String::new(),
        }
    }

    #[test]
    fn creates_task_run_under_issue_scoped_directory() {
        let dir = tempdir().unwrap();
        let run = create_task_run(
            dir.path(),
            "AF-TASK-001",
            "run-001",
            "work-agent.issue-loop@v1",
            Some("agentflow/AF-TASK-001".to_string()),
        )
        .unwrap();

        assert_eq!(run.status, TaskRunStatus::Queued);
        assert!(dir
            .path()
            .join(".agentflow/tasks/AF-TASK-001/runs/run-001/run.json")
            .is_file());
        assert!(dir
            .path()
            .join(".agentflow/tasks/AF-TASK-001/work-loop-contract.json")
            .is_file());
        assert!(dir
            .path()
            .join(".agentflow/tasks/AF-TASK-001/evidence")
            .is_dir());
    }

    #[test]
    fn writes_issue_scoped_work_loop_contract() {
        let dir = tempdir().unwrap();

        let contract = write_work_loop_filesystem_contract(
            dir.path(),
            "AF-TASK-001",
            "work-agent.issue-loop@v1",
        )
        .unwrap();

        assert_eq!(contract.version, WORK_LOOP_FILESYSTEM_CONTRACT_VERSION);
        assert_eq!(contract.issue_id, "AF-TASK-001");
        assert_eq!(
            contract.contract_path,
            ".agentflow/tasks/AF-TASK-001/work-loop-contract.json"
        );
        assert!(contract
            .role_aliases
            .iter()
            .any(|alias| alias.canonical_role == "work-agent"
                && alias.accepted_aliases == vec!["build-agent".to_string()]));
        assert!(contract.artifacts.iter().any(|artifact| {
            artifact.key == "task_event_stream"
                && artifact.location_ref == ".agentflow/events/task-events.jsonl"
        }));
        assert!(contract.artifacts.iter().any(|artifact| {
            artifact.key == "public_changelog_record" && artifact.location_ref == "CHANGELOG.md"
        }));
        assert_eq!(
            contract
                .stages
                .iter()
                .find(|stage| stage.stage == WorkLoopStage::Delivery)
                .unwrap()
                .outputs,
            vec![
                "public_pr_record".to_string(),
                "public_changelog_record".to_string(),
                "public_release_notes".to_string()
            ]
        );
        assert!(dir
            .path()
            .join(".agentflow/tasks/AF-TASK-001/work-loop-contract.json")
            .is_file());
    }

    #[test]
    fn writes_command_stdout_stderr_and_validation() {
        let dir = tempdir().unwrap();
        create_task_run(
            dir.path(),
            "AF-TASK-001",
            "run-001",
            "work-agent.issue-loop@v1",
            None,
        )
        .unwrap();
        let record =
            write_task_command_record(dir.path(), "AF-TASK-001", "run-001", command(0)).unwrap();
        let validation = write_task_validation(dir.path(), "AF-TASK-001", "run-001").unwrap();

        assert_eq!(record.command_id, "cmd-001");
        assert_eq!(validation.passed, true);
        assert!(dir
            .path()
            .join(".agentflow/tasks/AF-TASK-001/runs/run-001/commands/cmd-001.stdout.txt")
            .is_file());
    }

    #[test]
    fn updates_task_run_status() {
        let dir = tempdir().unwrap();
        create_task_run(
            dir.path(),
            "AF-TASK-001",
            "run-001",
            "work-agent.issue-loop@v1",
            None,
        )
        .unwrap();

        let updated = update_task_run_status(
            dir.path(),
            "AF-TASK-001",
            "run-001",
            TaskRunStatus::Completed,
        )
        .unwrap();

        assert_eq!(updated.status, TaskRunStatus::Completed);
        let loaded = load_task_run(dir.path(), "AF-TASK-001", "run-001").unwrap();
        assert_eq!(loaded.status, TaskRunStatus::Completed);
    }

    #[test]
    fn writes_issue_level_evidence_without_delivery_directory() {
        let dir = tempdir().unwrap();
        create_task_run(
            dir.path(),
            "AF-TASK-001",
            "run-001",
            "work-agent.issue-loop@v1",
            None,
        )
        .unwrap();
        write_task_command_record(dir.path(), "AF-TASK-001", "run-001", command(0)).unwrap();
        write_task_validation(dir.path(), "AF-TASK-001", "run-001").unwrap();

        let evidence =
            write_task_evidence(dir.path(), "AF-TASK-001", "run-001", "Build passed").unwrap();
        let loaded = load_task_evidence(dir.path(), "AF-TASK-001").unwrap();

        assert_eq!(evidence, loaded);
        assert_eq!(loaded.status, "passed");
        assert!(!dir
            .path()
            .join(".agentflow/tasks/AF-TASK-001/delivery")
            .exists());
        assert!(!dir.path().join(".agentflow/output/release").exists());
    }

    #[test]
    fn binds_trusted_validation_hashes_into_evidence() {
        let dir = tempdir().unwrap();
        create_task_run(
            dir.path(),
            "AF-TASK-001",
            "run-001",
            "work-agent.issue-loop@v1",
            None,
        )
        .unwrap();
        write_task_command_record(dir.path(), "AF-TASK-001", "run-001", command(0)).unwrap();
        write_task_changed_files(
            dir.path(),
            "AF-TASK-001",
            "run-001",
            vec![TaskChangedFile {
                path: "src/lib.rs".to_string(),
                change_type: "modified".to_string(),
                insertions: 1,
                deletions: 0,
                sources: Vec::new(),
            }],
            Some("base-001".to_string()),
            Some("head-001".to_string()),
            Some("tree-001".to_string()),
            "worktree-hash-001",
            "patch-hash-001",
            "file-content-hash-001",
        )
        .unwrap();

        let validation =
            write_task_validation_with_assessment(dir.path(), "AF-TASK-001", "run-001", Vec::new())
                .unwrap();
        let evidence =
            write_task_evidence(dir.path(), "AF-TASK-001", "run-001", "Trusted validation")
                .unwrap();

        assert!(validation.changed_files_path.is_some());
        assert!(validation.validation_command_hash.is_some());
        assert!(validation.validation_output_hash.is_some());
        assert!(validation.patch_sha256.is_some());
        assert!(validation.file_content_sha256.is_some());
        assert!(validation.tree_sha.is_some());
        assert!(validation.command_hash.is_some());
        assert!(validation.changed_file_hash.is_some());
        assert!(validation.validation_result_hash.is_some());
        assert_eq!(
            evidence.changed_files_path.as_deref(),
            validation.changed_files_path.as_deref()
        );
        assert_eq!(
            evidence.validation_command_hash.as_deref(),
            validation.validation_command_hash.as_deref()
        );
        assert_eq!(
            evidence.validation_output_hash.as_deref(),
            validation.validation_output_hash.as_deref()
        );
        assert_eq!(
            evidence.patch_sha256.as_deref(),
            validation.patch_sha256.as_deref()
        );
        assert_eq!(
            evidence.file_content_sha256.as_deref(),
            validation.file_content_sha256.as_deref()
        );
        assert_eq!(evidence.tree_sha.as_deref(), validation.tree_sha.as_deref());
        assert_eq!(
            evidence.command_hash.as_deref(),
            validation.command_hash.as_deref()
        );
        assert_eq!(
            evidence.changed_file_hash.as_deref(),
            validation.changed_file_hash.as_deref()
        );
        assert_eq!(
            evidence.validation_result_hash.as_deref(),
            validation.validation_result_hash.as_deref()
        );
        assert_eq!(evidence.base_commit.as_deref(), Some("base-001"));
        assert_eq!(evidence.head_commit.as_deref(), Some("head-001"));
        assert_eq!(evidence.tree_sha.as_deref(), Some("tree-001"));
        assert_eq!(
            evidence.working_tree_hash.as_deref(),
            Some("worktree-hash-001")
        );
        assert!(evidence.generated_at.is_some());
    }

    #[test]
    fn writes_checkpoints_and_builds_replay_cursor() {
        let dir = tempdir().unwrap();
        create_task_run(
            dir.path(),
            "AF-TASK-001",
            "run-001",
            "work-agent.issue-loop@v1",
            Some("agentflow/direct/AF-TASK-001".to_string()),
        )
        .unwrap();

        let checkpoint = write_task_run_checkpoint(
            dir.path(),
            "AF-TASK-001",
            "run-001",
            WorkflowFlowType::Work,
            "in_progress",
            "evt-000001",
            Some("corr-AF-TASK-001".to_string()),
            "已进入执行阶段",
        )
        .unwrap();

        let latest = latest_task_run_checkpoint(dir.path(), "AF-TASK-001", "run-001")
            .unwrap()
            .unwrap();
        let cursor = checkpoint_replay_cursor(&latest);

        assert_eq!(checkpoint.checkpoint_id, "checkpoint-001");
        assert_eq!(latest.state, "in_progress");
        assert_eq!(cursor.flow_type, WorkflowFlowType::Work);
        assert_eq!(cursor.run_id.as_deref(), Some("run-001"));
        assert_eq!(cursor.after_event_id, "evt-000001");
    }

    #[test]
    fn validation_fails_when_any_command_failed() {
        let dir = tempdir().unwrap();
        create_task_run(
            dir.path(),
            "AF-TASK-001",
            "run-001",
            "work-agent.issue-loop@v1",
            None,
        )
        .unwrap();
        write_task_command_record(dir.path(), "AF-TASK-001", "run-001", command(1)).unwrap();

        let validation = write_task_validation(dir.path(), "AF-TASK-001", "run-001").unwrap();

        assert!(!validation.passed);
        assert_eq!(validation.failed_command_ids, vec!["cmd-001"]);
    }

    #[test]
    fn rejects_path_like_issue_ids() {
        let dir = tempdir().unwrap();
        let err = create_task_run(
            dir.path(),
            "../bad",
            "run-001",
            "work-agent.issue-loop@v1",
            None,
        )
        .unwrap_err()
        .to_string();

        assert!(err.contains("local id"));
    }

    #[test]
    fn rejects_path_like_run_ids() {
        let dir = tempdir().unwrap();
        let err = create_task_run(
            dir.path(),
            "AF-TASK-001",
            "../run",
            "work-agent.issue-loop@v1",
            None,
        )
        .unwrap_err()
        .to_string();

        assert!(err.contains("local id"));
    }
}
