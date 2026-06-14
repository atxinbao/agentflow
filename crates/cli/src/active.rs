//! Active CLI boundary.
//!
//! Active commands are narrow wrappers around the current workspace crates.
//! They must not call archived 2026-05 writers.

use agentflow_execute::{
    mark_build_agent_launch_done, mark_build_agent_launch_in_review, BuildAgentCompletion,
    BuildAgentCompletionRequest,
};
use agentflow_input::issue::{
    AgentRole, InputIssue, InputIssueModel, InputIssueStatus, IssueCategory,
};
use agentflow_loop::{
    write_issue_merge_proof, DirectIssueLoop, IssueLoop, IssueLoopProjection,
    ProjectExecutionLaunch, ProjectExecutor, ProjectLoop,
};
use agentflow_task_loop::{AgentLaunchPayload, AGENT_LAUNCH_REQUESTED};
use agentflow_workflow_events::{
    append_event_once, BuildAgentMergeConfirmedPayload, BuildAgentSessionReviewReadyPayload,
    BuildAgentWritebackCompletedPayload, WorkflowEventDraft,
    EVENT_TYPE_BUILD_AGENT_MERGE_CONFIRMED, EVENT_TYPE_BUILD_AGENT_SESSION_REVIEW_READY,
    EVENT_TYPE_BUILD_AGENT_WRITEBACK_COMPLETED,
};
use anyhow::{Context, Result};
use std::{
    fs,
    path::{Path, PathBuf},
    time::SystemTime,
};

const CLI_FRESHNESS_PATHS: [&str; 9] = [
    "Cargo.toml",
    "Cargo.lock",
    "crates/cli/src",
    "crates/execute/src",
    "crates/input/src",
    "crates/state/src",
    "crates/panel/src",
    "crates/agent-manual/src",
    "crates/loop/src",
];

#[derive(Debug, Clone)]
pub(crate) struct BuildAgentStart {
    pub issue_id: String,
    pub run_id: String,
    pub stage: String,
    pub branch_name: Option<String>,
    pub project_id: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct BuildAgentMergeProof {
    pub issue_id: String,
    pub run_id: String,
    pub merged: bool,
    pub proof_path: PathBuf,
}

#[derive(Debug, Clone)]
pub(crate) struct BuildAgentLaunchClaim {
    pub event_id: String,
    pub issue_id: String,
    pub run_id: String,
    pub branch_name: Option<String>,
    pub launch_request_path: PathBuf,
}

#[derive(Debug, Clone)]
pub(crate) struct BuildAgentCompletionOutcome {
    pub completion: BuildAgentCompletion,
    pub next_launch: Option<ProjectExecutionLaunch>,
}

pub(crate) fn complete_build_agent_issue_from_request(
    root: &Path,
    request_path: &Path,
) -> Result<BuildAgentCompletionOutcome> {
    assert_current_cli_is_fresh(root)?;
    let raw = fs::read_to_string(request_path)
        .with_context(|| format!("read completion request {}", request_path.display()))?;
    let request: BuildAgentCompletionRequest = serde_json::from_str(&raw)
        .with_context(|| format!("parse completion request {}", request_path.display()))?;
    let completion = agentflow_execute::complete_build_agent_issue(root, request)?;
    mark_build_agent_launch_done(root, &completion.run.run_id)?;
    let next_launch = completion
        .run
        .project_id
        .as_deref()
        .map(|project_id| ProjectExecutor::new(project_id).tick(root))
        .transpose()?
        .and_then(|tick| tick.launch);
    append_build_agent_writeback_completed_event(root, &completion, next_launch.as_ref())?;
    agentflow_state::refresh_state(root)?;
    Ok(BuildAgentCompletionOutcome {
        completion,
        next_launch,
    })
}

pub(crate) fn prepare_build_agent_review_from_request(
    root: &Path,
    request_path: &Path,
) -> Result<BuildAgentCompletion> {
    assert_current_cli_is_fresh(root)?;
    let raw = fs::read_to_string(request_path)
        .with_context(|| format!("read review preparation request {}", request_path.display()))?;
    let request: BuildAgentCompletionRequest = serde_json::from_str(&raw).with_context(|| {
        format!(
            "parse review preparation request {}",
            request_path.display()
        )
    })?;
    let prepared = agentflow_execute::prepare_build_agent_review(root, request)?;
    mark_build_agent_launch_in_review(root, &prepared.run.run_id)?;
    append_build_agent_review_ready_event(root, &prepared)?;
    agentflow_state::refresh_state(root)?;
    Ok(prepared)
}

pub(crate) fn start_build_agent_issue(root: &Path, issue_id: &str) -> Result<BuildAgentStart> {
    assert_current_cli_is_fresh(root)?;
    let issue_id = issue_id.trim();
    if issue_id.is_empty() {
        anyhow::bail!("build agent start requires issueId");
    }
    let mut issue = agentflow_input::load_input_issue(root, issue_id)
        .with_context(|| format!("load input issue {issue_id}"))?;
    assert_build_agent_contract(&issue)?;
    if matches!(issue.status, InputIssueStatus::Backlog) {
        schedule_issue_for_runtime(root, &issue)?;
        issue = agentflow_input::load_input_issue(root, issue_id)
            .with_context(|| format!("reload input issue {issue_id} after scheduling"))?;
    }
    if !matches!(issue.status, InputIssueStatus::Todo) {
        anyhow::bail!(
            "build agent start requires todo issue after scheduling; {} is {}",
            issue.issue_id,
            issue.status.as_str()
        );
    }

    let projection = start_issue_runtime_preflight(root, &issue)?;
    agentflow_state::refresh_state(root)?;
    Ok(BuildAgentStart {
        issue_id: issue.issue_id,
        run_id: projection
            .run_id
            .clone()
            .ok_or_else(|| anyhow::anyhow!("runtime preflight did not produce runId"))?,
        stage: projection.stage.as_str().to_string(),
        branch_name: projection.branch_name,
        project_id: projection.project_id,
    })
}

pub(crate) fn claim_next_build_agent_launch(root: &Path) -> Result<Option<BuildAgentLaunchClaim>> {
    assert_current_cli_is_fresh(root)?;
    claim_next_build_agent_launch_with_bridge(
        root,
        &agentflow_agent_bridge::AgentBridge::with_default_providers(),
    )
}

fn claim_next_build_agent_launch_with_bridge(
    root: &Path,
    bridge: &agentflow_agent_bridge::AgentBridge,
) -> Result<Option<BuildAgentLaunchClaim>> {
    let Some(claim) = bridge.claim_next_launch(root)? else {
        return Ok(None);
    };
    let payload = load_agent_launch_payload(root, &claim.run_id)?;
    let launch_request_path = root.join(&payload.launch_request_path);
    if !launch_request_path.is_file() {
        anyhow::bail!(
            "build agent launch request is missing: {}",
            launch_request_path.display()
        );
    }
    Ok(Some(BuildAgentLaunchClaim {
        event_id: claim.created_event_id,
        issue_id: claim.issue_id,
        run_id: claim.run_id,
        branch_name: Some(payload.branch_name),
        launch_request_path,
    }))
}

fn load_agent_launch_payload(root: &Path, run_id: &str) -> Result<AgentLaunchPayload> {
    let event = agentflow_event_store::load_task_events(root)?
        .into_iter()
        .find(|event| {
            event.event_type == AGENT_LAUNCH_REQUESTED
                && event
                    .payload
                    .get("runId")
                    .and_then(serde_json::Value::as_str)
                    == Some(run_id)
        })
        .ok_or_else(|| anyhow::anyhow!("missing agent launch request for run {run_id}"))?;
    serde_json::from_value(event.payload)
        .with_context(|| format!("parse agent launch payload {}", event.event_id))
}

pub(crate) fn write_build_agent_merge_proof(
    root: &Path,
    issue_id: &str,
    run_id: &str,
    provider: &str,
    merge_mode: &str,
    remote_url: Option<String>,
    merged: bool,
) -> Result<BuildAgentMergeProof> {
    assert_current_cli_is_fresh(root)?;
    let issue = agentflow_input::load_input_issue(root, issue_id)
        .with_context(|| format!("load input issue {issue_id}"))?;
    assert_build_agent_contract(&issue)?;
    let proof_path = write_issue_merge_proof(
        root,
        &issue.issue_id,
        issue.project_id.as_deref(),
        run_id,
        provider,
        merge_mode,
        remote_url,
        merged,
    )?;
    if merged {
        append_build_agent_merge_confirmed_event(
            root,
            &issue,
            run_id,
            provider,
            merge_mode,
            proof_path.as_path(),
        )?;
    }
    agentflow_state::refresh_state(root)?;
    Ok(BuildAgentMergeProof {
        issue_id: issue.issue_id,
        run_id: run_id.to_string(),
        merged,
        proof_path,
    })
}

fn assert_build_agent_contract(issue: &InputIssue) -> Result<()> {
    if !matches!(issue.issue_category, IssueCategory::Spec) {
        anyhow::bail!(
            "build agent start only supports spec issues; {} is {}",
            issue.issue_id,
            issue.issue_category.as_str()
        );
    }
    if !matches!(issue.required_agent_role, AgentRole::BuildAgent) {
        anyhow::bail!(
            "build agent start only supports build-agent issues; {} is {}",
            issue.issue_id,
            issue.required_agent_role.as_str()
        );
    }
    Ok(())
}

fn append_build_agent_review_ready_event(
    root: &Path,
    completion: &BuildAgentCompletion,
) -> Result<()> {
    let run = &completion.run;
    append_event_once(
        root,
        WorkflowEventDraft {
            event_type: EVENT_TYPE_BUILD_AGENT_SESSION_REVIEW_READY.to_string(),
            source: "agentflow-cli".to_string(),
            subject_id: run.issue_id.clone(),
            subject_path: Some(format!(".agentflow/input/issues/{}.json", run.issue_id)),
            dedupe_key: format!("build-agent.session.review-ready:{}", run.run_id),
            payload: serde_json::to_value(BuildAgentSessionReviewReadyPayload {
                issue_id: run.issue_id.clone(),
                project_id: run.project_id.clone(),
                run_id: run.run_id.clone(),
                provider: "codex".to_string(),
                delivery_path: Some(format!(
                    ".agentflow/output/release/{}/delivery.json",
                    run.run_id
                )),
            })?,
        },
    )?;
    Ok(())
}

fn append_build_agent_merge_confirmed_event(
    root: &Path,
    issue: &InputIssue,
    run_id: &str,
    provider: &str,
    merge_mode: &str,
    proof_path: &Path,
) -> Result<()> {
    let payload = serde_json::to_value(BuildAgentMergeConfirmedPayload {
        issue_id: issue.issue_id.clone(),
        project_id: issue.project_id.clone(),
        run_id: run_id.to_string(),
        provider: provider.to_string(),
        merge_mode: merge_mode.to_string(),
        remote_url: serde_json::from_str::<serde_json::Value>(&fs::read_to_string(proof_path)?)
            .ok()
            .and_then(|value| {
                value
                    .get("remoteUrl")
                    .and_then(serde_json::Value::as_str)
                    .map(str::to_string)
            }),
        merged: true,
    })?;
    append_event_once(
        root,
        WorkflowEventDraft {
            event_type: EVENT_TYPE_BUILD_AGENT_MERGE_CONFIRMED.to_string(),
            source: "agentflow-cli".to_string(),
            subject_id: issue.issue_id.clone(),
            subject_path: Some(format!(
                ".agentflow/execute/runs/{run_id}/review/merge-proof.json"
            )),
            dedupe_key: format!("build-agent.merge.confirmed:{run_id}"),
            payload,
        },
    )?;
    Ok(())
}

fn append_build_agent_writeback_completed_event(
    root: &Path,
    completion: &BuildAgentCompletion,
    next_launch: Option<&ProjectExecutionLaunch>,
) -> Result<()> {
    let run = &completion.run;
    append_event_once(
        root,
        WorkflowEventDraft {
            event_type: EVENT_TYPE_BUILD_AGENT_WRITEBACK_COMPLETED.to_string(),
            source: "agentflow-cli".to_string(),
            subject_id: run.issue_id.clone(),
            subject_path: Some(format!(".agentflow/input/issues/{}.json", run.issue_id)),
            dedupe_key: format!("build-agent.writeback.completed:{}", run.run_id),
            payload: serde_json::to_value(BuildAgentWritebackCompletedPayload {
                issue_id: run.issue_id.clone(),
                project_id: run.project_id.clone(),
                run_id: run.run_id.clone(),
                provider: "codex".to_string(),
                delivery_path: Some(format!(
                    ".agentflow/output/release/{}/delivery.json",
                    run.run_id
                )),
                next_issue_id: next_launch.map(|launch| launch.issue_id.clone()),
            })?,
        },
    )?;
    Ok(())
}

fn schedule_issue_for_runtime(root: &Path, issue: &InputIssue) -> Result<()> {
    match issue.issue_model {
        InputIssueModel::Direct => {
            DirectIssueLoop::schedule_ready_issues(root)?;
        }
        InputIssueModel::Project => {
            let project_id = issue.project_id.as_deref().ok_or_else(|| {
                anyhow::anyhow!("project issue {} is missing projectId", issue.issue_id)
            })?;
            ProjectLoop::new(project_id).run_preflight(root)?;
            ProjectLoop::new(project_id).schedule_ready_issues(root)?;
        }
    }
    Ok(())
}

fn start_issue_runtime_preflight(root: &Path, issue: &InputIssue) -> Result<IssueLoopProjection> {
    match issue.issue_model {
        InputIssueModel::Direct => DirectIssueLoop::start_runtime_preflight(root, &issue.issue_id),
        InputIssueModel::Project => {
            let project_id = issue.project_id.as_deref().ok_or_else(|| {
                anyhow::anyhow!("project issue {} is missing projectId", issue.issue_id)
            })?;
            IssueLoop::new(project_id, &issue.issue_id).start_runtime_preflight(root)
        }
    }
}

fn assert_current_cli_is_fresh(root: &Path) -> Result<()> {
    let current_exe = std::env::current_exe().context("locate current agentflow CLI binary")?;
    if !is_local_target_binary(root, &current_exe) {
        return Ok(());
    }
    let binary_modified = file_modified(&current_exe)?;
    if let Some((newest_path, newest_modified)) = newest_source_mtime(root)? {
        if binary_is_stale(binary_modified, newest_modified) {
            anyhow::bail!(
                "current AgentFlow CLI is stale: {} is older than {}. Run `{}` before `build-agent complete`.",
                current_exe.display(),
                newest_path.display(),
                rebuild_hint(&current_exe)
            );
        }
    }
    Ok(())
}

fn newest_source_mtime(root: &Path) -> Result<Option<(PathBuf, SystemTime)>> {
    let mut newest = None;
    for relative in CLI_FRESHNESS_PATHS {
        collect_newest_mtime(&root.join(relative), &mut newest)?;
    }
    Ok(newest)
}

fn collect_newest_mtime(path: &Path, newest: &mut Option<(PathBuf, SystemTime)>) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }
    let metadata = fs::metadata(path)?;
    if metadata.is_file() {
        let modified = metadata.modified()?;
        let replace = newest
            .as_ref()
            .map(|(_, current)| modified > *current)
            .unwrap_or(true);
        if replace {
            *newest = Some((path.to_path_buf(), modified));
        }
        return Ok(());
    }
    if metadata.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            collect_newest_mtime(&entry.path(), newest)?;
        }
    }
    Ok(())
}

fn file_modified(path: &Path) -> Result<SystemTime> {
    Ok(fs::metadata(path)?.modified()?)
}

fn is_local_target_binary(root: &Path, executable: &Path) -> bool {
    executable.starts_with(root.join("target"))
        && executable
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| name == "agentflow")
            .unwrap_or(false)
}

fn rebuild_hint(executable: &Path) -> &'static str {
    if executable
        .components()
        .any(|component| component.as_os_str() == "release")
    {
        "cargo build --release --bin agentflow"
    } else {
        "cargo build --bin agentflow"
    }
}

fn binary_is_stale(binary_modified: SystemTime, newest_source_modified: SystemTime) -> bool {
    newest_source_modified > binary_modified
}

#[cfg(test)]
mod tests {
    use super::{
        binary_is_stale, claim_next_build_agent_launch_with_bridge,
        complete_build_agent_issue_from_request, is_local_target_binary,
        prepare_build_agent_review_from_request, rebuild_hint, write_build_agent_merge_proof,
    };
    use agentflow_execute::{
        acquire_execute_lease, apply_execute_patch, create_execute_checkpoint,
        load_build_agent_launch_state, run_execute_command, write_execute_plan,
        BuildAgentCompletionRequest, BuildAgentLaunchStatus, BuildAgentValidationCommand,
        ExecuteChangedFile, ExecuteCommandRequest, ExecutePlanDraft,
    };
    use agentflow_input::{
        issue::{
            AgentRole, InputIssue, InputIssueModel, InputIssueStatus, InputRiskLevel, IssueCategory,
        },
        project::InputProject,
        spec_gate::{InputIssueGenerationMode, InputSpecApproval},
    };
    use agentflow_loop::ProjectExecutor;
    use agentflow_mcp::{
        McpAgentProvider, McpLaunchMode, McpLaunchPlan, McpLaunchRequest, McpProviderBridge,
        McpProviderKind, McpProviderStatus, McpProviderStatusCode,
    };
    use agentflow_workflow_events::{
        load_events, load_pending_events, CONSUMER_BUILD_AGENT,
        EVENT_TYPE_BUILD_AGENT_LAUNCH_REQUESTED, EVENT_TYPE_BUILD_AGENT_MERGE_CONFIRMED,
        EVENT_TYPE_BUILD_AGENT_SESSION_REVIEW_READY, EVENT_TYPE_BUILD_AGENT_WRITEBACK_COMPLETED,
    };
    use anyhow::Result;
    use std::{
        fs,
        path::{Path, PathBuf},
        process::Command,
        time::{Duration, UNIX_EPOCH},
    };
    use tempfile::tempdir;

    struct FakeProvider;

    impl McpAgentProvider for FakeProvider {
        fn provider_id(&self) -> &'static str {
            "fake"
        }

        fn kind(&self) -> McpProviderKind {
            McpProviderKind::Codex
        }

        fn check_health(&self, _project_root: &Path) -> McpProviderStatus {
            let mut status = McpProviderStatus::new(McpProviderKind::Codex, 1);
            status.provider = "fake".to_string();
            status.status = McpProviderStatusCode::Ready;
            status
        }

        fn build_launch_plan(
            &self,
            _project_root: &Path,
            request: &McpLaunchRequest,
        ) -> Result<McpLaunchPlan> {
            let mut plan = McpLaunchPlan::new(
                "fake",
                format!("fake-{}", request.run_id),
                request.issue_id.clone(),
                request.run_id.clone(),
                McpLaunchMode::CliExecPromptFile,
                request.working_directory.clone(),
                "fake-agent",
            );
            plan.stdin_path = Some(request.launch_request_path.clone());
            Ok(plan)
        }
    }

    #[test]
    fn detects_local_target_binaries_only() {
        assert!(is_local_target_binary(
            Path::new("/repo"),
            Path::new("/repo/target/debug/agentflow")
        ));
        assert!(is_local_target_binary(
            Path::new("/repo"),
            Path::new("/repo/target/release/agentflow")
        ));
        assert!(!is_local_target_binary(
            Path::new("/repo"),
            Path::new("/usr/local/bin/agentflow")
        ));
    }

    #[test]
    fn stale_check_requires_newer_sources() {
        let binary = UNIX_EPOCH + Duration::from_secs(10);
        let source = UNIX_EPOCH + Duration::from_secs(11);
        assert!(binary_is_stale(binary, source));
        assert!(!binary_is_stale(source, binary));
    }

    #[test]
    fn rebuild_hint_matches_binary_profile() {
        assert_eq!(
            rebuild_hint(Path::new("/repo/target/debug/agentflow")),
            "cargo build --bin agentflow"
        );
        assert_eq!(
            rebuild_hint(Path::new("/repo/target/release/agentflow")),
            "cargo build --release --bin agentflow"
        );
    }

    #[test]
    fn claim_next_build_agent_launch_consumes_pending_event() {
        let dir = tempdir().unwrap();
        let requirement = dir.path().join("docs/requirements/034-claim-test.md");
        fs::create_dir_all(requirement.parent().unwrap()).unwrap();
        fs::write(
            &requirement,
            "# Claim Test\n\n验证 CLI claim 走 AgentBridge。\n",
        )
        .unwrap();
        let mut issue = agentflow_spec::SpecIssueDraft::new("AF-001");
        issue.project_id = Some("proj-001".to_string());
        let issue =
            agentflow_spec::issue_from_requirement(dir.path(), &requirement, issue).unwrap();
        agentflow_spec::write_spec_issue(dir.path(), &issue).unwrap();
        let mut project = agentflow_spec::SpecProjectDraft::new("proj-001");
        project.issue_ids = vec!["AF-001".to_string()];
        let project =
            agentflow_spec::project_from_requirement(dir.path(), &requirement, project).unwrap();
        agentflow_spec::write_spec_project(dir.path(), &project).unwrap();
        let loop_driver = agentflow_task_loop::TaskLoop::new("proj-001");
        loop_driver
            .schedule_next_issue(dir.path())
            .unwrap()
            .unwrap();
        let launch = loop_driver
            .request_agent_launch(dir.path(), "AF-001", "fake")
            .unwrap();
        let mut providers = McpProviderBridge::new();
        providers.register(Box::new(FakeProvider));
        let bridge = agentflow_agent_bridge::AgentBridge::new(providers);

        let claim = claim_next_build_agent_launch_with_bridge(dir.path(), &bridge)
            .unwrap()
            .expect("expected launch claim");
        assert_eq!(claim.issue_id, "AF-001");
        assert_eq!(claim.run_id, "run-001");
        assert_eq!(
            claim.branch_name.as_deref(),
            Some("agentflow/proj-001/AF-001")
        );
        assert_eq!(
            claim.launch_request_path,
            dir.path().join(&launch.launch_request_path)
        );
        let events = agentflow_event_store::load_task_events(dir.path()).unwrap();
        assert!(events
            .iter()
            .any(|event| event.event_type == agentflow_agent_bridge::AGENT_SESSION_CREATED));
        assert!(
            claim_next_build_agent_launch_with_bridge(dir.path(), &bridge)
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn completion_auto_launches_next_project_issue() {
        let dir = tempdir().unwrap();
        prepare_project_root(dir.path());
        write_approved_spec(dir.path(), "spec-001");
        write_project(dir.path(), "proj-001", &["AF-001", "AF-002"]);
        write_project_issue(dir.path(), "AF-001", Vec::new());
        write_project_issue(dir.path(), "AF-002", vec!["AF-001".to_string()]);
        agentflow_input::prepare_input_workspace(dir.path()).unwrap();

        let launch = ProjectExecutor::new("proj-001")
            .tick(dir.path())
            .unwrap()
            .launch
            .expect("first project runtime launch");

        acquire_execute_lease(dir.path(), launch.run_id.clone()).unwrap();
        write_execute_plan(
            dir.path(),
            launch.run_id.clone(),
            ExecutePlanDraft {
                steps: Vec::new(),
                allowed_write_paths: vec!["src/lib.rs".to_string()],
                allowed_commands: vec!["printf ok".to_string()],
            },
        )
        .unwrap();
        create_execute_checkpoint(dir.path(), launch.run_id.clone()).unwrap();
        apply_execute_patch(
            dir.path(),
            launch.run_id.clone(),
            "diff --git a/src/lib.rs b/src/lib.rs\n--- a/src/lib.rs\n+++ b/src/lib.rs\n@@ -1,3 +1,3 @@\n pub fn value() -> u8 {\n-    1\n+    2\n }\n"
                .to_string(),
        )
        .unwrap();
        run_execute_command(
            dir.path(),
            launch.run_id.clone(),
            ExecuteCommandRequest {
                label: "printf ok".to_string(),
                program: "printf".to_string(),
                args: vec!["ok".to_string()],
                source: Some("test".to_string()),
            },
        )
        .unwrap();

        let request_path = write_completion_request(dir.path(), &launch.run_id);
        let prepared = prepare_build_agent_review_from_request(dir.path(), &request_path).unwrap();
        assert_eq!(prepared.run.issue_id, "AF-001");
        let in_review_state = load_build_agent_launch_state(dir.path(), &launch.run_id).unwrap();
        assert_eq!(in_review_state.status, BuildAgentLaunchStatus::InReview);
        let review_events = load_events(dir.path()).unwrap();
        assert!(review_events.iter().any(|event| event.event_type
            == EVENT_TYPE_BUILD_AGENT_SESSION_REVIEW_READY
            && event.subject_id == "AF-001"));

        commit_and_merge_issue_branch(
            dir.path(),
            "agentflow/proj-001/AF-001",
            "main",
            "complete AF-001",
        );
        write_build_agent_merge_proof(
            dir.path(),
            "AF-001",
            &launch.run_id,
            "github",
            "auto-merge-if-eligible",
            Some("https://github.com/atxinbao/agentflow/pull/1".to_string()),
            true,
        )
        .unwrap();
        let merge_events = load_events(dir.path()).unwrap();
        assert!(merge_events.iter().any(|event| event.event_type
            == EVENT_TYPE_BUILD_AGENT_MERGE_CONFIRMED
            && event.subject_id == "AF-001"));

        let outcome = complete_build_agent_issue_from_request(dir.path(), &request_path).unwrap();
        assert_eq!(outcome.completion.run.issue_id, "AF-001");
        let done_state = load_build_agent_launch_state(dir.path(), &launch.run_id).unwrap();
        assert_eq!(done_state.status, BuildAgentLaunchStatus::Done);
        let completion_events = load_events(dir.path()).unwrap();
        assert!(completion_events.iter().any(|event| event.event_type
            == EVENT_TYPE_BUILD_AGENT_WRITEBACK_COMPLETED
            && event.subject_id == "AF-001"));

        let next_launch = outcome.next_launch.expect("next issue launch");
        assert_eq!(next_launch.issue_id, "AF-002");
        let next_issue = agentflow_input::load_input_issue(dir.path(), "AF-002").unwrap();
        assert_eq!(next_issue.status, InputIssueStatus::InProgress);
        assert_eq!(
            next_issue.latest_run_id.as_deref(),
            Some(next_launch.run_id.as_str())
        );
        let pending = load_pending_events(
            dir.path(),
            CONSUMER_BUILD_AGENT,
            &[EVENT_TYPE_BUILD_AGENT_LAUNCH_REQUESTED],
        )
        .unwrap();
        assert!(pending.iter().any(|event| event.subject_id == "AF-002"));
    }

    fn prepare_project_root(root: &Path) {
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("README.md"), "# fixture\n").unwrap();
        fs::write(
            root.join("src/lib.rs"),
            "pub fn value() -> u8 {\n    1\n}\n",
        )
        .unwrap();
        fs::write(root.join(".gitignore"), ".agentflow/\nAGENTS.md\n").unwrap();
        let init = Command::new("git")
            .args(["init", "-b", "main"])
            .current_dir(root)
            .output()
            .unwrap();
        assert!(init.status.success());
        let add = Command::new("git")
            .args(["add", ".gitignore", "README.md", "src/lib.rs"])
            .current_dir(root)
            .output()
            .unwrap();
        assert!(add.status.success());
        let commit = Command::new("git")
            .args([
                "-c",
                "user.name=AgentFlow Test",
                "-c",
                "user.email=agentflow-test@example.com",
                "commit",
                "-m",
                "initial",
            ])
            .current_dir(root)
            .output()
            .unwrap();
        assert!(
            commit.status.success(),
            "{}{}",
            String::from_utf8_lossy(&commit.stdout),
            String::from_utf8_lossy(&commit.stderr)
        );
        agentflow_input::prepare_input_workspace(root).unwrap();
    }

    fn write_approved_spec(root: &Path, spec_id: &str) {
        let spec_dir = root.join(".agentflow/input/specs/approved").join(spec_id);
        fs::create_dir_all(&spec_dir).unwrap();
        fs::write(spec_dir.join("product.md"), "# Product\n").unwrap();
        fs::write(spec_dir.join("tech.md"), "# Tech\n").unwrap();
        fs::write(spec_dir.join("spec.json"), "{}\n").unwrap();
        fs::write(
            spec_dir.join("approval.json"),
            serde_json::to_string_pretty(&InputSpecApproval {
                spec_id: spec_id.to_string(),
                issue_generation_mode: InputIssueGenerationMode::Project,
                ..InputSpecApproval::default()
            })
            .unwrap(),
        )
        .unwrap();
    }

    fn write_project(root: &Path, project_id: &str, issue_ids: &[&str]) {
        let project = InputProject {
            project_id: project_id.to_string(),
            source_spec_id: "spec-001".to_string(),
            title: "Project fixture".to_string(),
            summary: "Project fixture summary".to_string(),
            objective: "Complete project issue chain".to_string(),
            issue_ids: issue_ids.iter().map(|value| (*value).to_string()).collect(),
            scope: vec!["src/lib.rs".to_string()],
            success_criteria: vec!["All issues done".to_string()],
            ..InputProject::default()
        };
        fs::write(
            root.join(".agentflow/input/projects")
                .join(format!("{project_id}.json")),
            serde_json::to_string_pretty(&project).unwrap(),
        )
        .unwrap();
    }

    fn write_project_issue(root: &Path, issue_id: &str, blocked_by: Vec<String>) {
        let mut issue = InputIssue {
            issue_id: issue_id.to_string(),
            issue_model: InputIssueModel::Project,
            issue_category: IssueCategory::Spec,
            required_agent_role: AgentRole::BuildAgent,
            source_spec_id: "spec-001".to_string(),
            project_id: Some("proj-001".to_string()),
            title: format!("Execute {issue_id}"),
            summary: format!("Execute {issue_id}"),
            status: InputIssueStatus::Backlog,
            execution_risk: InputRiskLevel::Low,
            scope: vec!["src/lib.rs".to_string()],
            validation_hints: vec!["printf ok".to_string()],
            ..InputIssue::default()
        };
        issue.relations.blocked_by = blocked_by;
        issue.normalize_execution_metadata();
        fs::write(
            root.join(".agentflow/input/issues")
                .join(format!("{issue_id}.json")),
            serde_json::to_string_pretty(&issue).unwrap(),
        )
        .unwrap();
    }

    fn write_completion_request(root: &Path, run_id: &str) -> PathBuf {
        let path = root
            .join(".agentflow/tmp")
            .join(format!("{run_id}-completion-request.json"));
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(
            &path,
            serde_json::to_string_pretty(&BuildAgentCompletionRequest {
                issue_id: "AF-001".to_string(),
                run_id: Some(run_id.to_string()),
                changed_files: vec![ExecuteChangedFile {
                    path: "src/lib.rs".to_string(),
                    change_type: "modified".to_string(),
                    insertions: 1,
                    deletions: 1,
                }],
                validation_commands: vec![BuildAgentValidationCommand {
                    label: "printf ok".to_string(),
                    program: "printf".to_string(),
                    args: vec!["ok".to_string()],
                    exit_code: Some(0),
                    stdout: Some("ok".to_string()),
                    stderr: None,
                    source: Some("test".to_string()),
                }],
            })
            .unwrap(),
        )
        .unwrap();
        path
    }

    fn commit_and_merge_issue_branch(
        root: &Path,
        issue_branch: &str,
        base_branch: &str,
        message: &str,
    ) {
        let add = Command::new("git")
            .args(["add", "src/lib.rs"])
            .current_dir(root)
            .output()
            .unwrap();
        assert!(add.status.success());
        let commit = Command::new("git")
            .args([
                "-c",
                "user.name=AgentFlow Test",
                "-c",
                "user.email=agentflow-test@example.com",
                "commit",
                "-m",
                message,
            ])
            .current_dir(root)
            .output()
            .unwrap();
        assert!(
            commit.status.success(),
            "{}{}",
            String::from_utf8_lossy(&commit.stdout),
            String::from_utf8_lossy(&commit.stderr)
        );
        let switch_base = Command::new("git")
            .args(["switch", base_branch])
            .current_dir(root)
            .output()
            .unwrap();
        assert!(
            switch_base.status.success(),
            "{}{}",
            String::from_utf8_lossy(&switch_base.stdout),
            String::from_utf8_lossy(&switch_base.stderr)
        );
        let merge = Command::new("git")
            .args(["merge", "--no-ff", "--no-edit", issue_branch])
            .current_dir(root)
            .output()
            .unwrap();
        assert!(
            merge.status.success(),
            "{}{}",
            String::from_utf8_lossy(&merge.stdout),
            String::from_utf8_lossy(&merge.stderr)
        );
    }
}
