use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::Path;

use agentflow_action_arbitration::AcceptedAction;
use agentflow_workflow_core::{WorkflowAgentRole, WorkflowFlowType};

use crate::model::{EventActor, EventStateTransition, ReplayFilter, TaskEvent, TaskEventDraft};
use crate::storage::{append_task_event_once, replay_task_events};

pub const RUNTIME_EVENT_ENVELOPE_VERSION: &str = "runtime-event-envelope.v1";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeEventEnvelope {
    pub event_id: String,
    pub event_type: String,
    pub schema_version: String,
    pub ontology_version: String,
    pub action_contract_version: String,
    pub role_policy_version: String,
    pub state_machine_version: String,
    pub action_proposal_id: String,
    pub accepted_action_id: String,
    pub action_type: String,
    pub actor_role: String,
    pub object_type: String,
    pub object_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_state: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to_state: Option<String>,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
    #[serde(default)]
    pub artifact_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decision: Option<String>,
    pub causation_id: String,
    pub correlation_id: String,
    pub idempotency_key: String,
    pub occurred_at: u64,
    pub recorded_at: u64,
    pub payload: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompatibilityRuntimeEvent {
    pub envelope: RuntimeEventEnvelope,
    #[serde(default)]
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AcceptedActionAppendContext {
    pub flow_type: WorkflowFlowType,
    pub aggregate_type: String,
    pub aggregate_id: String,
    pub project_id: Option<String>,
    pub issue_id: Option<String>,
    pub run_id: Option<String>,
    pub event_type: String,
    pub authority_role: Option<WorkflowAgentRole>,
    pub actor_kind: String,
    pub correlation_id: String,
    pub causation_id: Option<String>,
    pub occurred_at: Option<u64>,
    pub decision: Option<String>,
    pub payload: Value,
}

pub fn append_accepted_action_event(
    project_root: impl AsRef<Path>,
    accepted_action: &AcceptedAction,
    context: AcceptedActionAppendContext,
) -> Result<TaskEvent> {
    let project_root = project_root.as_ref().to_path_buf();
    validate_append_request(accepted_action, &context)?;

    let draft = TaskEventDraft {
        flow_type: context.flow_type,
        aggregate_type: context.aggregate_type.clone(),
        aggregate_id: context.aggregate_id.clone(),
        project_id: context.project_id.clone(),
        issue_id: context.issue_id.clone(),
        run_id: context.run_id.clone(),
        event_type: context.event_type.clone(),
        authority_role: context.authority_role,
        actor: EventActor {
            role: accepted_action.actor_role.clone(),
            kind: context.actor_kind.clone(),
        },
        state: match (&accepted_action.from_state, &accepted_action.to_state) {
            (Some(from_state), Some(to_state)) => Some(EventStateTransition {
                from_state: from_state.clone(),
                to_state: to_state.clone(),
            }),
            _ => None,
        },
        correlation_id: Some(context.correlation_id.clone()),
        causation_id: Some(
            context
                .causation_id
                .clone()
                .unwrap_or_else(|| accepted_action.proposal_id.clone()),
        ),
        payload: attach_runtime_envelope_payload(accepted_action, &context, None, 0, 0),
        artifact_refs: accepted_action.artifact_refs.clone(),
        idempotency_key: Some(accepted_action.idempotency_key.clone()),
    };

    let event = append_task_event_once(&project_root, draft)?;
    let envelope = build_runtime_event_envelope(
        accepted_action,
        &context,
        &event.event_id,
        event.timestamp,
        event.timestamp,
    )?;

    let mut patched_event = event.clone();
    patched_event.payload = attach_runtime_envelope_payload(
        accepted_action,
        &context,
        Some(&event.event_id),
        event.timestamp,
        event.timestamp,
    );
    std::fs::write(
        project_root
            .join(".agentflow/events/task-events")
            .join(format!("{}.json", event.event_id)),
        serde_json::to_vec_pretty(&patched_event)?,
    )?;
    sync_stream_with_runtime_payload(project_root.as_path(), &event.event_id, &patched_event)?;

    let mut result = event;
    result.payload = serde_json::to_value(json!({ "runtimeEvent": envelope }))?;
    Ok(result)
}

pub fn build_runtime_event_envelope(
    accepted_action: &AcceptedAction,
    context: &AcceptedActionAppendContext,
    event_id: &str,
    occurred_at: u64,
    recorded_at: u64,
) -> Result<RuntimeEventEnvelope> {
    validate_append_request(accepted_action, context)?;

    Ok(RuntimeEventEnvelope {
        event_id: event_id.to_string(),
        event_type: context.event_type.clone(),
        schema_version: RUNTIME_EVENT_ENVELOPE_VERSION.into(),
        ontology_version: accepted_action.definition_versions.ontology_version.clone(),
        action_contract_version: accepted_action.definition_versions.contract_version.clone(),
        role_policy_version: accepted_action
            .definition_versions
            .role_policy_version
            .clone(),
        state_machine_version: accepted_action
            .definition_versions
            .object_state_version
            .clone(),
        action_proposal_id: accepted_action.proposal_id.clone(),
        accepted_action_id: accepted_action.accepted_action_id.clone(),
        action_type: accepted_action.action_type.clone(),
        actor_role: accepted_action.actor_role.clone(),
        object_type: context.aggregate_type.clone(),
        object_id: context.aggregate_id.clone(),
        from_state: accepted_action.from_state.clone(),
        to_state: accepted_action.to_state.clone(),
        evidence_refs: accepted_action.evidence_refs.clone(),
        artifact_refs: accepted_action.artifact_refs.clone(),
        decision: context.decision.clone(),
        causation_id: context
            .causation_id
            .clone()
            .unwrap_or_else(|| accepted_action.proposal_id.clone()),
        correlation_id: context.correlation_id.clone(),
        idempotency_key: accepted_action.idempotency_key.clone(),
        occurred_at,
        recorded_at,
        payload: context.payload.clone(),
    })
}

pub fn replay_runtime_events(
    project_root: impl AsRef<Path>,
    filter: ReplayFilter,
) -> Result<Vec<CompatibilityRuntimeEvent>> {
    replay_task_events(project_root, filter)?
        .into_iter()
        .map(|event| map_task_event_to_runtime_event(&event))
        .collect()
}

pub fn map_task_event_to_runtime_event(event: &TaskEvent) -> Result<CompatibilityRuntimeEvent> {
    if let Some(runtime_event) = event.payload.get("runtimeEvent") {
        let envelope: RuntimeEventEnvelope = serde_json::from_value(runtime_event.clone())?;
        return Ok(CompatibilityRuntimeEvent {
            envelope,
            warnings: Vec::new(),
        });
    }

    let mut warnings = vec!["legacy-task-event-envelope".to_string()];
    if event.causation_id.is_none() {
        warnings.push("missing-causation-id".into());
    }
    if event.idempotency_key.is_none() {
        warnings.push("missing-idempotency-key".into());
    }

    Ok(CompatibilityRuntimeEvent {
        envelope: RuntimeEventEnvelope {
            event_id: event.event_id.clone(),
            event_type: event.event_type.clone(),
            schema_version: event.event_version.clone(),
            ontology_version: payload_string(&event.payload, "ontologyVersion")
                .unwrap_or_else(|| "legacy".into()),
            action_contract_version: payload_string(&event.payload, "actionContractVersion")
                .unwrap_or_else(|| "legacy".into()),
            role_policy_version: payload_string(&event.payload, "rolePolicyVersion")
                .unwrap_or_else(|| "legacy".into()),
            state_machine_version: payload_string(&event.payload, "stateMachineVersion")
                .unwrap_or_else(|| "legacy".into()),
            action_proposal_id: payload_string(&event.payload, "actionProposalId")
                .or_else(|| event.causation_id.clone())
                .unwrap_or_else(|| format!("legacy-proposal-{}", event.event_id)),
            accepted_action_id: payload_string(&event.payload, "acceptedActionId")
                .unwrap_or_else(|| format!("legacy-accepted-{}", event.event_id)),
            action_type: payload_string(&event.payload, "actionType")
                .unwrap_or_else(|| event.event_type.clone()),
            actor_role: event.actor.role.clone(),
            object_type: event.aggregate_type.clone(),
            object_id: event.aggregate_id.clone(),
            from_state: event.state.as_ref().map(|state| state.from_state.clone()),
            to_state: event.state.as_ref().map(|state| state.to_state.clone()),
            evidence_refs: payload_string_vec(&event.payload, "evidenceRefs"),
            artifact_refs: event.artifact_refs.clone(),
            decision: payload_string(&event.payload, "decision"),
            causation_id: event
                .causation_id
                .clone()
                .unwrap_or_else(|| format!("legacy-cause-{}", event.event_id)),
            correlation_id: event.correlation_id.clone(),
            idempotency_key: event
                .idempotency_key
                .clone()
                .unwrap_or_else(|| format!("legacy:idempotency:{}", event.event_id)),
            occurred_at: event.timestamp,
            recorded_at: event.timestamp,
            payload: event.payload.clone(),
        },
        warnings,
    })
}

fn validate_append_request(
    accepted_action: &AcceptedAction,
    context: &AcceptedActionAppendContext,
) -> Result<()> {
    if accepted_action.accepted_action_id.trim().is_empty() {
        return Err(anyhow!("acceptedActionId is required"));
    }
    if accepted_action.proposal_id.trim().is_empty() {
        return Err(anyhow!("actionProposalId is required"));
    }
    if accepted_action.action_type.trim().is_empty() {
        return Err(anyhow!("actionType is required"));
    }
    if accepted_action.idempotency_key.trim().is_empty() {
        return Err(anyhow!("idempotencyKey is required"));
    }
    if context.correlation_id.trim().is_empty() {
        return Err(anyhow!("correlationId is required"));
    }
    if !accepted_action
        .expected_events
        .iter()
        .any(|event_type| event_type == &context.event_type)
    {
        return Err(anyhow!(
            "eventType `{}` does not match accepted action expected events",
            context.event_type
        ));
    }
    Ok(())
}

fn attach_runtime_envelope_payload(
    accepted_action: &AcceptedAction,
    context: &AcceptedActionAppendContext,
    event_id: Option<&str>,
    occurred_at: u64,
    recorded_at: u64,
) -> Value {
    let envelope = RuntimeEventEnvelope {
        event_id: event_id.unwrap_or("pending-event-id").to_string(),
        event_type: context.event_type.clone(),
        schema_version: RUNTIME_EVENT_ENVELOPE_VERSION.into(),
        ontology_version: accepted_action.definition_versions.ontology_version.clone(),
        action_contract_version: accepted_action.definition_versions.contract_version.clone(),
        role_policy_version: accepted_action
            .definition_versions
            .role_policy_version
            .clone(),
        state_machine_version: accepted_action
            .definition_versions
            .object_state_version
            .clone(),
        action_proposal_id: accepted_action.proposal_id.clone(),
        accepted_action_id: accepted_action.accepted_action_id.clone(),
        action_type: accepted_action.action_type.clone(),
        actor_role: accepted_action.actor_role.clone(),
        object_type: context.aggregate_type.clone(),
        object_id: context.aggregate_id.clone(),
        from_state: accepted_action.from_state.clone(),
        to_state: accepted_action.to_state.clone(),
        evidence_refs: accepted_action.evidence_refs.clone(),
        artifact_refs: accepted_action.artifact_refs.clone(),
        decision: context.decision.clone(),
        causation_id: context
            .causation_id
            .clone()
            .unwrap_or_else(|| accepted_action.proposal_id.clone()),
        correlation_id: context.correlation_id.clone(),
        idempotency_key: accepted_action.idempotency_key.clone(),
        occurred_at,
        recorded_at,
        payload: context.payload.clone(),
    };
    json!({ "runtimeEvent": envelope })
}

fn sync_stream_with_runtime_payload(
    project_root: &Path,
    event_id: &str,
    patched_event: &TaskEvent,
) -> Result<()> {
    let stream_path = project_root.join(".agentflow/events/task-events.jsonl");
    let raw = std::fs::read_to_string(&stream_path)?;
    let mut replaced = false;
    let mut lines = Vec::new();
    for line in raw.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let current: TaskEvent = serde_json::from_str(line)?;
        if current.event_id == event_id {
            lines.push(serde_json::to_string(patched_event)?);
            replaced = true;
        } else {
            lines.push(line.to_string());
        }
    }
    if !replaced {
        return Err(anyhow!(
            "failed to locate persisted task event `{event_id}`"
        ));
    }
    std::fs::write(&stream_path, format!("{}\n", lines.join("\n")))?;
    Ok(())
}

fn payload_string(payload: &Value, key: &str) -> Option<String> {
    payload.get(key).and_then(Value::as_str).map(str::to_string)
}

fn payload_string_vec(payload: &Value, key: &str) -> Vec<String> {
    payload
        .get(key)
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use tempfile::tempdir;

    use agentflow_action_arbitration::{AcceptedAction, DefinitionVersions};
    use agentflow_action_contract::ActionRef;
    use agentflow_workflow_core::{WorkflowAgentRole, WorkflowFlowType};

    use crate::model::{EventActor, TaskEventDraft};
    use crate::storage::{append_task_event, load_task_events};

    use super::{
        append_accepted_action_event, map_task_event_to_runtime_event, replay_runtime_events,
        AcceptedActionAppendContext,
    };

    #[test]
    fn accepted_action_appends_runtime_event() {
        let dir = tempdir().unwrap();
        let accepted = accepted_action();
        let event = append_accepted_action_event(dir.path(), &accepted, append_context()).unwrap();
        let mapped = map_task_event_to_runtime_event(&event).unwrap();
        assert_eq!(
            mapped.envelope.accepted_action_id,
            accepted.accepted_action_id
        );
        assert_eq!(mapped.envelope.action_proposal_id, accepted.proposal_id);
        assert_eq!(mapped.envelope.event_type, "IssueMarkedDone");
    }

    #[test]
    fn missing_accepted_action_id_fails() {
        let dir = tempdir().unwrap();
        let mut accepted = accepted_action();
        accepted.accepted_action_id.clear();
        let err = append_accepted_action_event(dir.path(), &accepted, append_context())
            .unwrap_err()
            .to_string();
        assert!(err.contains("acceptedActionId"));
    }

    #[test]
    fn missing_idempotency_key_fails() {
        let dir = tempdir().unwrap();
        let mut accepted = accepted_action();
        accepted.idempotency_key.clear();
        let err = append_accepted_action_event(dir.path(), &accepted, append_context())
            .unwrap_err()
            .to_string();
        assert!(err.contains("idempotencyKey"));
    }

    #[test]
    fn duplicate_idempotency_key_is_handled_deterministically() {
        let dir = tempdir().unwrap();
        let accepted = accepted_action();
        let first = append_accepted_action_event(dir.path(), &accepted, append_context()).unwrap();
        let second = append_accepted_action_event(dir.path(), &accepted, append_context()).unwrap();
        assert_eq!(first.event_id, second.event_id);
        assert_eq!(load_task_events(dir.path()).unwrap().len(), 1);
    }

    #[test]
    fn event_has_causation_and_correlation() {
        let dir = tempdir().unwrap();
        let accepted = accepted_action();
        let event = append_accepted_action_event(dir.path(), &accepted, append_context()).unwrap();
        let mapped = map_task_event_to_runtime_event(&event).unwrap();
        assert_eq!(mapped.envelope.causation_id, accepted.proposal_id);
        assert_eq!(mapped.envelope.correlation_id, "corr-issue-1");
    }

    #[test]
    fn expected_event_mismatch_fails() {
        let dir = tempdir().unwrap();
        let accepted = accepted_action();
        let mut context = append_context();
        context.event_type = "UnexpectedEvent".into();
        let err = append_accepted_action_event(dir.path(), &accepted, context)
            .unwrap_err()
            .to_string();
        assert!(err.contains("does not match accepted action expected events"));
    }

    #[test]
    fn replay_runtime_events_reads_events_only() {
        let dir = tempdir().unwrap();
        let accepted = accepted_action();
        append_accepted_action_event(dir.path(), &accepted, append_context()).unwrap();
        let replayed =
            replay_runtime_events(dir.path(), crate::model::ReplayFilter::issue("ISS-1")).unwrap();
        assert_eq!(replayed.len(), 1);
        assert!(replayed[0].warnings.is_empty());
    }

    #[test]
    fn old_task_event_maps_to_compatibility_event() {
        let dir = tempdir().unwrap();
        let legacy = append_task_event(
            dir.path(),
            TaskEventDraft {
                flow_type: WorkflowFlowType::Work,
                aggregate_type: "Issue".into(),
                aggregate_id: "ISS-1".into(),
                project_id: Some("PROJ-1".into()),
                issue_id: Some("ISS-1".into()),
                run_id: None,
                event_type: "issue.scheduled".into(),
                authority_role: Some(WorkflowAgentRole::WorkAgent),
                actor: EventActor {
                    role: "BuildAgent".into(),
                    kind: "agent".into(),
                },
                state: None,
                correlation_id: Some("corr-legacy".into()),
                causation_id: None,
                payload: json!({}),
                artifact_refs: Vec::new(),
                idempotency_key: None,
            },
        )
        .unwrap();

        let mapped = map_task_event_to_runtime_event(&legacy).unwrap();
        assert_eq!(mapped.envelope.object_type, "Issue");
        assert!(!mapped.warnings.is_empty());
    }

    #[test]
    fn old_task_event_remains_append_only() {
        let dir = tempdir().unwrap();
        append_task_event(
            dir.path(),
            TaskEventDraft {
                flow_type: WorkflowFlowType::Work,
                aggregate_type: "Issue".into(),
                aggregate_id: "ISS-1".into(),
                project_id: Some("PROJ-1".into()),
                issue_id: Some("ISS-1".into()),
                run_id: None,
                event_type: "issue.scheduled".into(),
                authority_role: Some(WorkflowAgentRole::WorkAgent),
                actor: EventActor {
                    role: "BuildAgent".into(),
                    kind: "agent".into(),
                },
                state: None,
                correlation_id: Some("corr-legacy".into()),
                causation_id: None,
                payload: json!({}),
                artifact_refs: Vec::new(),
                idempotency_key: None,
            },
        )
        .unwrap();
        append_accepted_action_event(dir.path(), &accepted_action(), append_context()).unwrap();
        assert_eq!(load_task_events(dir.path()).unwrap().len(), 2);
    }

    #[test]
    fn correction_appends_new_event_without_overwrite() {
        let dir = tempdir().unwrap();
        let first = accepted_action();
        let mut second = accepted_action();
        second.accepted_action_id = "accepted-proposal-markIssueDone-2".into();
        second.idempotency_key = "runtime:issue:ISS-1:markIssueDone:v2".into();
        append_accepted_action_event(dir.path(), &first, append_context()).unwrap();
        append_accepted_action_event(dir.path(), &second, append_context()).unwrap();
        assert_eq!(load_task_events(dir.path()).unwrap().len(), 2);
    }

    #[test]
    fn invalid_old_event_enters_compatibility_warning() {
        let legacy = crate::model::TaskEvent {
            event_id: "event-1".into(),
            event_version: "task-event.v2".into(),
            flow_type: WorkflowFlowType::Work,
            aggregate_type: "Issue".into(),
            aggregate_id: "ISS-1".into(),
            project_id: Some("PROJ-1".into()),
            issue_id: Some("ISS-1".into()),
            run_id: None,
            event_type: "issue.scheduled".into(),
            timestamp: 1,
            authority_role: Some(WorkflowAgentRole::WorkAgent),
            actor: EventActor {
                role: "BuildAgent".into(),
                kind: "agent".into(),
            },
            state: None,
            correlation_id: "corr-legacy".into(),
            causation_id: None,
            payload: json!({}),
            artifact_refs: Vec::new(),
            idempotency_key: None,
        };
        let mapped = map_task_event_to_runtime_event(&legacy).unwrap();
        assert!(mapped
            .warnings
            .iter()
            .any(|warning| warning == "legacy-task-event-envelope"));
    }

    fn accepted_action() -> AcceptedAction {
        AcceptedAction {
            accepted_action_id: "accepted-proposal-markIssueDone".into(),
            proposal_id: "proposal-markIssueDone".into(),
            idempotency_key: "runtime:issue:ISS-1:markIssueDone:v1".into(),
            action_type: "markIssueDone".into(),
            actor_role: "BuildAgent".into(),
            target_object_ref: Some(ActionRef {
                object_type: "Issue".into(),
                id: "ISS-1".into(),
            }),
            from_state: Some("reviewReady".into()),
            to_state: Some("done".into()),
            evidence_refs: vec!["verification-log".into()],
            artifact_refs: vec!["artifact-summary".into()],
            expected_events: vec!["IssueMarkedDone".into()],
            lock_plan: agentflow_action_arbitration::ObjectLockPlan::default(),
            definition_versions: DefinitionVersions {
                ontology_version: "v1-draft".into(),
                contract_version: "v1-draft".into(),
                role_policy_version: "v1-draft".into(),
                object_state_version: "v1-draft".into(),
            },
        }
    }

    fn append_context() -> AcceptedActionAppendContext {
        AcceptedActionAppendContext {
            flow_type: WorkflowFlowType::Work,
            aggregate_type: "Issue".into(),
            aggregate_id: "ISS-1".into(),
            project_id: Some("PROJ-1".into()),
            issue_id: Some("ISS-1".into()),
            run_id: Some("run-001".into()),
            event_type: "IssueMarkedDone".into(),
            authority_role: Some(WorkflowAgentRole::WorkAgent),
            actor_kind: "agent".into(),
            correlation_id: "corr-issue-1".into(),
            causation_id: None,
            occurred_at: None,
            decision: None,
            payload: json!({
                "summary": "done"
            }),
        }
    }
}
