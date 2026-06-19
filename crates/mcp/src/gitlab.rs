use crate::{
    health::unix_timestamp_seconds,
    model::{
        McpCapability, McpCloseoutAttestation, McpCloseoutIssueAttestation, McpProviderKind,
        McpProviderStatus, McpProviderStatusCode, MCP_CLOSEOUT_ATTESTATION_VERSION,
    },
    provider::run_command,
};
use anyhow::{Context, Result};
use serde_json::Value;
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
        "issue.close",
        "issue.closed_query",
    ]
    .into_iter()
    .map(|name| McpCapability::new(name, available))
    .collect()
}

pub fn query_gitlab_closeout_attestation(
    project_root: impl AsRef<Path>,
    review_ref: &str,
    issue_refs: &[String],
) -> Result<McpCloseoutAttestation> {
    if issue_refs.is_empty() {
        anyhow::bail!("gitlab closeout query requires explicit issue refs");
    }
    let mr = run_command(
        &project_root,
        "glab",
        &["mr", "view", review_ref, "--output", "json"],
    )?;
    if !mr.status_success {
        anyhow::bail!("glab mr view failed: {}", mr.combined_output().trim());
    }
    let mr_value: Value = serde_json::from_str(&mr.stdout).context("parse glab mr view json")?;
    let review_url = mr_value
        .get("web_url")
        .or_else(|| mr_value.get("webUrl"))
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let repository_full_name = mr_value
        .get("references")
        .and_then(|value| value.get("full"))
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .or_else(|| {
            mr_value
                .get("project")
                .and_then(|value| value.get("full_name"))
                .or_else(|| {
                    mr_value
                        .get("project")
                        .and_then(|value| value.get("fullPath"))
                })
                .and_then(Value::as_str)
                .map(ToString::to_string)
        });
    let source_branch = mr_value
        .get("source_branch")
        .or_else(|| mr_value.get("sourceBranch"))
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let target_branch = mr_value
        .get("target_branch")
        .or_else(|| mr_value.get("targetBranch"))
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let base_sha = mr_value
        .get("diff_refs")
        .and_then(|value| value.get("base_sha"))
        .or_else(|| {
            mr_value
                .get("diffRefs")
                .and_then(|value| value.get("baseSha"))
        })
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let head_sha = mr_value
        .get("sha")
        .or_else(|| mr_value.get("headSha"))
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let merge_commit_sha = mr_value
        .get("merge_commit_sha")
        .or_else(|| mr_value.get("mergeCommitSha"))
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let merged_at = mr_value
        .get("merged_at")
        .or_else(|| mr_value.get("mergedAt"))
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let merged = merged_at.is_some()
        || mr_value
            .get("state")
            .and_then(Value::as_str)
            .map(|state| state.eq_ignore_ascii_case("merged"))
            .unwrap_or(false);
    let issues = issue_refs
        .iter()
        .map(|issue_ref| query_gitlab_issue_status(&project_root, issue_ref))
        .collect::<Result<Vec<_>>>()?;
    let issue_closed = !issues.is_empty() && issues.iter().all(|issue| issue.closed);
    Ok(McpCloseoutAttestation {
        version: MCP_CLOSEOUT_ATTESTATION_VERSION.to_string(),
        provider: McpProviderKind::Gitlab.as_str().to_string(),
        review_ref: review_ref.to_string(),
        review_url,
        repository_full_name,
        source_branch,
        target_branch,
        base_sha,
        head_sha,
        merge_commit_sha,
        merged,
        merged_at,
        issue_closed,
        issues,
        queried_at: unix_timestamp_seconds(),
    })
}

fn query_gitlab_issue_status(
    project_root: impl AsRef<Path>,
    issue_ref: &str,
) -> Result<McpCloseoutIssueAttestation> {
    let issue = run_command(
        &project_root,
        "glab",
        &["issue", "view", issue_ref, "--output", "json"],
    )?;
    if !issue.status_success {
        anyhow::bail!(
            "glab issue view {} failed: {}",
            issue_ref,
            issue.combined_output().trim()
        );
    }
    parse_gitlab_issue_status(issue_ref, &issue.stdout)
}

fn parse_gitlab_issue_status(
    issue_ref: &str,
    issue_stdout: &str,
) -> Result<McpCloseoutIssueAttestation> {
    let issue: Value = serde_json::from_str(issue_stdout).context("parse glab issue view json")?;
    let closed = issue
        .get("closed")
        .and_then(Value::as_bool)
        .or_else(|| {
            issue
                .get("state")
                .and_then(Value::as_str)
                .map(|state| state.eq_ignore_ascii_case("closed"))
        })
        .unwrap_or(false);
    Ok(McpCloseoutIssueAttestation {
        issue_ref: issue_ref.to_string(),
        issue_url: issue
            .get("web_url")
            .or_else(|| issue.get("webUrl"))
            .and_then(Value::as_str)
            .map(ToString::to_string),
        closed,
        closed_at: issue
            .get("closed_at")
            .or_else(|| issue.get("closedAt"))
            .and_then(Value::as_str)
            .map(ToString::to_string),
    })
}

#[cfg(test)]
mod tests {
    use super::parse_gitlab_issue_status;

    #[test]
    fn parses_gitlab_issue_status_payload() {
        let issue = parse_gitlab_issue_status(
            "58",
            r#"{
              "state": "closed",
              "closed_at": "2026-06-19T12:00:00Z",
              "web_url": "https://gitlab.example.com/acme/repo/-/issues/58"
            }"#,
        )
        .unwrap();
        assert!(issue.closed);
        assert_eq!(issue.issue_ref, "58");
        assert_eq!(
            issue.issue_url.as_deref(),
            Some("https://gitlab.example.com/acme/repo/-/issues/58")
        );
        assert_eq!(issue.closed_at.as_deref(), Some("2026-06-19T12:00:00Z"));
    }
}
