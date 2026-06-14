use crate::{
    model::{
        ActionDefinition, GuardDefinition, WorkflowDefinition, AGENTFLOW_WORKFLOW_API_VERSION,
        TASK_WORKFLOW_KIND,
    },
    registry::WorkflowRegistry,
};
use anyhow::Result;
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowValidationReport {
    pub workflow_ref: String,
    pub state_count: usize,
    pub transition_count: usize,
    pub terminal_states: Vec<String>,
}

pub fn validate_workflow(workflow: &WorkflowDefinition) -> Result<WorkflowValidationReport> {
    validate_shape(workflow)?;
    Ok(report(workflow))
}

pub fn validate_workflow_with_registry(
    workflow: &WorkflowDefinition,
    registry: &WorkflowRegistry,
) -> Result<WorkflowValidationReport> {
    validate_shape(workflow)?;
    for transition in &workflow.spec.transitions {
        for guard in &transition.guards {
            let name = validate_registry_name("guard", guard.name())?;
            if !registry.has_guard(name) {
                anyhow::bail!(
                    "transition {} references unregistered guard {name}",
                    transition.id
                );
            }
        }
        for action in &transition.actions {
            let name = validate_registry_name("action", action.name())?;
            if !registry.has_action(name) {
                anyhow::bail!(
                    "transition {} references unregistered action {name}",
                    transition.id
                );
            }
        }
    }
    Ok(report(workflow))
}

fn validate_shape(workflow: &WorkflowDefinition) -> Result<()> {
    if workflow.api_version != AGENTFLOW_WORKFLOW_API_VERSION {
        anyhow::bail!(
            "workflow apiVersion must be {}, found {}",
            AGENTFLOW_WORKFLOW_API_VERSION,
            workflow.api_version
        );
    }
    if workflow.kind != TASK_WORKFLOW_KIND {
        anyhow::bail!(
            "workflow kind must be {}, found {}",
            TASK_WORKFLOW_KIND,
            workflow.kind
        );
    }
    validate_required("metadata.name", &workflow.metadata.name)?;
    validate_required("metadata.version", &workflow.metadata.version)?;
    validate_required("metadata.title", &workflow.metadata.title)?;
    validate_required("spec.initialState", &workflow.spec.initial_state)?;
    if workflow.spec.states.is_empty() {
        anyhow::bail!("workflow must define at least one state");
    }
    if workflow.spec.transitions.is_empty() {
        anyhow::bail!("workflow must define at least one transition");
    }
    if workflow.spec.terminal_states.is_empty() {
        anyhow::bail!("workflow must define terminal states");
    }
    let state_ids = workflow
        .spec
        .states
        .keys()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    if !state_ids.contains(workflow.spec.initial_state.as_str()) {
        anyhow::bail!(
            "initialState {} is not defined in states",
            workflow.spec.initial_state
        );
    }
    for (state_id, state) in &workflow.spec.states {
        validate_registry_name("state", state_id)?;
        validate_required("state.label", &state.label)?;
    }
    for terminal in &workflow.spec.terminal_states {
        validate_registry_name("terminalState", terminal)?;
        if !state_ids.contains(terminal.as_str()) {
            anyhow::bail!("terminal state {terminal} is not defined in states");
        }
    }
    for transition in &workflow.spec.transitions {
        validate_registry_name("transition", &transition.id)?;
        validate_required("transition.on", &transition.on)?;
        validate_registry_name("event", &transition.on)?;
        validate_registry_name("transition.to", &transition.to)?;
        if !state_ids.contains(transition.to.as_str()) {
            anyhow::bail!(
                "transition {} target state {} is not defined",
                transition.id,
                transition.to
            );
        }
        if transition.from_states.is_empty() {
            anyhow::bail!("transition {} must define from state", transition.id);
        }
        for from in &transition.from_states {
            validate_registry_name("transition.from", from)?;
            if !state_ids.contains(from.as_str()) {
                anyhow::bail!(
                    "transition {} source state {} is not defined",
                    transition.id,
                    from
                );
            }
        }
        for guard in &transition.guards {
            validate_guard(guard)?;
        }
        for action in &transition.actions {
            validate_action(action)?;
        }
    }
    Ok(())
}

fn validate_guard(guard: &GuardDefinition) -> Result<()> {
    validate_registry_name("guard", guard.name())?;
    Ok(())
}

fn validate_action(action: &ActionDefinition) -> Result<()> {
    validate_registry_name("action", action.name())?;
    Ok(())
}

fn validate_required(field: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        anyhow::bail!("{field} is required");
    }
    Ok(())
}

fn validate_registry_name<'a>(kind: &str, value: &'a str) -> Result<&'a str> {
    let value = value.trim();
    if value.is_empty() {
        anyhow::bail!("{kind} name is required");
    }
    if !value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-'))
    {
        anyhow::bail!(
            "{kind} name must be a registry reference, found {value}. Workflow YAML cannot embed shell or free-form commands"
        );
    }
    Ok(value)
}

fn report(workflow: &WorkflowDefinition) -> WorkflowValidationReport {
    WorkflowValidationReport {
        workflow_ref: workflow.workflow_ref(),
        state_count: workflow.spec.states.len(),
        transition_count: workflow.spec.transitions.len(),
        terminal_states: workflow.spec.terminal_states.clone(),
    }
}

#[cfg(test)]
mod tests {
    use crate::{parse_workflow_yaml, WorkflowRegistry};

    use super::*;

    fn sample_workflow() -> &'static str {
        r#"
apiVersion: agentflow.dev/v1
kind: TaskWorkflow
metadata:
  name: build-agent.issue-loop
  version: v1
  title: Build Agent Issue Loop
spec:
  initialState: backlog
  terminalStates:
    - done
    - cancel
  states:
    backlog:
      label: 待处理
      phase: future
    todo:
      label: 准备开工
      phase: current
    in_progress:
      label: 正在做
      phase: current
    in_review:
      label: 正在评审
      phase: current
    done:
      label: 已完成
      phase: past
    blocked:
      label: 已阻断
      phase: current
    cancel:
      label: 已取消
      phase: past
  transitions:
    - id: schedule
      from: backlog
      to: todo
      on: issue.scheduled
      guards:
        - issue.contract.complete
        - dependencies.done
        - context_pack.ready
        - workspace.clean
      actions:
        - task.context.prepare
        - task.todo.write
        - event.emit.issue_scheduled
    - id: start
      from: todo
      to: in_progress
      on: build_agent.started
      guards:
        - run.created
        - runtime.preflight.passed
        - lease.active
      actions:
        - run.plan.write
        - run.checkpoint.create
        - event.emit.task_started
    - id: request_review
      from: in_progress
      to: in_review
      on: verification.passed
      guards:
        - sandbox.validation.passed
        - pr_or_mr.created
      actions:
        - public_record.pr_body_draft
        - review.prepare
        - event.emit.review_requested
    - id: complete
      from: in_review
      to: done
      on: pr_or_mr.merged
      guards:
        - merge.proof.present
        - public_record.pr_body_ready
      actions:
        - build_agent.complete
        - public_record.pr_body_finalize
        - event.emit.task_completed
    - id: block
      from:
        - backlog
        - todo
        - in_progress
        - in_review
      to: blocked
      on: task.blocked
      actions:
        - blocker.write
        - event.emit.task_blocked
    - id: cancel
      from:
        - backlog
        - todo
        - in_progress
        - in_review
        - blocked
      to: cancel
      on: task.cancelled
      actions:
        - task.cancel.write
        - event.emit.task_cancelled
"#
    }

    #[test]
    fn validates_default_build_agent_issue_loop_yaml() {
        let workflow = parse_workflow_yaml(sample_workflow()).unwrap();
        let report =
            validate_workflow_with_registry(&workflow, &WorkflowRegistry::build_agent_issue_loop())
                .unwrap();

        assert_eq!(report.workflow_ref, "build-agent.issue-loop@v1");
        assert_eq!(report.state_count, 7);
        assert_eq!(report.transition_count, 6);
        assert_eq!(report.terminal_states, vec!["done", "cancel"]);
    }

    #[test]
    fn rejects_unknown_transition_state() {
        let raw = sample_workflow().replace("to: todo", "to: missing");
        let workflow = parse_workflow_yaml(&raw).unwrap();
        let err = validate_workflow(&workflow).unwrap_err().to_string();

        assert!(err.contains("missing"));
        assert!(err.contains("not defined"));
    }

    #[test]
    fn rejects_shell_like_action_names() {
        let raw = sample_workflow().replace("task.todo.write", "npm test");
        let workflow = parse_workflow_yaml(&raw).unwrap();
        let err = validate_workflow(&workflow).unwrap_err().to_string();

        assert!(err.contains("registry reference"));
    }

    #[test]
    fn registry_validation_rejects_unregistered_guard() {
        let raw = sample_workflow().replace("dependencies.done", "custom.guard");
        let workflow = parse_workflow_yaml(&raw).unwrap();
        let err =
            validate_workflow_with_registry(&workflow, &WorkflowRegistry::build_agent_issue_loop())
                .unwrap_err()
                .to_string();

        assert!(err.contains("unregistered guard"));
    }
}
