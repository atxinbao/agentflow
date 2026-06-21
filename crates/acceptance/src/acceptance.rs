use crate::{
    assertions::assert_path_exists,
    fixture::{create_fixture_project, WorkflowFixture},
};
use agentflow_audit::{AuditScope, AuditScopeRef, HumanAuditReport, HumanAuditRequestDraft};
use agentflow_event_store::{
    append_task_event_once, EventActor, EventStateTransition, TaskEventDraft,
};
use agentflow_spec::{
    write_spec_project, SpecIssue, SpecIssueDraft, SpecIssueStatus, SpecProjectDraft,
};
use agentflow_state::{
    StateStatusSnapshot, StateWorkspaceStatus, WorkflowAuditStatus, WorkflowStage,
};
use agentflow_task_artifacts::TaskCommandInput;
use agentflow_workflow_core::{WorkflowAgentRole, WorkflowFlowType};
use anyhow::Result;
use serde_json::json;
use std::{fs, path::Path, process::Command};

pub struct ExecutionCompletedFixture {
    pub fixture: WorkflowFixture,
    pub run_id: String,
    pub execution_state: StateStatusSnapshot,
}

pub struct ProjectLoopCloseoutFixture {
    pub fixture: WorkflowFixture,
    pub project_id: String,
    pub current_issue_id: String,
    pub next_issue_id: String,
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

pub fn prepare_done_writeback_fixture() -> Result<ExecutionCompletedFixture> {
    let ready = prepare_execution_completed_fixture()?;
    let root = ready.fixture.root();
    let issue = agentflow_spec::read_spec_issue(root, "iss-001")?;
    let run_id = ready.run_id.clone();

    append_task_event_once(
        root,
        TaskEventDraft {
            flow_type: WorkflowFlowType::Work,
            aggregate_type: "issue".to_string(),
            aggregate_id: issue.issue_id.clone(),
            project_id: issue.project_id.clone(),
            issue_id: Some(issue.issue_id.clone()),
            run_id: Some(run_id.clone()),
            event_type: "issue.closeout.proof.recorded".to_string(),
            authority_role: Some(WorkflowAgentRole::WorkAgent),
            actor: EventActor {
                role: "acceptance".to_string(),
                kind: "system".to_string(),
            },
            state: Some(EventStateTransition {
                from_state: "in_review".to_string(),
                to_state: "in_review".to_string(),
            }),
            correlation_id: Some(format!("corr-{}", issue.issue_id)),
            causation_id: None,
            payload: json!({
                "runId": run_id,
                "prUrl": "https://github.com/atxinbao/agentflow/pull/388",
                "mergeCommit": "abcdef1234567890donewriteback",
                "changelogPath": "CHANGELOG.md",
                "releaseNotesUrl": "docs/release-notes/agentflow-v060-012.md"
            }),
            artifact_refs: vec![issue.system.path.clone()],
            idempotency_key: Some(format!("issue.closeout.proof.recorded:{}", issue.issue_id)),
        },
    )?;
    append_task_event_once(
        root,
        TaskEventDraft {
            flow_type: WorkflowFlowType::Work,
            aggregate_type: "issue".to_string(),
            aggregate_id: issue.issue_id.clone(),
            project_id: issue.project_id.clone(),
            issue_id: Some(issue.issue_id.clone()),
            run_id: Some(run_id.clone()),
            event_type: "issue.pr.merged".to_string(),
            authority_role: Some(WorkflowAgentRole::WorkAgent),
            actor: EventActor {
                role: "acceptance".to_string(),
                kind: "system".to_string(),
            },
            state: Some(EventStateTransition {
                from_state: "in_review".to_string(),
                to_state: "in_review".to_string(),
            }),
            correlation_id: Some(format!("corr-{}", issue.issue_id)),
            causation_id: None,
            payload: json!({
                "runId": run_id,
                "prUrl": "https://github.com/atxinbao/agentflow/pull/388",
                "mergeCommit": "abcdef1234567890donewriteback",
                "changelogPath": "CHANGELOG.md",
                "releaseNotesUrl": "docs/release-notes/agentflow-v060-012.md"
            }),
            artifact_refs: vec![issue.system.path.clone()],
            idempotency_key: Some(format!("issue.pr.merged:{}", issue.issue_id)),
        },
    )?;
    append_task_event_once(
        root,
        TaskEventDraft {
            flow_type: WorkflowFlowType::Work,
            aggregate_type: "issue".to_string(),
            aggregate_id: issue.issue_id.clone(),
            project_id: issue.project_id.clone(),
            issue_id: Some(issue.issue_id.clone()),
            run_id: Some(run_id.clone()),
            event_type: "issue.completed".to_string(),
            authority_role: Some(WorkflowAgentRole::System),
            actor: EventActor {
                role: "acceptance".to_string(),
                kind: "system".to_string(),
            },
            state: Some(EventStateTransition {
                from_state: "in_review".to_string(),
                to_state: "done".to_string(),
            }),
            correlation_id: Some(format!("corr-{}", issue.issue_id)),
            causation_id: None,
            payload: json!({
                "issueId": issue.issue_id,
                "runId": run_id,
                "prUrl": "https://github.com/atxinbao/agentflow/pull/388",
                "mergeCommit": "abcdef1234567890donewriteback",
                "changelogPath": "CHANGELOG.md",
                "releaseNotesUrl": "docs/release-notes/agentflow-v060-012.md",
                "evidencePath": ".agentflow/tasks/iss-001/evidence/evidence.json"
            }),
            artifact_refs: vec![issue.system.path.clone()],
            idempotency_key: Some(format!("issue.completed:{}", issue.issue_id)),
        },
    )?;

    let mut completed_issue = issue;
    completed_issue.status = SpecIssueStatus::Done;
    agentflow_spec::write_spec_issue(root, &completed_issue)?;
    agentflow_projection::rebuild_projections(root)?;
    let execution_state = agentflow_state::refresh_state(root)?;

    anyhow::ensure!(
        execution_state.audit_status == WorkflowAuditStatus::NotRequested,
        "done writeback fixture must not create audit request automatically"
    );
    ready.fixture.assert_user_files_unchanged()?;

    Ok(ExecutionCompletedFixture {
        fixture: ready.fixture,
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

pub fn prepare_project_in_review_fixture() -> Result<ProjectLoopCloseoutFixture> {
    prepare_project_closeout_fixture(false)
}

pub fn prepare_project_done_fixture() -> Result<ProjectLoopCloseoutFixture> {
    prepare_project_closeout_fixture(true)
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

fn prepare_project_closeout_fixture(mark_done: bool) -> Result<ProjectLoopCloseoutFixture> {
    let fixture = create_fixture_project()?;
    let root = fixture.root();
    prepare_all_layers(root)?;

    let requirement = root.join("docs/requirements/project-closeout-fixture.md");
    fs::create_dir_all(requirement.parent().unwrap())?;
    fs::write(
        &requirement,
        "# project-closeout-fixture\n\n用于验证 project loop 顺序与收口门禁。\n",
    )?;

    let project_id = "project-closeout";
    let current_issue_id = "AF-CLOSE-001";
    let next_issue_id = "AF-CLOSE-002";
    let current_issue = write_issue_with_requirement(
        root,
        &requirement,
        current_issue_id,
        SpecIssueStatus::Backlog,
        Vec::new(),
        Some(project_id.to_string()),
        "验证当前任务收口门禁。",
    )?;
    let next_issue = write_issue_with_requirement(
        root,
        &requirement,
        next_issue_id,
        SpecIssueStatus::Backlog,
        Vec::new(),
        Some(project_id.to_string()),
        "验证下一条任务只能在 Done 后启动。",
    )?;

    let mut project_draft = SpecProjectDraft::new(project_id);
    project_draft.title = Some("Project Loop 收口验证".to_string());
    project_draft.summary =
        Some("验证 in_review 与 done 的边界，以及 project loop 顺序。".to_string());
    project_draft.objective = Some("确保当前任务未完成收口时，下一条任务不会被启动。".to_string());
    project_draft.issue_ids = vec![current_issue.issue_id.clone(), next_issue.issue_id.clone()];
    let project = agentflow_spec::project_from_requirement(root, &requirement, project_draft)?;
    write_spec_project(root, &project)?;
    init_git_repo(root)?;

    let run_id = "run-001";
    agentflow_task_artifacts::create_task_run(
        root,
        &current_issue.issue_id,
        run_id,
        &current_issue.workflow_ref,
        Some("agentflow/direct/AF-CLOSE-001".to_string()),
    )?;
    append_issue_event(root, &current_issue, "issue.scheduled", "todo", run_id)?;
    append_issue_event(
        root,
        &current_issue,
        "agent.launch.requested",
        "in_progress",
        run_id,
    )?;
    agentflow_task_artifacts::write_task_command_record(
        root,
        &current_issue.issue_id,
        run_id,
        TaskCommandInput {
            label: "printf closeout".to_string(),
            program: "printf".to_string(),
            args: vec!["closeout".to_string()],
            exit_code: Some(0),
            stdout: "closeout".to_string(),
            stderr: String::new(),
        },
    )?;
    agentflow_task_artifacts::write_task_validation(root, &current_issue.issue_id, run_id)?;
    agentflow_task_artifacts::write_task_evidence(
        root,
        &current_issue.issue_id,
        run_id,
        "Project closeout fixture evidence.",
    )?;
    append_issue_event(
        root,
        &current_issue,
        "issue.review.requested",
        "in_review",
        run_id,
    )?;
    append_issue_event(
        root,
        &current_issue,
        "issue.pr.created",
        "in_review",
        run_id,
    )?;
    append_issue_event(
        root,
        &current_issue,
        "issue.closeout.proof.recorded",
        "in_review",
        run_id,
    )?;
    append_issue_event(root, &current_issue, "issue.pr.merged", "in_review", run_id)?;
    if mark_done {
        append_issue_event(root, &current_issue, "issue.completed", "done", run_id)?;
    }

    agentflow_projection::rebuild_projections(root)?;
    agentflow_state::refresh_state(root)?;
    fixture.assert_user_files_unchanged()?;
    Ok(ProjectLoopCloseoutFixture {
        fixture,
        project_id: project_id.to_string(),
        current_issue_id: current_issue.issue_id,
        next_issue_id: next_issue.issue_id,
    })
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
    write_issue_with_requirement(
        root,
        &requirement,
        issue_id,
        status,
        blocked_by,
        None,
        "Validate AgentFlow end-to-end workflow without modifying user source.",
    )
}

fn write_issue_with_requirement(
    root: &Path,
    requirement: &Path,
    issue_id: &str,
    status: SpecIssueStatus,
    blocked_by: Vec<String>,
    project_id: Option<String>,
    summary: &str,
) -> Result<SpecIssue> {
    let mut draft = SpecIssueDraft::new(issue_id);
    draft.title = Some("End-to-end workflow acceptance issue".to_string());
    draft.summary = Some(summary.to_string());
    draft.allowed_paths = vec!["src/lib.rs".to_string()];
    draft.validation_commands = vec!["printf ok".to_string()];
    draft.blocked_by = blocked_by;
    draft.project_id = project_id;
    let mut issue = agentflow_spec::issue_from_requirement(root, requirement, draft)?;
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

fn init_git_repo(root: &Path) -> Result<()> {
    fs::write(root.join(".gitignore"), ".agentflow/\n")?;
    run_git(root, &["init"])?;
    run_git(root, &["config", "user.email", "codex@example.com"])?;
    run_git(root, &["config", "user.name", "Codex"])?;
    run_git(root, &["add", "."])?;
    run_git(root, &["commit", "-m", "initial fixture"])?;
    Ok(())
}

fn run_git(root: &Path, args: &[&str]) -> Result<()> {
    let status = Command::new("git").args(args).current_dir(root).status()?;
    anyhow::ensure!(status.success(), "git {:?} failed", args);
    Ok(())
}
