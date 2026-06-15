use crate::{
    assertions::{assert_path_exists, write_json},
    fixture::{create_fixture_project, WorkflowFixture},
};
use agentflow_execute::{
    acquire_execute_lease, create_execute_checkpoint, create_execute_run, execute_run_preflight,
    run_execute_command, validate_execute_run, write_execute_plan, ExecuteChangedFiles,
    ExecuteCommandRequest, ExecuteLease, ExecuteLeaseStatus, ExecutePlanDraft, ExecutePlanStep,
    ExecutePlanStepKind,
};
use agentflow_input::{
    issue::{InputIssue, InputIssueModel, InputIssueStatus, InputRiskLevel},
    spec_gate::{InputIssueGenerationMode, InputSpecApproval},
};
use agentflow_output::{AuditScope, AuditScopeRef, HumanAuditReport, HumanAuditRequestDraft};
use agentflow_state::{
    StateStatusSnapshot, StateWorkspaceStatus, WorkflowAuditStatus, WorkflowStage,
};
use anyhow::Result;
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
    write_approved_spec(root, "spec-001")?;
    write_issue(root, "iss-001", "spec-001", InputRiskLevel::Low)?;

    let run = create_execute_run(root, "iss-001".to_string())?;
    let preflight = execute_run_preflight(root, run.run_id.clone())?;
    anyhow::ensure!(preflight.status == "ready", "expected ready preflight");
    acquire_execute_lease(root, run.run_id.clone())?;
    write_execute_plan(
        root,
        run.run_id.clone(),
        ExecutePlanDraft {
            steps: vec![ExecutePlanStep {
                step_id: "step-001".to_string(),
                kind: ExecutePlanStepKind::Validate,
                target: None,
                command: Some("printf ok".to_string()),
                summary: "Record deterministic fixture validation command.".to_string(),
            }],
            allowed_write_paths: vec!["src/lib.rs".to_string()],
            allowed_commands: vec!["printf ok".to_string()],
        },
    )?;
    create_execute_checkpoint(root, run.run_id.clone())?;
    write_empty_changed_files(root, &run.run_id)?;
    run_execute_command(
        root,
        run.run_id.clone(),
        ExecuteCommandRequest {
            label: "printf ok".to_string(),
            program: "printf".to_string(),
            args: vec!["ok".to_string()],
            source: Some("014.workflow-acceptance".to_string()),
        },
    )?;
    let result = validate_execute_run(root, run.run_id.clone())?;
    anyhow::ensure!(
        result.validation.passed,
        "fixture validation command failed"
    );
    anyhow::ensure!(
        result.changed_files.is_empty(),
        "fixture must not change user files"
    );
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
        run_id: run.run_id,
        execution_state,
    })
}

pub fn create_high_risk_blocker_fixture() -> Result<WorkflowFixture> {
    let fixture = create_fixture_project()?;
    let root = fixture.root();
    prepare_all_layers(root)?;
    write_approved_spec(root, "spec-001")?;
    write_issue(root, "iss-001", "spec-001", InputRiskLevel::High)?;
    let run = create_execute_run(root, "iss-001".to_string())?;
    let preflight = execute_run_preflight(root, run.run_id)?;
    anyhow::ensure!(
        preflight.status == "blocked",
        "high risk preflight must block"
    );
    let state = agentflow_state::refresh_state(root)?;
    anyhow::ensure!(
        state
            .blockers
            .iter()
            .any(|blocker| blocker.reason.contains("High risk issue requires")),
        "high risk blocker was not surfaced in state"
    );
    fixture.assert_user_files_unchanged()?;
    Ok(fixture)
}

pub fn create_stale_lease_fixture() -> Result<WorkflowFixture> {
    let fixture = create_fixture_project()?;
    let root = fixture.root();
    prepare_all_layers(root)?;
    let stale_lease = ExecuteLease {
        version: "execute-lease.v1".to_string(),
        issue_id: "iss-stale".to_string(),
        run_id: "run-stale".to_string(),
        status: ExecuteLeaseStatus::Active,
        created_at: 1,
        released_at: None,
        expires_at: None,
        locked_files: Vec::new(),
    };
    write_json(
        &root.join(".agentflow/execute/leases/iss-stale.json"),
        &stale_lease,
    )?;
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
                    kind: "spec".to_string(),
                    id: "spec-001".to_string(),
                    path: ".agentflow/input/specs/approved/spec-001/".to_string(),
                },
                AuditScopeRef {
                    kind: "issue".to_string(),
                    id: "iss-001".to_string(),
                    path: ".agentflow/input/issues/iss-001.json".to_string(),
                },
                AuditScopeRef {
                    kind: "execute-run".to_string(),
                    id: run_id.to_string(),
                    path: format!(".agentflow/execute/runs/{run_id}/"),
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
    let report = agentflow_output::request_human_audit(root, human_audit_draft(run_id))?;
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
        matches!(
            panel.status,
            agentflow_panel::PanelStatus::Ready | agentflow_panel::PanelStatus::Degraded
        ),
        "panel layer is not ready or degraded"
    );
    agentflow_spec::prepare_spec_workspace(root)?;
    agentflow_projection::prepare_projection_workspace(root)?;
    let input = agentflow_input::prepare_input_workspace(root)?;
    anyhow::ensure!(input.ready, "input layer is not ready");
    let execute = agentflow_execute::prepare_execute_workspace(root)?;
    anyhow::ensure!(execute.ready, "execute layer is not ready");
    let output = agentflow_output::prepare_output_workspace(root)?;
    anyhow::ensure!(output.ready, "output layer is not ready");
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
        ".agentflow/input/manifest.json",
        ".agentflow/execute/manifest.json",
        ".agentflow/output/manifest.json",
        ".agentflow/state/manifest.json",
        ".agentflow/state/gates/workflow.json",
    ] {
        assert_path_exists(root, relative_path)?;
    }
    Ok(())
}

fn write_approved_spec(root: &Path, spec_id: &str) -> Result<()> {
    let spec_dir = root.join(".agentflow/input/specs/approved").join(spec_id);
    fs::create_dir_all(&spec_dir)?;
    fs::write(
        spec_dir.join("product.md"),
        "# Product\n\nFixture product acceptance.\n",
    )?;
    fs::write(
        spec_dir.join("tech.md"),
        "# Tech\n\nFixture tech acceptance.\n",
    )?;
    fs::write(spec_dir.join("spec.json"), "{}\n")?;
    write_json(
        &spec_dir.join("approval.json"),
        &InputSpecApproval {
            spec_id: spec_id.to_string(),
            issue_generation_mode: InputIssueGenerationMode::Direct,
            ..InputSpecApproval::default()
        },
    )
}

fn write_issue(
    root: &Path,
    issue_id: &str,
    spec_id: &str,
    risk_level: InputRiskLevel,
) -> Result<()> {
    let issue = InputIssue {
        issue_id: issue_id.to_string(),
        issue_model: InputIssueModel::Direct,
        source_spec_id: spec_id.to_string(),
        project_id: None,
        title: "End-to-end workflow acceptance issue".to_string(),
        summary: "Validate AgentFlow end-to-end workflow without modifying user source."
            .to_string(),
        status: InputIssueStatus::Todo,
        execution_risk: risk_level,
        scope: vec!["src/lib.rs".to_string()],
        non_goals: vec!["Do not modify fixture source.".to_string()],
        acceptance_criteria: vec!["Workflow reaches delivery-ready.".to_string()],
        validation_hints: vec!["printf ok".to_string()],
        ..InputIssue::default()
    };
    write_json(
        &root
            .join(".agentflow/input/issues")
            .join(format!("{issue_id}.json")),
        &issue,
    )
}

fn write_empty_changed_files(root: &Path, run_id: &str) -> Result<()> {
    write_json(
        &root
            .join(".agentflow/execute/runs")
            .join(run_id)
            .join("patches/changed-files.json"),
        &ExecuteChangedFiles {
            version: "execute-changed-files.v1".to_string(),
            run_id: run_id.to_string(),
            files: Vec::new(),
        },
    )
}
