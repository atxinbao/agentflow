//! Active CLI boundary.
//!
//! Active commands are narrow wrappers around the current workspace crates.
//! They must not call archived 2026-05 writers.

use agentflow_execute::{
    claim_build_agent_launch, mark_build_agent_launch_done, mark_build_agent_launch_in_review,
    BuildAgentCompletion, BuildAgentCompletionRequest,
};
use agentflow_input::issue::{
    AgentRole, InputIssue, InputIssueModel, InputIssueStatus, IssueCategory,
};
use agentflow_loop::{
    write_issue_merge_proof, DirectIssueLoop, IssueLoop, IssueLoopProjection,
    ProjectExecutionLaunch, ProjectExecutor, ProjectLoop,
};
use agentflow_workflow_events::{
    load_pending_events, mark_event_consumed, BuildAgentLaunchRequestedPayload,
    CONSUMER_BUILD_AGENT, EVENT_TYPE_BUILD_AGENT_LAUNCH_REQUESTED,
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
    "crates/agentflow-cli/src",
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
    let pending = load_pending_events(
        root,
        CONSUMER_BUILD_AGENT,
        &[EVENT_TYPE_BUILD_AGENT_LAUNCH_REQUESTED],
    )?;
    let Some(event) = pending.into_iter().next() else {
        return Ok(None);
    };
    let payload: BuildAgentLaunchRequestedPayload =
        serde_json::from_value(event.payload.clone())
            .with_context(|| format!("parse build-agent launch payload {}", event.event_id))?;
    let launch_request_path = root.join(&payload.launch_request_path);
    if !launch_request_path.is_file() {
        anyhow::bail!(
            "build agent launch request is missing: {}",
            launch_request_path.display()
        );
    }
    claim_build_agent_launch(
        root,
        &payload.issue_id,
        Some(&payload.project_id),
        &payload.run_id,
        payload.branch_name.clone(),
        payload.launch_request_path.clone(),
        event.event_id.clone(),
    )?;
    mark_event_consumed(root, CONSUMER_BUILD_AGENT, &event.event_id)?;
    Ok(Some(BuildAgentLaunchClaim {
        event_id: event.event_id,
        issue_id: payload.issue_id,
        run_id: payload.run_id,
        branch_name: payload.branch_name,
        launch_request_path,
    }))
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
        binary_is_stale, claim_next_build_agent_launch, complete_build_agent_issue_from_request,
        is_local_target_binary, prepare_build_agent_review_from_request, rebuild_hint,
        write_build_agent_merge_proof,
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
    use agentflow_workflow_events::{
        append_event_once, load_pending_events, BuildAgentLaunchRequestedPayload,
        WorkflowEventDraft, CONSUMER_BUILD_AGENT, EVENT_TYPE_BUILD_AGENT_LAUNCH_REQUESTED,
    };
    use std::{
        fs,
        path::{Path, PathBuf},
        process::Command,
        time::{Duration, UNIX_EPOCH},
    };
    use tempfile::tempdir;

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
        let request_path = dir
            .path()
            .join(".agentflow/execute/runs/run-001/launcher/build-agent-request.json");
        fs::create_dir_all(request_path.parent().unwrap()).unwrap();
        fs::write(&request_path, "{}\n").unwrap();
        append_event_once(
            dir.path(),
            WorkflowEventDraft {
                event_type: EVENT_TYPE_BUILD_AGENT_LAUNCH_REQUESTED.to_string(),
                source: "project-loop".to_string(),
                subject_id: "AF-001".to_string(),
                subject_path: Some(".agentflow/input/issues/AF-001.json".to_string()),
                dedupe_key: "build-agent.launch.requested:AF-001:run-001".to_string(),
                payload: serde_json::to_value(BuildAgentLaunchRequestedPayload {
                    issue_id: "AF-001".to_string(),
                    project_id: "proj-001".to_string(),
                    run_id: "run-001".to_string(),
                    branch_name: Some("agentflow/proj-001/AF-001".to_string()),
                    issue_path: ".agentflow/input/issues/AF-001.json".to_string(),
                    context_pack_path: ".agentflow/panel/context-packs/AF-001.json".to_string(),
                    launch_request_path:
                        ".agentflow/execute/runs/run-001/launcher/build-agent-request.json"
                            .to_string(),
                    display_status: "in_progress".to_string(),
                })
                .unwrap(),
            },
        )
        .unwrap();

        let claim = claim_next_build_agent_launch(dir.path())
            .unwrap()
            .expect("expected launch claim");
        assert_eq!(claim.issue_id, "AF-001");
        assert_eq!(claim.run_id, "run-001");
        assert_eq!(
            claim.branch_name.as_deref(),
            Some("agentflow/proj-001/AF-001")
        );
        assert_eq!(claim.launch_request_path, request_path);
        let state = load_build_agent_launch_state(dir.path(), "run-001").unwrap();
        assert_eq!(state.status, BuildAgentLaunchStatus::Claimed);
        assert_eq!(state.event_id.as_deref(), Some(claim.event_id.as_str()));

        let pending = load_pending_events(
            dir.path(),
            CONSUMER_BUILD_AGENT,
            &[EVENT_TYPE_BUILD_AGENT_LAUNCH_REQUESTED],
        )
        .unwrap();
        assert!(pending.is_empty());
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

        let outcome = complete_build_agent_issue_from_request(dir.path(), &request_path).unwrap();
        assert_eq!(outcome.completion.run.issue_id, "AF-001");
        let done_state = load_build_agent_launch_state(dir.path(), &launch.run_id).unwrap();
        assert_eq!(done_state.status, BuildAgentLaunchStatus::Done);

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
