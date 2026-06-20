use crate::commands::{validate_runtime_command, RuntimeCommandRequest};
use agentflow_action_contract::{ActionRef, ActionSourceSurface};
use agentflow_spec::{SpecExpectedOutputs, SpecIssue, SpecIssueCategory, SpecRequiredAgentRole};
use agentflow_task_artifacts::write_work_loop_filesystem_contract;
use agentflow_workflow_core::canonicalize_project_root;
use agentflow_workflow_runtime::{
    prepare_runtime_workspace, write_runtime_command_fact, RuntimeCommandFact,
    RuntimeCommandValidationFact, RUNTIME_COMMAND_FACT_VERSION,
};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

pub const WORK_COMMAND_HANDOFF_VERSION: &str = "work-command-handoff.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkCommandHandoff {
    pub version: String,
    pub command_id: String,
    pub command_path: String,
    pub runtime_role: String,
    pub provider_role_alias: String,
    pub issue_id: String,
    pub issue_path: String,
    pub work_loop_contract_path: String,
    pub source_requirement_id: String,
    pub source_requirement_path: String,
    pub source_spec_id: String,
    pub workflow_ref: String,
    pub allowed_paths: Vec<String>,
    pub forbidden_paths: Vec<String>,
    pub validation_commands: Vec<String>,
    pub expected_outputs: SpecExpectedOutputs,
}

pub fn write_work_command_handoff_from_spec_issue(
    project_root: impl AsRef<Path>,
    issue: &SpecIssue,
    run_id: &str,
) -> Result<WorkCommandHandoff> {
    let root = canonicalize_project_root(project_root)?;
    validate_spec_issue_handoff(issue)?;
    prepare_runtime_workspace(&root)?;
    let work_loop_contract =
        write_work_loop_filesystem_contract(&root, &issue.issue_id, &issue.workflow_ref)?;
    let command_id = format!("start-run-{}-{run_id}", issue.issue_id);
    let command_request = RuntimeCommandRequest {
        command_id: command_id.clone(),
        command_type: "startRun".to_string(),
        source_surface: ActionSourceSurface::System,
        actor_role: "work-agent".to_string(),
        target_object_ref: Some(ActionRef {
            object_type: "Issue".to_string(),
            id: issue.issue_id.clone(),
        }),
        input: json!({
            "issueId": issue.issue_id.clone(),
            "runId": run_id,
            "workflowRef": issue.workflow_ref.clone(),
            "sourceRequirementId": issue.source_requirement_id.clone(),
            "sourceRequirementPath": issue.source_requirement_path.clone(),
            "sourceSpecId": issue.source_spec_id.clone(),
            "allowedPaths": issue.allowed_paths.clone(),
            "forbiddenPaths": issue.forbidden_paths.clone(),
            "validationCommands": issue.validation_commands.clone(),
            "expectedOutputs": issue.expected_outputs.clone(),
            "issuePath": issue.system.path.clone(),
            "requiredAgentRole": issue.required_agent_role.provider_role_alias(),
            "issueCategory": "spec"
        }),
        evidence_refs: vec![
            issue.system.path.clone(),
            issue.source_requirement_path.clone(),
        ],
        artifact_refs: vec![
            issue.system.path.clone(),
            work_loop_contract.contract_path.clone(),
        ],
        idempotency_key: format!("spec-issue:{}:start-run:{run_id}", issue.issue_id),
        created_at: unix_timestamp_seconds().to_string(),
    };
    let validation = validate_runtime_command(&command_request);
    let command_fact = RuntimeCommandFact {
        version: RUNTIME_COMMAND_FACT_VERSION.to_string(),
        command_id: command_request.command_id.clone(),
        command_type: command_request.command_type.clone(),
        source_surface: command_request.source_surface.clone(),
        actor_role: command_request.actor_role.clone(),
        target_object_ref: command_request.target_object_ref.clone(),
        input: command_request.input.clone(),
        evidence_refs: command_request.evidence_refs.clone(),
        artifact_refs: command_request.artifact_refs.clone(),
        idempotency_key: command_request.idempotency_key.clone(),
        created_at: command_request.created_at.clone(),
        recorded_at: unix_timestamp_seconds(),
        validation: RuntimeCommandValidationFact {
            valid: validation.valid,
            normalized_action_type: validation.normalized_action_type.clone(),
            errors: validation
                .errors
                .iter()
                .map(|error| error.message.clone())
                .collect(),
            warnings: validation.warnings.clone(),
        },
    };
    let command_path = write_runtime_command_fact(&root, &command_fact)?;

    Ok(WorkCommandHandoff {
        version: WORK_COMMAND_HANDOFF_VERSION.to_string(),
        command_id,
        command_path: normalize_relative_to_root(&root, &command_path)?,
        runtime_role: issue
            .required_agent_role
            .runtime_role()
            .as_str()
            .to_string(),
        provider_role_alias: issue.required_agent_role.provider_role_alias().to_string(),
        issue_id: issue.issue_id.clone(),
        issue_path: issue.system.path.clone(),
        work_loop_contract_path: work_loop_contract.contract_path,
        source_requirement_id: issue.source_requirement_id.clone(),
        source_requirement_path: issue.source_requirement_path.clone(),
        source_spec_id: issue.source_spec_id.clone(),
        workflow_ref: issue.workflow_ref.clone(),
        allowed_paths: issue.allowed_paths.clone(),
        forbidden_paths: issue.forbidden_paths.clone(),
        validation_commands: issue.validation_commands.clone(),
        expected_outputs: issue.expected_outputs.clone(),
    })
}

fn validate_spec_issue_handoff(issue: &SpecIssue) -> Result<()> {
    if issue.issue_category != SpecIssueCategory::Spec {
        anyhow::bail!(
            "only spec issues can generate work commands, found {}",
            match &issue.issue_category {
                SpecIssueCategory::Spec => "spec",
                SpecIssueCategory::Audit => "audit",
            }
        );
    }
    if issue.required_agent_role != SpecRequiredAgentRole::BuildAgent {
        anyhow::bail!(
            "issue {} must require build-agent/work-agent, found {}",
            issue.issue_id,
            issue.required_agent_role.provider_role_alias()
        );
    }
    validate_required("sourceRequirementId", &issue.source_requirement_id)?;
    validate_required("sourceRequirementPath", &issue.source_requirement_path)?;
    validate_required("sourceSpecId", &issue.source_spec_id)?;
    validate_required("workflowRef", &issue.workflow_ref)?;
    validate_required("issuePath", &issue.system.path)?;
    if !issue.system.path.starts_with(".agentflow/spec/issues/") {
        anyhow::bail!(
            "issue {} must come from .agentflow/spec/issues/**, found {}",
            issue.issue_id,
            issue.system.path
        );
    }
    if issue.allowed_paths.is_empty() {
        anyhow::bail!("issue {} missing allowedPaths", issue.issue_id);
    }
    if issue.forbidden_paths.is_empty() {
        anyhow::bail!("issue {} missing forbiddenPaths", issue.issue_id);
    }
    if issue.validation_commands.is_empty() {
        anyhow::bail!("issue {} missing validationCommands", issue.issue_id);
    }
    validate_required(
        "expectedOutputs.taskRunDir",
        &issue.expected_outputs.task_run_dir,
    )?;
    validate_required(
        "expectedOutputs.evidencePath",
        &issue.expected_outputs.evidence_path,
    )?;
    validate_required(
        "expectedOutputs.publicDeliveryRecord.changelogOrReleaseNotes",
        &issue
            .expected_outputs
            .public_delivery_record
            .changelog_or_release_notes,
    )?;
    Ok(())
}

fn validate_required(field: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        anyhow::bail!("{field} is required");
    }
    Ok(())
}

fn normalize_relative_to_root(root: &Path, path: &Path) -> Result<String> {
    let relative = path
        .strip_prefix(root)
        .with_context(|| format!("{} is outside {}", path.display(), root.display()))?;
    Ok(relative
        .components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/"))
}

fn unix_timestamp_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use agentflow_spec::{issue_from_requirement, write_spec_issue, SpecIssueDraft, SpecPriority};
    use agentflow_workflow_runtime::load_runtime_command_fact;
    use std::path::PathBuf;
    use tempfile::tempdir;

    fn write_requirement(root: &Path) -> PathBuf {
        let path = root.join("docs/requirements/060-test.md");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, "# 测试需求\n\n用于 work command handoff。\n").unwrap();
        path
    }

    fn ready_issue(root: &Path, issue_id: &str) -> SpecIssue {
        let requirement = write_requirement(root);
        let mut draft = SpecIssueDraft::new(issue_id);
        draft.priority = SpecPriority::P1;
        draft.allowed_paths = vec!["apps/desktop/src/**".to_string()];
        draft.forbidden_paths = vec![".agentflow/**".to_string()];
        draft.validation_commands = vec!["npm --prefix apps/desktop run build".to_string()];
        issue_from_requirement(root, &requirement, draft).unwrap()
    }

    #[test]
    fn writes_runtime_command_from_valid_spec_issue() {
        let dir = tempdir().unwrap();
        let issue = ready_issue(dir.path(), "AF-WORK-001");
        write_spec_issue(dir.path(), &issue).unwrap();

        let handoff =
            write_work_command_handoff_from_spec_issue(dir.path(), &issue, "run-001").unwrap();

        assert_eq!(handoff.runtime_role, "work-agent");
        assert_eq!(handoff.provider_role_alias, "build-agent");
        assert_eq!(
            handoff.command_path,
            ".agentflow/runtime/commands/start-run-AF-WORK-001-run-001.json"
        );
        assert_eq!(
            handoff.work_loop_contract_path,
            ".agentflow/tasks/AF-WORK-001/work-loop-contract.json"
        );
        assert!(dir
            .path()
            .join(".agentflow/runtime/commands/start-run-AF-WORK-001-run-001.json")
            .is_file());
        assert!(dir
            .path()
            .join(".agentflow/tasks/AF-WORK-001/work-loop-contract.json")
            .is_file());

        let command =
            load_runtime_command_fact(dir.path(), "start-run-AF-WORK-001-run-001").unwrap();
        assert_eq!(command.command_type, "startRun");
        assert_eq!(command.actor_role, "work-agent");
        assert_eq!(
            command
                .input
                .get("sourceRequirementId")
                .and_then(|v| v.as_str()),
            Some(issue.source_requirement_id.as_str())
        );
        assert_eq!(
            command
                .input
                .get("validationCommands")
                .and_then(|v| v.as_array())
                .map(|v| v.len()),
            Some(1)
        );
    }

    #[test]
    fn rejects_spec_issue_with_missing_contract_fields() {
        let dir = tempdir().unwrap();
        let mut issue = ready_issue(dir.path(), "AF-WORK-002");
        issue.validation_commands.clear();

        let err = write_work_command_handoff_from_spec_issue(dir.path(), &issue, "run-001")
            .unwrap_err()
            .to_string();

        assert!(err.contains("validationCommands"));
    }
}
