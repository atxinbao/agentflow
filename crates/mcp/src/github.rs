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

pub fn query_github_closeout_attestation(
    project_root: impl AsRef<Path>,
    review_ref: &str,
    issue_refs: &[String],
) -> Result<McpCloseoutAttestation> {
    let pr = run_command(
        &project_root,
        "gh",
        &[
            "pr",
            "view",
            review_ref,
            "--json",
            "url,mergedAt,closingIssuesReferences,headRefName,headRefOid,baseRefName,baseRefOid,mergeCommit,repository",
        ],
    )?;
    if !pr.status_success {
        anyhow::bail!("gh pr view failed: {}", pr.combined_output().trim());
    }
    parse_github_closeout_attestation(&project_root, review_ref, &pr.stdout, issue_refs)
}

fn parse_github_closeout_attestation(
    project_root: impl AsRef<Path>,
    review_ref: &str,
    pr_stdout: &str,
    issue_refs: &[String],
) -> Result<McpCloseoutAttestation> {
    let pr: Value = serde_json::from_str(pr_stdout).context("parse gh pr view json")?;
    let review_url = pr
        .get("url")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let repository_full_name = pr
        .get("repository")
        .and_then(|value| value.get("nameWithOwner"))
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let source_branch = pr
        .get("headRefName")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let target_branch = pr
        .get("baseRefName")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let head_sha = pr
        .get("headRefOid")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let base_sha = pr
        .get("baseRefOid")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let merge_commit_sha = pr
        .get("mergeCommit")
        .and_then(|value| value.get("oid"))
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let merged_at = pr
        .get("mergedAt")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let resolved_issue_refs = if issue_refs.is_empty() {
        infer_github_issue_refs(&pr)
    } else {
        issue_refs.to_vec()
    };
    if resolved_issue_refs.is_empty() {
        anyhow::bail!("github closeout query requires explicit issue refs or PR closing issues");
    }
    let issues = resolved_issue_refs
        .iter()
        .map(|issue_ref| query_github_issue_status(&project_root, issue_ref))
        .collect::<Result<Vec<_>>>()?;
    Ok(build_github_closeout_attestation(
        review_ref,
        review_url,
        repository_full_name,
        source_branch,
        target_branch,
        base_sha,
        head_sha,
        merge_commit_sha,
        merged_at,
        issues,
    ))
}

fn build_github_closeout_attestation(
    review_ref: &str,
    review_url: Option<String>,
    repository_full_name: Option<String>,
    source_branch: Option<String>,
    target_branch: Option<String>,
    base_sha: Option<String>,
    head_sha: Option<String>,
    merge_commit_sha: Option<String>,
    merged_at: Option<String>,
    issues: Vec<McpCloseoutIssueAttestation>,
) -> McpCloseoutAttestation {
    let issue_closed = !issues.is_empty() && issues.iter().all(|issue| issue.closed);
    McpCloseoutAttestation {
        version: MCP_CLOSEOUT_ATTESTATION_VERSION.to_string(),
        provider: McpProviderKind::Github.as_str().to_string(),
        review_ref: review_ref.to_string(),
        review_url,
        repository_full_name,
        source_branch,
        target_branch,
        base_sha,
        head_sha,
        merge_commit_sha,
        merged: merged_at.is_some(),
        merged_at,
        issue_closed,
        issues,
        queried_at: unix_timestamp_seconds(),
    }
}

fn infer_github_issue_refs(pr: &Value) -> Vec<String> {
    pr.get("closingIssuesReferences")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|issue| {
            issue
                .get("number")
                .and_then(Value::as_u64)
                .map(|number| number.to_string())
                .or_else(|| {
                    issue
                        .get("url")
                        .and_then(Value::as_str)
                        .and_then(|url| url.rsplit('/').next())
                        .map(ToString::to_string)
                })
        })
        .collect()
}

fn query_github_issue_status(
    project_root: impl AsRef<Path>,
    issue_ref: &str,
) -> Result<McpCloseoutIssueAttestation> {
    let issue = run_command(
        &project_root,
        "gh",
        &[
            "issue",
            "view",
            issue_ref,
            "--json",
            "state,closed,closedAt,url",
        ],
    )?;
    if !issue.status_success {
        anyhow::bail!(
            "gh issue view {} failed: {}",
            issue_ref,
            issue.combined_output().trim()
        );
    }
    parse_github_issue_status(issue_ref, &issue.stdout)
}

fn parse_github_issue_status(
    issue_ref: &str,
    issue_stdout: &str,
) -> Result<McpCloseoutIssueAttestation> {
    let issue: Value = serde_json::from_str(issue_stdout).context("parse gh issue view json")?;
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
            .get("url")
            .and_then(Value::as_str)
            .map(ToString::to_string),
        closed,
        closed_at: issue
            .get("closedAt")
            .and_then(Value::as_str)
            .map(ToString::to_string),
    })
}

fn repo_permission(stdout: &str) -> Option<String> {
    serde_json::from_str::<serde_json::Value>(stdout)
        .ok()
        .and_then(|value| value.get("viewerPermission").cloned())
        .and_then(|value| value.as_str().map(ToString::to_string))
}

#[cfg(test)]
mod tests {
    use super::{
        build_github_closeout_attestation, infer_github_issue_refs,
        parse_github_closeout_attestation, parse_github_issue_status, repo_permission,
    };

    #[test]
    fn extracts_github_viewer_permission() {
        assert_eq!(
            repo_permission(r#"{"nameWithOwner":"atxinbao/agentflow","viewerPermission":"ADMIN"}"#)
                .as_deref(),
            Some("ADMIN")
        );
    }

    #[test]
    fn infers_issue_refs_from_pr_payload() {
        let pr: serde_json::Value = serde_json::from_str(
            r#"{
              "closingIssuesReferences": [
                { "number": 208, "url": "https://github.com/acme/repo/issues/208" },
                { "number": 209, "url": "https://github.com/acme/repo/issues/209" }
              ]
            }"#,
        )
        .unwrap();
        assert_eq!(infer_github_issue_refs(&pr), vec!["208", "209"]);
    }

    #[test]
    fn parses_github_issue_status_payload() {
        let issue = parse_github_issue_status(
            "208",
            r#"{
              "state": "CLOSED",
              "closed": true,
              "closedAt": "2026-06-19T11:22:33Z",
              "url": "https://github.com/acme/repo/issues/208"
            }"#,
        )
        .unwrap();
        assert!(issue.closed);
        assert_eq!(issue.issue_ref, "208");
        assert_eq!(
            issue.issue_url.as_deref(),
            Some("https://github.com/acme/repo/issues/208")
        );
        assert_eq!(issue.closed_at.as_deref(), Some("2026-06-19T11:22:33Z"));
    }

    #[test]
    fn parses_github_closeout_attestation_from_pr_payload() {
        let dir = tempfile::tempdir().unwrap();
        let err = parse_github_closeout_attestation(
            dir.path(),
            "208",
            r#"{
              "url": "https://github.com/acme/repo/pull/208",
              "mergedAt": "2026-06-19T11:20:00Z",
              "headRefName": "agentflow/direct/AF-001",
              "headRefOid": "head-001",
              "baseRefName": "main",
              "baseRefOid": "base-001",
              "mergeCommit": { "oid": "merge-001" },
              "repository": { "nameWithOwner": "acme/repo" },
              "closingIssuesReferences": []
            }"#,
            &[],
        )
        .unwrap_err();
        assert!(err
            .to_string()
            .contains("github closeout query requires explicit issue refs or PR closing issues"));
    }

    #[test]
    fn parses_github_closeout_attestation_v3_fields() {
        let pr: serde_json::Value = serde_json::from_str(
            r#"{
              "url": "https://github.com/acme/repo/pull/208",
              "mergedAt": "2026-06-19T11:20:00Z",
              "headRefName": "agentflow/direct/AF-001",
              "headRefOid": "head-001",
              "baseRefName": "main",
              "baseRefOid": "base-001",
              "mergeCommit": { "oid": "merge-001" },
              "repository": { "nameWithOwner": "acme/repo" }
            }"#,
        )
        .unwrap();
        let attestation = build_github_closeout_attestation(
            "208",
            pr.get("url")
                .and_then(serde_json::Value::as_str)
                .map(ToString::to_string),
            pr.get("repository")
                .and_then(|value| value.get("nameWithOwner"))
                .and_then(serde_json::Value::as_str)
                .map(ToString::to_string),
            pr.get("headRefName")
                .and_then(serde_json::Value::as_str)
                .map(ToString::to_string),
            pr.get("baseRefName")
                .and_then(serde_json::Value::as_str)
                .map(ToString::to_string),
            pr.get("baseRefOid")
                .and_then(serde_json::Value::as_str)
                .map(ToString::to_string),
            pr.get("headRefOid")
                .and_then(serde_json::Value::as_str)
                .map(ToString::to_string),
            pr.get("mergeCommit")
                .and_then(|value| value.get("oid"))
                .and_then(serde_json::Value::as_str)
                .map(ToString::to_string),
            pr.get("mergedAt")
                .and_then(serde_json::Value::as_str)
                .map(ToString::to_string),
            vec![super::McpCloseoutIssueAttestation {
                issue_ref: "208".to_string(),
                issue_url: Some("https://github.com/acme/repo/issues/208".to_string()),
                closed: true,
                closed_at: Some("2026-06-19T11:22:33Z".to_string()),
            }],
        );
        assert_eq!(
            attestation.repository_full_name.as_deref(),
            Some("acme/repo")
        );
        assert_eq!(
            attestation.source_branch.as_deref(),
            Some("agentflow/direct/AF-001")
        );
        assert_eq!(attestation.target_branch.as_deref(), Some("main"));
        assert_eq!(attestation.base_sha.as_deref(), Some("base-001"));
        assert_eq!(attestation.head_sha.as_deref(), Some("head-001"));
        assert_eq!(attestation.merge_commit_sha.as_deref(), Some("merge-001"));
    }
}
