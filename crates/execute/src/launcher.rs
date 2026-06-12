use crate::{
    model::{
        BuildAgentLaunchState, BuildAgentLaunchStatus, EXECUTE_BUILD_AGENT_LAUNCH_STATE_VERSION,
    },
    storage::{canonical_project_root, read_json, run_dir, unix_timestamp_seconds, write_json},
};
use anyhow::Result;
use std::path::{Path, PathBuf};

pub fn load_build_agent_launch_state(
    project_root: impl AsRef<Path>,
    run_id: &str,
) -> Result<BuildAgentLaunchState> {
    let root = canonical_project_root(project_root)?;
    read_json(&build_agent_launch_state_path(&root, run_id))
}

pub fn ensure_build_agent_launch_state(
    project_root: impl AsRef<Path>,
    issue_id: &str,
    project_id: Option<&str>,
    run_id: &str,
    branch_name: Option<String>,
    launch_request_path: String,
) -> Result<BuildAgentLaunchState> {
    let root = canonical_project_root(project_root)?;
    let path = build_agent_launch_state_path(&root, run_id);
    let now = unix_timestamp_seconds();
    let mut state = if path.is_file() {
        read_json::<BuildAgentLaunchState>(&path)?
    } else {
        BuildAgentLaunchState {
            version: EXECUTE_BUILD_AGENT_LAUNCH_STATE_VERSION.to_string(),
            issue_id: issue_id.to_string(),
            project_id: project_id.map(str::to_string),
            run_id: run_id.to_string(),
            branch_name: branch_name.clone(),
            launch_request_path: launch_request_path.clone(),
            status: BuildAgentLaunchStatus::Queued,
            event_id: None,
            claimed_at: None,
            review_prepared_at: None,
            completed_at: None,
            updated_at: now,
        }
    };
    state.version = EXECUTE_BUILD_AGENT_LAUNCH_STATE_VERSION.to_string();
    state.issue_id = issue_id.to_string();
    state.project_id = project_id.map(str::to_string);
    state.run_id = run_id.to_string();
    state.branch_name = branch_name;
    state.launch_request_path = launch_request_path;
    state.updated_at = now;
    write_json(&path, &state)?;
    Ok(state)
}

pub fn claim_build_agent_launch(
    project_root: impl AsRef<Path>,
    issue_id: &str,
    project_id: Option<&str>,
    run_id: &str,
    branch_name: Option<String>,
    launch_request_path: String,
    event_id: String,
) -> Result<BuildAgentLaunchState> {
    let root = canonical_project_root(project_root)?;
    let mut state = ensure_build_agent_launch_state(
        &root,
        issue_id,
        project_id,
        run_id,
        branch_name,
        launch_request_path,
    )?;
    let now = unix_timestamp_seconds();
    state.status = BuildAgentLaunchStatus::Claimed;
    state.event_id = Some(event_id);
    if state.claimed_at.is_none() {
        state.claimed_at = Some(now);
    }
    state.updated_at = now;
    write_json(&build_agent_launch_state_path(&root, run_id), &state)?;
    Ok(state)
}

pub fn mark_build_agent_launch_in_review(
    project_root: impl AsRef<Path>,
    run_id: &str,
) -> Result<Option<BuildAgentLaunchState>> {
    update_build_agent_launch_state_if_exists(project_root, run_id, |state, now| {
        state.status = BuildAgentLaunchStatus::InReview;
        if state.review_prepared_at.is_none() {
            state.review_prepared_at = Some(now);
        }
    })
}

pub fn mark_build_agent_launch_done(
    project_root: impl AsRef<Path>,
    run_id: &str,
) -> Result<Option<BuildAgentLaunchState>> {
    update_build_agent_launch_state_if_exists(project_root, run_id, |state, now| {
        state.status = BuildAgentLaunchStatus::Done;
        if state.completed_at.is_none() {
            state.completed_at = Some(now);
        }
    })
}

pub fn build_agent_launch_state_path(root: &Path, run_id: &str) -> PathBuf {
    run_dir(root, run_id).join("launcher/worker-state.json")
}

fn update_build_agent_launch_state_if_exists(
    project_root: impl AsRef<Path>,
    run_id: &str,
    mut mutate: impl FnMut(&mut BuildAgentLaunchState, u64),
) -> Result<Option<BuildAgentLaunchState>> {
    let root = canonical_project_root(project_root)?;
    let path = build_agent_launch_state_path(&root, run_id);
    if !path.is_file() {
        return Ok(None);
    }
    let mut state: BuildAgentLaunchState = read_json(&path)?;
    let now = unix_timestamp_seconds();
    mutate(&mut state, now);
    state.updated_at = now;
    write_json(&path, &state)?;
    Ok(Some(state))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn launcher_state_moves_from_queued_to_done() {
        let dir = tempdir().unwrap();

        let queued = ensure_build_agent_launch_state(
            dir.path(),
            "AF-001",
            Some("proj-001"),
            "run-001",
            Some("agentflow/proj-001/AF-001".to_string()),
            ".agentflow/execute/runs/run-001/launcher/build-agent-request.json".to_string(),
        )
        .unwrap();
        assert_eq!(queued.status, BuildAgentLaunchStatus::Queued);

        let claimed = claim_build_agent_launch(
            dir.path(),
            "AF-001",
            Some("proj-001"),
            "run-001",
            Some("agentflow/proj-001/AF-001".to_string()),
            ".agentflow/execute/runs/run-001/launcher/build-agent-request.json".to_string(),
            "event-001".to_string(),
        )
        .unwrap();
        assert_eq!(claimed.status, BuildAgentLaunchStatus::Claimed);
        assert_eq!(claimed.event_id.as_deref(), Some("event-001"));
        assert!(claimed.claimed_at.is_some());

        let review = mark_build_agent_launch_in_review(dir.path(), "run-001")
            .unwrap()
            .expect("review state");
        assert_eq!(review.status, BuildAgentLaunchStatus::InReview);
        assert!(review.review_prepared_at.is_some());

        let done = mark_build_agent_launch_done(dir.path(), "run-001")
            .unwrap()
            .expect("done state");
        assert_eq!(done.status, BuildAgentLaunchStatus::Done);
        assert!(done.completed_at.is_some());

        let loaded = load_build_agent_launch_state(dir.path(), "run-001").unwrap();
        assert_eq!(loaded.status, BuildAgentLaunchStatus::Done);
    }
}
