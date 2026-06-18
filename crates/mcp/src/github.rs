use crate::{
    health::unix_timestamp_seconds,
    model::{McpCapability, McpProviderKind, McpProviderStatus, McpProviderStatusCode},
    provider::run_command,
};
use std::path::Path;

pub fn check_github_provider(project_root: impl AsRef<Path>) -> McpProviderStatus {
    let mut status = McpProviderStatus::new(McpProviderKind::Github, unix_timestamp_seconds());
    status.cli = Some("gh".to_string());

    match run_command(&project_root, "gh", &["--version"]) {
        Ok(version) if version.status_success => {
            status.installed = true;
        }
        Ok(version) => {
            status.status = McpProviderStatusCode::Unavailable;
            status.errors.push(format!(
                "gh --version failed: {}",
                version.combined_output().trim()
            ));
            status.capabilities = github_capabilities(false);
            return status;
        }
        Err(error) => {
            status.status = McpProviderStatusCode::Unavailable;
            status.errors.push(error.to_string());
            status.capabilities = github_capabilities(false);
            return status;
        }
    }

    match run_command(&project_root, "gh", &["auth", "status"]) {
        Ok(auth) if auth.status_success => {
            status.authenticated = Some(true);
        }
        Ok(auth) => {
            status.status = McpProviderStatusCode::Unauthenticated;
            status.authenticated = Some(false);
            status.errors.push(format!(
                "gh auth status failed: {}",
                auth.combined_output().trim()
            ));
            status.capabilities = github_capabilities(false);
            return status;
        }
        Err(error) => {
            status.status = McpProviderStatusCode::Unauthenticated;
            status.authenticated = Some(false);
            status.errors.push(error.to_string());
            status.capabilities = github_capabilities(false);
            return status;
        }
    }

    match run_command(
        &project_root,
        "gh",
        &[
            "repo",
            "view",
            "--json",
            "nameWithOwner,viewerPermission,defaultBranchRef",
        ],
    ) {
        Ok(repo) if repo.status_success => {
            status.repo_permission_checked = true;
            status.repo_permission = repo_permission(&repo.stdout);
            status.status = McpProviderStatusCode::Ready;
            status.capabilities = github_capabilities(true);
            status
        }
        Ok(repo) => {
            status.status = McpProviderStatusCode::PermissionDenied;
            status.repo_permission_checked = false;
            status.errors.push(format!(
                "gh repo view failed: {}",
                repo.combined_output().trim()
            ));
            status.capabilities = github_capabilities(false);
            status
        }
        Err(error) => {
            status.status = McpProviderStatusCode::PermissionDenied;
            status.errors.push(error.to_string());
            status.capabilities = github_capabilities(false);
            status
        }
    }
}

fn github_capabilities(available: bool) -> Vec<McpCapability> {
    [
        "repo.read",
        "pull_request.create",
        "pull_request.ready",
        "pull_request.auto_merge",
        "pull_request.merged_query",
        "issue.close",
        "issue.closed_query",
    ]
    .into_iter()
    .map(|name| McpCapability::new(name, available))
    .collect()
}

fn repo_permission(stdout: &str) -> Option<String> {
    serde_json::from_str::<serde_json::Value>(stdout)
        .ok()
        .and_then(|value| value.get("viewerPermission").cloned())
        .and_then(|value| value.as_str().map(ToString::to_string))
}

#[cfg(test)]
mod tests {
    use super::repo_permission;

    #[test]
    fn extracts_github_viewer_permission() {
        assert_eq!(
            repo_permission(r#"{"nameWithOwner":"atxinbao/agentflow","viewerPermission":"ADMIN"}"#)
                .as_deref(),
            Some("ADMIN")
        );
    }
}
