use crate::model::{
    ActionOutcome, GuardOutcome, RuntimeContext, RuntimeHandoffBinding, RuntimeStateBinding,
    RuntimeTransition, RuntimeTransitionResult,
};
use agentflow_event_store::{append_task_event, EventStateTransition, TaskEventDraft};
use agentflow_workflow_core::{TransitionDefinition, WorkflowDefinition};
use anyhow::Result;
use serde_json::json;
use std::{collections::BTreeMap, path::Path};

pub trait GuardRegistry {
    fn evaluate(&self, guard_name: &str, context: &RuntimeContext) -> Result<GuardOutcome>;
}

pub trait ActionRegistry {
    fn execute(&self, action_name: &str, context: &RuntimeContext) -> Result<ActionOutcome>;
}

#[derive(Debug, Clone, Default)]
pub struct StaticGuardRegistry {
    outcomes: BTreeMap<String, GuardOutcome>,
}

impl StaticGuardRegistry {
    pub fn all_pass(guard_names: impl IntoIterator<Item = impl Into<String>>) -> Self {
        let mut registry = Self::default();
        for name in guard_names {
            let name = name.into();
            registry
                .outcomes
                .insert(name.clone(), GuardOutcome::passed(name));
        }
        registry
    }

    pub fn with_outcome(mut self, outcome: GuardOutcome) -> Self {
        self.outcomes.insert(outcome.name.clone(), outcome);
        self
    }
}

impl GuardRegistry for StaticGuardRegistry {
    fn evaluate(&self, guard_name: &str, _context: &RuntimeContext) -> Result<GuardOutcome> {
        Ok(self
            .outcomes
            .get(guard_name)
            .cloned()
            .unwrap_or_else(|| GuardOutcome::failed(guard_name, "guard outcome is not registered")))
    }
}

#[derive(Debug, Clone, Default)]
pub struct StaticActionRegistry {
    outcomes: BTreeMap<String, ActionOutcome>,
}

impl StaticActionRegistry {
    pub fn all_complete(action_names: impl IntoIterator<Item = impl Into<String>>) -> Self {
        let mut registry = Self::default();
        for name in action_names {
            let name = name.into();
            registry
                .outcomes
                .insert(name.clone(), ActionOutcome::completed(name));
        }
        registry
    }

    pub fn with_outcome(mut self, outcome: ActionOutcome) -> Self {
        self.outcomes.insert(outcome.name.clone(), outcome);
        self
    }
}

impl ActionRegistry for StaticActionRegistry {
    fn execute(&self, action_name: &str, _context: &RuntimeContext) -> Result<ActionOutcome> {
        Ok(self
            .outcomes
            .get(action_name)
            .cloned()
            .unwrap_or_else(|| ActionOutcome::completed(action_name)))
    }
}

pub fn find_transition<'a>(
    workflow: &'a WorkflowDefinition,
    current_state: &str,
    incoming_event_type: &str,
) -> Result<&'a TransitionDefinition> {
    let matches = workflow
        .spec
        .transitions
        .iter()
        .filter(|transition| transition.on == incoming_event_type)
        .filter(|transition| {
            transition
                .from_states
                .iter()
                .any(|state| state == current_state)
        })
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [transition] => Ok(*transition),
        [] => anyhow::bail!("no transition from {current_state} on event {incoming_event_type}"),
        _ => anyhow::bail!(
            "multiple transitions from {current_state} on event {incoming_event_type}"
        ),
    }
}

pub fn resolve_state_binding(
    workflow: &WorkflowDefinition,
    state_id: &str,
) -> Result<RuntimeStateBinding> {
    let state = workflow
        .spec
        .states
        .get(state_id)
        .ok_or_else(|| anyhow::anyhow!("state {state_id} is not defined"))?;
    let role = state
        .role
        .ok_or_else(|| anyhow::anyhow!("state {state_id} is missing role binding"))?;
    Ok(RuntimeStateBinding {
        state_id: state_id.to_string(),
        role,
        skill_pack: state.skill_pack,
    })
}

pub fn resolve_transition_handoff(
    workflow: &WorkflowDefinition,
    transition: &TransitionDefinition,
) -> Result<Option<RuntimeHandoffBinding>> {
    let Some(handoff) = &transition.handoff else {
        return Ok(None);
    };
    if !workflow.spec.states.contains_key(&transition.to) {
        anyhow::bail!("handoff target state {} is not defined", transition.to);
    }
    Ok(Some(RuntimeHandoffBinding {
        transition_id: transition.id.clone(),
        from_role: handoff.from_role,
        to_role: handoff.to_role,
        mode: handoff.mode,
        payload_ref: handoff.payload_ref.clone(),
        expected_state: handoff.expected_state.clone(),
    }))
}

pub fn apply_workflow_event(
    project_root: impl AsRef<Path>,
    workflow: &WorkflowDefinition,
    current_state: &str,
    incoming_event_type: &str,
    context: RuntimeContext,
    guards: &impl GuardRegistry,
    actions: &impl ActionRegistry,
) -> Result<RuntimeTransitionResult> {
    let transition = find_transition(workflow, current_state, incoming_event_type)?;
    let current_binding = resolve_state_binding(workflow, current_state)?;
    let next_binding = resolve_state_binding(workflow, &transition.to)?;
    let handoff = resolve_transition_handoff(workflow, transition)?;
    let runtime_transition = RuntimeTransition {
        transition_id: transition.id.clone(),
        from_state: current_state.to_string(),
        to_state: transition.to.clone(),
        event_type: incoming_event_type.to_string(),
    };

    let mut guard_outcomes = Vec::new();
    for guard in &transition.guards {
        let outcome = guards.evaluate(guard.name(), &context)?;
        if !outcome.passed {
            let reason = outcome
                .reason
                .clone()
                .unwrap_or_else(|| format!("guard {} failed", outcome.name));
            guard_outcomes.push(outcome);
            return Ok(RuntimeTransitionResult {
                applied: false,
                transition: Some(runtime_transition),
                current_binding: Some(current_binding),
                next_binding: Some(next_binding),
                handoff,
                guard_outcomes,
                action_outcomes: Vec::new(),
                event_id: None,
                blocked_reason: Some(reason),
            });
        }
        guard_outcomes.push(outcome);
    }

    let mut action_outcomes = Vec::new();
    let mut artifact_refs = context.artifact_refs.clone();
    for action in &transition.actions {
        let outcome = actions.execute(action.name(), &context)?;
        artifact_refs.extend(outcome.artifact_refs.iter().cloned());
        action_outcomes.push(outcome);
    }

    let event = append_task_event(
        project_root,
        TaskEventDraft {
            aggregate_type: context.aggregate_type,
            aggregate_id: context.aggregate_id,
            project_id: context.project_id,
            issue_id: context.issue_id,
            event_type: incoming_event_type.to_string(),
            actor: context.actor,
            state: Some(EventStateTransition {
                from_state: current_state.to_string(),
                to_state: transition.to.clone(),
            }),
            correlation_id: context.correlation_id,
            causation_id: context.causation_id,
            payload: json!({
                "workflowRef": workflow.workflow_ref(),
                "transitionId": transition.id,
                "guardsPassed": guard_outcomes
                    .iter()
                    .filter(|outcome| outcome.passed)
                    .map(|outcome| outcome.name.clone())
                    .collect::<Vec<_>>(),
                "actionsRun": action_outcomes
                    .iter()
                    .map(|outcome| outcome.name.clone())
                    .collect::<Vec<_>>(),
                "input": context.payload,
            }),
            artifact_refs,
            idempotency_key: None,
        },
    )?;

    Ok(RuntimeTransitionResult {
        applied: true,
        transition: Some(runtime_transition),
        current_binding: Some(current_binding),
        next_binding: Some(next_binding),
        handoff,
        guard_outcomes,
        action_outcomes,
        event_id: Some(event.event_id),
        blocked_reason: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use agentflow_event_store::{load_task_events, EventActor};
    use agentflow_workflow_core::{parse_workflow_yaml, validate_workflow};
    use tempfile::tempdir;

    fn workflow() -> WorkflowDefinition {
        let raw = r#"
apiVersion: agentflow.dev/v1
kind: TaskWorkflow
flowType: work
metadata:
  name: build-agent.issue-loop
  version: v1
  title: Build Agent Issue Loop
spec:
  initialState: backlog
  terminalStates:
    - done
  states:
    backlog:
      label: 待处理
      phase: future
      role: work-agent
      skillPack: execution-skills
    todo:
      label: 准备开工
      phase: current
      role: work-agent
      skillPack: execution-skills
    done:
      label: 已完成
      phase: past
      role: system
  transitions:
    - id: schedule
      from: backlog
      to: todo
      on: issue.scheduled
      guards:
        - issue.contract.complete
      actions:
        - task.todo.write
    - id: complete
      from: todo
      to: done
      on: issue.completed
      handoff:
        fromRole: work-agent
        toRole: system
        mode: ownership-transfer
        payloadRef: mergeProofRef
        expectedState: done
      actions:
        - event.emit.task_completed
"#;
        let workflow = parse_workflow_yaml(raw).unwrap();
        validate_workflow(&workflow).unwrap();
        workflow
    }

    fn context() -> RuntimeContext {
        RuntimeContext::issue(
            "AF-TASK-001",
            EventActor {
                role: "task-loop".to_string(),
                kind: "system".to_string(),
            },
        )
    }

    #[test]
    fn applies_transition_and_writes_task_event() {
        let dir = tempdir().unwrap();
        let guards = StaticGuardRegistry::all_pass(["issue.contract.complete"]);
        let actions = StaticActionRegistry::all_complete(["task.todo.write"]);

        let result = apply_workflow_event(
            dir.path(),
            &workflow(),
            "backlog",
            "issue.scheduled",
            context(),
            &guards,
            &actions,
        )
        .unwrap();
        let events = load_task_events(dir.path()).unwrap();

        assert!(result.applied);
        assert_eq!(
            result
                .current_binding
                .as_ref()
                .map(|binding| binding.role.as_str()),
            Some("work-agent")
        );
        assert_eq!(
            result
                .next_binding
                .as_ref()
                .map(|binding| binding.role.as_str()),
            Some("work-agent")
        );
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "issue.scheduled");
        assert_eq!(events[0].state.as_ref().unwrap().from_state, "backlog");
        assert_eq!(events[0].state.as_ref().unwrap().to_state, "todo");
        assert_eq!(events[0].payload["transitionId"], "schedule");
    }

    #[test]
    fn guard_failure_does_not_write_transition_event() {
        let dir = tempdir().unwrap();
        let guards = StaticGuardRegistry::default().with_outcome(GuardOutcome::failed(
            "issue.contract.complete",
            "missing field",
        ));
        let actions = StaticActionRegistry::all_complete(["task.todo.write"]);

        let result = apply_workflow_event(
            dir.path(),
            &workflow(),
            "backlog",
            "issue.scheduled",
            context(),
            &guards,
            &actions,
        )
        .unwrap();

        assert!(!result.applied);
        assert_eq!(
            result
                .current_binding
                .as_ref()
                .map(|binding| binding.role.as_str()),
            Some("work-agent")
        );
        assert_eq!(result.blocked_reason.as_deref(), Some("missing field"));
        assert!(load_task_events(dir.path()).unwrap().is_empty());
    }

    #[test]
    fn missing_transition_is_error() {
        let err = find_transition(&workflow(), "done", "issue.scheduled")
            .unwrap_err()
            .to_string();

        assert!(err.contains("no transition"));
    }

    #[test]
    fn resolves_handoff_on_role_transfer_transition() {
        let workflow = workflow();
        let transition = find_transition(&workflow, "todo", "issue.completed").unwrap();
        let handoff = resolve_transition_handoff(&workflow, transition)
            .unwrap()
            .unwrap();

        assert_eq!(handoff.mode.as_str(), "ownership-transfer");
        assert_eq!(handoff.from_role.as_str(), "work-agent");
        assert_eq!(handoff.to_role.as_str(), "system");
    }
}
