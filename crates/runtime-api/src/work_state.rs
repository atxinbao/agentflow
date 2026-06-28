use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, bail, Result};
use serde_json::{json, Value};

use agentflow_action_arbitration::{
    arbitrate_action, ArbitrationDecision, ArbitrationDecisionStatus, ArbitrationRequest,
    DefinitionVersions, EvidenceFact, StateFact,
};
use agentflow_action_contract::{core_action_contract_registry, ActionRef, ActionSourceSurface};
use agentflow_object_state::core_object_state_registry;
use agentflow_ontology::software_dev_reference_ontology_registry;
use agentflow_spec::SpecIssueStatus;
use agentflow_task_artifacts::TaskRunStatus;

use crate::commands::{build_project_arbitration_context, RuntimeCommandRequest};
use crate::mapping::map_command_to_action_proposal;

pub fn issue_surface_state_id(status: &SpecIssueStatus) -> &'static str {
    match status {
        SpecIssueStatus::Backlog => "proposed",
        SpecIssueStatus::Todo => "ready",
        SpecIssueStatus::InProgress => "running",
        SpecIssueStatus::InReview => "reviewReady",
        SpecIssueStatus::Done => "done",
        SpecIssueStatus::Blocked => "blocked",
        SpecIssueStatus::Cancel => "cancelled",
    }
}

pub fn run_surface_state_id(status: &TaskRunStatus) -> &'static str {
    match status {
        TaskRunStatus::Queued => "queued",
        TaskRunStatus::InProgress => "started",
        TaskRunStatus::Validating => "started",
        TaskRunStatus::Completed => "completed",
        TaskRunStatus::Failed => "failed",
        TaskRunStatus::Cancelled => "cancelled",
    }
}

pub fn assert_issue_activation_allowed(
    project_root: impl AsRef<Path>,
    issue_id: &str,
    status: &SpecIssueStatus,
) -> Result<String> {
    arbitrate_issue_action(
        project_root,
        issue_id,
        status,
        "activateIssue",
        json!({
            "activationReason": "task-loop scheduled issue into todo"
        }),
        &[],
    )
}

pub fn assert_issue_start_run_allowed(
    project_root: impl AsRef<Path>,
    issue_id: &str,
    status: &SpecIssueStatus,
    run_id: &str,
) -> Result<String> {
    arbitrate_issue_action(
        project_root,
        issue_id,
        status,
        "startRun",
        json!({
            "runId": run_id
        }),
        &[],
    )
}

pub fn assert_issue_mark_done_allowed(
    project_root: impl AsRef<Path>,
    issue_id: &str,
    status: &SpecIssueStatus,
) -> Result<String> {
    arbitrate_issue_action(
        project_root,
        issue_id,
        status,
        "markIssueDone",
        json!({
            "completionSummary": "build-agent completion gate passed"
        }),
        &[
            "implementationSummary",
            "verificationLog",
            "artifactSummary",
        ],
    )
}

pub fn assert_issue_transition(status: &SpecIssueStatus, action_type: &str) -> Result<String> {
    resolve_object_transition("Issue", issue_surface_state_id(status), action_type)
}

pub fn assert_run_transition(status: &TaskRunStatus, action_type: &str) -> Result<String> {
    resolve_object_transition("Run", run_surface_state_id(status), action_type)
}

fn arbitrate_issue_action(
    project_root: impl AsRef<Path>,
    issue_id: &str,
    status: &SpecIssueStatus,
    command_type: &str,
    input: Value,
    evidence_types: &[&str],
) -> Result<String> {
    let mut context = build_project_arbitration_context(project_root)?;
    context.insert_state(StateFact {
        object_type: "Issue".to_string(),
        object_id: issue_id.to_string(),
        state_id: issue_surface_state_id(status).to_string(),
    });

    let evidence_refs = evidence_types
        .iter()
        .map(|kind| {
            let evidence_ref = format!("state-guard:{issue_id}:{command_type}:{kind}");
            context.insert_evidence(EvidenceFact {
                evidence_ref: evidence_ref.clone(),
                evidence_type: (*kind).to_string(),
            });
            evidence_ref
        })
        .collect::<Vec<_>>();

    let request = RuntimeCommandRequest {
        command_id: format!("state-guard-{issue_id}-{command_type}"),
        command_type: command_type.to_string(),
        source_surface: ActionSourceSurface::System,
        actor_role: "work-agent".to_string(),
        target_object_ref: Some(ActionRef {
            object_type: "Issue".to_string(),
            id: issue_id.to_string(),
        }),
        input,
        evidence_refs,
        artifact_refs: Vec::new(),
        idempotency_key: format!(
            "system:work-agent:{command_type}:Issue:{issue_id}:{}",
            status.as_str()
        ),
        created_at: unix_timestamp_seconds().to_string(),
    };
    let proposal = map_command_to_action_proposal(&request)?;
    let arbitration_request = ArbitrationRequest {
        request_id: request.command_id.clone(),
        proposal,
        definition_versions: definition_versions(&context),
        requested_at: request.created_at.clone(),
    };
    let decision = arbitrate_action(&arbitration_request, &context);
    accepted_next_state(
        "Issue",
        issue_surface_state_id(status),
        command_type,
        decision,
    )
}

fn resolve_object_transition(
    object_type: &str,
    current_state: &str,
    action_type: &str,
) -> Result<String> {
    let ontology = software_dev_reference_ontology_registry();
    let contracts = core_action_contract_registry(&ontology);
    let registry = core_object_state_registry(&ontology, &contracts)
        .map_err(|report| anyhow!("load core object state registry: {:?}", report))?;
    let decision = registry.is_transition_defined(object_type, Some(current_state), action_type);
    if !decision.allowed {
        bail!(
            "illegal {} transition from {} via {}: {}",
            object_type,
            current_state,
            action_type,
            decision.reason
        );
    }
    decision.next_state.ok_or_else(|| {
        anyhow!(
            "{} transition from {} via {} returned no next state",
            object_type,
            current_state,
            action_type
        )
    })
}

fn accepted_next_state(
    object_type: &str,
    current_state: &str,
    action_type: &str,
    decision: ArbitrationDecision,
) -> Result<String> {
    match decision.status {
        ArbitrationDecisionStatus::Accepted => decision
            .accepted_action
            .and_then(|accepted| accepted.to_state)
            .ok_or_else(|| {
                anyhow!(
                    "{} transition from {} via {} returned no accepted next state",
                    object_type,
                    current_state,
                    action_type
                )
            }),
        _ => {
            let reasons = decision
                .rejected_reasons
                .iter()
                .map(|reason| reason.message.as_str())
                .collect::<Vec<_>>();
            bail!(
                "illegal {} transition from {} via {}: {}",
                object_type,
                current_state,
                action_type,
                if reasons.is_empty() {
                    "arbitration rejected transition".to_string()
                } else {
                    reasons.join("; ")
                }
            );
        }
    }
}

fn definition_versions(
    context: &agentflow_action_arbitration::ArbitrationContext,
) -> DefinitionVersions {
    DefinitionVersions {
        ontology_version: context
            .ontology_registry
            .bundle()
            .definition_version
            .clone(),
        contract_version: context
            .action_contract_registry
            .bundle()
            .definition_version
            .clone(),
        role_policy_version: context
            .role_policy_registry
            .bundle()
            .definition_version
            .clone(),
        object_state_version: context
            .state_machine_registry
            .bundle()
            .definition_version
            .clone(),
    }
}

fn unix_timestamp_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::{
        assert_issue_mark_done_allowed, assert_issue_transition, assert_run_transition,
        issue_surface_state_id, run_surface_state_id,
    };
    use agentflow_spec::SpecIssueStatus;
    use agentflow_task_artifacts::TaskRunStatus;
    use tempfile::tempdir;

    #[test]
    fn issue_surface_status_maps_to_object_state() {
        assert_eq!(
            issue_surface_state_id(&SpecIssueStatus::Backlog),
            "proposed"
        );
        assert_eq!(issue_surface_state_id(&SpecIssueStatus::Todo), "ready");
        assert_eq!(
            issue_surface_state_id(&SpecIssueStatus::InProgress),
            "running"
        );
        assert_eq!(
            issue_surface_state_id(&SpecIssueStatus::InReview),
            "reviewReady"
        );
    }

    #[test]
    fn run_surface_status_maps_to_object_state() {
        assert_eq!(run_surface_state_id(&TaskRunStatus::Queued), "queued");
        assert_eq!(run_surface_state_id(&TaskRunStatus::InProgress), "started");
        assert_eq!(run_surface_state_id(&TaskRunStatus::Validating), "started");
    }

    #[test]
    fn issue_cannot_jump_from_backlog_to_done() {
        let err = assert_issue_mark_done_allowed(
            tempdir().unwrap().path(),
            "AF-001",
            &SpecIssueStatus::Backlog,
        )
        .unwrap_err();
        assert!(err.to_string().contains("illegal Issue transition"));
    }

    #[test]
    fn issue_running_can_enter_review() {
        let next = assert_issue_transition(&SpecIssueStatus::InProgress, "submitDelivery").unwrap();
        assert_eq!(next, "reviewReady");
    }

    #[test]
    fn run_started_can_complete() {
        let next = assert_run_transition(&TaskRunStatus::Validating, "completeRun").unwrap();
        assert_eq!(next, "completed");
    }

    #[test]
    fn run_queued_cannot_complete() {
        let err = assert_run_transition(&TaskRunStatus::Queued, "completeRun").unwrap_err();
        assert!(err.to_string().contains("illegal Run transition"));
    }
}
