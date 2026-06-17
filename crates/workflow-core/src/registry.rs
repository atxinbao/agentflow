use crate::{
    parse_workflow_yaml, validate_workflow_with_registry, WorkflowDefinition, WorkflowFlowType,
    WorkflowStatePhase,
};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WorkflowRegistry {
    guards: BTreeSet<String>,
    actions: BTreeSet<String>,
}

impl WorkflowRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build_agent_issue_loop() -> Self {
        Self::new()
            .with_guard("issue.contract.complete")
            .with_guard("dependencies.done")
            .with_guard("context_pack.ready")
            .with_guard("workspace.clean")
            .with_guard("run.created")
            .with_guard("runtime.preflight.passed")
            .with_guard("lease.active")
            .with_guard("sandbox.validation.passed")
            .with_guard("pr_or_mr.created")
            .with_guard("merge.proof.present")
            .with_guard("public_record.pr_body_ready")
            .with_action("task.context.prepare")
            .with_action("task.todo.write")
            .with_action("run.plan.write")
            .with_action("run.checkpoint.create")
            .with_action("public_record.pr_body_draft")
            .with_action("review.prepare")
            .with_action("build_agent.complete")
            .with_action("public_record.pr_body_finalize")
            .with_action("blocker.write")
            .with_action("task.cancel.write")
            .with_action("event.emit.issue_scheduled")
            .with_action("event.emit.task_started")
            .with_action("event.emit.review_requested")
            .with_action("event.emit.task_completed")
            .with_action("event.emit.task_blocked")
            .with_action("event.emit.task_cancelled")
    }

    pub fn with_guard(mut self, name: impl Into<String>) -> Self {
        self.guards.insert(name.into());
        self
    }

    pub fn with_action(mut self, name: impl Into<String>) -> Self {
        self.actions.insert(name.into());
        self
    }

    pub fn has_guard(&self, name: &str) -> bool {
        self.guards.contains(name)
    }

    pub fn has_action(&self, name: &str) -> bool {
        self.actions.contains(name)
    }
}

pub const WORK_STATE_BACKLOG: &str = "backlog";
pub const WORK_STATE_TODO: &str = "todo";
pub const WORK_STATE_IN_PROGRESS: &str = "in_progress";
pub const WORK_STATE_IN_REVIEW: &str = "in_review";
pub const WORK_STATE_DONE: &str = "done";
pub const WORK_STATE_BLOCKED: &str = "blocked";
pub const WORK_STATE_CANCEL: &str = "cancel";

pub const AUDIT_STATE_PENDING: &str = "pending";
pub const AUDIT_STATE_READY: &str = "ready";
pub const AUDIT_STATE_IN_PROGRESS: &str = "in_progress";
pub const AUDIT_STATE_PASSED: &str = "passed";
pub const AUDIT_STATE_NEEDS_REPAIR: &str = "needs_repair";
pub const AUDIT_STATE_BLOCKED: &str = "blocked";
pub const AUDIT_STATE_CANCEL: &str = "cancel";

pub const DELIVERY_STATE_PENDING: &str = "pending";
pub const DELIVERY_STATE_READY: &str = "ready";
pub const DELIVERY_STATE_IN_PROGRESS: &str = "in_progress";
pub const DELIVERY_STATE_PUBLISHED: &str = "published";
pub const DELIVERY_STATE_RETURNED: &str = "returned";
pub const DELIVERY_STATE_BLOCKED: &str = "blocked";
pub const DELIVERY_STATE_CANCEL: &str = "cancel";

pub fn canonical_workflow(flow_type: WorkflowFlowType) -> WorkflowDefinition {
    let (raw, registry) = match flow_type {
        WorkflowFlowType::Project => (PROJECT_WORKFLOW_YAML, WorkflowRegistry::project_runtime()),
        WorkflowFlowType::Work => (
            WORK_WORKFLOW_YAML,
            WorkflowRegistry::build_agent_issue_loop(),
        ),
        WorkflowFlowType::Audit => (AUDIT_WORKFLOW_YAML, WorkflowRegistry::audit_runtime()),
        WorkflowFlowType::Delivery => {
            (DELIVERY_WORKFLOW_YAML, WorkflowRegistry::delivery_runtime())
        }
    };
    let workflow = parse_workflow_yaml(raw).expect("built-in workflow yaml must parse");
    validate_workflow_with_registry(&workflow, &registry)
        .expect("built-in workflow yaml must validate");
    workflow
}

pub fn work_state_phase(state: &str) -> Option<WorkflowStatePhase> {
    canonical_workflow(WorkflowFlowType::Work)
        .spec
        .states
        .get(state)
        .map(|definition| definition.phase)
}

pub fn work_state_is_terminal(state: &str) -> bool {
    canonical_workflow(WorkflowFlowType::Work)
        .spec
        .terminal_states
        .iter()
        .any(|terminal| terminal == state)
}

pub fn work_state_is_done(state: &str) -> bool {
    state == WORK_STATE_DONE
}

pub fn work_state_is_cancel(state: &str) -> bool {
    state == WORK_STATE_CANCEL
}

pub fn work_state_is_todo(state: &str) -> bool {
    state == WORK_STATE_TODO
}

pub fn work_state_is_in_progress(state: &str) -> bool {
    state == WORK_STATE_IN_PROGRESS
}

pub fn work_state_is_in_review(state: &str) -> bool {
    state == WORK_STATE_IN_REVIEW
}

pub fn work_state_is_blocked(state: &str) -> bool {
    state == WORK_STATE_BLOCKED
}

pub fn work_state_is_active(state: &str) -> bool {
    work_state_is_todo(state) || work_state_is_in_progress(state) || work_state_is_in_review(state)
}

pub fn work_state_is_ready_for_execution(state: &str) -> bool {
    work_state_is_todo(state)
}

impl WorkflowRegistry {
    pub fn project_runtime() -> Self {
        Self::new()
            .with_guard("goal.contract.ready")
            .with_guard("plan.contract.ready")
            .with_guard("project.confirmed")
            .with_guard("work.completed")
            .with_guard("audit.completed")
            .with_guard("delivery.completed")
            .with_guard("goal.recheck.requested")
            .with_action("project.goal.capture")
            .with_action("project.plan.capture")
            .with_action("project.confirm.write")
            .with_action("project.loop.open")
            .with_action("project.audit.open")
            .with_action("project.delivery.open")
            .with_action("project.goal_recheck.open")
            .with_action("project.accept.write")
    }

    pub fn audit_runtime() -> Self {
        Self::new()
            .with_guard("audit.request.present")
            .with_guard("audit.scope.ready")
            .with_guard("audit.findings.recorded")
            .with_guard("repair.required")
            .with_action("audit.request.write")
            .with_action("audit.run.start")
            .with_action("audit.result.pass")
            .with_action("audit.result.repair")
            .with_action("audit.cancel.write")
    }

    pub fn delivery_runtime() -> Self {
        Self::new()
            .with_guard("delivery.input.ready")
            .with_guard("delivery.public_record.ready")
            .with_guard("delivery.publish.confirmed")
            .with_action("delivery.ready.write")
            .with_action("delivery.summary.write")
            .with_action("delivery.publish.write")
            .with_action("delivery.return.write")
            .with_action("delivery.cancel.write")
    }
}

const WORK_WORKFLOW_YAML: &str = r#"
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
    - cancel
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
    in_progress:
      label: 正在做
      phase: current
      role: work-agent
      skillPack: execution-skills
    in_review:
      label: 正在评审
      phase: current
      role: work-agent
      skillPack: execution-skills
    done:
      label: 已完成
      phase: past
      role: system
    blocked:
      label: 已阻断
      phase: current
      role: work-agent
      skillPack: execution-skills
    cancel:
      label: 已取消
      phase: past
      role: system
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
      handoff:
        fromRole: work-agent
        toRole: system
        mode: ownership-transfer
        payloadRef: mergeProofRef
        expectedState: done
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
      handoff:
        fromRole: work-agent
        toRole: system
        mode: ownership-transfer
        payloadRef: cancellationRef
        expectedState: cancel
      actions:
        - task.cancel.write
        - event.emit.task_cancelled
"#;

const PROJECT_WORKFLOW_YAML: &str = r#"
apiVersion: agentflow.dev/v1
kind: TaskWorkflow
flowType: project
metadata:
  name: project.runtime-loop
  version: v1
  title: Project Runtime Loop
spec:
  initialState: intake
  terminalStates:
    - accepted
    - blocked
  states:
    intake:
      label: 需求进入
      phase: current
      role: goal-agent
      skillPack: brain-skills
    goal_draft:
      label: 目标草案
      phase: current
      role: goal-agent
      skillPack: brain-skills
    plan_draft:
      label: 计划草案
      phase: current
      role: spec-agent
      skillPack: contract-skills
    confirmed:
      label: 已确认
      phase: current
      role: system
    working:
      label: 执行中
      phase: current
      role: system
    auditing:
      label: 审计中
      phase: current
      role: audit-agent
      skillPack: judgment-skills
    delivering:
      label: 交付中
      phase: current
      role: delivery-agent
      skillPack: delivery-skills
    goal_recheck:
      label: 目标回看
      phase: current
      role: goal-agent
      skillPack: brain-skills
    paused:
      label: 已暂停
      phase: current
      role: system
    accepted:
      label: 已验收
      phase: past
      role: system
    blocked:
      label: 已阻断
      phase: current
      role: system
  transitions:
    - id: intake_to_goal
      from: intake
      to: goal_draft
      on: project.intake.accepted
      guards:
        - goal.contract.ready
      actions:
        - project.goal.capture
    - id: goal_to_plan
      from: goal_draft
      to: plan_draft
      on: goal.draft.confirmed
      handoff:
        fromRole: goal-agent
        toRole: spec-agent
        mode: ownership-transfer
        payloadRef: goalRef
        expectedState: plan_draft
      guards:
        - plan.contract.ready
      actions:
        - project.plan.capture
    - id: plan_to_confirmed
      from: plan_draft
      to: confirmed
      on: plan.draft.confirmed
      handoff:
        fromRole: spec-agent
        toRole: system
        mode: ownership-transfer
        payloadRef: projectPlanRef
        expectedState: confirmed
      guards:
        - project.confirmed
      actions:
        - project.confirm.write
    - id: confirmed_to_working
      from: confirmed
      to: working
      on: project.loop.started
      guards:
        - work.completed
      actions:
        - project.loop.open
    - id: working_to_auditing
      from: working
      to: auditing
      on: project.audit.requested
      handoff:
        fromRole: system
        toRole: audit-agent
        mode: ownership-transfer
        payloadRef: auditEntryRef
        expectedState: auditing
      guards:
        - audit.completed
      actions:
        - project.audit.open
    - id: auditing_to_delivering
      from: auditing
      to: delivering
      on: audit.passed
      handoff:
        fromRole: audit-agent
        toRole: delivery-agent
        mode: ownership-transfer
        payloadRef: auditResultRef
        expectedState: delivering
      guards:
        - delivery.completed
      actions:
        - project.delivery.open
    - id: delivering_to_recheck
      from: delivering
      to: goal_recheck
      on: delivery.completed
      handoff:
        fromRole: delivery-agent
        toRole: goal-agent
        mode: ownership-transfer
        payloadRef: deliverySummaryRef
        expectedState: goal_recheck
      guards:
        - goal.recheck.requested
      actions:
        - project.goal_recheck.open
    - id: recheck_to_accepted
      from: goal_recheck
      to: accepted
      on: project.accepted
      handoff:
        fromRole: goal-agent
        toRole: system
        mode: ownership-transfer
        payloadRef: completionDecisionRef
        expectedState: accepted
      actions:
        - project.accept.write
"#;

const AUDIT_WORKFLOW_YAML: &str = r#"
apiVersion: agentflow.dev/v1
kind: TaskWorkflow
flowType: audit
metadata:
  name: audit.issue-loop
  version: v1
  title: Audit Issue Loop
spec:
  initialState: pending
  terminalStates:
    - passed
    - cancel
  states:
    pending:
      label: 待请求
      phase: future
      role: audit-agent
      skillPack: judgment-skills
    ready:
      label: 可审计
      phase: current
      role: audit-agent
      skillPack: judgment-skills
    in_progress:
      label: 审计中
      phase: current
      role: audit-agent
      skillPack: judgment-skills
    passed:
      label: 审计通过
      phase: past
      role: system
    needs_repair:
      label: 需要修复
      phase: current
      role: audit-agent
      skillPack: judgment-skills
    blocked:
      label: 已阻断
      phase: current
      role: audit-agent
      skillPack: judgment-skills
    cancel:
      label: 已取消
      phase: past
      role: system
  transitions:
    - id: request
      from: pending
      to: ready
      on: audit.requested
      guards:
        - audit.request.present
      actions:
        - audit.request.write
    - id: start
      from: ready
      to: in_progress
      on: audit.started
      guards:
        - audit.scope.ready
      actions:
        - audit.run.start
    - id: pass
      from: in_progress
      to: passed
      on: audit.passed
      handoff:
        fromRole: audit-agent
        toRole: system
        mode: ownership-transfer
        payloadRef: auditResultRef
        expectedState: passed
      guards:
        - audit.findings.recorded
      actions:
        - audit.result.pass
    - id: repair
      from: in_progress
      to: needs_repair
      on: audit.needs_repair
      guards:
        - repair.required
      actions:
        - audit.result.repair
    - id: cancel
      from:
        - pending
        - ready
        - in_progress
        - needs_repair
        - blocked
      to: cancel
      on: audit.cancelled
      handoff:
        fromRole: audit-agent
        toRole: system
        mode: ownership-transfer
        payloadRef: auditCancellationRef
        expectedState: cancel
      actions:
        - audit.cancel.write
"#;

const DELIVERY_WORKFLOW_YAML: &str = r#"
apiVersion: agentflow.dev/v1
kind: TaskWorkflow
flowType: delivery
metadata:
  name: delivery.issue-loop
  version: v1
  title: Delivery Issue Loop
spec:
  initialState: pending
  terminalStates:
    - published
    - cancel
  states:
    pending:
      label: 待交付
      phase: future
      role: delivery-agent
      skillPack: delivery-skills
    ready:
      label: 可交付
      phase: current
      role: delivery-agent
      skillPack: delivery-skills
    in_progress:
      label: 交付中
      phase: current
      role: delivery-agent
      skillPack: delivery-skills
    published:
      label: 已发布
      phase: past
      role: system
    returned:
      label: 已退回
      phase: current
      role: delivery-agent
      skillPack: delivery-skills
    blocked:
      label: 已阻断
      phase: current
      role: delivery-agent
      skillPack: delivery-skills
    cancel:
      label: 已取消
      phase: past
      role: system
  transitions:
    - id: ready
      from: pending
      to: ready
      on: delivery.ready
      guards:
        - delivery.input.ready
      actions:
        - delivery.ready.write
    - id: start
      from: ready
      to: in_progress
      on: delivery.started
      guards:
        - delivery.public_record.ready
      actions:
        - delivery.summary.write
    - id: publish
      from: in_progress
      to: published
      on: delivery.published
      handoff:
        fromRole: delivery-agent
        toRole: system
        mode: ownership-transfer
        payloadRef: publicDeliveryRef
        expectedState: published
      guards:
        - delivery.publish.confirmed
      actions:
        - delivery.publish.write
    - id: return
      from: in_progress
      to: returned
      on: delivery.returned
      actions:
        - delivery.return.write
    - id: cancel
      from:
        - pending
        - ready
        - in_progress
        - returned
        - blocked
      to: cancel
      on: delivery.cancelled
      handoff:
        fromRole: delivery-agent
        toRole: system
        mode: ownership-transfer
        payloadRef: deliveryCancellationRef
        expectedState: cancel
      actions:
        - delivery.cancel.write
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn built_in_project_audit_and_delivery_workflows_validate() {
        let project = canonical_workflow(WorkflowFlowType::Project);
        let audit = canonical_workflow(WorkflowFlowType::Audit);
        let delivery = canonical_workflow(WorkflowFlowType::Delivery);

        assert_eq!(project.flow_type.as_str(), "project");
        assert_eq!(audit.flow_type.as_str(), "audit");
        assert_eq!(delivery.flow_type.as_str(), "delivery");
    }

    #[test]
    fn work_state_helpers_follow_canonical_workflow() {
        assert_eq!(
            work_state_phase(WORK_STATE_BACKLOG),
            Some(WorkflowStatePhase::Future)
        );
        assert!(work_state_is_ready_for_execution(WORK_STATE_TODO));
        assert!(work_state_is_active(WORK_STATE_IN_PROGRESS));
        assert!(work_state_is_terminal(WORK_STATE_DONE));
        assert!(work_state_is_cancel(WORK_STATE_CANCEL));
    }
}
