use crate::{
    health::unix_timestamp_seconds,
    model::{McpCapability, McpProviderKind, McpProviderStatus, McpProviderStatusCode},
    provider::run_command,
};
use std::path::Path;

pub fn check_gitlab_provider(project_root: impl AsRef<Path>) -> McpProviderStatus {
    let mut status = McpProviderStatus::new(McpProviderKind::Gitlab, unix_timestamp_seconds());
    status.cli = Some("glab".to_string());

    match run_command(&project_root, "glab", &["--version"]) {
        Ok(version) if version.status_success => {
            status.installed = true;
        }
        Ok(version) => {
            status.status = McpProviderStatusCode::Unavailable;
            status.errors.push(format!(
                "glab --version failed: {}",
                version.combined_output().trim()
            ));
            status.capabilities = gitlab_capabilities(false);
            return status;
        }
        Err(error) => {
            status.status = McpProviderStatusCode::Unavailable;
            status.errors.push(error.to_string());
            status.capabilities = gitlab_capabilities(false);
            return status;
        }
    }

    match run_command(&project_root, "glab", &["auth", "status"]) {
        Ok(auth) if auth.status_success => {
            status.authenticated = Some(true);
        }
        Ok(auth) => {
            status.status = McpProviderStatusCode::Unauthenticated;
            status.authenticated = Some(false);
            status.errors.push(format!(
                "glab auth status failed: {}",
                auth.combined_output().trim()
            ));
            status.capabilities = gitlab_capabilities(false);
            return status;
        }
        Err(error) => {
            status.status = McpProviderStatusCode::Unauthenticated;
            status.authenticated = Some(false);
            status.errors.push(error.to_string());
            status.capabilities = gitlab_capabilities(false);
            return status;
        }
    }

    match run_command(&project_root, "glab", &["repo", "view"]) {
        Ok(repo) if repo.status_success => {
            status.repo_permission_checked = true;
            status.status = McpProviderStatusCode::Ready;
            status.capabilities = gitlab_capabilities(true);
            status
        }
        Ok(repo) => {
            status.status = McpProviderStatusCode::PermissionDenied;
            status.errors.push(format!(
                "glab repo view failed: {}",
                repo.combined_output().trim()
            ));
            status.capabilities = gitlab_capabilities(false);
            status
        }
        Err(error) => {
            status.status = McpProviderStatusCode::PermissionDenied;
            status.errors.push(error.to_string());
            status.capabilities = gitlab_capabilities(false);
            status
        }
    }
}

fn gitlab_capabilities(available: bool) -> Vec<McpCapability> {
    [
        "repo.read",
        "merge_request.create",
        "merge_request.ready",
        "merge_request.auto_merge",
        "merge_request.merged_query",
    ]
    .into_iter()
    .map(|name| McpCapability::new(name, available))
    .collect()
}
