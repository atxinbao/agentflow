use agentflow_projection::IssueStatusIndex;
use agentflow_spec::SpecIssue;
use std::{collections::BTreeMap, path::Path};

#[derive(Debug, Clone)]
pub(crate) struct IssueReadinessBlocker {
    pub action: String,
    pub reason: String,
    pub source_path: Option<String>,
}

pub(crate) fn issue_readiness_blockers(
    root: &Path,
    issues: &[SpecIssue],
) -> Vec<IssueReadinessBlocker> {
    let projection_index = agentflow_projection::load_issue_status_index(root).ok();
    dependency_blockers(issues, projection_index.as_ref())
}

pub(crate) fn issue_has_readiness_blocker(
    issue_path: &str,
    blockers: &[crate::model::WorkflowBlockedAction],
) -> bool {
    if issue_path.trim().is_empty() {
        return false;
    }
    blockers.iter().any(|blocker| {
        blocker.action.as_str() == "dependency-ready"
            && blocker.source_path.as_deref() == Some(issue_path)
    })
}

fn dependency_blockers(
    issues: &[SpecIssue],
    projection_index: Option<&IssueStatusIndex>,
) -> Vec<IssueReadinessBlocker> {
    let status_by_id = issues
        .iter()
        .map(|issue| {
            (
                issue.issue_id.clone(),
                projected_status(issue, projection_index).to_string(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let mut blockers = Vec::new();

    for issue in issues {
        if matches!(projected_status(issue, projection_index), "done" | "cancel") {
            continue;
        }
        for dependency_id in &issue.blocked_by {
            match status_by_id.get(dependency_id) {
                Some(status) if status == "done" => {}
                Some(_) => blockers.push(IssueReadinessBlocker {
                    action: "dependency-ready".to_string(),
                    reason: format!(
                        "任务 {} 的前置依赖 {} 还没有完成，不能开始执行。",
                        issue.issue_id, dependency_id
                    ),
                    source_path: Some(issue.system.path.clone()),
                }),
                None => blockers.push(IssueReadinessBlocker {
                    action: "dependency-ready".to_string(),
                    reason: format!(
                        "任务 {} 的前置依赖 {} 不存在，不能开始执行。",
                        issue.issue_id, dependency_id
                    ),
                    source_path: Some(issue.system.path.clone()),
                }),
            }
        }
    }

    blockers
}

fn projected_status<'a>(
    issue: &'a SpecIssue,
    projection_index: Option<&'a IssueStatusIndex>,
) -> &'a str {
    projection_index
        .and_then(|index| {
            index
                .issues
                .iter()
                .find(|entry| entry.issue_id == issue.issue_id)
        })
        .map(|entry| entry.current_state.as_str())
        .unwrap_or_else(|| issue.status.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;
    use agentflow_spec::{SpecIssueDraft, SpecIssueStatus};
    use tempfile::tempdir;

    fn issue(root: &Path, issue_id: &str, status: SpecIssueStatus) -> SpecIssue {
        let requirement = root.join(format!("docs/requirements/{issue_id}.md"));
        std::fs::create_dir_all(requirement.parent().unwrap()).unwrap();
        std::fs::write(&requirement, format!("# {issue_id}\n")).unwrap();
        let mut draft = SpecIssueDraft::new(issue_id);
        draft.title = Some(issue_id.to_string());
        let mut issue = agentflow_spec::issue_from_requirement(root, &requirement, draft).unwrap();
        issue.status = status;
        issue
    }

    #[test]
    fn dependency_gate_blocks_until_blocked_by_issue_is_done() {
        let dir = tempdir().unwrap();
        let ready = issue(dir.path(), "AF-001", SpecIssueStatus::Todo);
        let mut blocked = issue(dir.path(), "AF-002", SpecIssueStatus::Todo);
        blocked.blocked_by = vec!["AF-001".to_string()];

        let blockers = dependency_blockers(&[ready, blocked], None);

        assert!(blockers.iter().any(|blocker| {
            blocker.action == "dependency-ready"
                && blocker.source_path.as_deref() == Some(".agentflow/spec/issues/AF-002.json")
        }));
    }

    #[test]
    fn dependency_gate_allows_when_blocked_by_issue_is_done() {
        let dir = tempdir().unwrap();
        let done = issue(dir.path(), "AF-001", SpecIssueStatus::Done);
        let mut blocked = issue(dir.path(), "AF-002", SpecIssueStatus::Todo);
        blocked.blocked_by = vec!["AF-001".to_string()];

        let blockers = dependency_blockers(&[done, blocked], None);

        assert!(blockers.is_empty());
    }
}
