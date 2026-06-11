use crate::{
    health::unix_timestamp_seconds,
    model::{McpCapability, McpProviderKind, McpProviderStatus, McpProviderStatusCode},
    provider::run_command,
};
use std::path::{Path, PathBuf};

pub fn check_codex_provider(project_root: impl AsRef<Path>) -> McpProviderStatus {
    let root = project_root.as_ref();
    let mut status = McpProviderStatus::new(McpProviderKind::Codex, unix_timestamp_seconds());

    for candidate in agentflow_cli_candidates(root) {
        let program = candidate.to_string_lossy().to_string();
        match run_command(root, &program, &["build-agent", "complete", "--help"]) {
            Ok(help) if help.status_success => {
                status.cli = Some(program);
                status.installed = true;
                status.status = McpProviderStatusCode::Ready;
                status.capabilities = vec![McpCapability::with_detail(
                    "build_agent.complete",
                    true,
                    "agentflow build-agent complete is available",
                )];
                return status;
            }
            Ok(help) => {
                status.warnings.push(format!(
                    "{} does not support build-agent complete: {}",
                    program,
                    help.combined_output().trim()
                ));
            }
            Err(error) => {
                status.warnings.push(format!("{program}: {error}"));
            }
        }
    }

    status.status = McpProviderStatusCode::Unavailable;
    status.capabilities = vec![McpCapability::with_detail(
        "build_agent.complete",
        false,
        "agentflow build-agent complete is unavailable",
    )];
    status
}

fn agentflow_cli_candidates(project_root: &Path) -> Vec<PathBuf> {
    vec![
        project_root.join("target/debug/agentflow"),
        project_root.join("target/release/agentflow"),
        PathBuf::from("agentflow"),
    ]
}

#[cfg(test)]
mod tests {
    use super::agentflow_cli_candidates;
    use std::path::Path;

    #[test]
    fn prefers_debug_agentflow_before_release_binary() {
        let candidates = agentflow_cli_candidates(Path::new("/repo"));
        assert_eq!(candidates[0], Path::new("/repo/target/debug/agentflow"));
        assert_eq!(candidates[1], Path::new("/repo/target/release/agentflow"));
    }
}
