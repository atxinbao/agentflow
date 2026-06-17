use crate::model::{
    TaskCommandInput, TaskCommandRecord, TaskEvidence, TaskRun, TaskRunCheckpoint, TaskRunStatus,
    TaskValidationRecord, TASK_COMMAND_VERSION, TASK_EVIDENCE_VERSION, TASK_RUN_CHECKPOINT_VERSION,
    TASK_RUN_VERSION, TASK_VALIDATION_VERSION,
};
use agentflow_event_store::TaskReplayCursor;
use agentflow_workflow_core::WorkflowFlowType;
use anyhow::{Context, Result};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

pub fn prepare_task_artifact_workspace(
    project_root: impl AsRef<Path>,
    issue_id: &str,
) -> Result<()> {
    let root = canonical_project_root(project_root)?;
    validate_id("issueId", issue_id)?;
    ensure_directory(&root.join(format!(".agentflow/tasks/{issue_id}/runs")))?;
    ensure_directory(&root.join(format!(".agentflow/tasks/{issue_id}/evidence")))?;
    Ok(())
}

pub fn task_run_dir(project_root: impl AsRef<Path>, issue_id: &str, run_id: &str) -> PathBuf {
    project_root
        .as_ref()
        .join(format!(".agentflow/tasks/{issue_id}/runs/{run_id}"))
}

pub fn task_evidence_dir(project_root: impl AsRef<Path>, issue_id: &str) -> PathBuf {
    project_root
        .as_ref()
        .join(format!(".agentflow/tasks/{issue_id}/evidence"))
}

pub fn create_task_run(
    project_root: impl AsRef<Path>,
    issue_id: &str,
    run_id: &str,
    workflow_ref: &str,
    branch_name: Option<String>,
) -> Result<TaskRun> {
    let root = canonical_project_root(project_root)?;
    validate_id("issueId", issue_id)?;
    validate_id("runId", run_id)?;
    validate_required("workflowRef", workflow_ref)?;
    prepare_task_artifact_workspace(&root, issue_id)?;
    let now = unix_timestamp_seconds();
    let run = TaskRun {
        version: TASK_RUN_VERSION.to_string(),
        issue_id: issue_id.to_string(),
        run_id: run_id.to_string(),
        workflow_ref: workflow_ref.to_string(),
        status: TaskRunStatus::Queued,
        branch_name,
        created_at: now,
        updated_at: now,
    };
    let run_directory = task_run_dir(&root, issue_id, run_id);
    ensure_directory(&run_directory)?;
    write_json(&run_directory.join("run.json"), &run)?;
    Ok(run)
}

pub fn load_task_run(
    project_root: impl AsRef<Path>,
    issue_id: &str,
    run_id: &str,
) -> Result<TaskRun> {
    let root = canonical_project_root(project_root)?;
    validate_id("issueId", issue_id)?;
    validate_id("runId", run_id)?;
    read_json(&task_run_dir(&root, issue_id, run_id).join("run.json"))
}

pub fn update_task_run_status(
    project_root: impl AsRef<Path>,
    issue_id: &str,
    run_id: &str,
    status: TaskRunStatus,
) -> Result<TaskRun> {
    let root = canonical_project_root(project_root)?;
    validate_id("issueId", issue_id)?;
    validate_id("runId", run_id)?;
    let path = task_run_dir(&root, issue_id, run_id).join("run.json");
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
    let root = canonical_project_root(project_root)?;
    let run = load_task_run(&root, issue_id, run_id)?;
    validate_required("label", &input.label)?;
    validate_required("program", &input.program)?;
    let command_dir = task_run_dir(&root, issue_id, run_id).join("commands");
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
        stdout_path: relative_command_path(issue_id, run_id, &command_id, "stdout.txt"),
        stderr_path: relative_command_path(issue_id, run_id, &command_id, "stderr.txt"),
        recorded_at: unix_timestamp_seconds(),
    };
    write_json(&command_dir.join(format!("{command_id}.json")), &record)?;
    Ok(record)
}

pub fn write_task_validation(
    project_root: impl AsRef<Path>,
    issue_id: &str,
    run_id: &str,
) -> Result<TaskValidationRecord> {
    let root = canonical_project_root(project_root)?;
    validate_id("issueId", issue_id)?;
    validate_id("runId", run_id)?;
    let command_records = load_command_records(&root, issue_id, run_id)?;
    if command_records.is_empty() {
        anyhow::bail!("task validation requires at least one command record");
    }
    let failed_command_ids = command_records
        .iter()
        .filter(|record| record.exit_code != Some(0))
        .map(|record| record.command_id.clone())
        .collect::<Vec<_>>();
    let validation = TaskValidationRecord {
        version: TASK_VALIDATION_VERSION.to_string(),
        issue_id: issue_id.to_string(),
        run_id: run_id.to_string(),
        passed: failed_command_ids.is_empty(),
        command_ids: command_records
            .iter()
            .map(|record| record.command_id.clone())
            .collect(),
        failed_command_ids,
        checked_at: unix_timestamp_seconds(),
    };
    write_json(
        &task_run_dir(&root, issue_id, run_id).join("validation.json"),
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
    let root = canonical_project_root(project_root)?;
    validate_id("issueId", issue_id)?;
    validate_id("runId", run_id)?;
    let validation_path = task_run_dir(&root, issue_id, run_id).join("validation.json");
    let validation: TaskValidationRecord = read_json(&validation_path)?;
    let command_records = load_command_records(&root, issue_id, run_id)?;
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
        run_path: format!(".agentflow/tasks/{issue_id}/runs/{run_id}/run.json"),
        command_paths: command_records
            .iter()
            .map(|record| {
                format!(
                    ".agentflow/tasks/{issue_id}/runs/{run_id}/commands/{}.json",
                    record.command_id
                )
            })
            .collect(),
        validation_path: format!(".agentflow/tasks/{issue_id}/runs/{run_id}/validation.json"),
        created_at: unix_timestamp_seconds(),
    };
    write_json(
        &task_evidence_dir(&root, issue_id).join("evidence.json"),
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
    let root = canonical_project_root(project_root)?;
    validate_id("issueId", issue_id)?;
    validate_id("runId", run_id)?;
    validate_required("state", state)?;
    validate_required("eventId", event_id)?;
    let checkpoint_dir = task_run_dir(&root, issue_id, run_id).join("checkpoints");
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
    let root = canonical_project_root(project_root)?;
    validate_id("issueId", issue_id)?;
    validate_id("runId", run_id)?;
    let checkpoint_dir = task_run_dir(&root, issue_id, run_id).join("checkpoints");
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
    let root = canonical_project_root(project_root)?;
    validate_id("issueId", issue_id)?;
    read_json(&task_evidence_dir(&root, issue_id).join("evidence.json"))
}

fn load_command_records(
    root: &Path,
    issue_id: &str,
    run_id: &str,
) -> Result<Vec<TaskCommandRecord>> {
    let command_dir = task_run_dir(root, issue_id, run_id).join("commands");
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

fn relative_command_path(issue_id: &str, run_id: &str, command_id: &str, suffix: &str) -> String {
    format!(".agentflow/tasks/{issue_id}/runs/{run_id}/commands/{command_id}.{suffix}")
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

fn validate_id(field: &str, value: &str) -> Result<()> {
    validate_required(field, value)?;
    if value.contains('/') || value.contains('\\') || value.contains("..") {
        anyhow::bail!("{field} must be a local id, found {value}");
    }
    Ok(())
}

fn validate_required(field: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        anyhow::bail!("{field} is required");
    }
    Ok(())
}

fn canonical_project_root(project_root: impl AsRef<Path>) -> Result<PathBuf> {
    project_root
        .as_ref()
        .canonicalize()
        .with_context(|| format!("canonicalize {}", project_root.as_ref().display()))
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

fn unix_timestamp_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
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
            "build-agent.issue-loop@v1",
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
            .join(".agentflow/tasks/AF-TASK-001/evidence")
            .is_dir());
    }

    #[test]
    fn writes_command_stdout_stderr_and_validation() {
        let dir = tempdir().unwrap();
        create_task_run(
            dir.path(),
            "AF-TASK-001",
            "run-001",
            "build-agent.issue-loop@v1",
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
            "build-agent.issue-loop@v1",
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
            "build-agent.issue-loop@v1",
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
    fn writes_checkpoints_and_builds_replay_cursor() {
        let dir = tempdir().unwrap();
        create_task_run(
            dir.path(),
            "AF-TASK-001",
            "run-001",
            "build-agent.issue-loop@v1",
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
            "build-agent.issue-loop@v1",
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
            "build-agent.issue-loop@v1",
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
            "build-agent.issue-loop@v1",
            None,
        )
        .unwrap_err()
        .to_string();

        assert!(err.contains("local id"));
    }
}
