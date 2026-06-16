use crate::{
    health::unix_timestamp_seconds,
    model::{
        McpLaunchPlan, McpLaunchRequest, McpLogChunk, McpProviderKind, McpProviderStatus,
        McpSessionSnapshot, McpSessionStatus,
    },
    storage::{read_session_snapshot, write_launch_plan, write_session_snapshot},
};
use anyhow::{Context, Result};
use std::{
    collections::BTreeMap,
    path::Path,
    process::{Command, Output},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandProbe {
    pub status_success: bool,
    pub stdout: String,
    pub stderr: String,
}

impl CommandProbe {
    pub fn from_output(output: Output) -> Self {
        Self {
            status_success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        }
    }

    pub fn combined_output(&self) -> String {
        format!("{}{}", self.stdout, self.stderr)
    }
}

pub fn run_command(
    project_root: impl AsRef<Path>,
    program: &str,
    args: &[&str],
) -> Result<CommandProbe> {
    let output = Command::new(program)
        .args(args)
        .current_dir(project_root.as_ref())
        .output()
        .with_context(|| format!("run {} {}", program, args.join(" ")))?;
    Ok(CommandProbe::from_output(output))
}

pub trait McpAgentProvider: Send + Sync {
    fn provider_id(&self) -> &'static str;

    fn kind(&self) -> McpProviderKind;

    fn check_health(&self, project_root: &Path) -> McpProviderStatus;

    fn build_launch_plan(
        &self,
        project_root: &Path,
        request: &McpLaunchRequest,
    ) -> Result<McpLaunchPlan>;

    fn create_session(
        &self,
        project_root: &Path,
        request: &McpLaunchRequest,
    ) -> Result<McpSessionSnapshot> {
        let plan = self.build_launch_plan(project_root, request)?;
        let now = unix_timestamp_seconds();
        let session = McpSessionSnapshot::queued(request, &plan, now);
        write_launch_plan(project_root, &plan)?;
        write_session_snapshot(project_root, &session)?;
        Ok(session)
    }

    fn poll_session(&self, project_root: &Path, session_id: &str) -> Result<McpSessionSnapshot> {
        read_session_snapshot(project_root, session_id)
    }

    fn fetch_logs(
        &self,
        _project_root: &Path,
        session_id: &str,
        _cursor: Option<&str>,
    ) -> Result<McpLogChunk> {
        Ok(McpLogChunk::empty(self.provider_id(), session_id))
    }

    fn cancel_session(&self, project_root: &Path, session_id: &str) -> Result<McpSessionSnapshot> {
        let mut session = read_session_snapshot(project_root, session_id)?;
        session.status = McpSessionStatus::Cancelled;
        session.updated_at = unix_timestamp_seconds();
        write_session_snapshot(project_root, &session)?;
        Ok(session)
    }
}

#[derive(Default)]
pub struct McpProviderBridge {
    providers: BTreeMap<String, Box<dyn McpAgentProvider>>,
}

impl McpProviderBridge {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, provider: Box<dyn McpAgentProvider>) {
        self.providers
            .insert(provider.provider_id().to_string(), provider);
    }

    pub fn provider(&self, provider_id: &str) -> Option<&dyn McpAgentProvider> {
        self.providers.get(provider_id).map(Box::as_ref)
    }

    pub fn providers(&self) -> impl Iterator<Item = &dyn McpAgentProvider> {
        self.providers.values().map(Box::as_ref)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{McpCapability, McpLaunchMode, McpProviderStatusCode};
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
            status.capabilities = vec![McpCapability::new("launch", true)];
            status
        }

        fn build_launch_plan(
            &self,
            _project_root: &Path,
            request: &McpLaunchRequest,
        ) -> Result<McpLaunchPlan> {
            let mut plan = McpLaunchPlan::new(
                request.provider.clone(),
                "fake-run-001",
                request.issue_id.clone(),
                request.run_id.clone(),
                McpLaunchMode::CliExecPromptFile,
                request.working_directory.clone(),
                "fake-cli",
            );
            plan.stdin_path = Some(request.launch_request_path.clone());
            Ok(plan)
        }
    }

    #[test]
    fn provider_bridge_creates_and_cancels_session() {
        let dir = tempdir().unwrap();
        let provider = FakeProvider;
        let request = McpLaunchRequest::new(
            "fake",
            "AF-001",
            "run-001",
            "build-agent",
            dir.path().display().to_string(),
            ".agentflow/tasks/AF-001/runs/run-001/launch/agent-request.json",
        );
        let session = provider.create_session(dir.path(), &request).unwrap();
        assert_eq!(session.status, McpSessionStatus::Queued);
        assert_eq!(
            session.plan_path,
            ".agentflow/state/mcp/plans/fake-run-001.json"
        );

        let cancelled = provider
            .cancel_session(dir.path(), &session.session_id)
            .unwrap();
        assert_eq!(cancelled.status, McpSessionStatus::Cancelled);
    }
}
