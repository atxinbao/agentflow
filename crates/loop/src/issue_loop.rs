use crate::{
    model::{IssueLoopProjection, IssueLoopStage, LoopBlocker},
    storage::write_issue_loop_projection,
};
use agentflow_execute::{
    create_execute_run, execute_run_preflight,
    storage::{read_json, run_dir},
};
use agentflow_input::issue::InputIssueStatus;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssueLoop {
    project_id: String,
    issue_id: String,
}

impl IssueLoop {
    pub fn new(project_id: impl Into<String>, issue_id: impl Into<String>) -> Self {
        Self {
            project_id: project_id.into(),
            issue_id: issue_id.into(),
        }
    }

    pub fn project_id(&self) -> &str {
        &self.project_id
    }

    pub fn issue_id(&self) -> &str {
        &self.issue_id
    }

    pub fn projection(&self, updated_at: u64) -> IssueLoopProjection {
        IssueLoopProjection::new(self.project_id.clone(), self.issue_id.clone(), updated_at)
    }

    pub fn start_runtime_preflight(
        &self,
        project_root: impl AsRef<Path>,
    ) -> Result<IssueLoopProjection> {
        let root = canonical_project_root(project_root)?;
        let issue = agentflow_input::load_input_issue(&root, &self.issue_id)?;
        if !matches!(issue.status, InputIssueStatus::Todo) {
            anyhow::bail!(
                "Issue Loop can only start todo issues; {} is {}",
                issue.issue_id,
                issue.status.as_str()
            );
        }

        let run = create_execute_run(&root, self.issue_id.clone())?;
        let preflight = execute_run_preflight(&root, run.run_id.clone())?;
        let mut projection = self.projection(now());
        projection.run_id = Some(run.run_id.clone());
        projection.branch_name = branch_name(&root, &run.run_id).ok();

        if preflight.status == "ready" {
            projection.stage = IssueLoopStage::InProgress;
            agentflow_input::update_input_issue_status(
                &root,
                &self.issue_id,
                InputIssueStatus::InProgress,
            )?;
        } else {
            projection.stage = IssueLoopStage::Blocked;
            projection.blockers.push(LoopBlocker {
                code: "runtime-preflight-blocked".to_string(),
                reason: preflight
                    .blocked_reason
                    .unwrap_or_else(|| "Runtime preflight blocked.".to_string()),
                source_path: Some(format!(
                    ".agentflow/execute/runs/{}/preflight.json",
                    run.run_id
                )),
            });
            agentflow_input::update_input_issue_status(
                &root,
                &self.issue_id,
                InputIssueStatus::Blocked,
            )?;
        }

        write_issue_loop_projection(&root, &projection)?;
        Ok(projection)
    }

    pub fn write_merge_proof(
        &self,
        project_root: impl AsRef<Path>,
        run_id: &str,
        provider: &str,
        merge_mode: &str,
        remote_url: Option<String>,
        merged: bool,
    ) -> Result<PathBuf> {
        let root = canonical_project_root(project_root)?;
        let proof_path = run_dir(&root, run_id).join("review/merge-proof.json");
        agentflow_execute::storage::write_json(
            &proof_path,
            &serde_json::json!({
                "version": "execute-merge-proof.v1",
                "runId": run_id,
                "issueId": self.issue_id,
                "projectId": self.project_id,
                "provider": provider,
                "mergeMode": merge_mode,
                "remoteUrl": remote_url,
                "merged": merged,
                "checkedAt": now()
            }),
        )?;
        Ok(proof_path)
    }
}

fn branch_name(root: &Path, run_id: &str) -> Result<String> {
    let value: serde_json::Value = read_json(&run_dir(root, run_id).join("branch.json"))?;
    value
        .get("issueBranch")
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
        .context("branch.json is missing issueBranch")
}

fn canonical_project_root(project_root: impl AsRef<Path>) -> Result<PathBuf> {
    project_root
        .as_ref()
        .canonicalize()
        .with_context(|| format!("canonicalize {}", project_root.as_ref().display()))
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}
