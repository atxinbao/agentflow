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
