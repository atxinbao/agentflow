use crate::{
    assertions::assert_path_exists,
    fixture::{create_fixture_project, WorkflowFixture},
};
use agentflow_audit::{AuditScope, AuditScopeRef, HumanAuditReport, HumanAuditRequestDraft};
use agentflow_event_store::{
    append_task_event_once, EventActor, EventStateTransition, TaskEventDraft,
};
use agentflow_spec::{SpecIssue, SpecIssueDraft, SpecIssueStatus};
use agentflow_state::{
    StateStatusSnapshot, StateWorkspaceStatus, WorkflowAuditStatus, WorkflowStage,
};
use agentflow_task_artifacts::TaskCommandInput;
use agentflow_workflow_core::{WorkflowAgentRole, WorkflowFlowType};
use anyhow::Result;
use serde_json::json;
use std::{fs, path::Path};

pub struct ExecutionCompletedFixture {
    pub fixture: WorkflowFixture,
    pub run_id: String,
    pub execution_state: StateStatusSnapshot,
}

impl ExecutionCompletedFixture {
    pub fn request_human_audit(&self) -> Result<HumanAuditReport> {
        request_human_audit_for_run(self.fixture.root(), &self.run_id)
    }
}

pub fn prepare_execution_completed_fixture() -> Result<ExecutionCompletedFixture> {
    let fixture = create_fixture_project()?;
    let root = fixture.root();
    prepare_all_layers(root)?;
    let issue = write_issue(root, "iss-001", SpecIssueStatus::Todo, Vec::new())?;
    let run_id = "run-001".to_string();
    agentflow_task_artifacts::create_task_run(
        root,
        &issue.issue_id,
        &run_id,
        &issue.workflow_ref,
        None,
    )?;
    append_issue_event(
        root,
        &issue,
        "agent.launch.requested",
        "in_progress",
        &run_id,
    )?;
    agentflow_task_artifacts::write_task_command_record(
        root,
        &issue.issue_id,
        &run_id,
        TaskCommandInput {
            label: "printf ok".to_string(),
            program: "printf".to_string(),
            args: vec!["ok".to_string()],
            exit_code: Some(0),
            stdout: "ok".to_string(),
            stderr: String::new(),
        },
    )?;
    agentflow_task_artifacts::write_task_validation(root, &issue.issue_id, &run_id)?;
    agentflow_task_artifacts::write_task_evidence(
        root,
        &issue.issue_id,
        &run_id,
        "Fixture validation evidence.",
    )?;
    append_issue_event(root, &issue, "issue.review.requested", "in_review", &run_id)?;
    agentflow_projection::rebuild_projections(root)?;

    let execution_state = agentflow_state::refresh_state(root)?;
    anyhow::ensure!(
        execution_state.current_stage == WorkflowStage::ExecuteCompleted,
        "expected execute-completed, got {:?}",
        execution_state.current_stage
    );
    anyhow::ensure!(
        execution_state.audit_status == WorkflowAuditStatus::NotRequested,
        "execute completed state must not register audit request automatically"
    );

    fixture.assert_user_files_unchanged()?;
    Ok(ExecutionCompletedFixture {
        fixture,
        run_id,
        execution_state,
    })
}

pub fn create_high_risk_blocker_fixture() -> Result<WorkflowFixture> {
    let fixture = create_fixture_project()?;
    let root = fixture.root();
    prepare_all_layers(root)?;
    let dependency = write_issue(root, "iss-001", SpecIssueStatus::Todo, Vec::new())?;
    let blocked = write_issue(
        root,
        "iss-002",
        SpecIssueStatus::Todo,
        vec![dependency.issue_id.clone()],
    )?;
    append_issue_event(root, &blocked, "issue.scheduled", "todo", "run-002")?;
    agentflow_projection::rebuild_projections(root)?;
    let state = agentflow_state::refresh_state(root)?;
    anyhow::ensure!(
        state
            .blockers
            .iter()
            .any(|blocker| blocker.reason.contains("前置依赖")),
        "dependency blocker was not surfaced in state"
    );
    fixture.assert_user_files_unchanged()?;
    Ok(fixture)
}

pub fn create_stale_lease_fixture() -> Result<WorkflowFixture> {
    let fixture = create_fixture_project()?;
    let root = fixture.root();
    prepare_all_layers(root)?;
    agentflow_state::refresh_state(root)?;
    fixture.assert_user_files_unchanged()?;
    Ok(fixture)
}

pub fn human_audit_draft(run_id: &str) -> HumanAuditRequestDraft {
    HumanAuditRequestDraft {
        reason: "Human requested end-to-end acceptance audit.".to_string(),
        scope: AuditScope {
            description: "Verify accepted delivery evidence chain.".to_string(),
            refs: vec![
                AuditScopeRef {
                    kind: "issue".to_string(),
                    id: "iss-001".to_string(),
                    path: ".agentflow/spec/issues/iss-001.json".to_string(),
                },
                AuditScopeRef {
                    kind: "task-run".to_string(),
                    id: run_id.to_string(),
                    path: format!(".agentflow/tasks/iss-001/runs/{run_id}/run.json"),
                },
                AuditScopeRef {
                    kind: "evidence".to_string(),
                    id: run_id.to_string(),
                    path: ".agentflow/tasks/iss-001/evidence/evidence.json".to_string(),
                },
                AuditScopeRef {
                    kind: "public-delivery".to_string(),
                    id: "CHANGELOG.md".to_string(),
                    path: "CHANGELOG.md".to_string(),
                },
            ],
        },
    }
}

fn request_human_audit_for_run(root: &Path, run_id: &str) -> Result<HumanAuditReport> {
    let report = agentflow_audit::request_human_audit(root, human_audit_draft(run_id))?;
    for artifact in [
        "audit-request.json",
        "audit.json",
        "audit-report.md",
        "findings.json",
        "checklist.md",
        "evidence-map.json",
        "traceability.json",
    ] {
        assert_path_exists(
            root,
            &format!(".agentflow/audit/{}/{}", report.audit.audit_id, artifact),
        )?;
    }
    Ok(report)
}

fn prepare_all_layers(root: &Path) -> Result<()> {
    let define = agentflow_agent_manual::prepare_agent_working_manual(root)?;
    anyhow::ensure!(define.ready, "define layer is not ready");
    let panel =
        agentflow_panel::prepare_project_panel(root, agentflow_panel::PanelPrepareMode::Blocking)?;
    anyhow::ensure!(
        matches!(panel.status, agentflow_panel::PanelStatus::Ready),
        "panel layer is not ready"
    );
    agentflow_spec::prepare_spec_workspace(root)?;
    agentflow_projection::prepare_projection_workspace(root)?;
    agentflow_event_store::prepare_event_store(root)?;
    agentflow_audit::prepare_audit_workspace(root)?;
    let state = agentflow_state::prepare_state_workspace(root)?;
    anyhow::ensure!(
        state.status == StateWorkspaceStatus::Ready,
        "state layer is not ready"
    );

    for relative_path in [
        "AGENTS.md",
        ".agentflow/workspace-manifest.json",
        ".agentflow/define/agent/Agentflow.md",
        ".agentflow/define/agent/skills-lock.json",
        ".agentflow/define/spec/SPEC.md",
        ".agentflow/define/tdd/TDD.md",
        ".agentflow/define/release/RELEASE.md",
        ".agentflow/define/audit/AUDIT.md",
        ".agentflow/panel/manifest.json",
        ".agentflow/spec/manifest.json",
        ".agentflow/audit/manifest.json",
        ".agentflow/state/manifest.json",
        ".agentflow/state/gates/workflow.json",
    ] {
        assert_path_exists(root, relative_path)?;
    }
    Ok(())
}

fn write_issue(
    root: &Path,
    issue_id: &str,
    status: SpecIssueStatus,
    blocked_by: Vec<String>,
) -> Result<SpecIssue> {
    let requirement = root.join(format!("docs/requirements/{issue_id}.md"));
    fs::create_dir_all(requirement.parent().unwrap())?;
    fs::write(
        &requirement,
        format!("# {issue_id}\n\nFixture requirement.\n"),
    )?;
    let mut draft = SpecIssueDraft::new(issue_id);
    draft.title = Some("End-to-end workflow acceptance issue".to_string());
    draft.summary =
        Some("Validate AgentFlow end-to-end workflow without modifying user source.".to_string());
    draft.allowed_paths = vec!["src/lib.rs".to_string()];
    draft.validation_commands = vec!["printf ok".to_string()];
    draft.blocked_by = blocked_by;
    let mut issue = agentflow_spec::issue_from_requirement(root, &requirement, draft)?;
    issue.status = status;
    agentflow_spec::write_spec_issue(root, &issue)?;
    Ok(issue)
}

fn append_issue_event(
    root: &Path,
    issue: &SpecIssue,
    event_type: &str,
    state: &str,
    run_id: &str,
) -> Result<()> {
    append_task_event_once(
        root,
        TaskEventDraft {
            flow_type: WorkflowFlowType::Work,
            aggregate_type: "issue".to_string(),
            aggregate_id: issue.issue_id.clone(),
            project_id: issue.project_id.clone(),
            issue_id: Some(issue.issue_id.clone()),
            run_id: Some(run_id.to_string()),
            event_type: event_type.to_string(),
            authority_role: Some(match state {
                "done" | "cancel" => WorkflowAgentRole::System,
                _ => WorkflowAgentRole::WorkAgent,
            }),
            actor: EventActor {
                role: "acceptance".to_string(),
                kind: "system".to_string(),
            },
            state: Some(EventStateTransition {
                from_state: issue.status.as_str().to_string(),
                to_state: state.to_string(),
            }),
            correlation_id: Some(format!("corr-{}", issue.issue_id)),
            causation_id: None,
            payload: json!({"runId": run_id}),
            artifact_refs: vec![issue.system.path.clone()],
            idempotency_key: Some(format!("{event_type}:{}", issue.issue_id)),
        },
    )?;
    Ok(())
}
