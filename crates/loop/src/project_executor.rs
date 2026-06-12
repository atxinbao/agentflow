use crate::{IssueLoop, IssueLoopStage, ProjectLoop, ProjectLoopSnapshot};
use agentflow_execute::{
    ensure_build_agent_launch_state,
    storage::{relative_path, run_dir, write_json},
};
use agentflow_input::issue::{InputIssue, InputIssueStatus};
use agentflow_workflow_events::{
    append_event_once, BuildAgentLaunchRequestedPayload, WorkflowEventDraft,
    EVENT_TYPE_BUILD_AGENT_LAUNCH_REQUESTED,
};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectExecutionLaunch {
    pub issue_id: String,
    pub run_id: String,
    pub branch_name: Option<String>,
    pub stage: IssueLoopStage,
    pub launch_request_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectExecutionTick {
    pub snapshot: ProjectLoopSnapshot,
    pub launch: Option<ProjectExecutionLaunch>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RuntimeAdvance {
    launch: Option<ProjectExecutionLaunch>,
    state_changed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectExecutor {
    project_id: String,
}

impl ProjectExecutor {
    pub fn new(project_id: impl Into<String>) -> Self {
        Self {
            project_id: project_id.into(),
        }
    }

    pub fn project_id(&self) -> &str {
        &self.project_id
    }

    pub fn tick(&self, project_root: impl AsRef<Path>) -> Result<ProjectExecutionTick> {
        let root = canonical_project_root(project_root)?;
        let loop_driver = ProjectLoop::new(self.project_id.clone());
        loop_driver.run_preflight(&root)?;
        let mut snapshot = loop_driver.schedule_ready_issues(&root)?;
        let runtime = self.maybe_start_runtime(&root)?;
        if runtime.state_changed {
            snapshot = loop_driver.schedule_ready_issues(&root)?;
        }
        Ok(ProjectExecutionTick {
            snapshot,
            launch: runtime.launch,
        })
    }

    fn maybe_start_runtime(&self, root: &Path) -> Result<RuntimeAdvance> {
        let workspace = agentflow_input::prepare_input_workspace(root)?;
        let project = workspace
            .projects
            .iter()
            .find(|project| project.project_id == self.project_id)
            .with_context(|| format!("project {} not found", self.project_id))?;
        let issues_by_id = workspace
            .issues
            .iter()
            .map(|issue| (issue.issue_id.as_str(), issue))
            .collect::<BTreeMap<_, _>>();

        if let Some(active_issue) = active_runtime_issue(project, &issues_by_id) {
            let restored_launch_request = ensure_existing_launch_request(root, active_issue)?;
            return Ok(RuntimeAdvance {
                launch: None,
                state_changed: restored_launch_request,
            });
        }

        let Some(issue) = next_todo_issue(project.issue_ids.iter(), &issues_by_id) else {
            return Ok(RuntimeAdvance {
                launch: None,
                state_changed: false,
            });
        };
        let projection =
            match IssueLoop::new(&self.project_id, &issue.issue_id).start_runtime_preflight(root) {
                Ok(projection) => projection,
                Err(error) => {
                    let refreshed_issue = agentflow_input::load_input_issue(root, &issue.issue_id)
                        .with_context(|| format!("reload input issue {}", issue.issue_id))?;
                    if matches!(refreshed_issue.status, InputIssueStatus::Blocked)
                        && refreshed_issue.latest_run_id.is_some()
                    {
                        return Ok(RuntimeAdvance {
                            launch: None,
                            state_changed: true,
                        });
                    }
                    return Err(error);
                }
            };
        if !matches!(projection.stage, IssueLoopStage::InProgress) {
            return Ok(RuntimeAdvance {
                launch: None,
                state_changed: true,
            });
        }
        let run_id = projection
            .run_id
            .clone()
            .ok_or_else(|| anyhow::anyhow!("issue runtime start did not produce runId"))?;
        let issue = agentflow_input::load_input_issue(root, &issue.issue_id)
            .with_context(|| format!("reload input issue {}", issue.issue_id))?;
        let launch_request_path = write_build_agent_launch_request(
            root,
            &issue,
            &run_id,
            projection.branch_name.clone(),
        )?;
        append_build_agent_launch_event(
            root,
            &issue,
            &run_id,
            projection.branch_name.clone(),
            &launch_request_path,
        )?;
        Ok(RuntimeAdvance {
            launch: Some(ProjectExecutionLaunch {
                issue_id: issue.issue_id.clone(),
                run_id,
                branch_name: projection.branch_name.clone(),
                stage: projection.stage,
                launch_request_path,
            }),
            state_changed: true,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BuildAgentRuntimeState {
    entrypoint: String,
    run_id: String,
    branch_name: Option<String>,
    issue_status: String,
    note: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BuildAgentReviewInstruction {
    cli: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BuildAgentLaunchRequest {
    version: String,
    launch_source: String,
    issue_id: String,
    project_id: String,
    run_id: String,
    branch_name: Option<String>,
    issue_path: String,
    input_issue_revision: u64,
    input_issue: InputIssue,
    runtime: BuildAgentRuntimeState,
    review_preparation: BuildAgentReviewInstruction,
    merge_proof_writeback: BuildAgentReviewInstruction,
    completion_writeback: BuildAgentReviewInstruction,
    generated_at: u64,
}

fn write_build_agent_launch_request(
    root: &Path,
    issue: &InputIssue,
    run_id: &str,
    branch_name: Option<String>,
) -> Result<String> {
    let project_id = issue
        .project_id
        .clone()
        .ok_or_else(|| anyhow::anyhow!("project issue {} is missing projectId", issue.issue_id))?;
    let request = BuildAgentLaunchRequest {
        version: "build-agent-launch-request.v1".to_string(),
        launch_source: "project-loop".to_string(),
        issue_id: issue.issue_id.clone(),
        project_id,
        run_id: run_id.to_string(),
        branch_name: branch_name.clone(),
        issue_path: issue.issue_path.clone(),
        input_issue_revision: issue.system.revision,
        input_issue: issue.clone(),
        runtime: BuildAgentRuntimeState {
            entrypoint: "project-loop".to_string(),
            run_id: run_id.to_string(),
            branch_name,
            issue_status: issue.status.as_str().to_string(),
            note: "current run is already created by AgentFlow project loop; Build Agent must not call build-agent start again".to_string(),
        },
        review_preparation: BuildAgentReviewInstruction {
            cli: "agentflow build-agent prepare-review --request <completion-request.json>"
                .to_string(),
        },
        merge_proof_writeback: BuildAgentReviewInstruction {
            cli: "agentflow build-agent write-merge-proof --issue-id <issue-id> --run-id <run-id> --provider <github|gitlab> --merge-mode <auto-merge-if-eligible|manual-merge> [--remote-url <url>] [--merged]"
                .to_string(),
        },
        completion_writeback: BuildAgentReviewInstruction {
            cli: "agentflow build-agent complete --request <completion-request.json>".to_string(),
        },
        generated_at: now(),
    };
    let path = run_dir(root, run_id).join("launcher/build-agent-request.json");
    write_json(&path, &request)?;
    let relative_path = relative_path(root, &path);
    ensure_build_agent_launch_state(
        root,
        &issue.issue_id,
        issue.project_id.as_deref(),
        run_id,
        request.runtime.branch_name.clone(),
        relative_path.clone(),
    )?;
    Ok(relative_path)
}

fn append_build_agent_launch_event(
    root: &Path,
    issue: &InputIssue,
    run_id: &str,
    branch_name: Option<String>,
    launch_request_path: &str,
) -> Result<()> {
    let payload = BuildAgentLaunchRequestedPayload {
        issue_id: issue.issue_id.clone(),
        project_id: issue.project_id.clone().ok_or_else(|| {
            anyhow::anyhow!("project issue {} is missing projectId", issue.issue_id)
        })?,
        run_id: run_id.to_string(),
        branch_name,
        issue_path: issue.issue_path.clone(),
        context_pack_path: issue.context_pack_path.clone(),
        launch_request_path: launch_request_path.to_string(),
        display_status: issue.display_status.as_str().to_string(),
    };
    append_event_once(
        root,
        WorkflowEventDraft {
            event_type: EVENT_TYPE_BUILD_AGENT_LAUNCH_REQUESTED.to_string(),
            source: "project-loop".to_string(),
            subject_id: issue.issue_id.clone(),
            subject_path: Some(issue.issue_path.clone()),
            dedupe_key: format!("build-agent.launch.requested:{}:{run_id}", issue.issue_id),
            payload: serde_json::to_value(payload)?,
        },
    )?;
    Ok(())
}

fn ensure_existing_launch_request(root: &Path, issue: &InputIssue) -> Result<bool> {
    let run_id = issue
        .latest_run_id
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("active issue {} is missing latestRunId", issue.issue_id))?;
    let branch_name = branch_name_from_run(root, run_id).ok();
    let launch_request_path = run_dir(root, run_id).join("launcher/build-agent-request.json");
    let (relative_launch_request_path, restored_launch_request) = if launch_request_path.is_file() {
        (relative_path(root, &launch_request_path), false)
    } else {
        (
            write_build_agent_launch_request(root, issue, run_id, branch_name.clone())?,
            true,
        )
    };
    append_build_agent_launch_event(
        root,
        issue,
        run_id,
        branch_name,
        &relative_launch_request_path,
    )?;
    Ok(restored_launch_request)
}

fn active_runtime_issue<'a>(
    project: &agentflow_input::project::InputProject,
    issues_by_id: &BTreeMap<&'a str, &'a InputIssue>,
) -> Option<&'a InputIssue> {
    project
        .issue_ids
        .iter()
        .filter_map(|issue_id| issues_by_id.get(issue_id.as_str()).copied())
        .find(|issue| {
            matches!(
                issue.status,
                InputIssueStatus::InProgress | InputIssueStatus::InReview
            ) && issue.latest_run_id.is_some()
        })
}

fn next_todo_issue<'a>(
    issue_ids: impl Iterator<Item = &'a String>,
    issues_by_id: &BTreeMap<&'a str, &'a InputIssue>,
) -> Option<&'a InputIssue> {
    issue_ids
        .filter_map(|issue_id| issues_by_id.get(issue_id.as_str()).copied())
        .find(|issue| matches!(issue.status, InputIssueStatus::Todo))
}

fn canonical_project_root(project_root: impl AsRef<Path>) -> Result<PathBuf> {
    project_root
        .as_ref()
        .canonicalize()
        .with_context(|| format!("canonicalize {}", project_root.as_ref().display()))
}

fn branch_name_from_run(root: &Path, run_id: &str) -> Result<String> {
    let value: serde_json::Value =
        agentflow_execute::storage::read_json(&run_dir(root, run_id).join("branch.json"))?;
    value
        .get("issueBranch")
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
        .context("branch.json is missing issueBranch")
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}
